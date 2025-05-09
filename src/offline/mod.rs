pub mod llm;
pub mod checkpointing;
pub mod sync;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};

use self::llm::LocalLLM;
use self::checkpointing::CheckpointManager;
use self::sync::{SyncManager, SyncConfig};

/// Offline mode status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OfflineStatus {
    /// Online mode (normal operation)
    Online,
    /// Offline mode (using local capabilities)
    Offline,
    /// Transitioning from online to offline
    GoingOffline,
    /// Transitioning from offline to online
    GoingOnline,
}

/// Offline mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Whether offline mode is enabled
    pub enabled: bool,
    /// Whether to automatically switch to offline mode when network is unavailable
    pub auto_switch: bool,
    /// Whether to use local LLM in offline mode
    pub use_local_llm: bool,
    /// How often to check network connectivity (in seconds)
    pub connectivity_check_interval: u64,
    /// Network timeout threshold (in milliseconds)
    pub network_timeout_ms: u64,
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,
    /// Sync configuration
    pub sync: SyncConfig,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_switch: true,
            use_local_llm: true,
            connectivity_check_interval: 30,
            network_timeout_ms: 5000,
            max_checkpoints: 10,
            sync: SyncConfig::default(),
        }
    }
}

/// Main offline mode manager
pub struct OfflineManager {
    status: Arc<Mutex<OfflineStatus>>,
    config: Arc<Mutex<OfflineConfig>>,
    llm: Arc<LocalLLM>,
    checkpoint_manager: Arc<Mutex<CheckpointManager>>,
    sync_manager: Arc<SyncManager>,
    running: Arc<Mutex<bool>>,
}

