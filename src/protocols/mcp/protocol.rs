use crate::models::messages::{Message, MessageError};
use crate::protocols::mcp::error::McpError;
use crate::protocols::mcp::message::{McpMessage, McpMessagePayload, McpResponseMessage};
use crate::protocols::mcp::session::{Session, SessionManager, SessionMessageHandler};
use crate::protocols::mcp::types::{McpCompletionRequest, McpMessageRole, McpMessageType};
use crate::protocols::mcp::{McpClient, McpConfig};
use crate::protocols::{ConnectionStatus, ProtocolHandler};
use crate::observability::metrics::{increment_counter, record_gauge, record_histogram, time_operation};
use crate::utils::safe_lock::SafeLock;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
enum ConnectionState {
    /// Not connected
    Disconnected,
    
    /// Connecting to server
    Connecting,
    
    /// Connected but not authenticated
    Connected,
    
    /// Connected and authenticated
    Authenticated,
    
    /// Connection error
    Error(String),
}

/// MCP Protocol Handler
pub struct McpProtocolHandler {
    /// MCP client
    client: Arc<McpClient>,
    
    /// Current connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Connection state
    connection_state: Arc<RwLock<ConnectionState>>,
    
    /// Protocol configuration
    config: McpConfig,
    
    /// Session manager
    session_manager: Arc<SessionManager>,
    
    /// Current session
    current_session: Arc<RwLock<Option<Arc<Session>>>>,
    
    /// Session message handler
    message_handler: Arc<Mutex<Option<SessionMessageHandler>>>,
    
    /// Event receiver
    event_receiver: Arc<Mutex<Option<UnboundedReceiver<McpMessage>>>>,
    
    /// Last reconnection attempt
    last_reconnect_attempt: Arc<RwLock<Option<Instant>>>,
    
    /// Reconnection attempts
    reconnect_attempts: Arc<RwLock<u32>>,
    
    /// Request tracking map for streaming requests
    streaming_requests: Arc<Mutex<HashMap<String, Sender<Result<Message, MessageError>>>>>,
    
    /// Message routing table (message type -> handler function)
    message_router: Arc<Mutex<HashMap<McpMessageType, Box<dyn Fn(McpMessage) -> Result<(), McpError> + Send + Sync>>>>,
    
    /// Message queue for outgoing messages
    outgoing_queue: Arc<Mutex<Vec<McpMessage>>>,
    
    /// Flag to determine if the protocol handler is active
    is_active: Arc<RwLock<bool>>,
}

