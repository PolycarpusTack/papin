use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};
use chrono::{DateTime, Utc};

/// Sync operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncOperationType {
    /// Create a new item
    Create,
    /// Update an existing item
    Update,
    /// Delete an item
    Delete,
}

/// Sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    /// Operation type
    pub operation_type: SyncOperationType,
    /// Key of the item
    pub key: String,
    /// New value of the item (null for delete)
    pub value: Option<String>,
    /// Timestamp of the operation
    pub timestamp: DateTime<Utc>,
    /// Device ID that created the operation
    pub device_id: String,
    /// Operation ID
    pub operation_id: String,
}

/// Sync conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// Key of the conflicted item
    pub key: String,
    /// Local operation
    pub local_operation: SyncOperation,
    /// Remote operation
    pub remote_operation: SyncOperation,
    /// Resolution strategy
    pub resolution: SyncResolutionStrategy,
    /// Resolved value
    pub resolved_value: Option<String>,
}

/// Sync resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncResolutionStrategy {
    /// Use local value
    UseLocal,
    /// Use remote value
    UseRemote,
    /// Merge values
    Merge,
    /// Manual resolution
    Manual,
}

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Number of local changes
    pub local_changes: usize,
    /// Number of remote changes
    pub remote_changes: usize,
    /// Number of conflicts
    pub conflicts: usize,
    /// Whether a sync is in progress
    pub syncing: bool,
    /// Current sync progress (0.0 to 1.0)
    pub progress: f32,
    /// Error message if sync failed
    pub error: Option<String>,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether sync is enabled
    pub enabled: bool,
    /// Sync interval in seconds
    pub interval_seconds: u64,
    /// Device ID
    pub device_id: String,
    /// Whether to sync automatically
    pub auto_sync: bool,
    /// Default conflict resolution strategy
    pub default_resolution: SyncResolutionStrategy,
    /// Whether to sync on startup
    pub sync_on_startup: bool,
    /// Whether to sync on shutdown
    pub sync_on_shutdown: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            device_id: generate_device_id(),
            auto_sync: true,
            default_resolution: SyncResolutionStrategy::UseRemote,
            sync_on_startup: true,
            sync_on_shutdown: true,
        }
    }
}

/// Sync result
pub struct SyncResult {
    /// Whether the sync was successful
    pub success: bool,
    /// Number of local changes applied
    pub local_applied: usize,
    /// Number of remote changes applied
    pub remote_applied: usize,
    /// Conflicts that occurred during sync
    pub conflicts: Vec<SyncConflict>,
    /// Error message if sync failed
    pub error: Option<String>,
}

