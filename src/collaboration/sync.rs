// Synchronization System
//
// This module handles synchronization of conversation data between devices and users:
// - Message synchronization
// - Conflict resolution
// - Cross-device state persistence
// - Operational transformation for concurrent edits

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::models::messages::{Conversation, Message};
use crate::observability::metrics::{record_counter, record_gauge, record_histogram};

/// Operation type for synchronization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operation {
    /// Add a message
    AddMessage(Message),
    
    /// Update a message
    UpdateMessage {
        id: String,
        content: String,
    },
    
    /// Delete a message
    DeleteMessage(String),
    
    /// Update conversation metadata
    UpdateMetadata {
        key: String,
        value: String,
    },
    
    /// Set conversation title
    SetTitle(String),
}

/// Change record for syncing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Change ID
    pub id: String,
    
    /// User ID who made the change
    pub user_id: String,
    
    /// Device ID where the change originated
    pub device_id: String,
    
    /// Session ID
    pub session_id: String,
    
    /// Conversation ID
    pub conversation_id: String,
    
    /// Operation to apply
    pub operation: Operation,
    
    /// Timestamp when the change was created
    pub timestamp: SystemTime,
    
    /// Vector clock for causality tracking
    pub vector_clock: HashMap<String, u64>,
}

/// Status of a sync operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Sync successful
    Success,
    
    /// Sync in progress
    InProgress,
    
    /// Sync failed
    Failed,
    
    /// Sync conflict detected
    Conflict,
}

/// Active conversation being synchronized
struct SyncedConversation {
    /// Conversation ID
    id: String,
    
    /// Session ID
    session_id: String,
    
    /// Last synchronized time
    last_sync: Instant,
    
    /// Vector clock tracking causality
    vector_clock: HashMap<String, u64>,
    
    /// Pending changes to be applied
    pending_changes: VecDeque<Change>,
    
    /// Applied changes
    applied_changes: Vec<Change>,
    
    /// Last sync status
    last_status: SyncStatus,
}

/// Synchronization manager for cross-device data sync
pub struct SyncManager {
    /// User ID
    user_id: String,
    
    /// Device ID
    device_id: String,
    
    /// Sync interval in milliseconds
    sync_interval_ms: u64,
    
    /// Active conversations being synced
    conversations: HashMap<String, SyncedConversation>,
    
    /// Outgoing changes queue
    outgoing_changes: Arc<Mutex<VecDeque<Change>>>,
    
    /// Incoming changes queue
    incoming_changes: Arc<Mutex<VecDeque<Change>>>,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
    
    /// Statistics
    statistics: Arc<RwLock<SyncStatistics>>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(user_id: String, device_id: String, sync_interval_ms: u64) -> Result<Self> {
        Ok(Self {
            user_id,
            device_id,
            sync_interval_ms,
            conversations: HashMap::new(),
            outgoing_changes: Arc::new(Mutex::new(VecDeque::new())),
            incoming_changes: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(RwLock::new(false)),
            statistics: Arc::new(RwLock::new(SyncStatistics {
                messages_sent: 0,
                messages_received: 0,
                sync_operations: 0,
                conflicts_resolved: 0,
                bytes_sent: 0,
                bytes_received: 0,
                last_sync_time: None,
            })),
        })
    }
    
    /// Start the sync service
    pub fn start(&self) -> Result<()> {
        // Mark as running
        *self.running.write().unwrap() = true;
        
        // Start the sync thread
        let running = self.running.clone();
        let incoming_changes = self.incoming_changes.clone();
        let outgoing_changes = self.outgoing_changes.clone();
        let statistics = self.statistics.clone();
        let sync_interval = self.sync_interval_ms;
        
        thread::spawn(move || {
            while *running.read().unwrap() {
                // Process incoming changes
                let mut incoming = incoming_changes.lock().unwrap();
                if !incoming.is_empty() {
                    let count = incoming.len();
                    
                    // In a real implementation, we would apply these changes
                    debug!("Processing {} incoming changes", count);
                    
                    // Update statistics
                    let mut stats = statistics.write().unwrap();
                    stats.messages_received += count;
                    stats.sync_operations += count;
                    
                    // Clear processed changes
                    incoming.clear();
                }
                
                // Process outgoing changes
                let mut outgoing = outgoing_changes.lock().unwrap();
                if !outgoing.is_empty() {
                    let count = outgoing.len();
                    
                    // In a real implementation, we would send these changes to other clients
                    debug!("Sending {} outgoing changes", count);
                    
                    // Update statistics
                    let mut stats = statistics.write().unwrap();
                    stats.messages_sent += count;
                    stats.sync_operations += count;
                    stats.last_sync_time = Some(SystemTime::now());
                    
                    // Clear processed changes
                    outgoing.clear();
                }
                
                // Sleep for sync interval
                thread::sleep(Duration::from_millis(sync_interval));
            }
        });
        
        info!("Sync service started with interval {}ms", self.sync_interval_ms);
        
        Ok(())
    }
    
