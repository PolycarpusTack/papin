// src/offline/mod.rs
// Enhanced to use platform-agnostic file operations and provider-based LLM system

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;

use crate::platform::fs::{platform_fs, PlatformFsError, PathExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

pub mod checkpointing;
pub mod llm;
pub mod sync;

use self::checkpointing::CheckpointManager;
use self::llm::discovery::{DiscoveryConfig, DiscoveryService};
use self::llm::migration::{MigrationConfig, MigrationOptions, MigrationService, MigrationStatus};
use self::llm::{LLMManager, provider::{GenerationOptions, ModelInfo, ProviderType}};
use self::sync::{SyncConfig, SyncManager};

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Connected
    Connected,
    /// Limited connectivity
    Limited,
    /// Disconnected
    Disconnected,
}

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
    /// Directory for offline data (will be resolved to platform-specific path)
    pub data_directory: Option<String>,
    /// LLM provider configuration
    pub llm_config: llm::LLMConfig,
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
            data_directory: None,
            llm_config: llm::LLMConfig::default(),
        }
    }
}

/// Main offline mode manager
pub struct OfflineManager {
    status: Arc<Mutex<OfflineStatus>>,
    network_status: Arc<Mutex<NetworkStatus>>,
    config: Arc<Mutex<OfflineConfig>>,
    llm_manager: Arc<TokioMutex<LLMManager>>,
    checkpoint_manager: Arc<Mutex<CheckpointManager>>,
    sync_manager: Arc<SyncManager>,
    llm_discovery: Arc<DiscoveryService>,
    llm_migration: Arc<MigrationService>,
    running: Arc<Mutex<bool>>,
    offline_dir: Arc<Mutex<PathBuf>>,
}

impl Default for OfflineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineManager {
    /// Create a new offline manager
    pub fn new() -> Self {
        // Determine the offline data directory
        let fs = platform_fs();
        let default_offline_dir = fs
            .app_data_dir("Papin")
            .map(|path| path.join("offline"))
            .unwrap_or_else(|_| PathBuf::from("offline"));

        Self {
            status: Arc::new(Mutex::new(OfflineStatus::Online)),
            network_status: Arc::new(Mutex::new(NetworkStatus::Connected)),
            config: Arc::new(Mutex::new(OfflineConfig::default())),
            llm_manager: Arc::new(TokioMutex::new(llm::create_llm_manager())),
            checkpoint_manager: Arc::new(Mutex::new(CheckpointManager::new())),
            sync_manager: Arc::new(SyncManager::new()),
            llm_discovery: Arc::new(DiscoveryService::new()),
            llm_migration: Arc::new(MigrationService::new()),
            running: Arc::new(Mutex::new(false)),
            offline_dir: Arc::new(Mutex::new(default_offline_dir)),
        }
    }

    /// Start the offline manager
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;

        // Ensure offline directory exists
        self.ensure_offline_directory();

        // Initialize LLM manager
        {
            let mut llm_manager = self.llm_manager.lock().await;
            if let Err(e) = llm_manager.initialize().await {
                error!("Failed to initialize LLM manager: {}", e);
            }
        }

        // Start sync manager
        self.sync_manager.start();

        // Initialize checkpoint manager
        {
            let mut checkpoint_manager = self.checkpoint_manager.lock().unwrap();
            if let Err(e) = checkpoint_manager.initialize() {
                error!("Failed to initialize checkpoint manager: {}", e);
            }
        }

        // Start LLM provider discovery service
        let discovery_service = self.llm_discovery.clone();
        tokio::spawn(async move {
            if let Err(e) = discovery_service.start_background_scanner().await {
                error!("Failed to start LLM provider discovery service: {}", e);
            }
        });

        // Start connectivity monitoring
        let status = self.status.clone();
        let network_status = self.network_status.clone();
        let config = self.config.clone();
        let running_clone = self.running.clone();

        // Initialize migration system
        let migration_service = self.llm_migration.clone();
        tokio::spawn(async move {
            match migration_service.detect_legacy_system().await {
                Ok(true) => {
                    info!("Legacy LLM system detected. Migration may be required.");
                    
                    // Check if auto-migrate is enabled
                    let migration_config = migration_service.get_config();
                    if migration_config.auto_migrate {
                        info!("Automatic migration is enabled. Starting migration...");
                        if let Err(e) = llm::migration::run_migration(&migration_service).await {
                            error!("Automatic migration failed: {}", e);
                        }
                    }
                },
                Ok(false) => {
                    debug!("No legacy LLM system detected.");
                },
                Err(e) => {
                    error!("Error detecting legacy LLM system: {}", e);
                }
            }
        });
        