/// Synchronization manager for offline capabilities
pub struct SyncManager {
    config: Arc<Mutex<SyncConfig>>,
    status: Arc<Mutex<SyncStatus>>,
    pending_operations: Arc<Mutex<Vec<SyncOperation>>>,
    resolved_conflicts: Arc<Mutex<HashMap<String, SyncConflict>>>,
    running: Arc<Mutex<bool>>,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(SyncConfig::default())),
            status: Arc::new(Mutex::new(SyncStatus {
                last_sync: None,
                local_changes: 0,
                remote_changes: 0,
                conflicts: 0,
                syncing: false,
                progress: 0.0,
                error: None,
            })),
            pending_operations: Arc::new(Mutex::new(Vec::new())),
            resolved_conflicts: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the sync manager
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        
        let config = self.config.clone();
        let status = self.status.clone();
        let pending_operations = self.pending_operations.clone();
        let resolved_conflicts = self.resolved_conflicts.clone();
        let running_clone = self.running.clone();
        
        // Start background sync task
        std::thread::spawn(move || {
            // Sync on startup if enabled
            {
                let cfg = config.lock().unwrap();
                if cfg.enabled && cfg.sync_on_startup {
                    drop(cfg);
                    let _ = Self::perform_sync(
                        &config,
                        &status,
                        &pending_operations,
                        &resolved_conflicts,
                    );
                }
            }
            
            // Background sync loop
            while *running_clone.lock().unwrap() {
                let interval = {
                    let cfg = config.lock().unwrap();
                    Duration::from_secs(cfg.interval_seconds)
                };
                
                // Sleep for the configured interval
                std::thread::sleep(interval);
                
                // Check if we're still running
                if !*running_clone.lock().unwrap() {
                    break;
                }
                
                // Check if auto-sync is enabled
                let should_sync = {
                    let cfg = config.lock().unwrap();
                    cfg.enabled && cfg.auto_sync
                };
                
                if should_sync {
                    let _ = Self::perform_sync(
                        &config,
                        &status,
                        &pending_operations,
                        &resolved_conflicts,
                    );
                }
            }
            
            // Sync on shutdown if enabled
            {
                let cfg = config.lock().unwrap();
                if cfg.enabled && cfg.sync_on_shutdown {
                    drop(cfg);
                    let _ = Self::perform_sync(
                        &config,
                        &status,
                        &pending_operations,
                        &resolved_conflicts,
                    );
                }
            }
        });
    }
    
    /// Stop the sync manager
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
    
    /// Perform a synchronization
    fn perform_sync(
        config: &Arc<Mutex<SyncConfig>>,
        status: &Arc<Mutex<SyncStatus>>,
        pending_operations: &Arc<Mutex<Vec<SyncOperation>>>,
        resolved_conflicts: &Arc<Mutex<HashMap<String, SyncConflict>>>,
    ) -> Result<SyncResult, String> {
        // Check if sync is enabled
        {
            let cfg = config.lock().unwrap();
            if !cfg.enabled {
                return Err("Sync is disabled".to_string());
            }
        }
        
        // Check if we're already syncing
        {
            let mut stat = status.lock().unwrap();
            if stat.syncing {
                return Err("Sync already in progress".to_string());
            }
            
            // Update status
            stat.syncing = true;
            stat.progress = 0.0;
            stat.error = None;
        }
        
        // Collect local changes
        let local_changes = {
            let operations = pending_operations.lock().unwrap();
            
            let mut changes = HashMap::new();
            for op in operations.iter() {
                changes.insert(op.key.clone(), op.value.clone().unwrap_or_default());
            }
            
            changes
        };
        
        // Update status
        {
            let mut stat = status.lock().unwrap();
            stat.local_changes = local_changes.len();
            stat.progress = 0.2;
        }
        
        // Simulate getting remote changes
        let remote_changes = generate_mock_remote_changes(&local_changes);
        
        // Update status
        {
            let mut stat = status.lock().unwrap();
            stat.remote_changes = remote_changes.len();
            stat.progress = 0.4;
        }
        
        // Perform sync (merging local and remote changes)
        let result = Self::sync(local_changes, remote_changes);
        
        // Update status
        {
            let mut stat = status.lock().unwrap();
            stat.conflicts = result.conflicts.len();
            stat.progress = 0.8;
        }
        
        // Store resolved conflicts
        {
            let mut conflicts = resolved_conflicts.lock().unwrap();
            for conflict in &result.conflicts {
                conflicts.insert(conflict.key.clone(), conflict.clone());
            }
        }
        
        // Clear pending operations if sync was successful
        if result.success {
            let mut operations = pending_operations.lock().unwrap();
            operations.clear();
        }
        
        // Update final status
        {
            let mut stat = status.lock().unwrap();
            stat.last_sync = Some(Utc::now());
            stat.progress = 1.0;
            stat.syncing = false;
            
            if !result.success {
                stat.error = result.error.clone();
            }
        }
        
        Ok(result)
    }
    
    /// Synchronize changes between local and remote
    pub fn sync(
        local_changes: HashMap<String, String>,
        remote_changes: HashMap<String, String>,
    ) -> SyncResult {
        let start = Instant::now();
        info!("Starting sync: {} local changes, {} remote changes",
              local_changes.len(), remote_changes.len());
        
        // Collect all keys
        let mut all_keys = HashSet::new();
        for key in local_changes.keys() {
            all_keys.insert(key.clone());
        }
        for key in remote_changes.keys() {
            all_keys.insert(key.clone());
        }
        
        let mut local_applied = 0;
        let mut remote_applied = 0;
        let mut conflicts = Vec::new();
        
        // Process each key
        for key in all_keys {
            let local_value = local_changes.get(&key);
            let remote_value = remote_changes.get(&key);
            
            match (local_value, remote_value) {
                // Both have changes
                (Some(local), Some(remote)) => {
                    if local == remote {
                        // Same changes, no conflict
                        debug!("Key '{}': Same changes in local and remote", key);
                    } else {
                        // Conflict
                        debug!("Key '{}': Conflict between local and remote", key);
                        
                        // Create conflict
                        let conflict = SyncConflict {
                            key: key.clone(),
                            local_operation: SyncOperation {
                                operation_type: SyncOperationType::Update,
                                key: key.clone(),
                                value: Some(local.clone()),
                                timestamp: Utc::now(),
                                device_id: "local".to_string(),
                                operation_id: generate_operation_id(),
                            },
                            remote_operation: SyncOperation {
                                operation_type: SyncOperationType::Update,
                                key: key.clone(),
                                value: Some(remote.clone()),
                                timestamp: Utc::now(),
                                device_id: "remote".to_string(),
                                operation_id: generate_operation_id(),
                            },
                            resolution: SyncResolutionStrategy::UseRemote,
                            resolved_value: Some(remote.clone()),
                        };
                        
                        conflicts.push(conflict);
                        remote_applied += 1;
                    }
                }
                // Only local has changes
                (Some(local), None) => {
                    debug!("Key '{}': Only local changes", key);
                    local_applied += 1;
                }
                // Only remote has changes
                (None, Some(remote)) => {
                    debug!("Key '{}': Only remote changes", key);
                    remote_applied += 1;
                }
                // Neither has changes (shouldn't happen)
                (None, None) => {
                    warn!("Key '{}': Neither local nor remote has changes", key);
                }
            }
        }
        
        let elapsed = start.elapsed();
        info!("Sync completed in {:?}: {} local applied, {} remote applied, {} conflicts",
              elapsed, local_applied, remote_applied, conflicts.len());
        
        SyncResult {
            success: true,
            local_applied,
            remote_applied,
            conflicts,
            error: None,
        }
    }
    
    /// Get current sync status
    pub fn get_status(&self) -> SyncStatus {
        self.status.lock().unwrap().clone()
    }
    
    /// Get sync configuration
    pub fn get_config(&self) -> SyncConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// Update sync configuration
    pub fn update_config(&self, config: SyncConfig) {
        *self.config.lock().unwrap() = config;
    }
    
    /// Add a pending operation
    pub fn add_operation(&self, operation: SyncOperation) {
        let mut operations = self.pending_operations.lock().unwrap();
        operations.push(operation);
        
        // Update status
        let mut status = self.status.lock().unwrap();
        status.local_changes = operations.len();
    }
    
    /// Get all pending operations
    pub fn get_pending_operations(&self) -> Vec<SyncOperation> {
        self.pending_operations.lock().unwrap().clone()
    }
    
    /// Get all conflicts
    pub fn get_conflicts(&self) -> Vec<SyncConflict> {
        self.resolved_conflicts.lock().unwrap().values().cloned().collect()
    }
    
    /// Resolve a conflict
    pub fn resolve_conflict(&self, key: &str, resolution: SyncResolutionStrategy, value: Option<String>) -> Result<(), String> {
        let mut conflicts = self.resolved_conflicts.lock().unwrap();
        
        if let Some(conflict) = conflicts.get_mut(key) {
            conflict.resolution = resolution;
            conflict.resolved_value = value;
            Ok(())
        } else {
            Err(format!("Conflict for key '{}' not found", key))
        }
    }
    
    /// Manual sync
    pub fn manual_sync(&self) -> Result<SyncResult, String> {
        Self::perform_sync(
            &self.config,
            &self.status,
            &self.pending_operations,
            &self.resolved_conflicts,
        )
    }
}

