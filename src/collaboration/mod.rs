// Collaboration System for MCP Client
//
// This module provides real-time collaboration features including:
// - Real-time synchronization of conversation data
// - Cursor presence and user awareness
// - Session management for multi-device usage
// - Cross-device synchronization
// - Infrastructure for audio/video communication

pub mod presence;
pub mod rtc;
pub mod sessions;
pub mod sync;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::Result;
use crate::models::messages::{Conversation, Message};
use crate::observability::metrics::{record_counter, record_gauge};
use crate::security::permissions;

/// Collaboration configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// Whether collaboration features are enabled
    pub enabled: bool,
    
    /// Maximum number of users per collaboration session
    pub max_users_per_session: usize,
    
    /// Whether to automatically discover other devices
    pub auto_discover: bool,
    
    /// Whether to show presence information (cursors, etc.)
    pub show_presence: bool,
    
    /// Whether to enable audio/video capabilities
    pub enable_av: bool,
    
    /// Sync interval in milliseconds
    pub sync_interval_ms: u64,
    
    /// P2P mode (direct connections between clients when possible)
    pub p2p_enabled: bool,
    
    /// Server URLs for signaling and STUN/TURN
    pub server_urls: Vec<String>,
    
    /// Custom username for collaboration
    pub username: Option<String>,
    
    /// Custom user avatar (base64 encoded)
    pub user_avatar: Option<String>,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_users_per_session: 10,
            auto_discover: true,
            show_presence: true,
            enable_av: false, // Disabled by default
            sync_interval_ms: 1000, // 1 second
            p2p_enabled: true,
            server_urls: vec![
                "https://signaling.mcp-client.com".to_string(),
                "stun:stun.mcp-client.com:19302".to_string(),
                "turn:turn.mcp-client.com:3478".to_string(),
            ],
            username: None,
            user_avatar: None,
        }
    }
}

/// User role in a collaboration session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// Owner of the session (has all privileges)
    Owner,
    
    /// Co-owner with administrative privileges
    CoOwner,
    
    /// Editor can make changes but not manage users
    Editor,
    
    /// Commentator can only add comments
    Commentator,
    
    /// Viewer can only view but not edit
    Viewer,
}

/// User information for collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique ID for the user
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// User role
    pub role: UserRole,
    
    /// User avatar (base64 encoded)
    pub avatar: Option<String>,
    
    /// User's color for cursor and other UI elements
    pub color: String,
    
    /// Online status
    pub online: bool,
    
    /// Last active timestamp
    pub last_active: SystemTime,
    
    /// Current device ID
    pub device_id: String,
    
    /// Custom user metadata
    pub metadata: HashMap<String, String>,
}

/// A collaborative session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique ID for the session
    pub id: String,
    
    /// Session name
    pub name: String,
    
    /// Whether the session is active
    pub active: bool,
    
    /// Session creation time
    pub created_at: SystemTime,
    
    /// Last update time
    pub updated_at: SystemTime,
    
    /// Conversation ID associated with this session
    pub conversation_id: String,
    
    /// Users in the session
    pub users: HashMap<String, User>,
    
    /// Custom session metadata
    pub metadata: HashMap<String, String>,
}

/// Connection status for collaboration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Not connected to collaboration services
    Disconnected,
    
    /// Currently connecting
    Connecting,
    
    /// Connected and ready
    Connected,
    
    /// Connected with limited functionality
    Limited,
    
    /// Connection error
    Error,
}

/// Collaboration manager
pub struct CollaborationManager {
    /// Configuration
    config: Arc<RwLock<CollaborationConfig>>,
    
    /// Current user
    current_user: Arc<RwLock<User>>,
    
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    
    /// Current session ID
    current_session_id: Arc<RwLock<Option<String>>>,
    
    /// Connection status
    connection_status: Arc<RwLock<ConnectionStatus>>,
    
    /// Session manager
    session_manager: Arc<RwLock<sessions::SessionManager>>,
    
    /// Presence manager
    presence_manager: Arc<RwLock<presence::PresenceManager>>,
    
    /// Sync manager
    sync_manager: Arc<RwLock<sync::SyncManager>>,
    
