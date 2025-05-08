use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use uuid::Uuid;

use super::{ConnectionStatus, ProtocolConfig, ProtocolHandler, WebSocketClient, WebSocketConfig};
use crate::error::{McpError, McpResult};
use crate::models::{ContentType, Message, MessageContent, MessageRole};

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpMessageType {
    AuthRequest,
    AuthResponse,
    CompletionRequest,
    CompletionResponse,
    StreamingStart,
    StreamingMessage,
    StreamingEnd,
    CancelStream,
    Ping,
    Pong,
    Error,
}

/// MCP message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    /// Message ID
    pub id: String,
    
    /// Protocol version
    pub version: String,
    
    /// Message type
    #[serde(rename = "type")]
    pub message_type: McpMessageType,
    
    /// Message payload
    pub payload: serde_json::Value,
}

/// MCP protocol configuration
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// API key for authentication
    pub api_key: String,
    
    /// Server URL
    pub url: String,
    
    /// Protocol version
    pub version: String,
    
    /// Default model ID
    pub model: String,
}

/// MCP client
pub struct McpClient {
    /// Configuration
    config: McpConfig,
    
    /// WebSocket client
    ws_client: Arc<WebSocketClient>,
    
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Active streaming sessions
    streaming_sessions: Arc<Mutex<HashMap<String, mpsc::Sender<Message>>>>,
}

/// MCP protocol handler implementation
pub struct McpProtocolHandler {
    /// MCP client
    client: Arc<McpClient>,
    
    /// Shared connection status
    status: Arc<RwLock<ConnectionStatus>>,
}

impl McpMessage {
    /// Create a new MCP message
    pub fn new(message_type: McpMessageType, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            version: "v1".to_string(),
            message_type,
            payload,
        }
    }
    
    /// Create an authentication request message
    pub fn auth_request(api_key: &str) -> Self {
        Self::new(
            McpMessageType::AuthRequest,
            serde_json::json!({
                "api_key": api_key,
            }),
        )
    }
    
    /// Create a completion request message
    pub fn completion_request(
        model: &str,
        messages: &[Message],
        max_tokens: u32,
        temperature: f32,
        stream: bool,
    ) -> Self {
        // Convert messages to MCP format
        let mcp_messages = messages
            .iter()
            .map(|msg| {
                let content = msg.content.parts.iter().map(|part| {
                    match part {
                        ContentType::Text { text } => {
                            serde_json::json!({
                                "type": "text",
                                "text": text
                            })
                        }
                        ContentType::Image { url, alt_text } => {
                            serde_json::json!({
                                "type": "image",
                                "source": {
                                    "type": "url",
                                    "url": url
                                },
                                "alt_text": alt_text
                            })
                        }
                        _ => serde_json::json!(null),
                    }
                }).collect::<Vec<_>>();
                
                serde_json::json!({
                    "role": match msg.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                    },
                    "content": content
                })
            })
            .collect::<Vec<_>>();
        
        Self::new(
            McpMessageType::CompletionRequest,
            serde_json::json!({
                "model": model,
                "messages": mcp_messages,
                "max_tokens": max_tokens,
                "temperature": temperature,
                "stream": stream,
            }),
        )
    }
    
    /// Create a cancel stream message
    pub fn cancel_stream(stream_id: &str) -> Self {
        Self::new(
            McpMessageType::CancelStream,
            serde_json::json!({
                "stream_id": stream_id,
            }),
        )
    }
    
    /// Create a ping message
    pub fn ping() -> Self {
        Self::new(McpMessageType::Ping, serde_json::json!({}))
    }
}