/// Generate a unique device ID
fn generate_device_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Generate a unique operation ID
fn generate_operation_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Generate mock remote changes for testing
fn generate_mock_remote_changes(local_changes: &HashMap<String, String>) -> HashMap<String, String> {
    let mut remote_changes = HashMap::new();
    
    // Copy some local changes to simulate same changes
    for (key, value) in local_changes.iter() {
        if rand::random::<f32>() < 0.3 {
            remote_changes.insert(key.clone(), value.clone());
        }
    }
    
    // Add some conflicting changes
    for (key, _) in local_changes.iter() {
        if rand::random::<f32>() < 0.1 && !remote_changes.contains_key(key) {
            remote_changes.insert(key.clone(), format!("remote_{}", key));
        }
    }
    
    // Add some remote-only changes
    for i in 0..5 {
        let key = format!("remote_key_{}", i);
        if !local_changes.contains_key(&key) {
            remote_changes.insert(key, format!("remote_value_{}", i));
        }
    }
    
    remote_changes
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_with_no_conflicts() {
        // Create test data
        let mut local_changes = HashMap::new();
        local_changes.insert("key1".to_string(), "value1".to_string());
        local_changes.insert("key2".to_string(), "value2".to_string());
        
        let mut remote_changes = HashMap::new();
        remote_changes.insert("key3".to_string(), "value3".to_string());
        remote_changes.insert("key4".to_string(), "value4".to_string());
        
        // Perform sync
        let result = SyncManager::sync(local_changes, remote_changes);
        
        // Verify result
        assert!(result.success);
        assert_eq!(result.local_applied, 2);
        assert_eq!(result.remote_applied, 2);
        assert_eq!(result.conflicts.len(), 0);
    }
    
    #[test]
    fn test_sync_with_conflicts() {
        // Create test data
        let mut local_changes = HashMap::new();
        local_changes.insert("key1".to_string(), "local_value1".to_string());
        local_changes.insert("key2".to_string(), "value2".to_string());
        
        let mut remote_changes = HashMap::new();
        remote_changes.insert("key1".to_string(), "remote_value1".to_string());
        remote_changes.insert("key3".to_string(), "value3".to_string());
        
        // Perform sync
        let result = SyncManager::sync(local_changes, remote_changes);
        
        // Verify result
        assert!(result.success);
        assert_eq!(result.local_applied, 1);
        assert_eq!(result.remote_applied, 2);
        assert_eq!(result.conflicts.len(), 1);
        
        // Verify conflict
        let conflict = &result.conflicts[0];
        assert_eq!(conflict.key, "key1");
        assert_eq!(conflict.local_operation.value, Some("local_value1".to_string()));
        assert_eq!(conflict.remote_operation.value, Some("remote_value1".to_string()));
        assert_eq!(conflict.resolution, SyncResolutionStrategy::UseRemote);
        assert_eq!(conflict.resolved_value, Some("remote_value1".to_string()));
    }
    
    #[test]
    fn test_sync_with_same_changes() {
        // Create test data
        let mut local_changes = HashMap::new();
        local_changes.insert("key1".to_string(), "value1".to_string());
        local_changes.insert("key2".to_string(), "value2".to_string());
        
        let mut remote_changes = HashMap::new();
        remote_changes.insert("key1".to_string(), "value1".to_string()); // Same as local
        remote_changes.insert("key3".to_string(), "value3".to_string());
        
        // Perform sync
        let result = SyncManager::sync(local_changes, remote_changes);
        
        // Verify result
        assert!(result.success);
        assert_eq!(result.local_applied, 1); // key2
        assert_eq!(result.remote_applied, 1); // key3
        assert_eq!(result.conflicts.len(), 0);
    }
}