        // Spawn network monitoring in a tokio task
        let network_monitor = {
            let status = status.clone();
            let network_status = network_status.clone();
            let config = config.clone();
            let running_clone = running_clone.clone();
            
            tokio::spawn(async move {
                while *running_clone.lock().unwrap() {
                    // Check network connectivity
                    let network_check = Self::check_network_connectivity().await;
                    let current_status = { *status.lock().unwrap() };
                    let config_values = { config.lock().unwrap().clone() };
                    
                    // Update network status
                    {
                        let mut ns = network_status.lock().unwrap();
                        *ns = network_check;
                    }
                    
                    // Handle auto-switching
                    if config_values.auto_switch {
                        match network_check {
                            NetworkStatus::Connected => {
                                if current_status == OfflineStatus::Offline {
                                    // Going back online
                                    debug!("Network connectivity restored, switching to online mode");
                                    
                                    {
                                        let mut status_lock = status.lock().unwrap();
                                        *status_lock = OfflineStatus::GoingOnline;
                                    }
                                    
                                    // Perform sync
                                    // (In a real implementation, we would initiate sync here)
                                    tokio::time::sleep(Duration::from_millis(1000)).await;
                                    
                                    {
                                        let mut status_lock = status.lock().unwrap();
                                        *status_lock = OfflineStatus::Online;
                                    }
                                    
                                    info!("Switched to online mode");
                                }
                            },
                            NetworkStatus::Disconnected => {
                                if current_status == OfflineStatus::Online {
                                    // Going offline
                                    debug!("Network connectivity lost, switching to offline mode");
                                    
                                    {
                                        let mut status_lock = status.lock().unwrap();
                                        *status_lock = OfflineStatus::GoingOffline;
                                    }
                                    
                                    // Create checkpoint
                                    // (In a real implementation, we would create a checkpoint here)
                                    tokio::time::sleep(Duration::from_millis(1000)).await;
                                    
                                    {
                                        let mut status_lock = status.lock().unwrap();
                                        *status_lock = OfflineStatus::Offline;
                                    }
                                    
                                    info!("Switched to offline mode");
                                }
                            },
                            NetworkStatus::Limited => {
                                // Handle limited connectivity - maybe warn the user
                                if current_status == OfflineStatus::Online && config_values.auto_switch {
                                    debug!("Limited network connectivity detected, consider switching to offline mode");
                                }
                            }
                        }
                    }
                    
                    // Sleep for the configured interval
                    tokio::time::sleep(Duration::from_secs(config_values.connectivity_check_interval)).await;
                }
            })
        };

