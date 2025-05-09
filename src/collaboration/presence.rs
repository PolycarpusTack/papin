// Presence Management System
//
// This module handles user presence information including:
// - Cursor positions
// - Text selections
// - Active element tracking
// - User online status

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::observability::metrics::{record_counter, record_gauge};

/// Cursor position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    /// User ID
    pub user_id: String,
    
    /// Device ID
    pub device_id: String,
    
    /// X coordinate (normalized 0.0-1.0)
    pub x: f32,
    
    /// Y coordinate (normalized 0.0-1.0)
    pub y: f32,
    
    /// Element ID the cursor is over
    pub element_id: Option<String>,
    
    /// Timestamp of the update
    pub timestamp: SystemTime,
}

/// Text selection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    /// User ID
    pub user_id: String,
    
    /// Device ID
    pub device_id: String,
    
    /// Start element ID
    pub start_id: String,
    
    /// End element ID
    pub end_id: String,
    
    /// Start offset within element
    pub start_offset: usize,
    
    /// End offset within element
    pub end_offset: usize,
    
    /// Timestamp of the update
    pub timestamp: SystemTime,
}

/// Active element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveElement {
    /// User ID
    pub user_id: String,
    
    /// Device ID
    pub device_id: String,
    
    /// Element ID
    pub element_id: String,
    
    /// Timestamp of the update
    pub timestamp: SystemTime,
}

/// User presence update message for network communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceUpdate {
    /// Cursor position update
    Cursor(CursorPosition),
    
    /// Selection update
    Selection(Selection),
    
    /// Active element update
    ActiveElement(ActiveElement),
    
    /// User joined
    UserJoined {
        user_id: String,
        device_id: String,
        timestamp: SystemTime,
    },
    
    /// User left
    UserLeft {
        user_id: String,
        device_id: String,
        timestamp: SystemTime,
    },
}

/// Presence tracking for a session
struct SessionPresence {
    /// Session ID
    session_id: String,
    
    /// Cursor positions by user ID
    cursors: HashMap<String, CursorPosition>,
    
    /// Selections by user ID
    selections: HashMap<String, Selection>,
    
    /// Active elements by user ID
    active_elements: HashMap<String, ActiveElement>,
    
    /// Last update time
    last_update: Instant,
}

impl SessionPresence {
    fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            cursors: HashMap::new(),
            selections: HashMap::new(),
            active_elements: HashMap::new(),
            last_update: Instant::now(),
        }
    }
}

/// Presence manager for keeping track of users' presence
pub struct PresenceManager {
    /// User ID
    user_id: String,
    
    /// Device ID
    device_id: String,
    
    /// Whether presence features are enabled
    enabled: bool,
    
    /// Active sessions by session ID
    sessions: HashMap<String, SessionPresence>,
    
    /// Update queue for sending to other clients
    update_queue: Arc<Mutex<Vec<(String, PresenceUpdate)>>>,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
    
    /// Statistics
    cursor_updates: Arc<RwLock<usize>>,
    selection_updates: Arc<RwLock<usize>>,
}

impl PresenceManager {
    /// Create a new presence manager
    pub fn new(user_id: String, device_id: String, enabled: bool) -> Result<Self> {
        Ok(Self {
            user_id,
            device_id,
            enabled,
            sessions: HashMap::new(),
            update_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            cursor_updates: Arc::new(RwLock::new(0)),
            selection_updates: Arc::new(RwLock::new(0)),
        })
    }
    
    /// Start the presence service
    pub fn start(&self) -> Result<()> {
        if !self.enabled {
            info!("Presence service is disabled");
            return Ok(());
        }
        
        // Mark as running
        *self.running.write().unwrap() = true;
        
        // Start the update thread
        let running = self.running.clone();
        let update_queue = self.update_queue.clone();
        
        thread::spawn(move || {
            while *running.read().unwrap() {
                // Process presence updates
                let updates = {
                    let mut queue = update_queue.lock().unwrap();
                    let updates = queue.clone();
                    queue.clear();
                    updates
                };
                
                if !updates.is_empty() {
                    // In a real implementation, we would send these updates to other clients
                    // For now, just log them
                    debug!("Sending {} presence updates", updates.len());
                    
                    // In a real implementation, we'd use WebSockets or similar to send updates
                    // self.send_presence_updates(&updates);
                }
                
                // Sleep briefly
                thread::sleep(Duration::from_millis(50));
            }
        });
        
        info!("Presence service started");
        
        Ok(())
    }
    
    /// Stop the presence service
    pub fn stop(&self) -> Result<()> {
        *self.running.write().unwrap() = false;
        
        info!("Presence service stopped");
        
        Ok(())
    }
    
    /// Enable or disable presence tracking
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mut this = unsafe { &mut *(self as *const Self as *mut Self) };
        
        if this.enabled == enabled {
            return Ok(());
        }
        
        this.enabled = enabled;
        
        if enabled {
            // Start service if not running
            if !*this.running.read().unwrap() {
                this.start()?;
            }
        } else {
            // Stop service if running
            if *this.running.read().unwrap() {
                this.stop()?;
            }
        }
        
