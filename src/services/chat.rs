use crate::models::messages::{Message, MessageError, ConversationMessage, MessageStatus};
use crate::models::{Conversation, Model};
use crate::services::mcp::{get_mcp_service, McpService};
use crate::utils::config;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Service for managing chat functionality
pub struct ChatService {
    /// MCP service for communication
    mcp_service: &'static McpService,
    
    /// Active conversations with their message history
    conversations: Arc<RwLock<HashMap<String, Vec<ConversationMessage>>>>,
    
    /// Message listeners (for UI updates)
    message_listeners: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<ConversationMessage>>>>>,
}

impl ChatService {
    /// Create a new chat service
    pub fn new() -> Self {
        Self {
            mcp_service: get_mcp_service(),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            message_listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get available models
    pub fn available_models(&self) -> Vec<Model> {
        self.mcp_service.available_models()
    }
    
    /// Create a new conversation
    pub fn create_conversation(&self, title: &str, model: Model) -> Conversation {
        let conversation = self.mcp_service.create_conversation(title, model);
        
        // Initialize message history
        {
            let mut conversations = self.conversations.write().unwrap();
            conversations.insert(conversation.id.clone(), Vec::new());
        }
        
        conversation
    }
    
    /// Get a conversation by ID
    pub fn get_conversation(&self, id: &str) -> Option<Conversation> {
        self.mcp_service.get_conversation(id)
    }
    
    /// Delete a conversation
    pub fn delete_conversation(&self, id: &str) -> Result<(), String> {
        // Remove from MCP service
        let result = self.mcp_service.delete_conversation(id);
        
        // Remove message history
        if result.is_ok() {
            let mut conversations = self.conversations.write().unwrap();
            conversations.remove(id);
            
            // Notify listeners that conversation was deleted
            let mut listeners = self.message_listeners.lock().unwrap();
            listeners.remove(id);
        }
        
        result
    }
    
    /// Get conversation message history
    pub fn get_messages(&self, conversation_id: &str) -> Vec<ConversationMessage> {
        let conversations = self.conversations.read().unwrap();
        conversations
            .get(conversation_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Send a message in a conversation
    pub async fn send_message(
        &self,
        conversation_id: &str,
        message: Message,
    ) -> Result<ConversationMessage, MessageError> {
        // Store message in history with 'sending' status
        let conversation_message = ConversationMessage {
            message: message.clone(),
            parent_ids: Vec::new(),
            completed_at: None,
            partial_content: None,
            status: MessageStatus::Sending,
        };
        
        self.add_message_to_history(conversation_id, conversation_message.clone());
        
        // Send message through MCP service
        match self.mcp_service.send_message(conversation_id, message).await {
            Ok(response) => {
                // Create response message
                let response_message = ConversationMessage {
                    message: response,
                    parent_ids: vec![conversation_message.message.id.clone()],
                    completed_at: Some(std::time::SystemTime::now()),
                    partial_content: None,
                    status: MessageStatus::Complete,
                };
                
                // Update original message status to complete
                self.update_message_status(
                    conversation_id,
                    &conversation_message.message.id,
                    MessageStatus::Complete,
                );
                
                // Store response in history
                self.add_message_to_history(conversation_id, response_message.clone());
                
                Ok(response_message)
            }
            Err(e) => {
                // Update message status to failed
                self.update_message_status(
                    conversation_id,
                    &conversation_message.message.id,
                    MessageStatus::Failed,
                );
                
                Err(e)
            }
        }
    }
    
    /// Stream a message in a conversation
    pub async fn stream_message(
        &self,
        conversation_id: &str,
        message: Message,
    ) -> Result<mpsc::Receiver<ConversationMessage>, MessageError> {
        // Create streaming channel for UI
        let (tx, rx) = mpsc::channel(32);
        
        // Store message in history with 'sending' status
        let conversation_message = ConversationMessage {
            message: message.clone(),
            parent_ids: Vec::new(),
            completed_at: None,
            partial_content: None,
            status: MessageStatus::Sending,
        };
        
        self.add_message_to_history(conversation_id, conversation_message.clone());
        
        // Start streaming through MCP service
        match self.mcp_service.stream_message(conversation_id, message).await {
            Ok(mut stream) => {
                // Create initial response message
                let response_id = Uuid::new_v4().to_string();
                let mut response_message = ConversationMessage {
                    message: Message {
                        id: response_id.clone(),
                        role: crate::models::messages::MessageRole::Assistant,
                        content: crate::models::messages::MessageContent {
                            parts: vec![crate::models::messages::ContentType::Text {
                                text: String::new(),
                            }],
                        },
                        metadata: None,
                        created_at: std::time::SystemTime::now(),
                    },
                    parent_ids: vec![conversation_message.message.id.clone()],
                    completed_at: None,
                    partial_content: Some(String::new()),
                    status: MessageStatus::Streaming,
                };
                
                // Update original message status to complete
                self.update_message_status(
                    conversation_id,
                    &conversation_message.message.id,
                    MessageStatus::Complete,
                );
                
                // Store initial response in history
                self.add_message_to_history(conversation_id, response_message.clone());
                
                // Send initial message to UI
                let _ = tx.send(response_message.clone()).await;
                
                // Handle streaming in a separate task
                let conversations = self.conversations.clone();
                let tx_clone = tx.clone();
                let conversation_id = conversation_id.to_string();
                
                tokio::spawn(async move {
                    let mut full_text = String::new();
                    
                    // Process streaming messages
                    while let Some(result) = stream.recv().await {
                        match result {
                            Ok(chunk) => {
                                // Extract text content
                                if let Some(text) = chunk.text_content() {
                                    // Append to full text
                                    full_text.push_str(text);
                                    
                                    // Update response message
                                    response_message.partial_content = Some(full_text.clone());
                                    response_message.message.content.parts = vec![
                                        crate::models::messages::ContentType::Text {
                                            text: full_text.clone(),
                                        },
                                    ];
                                    
                                    // Update in history
                                    {
                                        let mut convos = conversations.write().unwrap();
                                        if let Some(messages) = convos.get_mut(&conversation_id) {
                                            for msg in messages.iter_mut() {
                                                if msg.message.id == response_id {
                                                    *msg = response_message.clone();
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Send update to UI
                                    let _ = tx_clone.send(response_message.clone()).await;
                                }
                            }
                            Err(e) => {
                                error!("Streaming error: {}", e);
                                
                                // Update status to failed
                                response_message.status = MessageStatus::Failed;
                                
                                // Update in history
                                {
                                    let mut convos = conversations.write().unwrap();
                                    if let Some(messages) = convos.get_mut(&conversation_id) {
                                        for msg in messages.iter_mut() {
                                            if msg.message.id == response_id {
                                                *msg = response_message.clone();
                                                break;
                                            }
                                        }
                                    }
                                }
                                
                                // Send final update to UI
                                let _ = tx_clone.send(response_message.clone()).await;
                                break;
                            }
                        }
                    }
                    
                    // If we got here, streaming is complete
                    if response_message.status == MessageStatus::Streaming {
                        response_message.status = MessageStatus::Complete;
                        response_message.completed_at = Some(std::time::SystemTime::now());
                        response_message.partial_content = None;
                        
                        // Update in history
                        {
                            let mut convos = conversations.write().unwrap();
                            if let Some(messages) = convos.get_mut(&conversation_id) {
                                for msg in messages.iter_mut() {
                                    if msg.message.id == response_id {
                                        *msg = response_message.clone();
                                        break;
                                    }
                                }
                            }
                        }
                        
                        // Send final update to UI
                        let _ = tx_clone.send(response_message).await;
                    }
                });
                
                Ok(rx)
            }
            Err(e) => {
                // Update message status to failed
                self.update_message_status(
                    conversation_id,
                    &conversation_message.message.id,
                    MessageStatus::Failed,
                );
                
                Err(e)
            }
        }
    }
    
    /// Cancel a streaming message
    pub async fn cancel_streaming(
        &self,
        conversation_id: &str,
        message_id: &str,
    ) -> Result<(), MessageError> {
        // Tell MCP service to cancel
        let result = self.mcp_service.cancel_streaming(message_id).await;
        
        // Update message status to cancelled
        if result.is_ok() {
            self.update_message_status(conversation_id, message_id, MessageStatus::Cancelled);
        }
        
        result
    }
    
    /// Register a message listener for a conversation
    pub async fn register_listener(
        &self,
        conversation_id: &str,
    ) -> mpsc::Receiver<ConversationMessage> {
        let (tx, rx) = mpsc::channel(32);
        
        // Store listener
        {
            let mut listeners = self.message_listeners.lock().unwrap();
            let conversation_listeners = listeners
                .entry(conversation_id.to_string())
                .or_insert_with(Vec::new);
            conversation_listeners.push(tx);
        }
        
        // Send existing messages
        let messages = self.get_messages(conversation_id);
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            for message in messages {
                let _ = tx_clone.send(message).await;
            }
        });
        
        rx
    }
    
    /// Add a message to conversation history
    fn add_message_to_history(&self, conversation_id: &str, message: ConversationMessage) {
        // Add to history
        {
            let mut conversations = self.conversations.write().unwrap();
            let conversation_messages = conversations
                .entry(conversation_id.to_string())
                .or_insert_with(Vec::new);
            conversation_messages.push(message.clone());
        }
        
        // Notify listeners
        self.notify_listeners(conversation_id, &message);
    }
    
    /// Update message status in history
    fn update_message_status(&self, conversation_id: &str, message_id: &str, status: MessageStatus) {
        let mut updated_message = None;
        
        // Update in history
        {
            let mut conversations = self.conversations.write().unwrap();
            if let Some(messages) = conversations.get_mut(conversation_id) {
                for msg in messages.iter_mut() {
                    if msg.message.id == message_id {
                        msg.status = status;
                        updated_message = Some(msg.clone());
                        break;
                    }
                }
            }
        }
        
        // Notify listeners
        if let Some(message) = updated_message {
            self.notify_listeners(conversation_id, &message);
        }
    }
    
    /// Notify all listeners for a conversation
    fn notify_listeners(&self, conversation_id: &str, message: &ConversationMessage) {
        let listeners = self.message_listeners.lock().unwrap();
        if let Some(conversation_listeners) = listeners.get(conversation_id) {
            let message = message.clone();
            for listener in conversation_listeners {
                let tx = listener.clone();
                let message = message.clone();
                
                tokio::spawn(async move {
                    let _ = tx.send(message).await;
                });
            }
        }
    }
}

/// Global chat service instance
static CHAT_SERVICE: once_cell::sync::OnceCell<ChatService> = once_cell::sync::OnceCell::new();

/// Get the global chat service instance
pub fn get_chat_service() -> &'static ChatService {
    CHAT_SERVICE.get_or_init(|| ChatService::new())
}
