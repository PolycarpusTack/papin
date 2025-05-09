use crate::models::messages::MessageError;
use crate::protocols::mcp::error::McpError;
use crate::protocols::mcp::message::{McpMessage, McpMessagePayload};
use crate::protocols::mcp::types::McpMessageType;
use crate::protocols::mcp::McpConfig;

use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is initializing
    Initializing,
    
    /// Session is active
    Active,
    
    /// Session is idle
    Idle,
    
    /// Session has timed out
    TimedOut,
    
    /// Session has been terminated
    Terminated,
}

/// Session data
#[derive(Debug)]
pub struct Session {
    /// Session ID
    pub id: String,
    
    /// Current session status
    pub status: Arc<RwLock<SessionStatus>>,
    
    /// Last activity timestamp
    pub last_activity: Arc<RwLock<Instant>>,
    
    /// Creation timestamp
    pub created_at: SystemTime,
    
    /// User ID associated with this session
    pub user_id: Option<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Session timeout
    pub timeout: Duration,
    
    /// Session metadata
    pub metadata: Arc<RwLock<HashMap<String, String>>>,
}

impl Session {
    /// Create a new session
    pub fn new(id: Option<String>, timeout: Option<Duration>) -> Self {
        let session_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let session_timeout = timeout.unwrap_or(Duration::from_secs(3600)); // Default: 1 hour
        
        Self {
            id: session_id,
            status: Arc::new(RwLock::new(SessionStatus::Initializing)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            created_at: SystemTime::now(),
            user_id: None,
            auth_token: None,
            timeout: session_timeout,
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        let status = self.status.read().unwrap();
        *status == SessionStatus::Active
    }
    
    /// Check if the session has timed out
    pub fn is_timed_out(&self) -> bool {
        let last_activity = self.last_activity.read().unwrap();
        let elapsed = last_activity.elapsed();
        elapsed >= self.timeout
    }
    
    /// Update the session status
    pub fn update_status(&self, status: SessionStatus) {
        let mut status_guard = self.status.write().unwrap();
        *status_guard = status;
    }
    
    /// Update the last activity timestamp
    pub fn update_activity(&self) {
        let mut last_activity = self.last_activity.write().unwrap();
        *last_activity = Instant::now();
    }
    
    /// Set metadata value
    pub fn set_metadata(&self, key: String, value: String) {
        let mut metadata = self.metadata.write().unwrap();
        metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        let metadata = self.metadata.read().unwrap();
        metadata.get(key).cloned()
    }
    
    /// Generate heartbeat message
    pub fn generate_heartbeat(&self) -> McpMessage {
        McpMessage {
            id: Uuid::new_v4().to_string(),
            version: "v1".to_string(),
            type_: McpMessageType::Ping,
            payload: McpMessagePayload::Ping {},
        }
    }
}

/// Session manager for handling multiple sessions
pub struct SessionManager {
    /// Active sessions
    sessions: Arc<Mutex<HashMap<String, Arc<Session>>>>,
    
    /// Session timeout duration
    timeout_duration: Duration,
    
    /// Session cleanup interval
    cleanup_interval: Duration,
    
    /// Session persistence enabled
    persistence_enabled: bool,
    
    /// Protocol configuration
    config: McpConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: McpConfig) -> Self {
        let timeout = config.request_timeout;
        
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            timeout_duration: timeout,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            persistence_enabled: true,
            config,
        }
    }
    
    /// Start the session cleanup task
    pub fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let interval = self.cleanup_interval;
        
