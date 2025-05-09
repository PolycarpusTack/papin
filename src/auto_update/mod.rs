use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Wry};
use std::time::{Duration, SystemTime};
use reqwest::Client;
use log::{info, error, warn};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

/// Configuration for the auto-updater
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    /// Whether auto-updates are enabled
    pub enabled: bool,
    /// How often to check for updates (in hours)
    pub check_interval: u64,
    /// When the last check was performed
    pub last_check: Option<SystemTime>,
    /// Whether to download updates automatically
    pub auto_download: bool,
    /// Whether to install updates automatically
    pub auto_install: bool,
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 24, // Check once per day by default
            last_check: None,
            auto_download: true,
            auto_install: false,
        }
    }
}

/// Update information returned from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub pub_date: String,
    pub url: String,
    pub signature: String,
}

/// Manager for handling application updates
pub struct UpdateManager {
    config: Arc<Mutex<UpdaterConfig>>,
    client: Client,
    app: AppHandle<Wry>,
    config_path: PathBuf,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(app: AppHandle<Wry>) -> Self {
        let config_path = app
            .path_resolver()
            .app_config_dir()
            .unwrap()
            .join("updater_config.json");
        
        let config = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        error!("Failed to parse updater config: {}", e);
                        UpdaterConfig::default()
                    }
                },
                Err(e) => {
                    error!("Failed to read updater config: {}", e);
                    UpdaterConfig::default()
                }
            }
        } else {
            UpdaterConfig::default()
        };
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            config: Arc::new(Mutex::new(config)),
            client,
            app,
            config_path,
        }
    }

    /// Start the update checker
    pub async fn start(&self) {
        // Save default config if it doesn't exist
        if !self.config_path.exists() {
            self.save_config().await;
        }

        let config = self.config.lock().unwrap().clone();
        if !config.enabled {
            info!("Auto-updates are disabled");
            return;
        }

        // Check if it's time to check for updates
        let should_check = match config.last_check {
            Some(last_check) => {
                match SystemTime::now().duration_since(last_check) {
                    Ok(duration) => duration.as_secs() >= config.check_interval * 3600,
                    Err(_) => true, // System time error, check anyway
                }
            }
            None => true, // First time checking
        };

        if should_check {
            self.check_for_updates().await;
        }
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) {
        info!("Checking for updates...");
        
        // Update last check time
        {
            let mut config = self.config.lock().unwrap();
            config.last_check = Some(SystemTime::now());
        }
        self.save_config().await;

        // Use built-in Tauri updater to check for updates
        match tauri::updater::builder(self.app.clone()).check().await {
            Ok(update) => {
                if update.is_update_available() {
                    info!("Update available: {}", update.latest_version());
                    
                    let config = self.config.lock().unwrap().clone();
                    if config.auto_download {
                        // Emit event to notify frontend about available update
                        self.app.emit_all("update-available", update.latest_version()).unwrap();
                        
                        if config.auto_install {
                            info!("Auto-installing update...");
                            self.app.updater().unwrap().install().await.unwrap();
                        } else {
                            // Show update dialog
                            self.app.updater().unwrap().show().await.unwrap();
                        }
                    } else {
                        // Just notify about available update
                        self.app.emit_all("update-available", update.latest_version()).unwrap();
                    }
                } else {
                    info!("No updates available");
                }
            }
            Err(e) => {
                error!("Failed to check for updates: {}", e);
                self.app.emit_all("update-check-error", e.to_string()).unwrap();
            }
        }
    }

    /// Save the current updater configuration to disk
    async fn save_config(&self) {
        let config = self.config.lock().unwrap().clone();
        
        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }
        
        match serde_json::to_string_pretty(&config) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.config_path, json) {
                    error!("Failed to save updater config: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to serialize updater config: {}", e);
            }
        }
    }

    /// Update the updater configuration
    pub async fn update_config(&self, new_config: UpdaterConfig) {
        {
            let mut config = self.config.lock().unwrap();
            *config = new_config;
        }
        self.save_config().await;
    }

    /// Get the current updater configuration
    pub fn get_config(&self) -> UpdaterConfig {
        self.config.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_updater_config_default() {
        let config = UpdaterConfig::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval, 24);
        assert!(config.auto_download);
        assert!(!config.auto_install);
    }

    #[test]
    fn test_updater_config_serialization() {
        let config = UpdaterConfig {
            enabled: true,
            check_interval: 12,
            last_check: Some(SystemTime::now()),
            auto_download: false,
            auto_install: true,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: UpdaterConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.check_interval, deserialized.check_interval);
        assert_eq!(config.auto_download, deserialized.auto_download);
        assert_eq!(config.auto_install, deserialized.auto_install);
    }
}