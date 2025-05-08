use crate::models::messages::{Message, MessageError};
use crate::protocols::mcp::error::McpError;
use crate::protocols::mcp::message::{McpMessage, McpMessagePayload, McpResponseMessage};
use crate::protocols::mcp::types::{McpCompletionRequest, McpMessageRole, McpMessageType};
use crate::protocols::mcp::websocket::WebSocketClient;
use crate::protocols::mcp::McpConfig;
use crate::protocols::ConnectionStatus;
use log::{debug, error, info, warn};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;

/// Struct responsible for handling MCP communication
pub struct McpClient {
    /// WebSocket client for communication
    websocket: Arc<WebSocketClient>,
    
    /// Client configuration
    config: McpConfig,
    
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Request tracking map (request ID -> response channel)
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Result<McpResponseMessage, McpError>>>>>,
    
    /// Channel for receiving unsolicited messages (events, etc.)
    event_tx: UnboundedSender<McpMessage>,
    
    /// Message streaming channel (streaming ID -> message channel)
    streaming_channels: Arc<Mutex<HashMap<String, Sender<Result<McpMessage, McpError>>>>>,
}

/// Implementation for McpClient
impl McpClient {
    /// Create a new MCP client with the given configuration
    pub fn new(config: McpConfig) -> Self {
        let (event_tx, _) = mpsc::unbounded_channel();
        
        Self {
            websocket: Arc::new(WebSocketClient::new(&config.url)),
            config,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            streaming_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get a receiver for unsolicited events
    pub fn get_event_receiver(&self) -> UnboundedReceiver<McpMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_tx = tx;
        rx
    }
    
    /// Connect to the MCP server
    pub async fn connect(&self) -> Result<(), McpError> {
        let mut status = self.status.write().unwrap();
        *status = ConnectionStatus::Connecting;
        drop(status);
        
        // Connect to WebSocket
        let ws_client = self.websocket.clone();
        let pending_requests = self.pending_requests.clone();
        let streaming_channels = self.streaming_channels.clone();
        let status_clone = self.status.clone();
        let event_tx = self.event_tx.clone();
        
        match ws_client.connect().await {
            Ok(_) => {
                let mut status = self.status.write().unwrap();
                *status = ConnectionStatus::Connected;
                
                // Start message handler
                tokio::spawn(async move {
                    Self::handle_incoming_messages(
                        ws_client.clone(),
                        pending_requests,
                        streaming_channels,
                        status_clone,
                        event_tx,
                    )
                    .await;
                });
                
                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().unwrap();
                *status = ConnectionStatus::ConnectionError(e.to_string());
                Err(McpError::ConnectionError(e))
            }
        }
    }
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> Result<(), McpError> {
        // Close WebSocket connection
        self.websocket.disconnect().await?;
        
        // Update status
        let mut status = self.status.write().unwrap();
        *status = ConnectionStatus::Disconnected;
        
        // Cancel all pending requests
        let mut pending = self.pending_requests.lock().unwrap();
        for (_, sender) in pending.drain() {
            let _ = sender.send(Err(McpError::ConnectionClosed));
        }
        
        // Cancel all streaming channels
        let mut streaming = self.streaming_channels.lock().unwrap();
        for (_, sender) in streaming.drain() {
            let _ = sender.send(Err(McpError::ConnectionClosed)).await;
        }
        
        Ok(())
    }
    
    /// Get the current connection status
    pub fn status(&self) -> ConnectionStatus {
        self.status.read().unwrap().clone()
    }
    
    /// Send a message and wait for response
    pub async fn send(&self, message: Message) -> Result<Message, MessageError> {
        // Convert to MCP message format
        let mcp_message = Self::convert_to_mcp_message(message)?;
        
        // Setup response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(mcp_message.id.clone(), tx);
        }
        
        // Send message
        self.websocket
            .send(serde_json::to_string(&mcp_message)?)
            .await
            .map_err(|e| MessageError::NetworkError(e.to_string()))?;
        
        // Wait for response with timeout
        match timeout(Duration::from_secs(60), rx).await {
            Ok(result) => match result {
                Ok(response) => match response {
                    Ok(mcp_response) => Self::convert_from_mcp_response(mcp_response),
                    Err(e) => Err(MessageError::ProtocolError(e.to_string())),
                },
                Err(_) => Err(MessageError::Unknown("Response channel closed".to_string())),
            },
            Err(_) => Err(MessageError::Timeout(Duration::from_secs(60))),
        }
    }
    
