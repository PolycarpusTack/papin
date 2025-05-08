use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, Mutex, RwLock};
use log::{debug, error, info, warn};

use crate::config::{get_settings, get_storage_manager};
use crate::error::{McpError, McpResult};
use crate::models::{Conversation, Message, Model};
use crate::protocol::{ConnectionStatus, McpClient, McpConfig};

/// Service for interacting with the MCP protocol
pub struct McpService {
    /// MCP client
    client: Arc<McpClient>,
    
    /// Available models
    models: Arc<RwLock<Vec<Model>>>,
    
    /// Active conversations
    conversations: Arc<RwLock<HashMap<String, Conversation>>>,
    
    /// Active streaming sessions
    streaming_sessions: Arc<Mutex<HashMap<String, mpsc::Sender<McpResult<Message>>>>>,
}

impl McpService {
    /// Create a new MCP service
    pub fn new() -> Self {
        // Load settings
        let settings = get_settings();
        let settings_guard = settings.lock().unwrap();
        
        // Get API key
        let api_key = settings_guard
            .get_api_key()
            .unwrap_or(Ok(None))
            .unwrap_or(None)
            .unwrap_or_default();
        
        // Create MCP configuration
        let mcp_config = McpConfig::with_api_key(api_key)
            .with_url(settings_guard.api.url.clone())
            .with_model(settings_guard.api.model.clone());
        
        // Create MCP client
        let client = Arc::new(McpClient::new(mcp_config));
        
        // Define available models
        let models = Model::available_claude_models();
        
        Self {
            client,
            models: Arc::new(RwLock::new(models)),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            streaming_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Initialize the service - load saved data
    pub async fn initialize(&self) -> McpResult<()> {
        // Load saved conversations
        let storage = get_storage_manager();
        let conversations = storage.list_conversations()?;
        
        // Store in memory
        {
            let mut conv_map = self.conversations.write().await;
            for conversation in conversations {
                conv_map.insert(conversation.id.clone(), conversation);
            }
        }
        
        Ok(())
    }
    
    /// Get the current connection status
    pub fn connection_status(&self) -> ConnectionStatus {
        self.client.connection_status()
    }
    
    /// Connect to the MCP server
    pub async fn connect(&self) -> McpResult<()> {
        self.client.connect().await
    }
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> McpResult<()> {
        self.client.disconnect().await
    }
    
    /// Get available models
    pub async fn available_models(&self) -> Vec<Model> {
        self.models.read().await.clone()
    }
    
    /// Get active conversations
    pub async fn active_conversations(&self) -> Vec<Conversation> {
        self.conversations
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }
    
    /// Create a new conversation
    pub async fn create_conversation(&self, title: &str, model: &Model) -> McpResult<Conversation> {
        let conversation = Conversation::new(title, model.clone());
        
        // Store conversation
        {
            let mut conversations = self.conversations.write().await;
            conversations.insert(conversation.id.clone(), conversation.clone());
        }
        
        // Save to storage
        let storage = get_storage_manager();
        storage.save_conversation(&conversation)?;
        
        Ok(conversation)
    }
    
    /// Get a conversation by ID
    pub async fn get_conversation(&self, id: &str) -> McpResult<Conversation> {
        // Try to get from memory
        {
            let conversations = self.conversations.read().await;
            if let Some(conv) = conversations.get(id) {
                return Ok(conv.clone());
            }
        }
        
        // Try to load from storage
        let storage = get_storage_manager();
        let conversation = storage.load_conversation(id)?;
        
        // Store in memory
        {
            let mut conversations = self.conversations.write().await;
            conversations.insert(conversation.id.clone(), conversation.clone());
        }
        
        Ok(conversation)
    }
    
    /// Update a conversation
    pub async fn update_conversation(&self, conversation: Conversation) -> McpResult<()> {
        // Store in memory
        {
            let mut conversations = self.conversations.write().await;
            conversations.insert(conversation.id.clone(), conversation.clone());
        }
        
        // Save to storage
        let storage = get_storage_manager();
        storage.save_conversation(&conversation)?;
        
        Ok(())
    }
    
    /// Delete a conversation
    pub async fn delete_conversation(&self, id: &str) -> McpResult<()> {
        // Remove from memory
        {
            let mut conversations = self.conversations.write().await;
            conversations.remove(id);
        }
        
        // Remove from storage
        let storage = get_storage_manager();
        storage.delete_conversation(id)?;
        
        Ok(())
    }
    
    /// Send a message in a conversation
    pub async fn send_message(&self, conversation_id: &str, message: Message) -> McpResult<Message> {
        // Get conversation
        let mut conversation = self.get_conversation(conversation_id).await?;
        
        // Add user message to conversation
        conversation.add_message(message.clone());
        
        // Save conversation with user message
        self.update_conversation(conversation.clone()).await?;
        
        // Check connection status
        if self.connection_status() != ConnectionStatus::Connected {
            self.connect().await?;
        }
        
        // Get settings
        let settings = get_settings();
        let settings_guard = settings.lock().unwrap();
        
        // Send message to MCP server
        let response = self
            .client
            .send_completion(
                &conversation.model.id,
                &conversation.messages,
                settings_guard.model.max_tokens,
                settings_guard.model.temperature,
            )
            .await?;
        
        // Add assistant response to conversation
        conversation.add_message(response.clone());
        
        // Save conversation with assistant response
        self.update_conversation(conversation).await?;
        
        Ok(response)
    }
    
    /// Start a streaming message in a conversation
    pub async fn stream_message(
        &self,
        conversation_id: &str,
        message: Message,
    ) -> McpResult<mpsc::Receiver<McpResult<Message>>> {
        // Get conversation
        let mut conversation = self.get_conversation(conversation_id).await?;
        
        // Add user message to conversation
        conversation.add_message(message.clone());
        
        // Save conversation with user message
        self.update_conversation(conversation.clone()).await?;
        
        // Check connection status
        if self.connection_status() != ConnectionStatus::Connected {
            self.connect().await?;
        }
        
        // Get settings
        let settings = get_settings();
        let settings_guard = settings.lock().unwrap();
        
        // Create streaming channel
        let (tx, rx) = mpsc::channel(32);
        
        // Store streaming session
        {
            let mut sessions = self.streaming_sessions.lock().await;
            sessions.insert(message.id.clone(), tx.clone());
        }
        
        // Start streaming
        let client_clone = self.client.clone();
        let model_id = conversation.model.id.clone();
        let messages = conversation.messages.clone();
        let max_tokens = settings_guard.model.max_tokens;
        let temperature = settings_guard.model.temperature;
        let session_id = message.id.clone();
        let conversation_id = conversation_id.to_string();
        let service = Arc::new(self.clone());
        
        tokio::spawn(async move {
            // Start streaming
            match client_clone
                .stream_completion(&model_id, &messages, max_tokens, temperature)
                .await
            {
                Ok(mut receiver) => {
                    let mut full_response = Message {
                        id: session_id.clone(),
                        role: crate::models::MessageRole::Assistant,
                        content: crate::models::MessageContent {
                            parts: vec![crate::models::ContentType::Text {
                                text: String::new(),
                            }],
                        },
                        metadata: None,
                        created_at: SystemTime::now(),
                    };
                    
                    // Process streaming chunks
                    while let Some(chunk) = receiver.recv().await {
                        // Accumulate text
                        if let crate::models::ContentType::Text { ref text } = chunk.content.parts[0] {
                            if let crate::models::ContentType::Text { ref mut text } = full_response.content.parts[0] {
                                *text = text.clone();
                            }
                        }
                        
                        // Send chunk to receiver
                        if tx.send(Ok(chunk)).await.is_err() {
                            // Receiver dropped, cancel streaming
                            let _ = client_clone.cancel_streaming(&session_id).await;
                            break;
                        }
                    }
                    
                    // Add the complete message to the conversation
                    let mut conversation = match service.get_conversation(&conversation_id).await {
                        Ok(conv) => conv,
                        Err(_) => return,
                    };
                    
                    conversation.add_message(full_response);
                    let _ = service.update_conversation(conversation).await;
                }
                Err(e) => {
                    // Send error to receiver
                    let _ = tx.send(Err(e)).await;
                }
            }
            
            // Remove streaming session
            let mut sessions = service.streaming_sessions.lock().await;
            sessions.remove(&session_id);
        });
        
        Ok(rx)
    }
    
    /// Cancel a streaming message
    pub async fn cancel_streaming(&self, message_id: &str) -> McpResult<()> {
        // Cancel streaming with MCP client
        let result = self.client.cancel_streaming(message_id).await;
        
        // Remove streaming session
        {
            let mut sessions = self.streaming_sessions.lock().await;
            sessions.remove(message_id);
        }
        
        result
    }
}

impl Clone for McpService {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            models: self.models.clone(),
            conversations: self.conversations.clone(),
            streaming_sessions: self.streaming_sessions.clone(),
        }
    }
}