impl McpConfig {
    /// Create a new MCP configuration with an API key
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key,
            url: "wss://api.anthropic.com/v1/messages".to_string(),
            version: "v1".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
        }
    }
    
    /// Set the server URL
    pub fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
    
    /// Set the protocol version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }
    
    /// Set the default model
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(config: McpConfig) -> Self {
        // Create websocket configuration
        let ws_config = WebSocketConfig {
            url: config.url.clone(),
            headers: vec![
                ("X-API-Key".to_string(), config.api_key.clone()),
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ],
            ..Default::default()
        };
        
        // Create websocket client
        let ws_client = Arc::new(WebSocketClient::new(ws_config));
        
        // Create MCP client
        Self {
            config,
            ws_client,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            streaming_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get the current connection status
    pub fn connection_status(&self) -> ConnectionStatus {
        self.ws_client.status()
    }
    
    /// Connect to the MCP server
    pub async fn connect(&self) -> McpResult<()> {
        // Connect WebSocket
        self.ws_client.connect().await?;
        
        // Send authentication message
        let auth_message = McpMessage::auth_request(&self.config.api_key);
        self.send_message(&auth_message).await?;
        
        // Wait for authentication response
        let response = self.receive_message().await?;
        
        if response.message_type == McpMessageType::AuthResponse {
            // Check if authentication was successful
            if let Some(success) = response.payload.get("success") {
                if success.as_bool().unwrap_or(false) {
                    *self.status.write().await = ConnectionStatus::Connected;
                    Ok(())
                } else {
                    *self.status.write().await = ConnectionStatus::AuthFailed;
                    Err(McpError::Authentication("Authentication failed".to_string()))
                }
            } else {
                *self.status.write().await = ConnectionStatus::Error("Invalid auth response".to_string());
                Err(McpError::Protocol("Invalid authentication response".to_string()))
            }
        } else if response.message_type == McpMessageType::Error {
            // Authentication error
            *self.status.write().await = ConnectionStatus::AuthFailed;
            Err(McpError::Authentication(
                response
                    .payload
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Authentication failed")
                    .to_string(),
            ))
        } else {
            // Unexpected response
            *self.status.write().await = ConnectionStatus::Error("Unexpected response".to_string());
            Err(McpError::Protocol("Unexpected response type".to_string()))
        }
    }
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> McpResult<()> {
        self.ws_client.disconnect().await?;
        *self.status.write().await = ConnectionStatus::Disconnected;
        Ok(())
    }
    
    /// Send an MCP message
    pub async fn send_message(&self, message: &McpMessage) -> McpResult<()> {
        // Serialize message
        let json = serde_json::to_string(message)
            .map_err(|e| McpError::Serialization(e))?;
            
        // Send via websocket
        self.ws_client
            .send(WsMessage::Text(json))
            .await
            .map_err(|e| McpError::Protocol(format!("Failed to send message: {}", e)))
    }
    
    /// Receive an MCP message with timeout
    pub async fn receive_message(&self) -> McpResult<McpMessage> {
        // Receive from websocket with timeout
        let message = self
            .ws_client
            .receive(Duration::from_secs(60))
            .await
            .map_err(|e| McpError::Protocol(format!("Failed to receive message: {}", e)))?;
            
        // Parse message
        if let WsMessage::Text(text) = message {
            let mcp_message: McpMessage = serde_json::from_str(&text)
                .map_err(|e| McpError::Serialization(e))?;
                
            Ok(mcp_message)
        } else {
            Err(McpError::Protocol("Unexpected message type".to_string()))
        }
    }
    
    /// Send a completion request
    pub async fn send_completion(
        &self,
        model: &str,
        messages: &[Message],
        max_tokens: u32,
        temperature: f32,
    ) -> McpResult<Message> {
        // Check if connected
        if !matches!(self.connection_status(), ConnectionStatus::Connected) {
            return Err(McpError::Connection("Not connected".to_string()));
        }
        
        // Create completion request
        let request = McpMessage::completion_request(
            model,
            messages,
            max_tokens,
            temperature,
            false, // No streaming
        );
        
        // Send request
        self.send_message(&request).await?;
        
        // Wait for response
        let response = self.receive_message().await?;
        
        if response.message_type == McpMessageType::CompletionResponse {
            // Parse response
            let content = response
                .payload
                .get("content")
                .ok_or_else(|| McpError::Protocol("Missing content in response".to_string()))?;
                
            // Convert to Message format
            let message = Message {
                id: response.id,
                role: MessageRole::Assistant,
                content: MessageContent {
                    parts: vec![ContentType::Text {
                        text: content
                            .as_str()
                            .ok_or_else(|| {
                                McpError::Protocol("Invalid content type in response".to_string())
                            })?
                            .to_string(),
                    }],
                },
                metadata: None,
                created_at: std::time::SystemTime::now(),
            };
            
            Ok(message)
        } else if response.message_type == McpMessageType::Error {
            // Error response
            Err(McpError::Protocol(
                response
                    .payload
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string(),
            ))
        } else {
            // Unexpected response
            Err(McpError::Protocol("Unexpected response type".to_string()))
        }
    }
    
    /// Start a streaming completion request
    pub async fn stream_completion(
        &self,
        model: &str,
        messages: &[Message],
        max_tokens: u32,
        temperature: f32,
    ) -> McpResult<mpsc::Receiver<Message>> {
        // Check if connected
        if !matches!(self.connection_status(), ConnectionStatus::Connected) {
            return Err(McpError::Connection("Not connected".to_string()));
        }
        
        // Create completion request
        let request = McpMessage::completion_request(
            model,
            messages,
            max_tokens,
            temperature,
            true, // Enable streaming
        );
        
        // Create channel for streaming
        let (tx, rx) = mpsc::channel::<Message>(32);
        
        // Store streaming session
        {
            let mut sessions = self.streaming_sessions.lock().await;
            sessions.insert(request.id.clone(), tx.clone());
        }
        
        // Send request
        self.send_message(&request).await?;
        
        // Start streaming task
        let client_clone = Arc::new(self.clone());
        let request_id = request.id.clone();
        
        tokio::spawn(async move {
            // Process streaming messages
            loop {
                match client_clone.receive_message().await {
                    Ok(message) => {
                        match message.message_type {
                            McpMessageType::StreamingStart => {
                                // Stream started - just log it
                                debug!("Streaming started for {}", request_id);
                            }
                            McpMessageType::StreamingMessage => {
                                // Process streaming message
                                if let Some(content) = message.payload.get("content") {
                                    if let Some(text) = content.as_str() {
                                        // Create message
                                        let chunk = Message {
                                            id: request_id.clone(),
                                            role: MessageRole::Assistant,
                                            content: MessageContent {
                                                parts: vec![ContentType::Text {
                                                    text: text.to_string(),
                                                }],
                                            },
                                            metadata: None,
                                            created_at: std::time::SystemTime::now(),
                                        };
                                        
                                        // Send to receiver
                                        if tx.send(chunk).await.is_err() {
                                            // Receiver dropped, stop streaming
                                            break;
                                        }
                                    }
                                }
                            }
                            McpMessageType::StreamingEnd => {
                                // Stream ended
                                debug!("Streaming ended for {}", request_id);
                                break;
                            }
                            McpMessageType::Error => {
                                // Error occurred
                                error!(
                                    "Streaming error: {}",
                                    message
                                        .payload
                                        .get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Unknown error")
                                );
                                break;
                            }
                            _ => {
                                // Ignore other message types
                            }
                        }
                    }
                    Err(e) => {
                        // Error receiving message
                        error!("Error receiving streaming message: {}", e);
                        break;
                    }
                }
            }
            
            // Remove streaming session
            let client = client_clone.as_ref();
            let mut sessions = client.streaming_sessions.lock().await;
            sessions.remove(&request_id);
        });
        
        Ok(rx)
    }
    
    /// Cancel a streaming completion request
    pub async fn cancel_streaming(&self, stream_id: &str) -> McpResult<()> {
        // Check if connected
        if !matches!(self.connection_status(), ConnectionStatus::Connected) {
            return Err(McpError::Connection("Not connected".to_string()));
        }
        
        // Create cancel message
        let cancel = McpMessage::cancel_stream(stream_id);
        
        // Send cancel message
        self.send_message(&cancel).await?;
        
        // Remove streaming session
        let mut sessions = self.streaming_sessions.lock().await;
        sessions.remove(stream_id);
        
        Ok(())
    }
}

impl Clone for McpClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            ws_client: self.ws_client.clone(),
            status: self.status.clone(),
            streaming_sessions: self.streaming_sessions.clone(),
        }
    }
}

