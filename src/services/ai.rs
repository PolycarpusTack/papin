use crate::ai::router::{get_model_router, NetworkStatus, RouterStrategy};
use crate::models::messages::{Message, MessageError, ConversationMessage, MessageStatus};
use crate::models::{Conversation, Model};
use crate::utils::config;
use crate::utils::events::{events, get_event_system};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Service for interacting with AI models
pub struct AiService {
    /// Model router
    router: &'static crate::ai::router::ModelRouter,
    
    /// Available models cache
    models: Arc<RwLock<Vec<Model>>>,
    
    /// Active conversations with their message history
    conversations: Arc<RwLock<HashMap<String, Vec<ConversationMessage>>>>,
    
    /// Message listeners (for UI updates)
    message_listeners: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<ConversationMessage>>>>>,
    
    /// Streaming sessions
    streaming_sessions: Arc<Mutex<HashMap<String, mpsc::Sender<Result<Message, MessageError>>>>>,
}

impl AiService {
    /// Create a new AI service
    pub fn new() -> Self {
        // Get model router
        let router = get_model_router();
        
        // Set router strategy based on config
        let config = config::get_config();
        let config_guard = config.lock().unwrap();
        
        let strategy_str = config_guard
            .get_string("ai.router.strategy")
            .unwrap_or_else(|| "prefer_online".to_string());
        
        let strategy = match strategy_str.as_str() {
            "prefer_online" => RouterStrategy::PreferOnline,
            "prefer_local" => RouterStrategy::PreferLocal,
            "online_only" => RouterStrategy::OnlineOnly,
            "local_only" => RouterStrategy::LocalOnly,
            "round_robin" => RouterStrategy::RoundRobin,
            "rules_based" => RouterStrategy::RulesBased,
            _ => RouterStrategy::PreferOnline,
        };
        
        router.set_strategy(strategy);
        
        // Initialize service
        let service = Self {
            router,
            models: Arc::new(RwLock::new(Vec::new())),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            message_listeners: Arc::new(Mutex::new(HashMap::new())),
            streaming_sessions: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Set up a background task to periodically refresh available models
        let models_clone = service.models.clone();
        tokio::spawn(async move {
            loop {
                match get_model_router().get_available_models().await {
                    models => {
                        // Update cache
                        let mut models_guard = models_clone.write().unwrap();
                        *models_guard = models;
                    }
                }
                
                // Sleep for 5 minutes before refreshing again
                tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
            }
        });
        
        service
    }
    
    /// Set network status
    pub fn set_network_status(&self, status: NetworkStatus) {
        self.router.set_network_status(status);
    }
    
    /// Get available models
    pub async fn available_models(&self) -> Vec<Model> {
        // Check cache first
        {
            let models = self.models.read().unwrap();
            if !models.is_empty() {
                return models.clone();
            }
        }
        
        // Fetch models from router
        let models = self.router.get_available_models().await;
        
        // Update cache
        {
            let mut models_guard = self.models.write().unwrap();
            *models_guard = models.clone();
        }
        
        models
    }
    
    /// Create a new conversation
    pub fn create_conversation(&self, title: &str, model: Model) -> Conversation {
        let conversation = Conversation::new(title, model);
        
        // Store conversation
        {
            let mut conversations = self.conversations.write().unwrap();
            conversations.insert(conversation.id.clone(), Vec::new());
        }
        
        conversation
    }
    
    /// Get a conversation by ID
    pub fn get_conversation(&self, id: &str) -> Option<Conversation> {
        // In a real implementation, this would fetch from storage
        None
    }
    
    /// Delete a conversation
    pub fn delete_conversation(&self, id: &str) -> Result<(), String> {
        // Remove conversation messages
        {
            let mut conversations = self.conversations.write().unwrap();
            if conversations.remove(id).is_none() {
                return Err(format!("Conversation {} not found", id));
            }
        }
        
        // Remove listeners
        {
            let mut listeners = self.message_listeners.lock().unwrap();
            listeners.remove(id);
        }
        
        Ok(())
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
        model_id: &str,
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
        
        // Send message through router
        match self.router.complete(model_id, message).await {
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
        model_id: &str,
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
        
        // Start streaming through router
        match self.router.stream(model_id, message).await {
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
                        metadata: Some(HashMap::from([(
                            "model".to_string(),
                            serde_json::to_value(model_id).unwrap(),
                        )])),
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
                
                // Store streaming session
                let stream_id = Uuid::new_v4().to_string();
                {
                    let mut sessions = self.streaming_sessions.lock().unwrap();
                    sessions.insert(stream_id.clone(), stream);
                }
                
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
        // Update message status to cancelled
        self.update_message_status(conversation_id, message_id, MessageStatus::Cancelled);
        
        // Tell router to cancel the stream
        // In a real implementation, we would track which stream belongs to which message
        self.router.cancel_stream(message_id).await
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

/// Global AI service instance
static AI_SERVICE: once_cell::sync::OnceCell<AiService> = once_cell::sync::OnceCell::new();

/// Get the global AI service instance
pub fn get_ai_service() -> &'static AiService {
    AI_SERVICE.get_or_init(|| AiService::new())
}