    /// Start a streaming completion
    pub async fn stream(
        &self,
        message: Message,
    ) -> Result<Receiver<Result<Message, MessageError>>, MessageError> {
        // Create a streaming ID
        let streaming_id = Uuid::new_v4().to_string();
        
        // Convert to MCP format
        let mut mcp_message = Self::convert_to_mcp_message(message)?;
        
        // Update payload for streaming
        if let McpMessagePayload::CompletionRequest(ref mut req) = mcp_message.payload {
            req.stream = true;
            req.streaming_id = Some(streaming_id.clone());
        } else {
            return Err(MessageError::ProtocolError(
                "Invalid message type for streaming".to_string(),
            ));
        }
        
        // Create streaming channel
        let (tx, rx) = mpsc::channel(32);
        {
            let mut streaming = self.streaming_channels.lock().unwrap();
            streaming.insert(streaming_id.clone(), tx);
        }
        
        // Setup response channel for initial acknowledgment
        let (ack_tx, ack_rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(mcp_message.id.clone(), ack_tx);
        }
        
        // Send message
        self.websocket
            .send(serde_json::to_string(&mcp_message)?)
            .await
            .map_err(|e| MessageError::NetworkError(e.to_string()))?;
        
        // Wait for initial acknowledgment
        match timeout(Duration::from_secs(10), ack_rx).await {
            Ok(result) => match result {
                Ok(response) => {
                    match response {
                        Ok(_) => {
                            // Create message adapter channel
                            let (adapter_tx, adapter_rx) = mpsc::channel(32);
                            
                            // Spawn adapter task to convert MCP messages to app messages
                            let streaming_clone = self.streaming_channels.clone();
                            let streaming_id_clone = streaming_id.clone();
                            
                            tokio::spawn(async move {
                                let streaming = streaming_clone.lock().unwrap();
                                if let Some(rx_channel) = streaming.get(&streaming_id_clone) {
                                    // TODO: Implement streaming message adapter
                                    // This would convert the MCP protocol streaming messages
                                    // into the application's Message format
                                }
                            });
                            
                            Ok(adapter_rx)
                        }
                        Err(e) => Err(MessageError::ProtocolError(e.to_string())),
                    }
                }
                Err(_) => Err(MessageError::Unknown("Response channel closed".to_string())),
            },
            Err(_) => Err(MessageError::Timeout(Duration::from_secs(10))),
        }
    }
    