        tokio::spawn(async move {
            loop {
                // Sleep for the cleanup interval
                tokio::time::sleep(interval).await;
                
                // Find and remove expired sessions
                let mut to_remove = Vec::new();
                {
                    let sessions_guard = sessions.lock().unwrap();
                    for (id, session) in sessions_guard.iter() {
                        if session.is_timed_out() {
                            to_remove.push(id.clone());
                        }
                    }
                }
                
                if !to_remove.is_empty() {
                    let mut sessions_guard = sessions.lock().unwrap();
                    for id in &to_remove {
                        if let Some(session) = sessions_guard.remove(id) {
                            info!("Session {} timed out and was removed", id);
                            session.update_status(SessionStatus::TimedOut);
                            
                            // Attempt to persist session data if enabled
                            if Self::is_persistence_enabled() {
                                if let Err(e) = Self::persist_session(&session) {
                                    error!("Failed to persist session {}: {}", id, e);
                                }
                            }
                        }
                    }
                    
                    debug!("Cleaned up {} expired sessions", to_remove.len());
                }
            }
        });
    }
    
    /// Create a new session
    pub fn create_session(&self, id: Option<String>) -> Arc<Session> {
        let session = Arc::new(Session::new(id, Some(self.timeout_duration)));
        
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        
        debug!("Created new session with ID: {}", session.id);
        session
    }
    
    /// Get session by ID
    pub fn get_session(&self, id: &str) -> Option<Arc<Session>> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(id).cloned()
    }
    
    /// Terminate a session
    pub fn terminate_session(&self, id: &str) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        let result = sessions.remove(id).is_some();
        
        if result {
            debug!("Terminated session with ID: {}", id);
        }
        
        result
    }
    
    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<Arc<Session>> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.is_active())
            .cloned()
            .collect()
    }
    
    /// Count active sessions
    pub fn count_active_sessions(&self) -> usize {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().filter(|s| s.is_active()).count()
    }
    
    /// Check if persistence is enabled
    fn is_persistence_enabled() -> bool {
        // This is a class method to avoid borrowing self
        // In a real implementation, this would check some global setting
        true
    }
    
    /// Persist session data
    fn persist_session(session: &Session) -> Result<(), String> {
        // In a real implementation, this would save to disk or database
        // For now, just log that we would persist
        debug!("Would persist session {}", session.id);
        Ok(())
    }
    
    /// Recover a session from persistent storage
    pub fn recover_session(&self, id: &str) -> Result<Arc<Session>, String> {
        // In a real implementation, this would load from disk or database
        Err(format!("No persisted session found with ID: {}", id))
    }
}

/// Session message handler
pub struct SessionMessageHandler {
    /// Session
    session: Arc<Session>,
    
    /// Response channels for requests
    response_channels: Arc<Mutex<HashMap<String, oneshot::Sender<Result<McpMessage, McpError>>>>>,
    
    /// Event channel
    event_sender: UnboundedSender<McpMessage>,
}

impl SessionMessageHandler {
    /// Create a new session message handler
    pub fn new(session: Arc<Session>) -> (Self, UnboundedReceiver<McpMessage>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        (
            Self {
                session,
                response_channels: Arc::new(Mutex::new(HashMap::new())),
                event_sender,
            },
            event_receiver,
        )
    }
    
    /// Register a response channel for a request
    pub fn register_response_channel(
        &self,
        request_id: String,
        channel: oneshot::Sender<Result<McpMessage, McpError>>,
    ) {
        let mut channels = self.response_channels.lock().unwrap();
        channels.insert(request_id, channel);
    }
    
    /// Handle an incoming message
    pub fn handle_message(&self, message: McpMessage) -> Result<(), McpError> {
        // Update session activity
        self.session.update_activity();
        
        match message.type_ {
            McpMessageType::CompletionResponse => {
                self.handle_completion_response(message)
            }
            McpMessageType::StreamingMessage => {
                self.handle_streaming_message(message)
            }
            McpMessageType::StreamingEnd => {
                self.handle_streaming_end(message)
            }
            McpMessageType::Error => {
                self.handle_error_message(message)
            }
            McpMessageType::Pong => {
                // Heartbeat response, just log it
                debug!("Received pong message");
                Ok(())
            }
            McpMessageType::AuthResponse => {
                self.handle_auth_response(message)
            }
            _ => {
                // Unsupported message type, send as event
                warn!("Received unsupported message type: {:?}", message.type_);
                self.event_sender.send(message).map_err(|_| {
                    McpError::ProtocolError("Failed to send message to event channel".to_string())
                })?;
                Ok(())
            }
        }
    }
    
