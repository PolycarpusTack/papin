use crate::models::messages::{Message, MessageError};
use crate::protocols::mcp::error::McpError;
use crate::protocols::mcp::message::{McpMessage, McpMessagePayload, McpResponseMessage};
use crate::protocols::mcp::session::{Session, SessionManager, SessionMessageHandler};
use crate::protocols::mcp::types::{McpCompletionRequest, McpMessageRole, McpMessageType};
use crate::protocols::mcp::{McpClient, McpConfig};
use crate::protocols::{ConnectionStatus, ProtocolHandler};
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
        }
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
        
        session
    }
    
    /// Authenticate with the server
    async fn authenticate(&self) -> Result<(), McpError> {
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
                                
                                Ok(())
                            } else {
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
            Err(_) => Err(McpError::Timeout(self.config.connection_timeout)),
        }
    }
    
    /// Start the message handling task
    fn start_message_handler(&self) {
        let client_clone = self.client.clone();
        let message_handler_clone = self.message_handler.clone();
        let connection_state_clone = self.connection_state.clone();
        let status_clone = self.status.clone();
        let config_clone = self.config.clone();
        let streaming_requests_clone = self.streaming_requests.clone();
        
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
                        match serde_json::from_str::<McpMessage>(&message_str) {
                            Ok(message) => {
                                // Handle message
                                let handler_guard = message_handler_clone.lock().unwrap();
                                if let Some(handler) = handler_guard.as_ref() {
                                    if let Err(e) = handler.handle_message(message.clone()) {
                                        error!("Error handling message: {}", e);
                                    }
                                    
                                    // Handle streaming messages separately
                                    if let McpMessageType::StreamingMessage = message.type_ {
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
                                                
                                                // Send to application
                                                let _ = sender.send(Ok(app_message)).await;
                                                
                                                // If final, remove from tracking
                                                if *is_final {
                                                    streaming_requests.remove(streaming_id);
                                                }
                                            }
                                        }
                                    } else if let McpMessageType::StreamingEnd = message.type_ {
                                        if let McpMessagePayload::StreamingEnd { streaming_id } = &message.payload {
                                            let mut streaming_requests = streaming_requests_clone.lock().unwrap();
                                            streaming_requests.remove(streaming_id);
                                        }
                                    }
                                } else {
                                    error!("No message handler available");
                                }
                            }
                            Err(e) => {
                                error!("Error parsing message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        
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
                            tokio::spawn(async move {
                                // TODO: Implement reconnection logic
                            });
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
        
        tokio::spawn(async move {
            let heartbeat_interval = Duration::from_secs(30); // 30 seconds between heartbeats
            
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
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }
    
    /// Validate message before sending
    fn validate_message(&self, message: &Message) -> Result<(), MessageError> {
        // Check message ID
        if message.id.is_empty() {
            return Err(MessageError::ValidationError("Message ID cannot be empty".to_string()));
        }
        
        // Check message content
        if message.content.parts.is_empty() {
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
                    return Err(MessageError::ValidationError(
                        "Tool messages must have tool result content".to_string(),
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Create a streaming channel
    async fn create_streaming_channel(
        &self,
        message: Message,
    ) -> Result<Receiver<Result<Message, MessageError>>, MessageError> {
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
        }
        
        // Convert to MCP message
        let mut mcp_message = McpClient::convert_to_mcp_message(message)?;
        
        // Set streaming parameters
        if let McpMessagePayload::CompletionRequest(ref mut req) = mcp_message.payload {
            req.stream = Some(true);
            req.streaming_id = Some(streaming_id.clone());
        }
        
        // Send message
        self.client
            .send_raw(serde_json::to_string(&mcp_message)?)
            .await
            .map_err(|e| MessageError::NetworkError(e.to_string()))?;
        
        Ok(rx)
    }
    
    /// Cancel streaming request
    async fn cancel_streaming(&self, streaming_id: &str) -> Result<(), MessageError> {
        // Remove channel
        {
            let mut streaming_requests = self.streaming_requests.lock().unwrap();
            streaming_requests.remove(streaming_id);
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
        self.client
            .send_raw(serde_json::to_string(&cancel_message)?)
            .await
            .map_err(|e| MessageError::NetworkError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Initialize reconnection process
    async fn handle_reconnection(&self) -> Result<(), McpError> {
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
                return Err(McpError::ConnectionError(
                    "Maximum reconnection attempts reached".to_string(),
                ));
            }
        }
        
        // Update reconnection attempts
        {
            let mut attempts = self.reconnect_attempts.write().unwrap();
            *attempts += 1;
            
            info!("Reconnection attempt {}/{}", *attempts, self.config.max_reconnect_attempts);
        }
        
        // Calculate backoff delay
        let backoff_delay = {
            let attempts = self.reconnect_attempts.read().unwrap();
            // Exponential backoff: base_delay * 2^attempt
            self.config.reconnect_backoff.mul_f64(2f64.powi(*attempts as i32 - 1))
        };
        
        // Wait for backoff delay
        tokio::time::sleep(backoff_delay).await;
        
        // Update last reconnection attempt
        {
            let mut last_attempt = self.last_reconnect_attempt.write().unwrap();
            *last_attempt = Some(Instant::now());
        }
        
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
                
                // Create a new session
                self.create_session();
                
                // Start message handler
                self.start_message_handler();
                
                // Authenticate
                self.authenticate().await?;
                
                // Reset reconnection attempts on success
                {
                    let mut attempts = self.reconnect_attempts.write().unwrap();
                    *attempts = 0;
                }
                
                Ok(())
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
                
                Err(e)
            }
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
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let mut status = self.status.write().unwrap();
                        *status = ConnectionStatus::AuthFailed;
                        
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
                
                Err(e.to_string())
            }
        }
    }
    
    async fn disconnect(&self) -> Result<(), String> {
        // Update connection state
        {
            let mut state = self.connection_state.write().unwrap();
            *state = ConnectionState::Disconnected;
        }
        
        // Cancel all streaming requests
        {
            let mut streaming_requests = self.streaming_requests.lock().unwrap();
            for (_, sender) in streaming_requests.drain() {
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
                
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
    
    async fn send_message(&self, message: Message) -> Result<(), MessageError> {
        // Check connection
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        // Validate message
        self.validate_message(&message)?;
        
        // Convert to MCP message
        let mcp_message = McpClient::convert_to_mcp_message(message)?;
        
        // Send message
        self.client
            .send_raw(serde_json::to_string(&mcp_message)?)
            .await
            .map_err(|e| MessageError::NetworkError(e.to_string()))?;
        
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
            
            Ok(messages)
        } else {
            Ok(Vec::new())
        }
    }
    
    // Additional methods that could be added to extend the implementation
    
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
}
