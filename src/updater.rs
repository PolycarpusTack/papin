// Updater module for MCP Client
//
// This module handles the auto-update functionality for the application,
// leveraging Tauri's built-in updater system.

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager, Runtime};
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::feature_flags::FeatureFlags;
use crate::observability::metrics::{record_counter, record_gauge};

/// Update check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateStatus {
    /// Checking for updates
    Checking,
    
    /// Update available
    Available,
    
    /// No updates available
    UpToDate,
    
    /// Update downloaded and ready to install
    Ready,
    
    /// Update error
    Error,
    
    /// Update disabled
    Disabled,
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Current version
    pub current_version: String,
    
    /// Available version (if any)
    pub available_version: Option<String>,
    
    /// Update status
    pub status: UpdateStatus,
    
    /// Last check time
    pub last_check: Option<String>,
    
    /// Error message (if any)
    pub error: Option<String>,
}

/// Update manager for handling auto-updates
pub struct UpdateManager<R: Runtime> {
    /// Application handle
    app_handle: AppHandle<R>,
    
    /// Update status
    status: Arc<RwLock<UpdateStatus>>,
    
    /// Available version
    available_version: Arc<RwLock<Option<String>>>,
    
    /// Last check time
    last_check: Arc<RwLock<Option<Instant>>>,
    
    /// Update check interval in hours
    check_interval_hours: Arc<RwLock<u64>>,
    
    /// Whether auto-updates are enabled
    enabled: Arc<RwLock<bool>>,
    
    /// Last error message
    error: Arc<RwLock<Option<String>>>,
}

