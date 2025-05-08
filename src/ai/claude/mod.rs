mod api;
mod mcp;
mod streaming;

use self::api::{ClaudeApi, ClaudeApiClient, ClaudeResponse};
use self::mcp::ClaudeMcpClient;
use self::streaming::ClaudeStreamHandler;
use crate::ai::{ModelError, ModelProvider, ModelProviderConfig, ModelStatus, ProviderType};
use crate::models::messages::{ContentType, Message, MessageContent, MessageError, MessageRole};
use crate::models::Model;
use crate::services::auth::get_auth_service;
use crate::utils::config;
use crate::utils::events::{events, get_event_system};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Claude AI provider
pub struct ClaudeProvider {
    /// Provider configuration
    config: ModelProviderConfig,
    
    /// Claude API client
    api_client: ClaudeApiClient,
    
    /// Claude MCP client
    mcp_client: Option<ClaudeMcpClient>,
    
    /// Available models cache
    models: Arc<RwLock<Vec<Model>>>,
    
    /// Model status cache
    model_status: Arc<RwLock<HashMap<String, ModelStatus>>>,
    
    /// Active streaming sessions
    active_streams: Arc<Mutex<HashMap<String, ClaudeStreamHandler>>>,
}

impl ClaudeProvider {
    /// Create a new Claude provider
    pub fn new() -> Result<Self, ModelError> {
        // Load configuration
        let config = config::get_config();
        let config_guard = config.lock().unwrap();
        
        let api_key = config_guard
            .get_string("api.key")
            .unwrap_or_else(|| String::new());
        
        if api_key.is_empty() {
            warn!("Claude API key not found in configuration");
            return Err(ModelError::AuthError);
        }
        
        let base_url = config_guard
            .get_string("api.base_url")
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
        
        let provider_config = ModelProviderConfig {
            provider_type: ProviderType::Claude,
            name: "Claude".to_string(),
            base_url,
            api_key: api_key.clone(),
            organization_id: None, // Could be loaded from config if needed
            timeout: Duration::from_secs(120),
            default_model: "claude-3-opus-20240229".to_string(),
            fallback_model: Some("claude-3-haiku-20240307".to_string()),
            enable_mcp: true,
            enable_streaming: true,
            settings: serde_json::Map::new(),
        };
        
        // Create API client
        let api_client = ClaudeApiClient::new(api_key, &base_url);
        
        // Create MCP client if enabled
        let mcp_client = if provider_config.enable_mcp {
            match ClaudeMcpClient::new(&provider_config) {
                Ok(client) => Some(client),
                Err(e) => {
                    warn!("Failed to create Claude MCP client: {:?}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Create default models
        let default_models = vec![
            Model {
                id: "claude-3-opus-20240229".to_string(),
                provider: "anthropic".to_string(),
                name: "Claude 3 Opus".to_string(),
                version: "20240229".to_string(),
                capabilities: crate::models::ModelCapabilities {
                    vision: true,
                    max_context_length: 200_000,
                    functions: true,
                    streaming: true,
                },
            },
            Model {
                id: "claude-3-sonnet-20240229".to_string(),
                provider: "anthropic".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                version: "20240229".to_string(),
                capabilities: crate::models::ModelCapabilities {
                    vision: true,
                    max_context_length: 180_000,
                    functions: true,
                    streaming: true,
                },
            },
            Model {
                id: "claude-3-haiku-20240307".to_string(),
                provider: "anthropic".to_string(),
                name: "Claude 3 Haiku".to_string(),
                version: "20240307".to_string(),
                capabilities: crate::models::ModelCapabilities {
                    vision: true,
                    max_context_length: 150_000,
                    functions: true,
                    streaming: true,
                },
            },
        ];
        
        Ok(Self {
            config: provider_config,
            api_client,
            mcp_client,
            models: Arc::new(RwLock::new(default_models)),
            model_status: Arc::new(RwLock::new(HashMap::new())),
            active_streams: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Convert a Message to Claude API format
    fn convert_to_claude_format(&self, message: &Message) -> serde_json::Value {
        // Convert role
        let role = match message.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        };
        
        // Build content
        let mut contents = Vec::new();
        for part in &message.content.parts {
            match part {
                ContentType::Text { text } => {
                    contents.push(serde_json::json!({
                        "type": "text",
                        "text": text
                    }));
                }
                ContentType::Image { url, media_type } => {
                    contents.push(serde_json::json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": url.trim_start_matches("data:").to_string()
                        }
                    }));
                }
                ContentType::ToolCall { id, name, arguments } => {
                    contents.push(serde_json::json!({
                        "type": "tool_call",
                        "id": id,
                        "name": name,
                        "arguments": arguments
                    }));
                }
                ContentType::ToolResult { tool_call_id, result } => {
                    contents.push(serde_json::json!({
                        "type": "tool_result",
                        "tool_call_id": tool_call_id,
                        "result": result
                    }));
                }
            }
        }
        
        serde_json::json!({
            "role": role,
            "content": contents
        })
    }
    
    /// Convert Claude API response to Message
    fn convert_from_claude_response(&self, response: &ClaudeResponse) -> Message {
        // Extract text content from response
        let text = response.content.iter()
            .filter_map(|content| {
                if let Some(content_type) = content.get("type") {
                    if content_type == "text" {
                        content.get("text").and_then(|text| text.as_str())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>()
            .join("");
        
        Message {
            id: response.id.clone(),
            role: MessageRole::Assistant,
            content: MessageContent {
                parts: vec![ContentType::Text { text: text.to_string() }],
            },
            metadata: Some(HashMap::from([
                ("model".to_string(), serde_json::to_value(&response.model).unwrap()),
                ("stop_reason".to_string(), serde_json::to_value(&response.stop_reason).unwrap()),
                ("usage".to_string(), serde_json::to_value(&response.usage).unwrap()),
            ])),
            created_at: SystemTime::now(),
        }
    }
}

#[async_trait]
impl ModelProvider for ClaudeProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Claude
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModelProviderConfig {
        &self.config
    }
    
    async fn available_models(&self) -> Result<Vec<Model>, ModelError> {
        // Return cached models
        let models = self.models.read().unwrap().clone();
        
        // In a real implementation, we would fetch models from the API
        // For now, return the default models
        
        Ok(models)
    }
    
    async fn is_available(&self, model_id: &str) -> bool {
        // Check if model exists in available models
        let models = self.models.read().unwrap();
        models.iter().any(|m| m.id == model_id)
    }
    
    async fn model_status(&self, model_id: &str) -> ModelStatus {
        // Check cached status
        {
            let statuses = self.model_status.read().unwrap();
            if let Some(status) = statuses.get(model_id) {
                return *status;
            }
        }
        
        // Check if model exists
        if self.is_available(model_id).await {
            // Update cache
            {
                let mut statuses = self.model_status.write().unwrap();
                statuses.insert(model_id.to_string(), ModelStatus::Available);
            }
            
            ModelStatus::Available
        } else {
            // Update cache
            {
                let mut statuses = self.model_status.write().unwrap();
                statuses.insert(model_id.to_string(), ModelStatus::Unavailable);
            }
            
            ModelStatus::Unavailable
        }
    }
    
    async fn complete(&self, model_id: &str, message: Message) -> Result<Message, MessageError> {
        // Check if model is available
        if !self.is_available(model_id).await {
            return Err(MessageError::ProtocolError(format!(
                "Model {} is not available",
                model_id
            )));
        }
        
        // If MCP client is available and enabled, use it
        if let Some(mcp_client) = &self.mcp_client {
            if self.config.enable_mcp {
                return mcp_client.complete(model_id, message).await;
            }
        }
        
        // Otherwise use REST API
        let claude_message = self.convert_to_claude_format(&message);
        
        // Create request body
        let request_body = serde_json::json!({
            "model": model_id,
            "messages": [claude_message],
            "max_tokens": 4096,
            "temperature": 0.7,
            "system": "You are Claude, an AI assistant created by Anthropic. You are helpful, harmless, and honest."
        });
        
        // Send request
        match self.api_client.create_message(&request_body).await {
            Ok(response) => Ok(self.convert_from_claude_response(&response)),
            Err(e) => Err(MessageError::NetworkError(e.to_string())),
        }
    }
    
    async fn stream(
        &self,
        model_id: &str,
        message: Message,
    ) -> Result<mpsc::Receiver<Result<Message, MessageError>>, MessageError> {
        // Check if model is available
        if !self.is_available(model_id).await {
            return Err(MessageError::ProtocolError(format!(
                "Model {} is not available",
                model_id
            )));
        }
        
        // Check if streaming is enabled
        if !self.config.enable_streaming {
            return Err(MessageError::ProtocolError(
                "Streaming is not enabled".to_string(),
            ));
        }
        
        // If MCP client is available and enabled, use it
        if let Some(mcp_client) = &self.mcp_client {
            if self.config.enable_mcp {
                return mcp_client.stream(model_id, message).await;
            }
        }
        
        // Otherwise use REST API with streaming
        let claude_message = self.convert_to_claude_format(&message);
        
        // Create request body
        let request_body = serde_json::json!({
            "model": model_id,
            "messages": [claude_message],
            "max_tokens": 4096,
            "temperature": 0.7,
            "system": "You are Claude, an AI assistant created by Anthropic. You are helpful, harmless, and honest.",
            "stream": true
        });
        
        // Create stream handler
        let stream_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::channel(32);
        
        let handler = ClaudeStreamHandler::new(
            stream_id.clone(),
            tx.clone(),
            message.id.clone(),
        );
        
        // Store stream handler
        {
            let mut streams = self.active_streams.lock().unwrap();
            streams.insert(stream_id.clone(), handler.clone());
        }
        
        // Start streaming in separate task
        let api_client = self.api_client.clone();
        let self_clone = self.clone();
        let message_id = message.id.clone();
        
        tokio::spawn(async move {
            match api_client.create_message_stream(&request_body).await {
                Ok(mut stream) => {
                    handler.handle_stream(&mut stream, &message_id).await;
                    
                    // Remove stream handler when done
                    let mut streams = self_clone.active_streams.lock().unwrap();
                    streams.remove(&stream_id);
                }
                Err(e) => {
                    // Send error and remove stream handler
                    let _ = tx
                        .send(Err(MessageError::NetworkError(e.to_string())))
                        .await;
                    
                    let mut streams = self_clone.active_streams.lock().unwrap();
                    streams.remove(&stream_id);
                }
            }
        });
        
        Ok(rx)
    }
    
    async fn cancel_stream(&self, stream_id: &str) -> Result<(), MessageError> {
        // Find stream handler
        let mut handler_opt = None;
        
        {
            let mut streams = self.active_streams.lock().unwrap();
            handler_opt = streams.remove(stream_id);
        }
        
        if let Some(handler) = handler_opt {
            // Cancel the stream
            handler.cancel().await;
            Ok(())
        } else {
            Err(MessageError::Unknown(format!(
                "Stream {} not found",
                stream_id
            )))
        }
    }
}

impl Clone for ClaudeProvider {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            api_client: self.api_client.clone(),
            mcp_client: self.mcp_client.clone(),
            models: self.models.clone(),
            model_status: self.model_status.clone(),
            active_streams: self.active_streams.clone(),
        }
    }
}