    /// Stop the sync service
    pub fn stop(&self) -> Result<()> {
        *self.running.write().unwrap() = false;
        
        info!("Sync service stopped");
        
        Ok(())
    }
    
    /// Set sync interval
    pub fn set_sync_interval(&mut self, interval_ms: u64) -> Result<()> {
        self.sync_interval_ms = interval_ms;
        
        info!("Updated sync interval to {}ms", interval_ms);
        
        Ok(())
    }
    
    /// Initialize synchronization for a session
    pub fn init_session(&mut self, session_id: &str, conversation_id: &str) -> Result<()> {
        // Create a synced conversation
        let conversation = SyncedConversation {
            id: conversation_id.to_string(),
            session_id: session_id.to_string(),
            last_sync: Instant::now(),
            vector_clock: HashMap::new(),
            pending_changes: VecDeque::new(),
            applied_changes: Vec::new(),
            last_status: SyncStatus::Success,
        };
        
        // Store it
        self.conversations.insert(conversation_id.to_string(), conversation);
        
        info!("Initialized sync for conversation {} in session {}", conversation_id, session_id);
        
        Ok(())
    }
    
    /// Join an existing sync session
    pub fn join_session(&mut self, session_id: &str, conversation_id: &str) -> Result<()> {
        // First check if we already have this conversation
        if self.conversations.contains_key(conversation_id) {
            return Ok(());
        }
        
        // Create a synced conversation
        let conversation = SyncedConversation {
            id: conversation_id.to_string(),
            session_id: session_id.to_string(),
            last_sync: Instant::now(),
            vector_clock: HashMap::new(),
            pending_changes: VecDeque::new(),
            applied_changes: Vec::new(),
            last_status: SyncStatus::Success,
        };
        
        // Store it
        self.conversations.insert(conversation_id.to_string(), conversation);
        
        info!("Joined sync for conversation {} in session {}", conversation_id, session_id);
        
        Ok(())
    }
    
    /// Leave a sync session
    pub fn leave_session(&mut self, session_id: &str) -> Result<()> {
        // Find conversation for this session
        let conversation_ids: Vec<String> = self.conversations.iter()
            .filter(|(_, conv)| conv.session_id == session_id)
            .map(|(id, _)| id.clone())
            .collect();
            
        // Remove conversations
        for id in conversation_ids {
            self.conversations.remove(&id);
        }
        
        info!("Left sync for session {}", session_id);
        
        Ok(())
    }
    
    /// Synchronize a conversation
    pub fn sync_conversation(&mut self, session_id: &str, conversation: &Conversation) -> Result<()> {
        let conversation_id = &conversation.id;
        
        // Get synced conversation
        let synced = match self.conversations.get_mut(conversation_id) {
            Some(conv) => conv,
            None => {
                // Initialize new sync
                self.init_session(session_id, conversation_id)?;
                self.conversations.get_mut(conversation_id).unwrap()
            }
        };
        
        // Update last sync time
        synced.last_sync = Instant::now();
        
        // Update statistics
        let mut stats = self.statistics.write().unwrap();
        stats.sync_operations += 1;
        stats.last_sync_time = Some(SystemTime::now());
        
        record_counter("collaboration.sync_operation", 1.0, None);
        
        Ok(())
    }
    
    /// Send a message through sync
    pub fn send_message(&mut self, session_id: &str, message: &Message) -> Result<()> {
        let conversation_id = &message.conversation_id;
        
        // Get synced conversation
        let synced = match self.conversations.get_mut(conversation_id) {
            Some(conv) => conv,
            None => {
                // Initialize new sync
                self.init_session(session_id, conversation_id)?;
                self.conversations.get_mut(conversation_id).unwrap()
            }
        };
        
        // Increment vector clock for this user
        let user_count = synced.vector_clock.entry(self.user_id.clone()).or_insert(0);
        *user_count += 1;
        
        // Create change record
        let change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            session_id: session_id.to_string(),
            conversation_id: conversation_id.to_string(),
            operation: Operation::AddMessage(message.clone()),
            timestamp: SystemTime::now(),
            vector_clock: synced.vector_clock.clone(),
        };
        