    /// RTC manager for audio/video
    rtc_manager: Arc<RwLock<rtc::RTCManager>>,
}

impl CollaborationManager {
    /// Create a new collaboration manager
    pub fn new(config: CollaborationConfig) -> Result<Self> {
        // Generate a unique user ID if none exists yet
        let user_id = Uuid::new_v4().to_string();
        
        // Generate a unique device ID
        let device_id = format!("{}-{}", whoami::hostname(), Uuid::new_v4().to_string());
        
        // Create default user
        let username = config.username.clone().unwrap_or_else(|| {
            whoami::username()
        });
        
        // Create current user
        let current_user = User {
            id: user_id.clone(),
            name: username,
            role: UserRole::Owner, // Default to owner for the local user
            avatar: config.user_avatar.clone(),
            color: generate_user_color(&user_id),
            online: true,
            last_active: SystemTime::now(),
            device_id: device_id.clone(),
            metadata: HashMap::new(),
        };
        
        // Create managers
        let session_manager = sessions::SessionManager::new(
            user_id.clone(), 
            device_id.clone(),
            config.server_urls.clone(),
        )?;
        
        let presence_manager = presence::PresenceManager::new(
            user_id.clone(),
            device_id.clone(),
            config.show_presence,
        )?;
        
        let sync_manager = sync::SyncManager::new(
            user_id.clone(),
            device_id.clone(),
            config.sync_interval_ms,
        )?;
        
        let rtc_manager = rtc::RTCManager::new(
            user_id,
            device_id,
            config.enable_av,
            config.server_urls.clone(),
        )?;
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            current_user: Arc::new(RwLock::new(current_user)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            current_session_id: Arc::new(RwLock::new(None)),
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            session_manager: Arc::new(RwLock::new(session_manager)),
            presence_manager: Arc::new(RwLock::new(presence_manager)),
            sync_manager: Arc::new(RwLock::new(sync_manager)),
            rtc_manager: Arc::new(RwLock::new(rtc_manager)),
        })
    }
    
    /// Initialize the collaboration system and start services
    pub fn start(&self) -> Result<()> {
        // Check if collaboration is enabled
        if !self.config.read().unwrap().enabled {
            info!("Collaboration system is disabled");
            return Ok(());
        }
        
        // Check for permissions
        let permission_granted = permissions::check_permission("collaboration")?;
        if !permission_granted {
            let permission_granted = permissions::request_permission(
                "collaboration", 
                "Enable real-time collaboration with other users",
            )?;
            
            if !permission_granted {
                warn!("Collaboration permission denied");
                return Ok(());
            }
        }
        
        // Update connection status
        *self.connection_status.write().unwrap() = ConnectionStatus::Connecting;
        
        // Start session manager
        self.session_manager.read().unwrap().start()?;
        
        // Start presence manager
        self.presence_manager.read().unwrap().start()?;
        
        // Start sync manager
        self.sync_manager.read().unwrap().start()?;
        
        // Start RTC manager if enabled
        if self.config.read().unwrap().enable_av {
            self.rtc_manager.read().unwrap().start()?;
        }
        
        // Update connection status
        *self.connection_status.write().unwrap() = ConnectionStatus::Connected;
        
        info!("Collaboration system started");
        record_counter("collaboration.system_started", 1.0, None);
        
        Ok(())
    }
    
    /// Stop collaboration services
    pub fn stop(&self) -> Result<()> {
        // Update connection status
        *self.connection_status.write().unwrap() = ConnectionStatus::Disconnected;
        
        // Stop session manager
        self.session_manager.read().unwrap().stop()?;
        
        // Stop presence manager
        self.presence_manager.read().unwrap().stop()?;
        
        // Stop sync manager
        self.sync_manager.read().unwrap().stop()?;
        
        // Stop RTC manager
        self.rtc_manager.read().unwrap().stop()?;
        
        info!("Collaboration system stopped");
        record_counter("collaboration.system_stopped", 1.0, None);
        
        Ok(())
    }
    
    /// Create a new collaborative session
    pub fn create_session(&self, name: &str, conversation_id: &str) -> Result<Session> {
        // Generate a new session ID
        let session_id = Uuid::new_v4().to_string();
        
        // Create a new session
        let mut users = HashMap::new();
        users.insert(
            self.current_user.read().unwrap().id.clone(),
            self.current_user.read().unwrap().clone(),
        );
        
        let session = Session {
            id: session_id.clone(),
            name: name.to_string(),
            active: true,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            conversation_id: conversation_id.to_string(),
            users,
            metadata: HashMap::new(),
        };
        
        // Store the session
        self.sessions.write().unwrap().insert(session_id.clone(), session.clone());
        
        // Set as current session
        *self.current_session_id.write().unwrap() = Some(session_id.clone());
        
        // Initialize session in session manager
        self.session_manager.write().unwrap().create_session(&session_id, &session.name, conversation_id)?;
        
        // Initialize presence for this session
        self.presence_manager.write().unwrap().join_session(&session_id)?;
        
        // Initialize sync for this session
        self.sync_manager.write().unwrap().init_session(&session_id, conversation_id)?;
        
        info!("Created collaboration session: {}", session_id);
        record_counter("collaboration.session_created", 1.0, None);
        
        Ok(session)
    }
    
    /// Join an existing collaborative session
    pub fn join_session(&self, session_id: &str) -> Result<Session> {
        // Get session information from session manager
        let session_info = self.session_manager.write().unwrap().join_session(session_id)?;
        
        // Initialize presence for this session
        self.presence_manager.write().unwrap().join_session(session_id)?;
        
        // Initialize sync for this session
        self.sync_manager.write().unwrap().join_session(session_id, &session_info.conversation_id)?;
        
        // Update current session
        *self.current_session_id.write().unwrap() = Some(session_id.to_string());
        
        // Store session locally
        self.sessions.write().unwrap().insert(session_id.to_string(), session_info.clone());
        
        info!("Joined collaboration session: {}", session_id);
        record_counter("collaboration.session_joined", 1.0, None);
        
        Ok(session_info)
    }
    
    /// Leave the current collaborative session
    pub fn leave_session(&self) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(()),  // No active session
        };
        
        // Leave session in managers
        self.session_manager.write().unwrap().leave_session(&session_id)?;
        self.presence_manager.write().unwrap().leave_session(&session_id)?;
        self.sync_manager.write().unwrap().leave_session(&session_id)?;
        
        // Clear current session
        *self.current_session_id.write().unwrap() = None;
        
        info!("Left collaboration session: {}", session_id);
        record_counter("collaboration.session_left", 1.0, None);
        
        Ok(())
    }
    
    /// Invite a user to the current session
    pub fn invite_user(&self, email: &str, role: UserRole) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // Check if user has permission to invite
        let session = match self.sessions.read().unwrap().get(&session_id) {
            Some(session) => session.clone(),
            None => return Err("Session not found".into()),
        };
        
        let current_user_id = self.current_user.read().unwrap().id.clone();
        let current_user = match session.users.get(&current_user_id) {
            Some(user) => user,
            None => return Err("Current user not in session".into()),
        };
        
        // Only owners and co-owners can invite
        if current_user.role != UserRole::Owner && current_user.role != UserRole::CoOwner {
            return Err("You don't have permission to invite users".into());
        }
        
        // Invite the user via session manager
        self.session_manager.write().unwrap().invite_user(&session_id, email, role)?;
        
        info!("Invited user {} to session {}", email, session_id);
        record_counter("collaboration.user_invited", 1.0, None);
        
        Ok(())
    }
    
    /// Remove a user from the current session
    pub fn remove_user(&self, user_id: &str) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // Check if user has permission to remove
        let session = match self.sessions.read().unwrap().get(&session_id) {
            Some(session) => session.clone(),
            None => return Err("Session not found".into()),
        };
        
        let current_user_id = self.current_user.read().unwrap().id.clone();
        let current_user = match session.users.get(&current_user_id) {
            Some(user) => user,
            None => return Err("Current user not in session".into()),
        };
        
        // Only owners and co-owners can remove users
        if current_user.role != UserRole::Owner && current_user.role != UserRole::CoOwner {
            return Err("You don't have permission to remove users".into());
        }
        
        // Can't remove yourself this way
        if user_id == &current_user_id {
            return Err("Can't remove yourself from session. Use leave_session instead.".into());
        }
        
        // Remove the user via session manager
        self.session_manager.write().unwrap().remove_user(&session_id, user_id)?;
        
        info!("Removed user {} from session {}", user_id, session_id);
        record_counter("collaboration.user_removed", 1.0, None);
        
        Ok(())
    }
    
    /// Change a user's role in the current session
    pub fn change_user_role(&self, user_id: &str, role: UserRole) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // Check if user has permission to change roles
        let session = match self.sessions.read().unwrap().get(&session_id) {
            Some(session) => session.clone(),
            None => return Err("Session not found".into()),
        };
        
        let current_user_id = self.current_user.read().unwrap().id.clone();
        let current_user = match session.users.get(&current_user_id) {
            Some(user) => user,
            None => return Err("Current user not in session".into()),
        };
        
        // Only owners and co-owners can change roles
        if current_user.role != UserRole::Owner && current_user.role != UserRole::CoOwner {
            return Err("You don't have permission to change user roles".into());
        }
        
        // Change the user's role via session manager
        self.session_manager.write().unwrap().change_user_role(&session_id, user_id, role)?;
        
        // Update local session data
        if let Some(session) = self.sessions.write().unwrap().get_mut(&session_id) {
            if let Some(user) = session.users.get_mut(user_id) {
                user.role = role;
            }
        }
        
        info!("Changed role for user {} in session {}", user_id, session_id);
        record_counter("collaboration.role_changed", 1.0, None);
        
        Ok(())
    }
    
    /// Update user cursor position
    pub fn update_cursor_position(&self, x: f32, y: f32, element_id: Option<&str>) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(()),  // No active session
        };
        
        // Update cursor via presence manager
        self.presence_manager.write().unwrap().update_cursor_position(&session_id, x, y, element_id)?;
        
        // Update last active time
        if let Some(session) = self.sessions.write().unwrap().get_mut(&session_id) {
            let current_user_id = self.current_user.read().unwrap().id.clone();
            if let Some(user) = session.users.get_mut(&current_user_id) {
                user.last_active = SystemTime::now();
            }
        }
        
        Ok(())
    }
    
    /// Update user selection
    pub fn update_selection(&self, start_id: &str, end_id: &str, start_offset: usize, end_offset: usize) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(()),  // No active session
        };
        
        // Update selection via presence manager
        self.presence_manager.write().unwrap().update_selection(
            &session_id, 
            start_id, 
            end_id, 
            start_offset, 
            end_offset
        )?;
        
        Ok(())
    }
    
    /// Get all users in the current session
    pub fn get_session_users(&self) -> Result<Vec<User>> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(Vec::new()),  // No active session
        };
        
        // Get session
        let session = match self.sessions.read().unwrap().get(&session_id) {
            Some(session) => session.clone(),
            None => return Ok(Vec::new()), // Session not found
        };
        
        // Return all users
        Ok(session.users.values().cloned().collect())
    }
    
    /// Get all cursors in the current session
    pub fn get_cursors(&self) -> Result<HashMap<String, presence::CursorPosition>> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(HashMap::new()),  // No active session
        };
        
        // Get cursors from presence manager
        self.presence_manager.read().unwrap().get_cursors(&session_id)
    }
    
    /// Get all selections in the current session
    pub fn get_selections(&self) -> Result<HashMap<String, presence::Selection>> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(HashMap::new()),  // No active session
        };
        
        // Get selections from presence manager
        self.presence_manager.read().unwrap().get_selections(&session_id)
    }
    
    /// Synchronize a conversation
    pub fn sync_conversation(&self, conversation: &Conversation) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Ok(()),  // No active session
        };
        
        // Sync via sync manager
        self.sync_manager.write().unwrap().sync_conversation(&session_id, conversation)?;
        
        Ok(())
    }
    
    /// Send a message in the collaborative session
    pub fn send_message(&self, message: &Message) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // Send message via sync manager
        self.sync_manager.write().unwrap().send_message(&session_id, message)?;
        
        // Update last active time
        if let Some(session) = self.sessions.write().unwrap().get_mut(&session_id) {
            let current_user_id = self.current_user.read().unwrap().id.clone();
            if let Some(user) = session.users.get_mut(&current_user_id) {
                user.last_active = SystemTime::now();
            }
        }
        
        record_counter("collaboration.message_sent", 1.0, None);
        
        Ok(())
    }
    
    /// Start an audio call in the current session
    pub fn start_audio_call(&self) -> Result<()> {
        // Check if audio is enabled
        if !self.config.read().unwrap().enable_av {
            return Err("Audio/video features are not enabled".into());
        }
        
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // Start call via RTC manager
        self.rtc_manager.write().unwrap().start_audio_call(&session_id)?;
        
        info!("Started audio call in session {}", session_id);
        record_counter("collaboration.audio_call_started", 1.0, None);
        
        Ok(())
    }
    
    /// Start a video call in the current session
    pub fn start_video_call(&self) -> Result<()> {
        // Check if video is enabled
        if !self.config.read().unwrap().enable_av {
            return Err("Audio/video features are not enabled".into());
        }
        
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // Start call via RTC manager
        self.rtc_manager.write().unwrap().start_video_call(&session_id)?;
        
        info!("Started video call in session {}", session_id);
        record_counter("collaboration.video_call_started", 1.0, None);
        
        Ok(())
    }
    
    /// End the current call
    pub fn end_call(&self) -> Result<()> {
        // Get current session ID
        let session_id = match *self.current_session_id.read().unwrap() {
            Some(ref id) => id.clone(),
            None => return Err("No active collaboration session".into()),
        };
        
        // End call via RTC manager
        self.rtc_manager.write().unwrap().end_call(&session_id)?;
        
        info!("Ended call in session {}", session_id);
        record_counter("collaboration.call_ended", 1.0, None);
        
        Ok(())
    }
    
    /// Update collaboration configuration
    pub fn update_config(&self, config: CollaborationConfig) -> Result<()> {
        let old_config = self.config.read().unwrap().clone();
        
        // Update config
        *self.config.write().unwrap() = config.clone();
        
        // Update presence if needed
        if old_config.show_presence != config.show_presence {
            self.presence_manager.write().unwrap().set_enabled(config.show_presence)?;
        }
        
        // Update sync interval if needed
        if old_config.sync_interval_ms != config.sync_interval_ms {
            self.sync_manager.write().unwrap().set_sync_interval(config.sync_interval_ms)?;
        }
        
        // Update A/V if needed
        if old_config.enable_av != config.enable_av {
            self.rtc_manager.write().unwrap().set_enabled(config.enable_av)?;
        }
        
        // Update server URLs if needed
        if old_config.server_urls != config.server_urls {
            self.session_manager.write().unwrap().update_server_urls(config.server_urls.clone())?;
            self.rtc_manager.write().unwrap().update_server_urls(config.server_urls.clone())?;
        }
        
        info!("Updated collaboration config");
        
        Ok(())
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> CollaborationConfig {
        self.config.read().unwrap().clone()
    }
    
    /// Get the current connection status
    pub fn get_connection_status(&self) -> ConnectionStatus {
        *self.connection_status.read().unwrap()
    }
    
    /// Get the current user
    pub fn get_current_user(&self) -> User {
        self.current_user.read().unwrap().clone()
    }
    
    /// Update the current user's name
    pub fn update_username(&self, name: &str) -> Result<()> {
        // Update local user
        self.current_user.write().unwrap().name = name.to_string();
        
        // Get current session ID
        if let Some(session_id) = self.current_session_id.read().unwrap().clone() {
            // Update user in session manager
            self.session_manager.write().unwrap().update_username(&session_id, name)?;
            
            // Update in current session
            if let Some(session) = self.sessions.write().unwrap().get_mut(&session_id) {
                let user_id = self.current_user.read().unwrap().id.clone();
                if let Some(user) = session.users.get_mut(&user_id) {
                    user.name = name.to_string();
                }
            }
        }
        
        Ok(())
    }
    
    /// Update the current user's avatar
    pub fn update_avatar(&self, avatar: Option<&str>) -> Result<()> {
        // Update local user
        self.current_user.write().unwrap().avatar = avatar.map(|s| s.to_string());
        
        // Get current session ID
        if let Some(session_id) = self.current_session_id.read().unwrap().clone() {
            // Update user in session manager
            self.session_manager.write().unwrap().update_avatar(&session_id, avatar)?;
            
            // Update in current session
            if let Some(session) = self.sessions.write().unwrap().get_mut(&session_id) {
                let user_id = self.current_user.read().unwrap().id.clone();
                if let Some(user) = session.users.get_mut(&user_id) {
                    user.avatar = avatar.map(|s| s.to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Get statistics about collaboration
    pub fn get_statistics(&self) -> Result<CollaborationStatistics> {
        // Get session stats
        let session_stats = self.session_manager.read().unwrap().get_statistics()?;
        
        // Get presence stats
        let presence_stats = self.presence_manager.read().unwrap().get_statistics()?;
        
        // Get sync stats
        let sync_stats = self.sync_manager.read().unwrap().get_statistics()?;
        
        // Get RTC stats
        let rtc_stats = self.rtc_manager.read().unwrap().get_statistics()?;
        
        // Combine stats
        let stats = CollaborationStatistics {
            session_count: session_stats.session_count,
            total_users: session_stats.total_users,
            active_sessions: session_stats.active_sessions,
            cursor_updates: presence_stats.cursor_updates,
            selection_updates: presence_stats.selection_updates,
            messages_sent: sync_stats.messages_sent,
            messages_received: sync_stats.messages_received,
            sync_operations: sync_stats.sync_operations,
            conflicts_resolved: sync_stats.conflicts_resolved,
            calls_initiated: rtc_stats.calls_initiated,
            call_duration_seconds: rtc_stats.call_duration_seconds,
            current_session_id: self.current_session_id.read().unwrap().clone(),
            connection_status: *self.connection_status.read().unwrap(),
        };
        
        Ok(stats)
    }
}

/// Statistics about collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationStatistics {
    /// Number of sessions the user has participated in
    pub session_count: usize,
    
    /// Total users collaborated with
    pub total_users: usize,
    
    /// Number of currently active sessions
    pub active_sessions: usize,
    
    /// Number of cursor updates sent/received
    pub cursor_updates: usize,
    
    /// Number of selection updates sent/received
    pub selection_updates: usize,
    
    /// Number of messages sent
    pub messages_sent: usize,
    
    /// Number of messages received
    pub messages_received: usize,
    
    /// Number of sync operations
    pub sync_operations: usize,
    
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    
    /// Number of calls initiated
    pub calls_initiated: usize,
    
    /// Total duration of calls in seconds
    pub call_duration_seconds: u64,
    
    /// Current session ID
    pub current_session_id: Option<String>,
    
    /// Current connection status
    pub connection_status: ConnectionStatus,
}

// Global collaboration manager instance
lazy_static::lazy_static! {
    static ref COLLABORATION_MANAGER: Arc<RwLock<Option<CollaborationManager>>> = Arc::new(RwLock::new(None));
}

/// Initialize the collaboration system
pub fn init_collaboration(config: Option<CollaborationConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    
    // Create manager
    let manager = CollaborationManager::new(config)?;
    
    // Start services if enabled
    if manager.get_config().enabled {
        manager.start()?;
    }
    
    // Store globally
    *COLLABORATION_MANAGER.write().unwrap() = Some(manager);
    
    info!("Collaboration system initialized");
    
    Ok(())
}

/// Get a reference to the collaboration manager
pub fn get_collaboration_manager() -> Result<Arc<CollaborationManager>> {
    match COLLABORATION_MANAGER.read().unwrap().as_ref() {
        Some(manager) => Ok(Arc::new(manager.clone())),
        None => Err("Collaboration system not initialized".into()),
    }
}

// Helper functions for common operations

/// Generate a user color based on user ID
fn generate_user_color(user_id: &str) -> String {
    // Use a simple hash of the user ID to generate a hue value
    let hash: u32 = user_id.bytes().fold(0, |acc, byte| acc.wrapping_add(byte as u32));
    let hue = hash % 360;
    
    // Use HSL with high saturation and medium lightness for vibrant, distinguishable colors
    format!("hsl({}, 70%, 50%)", hue)
}
