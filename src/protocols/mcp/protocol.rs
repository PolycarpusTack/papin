use crate::models::messages::{Message, MessageError};
use crate::protocols::mcp::error::McpError;
use crate::protocols::mcp::message::{McpMessage, McpMessagePayload, McpResponseMessage};
use crate::protocols::mcp::session::{Session, SessionManager, SessionMessageHandler};
use crate::protocols::mcp::types::{McpCompletionRequest, McpMessageRole, McpMessageType};
use crate::protocols::mcp::{McpClient, McpConfig};
use crate::protocols::{ConnectionStatus, ProtocolHandler};
use crate::observability::metrics::{increment_counter, record_gauge, record_histogram, time_operation};
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
        let mut is_active = self.is_active.write().unwrap();
        *is_active = true;
    }
    
    /// Set up message routing table
    fn setup_message_routing(&self) {
        let mut router = self.message_router.lock().unwrap();
        let message_handler = self.message_handler.clone();
        let streaming_requests = self.streaming_requests.clone();
        
        // Set up routing for each message type
        router.insert(
            McpMessageType::CompletionResponse,
            Box::new(move |message| {
                let handler_guard = message_handler.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.handle_message(message)
                } else {
                    Err(McpError::ProtocolError("No message handler available".to_string()))
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
                let handler_guard = message_handler_clone.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.handle_message(message.clone())?;
                }
                
                // Then, handle streaming delivery
                if let McpMessagePayload::StreamingMessage { streaming_id, chunk, is_final } = &message.payload {
                    let mut streaming_requests = streaming_requests_clone.lock().unwrap();
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
                let handler_guard = message_handler_clone.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.handle_message(message.clone())?;
                }
                
                // Then, handle streaming cleanup
                if let McpMessagePayload::StreamingEnd { streaming_id } = &message.payload {
                    let mut streaming_requests = streaming_requests_clone.lock().unwrap();
                    if streaming_requests.remove(streaming_id).is_some() {
                        debug!("Removed streaming request {}", streaming_id);
                        increment_counter("protocol.streaming.ended", None);
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
                let handler_guard = message_handler_clone.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.handle_message(message)
                } else {
                    Err(McpError::ProtocolError("No message handler available".to_string()))
                }
            }),
        );
        
        // Set up routing for ping/pong
        let connection_state_clone = self.connection_state.clone();
        router.insert(
            McpMessageType::Pong,
            Box::new(move |_| {
                // Just update connection state
                let state = connection_state_clone.read().unwrap();
                if *state == ConnectionState::Authenticated {
                    debug!("Received pong message, connection is active");
                    // Record heartbeat success metric
                    increment_counter("protocol.heartbeat.success", None);
                } else {
                    warn!("Received pong message but connection is not authenticated");
                }
                Ok(())
            }),
        );
        
        // Set up routing for auth response
        let message_handler_clone = self.message_handler.clone();
        router.insert(
            McpMessageType::AuthResponse,
            Box::new(move |message| {
                let handler_guard = message_handler_clone.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.handle_message(message)
                } else {
                    Err(McpError::ProtocolError("No message handler available".to_string()))
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
                let handler_guard = message_handler_clone.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.handle_message(message)
                } else {
                    Err(McpError::ProtocolError("No message handler available".to_string()))
                }
            }),
        );
    }
    
    /// Start outgoing message processor
    fn start_outgoing_processor(&self) {
        let outgoing_queue = self.outgoing_queue.clone();
        let client = self.client.clone();
        let is_active = self.is_active.clone();
        
        tokio::spawn(async move {
            while *is_active.read().unwrap() {
                // Get messages from queue
                let messages = {
                    let mut queue = outgoing_queue.lock().unwrap();
                    let messages = queue.clone();
                    queue.clear();
                    messages
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
            let mut handler_guard = self.message_handler.lock().unwrap();
            *handler_guard = Some(handler);
        }
        {
            let mut receiver_guard = self.event_receiver.lock().unwrap();
            *receiver_guard = Some(receiver);
        }
        
        // Store session
        let mut session_guard = self.current_session.write().unwrap();
        *session_guard = Some(session.clone());
        
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
                    let mut handler_guard = self.message_handler.lock().unwrap();
                    *handler_guard = Some(handler);
                }
                {
                    let mut receiver_guard = self.event_receiver.lock().unwrap();
                    *receiver_guard = Some(receiver);
                }
                
                // Store session
                let mut session_guard = self.current_session.write().unwrap();
                *session_guard = Some(session.clone());
                
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
            let handler_guard = self.message_handler.lock().unwrap();
            if let Some(handler) = handler_guard.as_ref() {
                handler.register_response_channel(recovery_message.id.clone(), tx);
            } else {
                return Err(McpError::ProtocolError(
                    "No message handler available".to_string(),
                ));
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
        let session_guard = self.current_session.read().unwrap();
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
            let handler_guard = self.message_handler.lock().unwrap();
            if let Some(handler) = handler_guard.as_ref() {
                handler.register_response_channel(auth_message.id.clone(), tx);
            } else {
                return Err(McpError::ProtocolError(
                    "No message handler available".to_string(),
                ));
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
                                let mut state = self.connection_state.write().unwrap();
                                *state = ConnectionState::Authenticated;
                                
                                // Update connection status
                                let mut status = self.status.write().unwrap();
                                *status = ConnectionStatus::Connected;
                                
                                // Start heartbeat task
                                self.start_heartbeat_task();
                                
                                // Reset reconnection attempts
                                let mut attempts = self.reconnect_attempts.write().unwrap();
                                *attempts = 0;
                                
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
                    let mut state = connection_state.write().unwrap();
                    *state = ConnectionState::Error("Connection lost, attempting to reconnect".to_string());
                }
                {
                    let mut status = status.write().unwrap();
                    *status = ConnectionStatus::Reconnecting;
                }
                
                // Implement exponential backoff for reconnection
                // This will be completed in the handle_reconnection method
            });
        });
        
        tokio::spawn(async move {
            loop {
                // Check connection state
                {
                    let state = connection_state_clone.read().unwrap();
                    if *state == ConnectionState::Disconnected {
                        break;
                    }
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
                                let router = message_router.lock().unwrap();
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
                            let mut state = connection_state_clone.write().unwrap();
                            *state = ConnectionState::Error(e.to_string());
                        }
                        {
                            let mut status = status_clone.write().unwrap();
                            *status = ConnectionStatus::ConnectionError(e.to_string());
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
                {
                    let state = connection_state_clone.read().unwrap();
                    if *state != ConnectionState::Authenticated {
                        break;
                    }
                }
                
                // Generate heartbeat message
                let session_guard = current_session_clone.read().unwrap();
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
                            let mut state = connection_state_clone.write().unwrap();
                            *state = ConnectionState::Error("Connection lost due to missed heartbeats".to_string());
                            
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
    
    /// Validate message before sending
    fn validate_message(&self, message: &Message) -> Result<(), MessageError> {
        // Record validation metric
        increment_counter("protocol.validation.attempt", None);
        
        // Check message ID
        if message.id.is_empty() {
            increment_counter("protocol.validation.error.empty_id", None);
            return Err(MessageError::ValidationError("Message ID cannot be empty".to_string()));
        }
        
        // Check message content
        if message.content.parts.is_empty() {
            increment_counter("protocol.validation.error.empty_content", None);
            return Err(MessageError::ValidationError("Message content cannot be empty".to_string()));
        }
        
        // Validate by message role
        match message.role {
            crate::models::messages::MessageRole::User => {
                // User messages must have text or image content
                let has_valid_content = message.content.parts.iter().any(|part| {
                    matches!(
                        part,
                        crate::models::messages::ContentType::Text { .. } |
                        crate::models::messages::ContentType::Image { .. }
                    )
                });
                
                if !has_valid_content {
                    increment_counter("protocol.validation.error.user_invalid_content", None);
                    return Err(MessageError::ValidationError(
                        "User messages must have text or image content".to_string(),
                    ));
                }
            }
            crate::models::messages::MessageRole::Assistant => {
                // Assistant messages must have text content
                let has_text = message.content.parts.iter().any(|part| {
                    matches!(part, crate::models::messages::ContentType::Text { .. })
                });
                
                if !has_text {
                    increment_counter("protocol.validation.error.assistant_no_text", None);
                    return Err(MessageError::ValidationError(
                        "Assistant messages must have text content".to_string(),
                    ));
                }
            }
            crate::models::messages::MessageRole::System => {
                // System messages must only have text content
                let all_text = message.content.parts.iter().all(|part| {
                    matches!(part, crate::models::messages::ContentType::Text { .. })
                });
                
                if !all_text {
                    increment_counter("protocol.validation.error.system_non_text", None);
                    return Err(MessageError::ValidationError(
                        "System messages must only have text content".to_string(),
                    ));
                }
            }
            crate::models::messages::MessageRole::Tool => {
                // Tool messages must have tool result content
                let has_tool_result = message.content.parts.iter().any(|part| {
                    matches!(part, crate::models::messages::ContentType::ToolResult { .. })
                });
                
                if !has_tool_result {
                    increment_counter("protocol.validation.error.tool_no_result", None);
                    return Err(MessageError::ValidationError(
                        "Tool messages must have tool result content".to_string(),
                    ));
                }
            }
        }
        
        // Validate message size
        if let Ok(json) = serde_json::to_string(message) {
            if json.len() > self.config.max_message_size {
                increment_counter("protocol.validation.error.message_too_large", None);
                return Err(MessageError::ValidationError(
                    format!("Message exceeds maximum size of {} bytes", self.config.max_message_size)
                ));
            }
        }
        
        // Record successful validation
        increment_counter("protocol.validation.success", None);
        
        Ok(())
    }
    
    /// Create a streaming channel
    async fn create_streaming_channel(
        &self,
        message: Message,
    ) -> Result<Receiver<Result<Message, MessageError>>, MessageError> {
        // Measure operation duration
        time_operation!("protocol.streaming.setup", None, {
            // Validate message
            self.validate_message(&message)?;
            
            // Generate streaming ID
            let streaming_id = Uuid::new_v4().to_string();
            
            // Create streaming channel
            let (tx, rx) = mpsc::channel(32);
            
            // Store channel
            {
                let mut streaming_requests = self.streaming_requests.lock().unwrap();
                streaming_requests.insert(streaming_id.clone(), tx);
                
                // Record active streaming channels metric
                record_gauge("protocol.streaming.active_channels", streaming_requests.len() as f64, None);
            }
            
            // Convert to MCP message
            let mut mcp_message = McpClient::convert_to_mcp_message(message)?;
            
            // Set streaming parameters
            if let McpMessagePayload::CompletionRequest(ref mut req) = mcp_message.payload {
                req.stream = Some(true);
                req.streaming_id = Some(streaming_id.clone());
            }
            
            // Send message
            match self.client
                .send_raw(serde_json::to_string(&mcp_message)?)
                .await
            {
                Ok(_) => {
                    // Record streaming start metric
                    increment_counter("protocol.streaming.started", None);
                },
                Err(e) => {
                    // Clean up streaming request on error
                    let mut streaming_requests = self.streaming_requests.lock().unwrap();
                    streaming_requests.remove(&streaming_id);
                    
                    // Record streaming error metric
                    increment_counter("protocol.streaming.start_error", None);
                    
                    return Err(MessageError::NetworkError(e.to_string()));
                }
            }
            
            Ok(rx)
        })
    }
    
    /// Cancel streaming request
    async fn cancel_streaming(&self, streaming_id: &str) -> Result<(), MessageError> {
        // Record cancel attempt metric
        increment_counter("protocol.streaming.cancel_attempt", None);
        
        // Remove channel
        let channel_existed = {
            let mut streaming_requests = self.streaming_requests.lock().unwrap();
            streaming_requests.remove(streaming_id).is_some()
        };
        
        if !channel_existed {
            warn!("Attempted to cancel non-existent streaming request: {}", streaming_id);
            increment_counter("protocol.streaming.cancel_nonexistent", None);
            return Err(MessageError::InvalidRequest(format!("Streaming request {} not found", streaming_id)));
        }
        
        // Create cancel message
        let cancel_message = McpMessage {
            id: Uuid::new_v4().to_string(),
            version: "v1".to_string(),
            type_: McpMessageType::CancelStream,
            payload: McpMessagePayload::CancelStream {
                streaming_id: streaming_id.to_string(),
            },
        };
        
        // Send cancel message
        match self.client
            .send_raw(serde_json::to_string(&cancel_message)?)
            .await
        {
            Ok(_) => {
                // Record successful cancellation
                increment_counter("protocol.streaming.cancelled", None);
                Ok(())
            }
            Err(e) => {
                // Record failed cancellation
                increment_counter("protocol.streaming.cancel_error", None);
                Err(MessageError::NetworkError(e.to_string()))
            }
        }
    }
    
    /// Initialize reconnection process
    async fn handle_reconnection(&self) -> Result<(), McpError> {
        // Record reconnection attempt metric
        increment_counter("protocol.reconnection.attempt", None);
        
        // Check if reconnection is enabled
        if !self.config.auto_reconnect {
            return Err(McpError::ConnectionError(
                "Automatic reconnection is disabled".to_string(),
            ));
        }
        
        // Check reconnection attempts
        {
            let attempts = self.reconnect_attempts.read().unwrap();
            if *attempts >= self.config.max_reconnect_attempts {
                increment_counter("protocol.reconnection.max_attempts_reached", None);
                return Err(McpError::ConnectionError(
                    format!("Maximum reconnection attempts reached ({})", self.config.max_reconnect_attempts)
                ));
            }
        }
        
        // Update reconnection attempts
        let current_attempt;
        {
            let mut attempts = self.reconnect_attempts.write().unwrap();
            *attempts += 1;
            current_attempt = *attempts;
            
            info!("Reconnection attempt {}/{}", current_attempt, self.config.max_reconnect_attempts);
        }
        
        // Calculate backoff delay
        let backoff_delay = {
            // Exponential backoff: base_delay * 2^attempt with jitter
            let base_ms = self.config.reconnect_backoff.as_millis() as u64;
            let exp_factor = 2u64.pow(current_attempt as u32 - 1);
            let delay_ms = base_ms * exp_factor;
            
            // Add jitter (±20%)
            let jitter_factor = 0.8 + (rand::random::<f64>() * 0.4); // 0.8 to 1.2
            let jittered_delay_ms = (delay_ms as f64 * jitter_factor) as u64;
            
            Duration::from_millis(jittered_delay_ms.min(30000)) // Cap at 30 seconds
        };
        
        // Update last reconnection attempt
        {
            let mut last_attempt = self.last_reconnect_attempt.write().unwrap();
            *last_attempt = Some(Instant::now());
            
            // Record backoff delay metric
            record_histogram("protocol.reconnection.backoff_ms", backoff_delay.as_millis() as f64, None);
        }
        
        // Wait for backoff delay
        debug!("Waiting for {} ms before reconnection attempt", backoff_delay.as_millis());
        tokio::time::sleep(backoff_delay).await;
        
        // Update connection state
        {
            let mut state = self.connection_state.write().unwrap();
            *state = ConnectionState::Connecting;
        }
        {
            let mut status = self.status.write().unwrap();
            *status = ConnectionStatus::Connecting;
        }
        
        // Attempt to reconnect
        match self.client.connect().await {
            Ok(_) => {
                // Update connection state
                {
                    let mut state = self.connection_state.write().unwrap();
                    *state = ConnectionState::Connected;
                }
                
                // Try to recover existing session if we have one
                let session_id = {
                    let session_guard = self.current_session.read().unwrap();
                    session_guard.as_ref().map(|s| s.id.clone())
                };
                
                if let Some(id) = session_id {
                    // Attempt session recovery
                    match self.recover_session(&id).await {
                        Ok(_) => {
                            debug!("Successfully recovered session during reconnection");
                            increment_counter("protocol.reconnection.session_recovered", None);
                        }
                        Err(e) => {
                            warn!("Failed to recover session during reconnection: {}", e);
                            increment_counter("protocol.reconnection.session_recovery_failed", None);
                            
                            // Create a new session instead
                            self.create_session();
                        }
                    }
                } else {
                    // Create a new session
                    self.create_session();
                }
                
                // Start message handler
                self.start_message_handler();
                
                // Authenticate
                match self.authenticate().await {
                    Ok(_) => {
                        // Reset reconnection attempts on success
                        {
                            let mut attempts = self.reconnect_attempts.write().unwrap();
                            *attempts = 0;
                        }
                        
                        // Record successful reconnection
                        increment_counter("protocol.reconnection.success", None);
                        
                        Ok(())
                    }
                    Err(e) => {
                        // Record authentication failure during reconnection
                        increment_counter("protocol.reconnection.auth_failed", None);
                        
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // Update connection state
                {
                    let mut state = self.connection_state.write().unwrap();
                    *state = ConnectionState::Error(e.to_string());
                }
                {
                    let mut status = self.status.write().unwrap();
                    *status = ConnectionStatus::ConnectionError(e.to_string());
                }
                
                // Record failed reconnection attempt
                increment_counter("protocol.reconnection.failed", None);
                
                Err(e)
            }
        }
    }
    
    /// Process synchronous message (send and wait for response)
    async fn process_sync_message(&self, message: Message) -> Result<Message, MessageError> {
        // Measure operation duration
        time_operation!("protocol.sync_message.process", None, {
            // Validate message
            self.validate_message(&message)?;
            
            // Convert to MCP message
            let mcp_message = McpClient::convert_to_mcp_message(message)?;
            
            // Create response channel
            let (tx, rx) = oneshot::channel();
            
            // Register response channel
            {
                let handler_guard = self.message_handler.lock().unwrap();
                if let Some(handler) = handler_guard.as_ref() {
                    handler.register_response_channel(mcp_message.id.clone(), tx);
                } else {
                    return Err(MessageError::InvalidState("No message handler available".to_string()));
                }
            }
            
            // Send message
            match self.client
                .send_raw(serde_json::to_string(&mcp_message)?)
                .await
            {
                Ok(_) => {
                    // Record sync message sent metric
                    increment_counter("protocol.sync_message.sent", None);
                },
                Err(e) => {
                    // Clean up response channel on error
                    let handler_guard = self.message_handler.lock().unwrap();
                    if let Some(handler) = handler_guard.as_ref() {
                        handler.cancel_request(&mcp_message.id);
                    }
                    
                    // Record error metric
                    increment_counter("protocol.sync_message.send_error", None);
                    
                    return Err(MessageError::NetworkError(e.to_string()));
                }
            }
            
            // Wait for response with timeout
            match timeout(self.config.request_timeout, rx).await {
                Ok(result) => match result {
                    Ok(response) => match response {
                        Ok(mcp_response) => {
                            // Convert MCP response to application message
                            match McpClient::convert_from_mcp_message(mcp_response) {
                                Ok(app_message) => {
                                    // Record success metric
                                    increment_counter("protocol.sync_message.success", None);
                                    Ok(app_message)
                                },
                                Err(e) => {
                                    // Record conversion error
                                    increment_counter("protocol.sync_message.conversion_error", None);
                                    Err(e)
                                }
                            }
                        },
                        Err(e) => {
                            // Record error response
                            increment_counter("protocol.sync_message.response_error", None);
                            Err(MessageError::ProtocolError(e.to_string()))
                        }
                    },
                    Err(_) => {
                        // Record channel closed error
                        increment_counter("protocol.sync_message.channel_closed", None);
                        Err(MessageError::ProtocolError("Response channel closed".to_string()))
                    }
                },
                Err(_) => {
                    // Record timeout
                    increment_counter("protocol.sync_message.timeout", None);
                    Err(MessageError::Timeout(self.config.request_timeout.as_secs() as u32))
                }
            }
        })
    }
    
    /// Check if the handler is connected and authenticated
    pub fn is_connected(&self) -> bool {
        let state = self.connection_state.read().unwrap();
        *state == ConnectionState::Authenticated
    }
    
    /// Get connection state as string
    pub fn get_connection_state_str(&self) -> String {
        let state = self.connection_state.read().unwrap();
        match *state {
            ConnectionState::Disconnected => "Disconnected".to_string(),
            ConnectionState::Connecting => "Connecting".to_string(),
            ConnectionState::Connected => "Connected".to_string(),
            ConnectionState::Authenticated => "Authenticated".to_string(),
            ConnectionState::Error(ref msg) => format!("Error: {}", msg),
        }
    }
}

#[async_trait]
impl ProtocolHandler for McpProtocolHandler {
    fn protocol_name(&self) -> &'static str {
        "Model Context Protocol"
    }
    
    fn connection_status(&self) -> ConnectionStatus {
        self.status.read().unwrap().clone()
    }
    
    async fn connect(&self) -> Result<(), String> {
        // Initialize the protocol handler if not already done
        {
            let is_active = self.is_active.read().unwrap();
            if !*is_active {
                self.initialize();
            }
        }
        
        // Record connection attempt
        increment_counter("protocol.connection.attempt", None);
        
        // Update connection state and status
        {
            let mut state = self.connection_state.write().unwrap();
            *state = ConnectionState::Connecting;
        }
        {
            let mut status = self.status.write().unwrap();
            *status = ConnectionStatus::Connecting;
        }
        
        // Connect to WebSocket
        match self.client.connect().await {
            Ok(_) => {
                // Update connection state
                {
                    let mut state = self.connection_state.write().unwrap();
                    *state = ConnectionState::Connected;
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
                        let mut status = self.status.write().unwrap();
                        *status = ConnectionStatus::AuthFailed;
                        
                        // Record authentication failure
                        increment_counter("protocol.connection.auth_failed", None);
                        
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                // Update connection state and status
                {
                    let mut state = self.connection_state.write().unwrap();
                    *state = ConnectionState::Error(e.to_string());
                }
                {
                    let mut status = self.status.write().unwrap();
                    *status = ConnectionStatus::ConnectionError(e.to_string());
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
        {
            let mut state = self.connection_state.write().unwrap();
            *state = ConnectionState::Disconnected;
        }
        
        // Cancel all streaming requests
        {
            let mut streaming_requests = self.streaming_requests.lock().unwrap();
            for (id, sender) in streaming_requests.drain() {
                debug!("Canceling streaming request {} due to disconnect", id);
                let _ = sender.send(Err(MessageError::ConnectionClosed)).await;
            }
        }
        
        // Cancel all pending requests
        {
            let handler_guard = self.message_handler.lock().unwrap();
            if let Some(handler) = handler_guard.as_ref() {
                handler.cancel_all_requests(McpError::ConnectionClosed);
            }
        }
        
        // Disconnect WebSocket
        match self.client.disconnect().await {
            Ok(_) => {
                // Update connection status
                let mut status = self.status.write().unwrap();
                *status = ConnectionStatus::Disconnected;
                
                // Set active flag to false
                {
                    let mut is_active = self.is_active.write().unwrap();
                    *is_active = false;
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
        // Check connection
        if !self.is_connected() {
            increment_counter("protocol.message.error.not_connected", None);
            return Err(MessageError::ConnectionClosed);
        }
        
        // Validate message
        self.validate_message(&message)?;
        
        // Convert to MCP message
        let mcp_message = McpClient::convert_to_mcp_message(message)?;
        
        // Queue message for sending
        {
            let mut queue = self.outgoing_queue.lock().unwrap();
            queue.push(mcp_message.clone());
        }
        
        // Record message queued metric
        increment_counter("protocol.message.queued", None);
        
        Ok(())
    }
    
    async fn receive_messages(&self) -> Result<Vec<Message>, MessageError> {
        // Check connection
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        // Get event receiver
        let mut event_receiver_guard = self.event_receiver.lock().unwrap();
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
    }
    
    // Additional methods required by the interface
    
    /// Stream a message and get chunks
    async fn stream_message(&self, message: Message) -> Result<Receiver<Result<Message, MessageError>>, MessageError> {
        // Check connection
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        // Create streaming channel
        self.create_streaming_channel(message).await
    }
    
    /// Cancel a streaming request
    async fn cancel_stream(&self, streaming_id: &str) -> Result<(), MessageError> {
        // Check connection
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        // Cancel streaming
        self.cancel_streaming(streaming_id).await
    }
    
    /// Send a message and wait for a response
    async fn send_and_receive(&self, message: Message) -> Result<Message, MessageError> {
        // Check connection
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        // Process synchronous message
        self.process_sync_message(message).await
    }
}