        // Add to outgoing changes
        self.outgoing_changes.lock().unwrap().push_back(change.clone());
        
        // Add to applied changes
        synced.applied_changes.push(change);
        
        // Update last sync time
        synced.last_sync = Instant::now();
        
        // Update statistics
        let mut stats = self.statistics.write().unwrap();
        stats.messages_sent += 1;
        stats.last_sync_time = Some(SystemTime::now());
        
        record_counter("collaboration.message_sent", 1.0, None);
        
        Ok(())
    }
    
    /// Process an incoming change
    pub fn process_change(&mut self, change: Change) -> Result<SyncStatus> {
        let conversation_id = &change.conversation_id;
        
        // Get synced conversation
        let synced = match self.conversations.get_mut(conversation_id) {
            Some(conv) => conv,
            None => {
                // Initialize new sync if session exists
                if let Some((session_id, _)) = self.conversations.iter()
                    .find(|(_, conv)| conv.session_id == change.session_id) {
                    self.init_session(&session_id, conversation_id)?;
                    self.conversations.get_mut(conversation_id).unwrap()
                } else {
                    return Err(format!("No active session for change in conversation {}", conversation_id).into());
                }
            }
        };
        
        // Check for conflicts
        let has_conflict = self.detect_conflict(&synced.vector_clock, &change.vector_clock);
        
        if has_conflict {
            // Handle conflict based on operation type
            info!("Conflict detected for change in conversation {}", conversation_id);
            
            // In a real implementation, we would resolve the conflict
            // For now, just accept the incoming change
            
            // Update statistics
            let mut stats = self.statistics.write().unwrap();
            stats.conflicts_resolved += 1;
            
            record_counter("collaboration.conflict_resolved", 1.0, None);
        }
        
        // Merge vector clocks
        self.merge_vector_clocks(&mut synced.vector_clock, &change.vector_clock);
        
        // Apply the change
        // In a real implementation, we would apply the change to the conversation
        
        // Add to applied changes
        synced.applied_changes.push(change);
        
        // Update last sync time
        synced.last_sync = Instant::now();
        
        // Update statistics
        let mut stats = self.statistics.write().unwrap();
        stats.messages_received += 1;
        stats.sync_operations += 1;
        stats.last_sync_time = Some(SystemTime::now());
        
        record_counter("collaboration.change_processed", 1.0, None);
        
        Ok(if has_conflict { SyncStatus::Conflict } else { SyncStatus::Success })
    }
    
    /// Detect conflicts between vector clocks
    fn detect_conflict(&self, local: &HashMap<String, u64>, remote: &HashMap<String, u64>) -> bool {
        // Check if either clock has events the other doesn't know about
        let mut local_ahead = false;
        let mut remote_ahead = false;
        
        // Check all keys in local clock
        for (user, local_count) in local {
            let remote_count = remote.get(user).unwrap_or(&0);
            
            if local_count > remote_count {
                local_ahead = true;
            }
        }
        
        // Check all keys in remote clock
        for (user, remote_count) in remote {
            let local_count = local.get(user).unwrap_or(&0);
            
            if remote_count > local_count {
                remote_ahead = true;
            }
        }
        
        // Conflict if both clocks have events the other doesn't know about
        local_ahead && remote_ahead
    }
    
    /// Merge vector clocks
    fn merge_vector_clocks(&self, local: &mut HashMap<String, u64>, remote: &HashMap<String, u64>) {
        for (user, remote_count) in remote {
            let local_count = local.entry(user.clone()).or_insert(0);
            *local_count = (*local_count).max(*remote_count);
        }
    }
    
    /// Get statistics about sync
    pub fn get_statistics(&self) -> Result<SyncStatistics> {
        Ok(self.statistics.read().unwrap().clone())
    }
}

/// Statistics about synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatistics {
    /// Number of messages sent
    pub messages_sent: usize,
    
    /// Number of messages received
    pub messages_received: usize,
    
    /// Number of sync operations
    pub sync_operations: usize,
    
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    
    /// Bytes sent
    pub bytes_sent: usize,
    
    /// Bytes received
    pub bytes_received: usize,
    
    /// Last sync time
    pub last_sync_time: Option<SystemTime>,
}