        Ok(())
    }
    
    /// Join a session for presence tracking
    pub fn join_session(&mut self, session_id: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Create a new session presence
        let session = SessionPresence::new(session_id);
        self.sessions.insert(session_id.to_string(), session);
        
        // Send join notification
        let update = PresenceUpdate::UserJoined {
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            timestamp: SystemTime::now(),
        };
        
        self.queue_update(session_id, update)?;
        
        info!("Joined presence tracking for session {}", session_id);
        
        Ok(())
    }
    
    /// Leave a session
    pub fn leave_session(&mut self, session_id: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Remove session
        self.sessions.remove(session_id);
        
        // Send leave notification
        let update = PresenceUpdate::UserLeft {
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            timestamp: SystemTime::now(),
        };
        
        self.queue_update(session_id, update)?;
        
        info!("Left presence tracking for session {}", session_id);
        
        Ok(())
    }
    
    /// Update cursor position
    pub fn update_cursor_position(
        &mut self, 
        session_id: &str, 
        x: f32, 
        y: f32, 
        element_id: Option<&str>
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let now = SystemTime::now();
        
        // Create cursor position
        let cursor = CursorPosition {
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            x,
            y,
            element_id: element_id.map(|s| s.to_string()),
            timestamp: now,
        };
        
        // Update local state
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.cursors.insert(self.user_id.clone(), cursor.clone());
            session.last_update = Instant::now();
        } else {
            return Err(format!("Session {} not found", session_id).into());
        }
        
        // Send update
        let update = PresenceUpdate::Cursor(cursor);
        self.queue_update(session_id, update)?;
        
        // Update stats
        *self.cursor_updates.write().unwrap() += 1;
        record_counter("collaboration.cursor_updates", 1.0, None);
        
        Ok(())
    }
    
    /// Update selection
    pub fn update_selection(
        &mut self,
        session_id: &str,
        start_id: &str,
        end_id: &str,
        start_offset: usize,
        end_offset: usize,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let now = SystemTime::now();
        
        // Create selection
        let selection = Selection {
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            start_id: start_id.to_string(),
            end_id: end_id.to_string(),
            start_offset,
            end_offset,
            timestamp: now,
        };
        
        // Update local state
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.selections.insert(self.user_id.clone(), selection.clone());
            session.last_update = Instant::now();
        } else {
            return Err(format!("Session {} not found", session_id).into());
        }
        
        // Send update
        let update = PresenceUpdate::Selection(selection);
        self.queue_update(session_id, update)?;
        
        // Update stats
        *self.selection_updates.write().unwrap() += 1;
        record_counter("collaboration.selection_updates", 1.0, None);
        
        Ok(())
    }
    
    /// Get all cursor positions for a session
    pub fn get_cursors(&self, session_id: &str) -> Result<HashMap<String, CursorPosition>> {
        if !self.enabled {
            return Ok(HashMap::new());
        }
        
        if let Some(session) = self.sessions.get(session_id) {
            // Return all cursors except our own
            let cursors = session.cursors.iter()
                .filter(|(id, _)| id != &&self.user_id)
                .map(|(id, cursor)| (id.clone(), cursor.clone()))
                .collect();
                
            return Ok(cursors);
        }
        
        Err(format!("Session {} not found", session_id).into())
    }
    
    /// Get all selections for a session
    pub fn get_selections(&self, session_id: &str) -> Result<HashMap<String, Selection>> {
        if !self.enabled {
            return Ok(HashMap::new());
        }
        
        if let Some(session) = self.sessions.get(session_id) {
            // Return all selections except our own
            let selections = session.selections.iter()
                .filter(|(id, _)| id != &&self.user_id)
                .map(|(id, selection)| (id.clone(), selection.clone()))
                .collect();
                
            return Ok(selections);
        }
        
        Err(format!("Session {} not found", session_id).into())
    }
    
    /// Handle a presence update from another user
    pub fn handle_update(&mut self, session_id: &str, update: PresenceUpdate) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let session = match self.sessions.get_mut(session_id) {
            Some(session) => session,
            None => return Err(format!("Session {} not found", session_id).into()),
        };
        
        match update {
            PresenceUpdate::Cursor(cursor) => {
                // Ignore our own cursor
                if cursor.user_id != self.user_id {
                    session.cursors.insert(cursor.user_id.clone(), cursor);
                    *self.cursor_updates.write().unwrap() += 1;
                }
            },
            PresenceUpdate::Selection(selection) => {
                // Ignore our own selection
                if selection.user_id != self.user_id {
                    session.selections.insert(selection.user_id.clone(), selection);
                    *self.selection_updates.write().unwrap() += 1;
                }
            },
            PresenceUpdate::ActiveElement(element) => {
                // Ignore our own active element
                if element.user_id != self.user_id {
                    session.active_elements.insert(element.user_id.clone(), element);
                }
            },
            PresenceUpdate::UserJoined { user_id, .. } => {
                info!("User {} joined session {}", user_id, session_id);
            },
            PresenceUpdate::UserLeft { user_id, .. } => {
                info!("User {} left session {}", user_id, session_id);
                
                // Remove user's presence data
                session.cursors.remove(&user_id);
                session.selections.remove(&user_id);
                session.active_elements.remove(&user_id);
            },
        }
        
        session.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Queue an update to be sent to other clients
    fn queue_update(&self, session_id: &str, update: PresenceUpdate) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut queue = self.update_queue.lock().unwrap();
        queue.push((session_id.to_string(), update));
        
        Ok(())
    }
    
    /// Get statistics about presence
    pub fn get_statistics(&self) -> Result<PresenceStatistics> {
        Ok(PresenceStatistics {
            cursor_updates: *self.cursor_updates.read().unwrap(),
            selection_updates: *self.selection_updates.read().unwrap(),
            active_sessions: self.sessions.len(),
        })
    }
}

/// Statistics about presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceStatistics {
    /// Number of cursor updates
    pub cursor_updates: usize,
    
    /// Number of selection updates
    pub selection_updates: usize,
    
    /// Number of active sessions
    pub active_sessions: usize,
}