    /// Cancel a streaming request
    pub async fn cancel_streaming(&self, streaming_id: &str) -> Result<(), MessageError> {
        // Remove streaming channel
        {
            let mut streaming = self.streaming_channels.lock().unwrap();
            streaming.remove(streaming_id);
        }
        
        // Send cancel message
        let cancel_message = McpMessage {
            id: Uuid::new_v4().to_string(),
            version: "v1".to_string(),
            type_: McpMessageType::CancelStream,
            payload: McpMessagePayload::CancelStream {
                streaming_id: streaming_id.to_string(),
            },
        };
        
        self.websocket
            .send(serde_json::to_string(&cancel_message)?)
            .await
            .map_err(|e| MessageError::NetworkError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Handle incoming messages from the WebSocket
    async fn handle_incoming_messages(
        websocket: Arc<WebSocketClient>,
        pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Result<McpResponseMessage, McpError>>>>>,
        streaming_channels: Arc<Mutex<HashMap<String, Sender<Result<McpMessage, McpError>>>>>,
        status: Arc<RwLock<ConnectionStatus>>,
        event_tx: UnboundedSender<McpMessage>,
    ) {
        loop {
            match websocket.receive().await {
                Ok(message) => {
                    match serde_json::from_str::<McpMessage>(&message) {
                        Ok(mcp_message) => {
                            // Handle based on message type
                            match mcp_message.type_ {
                                McpMessageType::CompletionResponse => {
                                    // Check if this is part of a pending request
                                    let mut pending = pending_requests.lock().unwrap();
                                    if let Some(sender) = pending.remove(&mcp_message.id) {
                                        let _ = sender.send(Ok(McpResponseMessage::Completion(mcp_message)));
                                    } else {
                                        // This could be an unsolicited message or event
                                        let _ = event_tx.send(mcp_message);
                                    }
                                }
                                McpMessageType::StreamingMessage => {
                                    // Extract streaming ID from payload
                                    if let McpMessagePayload::StreamingMessage { streaming_id, .. } = 
                                        &mcp_message.payload 
                                    {
                                        let streaming = streaming_channels.lock().unwrap();
                                        if let Some(sender) = streaming.get(streaming_id) {
                                            let _ = sender.send(Ok(mcp_message.clone())).await;
                                        }
                                    }
                                }
                                McpMessageType::StreamingEnd => {
                                    // Extract streaming ID from payload
                                    if let McpMessagePayload::StreamingEnd { streaming_id } = 
                                        &mcp_message.payload 
                                    {
                                        let mut streaming = streaming_channels.lock().unwrap();
                                        if let Some(sender) = streaming.remove(streaming_id) {
                                            let _ = sender.send(Ok(mcp_message.clone())).await;
                                        }
                                    }
                                }
                                McpMessageType::Error => {
                                    // Extract request_id from payload
                                    if let McpMessagePayload::Error { request_id, .. } = 
                                        &mcp_message.payload 
                                    {
                                        let mut pending = pending_requests.lock().unwrap();
                                        if let Some(sender) = pending.remove(request_id) {
                                            if let McpMessagePayload::Error { code, message, .. } = 
                                                mcp_message.payload.clone() 
                                            {
                                                let _ = sender.send(Err(
                                                    McpError::ProtocolError(format!("{}: {}", code, message))
                                                ));
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    // Handle other message types or unsolicited messages
                                    let _ = event_tx.send(mcp_message);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse MCP message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    
                    // Update connection status
                    let mut status_guard = status.write().unwrap();
                    *status_guard = ConnectionStatus::ConnectionError(e.to_string());
                    
                    // Cancel all pending requests
                    let mut pending = pending_requests.lock().unwrap();
                    for (_, sender) in pending.drain() {
                        let _ = sender.send(Err(McpError::ConnectionClosed));
                    }
                    
                    // Cancel all streaming channels
                    let mut streaming = streaming_channels.lock().unwrap();
                    for (_, sender) in streaming.drain() {
                        let _ = sender.send(Err(McpError::ConnectionClosed)).await;
                    }
                    
                    // Break the loop, ending the handler
                    break;
                }
            }
        }
    }
    
    /// Convert application Message to MCP format
    fn convert_to_mcp_message(message: Message) -> Result<McpMessage, MessageError> {
        // Convert role
        let role = match message.role {
            crate::models::messages::MessageRole::User => McpMessageRole::User,
            crate::models::messages::MessageRole::Assistant => McpMessageRole::Assistant,
            crate::models::messages::MessageRole::System => McpMessageRole::System,
            crate::models::messages::MessageRole::Tool => McpMessageRole::Tool,
        };
        
        // Build MCP message
        let mut parts = Vec::new();
        for part in &message.content.parts {
            match part {
                crate::models::messages::ContentType::Text { text } => {
                    parts.push(json!({
                        "type": "text",
                        "text": text
                    }));
                }
                crate::models::messages::ContentType::Image { url, media_type } => {
                    parts.push(json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": url.trim_start_matches("data:").to_string()
                        }
                    }));
                }
                crate::models::messages::ContentType::ToolCall { id, name, arguments } => {
                    parts.push(json!({
                        "type": "tool_call",
                        "id": id,
                        "name": name,
                        "arguments": arguments
                    }));
                }
                crate::models::messages::ContentType::ToolResult { tool_call_id, result } => {
                    parts.push(json!({
                        "type": "tool_result",
                        "tool_call_id": tool_call_id,
                        "result": result
                    }));
                }
            }
        }
        
        // Create completion request
        let completion_request = McpCompletionRequest {
            model: "claude-3-opus-20240229".to_string(), // TODO: Get from config
            messages: vec![json!({
                "role": role.to_string(),
                "content": parts
            })],
            max_tokens: 4000,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            stream: false,
            stop_sequences: Vec::new(),
            system_prompt: None,
            streaming_id: None,
        };
        
        // Build final MCP message
        let mcp_message = McpMessage {
            id: message.id.clone(),
            version: "v1".to_string(),
            type_: McpMessageType::CompletionRequest,
            payload: McpMessagePayload::CompletionRequest(completion_request),
        };
        
        Ok(mcp_message)
    }
    
    /// Convert MCP response to application Message
    fn convert_from_mcp_response(
        response: McpResponseMessage,
    ) -> Result<Message, MessageError> {
        match response {
            McpResponseMessage::Completion(completion) => {
                if let McpMessagePayload::CompletionResponse { response } = completion.payload {
                    // Extract first content part - text
                    // In a real implementation, we would handle all content types properly
                    if let Some(content) = response.get("content") {
                        if let Some(text) = content.as_str() {
                            // Create a message
                            let message = Message {
                                id: Uuid::new_v4().to_string(),
                                role: crate::models::messages::MessageRole::Assistant,
                                content: crate::models::messages::MessageContent {
                                    parts: vec![crate::models::messages::ContentType::Text {
                                        text: text.to_string(),
                                    }],
                                },
                                metadata: None,
                                created_at: SystemTime::now(),
                            };
                            
                            return Ok(message);
                        }
                    }
                    
                    return Err(MessageError::SerializationError(
                        "Invalid completion response format".to_string(),
                    ));
                }
                
                Err(MessageError::ProtocolError(
                    "Invalid response type".to_string(),
                ))
            }
            _ => Err(MessageError::ProtocolError(
                "Unexpected response type".to_string(),
            )),
        }
    }
}