impl McpProtocolHandler {
    /// Create a new MCP protocol handler
    pub fn new(config: McpConfig) -> Self {
        let client = Arc::new(McpClient::new(config));
        Self {
            client: client.clone(),
            status: client.status.clone(),
        }
    }
}

#[async_trait]
impl ProtocolHandler for McpProtocolHandler {
    fn protocol_name(&self) -> &'static str {
        "Model Context Protocol"
    }
    
    fn connection_status(&self) -> ConnectionStatus {
        self.client.connection_status()
    }
    
    async fn connect(&self) -> McpResult<()> {
        self.client.connect().await
    }
    
    async fn disconnect(&self) -> McpResult<()> {
        self.client.disconnect().await
    }
    
    async fn send_message(&self, message: Message) -> McpResult<()> {
        // Get model from message metadata or use default
        let model = message
            .metadata
            .as_ref()
            .and_then(|m| m.get("model"))
            .and_then(|m| m.as_str())
            .unwrap_or(&self.client.config.model)
            .to_string();
        
        // Get conversation history from message metadata or create new
        let history = message
            .metadata
            .as_ref()
            .and_then(|m| m.get("history"))
            .and_then(|m| m.as_array())
            .map(|h| {
                h.iter()
                    .filter_map(|m| serde_json::from_value::<Message>(m.clone()).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![]);
        
        // Create message history including the new message
        let mut messages = history;
        messages.push(message.clone());
        
        // Send completion request
        let _response = self
            .client
            .send_completion(&model, &messages, 4096, 0.7)
            .await?;
        
        Ok(())
    }
    
    async fn receive_messages(&self) -> McpResult<Vec<Message>> {
        // This would normally process all pending messages
        // For now, just return an empty vector
        Ok(Vec::new())
    }
}

impl ProtocolConfig for McpConfig {
    fn validate(&self) -> McpResult<()> {
        if self.api_key.is_empty() {
            return Err(McpError::Config("API key is required".to_string()));
        }
        
        if self.url.is_empty() {
            return Err(McpError::Config("URL is required".to_string()));
        }
        
        if self.model.is_empty() {
            return Err(McpError::Config("Model ID is required".to_string()));
        }
        
        Ok(())
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            url: "wss://api.anthropic.com/v1/messages".to_string(),
            version: "v1".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
        }
    }
}