        Ok(())
    }
    
    /// Ensure the offline directory exists
    fn ensure_offline_directory(&self) {
        let fs = platform_fs();
        let config = self.config.lock().unwrap();
        let mut offline_dir = self.offline_dir.lock().unwrap();
        
        // If a custom directory is specified, use it
        if let Some(custom_dir) = &config.data_directory {
            let dir = Path::new(custom_dir).normalize();
            *offline_dir = dir;
        }
        
        // Ensure the directory exists
        if let Err(e) = fs.ensure_dir_exists(&offline_dir) {
            error!("Failed to create offline directory at {}: {}", offline_dir.display(), e);
            
            // Fallback to a default directory
            let fallback_dir = fs.cache_dir("Papin")
                .map(|path| path.join("offline_fallback"))
                .unwrap_or_else(|_| PathBuf::from("offline_fallback"));
                
            if let Err(e) = fs.ensure_dir_exists(&fallback_dir) {
                error!("Failed to create fallback offline directory at {}: {}", fallback_dir.display(), e);
            } else {
                *offline_dir = fallback_dir;
                info!("Using fallback offline directory: {}", offline_dir.display());
            }
        } else {
            debug!("Using offline directory: {}", offline_dir.display());
        }
    }
    
    /// Stop the offline manager
    pub async fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        
        // Stop sync manager
        self.sync_manager.stop();
        
        // Stop LLM provider discovery service
        self.llm_discovery.stop_background_scanner();
    }
    
    /// Check network connectivity using platform-specific methods
    async fn check_network_connectivity() -> NetworkStatus {
        // In a real implementation, we would use platform-specific
        // network checking. For now, we'll just use a generic method.
        
        let result = Self::generic_network_check().await;
        if result {
            // Perform an additional check to see if we can reach the API
            let api_check = Self::check_api_connectivity().await;
            
            if api_check {
                NetworkStatus::Connected
            } else {
                NetworkStatus::Limited
            }
        } else {
            NetworkStatus::Disconnected
        }
    }
    
    /// Generic network connectivity check using ping
    async fn generic_network_check() -> bool {
        // Simple ping to check connectivity
        #[cfg(target_os = "windows")]
        let args = &["-n", "1", "-w", "2000", "8.8.8.8"];
        
        #[cfg(not(target_os = "windows"))]
        let args = &["-c", "1", "-W", "2", "8.8.8.8"];
        
        let result = tokio::process::Command::new("ping")
            .args(args)
            .output()
            .await;
        
        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Check if we can connect to the MCP API
    async fn check_api_connectivity() -> bool {
        // Use reqwest to check if we can connect to the API
        let client = reqwest::Client::new();
        let response = client.get("https://api.anthropic.com/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await;
            
        match response {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
    
    /// Manually switch to offline mode
    pub async fn go_offline(&self) -> Result<(), String> {
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
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = OfflineStatus::Offline;
        }
        
        info!("Manually switched to offline mode");
        Ok(())
    }
    
    /// Manually switch to online mode
    pub async fn go_online(&self) -> Result<(), String> {
        let current_status = { *self.status.lock().unwrap() };
        
        if current_status == OfflineStatus::Online {
            return Err("Already in online mode".to_string());
        }
        
        if current_status == OfflineStatus::GoingOnline {
            return Err("Already transitioning to online mode".to_string());
        }
        
        // Check connectivity
        let network_check = Self::check_network_connectivity().await;
        if network_check == NetworkStatus::Disconnected {
            return Err("Network is not available".to_string());
        }
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = OfflineStatus::GoingOnline;
        }
        
        // Perform sync
        // (In a real implementation, we would initiate sync here)
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
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
    
    /// Get current network status
    pub fn get_network_status(&self) -> NetworkStatus {
        *self.network_status.lock().unwrap()
    }
    
    /// Get offline configuration
    pub fn get_config(&self) -> OfflineConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// Update offline configuration
    pub async fn update_config(&self, config: OfflineConfig) -> Result<(), String> {
        // If data directory changed, update and ensure it exists
        let data_dir_changed = match (&self.config.lock().unwrap().data_directory, &config.data_directory) {
            (Some(old), Some(new)) => old != new,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (None, None) => false,
        };
        
        // Check if LLM config changed
        let llm_config_changed = {
            let old_config = self.config.lock().unwrap();
            old_config.llm_config != config.llm_config
        };
        
        // Update sync config
        self.sync_manager.update_config(config.sync.clone());
        
        // Update main config
        *self.config.lock().unwrap() = config.clone();
        
        // Update offline directory if needed
        if data_dir_changed {
            self.ensure_offline_directory();
        }
        
        // Update LLM manager if needed
        if llm_config_changed {
            let mut llm_manager = self.llm_manager.lock().await;
            *llm_manager = llm::LLMManager::with_config(config.llm_config);
            
            // Initialize the new manager
            if let Err(e) = llm_manager.initialize().await {
                return Err(format!("Failed to initialize LLM manager with new config: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Get the offline directory
    pub fn get_offline_directory(&self) -> PathBuf {
        self.offline_dir.lock().unwrap().clone()
    }
    
    /// Get the LLM manager
    pub fn llm_manager(&self) -> Arc<TokioMutex<LLMManager>> {
        self.llm_manager.clone()
    }
    
    /// Get the checkpoint manager
    pub fn get_checkpoint_manager(&self) -> Arc<Mutex<CheckpointManager>> {
        self.checkpoint_manager.clone()
    }
    
    /// Get the sync manager
    pub fn get_sync_manager(&self) -> Arc<SyncManager> {
        self.sync_manager.clone()
    }
    
    /// Get the LLM discovery service
    pub fn get_llm_discovery(&self) -> Arc<DiscoveryService> {
        self.llm_discovery.clone()
    }
    
    /// Get the LLM migration service
    pub fn get_llm_migration(&self) -> Arc<MigrationService> {
        self.llm_migration.clone()
    }
    
    /// Scan for LLM providers
    pub async fn scan_for_llm_providers(&self) -> Result<(), String> {
        match self.llm_discovery.scan_for_providers().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to scan for LLM providers: {}", e)),
        }
    }
    
    /// Run LLM migration
    pub async fn run_llm_migration(&self, options: MigrationOptions) -> Result<llm::migration::MigrationNotification, String> {
        match self.llm_migration.run_migration(options).await {
            Ok(notification) => Ok(notification),
            Err(e) => Err(format!("Failed to run LLM migration: {}", e)),
        }
    }
    
    /// Get available LLM providers
    pub async fn get_available_providers(&self) -> Result<Vec<ProviderType>, String> {
        let manager = self.llm_manager.lock().await;
        match manager.get_available_providers().await {
            Ok(providers) => Ok(providers),
            Err(e) => Err(format!("Failed to get available providers: {}", e)),
        }
    }
    
    /// Generate text using local LLM
    pub async fn generate_text(
        &self,
        prompt: &str,
        model_id: Option<&str>,
    ) -> Result<String, String> {
        // Check if offline mode is enabled and we're in offline mode
        let config = self.get_config();
        let status = self.get_status();
        
        if !config.enabled || !config.use_local_llm {
            return Err("Local LLM not enabled".to_string());
        }
        
        if status != OfflineStatus::Offline && !model_id.is_some() {
            return Err("Not in offline mode and no model specified".to_string());
        }
        
        // Generate text using the LLM manager
        let manager = self.llm_manager.lock().await;
        let model = model_id.unwrap_or("");
        
        match manager.generate_text(model, prompt, None).await {
            Ok(text) => Ok(text),
            Err(e) => Err(format!("Failed to generate text: {}", e)),
        }
    }
    
    /// List downloaded models
    pub async fn list_downloaded_models(&self) -> Result<Vec<ModelInfo>, String> {
        let manager = self.llm_manager.lock().await;
        match manager.list_downloaded_models().await {
            Ok(models) => Ok(models),
            Err(e) => Err(format!("Failed to list models: {}", e)),
        }
    }
    
    /// Download a model
    pub async fn download_model(&self, model_id: &str) -> Result<(), String> {
        let manager = self.llm_manager.lock().await;
        match manager.download_model(model_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to download model: {}", e)),
        }
    }
    
    /// Set the default model
    pub async fn set_default_model(&self, model_id: &str) -> Result<(), String> {
        // Update LLM config
        let mut config = self.get_config();
        config.llm_config.default_model = Some(model_id.to_string());
        
        // Update config
        self.update_config(config).await
    }
}

/// Create a new offline manager
pub async fn create_offline_manager() -> Arc<OfflineManager> {
    let manager = Arc::new(OfflineManager::new());
    if let Err(e) = manager.start().await {
        error!("Failed to start offline manager: {}", e);
    }
    manager
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_offline_manager_creation() {
        let manager = OfflineManager::new();
        
        // Check initial status
        assert_eq!(manager.get_status(), OfflineStatus::Online);
        
        // Check default config
        let config = manager.get_config();
        assert!(config.enabled);
        assert!(config.auto_switch);
        assert!(config.use_local_llm);
        
        // Check offline directory is set
        let offline_dir = manager.get_offline_directory();
        assert!(!offline_dir.to_string_lossy().is_empty());
    }
    
    #[tokio::test]
    async fn test_manual_offline_switching() {
        let manager = OfflineManager::new();
        manager.start().await.unwrap();
        
        // Switch to offline mode
        let result = manager.go_offline().await;
        assert!(result.is_ok());
        assert_eq!(manager.get_status(), OfflineStatus::Offline);
        
        // Try to switch to offline again (should fail)
        let result = manager.go_offline().await;
        assert!(result.is_err());
        
        // Stop the manager
        manager.stop().await;
    }
    
    #[tokio::test]
    async fn test_config_update() {
        let manager = OfflineManager::new();
        
        // Create a new config
        let mut config = OfflineConfig::default();
        config.enabled = false;
        config.auto_switch = false;
        config.data_directory = Some("/custom/path".to_string());
        
        // Update config
        let result = manager.update_config(config.clone()).await;
        assert!(result.is_ok());
        
        // Check if config was updated
        let updated_config = manager.get_config();
        assert_eq!(updated_config.enabled, config.enabled);
        assert_eq!(updated_config.auto_switch, config.auto_switch);
        assert_eq!(updated_config.data_directory, config.data_directory);
    }
}