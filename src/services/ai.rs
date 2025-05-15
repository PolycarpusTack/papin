use crate::ai::router::{get_model_router, NetworkStatus, RouterStrategy};
use crate::models::messages::{Message, MessageError, ConversationMessage, MessageStatus};
use crate::models::{Conversation, Model};
use crate::utils::config;
use crate::utils::events::{events, get_event_system};
use crate::utils::locks::{SafeLock, SafeRwLock, LockError};
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
        
        // Use SafeLock instead of unwrap()
        let strategy = match config.safe_lock() {
            Ok(config_guard) => {
                let strategy_str = config_guard
                    .get_string("ai.router.strategy")
                    .unwrap_or_else(|| "prefer_online".to_string());
                
                match strategy_str.as_str() {
                    "prefer_online" => RouterStrategy::PreferOnline,
                    "prefer_local" => RouterStrategy::PreferLocal,
                    "online_only" => RouterStrategy::OnlineOnly,
                    "local_only" => RouterStrategy::LocalOnly,
                    "round_robin" => RouterStrategy::RoundRobin,
                    "rules_based" => RouterStrategy::RulesBased,
                    _ => RouterStrategy::PreferOnline,
                }
            },
            Err(e) => {
                // Log error but use a default strategy
                error!("Failed to acquire config lock: {}", e);
                warn!("Using default router strategy (PreferOnline) due to lock error");
                RouterStrategy::PreferOnline
            }
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
                        // Update cache - use SafeRwLock instead of unwrap()
                        match models_clone.safe_write() {
                            Ok(mut models_guard) => {
                                *models_guard = models;
                            },
                            Err(e) => {
                                error!("Failed to acquire write lock on models: {}", e);
                            }
                        }
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
            match self.models.safe_read() {
                Ok(models) => {
                    if !models.is_empty() {
                        return models.clone();
                    }
                },
                Err(e) => {
                    error!("Failed to acquire read lock on models: {}", e);
                    // Continue to fetching from router
                }
            }
        }
        
        // Fetch models from router
        let models = self.router.get_available_models().await;
        
        // Update cache
        {
            match self.models.safe_write() {
                Ok(mut models_guard) => {
                    *models_guard = models.clone();
                },
                Err(e) => {
                    error!("Failed to acquire write lock on models: {}", e);
                    // Continue with returning the models we fetched
                }
            }
        }
        
        models
    }
    
    /// Create a new conversation
    pub fn create_conversation(&self, title: &str, model: Model) -> Conversation {
        let conversation = Conversation::new(title, model);
        
        // Store conversation
        match self.conversations.safe_write() {
            Ok(mut conversations) => {
                conversations.insert(conversation.id.clone(), Vec::new());
            },
            Err(e) => {
                error!("Failed to acquire write lock on conversations: {}", e);
                // Even if we fail to store, return the conversation object
            }
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
            match self.conversations.safe_write() {
                Ok(mut conversations) => {
                    if conversations.remove(id).is_none() {
                        return Err(format!("Conversation {} not found", id));
                    }
                },
                Err(e) => {
                    let error_msg = format!("Failed to acquire write lock on conversations: {}", e);
                    error!("{}", error_msg);
                    return Err(error_msg);
                }
            }
        }
        
        // Remove listeners
        {
            match self.message_listeners.safe_lock() {
                Ok(mut listeners) => {
                    listeners.remove(id);
                },
                Err(e) => {
                    let error_msg = format!("Failed to acquire lock on message listeners: {}", e);
                    error!("{}", error_msg);
                    return Err(error_msg);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get conversation message history
    pub fn get_messages(&self, conversation_id: &str) -> Vec<ConversationMessage> {
        match self.conversations.safe_read() {
            Ok(conversations) => {
                conversations
                    .get(conversation_id)
                    .cloned()
                    .unwrap_or_default()
            },
            Err(e) => {
                error!("Failed to acquire read lock on conversations: {}", e);
                Vec::new() // Return empty vector instead of panicking
            }
        }
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
                    match self.streaming_sessions.safe_lock() {
                        Ok(mut sessions) => {
                            sessions.insert(stream_id.clone(), stream);
                        },
                        Err(e) => {
                            error!("Failed to acquire lock on streaming sessions: {}", e);
                            // Continue with streaming even if we couldn't store the session
                        }
                    }
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
                                        match conversations.safe_write() {
                                            Ok(mut convos) => {
                                                if let Some(messages) = convos.get_mut(&conversation_id) {
                                                    for msg in messages.iter_mut() {
                                                        if msg.message.id == response_id {
                                                            *msg = response_message.clone();
                                                            break;
                                                        }
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                error!("Failed to acquire write lock on conversations in streaming task: {}", e);
                                                // Continue sending updates to UI even if we couldn't update history
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
                                    match conversations.safe_write() {
                                        Ok(mut convos) => {
                                            if let Some(messages) = convos.get_mut(&conversation_id) {
                                                for msg in messages.iter_mut() {
                                                    if msg.message.id == response_id {
                                                        *msg = response_message.clone();
                                                        break;
                                                    }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            error!("Failed to acquire write lock on conversations in error handler: {}", e);
                                            // Continue sending updates to UI even if we couldn't update history
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
                            match conversations.safe_write() {
                                Ok(mut convos) => {
                                    if let Some(messages) = convos.get_mut(&conversation_id) {
                                        for msg in messages.iter_mut() {
                                            if msg.message.id == response_id {
                                                *msg = response_message.clone();
                                                break;
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to acquire write lock on conversations in streaming completion: {}", e);
                                    // Continue sending updates to UI even if we couldn't update history
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
            match self.message_listeners.safe_lock() {
                Ok(mut listeners) => {
                    let conversation_listeners = listeners
                        .entry(conversation_id.to_string())
                        .or_insert_with(Vec::new);
                    conversation_listeners.push(tx.clone());
                },
                Err(e) => {
                    error!("Failed to acquire lock on message listeners: {}", e);
                    // Continue to send existing messages even if we couldn't register
                }
            }
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
            match self.conversations.safe_write() {
                Ok(mut conversations) => {
                    let conversation_messages = conversations
                        .entry(conversation_id.to_string())
                        .or_insert_with(Vec::new);
                    conversation_messages.push(message.clone());
                },
                Err(e) => {
                    error!("Failed to acquire write lock on conversations: {}", e);
                    // Still try to notify listeners even if we couldn't update history
                }
            }
        }
        
        // Notify listeners
        self.notify_listeners(conversation_id, &message);
    }
    
    /// Update message status in history
    fn update_message_status(&self, conversation_id: &str, message_id: &str, status: MessageStatus) {
        let mut updated_message = None;
        
        // Update in history
        {
            match self.conversations.safe_write() {
                Ok(mut conversations) => {
                    if let Some(messages) = conversations.get_mut(conversation_id) {
                        for msg in messages.iter_mut() {
                            if msg.message.id == message_id {
                                msg.status = status;
                                updated_message = Some(msg.clone());
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to acquire write lock on conversations: {}", e);
                    // Can't update message status
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
        match self.message_listeners.safe_lock() {
            Ok(listeners) => {
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
            },
            Err(e) => {
                error!("Failed to acquire lock on message listeners: {}", e);
                // Can't notify listeners
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
