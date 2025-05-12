use crate::models::messages::MessageError;
use crate::protocols::mcp::error::McpError;
use crate::protocols::mcp::message::{McpMessage, McpMessagePayload};
use crate::protocols::mcp::types::McpMessageType;
use crate::protocols::mcp::McpConfig;
use crate::observability::metrics::{increment_counter, record_gauge, record_histogram, time_operation};

use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Maximum time without activity before a session times out
const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Persistent session data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPersistentData {
    /// Session ID
    pub id: String,
    
    /// Status at time of persistence
    pub status: SessionStatus,
    
    /// Creation timestamp as ISO string
    pub created_at: String,
    
    /// Last activity timestamp as ISO string
    pub last_activity: String,
    
    /// User ID associated with this session
    pub user_id: Option<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Session timeout in seconds
    pub timeout_seconds: u64,
    
    /// Session metadata
    pub metadata: HashMap<String, String>,
    
    /// Server-side session ID
    pub server_session_id: Option<String>,
    
    /// Conversation history
    pub conversation_history: Vec<String>,
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
    
    /// Conversation history
    pub conversation_history: Arc<RwLock<Vec<String>>>,
}

impl Session {
    /// Create a new session
    pub fn new(id: Option<String>, timeout: Option<Duration>) -> Self {
        let session_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let session_timeout = timeout.unwrap_or(DEFAULT_SESSION_TIMEOUT);
        
        let session = Self {
            id: session_id,
            status: Arc::new(RwLock::new(SessionStatus::Initializing)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            created_at: SystemTime::now(),
            user_id: None,
            auth_token: None,
            timeout: session_timeout,
            metadata: Arc::new(RwLock::new(HashMap::new())),
            conversation_history: Arc::new(RwLock::new(Vec::new())),
        };
        
        // Record metric for session creation
        increment_counter("session.created", None);
        
        session
    }
    
    /// Create session from persistent data
    pub fn from_persistent_data(data: SessionPersistentData) -> Self {
        let session = Self {
            id: data.id,
            status: Arc::new(RwLock::new(data.status)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            created_at: SystemTime::now(), // Cannot reliably restore original timestamp
            user_id: data.user_id,
            auth_token: data.auth_token,
            timeout: Duration::from_secs(data.timeout_seconds),
            metadata: Arc::new(RwLock::new(data.metadata)),
            conversation_history: Arc::new(RwLock::new(data.conversation_history)),
        };
        
        // Set server session ID if available
        if let Some(server_id) = data.server_session_id {
            session.set_metadata("server_session_id".to_string(), server_id);
        }
        
        // Record metric for session restoration
        increment_counter("session.restored", None);
        
        session
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
        
        // Record session age metric
        record_histogram("session.age_seconds", elapsed.as_secs_f64(), None);
        
        elapsed >= self.timeout
    }
    
    /// Update the session status
    pub fn update_status(&self, status: SessionStatus) {
        let mut status_guard = self.status.write().unwrap();
        *status_guard = status;
        
        // Record metric for status change
        increment_counter(&format!("session.status.{:?}", status), None);
    }
    
    /// Update the last activity timestamp
    pub fn update_activity(&self) {
        let mut last_activity = self.last_activity.write().unwrap();
        *last_activity = Instant::now();
        
        // Record activity update
        increment_counter("session.activity_updated", None);
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
    
    /// Add a message to conversation history
    pub fn add_to_history(&self, message_json: String) {
        let mut history = self.conversation_history.write().unwrap();
        history.push(message_json);
        
        // Record history size metric
        record_gauge("session.history_size", history.len() as f64, None);
    }
    
    /// Get conversation history
    pub fn get_history(&self) -> Vec<String> {
        let history = self.conversation_history.read().unwrap();
        history.clone()
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
    
    /// Convert session to persistent data format
    pub fn to_persistent_data(&self) -> SessionPersistentData {
        let metadata = self.metadata.read().unwrap().clone();
        let history = self.conversation_history.read().unwrap().clone();
        let status = *self.status.read().unwrap();
        
        let created_at = match self.created_at.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                chrono::NaiveDateTime::from_timestamp_opt(secs as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            },
            Err(_) => "unknown".to_string(),
        };
        
        // Get last activity as ISO string
        let last_activity = {
            let now = SystemTime::now();
            let last_activity_duration = self.last_activity.read().unwrap().elapsed();
            
            match now.checked_sub(last_activity_duration) {
                Some(last_activity_time) => {
                    match last_activity_time.duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(duration) => {
                            let secs = duration.as_secs();
                            chrono::NaiveDateTime::from_timestamp_opt(secs as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        },
                        Err(_) => "unknown".to_string(),
                    }
                },
                None => "unknown".to_string(),
            }
        };
        
        SessionPersistentData {
            id: self.id.clone(),
            status,
            created_at,
            last_activity,
            user_id: self.user_id.clone(),
            auth_token: self.auth_token.clone(),
            timeout_seconds: self.timeout.as_secs(),
            metadata,
            server_session_id: self.get_metadata("server_session_id"),
            conversation_history: history,
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
    
    /// Session storage directory
    storage_dir: PathBuf,
    
    /// Protocol configuration
    config: McpConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: McpConfig) -> Self {
        let timeout = config.session_timeout.unwrap_or(DEFAULT_SESSION_TIMEOUT);
        let storage_dir = config.session_storage_dir.clone().unwrap_or_else(|| {
            let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
            dir.push("mcp");
            dir.push("sessions");
            dir
        });
        
        let manager = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            timeout_duration: timeout,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            persistence_enabled: config.enable_session_persistence.unwrap_or(true),
            storage_dir,
            config,
        };
        
        // Create storage directory if it doesn't exist
        if manager.persistence_enabled {
            if let Err(e) = fs::create_dir_all(&manager.storage_dir) {
                error!("Failed to create session storage directory: {}", e);
            }
            
            // Load persisted sessions
            if let Err(e) = manager.load_persisted_sessions() {
                error!("Failed to load persisted sessions: {}", e);
            }
        }
        
        manager
    }
    
    /// Start the session cleanup task
    pub fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let interval = self.cleanup_interval;
        let persistence_enabled = self.persistence_enabled;
        let storage_dir = self.storage_dir.clone();
        
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
                            if persistence_enabled {
                                if let Err(e) = Self::persist_session_to_disk(&session, &storage_dir) {
                                    error!("Failed to persist session {}: {}", id, e);
                                }
                            }
                            
                            // Record session timeout metric
                            increment_counter("session.timeout", None);
                        }
                    }
                    
                    debug!("Cleaned up {} expired sessions", to_remove.len());
                    
                    // Record active sessions metric
                    record_gauge("session.active_count", sessions_guard.len() as f64, None);
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
        
        // Record active sessions metric
        record_gauge("session.active_count", sessions.len() as f64, None);
        
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
            increment_counter("session.terminated", None);
            
            // Record active sessions metric
            record_gauge("session.active_count", sessions.len() as f64, None);
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
    pub fn is_persistence_enabled(&self) -> bool {
        self.persistence_enabled
    }
    
    /// Get session file path
    fn get_session_file_path(id: &str, storage_dir: &Path) -> PathBuf {
        let mut path = storage_dir.to_path_buf();
        path.push(format!("{}.json", id));
        path
    }
    
    /// Persist session to disk
    fn persist_session_to_disk(session: &Session, storage_dir: &Path) -> Result<(), String> {
        // Create persistent data representation
        let persistent_data = session.to_persistent_data();
        
        // Create storage directory if it doesn't exist
        if !storage_dir.exists() {
            if let Err(e) = fs::create_dir_all(storage_dir) {
                return Err(format!("Failed to create session storage directory: {}", e));
            }
        }
        
        // Get file path
        let file_path = Self::get_session_file_path(&session.id, storage_dir);
        
        // Serialize to JSON
        let json = match serde_json::to_string_pretty(&persistent_data) {
            Ok(json) => json,
            Err(e) => return Err(format!("Failed to serialize session data: {}", e)),
        };
        
        // Write to file
        let mut file = match File::create(&file_path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to create session file: {}", e)),
        };
        
        if let Err(e) = file.write_all(json.as_bytes()) {
            return Err(format!("Failed to write session data: {}", e));
        }
        
        debug!("Persisted session {} to {}", session.id, file_path.display());
        increment_counter("session.persisted", None);
        
        Ok(())
    }
    
    /// Load persisted sessions
    fn load_persisted_sessions(&self) -> Result<(), String> {
        // Check if storage directory exists
        if !self.storage_dir.exists() {
            return Ok(());
        }
        
        // Read directory entries
        let entries = match fs::read_dir(&self.storage_dir) {
            Ok(entries) => entries,
            Err(e) => return Err(format!("Failed to read session storage directory: {}", e)),
        };
        
        // Load each session file
        let mut loaded_count = 0;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                // Skip non-JSON files
                if path.extension().map_or(true, |ext| ext != "json") {
                    continue;
                }
                
                // Read file
                let mut file = match File::open(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        warn!("Failed to open session file {}: {}", path.display(), e);
                        continue;
                    }
                };
                
                let mut content = String::new();
                if let Err(e) = file.read_to_string(&mut content) {
                    warn!("Failed to read session file {}: {}", path.display(), e);
                    continue;
                }
                
                // Parse JSON
                match serde_json::from_str::<SessionPersistentData>(&content) {
                    Ok(data) => {
                        // Restore session
                        let session = Arc::new(Session::from_persistent_data(data));
                        
                        // Store in sessions map
                        let mut sessions = self.sessions.lock().unwrap();
                        sessions.insert(session.id.clone(), session);
                        
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to parse session file {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        info!("Loaded {} persisted sessions", loaded_count);
        
        // Record loaded sessions metric
        increment_counter("session.loaded_from_disk", Some({
            let mut map = HashMap::new();
            map.insert("count".to_string(), loaded_count.to_string());
            map
        }));
        
        Ok(())
    }
    
    /// Persist a session
    pub fn persist_session(&self, session: &Session) -> Result<(), String> {
        if !self.persistence_enabled {
            return Err("Session persistence is disabled".to_string());
        }
        
        Self::persist_session_to_disk(session, &self.storage_dir)
    }
    
    /// Recover a session from persistent storage
    pub fn recover_session(&self, id: &str) -> Result<Arc<Session>, String> {
        if !self.persistence_enabled {
            return Err("Session persistence is disabled".to_string());
        }
        
        // Check if session is already active
        {
            let sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get(id) {
                return Ok(session.clone());
            }
        }
        
        // Get file path
        let file_path = Self::get_session_file_path(id, &self.storage_dir);
        
        // Check if file exists
        if !file_path.exists() {
            return Err(format!("No persisted session found with ID: {}", id));
        }
        
        // Read file
        let mut file = match File::open(&file_path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to open session file: {}", e)),
        };
        
        let mut content = String::new();
        if let Err(e) = file.read_to_string(&mut content) {
            return Err(format!("Failed to read session file: {}", e));
        }
        
        // Parse JSON
        let data = match serde_json::from_str::<SessionPersistentData>(&content) {
            Ok(data) => data,
            Err(e) => return Err(format!("Failed to parse session data: {}", e)),
        };
        
        // Create session
        let session = Arc::new(Session::from_persistent_data(data));
        
        // Store in sessions map
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        
        // Record session recovery metric
        increment_counter("session.recovered_from_disk", None);
        
        Ok(session)
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
        
        // Record pending requests metric
        record_gauge("session.pending_requests", channels.len() as f64, None);
    }
    
    /// Handle an incoming message
    pub fn handle_message(&self, message: McpMessage) -> Result<(), McpError> {
        // Measure message handling time
        time_operation!("session.message_handling", None, {
            // Update session activity
            self.session.update_activity();
            
            // Record message to conversation history if it's a significant message
            match message.type_ {
                McpMessageType::CompletionRequest | 
                McpMessageType::CompletionResponse => {
                    if let Ok(json) = serde_json::to_string(&message) {
                        self.session.add_to_history(json);
                    }
                },
                _ => {}
            }
            
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
                    increment_counter("session.heartbeat_response", None);
                    Ok(())
                }
                McpMessageType::AuthResponse => {
                    self.handle_auth_response(message)
                }
                McpMessageType::SessionRecoveryResponse => {
                    self.handle_session_recovery_response(message)
                }
                _ => {
                    // Unsupported message type, send as event
                    debug!("Received message of type: {:?}", message.type_);
                    increment_counter(&format!("session.message.type.{:?}", message.type_).to_lowercase(), None);
                    
                    self.event_sender.send(message).map_err(|_| {
                        McpError::ProtocolError("Failed to send message to event channel".to_string())
                    })?;
                    Ok(())
                }
            }
        })
    }
    
    /// Handle a completion response message
    fn handle_completion_response(&self, message: McpMessage) -> Result<(), McpError> {
        // Record metric for completion response
        increment_counter("session.completion_response", None);
        
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
        
        // Update pending requests metric
        record_gauge("session.pending_requests", channels.len() as f64, None);
        
        Ok(())
    }
    
    /// Handle a streaming message
    fn handle_streaming_message(&self, message: McpMessage) -> Result<(), McpError> {
        // Record metric for streaming message
        increment_counter("session.streaming_message", None);
        
        // Extract streaming ID from payload
        if let McpMessagePayload::StreamingMessage { streaming_id, chunk, .. } = &message.payload {
            // Record chunk size metric
            record_histogram("session.streaming_chunk_size", chunk.len() as f64, None);
            
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
        // Record metric for streaming end
        increment_counter("session.streaming_end", None);
        
        // Extract streaming ID from payload
        if let McpMessagePayload::StreamingEnd { streaming_id } = &message.payload {
            // Use the streaming ID as the key for finding the response channel
            let mut channels = self.response_channels.lock().unwrap();
            if let Some(channel) = channels.remove(streaming_id) {
                let _ = channel.send(Ok(message.clone()));
                
                // Update pending requests metric
                record_gauge("session.pending_requests", channels.len() as f64, None);
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
        // Record metric for error message
        increment_counter("session.error_message", None);
        
        // Extract request ID from payload
        if let McpMessagePayload::Error { request_id, code, message: error_msg, .. } = &message.payload {
            // Record error type metric
            increment_counter(&format!("session.error.{:?}", code).to_lowercase(), None);
            
            let mut channels = self.response_channels.lock().unwrap();
            if let Some(channel) = channels.remove(request_id) {
                let error = McpError::ProtocolError(
                    format!("{:?}: {}", code, error_msg),
                );
                let _ = channel.send(Err(error));
                
                // Update pending requests metric
                record_gauge("session.pending_requests", channels.len() as f64, None);
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
        // Record metric for auth response
        increment_counter("session.auth_response", None);
        
        // Parse authentication response
        if let McpMessagePayload::AuthResponse { success, session_id } = &message.payload {
            if *success {
                if let Some(sid) = session_id {
                    // Store session ID in metadata
                    self.session.set_metadata("server_session_id".to_string(), sid.clone());
                    
                    // Update session status
                    self.session.update_status(SessionStatus::Active);
                    
                    info!("Successfully authenticated, server session ID: {}", sid);
                    increment_counter("session.auth_success", None);
                } else {
                    warn!("Authentication successful but no session ID provided");
                    
                    // Update session status anyway
                    self.session.update_status(SessionStatus::Active);
                    
                    increment_counter("session.auth_success_no_id", None);
                }
            } else {
                // Authentication failed
                self.session.update_status(SessionStatus::Terminated);
                
                increment_counter("session.auth_failed", None);
                
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
            
            // Update pending requests metric
            record_gauge("session.pending_requests", channels.len() as f64, None);
        }
        
        Ok(())
    }
    
    /// Handle a session recovery response message
    fn handle_session_recovery_response(&self, message: McpMessage) -> Result<(), McpError> {
        // Record metric for session recovery response
        increment_counter("session.recovery_response", None);
        
        // Parse session recovery response
        if let McpMessagePayload::SessionRecoveryResponse { success, error, session_id } = &message.payload {
            if *success {
                // Update session status
                self.session.update_status(SessionStatus::Active);
                
                // Store server session ID if provided
                if let Some(sid) = session_id {
                    self.session.set_metadata("server_session_id".to_string(), sid.clone());
                    info!("Successfully recovered session, server session ID: {}", sid);
                } else {
                    info!("Successfully recovered session");
                }
                
                increment_counter("session.recovery_success", None);
            } else {
                // Recovery failed
                let error_msg = error.clone().unwrap_or_else(|| "Unknown error".to_string());
                self.session.update_status(SessionStatus::Terminated);
                
                warn!("Session recovery failed: {}", error_msg);
                increment_counter("session.recovery_failed", None);
                
                return Err(McpError::SessionError(
                    format!("Session recovery failed: {}", error_msg),
                ));
            }
        } else {
            return Err(McpError::ProtocolError(
                "Invalid session recovery response format".to_string(),
            ));
        }
        
        // Also send to any waiting response channel
        let mut channels = self.response_channels.lock().unwrap();
        if let Some(channel) = channels.remove(&message.id) {
            let _ = channel.send(Ok(message));
            
            // Update pending requests metric
            record_gauge("session.pending_requests", channels.len() as f64, None);
        }
        
        Ok(())
    }
    
    /// Cancel a specific request
    pub fn cancel_request(&self, request_id: &str) {
        let mut channels = self.response_channels.lock().unwrap();
        if let Some(channel) = channels.remove(request_id) {
            let _ = channel.send(Err(McpError::RequestCancelled));
            increment_counter("session.request_cancelled", None);
        }
        
        // Update pending requests metric
        record_gauge("session.pending_requests", channels.len() as f64, None);
    }
    
    /// Cancel all pending requests
    pub fn cancel_all_requests(&self, error: McpError) {
        let mut channels = self.response_channels.lock().unwrap();
        let count = channels.len();
        
        for (_, channel) in channels.drain() {
            let _ = channel.send(Err(error.clone()));
        }
        
        if count > 0 {
            info!("Cancelled {} pending requests", count);
            
            // Record metric for mass cancellation
            increment_counter("session.requests_mass_cancelled", Some({
                let mut map = HashMap::new();
                map.insert("count".to_string(), count.to_string());
                map
            }));
            
            // Update pending requests metric
            record_gauge("session.pending_requests", 0.0, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let session = Session::new(None, None);
        assert_eq!(*session.status.read().unwrap(), SessionStatus::Initializing);
        assert!(!session.is_active());
        assert!(!session.is_timed_out());
    }
    
    #[test]
    fn test_session_activity() {
        let session = Session::new(None, Some(Duration::from_millis(100)));
        
        // Initial state
        assert!(!session.is_timed_out());
        
        // Wait for timeout
        std::thread::sleep(Duration::from_millis(110));
        
        // Should be timed out now
        assert!(session.is_timed_out());
        
        // Update activity
        session.update_activity();
        
        // Should not be timed out after update
        assert!(!session.is_timed_out());
    }
    
    #[test]
    fn test_session_metadata() {
        let session = Session::new(None, None);
        
        // Set metadata
        session.set_metadata("test_key".to_string(), "test_value".to_string());
        
        // Get metadata
        let value = session.get_metadata("test_key");
        assert_eq!(value, Some("test_value".to_string()));
        
        // Get non-existent metadata
        let value = session.get_metadata("non_existent");
        assert_eq!(value, None);
    }
    
    #[test]
    fn test_session_persistence() {
        let session = Session::new(None, None);
        
        // Add some metadata
        session.set_metadata("test_key".to_string(), "test_value".to_string());
        
        // Convert to persistent data
        let persistent_data = session.to_persistent_data();
        
        // Check data
        assert_eq!(persistent_data.id, session.id);
        assert_eq!(persistent_data.status, *session.status.read().unwrap());
        assert_eq!(persistent_data.metadata.get("test_key"), Some(&"test_value".to_string()));
        
        // Convert back to session
        let restored_session = Session::from_persistent_data(persistent_data);
        
        // Check restored data
        assert_eq!(restored_session.id, session.id);
        assert_eq!(*restored_session.status.read().unwrap(), *session.status.read().unwrap());
        assert_eq!(restored_session.get_metadata("test_key"), Some("test_value".to_string()));
    }
}