impl McpProtocolHandler {
    /// Create a new MCP protocol handler
    pub fn new(config: McpConfig) -> Self {
        let status = Arc::new(RwLock::new(ConnectionStatus::Disconnected));
        let client = Arc::new(McpClient::new(config.clone()));
        let session_manager = Arc::new(SessionManager::new(config.clone()));
        
        // Start session cleanup task
        session_manager.start_cleanup_task();
        
        Self {
            client,
            status,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            config,
            session_manager,
            current_session: Arc::new(RwLock::new(None)),
            message_handler: Arc::new(Mutex::new(None)),
            event_receiver: Arc::new(Mutex::new(None)),
            last_reconnect_attempt: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(RwLock::new(0)),
            streaming_requests: Arc::new(Mutex::new(HashMap::new())),
            message_router: Arc::new(Mutex::new(HashMap::new())),
            outgoing_queue: Arc::new(Mutex::new(Vec::new())),
            is_active: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Initialize the protocol handler
    pub fn initialize(&self) {
        // Set up message routing table
        self.setup_message_routing();
        
        // Start the outgoing message processor
        self.start_outgoing_processor();
        
        // Set active flag
        if let Ok(mut is_active) = self.is_active.safe_lock() {
            *is_active = true;
        } else {
            error!("Failed to acquire write lock on is_active during initialization");
        }
    }
    
    /// Set up message routing table
    fn setup_message_routing(&self) {
        if let Ok(mut router) = self.message_router.safe_lock_with_context("setup_message_routing") {
            let message_handler = self.message_handler.clone();
            let streaming_requests = self.streaming_requests.clone();
            
            // Set up routing for each message type
            router.insert(
                McpMessageType::CompletionResponse,
                Box::new(move |message| {
                    if let Ok(handler_guard) = message_handler.safe_lock() {
                        if let Some(handler) = handler_guard.as_ref() {
                            handler.handle_message(message)
                        } else {
                            Err(McpError::ProtocolError("No message handler available".to_string()))
                        }
                    } else {
                        Err(McpError::ProtocolError("Failed to lock message handler".to_string()))
                    }
                }),
            );
            
            // Set up routing for streaming messages
            let message_handler_clone = self.message_handler.clone();
            let streaming_requests_clone = self.streaming_requests.clone();
            
            router.insert(
                McpMessageType::StreamingMessage,
                Box::new(move |message| {
                    // First, handle using the message handler
                    if let Ok(handler_guard) = message_handler_clone.safe_lock() {
                        if let Some(handler) = handler_guard.as_ref() {
                            if let Err(e) = handler.handle_message(message.clone()) {
                                warn!("Error handling streaming message: {}", e);
                            }
                        }
                    } else {
                        warn!("Failed to lock message handler for streaming message");
                    }
                    
                    // Then, handle streaming delivery
                    if let McpMessagePayload::StreamingMessage { streaming_id, chunk, is_final } = &message.payload {
                        if let Ok(mut streaming_requests) = streaming_requests_clone.safe_lock() {
                            if let Some(sender) = streaming_requests.get(streaming_id) {
                                // Convert to application message format
                                let app_message = Message {
                                    id: Uuid::new_v4().to_string(),
                                    role: crate::models::messages::MessageRole::Assistant,
                                    content: crate::models::messages::MessageContent {
                                        parts: vec![crate::models::messages::ContentType::Text {
                                            text: chunk.to_string(),
                                        }],
                                    },
                                    metadata: None,
                                    created_at: SystemTime::now(),
                                };
                                
                                // Collect metrics
                                increment_counter("protocol.streaming.chunks", None);
                                record_histogram("protocol.streaming.chunk_size", chunk.len() as f64, None);
                                
                                // Try to send to application
                                if let Err(_) = sender.try_send(Ok(app_message)) {
                                    warn!("Failed to send streaming chunk to application: channel full or closed");
                                }
                                
                                // If final, remove from tracking
                                if *is_final {
                                    debug!("Received final chunk for streaming request {}", streaming_id);
                                    streaming_requests.remove(streaming_id);
                                    increment_counter("protocol.streaming.completed", None);
                                }
                            } else {
                                warn!("Received streaming chunk for unknown streaming ID: {}", streaming_id);
                            }
                        } else {
                            error!("Failed to lock streaming requests map");
                        }
                    }
                    
                    Ok(())
                }),
            );
            
            // Set up routing for streaming end
            let message_handler_clone = self.message_handler.clone();
            let streaming_requests_clone = self.streaming_requests.clone();
            
            router.insert(
                McpMessageType::StreamingEnd,
                Box::new(move |message| {
                    // First, handle using the message handler
                    if let Ok(handler_guard) = message_handler_clone.safe_lock() {
                        if let Some(handler) = handler_guard.as_ref() {
                            if let Err(e) = handler.handle_message(message.clone()) {
                                warn!("Error handling streaming end message: {}", e);
                            }
                        }
                    } else {
                        warn!("Failed to lock message handler for streaming end message");
                    }
                    
                    // Then, handle streaming cleanup
                    if let McpMessagePayload::StreamingEnd { streaming_id } = &message.payload {
                        if let Ok(mut streaming_requests) = streaming_requests_clone.safe_lock() {
                            if streaming_requests.remove(streaming_id).is_some() {
                                debug!("Removed streaming request {}", streaming_id);
                                increment_counter("protocol.streaming.ended", None);
                            }
                        } else {
                            error!("Failed to lock streaming requests map");
                        }
                    }
                    
                    Ok(())
                }),
            );
            
            // Set up routing for error messages
            let message_handler_clone = self.message_handler.clone();
            router.insert(
                McpMessageType::Error,
                Box::new(move |message| {
                    if let Ok(handler_guard) = message_handler_clone.safe_lock() {
                        if let Some(handler) = handler_guard.as_ref() {
                            handler.handle_message(message)
                        } else {
                            Err(McpError::ProtocolError("No message handler available".to_string()))
                        }
                    } else {
                        Err(McpError::ProtocolError("Failed to lock message handler".to_string()))
                    }
                }),
            );
            
            // Set up routing for ping/pong
            let connection_state_clone = self.connection_state.clone();
            router.insert(
                McpMessageType::Pong,
                Box::new(move |_| {
                    // Just update connection state
                    if let Ok(state) = connection_state_clone.safe_lock() {
                        if *state == ConnectionState::Authenticated {
                            debug!("Received pong message, connection is active");
                            // Record heartbeat success metric
                            increment_counter("protocol.heartbeat.success", None);
                        } else {
                            warn!("Received pong message but connection is not authenticated");
                        }
                    } else {
                        error!("Failed to read connection state");
                    }
                    Ok(())
                }),
            );
            
            // Set up routing for auth response
            let message_handler_clone = self.message_handler.clone();
            router.insert(
                McpMessageType::AuthResponse,
                Box::new(move |message| {
                    if let Ok(handler_guard) = message_handler_clone.safe_lock() {
                        if let Some(handler) = handler_guard.as_ref() {
                            handler.handle_message(message)
                        } else {
                            Err(McpError::ProtocolError("No message handler available".to_string()))
                        }
                    } else {
                        Err(McpError::ProtocolError("Failed to lock message handler".to_string()))
                    }
                }),
            );
            
            // Add a default handler for unhandled message types
            let message_handler_clone = self.message_handler.clone();
            router.insert(
                McpMessageType::Unknown,
                Box::new(move |message| {
                    warn!("Received unknown message type: {:?}", message.type_);
                    // Record metric for unknown message types
                    increment_counter("protocol.messages.unknown", None);
                    
                    // Try to handle with generic handler
                    if let Ok(handler_guard) = message_handler_clone.safe_lock() {
                        if let Some(handler) = handler_guard.as_ref() {
                            handler.handle_message(message)
                        } else {
                            Err(McpError::ProtocolError("No message handler available".to_string()))
                        }
                    } else {
                        Err(McpError::ProtocolError("Failed to lock message handler".to_string()))
                    }
                }),
            );
        } else {
            error!("Failed to acquire write lock on message_router");
        }
    }
    
    /// Start outgoing message processor
    fn start_outgoing_processor(&self) {
        let outgoing_queue = self.outgoing_queue.clone();
        let client = self.client.clone();
        let is_active = self.is_active.clone();
        
        tokio::spawn(async move {
            loop {
                // Check if we're still active
                let is_still_active = match is_active.safe_lock() {
                    Ok(guard) => *guard,
                    Err(e) => {
                        error!("Failed to read is_active flag: {}", e);
                        // Assume we're shutting down if we can't read the flag
                        false
                    }
                };
                
                if !is_still_active {
                    debug!("Outgoing processor shutting down");
                    break;
                }
                
                // Get messages from queue
                let messages = {
                    match outgoing_queue.safe_lock() {
                        Ok(mut queue) => {
                            let messages = queue.clone();
                            queue.clear();
                            messages
                        },
                        Err(e) => {
                            error!("Failed to lock outgoing queue: {}", e);
                            Vec::new()
                        }
                    }
                };
                
                // Send messages
                for message in messages {
                    match serde_json::to_string(&message) {
                        Ok(message_str) => {
                            if let Err(e) = client.send_raw(message_str).await {
                                error!("Failed to send message: {}", e);
                                increment_counter("protocol.messages.send_error", None);
                            } else {
                                debug!("Sent message: {:?}", message.type_);
                                increment_counter("protocol.messages.sent", None);
                                
                                // Record message size metrics
                                if let Ok(message_size) = serde_json::to_string(&message) {
                                    record_histogram("protocol.message.size", message_size.len() as f64, None);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize message: {}", e);
                            increment_counter("protocol.messages.serialize_error", None);
                        }
                    }
                }
                
                // Sleep to avoid busy loop
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }
    
    /// Create a new session
    fn create_session(&self) -> Arc<Session> {
        let session = self.session_manager.create_session(None);
        
        // Create a message handler for the session
        let (handler, receiver) = SessionMessageHandler::new(session.clone());
        {
            if let Ok(mut handler_guard) = self.message_handler.safe_lock() {
                *handler_guard = Some(handler);
            } else {
                error!("Failed to store message handler for new session");
            }
        }
        {
            if let Ok(mut receiver_guard) = self.event_receiver.safe_lock() {
                *receiver_guard = Some(receiver);
            } else {
                error!("Failed to store event receiver for new session");
            }
        }
        
        // Store session
        if let Ok(mut session_guard) = self.current_session.safe_lock() {
            *session_guard = Some(session.clone());
        } else {
            error!("Failed to store new session");
        }
        
        // Record session creation metric
        increment_counter("protocol.session.created", None);
        
        session
    }
    
    /// Recover an existing session
    async fn recover_session(&self, session_id: &str) -> Result<Arc<Session>, McpError> {
        match self.session_manager.recover_session(session_id) {
            Ok(session) => {
                // Create a message handler for the session
                let (handler, receiver) = SessionMessageHandler::new(session.clone());
                {
                    if let Ok(mut handler_guard) = self.message_handler.safe_lock() {
                        *handler_guard = Some(handler);
                    } else {
                        return Err(McpError::InternalError("Failed to store message handler".to_string()));
                    }
                }
                {
                    if let Ok(mut receiver_guard) = self.event_receiver.safe_lock() {
                        *receiver_guard = Some(receiver);
                    } else {
                        return Err(McpError::InternalError("Failed to store event receiver".to_string()));
                    }
                }
                
                // Store session
                if let Ok(mut session_guard) = self.current_session.safe_lock() {
                    *session_guard = Some(session.clone());
                } else {
                    return Err(McpError::InternalError("Failed to store session".to_string()));
                }
                
                // Record session recovery metric
                increment_counter("protocol.session.recovered", None);
                
                // Send session recovery message to server
                self.send_session_recovery(session_id).await?;
                
                Ok(session)
            }
            Err(e) => {
                error!("Failed to recover session {}: {}", session_id, e);
                increment_counter("protocol.session.recovery_failed", None);
                Err(McpError::SessionError(format!("Failed to recover session: {}", e)))
            }
        }
    }
    
    /// Send session recovery message to server
    async fn send_session_recovery(&self, session_id: &str) -> Result<(), McpError> {
        let recovery_message = McpMessage {
            id: Uuid::new_v4().to_string(),
            version: "v1".to_string(),
            type_: McpMessageType::SessionRecovery,
            payload: McpMessagePayload::SessionRecovery {
                session_id: session_id.to_string(),
            },
        };
        
        // Set up response channel
        let (tx, rx) = oneshot::channel();
        
        // Register response channel
        {
            if let Ok(handler_guard) = self.message_handler.safe_lock() {
                if let Some(handler) = handler_guard.as_ref() {
                    handler.register_response_channel(recovery_message.id.clone(), tx);
                } else {
                    return Err(McpError::ProtocolError(
                        "No message handler available".to_string(),
                    ));
                }
            } else {
                return Err(McpError::InternalError("Failed to lock message handler".to_string()));
            }
        }
        
        // Send recovery request
        self.client
            .send_raw(serde_json::to_string(&recovery_message)?)
            .await?;
        
        // Wait for response with timeout
        match timeout(self.config.connection_timeout, rx).await {
            Ok(result) => match result {
                Ok(response) => match response {
                    Ok(msg) => {
                        if let McpMessagePayload::SessionRecoveryResponse { success, error } = msg.payload {
                            if success {
                                info!("Successfully recovered session {}", session_id);
                                increment_counter("protocol.session.recovery_success", None);
                                Ok(())
                            } else {
                                let error_msg = error.unwrap_or_else(|| "Unknown error".to_string());
                                warn!("Failed to recover session {}: {}", session_id, error_msg);
                                increment_counter("protocol.session.recovery_rejected", None);
                                Err(McpError::SessionError(format!("Session recovery failed: {}", error_msg)))
                            }
                        } else {
                            Err(McpError::ProtocolError(
                                "Unexpected response type".to_string(),
                            ))
                        }
                    }
                    Err(e) => Err(e),
                },
                Err(_) => Err(McpError::ProtocolError(
                    "Response channel closed".to_string(),
                )),
            },
            Err(_) => Err(McpError::Timeout(self.config.connection_timeout)),
        }
    }
    
    /// Persist the current session
    async fn persist_current_session(&self) -> Result<(), McpError> {
        let session_guard = match self.current_session.safe_lock() {
            Ok(guard) => guard,
            Err(e) => return Err(McpError::InternalError(format!("Failed to read current session: {}", e))),
        };
        
        if let Some(session) = session_guard.as_ref() {
            if let Err(e) = self.session_manager.persist_session(session) {
                error!("Failed to persist session {}: {}", session.id, e);
                increment_counter("protocol.session.persistence_failed", None);
                return Err(McpError::SessionError(format!("Failed to persist session: {}", e)));
            }
            increment_counter("protocol.session.persisted", None);
            Ok(())
        } else {
            Err(McpError::SessionError("No active session to persist".to_string()))
        }
    }
    
    /// Authenticate with the server
    async fn authenticate(&self) -> Result<(), McpError> {
        // Record authentication attempt metric
        increment_counter("protocol.auth.attempt", None);
        
        // Generate authentication message
        let auth_message = McpMessage {
            id: Uuid::new_v4().to_string(),
            version: "v1".to_string(),
            type_: McpMessageType::AuthRequest,
            payload: McpMessagePayload::AuthRequest {
                api_key: self.config.api_key.clone(),
                organization_id: self.config.organization_id.clone(),
            },
        };
        
        // Set up response channel
        let (tx, rx) = oneshot::channel();
        
        // Register response channel
        {
            if let Ok(handler_guard) = self.message_handler.safe_lock() {
                if let Some(handler) = handler_guard.as_ref() {
                    handler.register_response_channel(auth_message.id.clone(), tx);
                } else {
                    return Err(McpError::ProtocolError(
                        "No message handler available".to_string(),
                    ));
                }
            } else {
                return Err(McpError::InternalError("Failed to lock message handler".to_string()));
            }
        }
        
        // Send authentication request
        self.client
            .send_raw(serde_json::to_string(&auth_message)?)
            .await?;
        
        // Wait for response with timeout
        match timeout(self.config.connection_timeout, rx).await {
            Ok(result) => match result {
                Ok(response) => match response {
                    Ok(msg) => {
                        if let McpMessagePayload::AuthResponse { success, session_id } = msg.payload {
                            if success {
                                // Update connection state
                                if let Ok(mut state) = self.connection_state.safe_lock() {
                                    *state = ConnectionState::Authenticated;
                                } else {
                                    return Err(McpError::InternalError("Failed to update connection state".to_string()));
                                }
                                
                                // Update connection status
                                if let Ok(mut status) = self.status.safe_lock() {
                                    *status = ConnectionStatus::Connected;
                                } else {
                                    return Err(McpError::InternalError("Failed to update connection status".to_string()));
                                }
                                
                                // Start heartbeat task
                                self.start_heartbeat_task();
                                
                                // Reset reconnection attempts
                                if let Ok(mut attempts) = self.reconnect_attempts.safe_lock() {
                                    *attempts = 0;
                                } else {
                                    warn!("Failed to reset reconnection attempts");
                                }
                                
                                // Record successful authentication
                                increment_counter("protocol.auth.success", None);
                                
                                Ok(())
                            } else {
                                // Record failed authentication
                                increment_counter("protocol.auth.failed", None);
                                
                                Err(McpError::AuthenticationFailed(
                                    "Authentication failed".to_string(),
                                ))
                            }
                        } else {
                            Err(McpError::ProtocolError(
                                "Unexpected response type".to_string(),
                            ))
                        }
                    }
                    Err(e) => Err(e),
                },
                Err(_) => Err(McpError::ProtocolError(
                    "Response channel closed".to_string(),
                )),
            },
            Err(_) => {
                // Record timeout
                increment_counter("protocol.auth.timeout", None);
                
                Err(McpError::Timeout(self.config.connection_timeout))
            }
        }
    }

    /// Check if the handler is connected and authenticated
    fn is_authenticated(&self) -> bool {
        match self.connection_state.safe_lock() {
            Ok(state) => matches!(*state, ConnectionState::Authenticated),
            Err(e) => {
                error!("Failed to acquire read lock on connection_state: {}", e);
                false // If we can't read the state, assume not authenticated for safety
            }
        }
    }
    
    /// Get connection state as string
    pub fn get_connection_state_str(&self) -> String {
        if let Ok(state) = self.connection_state.safe_lock() {
            match *state {
                ConnectionState::Disconnected => "Disconnected".to_string(),
                ConnectionState::Connecting => "Connecting".to_string(),
                ConnectionState::Connected => "Connected".to_string(),
                ConnectionState::Authenticated => "Authenticated".to_string(),
                ConnectionState::Error(ref msg) => format!("Error: {}", msg),
            }
        } else {
            "Unknown (lock error)".to_string()
        }
    }
    
    // Other methods remain the same...
    
    /// Start the message handling task
    fn start_message_handler(&self) {
        let client_clone = self.client.clone();
        let message_router = self.message_router.clone();
        let connection_state_clone = self.connection_state.clone();
        let status_clone = self.status.clone();
        let config_clone = self.config.clone();
        let reconnect_handler = Arc::new(move || {
            let connection_state = connection_state_clone.clone();
            let status = status_clone.clone();
            let config = config_clone.clone();
            
            tokio::spawn(async move {
                // Update connection state
                {
                    if let Ok(mut state) = connection_state.safe_lock() {
                        *state = ConnectionState::Error("Connection lost, attempting to reconnect".to_string());
                    } else {
                        error!("Failed to update connection state during reconnection handler");
                    }
                }
                {
                    if let Ok(mut status) = status.safe_lock() {
                        *status = ConnectionStatus::Reconnecting;
                    } else {
                        error!("Failed to update connection status during reconnection handler");
                    }
                }
                
                // Implement exponential backoff for reconnection
                // This will be completed in the handle_reconnection method
            });
        });
        
        tokio::spawn(async move {
            loop {
                // Check connection state
                let should_continue = {
                    if let Ok(state) = connection_state_clone.safe_lock() {
                        *state != ConnectionState::Disconnected
                    } else {
                        error!("Failed to read connection state in message handler");
                        false // Exit loop if we can't read the state
                    }
                };
                
                if !should_continue {
                    break;
                }
                
                // Receive message from WebSocket
                match client_clone.receive_raw().await {
                    Ok(message_str) => {
                        // Record message received metric
                        increment_counter("protocol.messages.received", None);
                        
                        // Record message size metric
                        record_histogram("protocol.message.received_size", message_str.len() as f64, None);
                        
                        // Parse message
                        match serde_json::from_str::<McpMessage>(&message_str) {
                            Ok(message) => {
                                // Record message type metric
                                let metric_name = format!("protocol.messages.type.{:?}", message.type_);
                                increment_counter(&metric_name.to_lowercase(), None);
                                
                                // Route message to appropriate handler
                                if let Ok(router) = message_router.safe_lock() {
                                    let handler = router.get(&message.type_).or_else(|| router.get(&McpMessageType::Unknown));
                                    
                                    if let Some(handler) = handler {
                                        if let Err(e) = handler(message.clone()) {
                                            error!("Error handling message: {}", e);
                                            increment_counter("protocol.messages.handler_error", None);
                                        }
                                    } else {
                                        error!("No handler found for message type: {:?}", message.type_);
                                        increment_counter("protocol.messages.no_handler", None);
                                    }
                                } else {
                                    error!("Failed to acquire lock on message router");
                                }
                            }
                            Err(e) => {
                                error!("Error parsing message: {}", e);
                                increment_counter("protocol.messages.parse_error", None);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        increment_counter("protocol.connection.error", None);
                        
                        // Update connection state and status
                        {
                            if let Ok(mut state) = connection_state_clone.safe_lock() {
                                *state = ConnectionState::Error(e.to_string());
                            } else {
                                error!("Failed to update connection state after error");
                            }
                        }
                        {
                            if let Ok(mut status) = status_clone.safe_lock() {
                                *status = ConnectionStatus::ConnectionError(e.to_string());
                            } else {
                                error!("Failed to update connection status after error");
                            }
                        }
                        
                        // Attempt to reconnect if enabled
                        if config_clone.auto_reconnect {
                            reconnect_handler();
                        }
                        
                        break;
                    }
                }
            }
        });
    }
    
    /// Start heartbeat task
    fn start_heartbeat_task(&self) {
        let client_clone = self.client.clone();
        let current_session_clone = self.current_session.clone();
        let connection_state_clone = self.connection_state.clone();
        let config_clone = self.config.clone();
        
        tokio::spawn(async move {
            let heartbeat_interval = Duration::from_secs(30); // 30 seconds between heartbeats
            let mut missed_heartbeats = 0;
            let max_missed_heartbeats = 3; // Allow up to 3 missed heartbeats before considering connection lost
            
            loop {
                // Sleep for heartbeat interval
                tokio::time::sleep(heartbeat_interval).await;
                
                // Check connection state
                let should_continue = {
                    if let Ok(state) = connection_state_clone.safe_lock() {
                        *state == ConnectionState::Authenticated
                    } else {
                        error!("Failed to read connection state in heartbeat task");
                        false
                    }
                };
                
                if !should_continue {
                    break;
                }
                
                // Generate heartbeat message
                let session_guard = match current_session_clone.safe_lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        error!("Failed to read current session in heartbeat task: {}", e);
                        continue; // Skip this iteration and try again later
                    }
                };
                
                if let Some(session) = session_guard.as_ref() {
                    let heartbeat = session.generate_heartbeat();
                    
                    // Send heartbeat
                    if let Err(e) = client_clone.send_raw(serde_json::to_string(&heartbeat).unwrap()).await {
                        error!("Error sending heartbeat: {}", e);
                        increment_counter("protocol.heartbeat.error", None);
                        
                        // Increment missed heartbeats
                        missed_heartbeats += 1;
                        
                        // Check if we've missed too many heartbeats
                        if missed_heartbeats >= max_missed_heartbeats {
                            error!("Too many missed heartbeats ({}/{}), considering connection lost", 
                                   missed_heartbeats, max_missed_heartbeats);
                            
                            // Update connection state
                            if let Ok(mut state) = connection_state_clone.safe_lock() {
                                *state = ConnectionState::Error("Connection lost due to missed heartbeats".to_string());
                            } else {
                                error!("Failed to update connection state after missed heartbeats");
                            }
                            
                            // Attempt to reconnect if enabled
                            if config_clone.auto_reconnect {
                                // Connection lost, will trigger reconnection in message handler
                                break;
                            }
                        }
                    } else {
                        // Reset missed heartbeats counter on successful sending
                        missed_heartbeats = 0;
                        
                        // Record heartbeat sent metric
                        increment_counter("protocol.heartbeat.sent", None);
                    }
                } else {
                    break;
                }
            }
        });
    }
}

#[async_trait]
impl ProtocolHandler for McpProtocolHandler {
    fn protocol_name(&self) -> &'static str {
        "Model Context Protocol"
    }
    
    fn connection_status(&self) -> ConnectionStatus {
        if let Ok(status) = self.status.safe_lock() {
            status.clone()
        } else {
            error!("Failed to read connection status");
            ConnectionStatus::Unknown("Failed to read status".to_string())
        }
    }
    
    async fn connect(&self) -> Result<(), String> {
        // Initialize the protocol handler if not already done
        {
            let is_active = match self.is_active.safe_lock() {
                Ok(guard) => *guard,
                Err(e) => {
                    error!("Failed to read is_active flag: {}", e);
                    false
                }
            };
            
            if !is_active {
                self.initialize();
            }
        }
        
        // Record connection attempt
        increment_counter("protocol.connection.attempt", None);
        
        // Update connection state and status
        {
            if let Ok(mut state) = self.connection_state.safe_lock() {
                *state = ConnectionState::Connecting;
            } else {
                return Err("Failed to update connection state".to_string());
            }
        }
        {
            if let Ok(mut status) = self.status.safe_lock() {
                *status = ConnectionStatus::Connecting;
            } else {
                return Err("Failed to update connection status".to_string());
            }
        }
        
        // Connect to WebSocket
        match self.client.connect().await {
            Ok(_) => {
                // Update connection state
                if let Ok(mut state) = self.connection_state.safe_lock() {
                    *state = ConnectionState::Connected;
                } else {
                    return Err("Failed to update connection state after connect".to_string());
                }
                
                // Create a new session
                self.create_session();
                
                // Start message handler
                self.start_message_handler();
                
                // Authenticate
                match self.authenticate().await {
                    Ok(_) => {
                        // Record successful connection
                        increment_counter("protocol.connection.success", None);
                        
                        Ok(())
                    }
                    Err(e) => {
                        if let Ok(mut status) = self.status.safe_lock() {
                            *status = ConnectionStatus::AuthFailed;
                        } else {
                            error!("Failed to update connection status after auth failure");
                        }
                        
                        // Record authentication failure
                        increment_counter("protocol.connection.auth_failed", None);
                        
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                // Update connection state and status
                {
                    if let Ok(mut state) = self.connection_state.safe_lock() {
                        *state = ConnectionState::Error(e.to_string());
                    } else {
                        error!("Failed to update connection state after connection error");
                    }
                }
                {
                    if let Ok(mut status) = self.status.safe_lock() {
                        *status = ConnectionStatus::ConnectionError(e.to_string());
                    } else {
                        error!("Failed to update connection status after connection error");
                    }
                }
                
                // Record connection failure
                increment_counter("protocol.connection.failed", None);
                
                Err(e.to_string())
            }
        }
    }
    
    async fn disconnect(&self) -> Result<(), String> {
        // Record disconnect attempt
        increment_counter("protocol.disconnect.attempt", None);
        
        // Persist current session if possible
        if let Err(e) = self.persist_current_session().await {
            warn!("Failed to persist session during disconnect: {}", e);
        }
        
        // Update connection state
        if let Ok(mut state) = self.connection_state.safe_lock() {
            *state = ConnectionState::Disconnected;
        } else {
            return Err("Failed to update connection state for disconnect".to_string());
        }
        
        // Cancel all streaming requests
        {
            if let Ok(mut streaming_requests) = self.streaming_requests.safe_lock() {
                for (id, sender) in streaming_requests.drain() {
                    debug!("Canceling streaming request {} due to disconnect", id);
                    let _ = sender.send(Err(MessageError::ConnectionClosed)).await;
                }
            } else {
                warn!("Failed to cancel streaming requests during disconnect");
            }
        }
        
        // Cancel all pending requests
        {
            if let Ok(handler_guard) = self.message_handler.safe_lock() {
                if let Some(handler) = handler_guard.as_ref() {
                    handler.cancel_all_requests(McpError::ConnectionClosed);
                }
            } else {
                warn!("Failed to cancel pending requests during disconnect");
            }
        }
        
        // Disconnect WebSocket
        match self.client.disconnect().await {
            Ok(_) => {
                // Update connection status
                if let Ok(mut status) = self.status.safe_lock() {
                    *status = ConnectionStatus::Disconnected;
                } else {
                    warn!("Failed to update connection status after disconnect");
                }
                
                // Set active flag to false
                if let Ok(mut is_active) = self.is_active.safe_lock() {
                    *is_active = false;
                } else {
                    warn!("Failed to update active flag after disconnect");
                }
                
                // Record successful disconnect
                increment_counter("protocol.disconnect.success", None);
                
                Ok(())
            }
            Err(e) => {
                // Record disconnect failure
                increment_counter("protocol.disconnect.error", None);
                
                Err(e.to_string())
            }
        }
    }
    
    async fn send_message(&self, message: Message) -> Result<(), MessageError> {
        // First verify authenticated state directly
        if !self.is_authenticated() {
            increment_counter("protocol.message.error.not_authenticated", None);
            log::warn!("Attempted to send message while not authenticated.");
            return Err(MessageError::AuthError("Client not authenticated".to_string()));
        }
        
        // Validate message
        self.validate_message(&message)?;
        
        // Convert to MCP message
        let mcp_message = McpClient::convert_to_mcp_message(message)?;
        
        // Queue message for sending
        if let Ok(mut queue) = self.outgoing_queue.safe_lock() {
            queue.push(mcp_message.clone());
            
            // Record message queued metric
            increment_counter("protocol.message.queued", None);
            
            Ok(())
        } else {
            error!("Failed to acquire lock on outgoing queue");
            Err(MessageError::InternalError("Failed to queue message for sending".to_string()))
        }
    }
    
    // Other required methods remain mostly the same, updated to use safe_lock...
    
    /// Stream a message and get chunks
    async fn stream_message(&self, message: Message) -> Result<Receiver<Result<Message, MessageError>>, MessageError> {
        // Check authentication status directly
        if !self.is_authenticated() {
            log::warn!("Attempted to stream message while not authenticated.");
            return Err(MessageError::AuthError("Client not authenticated".to_string()));
        }
        
        // Create streaming channel
        self.create_streaming_channel(message).await
    }
    
    /// Cancel a streaming request
    async fn cancel_stream(&self, streaming_id: &str) -> Result<(), MessageError> {
        // Check authentication status directly
        if !self.is_authenticated() {
            log::warn!("Attempted to cancel stream while not authenticated.");
            return Err(MessageError::AuthError("Client not authenticated".to_string()));
        }
        
        // Cancel streaming
        self.cancel_streaming(streaming_id).await
    }
    
    /// Send a message and wait for a response
    async fn send_and_receive(&self, message: Message) -> Result<Message, MessageError> {
        // Check authentication status directly
        if !self.is_authenticated() {
            log::warn!("Attempted to send and receive message while not authenticated.");
            return Err(MessageError::AuthError("Client not authenticated".to_string()));
        }
        
        // Process synchronous message
        self.process_sync_message(message).await
    }
    
    // Other required methods...
    async fn receive_messages(&self) -> Result<Vec<Message>, MessageError> {
        // Implementation remains similar with safe_lock usage
        // Not fully shown to keep the response concise
        
        // Check authentication status directly
        if !self.is_authenticated() {
            return Err(MessageError::AuthError("Client not authenticated".to_string()));
        }
        
        // Get event receiver
        if let Ok(mut event_receiver_guard) = self.event_receiver.safe_lock() {
            if let Some(receiver) = event_receiver_guard.as_mut() {
                let mut messages = Vec::new();
                
                // Poll for messages without blocking
                while let Ok(Some(mcp_message)) = tokio::time::timeout(Duration::from_millis(10), receiver.recv()).await {
                    // Convert MCP message to application message
                    if let Ok(app_message) = McpClient::convert_from_mcp_message(mcp_message) {
                        messages.push(app_message);
                    }
                }
                
                // Record number of messages received
                if !messages.is_empty() {
                    record_gauge("protocol.receive_messages.count", messages.len() as f64, None);
                }
                
                Ok(messages)
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(MessageError::InternalError("Failed to lock event receiver".to_string()))
        }
    }
}