impl Default for OfflineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineManager {
    /// Create a new offline manager
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(OfflineStatus::Online)),
            config: Arc::new(Mutex::new(OfflineConfig::default())),
            llm: Arc::new(LocalLLM::new_manager()),
            checkpoint_manager: Arc::new(Mutex::new(CheckpointManager::new())),
            sync_manager: Arc::new(SyncManager::new()),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the offline manager
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        
        // Start sync manager
        self.sync_manager.start();
        
        // Initialize checkpoint manager
        {
            let mut checkpoint_manager = self.checkpoint_manager.lock().unwrap();
            if let Err(e) = checkpoint_manager.initialize() {
                error!("Failed to initialize checkpoint manager: {}", e);
            }
        }
        
        // Start connectivity monitoring
        let status = self.status.clone();
        let config = self.config.clone();
        let running_clone = self.running.clone();
        
        std::thread::spawn(move || {
            while *running_clone.lock().unwrap() {
                // Check network connectivity
                let is_online = Self::check_network_connectivity();
                let current_status = { *status.lock().unwrap() };
                let config_values = { config.lock().unwrap().clone() };
                
                if config_values.auto_switch {
                    // Automatically switch modes based on connectivity
                    if is_online && current_status == OfflineStatus::Offline {
                        // Going back online
                        debug!("Network connectivity restored, switching to online mode");
                        
                        {
                            let mut status_lock = status.lock().unwrap();
                            *status_lock = OfflineStatus::GoingOnline;
                        }
                        
                        // Perform sync
                        // (In a real implementation, we would initiate sync here)
                        std::thread::sleep(Duration::from_millis(1000));
                        
                        {
                            let mut status_lock = status.lock().unwrap();
                            *status_lock = OfflineStatus::Online;
                        }
                        
                        info!("Switched to online mode");
                    } else if !is_online && current_status == OfflineStatus::Online {
                        // Going offline
                        debug!("Network connectivity lost, switching to offline mode");
                        
                        {
                            let mut status_lock = status.lock().unwrap();
                            *status_lock = OfflineStatus::GoingOffline;
                        }
                        
                        // Create checkpoint
                        // (In a real implementation, we would create a checkpoint here)
                        std::thread::sleep(Duration::from_millis(1000));
                        
                        {
                            let mut status_lock = status.lock().unwrap();
                            *status_lock = OfflineStatus::Offline;
                        }
                        
                        info!("Switched to offline mode");
                    }
                }
                
                // Sleep for the configured interval
                std::thread::sleep(Duration::from_secs(config_values.connectivity_check_interval));
            }
        });
    }
    
    /// Stop the offline manager
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        
        // Stop sync manager
        self.sync_manager.stop();
    }
    
    /// Check network connectivity
    fn check_network_connectivity() -> bool {
        // Simple ping to check connectivity
        let result = std::process::Command::new("ping")
            .args(&["-c", "1", "-W", "2", "8.8.8.8"])
            .output();
        
        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Manually switch to offline mode
    pub fn go_offline(&self) -> Result<(), String> {
        let current_status = { *self.status.lock().unwrap() };
        
        if current_status == OfflineStatus::Offline {
            return Err("Already in offline mode".to_string());
        }
        
        if current_status == OfflineStatus::GoingOffline {
            return Err("Already transitioning to offline mode".to_string());
        }
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = OfflineStatus::GoingOffline;
        }
        
        // Create checkpoint
        // (In a real implementation, we would create a checkpoint here)
        std::thread::sleep(Duration::from_millis(1000));
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = OfflineStatus::Offline;
        }
        
        info!("Manually switched to offline mode");
        Ok(())
    }
    
    /// Manually switch to online mode
    pub fn go_online(&self) -> Result<(), String> {
        let current_status = { *self.status.lock().unwrap() };
        
        if current_status == OfflineStatus::Online {
            return Err("Already in online mode".to_string());
        }
        
        if current_status == OfflineStatus::GoingOnline {
            return Err("Already transitioning to online mode".to_string());
        }
        
        // Check connectivity
        if !Self::check_network_connectivity() {
            return Err("Network is not available".to_string());
        }
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = OfflineStatus::GoingOnline;
        }
        
        // Perform sync
        // (In a real implementation, we would initiate sync here)
        std::thread::sleep(Duration::from_millis(1000));
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = OfflineStatus::Online;
        }
        
        info!("Manually switched to online mode");
        Ok(())
    }
    
    /// Get current offline status
    pub fn get_status(&self) -> OfflineStatus {
        *self.status.lock().unwrap()
    }
    
    /// Get offline configuration
    pub fn get_config(&self) -> OfflineConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// Update offline configuration
    pub fn update_config(&self, config: OfflineConfig) {
        // Update sync config
        self.sync_manager.update_config(config.sync.clone());
        
        // Update main config
        *self.config.lock().unwrap() = config;
    }
    
    /// Get the local LLM manager
    pub fn get_llm(&self) -> Arc<LocalLLM> {
        self.llm.clone()
    }
    
    /// Get the checkpoint manager
    pub fn get_checkpoint_manager(&self) -> Arc<Mutex<CheckpointManager>> {
        self.checkpoint_manager.clone()
    }
    
    /// Get the sync manager
    pub fn get_sync_manager(&self) -> Arc<SyncManager> {
        self.sync_manager.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_offline_manager_creation() {
        let manager = OfflineManager::new();
        
        // Check initial status
        assert_eq!(manager.get_status(), OfflineStatus::Online);
        
        // Check default config
        let config = manager.get_config();
        assert!(config.enabled);
        assert!(config.auto_switch);
        assert!(config.use_local_llm);
    }
    
    #[test]
    fn test_manual_offline_switching() {
        let manager = OfflineManager::new();
        
        // Switch to offline mode
        let result = manager.go_offline();
        assert!(result.is_ok());
        assert_eq!(manager.get_status(), OfflineStatus::Offline);
        
        // Try to switch to offline again (should fail)
        let result = manager.go_offline();
        assert!(result.is_err());
        
        // Switch back to online mode (might fail if network is not available)
        let _ = manager.go_online();
    }
}