    /// Handle a completion response message
    fn handle_completion_response(&self, message: McpMessage) -> Result<(), McpError> {
        let mut channels = self.response_channels.lock().unwrap();
        if let Some(channel) = channels.remove(&message.id) {
            channel.send(Ok(message)).map_err(|_| {
                McpError::ProtocolError("Failed to send response to channel".to_string())
            })?;
        } else {
            // No channel found, send as event
            self.event_sender.send(message).map_err(|_| {
                McpError::ProtocolError("Failed to send message to event channel".to_string())
            })?;
        }
        
        Ok(())
    }
    
    /// Handle a streaming message
    fn handle_streaming_message(&self, message: McpMessage) -> Result<(), McpError> {
        // Extract streaming ID from payload
        if let McpMessagePayload::StreamingMessage { streaming_id, .. } = &message.payload {
            // Use the streaming ID as the key for finding the response channel
            // This is different from the message ID, which is unique per message
            let mut channels = self.response_channels.lock().unwrap();
            if let Some(channel) = channels.get(streaming_id) {
                let _ = channel.send(Ok(message.clone()));
            } else {
                // No channel found, send as event
                self.event_sender.send(message).map_err(|_| {
                    McpError::ProtocolError("Failed to send message to event channel".to_string())
                })?;
            }
        } else {
            return Err(McpError::ProtocolError(
                "Invalid streaming message format".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Handle a streaming end message
    fn handle_streaming_end(&self, message: McpMessage) -> Result<(), McpError> {
        // Extract streaming ID from payload
        if let McpMessagePayload::StreamingEnd { streaming_id } = &message.payload {
            // Use the streaming ID as the key for finding the response channel
            let mut channels = self.response_channels.lock().unwrap();
            if let Some(channel) = channels.remove(streaming_id) {
                let _ = channel.send(Ok(message.clone()));
            } else {
                // No channel found, send as event
                self.event_sender.send(message).map_err(|_| {
                    McpError::ProtocolError("Failed to send message to event channel".to_string())
                })?;
            }
        } else {
            return Err(McpError::ProtocolError(
                "Invalid streaming end message format".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Handle an error message
    fn handle_error_message(&self, message: McpMessage) -> Result<(), McpError> {
        // Extract request ID from payload
        if let McpMessagePayload::Error { request_id, .. } = &message.payload {
            let mut channels = self.response_channels.lock().unwrap();
            if let Some(channel) = channels.remove(request_id) {
                if let McpMessagePayload::Error { code, message: error_msg, .. } = 
                    &message.payload 
                {
                    let error = McpError::ProtocolError(
                        format!("{:?}: {}", code, error_msg),
                    );
                    let _ = channel.send(Err(error));
                }
            } else {
                // No channel found, send as event
                self.event_sender.send(message).map_err(|_| {
                    McpError::ProtocolError("Failed to send message to event channel".to_string())
                })?;
            }
        } else {
            return Err(McpError::ProtocolError(
                "Invalid error message format".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Handle an authentication response message
    fn handle_auth_response(&self, message: McpMessage) -> Result<(), McpError> {
        // Parse authentication response
        if let McpMessagePayload::AuthResponse { success, session_id } = &message.payload {
            if *success {
                if let Some(sid) = session_id {
                    // Store session ID in metadata
                    self.session.set_metadata("server_session_id".to_string(), sid.clone());
                    
                    // Update session status
                    self.session.update_status(SessionStatus::Active);
                    
                    info!("Successfully authenticated, server session ID: {}", sid);
                } else {
                    warn!("Authentication successful but no session ID provided");
                    
                    // Update session status anyway
                    self.session.update_status(SessionStatus::Active);
                }
            } else {
                // Authentication failed
                self.session.update_status(SessionStatus::Terminated);
                
                return Err(McpError::AuthenticationFailed(
                    "Authentication failed".to_string(),
                ));
            }
        } else {
            return Err(McpError::ProtocolError(
                "Invalid authentication response format".to_string(),
            ));
        }
        
        // Also send to any waiting response channel
        let mut channels = self.response_channels.lock().unwrap();
        if let Some(channel) = channels.remove(&message.id) {
            let _ = channel.send(Ok(message));
        }
        
        Ok(())
    }
    
    /// Cancel all pending requests
    pub fn cancel_all_requests(&self, error: McpError) {
        let mut channels = self.response_channels.lock().unwrap();
        for (_, channel) in channels.drain() {
            let _ = channel.send(Err(error.clone()));
        }
    }
}
