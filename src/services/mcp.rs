use crate::models::messages::{Message, MessageError};
use crate::models::{Conversation, Model};
use crate::protocols::mcp::{McpClient, McpConfig, McpError, McpProtocolHandler};
use crate::protocols::{ConnectionStatus, ProtocolHandler};
use crate::utils::config::Config;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use uuid::Uuid;

/// Service for interacting with the MCP protocol
pub struct McpService {
    /// MCP client
    client: Arc<McpClient>,
    
    /// Protocol handler
    handler: Arc<McpProtocolHandler>,
    
    /// Available models
    models: Arc<RwLock<Vec<Model>>>,
    
    /// Active conversations
    conversations: Arc<RwLock<HashMap<String, Conversation>>>,
    
    /// Active streaming sessions
    streaming_sessions: Arc<Mutex<HashMap<String, mpsc::Sender<Result<Message, MessageError>>>>>,
}

impl McpService {
    /// Create a new MCP service
    pub fn new() -> Self {
        // Load configuration
        let config = Config::global();
        let config_guard = config.lock().unwrap();
        
        // Create MCP configuration
        let api_key = config_guard
            .get_string("api.key")
            .unwrap_or_else(|| String::new());
        
        let mcp_config = McpConfig::with_api_key(api_key)
            .with_url(
                config_guard
                    .get_string("api.url")
                    .unwrap_or_else(|| "wss://api.anthropic.com/v1/messages".to_string()),
            )
            .with_model(
                config_guard
                    .get_string("api.model")
                    .unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
            );
        
        // Create client and handler
        let client = Arc::new(McpClient::new(mcp_config.clone()));
        let handler = Arc::new(McpProtocolHandler::new(mcp_config));
        
        // Define available models
        let models = vec![
            Model::claude("opus", "20240229"),
            Model::claude("sonnet", "20240229"),
            Model::claude("haiku", "20240229"),
        ];
        
        Self {
            client,
            handler,
            models: Arc::new(RwLock::new(models)),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            streaming_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get the current connection status
    pub fn connection_status(&self) -> ConnectionStatus {
        self.handler.connection_status()
    }
    
    /// Connect to the MCP server
    pub async fn connect(&self) -> Result<(), String> {
        self.handler.connect().await
    }
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> Result<(), String> {
        self.handler.disconnect().await
    }
    
    /// Get available models
    pub fn available_models(&self) -> Vec<Model> {
        self.models.read().unwrap().clone()
    }
    
    /// Get active conversations
    pub fn active_conversations(&self) -> Vec<Conversation> {
        self.conversations
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
    
    /// Create a new conversation
    pub fn create_conversation(&self, title: &str, model: Model) -> Conversation {
        let conversation = Conversation::new(title, model);
        
        // Store conversation
        {
            let mut conversations = self.conversations.write().unwrap();
            conversations.insert(conversation.id.clone(), conversation.clone());
        }
        
        conversation
    }
    
    /// Get a conversation by ID
    pub fn get_conversation(&self, id: &str) -> Option<Conversation> {
        self.conversations.read().unwrap().get(id).cloned()
    }
    
    /// Delete a conversation
    pub fn delete_conversation(&self, id: &str) -> Result<(), String> {
        let mut conversations = self.conversations.write().unwrap();
        
        if conversations.remove(id).is_some() {
            Ok(())
        } else {
            Err(format!("Conversation with ID {} not found", id))
        }
    }
    
    /// Send a message in a conversation
    pub async fn send_message(&self, conversation_id: &str, message: Message) -> Result<Message, MessageError> {
        // Check if conversation exists
        if !self.conversations.read().unwrap().contains_key(conversation_id) {
            return Err(MessageError::Unknown(format!(
                "Conversation with ID {} not found",
                conversation_id
            )));
        }
        
        // Add conversation context in metadata
        let message_with_context = Message {
            metadata: Some(HashMap::from([(
                "conversation_id".to_string(),
                serde_json::to_value(conversation_id).unwrap(),
            )])),
            ..message
        };
        
        // Send message through protocol handler
        match timeout(Duration::from_secs(120), self.handler.send_message(message_with_context.clone())).await {
            Ok(result) => match result {
                Ok(_) => {
                    // In a real implementation, we would get the response from the handler
                    // For now, we'll simulate a response
                    
                    // Create a dummy response
                    let response = Message {
                        id: Uuid::new_v4().to_string(),
                        role: crate::models::messages::MessageRole::Assistant,
                        content: crate::models::messages::MessageContent {
                            parts: vec![crate::models::messages::ContentType::Text {
                                text: "This is a simulated response from the MCP server.".to_string(),
                            }],
                        },
                        metadata: message_with_context.metadata,
                        created_at: std::time::SystemTime::now(),
                    };
                    
                    Ok(response)
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(MessageError::Timeout(Duration::from_secs(120))),
        }
    }
    
    /// Start a streaming message in a conversation
    pub async fn stream_message(
        &self,
        conversation_id: &str,
        message: Message,
    ) -> Result<mpsc::Receiver<Result<Message, MessageError>>, MessageError> {
        // Check if conversation exists
        if !self.conversations.read().unwrap().contains_key(conversation_id) {
            return Err(MessageError::Unknown(format!(
                "Conversation with ID {} not found",
                conversation_id
            )));
        }
        
        // Create streaming channel
        let (tx, rx) = mpsc::channel(32);
        
        // Store streaming session
        {
            let mut sessions = self.streaming_sessions.lock().unwrap();
            sessions.insert(message.id.clone(), tx.clone());
        }
        
        // Add conversation context in metadata
        let message_with_context = Message {
            metadata: Some(HashMap::from([(
                "conversation_id".to_string(),
                serde_json::to_value(conversation_id).unwrap(),
            )])),
            ..message.clone()
        };
        
        // Send message through client directly for streaming
        // In a real implementation, we would use client.stream() and adapt its output
        
        // For now, simulate streaming with a few chunks
        let tx_clone = tx.clone();
        let message_id = message.id.clone();
        
        tokio::spawn(async move {
            // Simulate streaming with delay
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Send first chunk
            let _ = tx_clone
                .send(Ok(Message {
                    id: message_id.clone(),
                    role: crate::models::messages::MessageRole::Assistant,
                    content: crate::models::messages::MessageContent {
                        parts: vec![crate::models::messages::ContentType::Text {
                            text: "This is a simulated ".to_string(),
                        }],
                    },
                    metadata: message_with_context.metadata.clone(),
                    created_at: std::time::SystemTime::now(),
                }))
                .await;
            
            // Delay between chunks
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Send second chunk
            let _ = tx_clone
                .send(Ok(Message {
                    id: message_id.clone(),
                    role: crate::models::messages::MessageRole::Assistant,
                    content: crate::models::messages::MessageContent {
                        parts: vec![crate::models::messages::ContentType::Text {
                            text: "This is a simulated streaming ".to_string(),
                        }],
                    },
                    metadata: message_with_context.metadata.clone(),
                    created_at: std::time::SystemTime::now(),
                }))
                .await;
            
            // Delay between chunks
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Send final chunk
            let _ = tx_clone
                .send(Ok(Message {
                    id: message_id,
                    role: crate::models::messages::MessageRole::Assistant,
                    content: crate::models::messages::MessageContent {
                        parts: vec![crate::models::messages::ContentType::Text {
                            text: "This is a simulated streaming response from the MCP server.".to_string(),
                        }],
                    },
                    metadata: message_with_context.metadata,
                    created_at: std::time::SystemTime::now(),
                }))
                .await;
            
            // Clean up streaming session
            tokio::time::sleep(Duration::from_millis(100)).await;
            // Note: In a real implementation, we would remove the session when streaming ends
        });
        
        Ok(rx)
    }
    
    /// Cancel a streaming message
    pub async fn cancel_streaming(&self, message_id: &str) -> Result<(), MessageError> {
        // Remove streaming session
        let mut sessions = self.streaming_sessions.lock().unwrap();
        if sessions.remove(message_id).is_some() {
            // In a real implementation, we would also tell the MCP server to cancel
            Ok(())
        } else {
            Err(MessageError::Unknown(format!(
                "Streaming session with ID {} not found",
                message_id
            )))
        }
    }
}

/// Global MCP service instance
static MCP_SERVICE: once_cell::sync::OnceCell<McpService> = once_cell::sync::OnceCell::new();

/// Get the global MCP service instance
pub fn get_mcp_service() -> &'static McpService {
    MCP_SERVICE.get_or_init(|| McpService::new())
}
