use std::sync::Arc;
use tokio::sync::mpsc;
use log::{debug, error, info, warn};

use crate::error::{McpError, McpResult};
use crate::models::{Conversation, Message, Model};
use crate::service::mcp::McpService;

/// Service for managing chat interactions
pub struct ChatService {
    /// MCP service for communication
    mcp_service: Arc<McpService>,
}

impl ChatService {
    /// Create a new chat service
    pub fn new(mcp_service: Arc<McpService>) -> Self {
        Self { mcp_service }
    }
    
    /// Create a new conversation
    pub async fn create_conversation(&self, title: &str, model: Option<Model>) -> McpResult<Conversation> {
        // Use provided model or default
        let model = match model {
            Some(m) => m,
            None => {
                let models = self.mcp_service.available_models().await;
                models.into_iter().next().unwrap_or_else(Model::default_claude)
            }
        };
        
        self.mcp_service.create_conversation(title, &model).await
    }
    
    /// Get a conversation by ID
    pub async fn get_conversation(&self, id: &str) -> McpResult<Conversation> {
        self.mcp_service.get_conversation(id).await
    }
    
    /// List all conversations
    pub async fn list_conversations(&self) -> McpResult<Vec<Conversation>> {
        Ok(self.mcp_service.active_conversations().await)
    }
    
    /// Delete a conversation
    pub async fn delete_conversation(&self, id: &str) -> McpResult<()> {
        self.mcp_service.delete_conversation(id).await
    }
    
    /// Send a message in a conversation
    pub async fn send_message(&self, conversation_id: &str, content: &str) -> McpResult<Message> {
        // Create user message
        let message = Message::user(content);
        
        // Send via MCP service
        self.mcp_service.send_message(conversation_id, message).await
    }
    
    /// Send a message with streaming response
    pub async fn send_message_streaming(
        &self,
        conversation_id: &str,
        content: &str,
    ) -> McpResult<mpsc::Receiver<McpResult<Message>>> {
        // Create user message
        let message = Message::user(content);
        
        // Send via MCP service with streaming
        self.mcp_service.stream_message(conversation_id, message).await
    }
    
    /// Set a system message for a conversation
    pub async fn set_system_message(&self, conversation_id: &str, content: &str) -> McpResult<()> {
        // Get current conversation
        let mut conversation = self.mcp_service.get_conversation(conversation_id).await?;
        
        // Find existing system message, if any
        let has_system_message = conversation
            .messages
            .iter()
            .any(|msg| msg.role == crate::models::MessageRole::System);
        
        // If there's an existing system message, replace it
        if has_system_message {
            conversation.messages = conversation
                .messages
                .into_iter()
                .map(|msg| {
                    if msg.role == crate::models::MessageRole::System {
                        Message::system(content)
                    } else {
                        msg
                    }
                })
                .collect();
        } else {
            // Otherwise, add a new system message at the beginning
            let mut new_messages = vec![Message::system(content)];
            new_messages.extend(conversation.messages);
            conversation.messages = new_messages;
        }
        
        // Update the conversation
        self.mcp_service.update_conversation(conversation).await
    }
    
    /// Get available models
    pub async fn available_models(&self) -> McpResult<Vec<Model>> {
        Ok(self.mcp_service.available_models().await)
    }
}