impl<R: Runtime> UpdateManager<R> {
    /// Create a new update manager
    pub fn new(app_handle: AppHandle<R>, feature_flags: FeatureFlags) -> Self {
        // Check if auto-update is enabled in feature flags
        let auto_update_enabled = feature_flags.contains(FeatureFlags::AUTO_UPDATE);
        
        Self {
            app_handle,
            status: Arc::new(RwLock::new(
                if auto_update_enabled {
                    UpdateStatus::UpToDate
                } else {
                    UpdateStatus::Disabled
                }
            )),
            available_version: Arc::new(RwLock::new(None)),
            last_check: Arc::new(RwLock::new(None)),
            check_interval_hours: Arc::new(RwLock::new(24)), // Default to daily checks
            enabled: Arc::new(RwLock::new(auto_update_enabled)),
            error: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Start the update manager
    pub async fn start(&self) -> Result<()> {
        if !*self.enabled.read().unwrap() {
            info!("Auto-update is disabled");
            return Ok(());
        }
        
        info!("Starting update manager");
        
        // Perform initial update check
        let _ = self.check_for_updates().await;
        
        // Setup periodic update checks
        let app_handle = self.app_handle.clone();
        let enabled = self.enabled.clone();
        let check_interval_hours = self.check_interval_hours.clone();
        
        // Spawn a background task to periodically check for updates
        tauri::async_runtime::spawn(async move {
            loop {
                // Sleep for the specified interval
                let interval_hours = *check_interval_hours.read().unwrap();
                let interval = Duration::from_secs(interval_hours * 3600);
                tokio::time::sleep(interval).await;
                
                // Skip if disabled
                if !*enabled.read().unwrap() {
                    continue;
                }
                
                // Check for updates
                let updater = app_handle.updater();
                match updater.check().await {
                    Ok(update) => {
                        if update.is_update_available() {
                            info!("Update available: {:?}", update);
                            record_counter("updater.update_available", 1.0, None);
                            
                            // Notify the user
                            app_handle.emit_all("update-available", update).unwrap_or_else(|e| {
                                error!("Failed to emit update-available event: {}", e);
                            });
                        } else {
                            debug!("No updates available");
                        }
                    }
                    Err(e) => {
                        error!("Failed to check for updates: {}", e);
                        record_counter("updater.check_error", 1.0, None);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Check for updates
    pub async fn check_for_updates(&self) -> Result<bool> {
        if !*self.enabled.read().unwrap() {
            *self.status.write().unwrap() = UpdateStatus::Disabled;
            return Ok(false);
        }
        
        *self.status.write().unwrap() = UpdateStatus::Checking;
        *self.last_check.write().unwrap() = Some(Instant::now());
        
        info!("Checking for updates");
        record_counter("updater.check", 1.0, None);
        
        let updater = self.app_handle.updater();
        match updater.check().await {
            Ok(update) => {
                if update.is_update_available() {
                    // Update available
                    info!("Update available: {:?}", update);
                    record_counter("updater.update_available", 1.0, None);
                    
                    // Extract version from update
                    if let Some(version) = update.current_version() {
                        if let Some(available) = update.latest_version() {
                            *self.available_version.write().unwrap() = Some(available.to_string());
                        }
                    }
                    
                    *self.status.write().unwrap() = UpdateStatus::Available;
                    *self.error.write().unwrap() = None;
                    
                    // Notify the user
                    self.app_handle.emit_all("update-available", update).unwrap_or_else(|e| {
                        error!("Failed to emit update-available event: {}", e);
                    });
                    
                    return Ok(true);
                } else {
                    // No update available
                    debug!("No updates available");
                    *self.status.write().unwrap() = UpdateStatus::UpToDate;
                    *self.error.write().unwrap() = None;
                    
                    return Ok(false);
                }
            }
            Err(e) => {
                // Update check failed
                error!("Failed to check for updates: {}", e);
                record_counter("updater.check_error", 1.0, None);
                
                *self.status.write().unwrap() = UpdateStatus::Error;
                *self.error.write().unwrap() = Some(e.to_string());
                
                return Err(e.into());
            }
        }
    }
    
    /// Get update info
    pub fn get_update_info(&self) -> UpdateInfo {
        let current_version = self.app_handle.package_info().version.to_string();
        let status = self.status.read().unwrap().clone();
        let available_version = self.available_version.read().unwrap().clone();
        let error = self.error.read().unwrap().clone();
        
        // Format last check time
        let last_check = if let Some(time) = *self.last_check.read().unwrap() {
            let elapsed = time.elapsed();
            let minutes = elapsed.as_secs() / 60;
            
            if minutes < 60 {
                Some(format!("{} minutes ago", minutes))
            } else {
                let hours = minutes / 60;
                Some(format!("{} hours ago", hours))
            }
        } else {
            None
        };
        
        UpdateInfo {
            current_version,
            available_version,
            status,
            last_check,
            error,
        }
    }
    
    /// Install update
    pub async fn install_update(&self) -> Result<()> {
        if !*self.enabled.read().unwrap() {
            return Err("Auto-update is disabled".into());
        }
        
        if *self.status.read().unwrap() != UpdateStatus::Available {
            return Err("No update available to install".into());
        }
        
        info!("Installing update");
        record_counter("updater.install", 1.0, None);
        
        // Let the Tauri's builtin updater handle the installation
        // This will typically restart the application
        match self.app_handle.updater().check().await {
            Ok(update) => {
                if update.is_update_available() {
                    info!("Installing update: {:?}", update);
                    Ok(())
                } else {
                    Err("No update available".into())
                }
            },
            Err(e) => {
                error!("Failed to install update: {}", e);
                record_counter("updater.install_error", 1.0, None);
                Err(e.into())
            }
        }
    }
    
    /// Enable or disable auto-updates
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        *self.enabled.write().unwrap() = enabled;
        
        if enabled {
            *self.status.write().unwrap() = UpdateStatus::UpToDate;
            info!("Auto-update enabled");
            
            // Trigger an update check
            let manager = self.clone();
            tauri::async_runtime::spawn(async move {
                let _ = manager.check_for_updates().await;
            });
        } else {
            *self.status.write().unwrap() = UpdateStatus::Disabled;
            info!("Auto-update disabled");
        }
        
        Ok(())
    }
    
    /// Set update check interval in hours
    pub fn set_check_interval(&self, hours: u64) -> Result<()> {
        if hours < 1 {
            return Err("Interval must be at least 1 hour".into());
        }
        
        *self.check_interval_hours.write().unwrap() = hours;
        info!("Update check interval set to {} hours", hours);
        
        Ok(())
    }
}

impl<R: Runtime> Clone for UpdateManager<R> {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            status: self.status.clone(),
            available_version: self.available_version.clone(),
            last_check: self.last_check.clone(),
            check_interval_hours: self.check_interval_hours.clone(),
            enabled: self.enabled.clone(),
            error: self.error.clone(),
        }
    }
}

// Global update manager
lazy_static::lazy_static! {
    static ref UPDATE_MANAGER: Arc<RwLock<Option<UpdateManager<tauri::Wry>>>> = Arc::new(RwLock::new(None));
}

/// Initialize the update manager
pub fn init_updater(app_handle: AppHandle<tauri::Wry>, feature_flags: FeatureFlags) -> Result<()> {
    let manager = UpdateManager::new(app_handle, feature_flags);
    
    // Store globally
    *UPDATE_MANAGER.write().unwrap() = Some(manager);
    
    // Start the update manager
    if let Some(manager) = &*UPDATE_MANAGER.read().unwrap() {
        tauri::async_runtime::spawn(async move {
            if let Err(e) = manager.start().await {
                error!("Failed to start update manager: {}", e);
            }
        });
    }
    
    info!("Update manager initialized");
    
    Ok(())
}

/// Register update commands
pub fn register_commands(app: &mut tauri::App) -> Result<()> {
    // Register commands for frontend to interact with updater
    app.register_command("checkForUpdates", |_app| {
        // Get the update manager
        if let Some(manager) = &*UPDATE_MANAGER.read().unwrap() {
            // Clone the manager for use in async block
            let manager_clone = manager.clone();
            
            // Spawn async task to check for updates
            tauri::async_runtime::spawn(async move {
                let _ = manager_clone.check_for_updates().await;
            });
        }
        
        Ok(())
    })?;
    
    app.register_command("getUpdateInfo", |_app| {
        if let Some(manager) = &*UPDATE_MANAGER.read().unwrap() {
            Ok(Some(manager.get_update_info()))
        } else {
            Ok(None)
        }
    })?;
    
    app.register_command("installUpdate", |_app| {
        if let Some(manager) = &*UPDATE_MANAGER.read().unwrap() {
            // Clone the manager for use in async block
            let manager_clone = manager.clone();
            
            // Spawn async task to install update
            tauri::async_runtime::spawn(async move {
                let _ = manager_clone.install_update().await;
            });
        }
        
        Ok(())
    })?;
    
    app.register_command("setUpdateEnabled", |app, enabled: bool| {
        if let Some(manager) = &*UPDATE_MANAGER.read().unwrap() {
            manager.set_enabled(enabled)?;
        }
        
        Ok(())
    })?;
    
    app.register_command("setUpdateCheckInterval", |app, hours: u64| {
        if let Some(manager) = &*UPDATE_MANAGER.read().unwrap() {
            manager.set_check_interval(hours)?;
        }
        
        Ok(())
    })?;
    
    Ok(())
}
