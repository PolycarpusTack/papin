// Granular Permission Management System
//
// This module provides a comprehensive permission management system that allows users
// to control access to features, data, and system resources at a granular level.

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::config::config_path;
use crate::error::Result;
use crate::observability::metrics::{record_counter, record_gauge};
use crate::security::PermissionLevel;

const PERMISSIONS_FILE: &str = "permissions.json";

/// A permission setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Description of what the permission grants access to
    pub description: String,
    
    /// Access level
    pub level: PermissionLevel,
    
    /// Category of the permission
    pub category: String,
    
    /// When the permission was last modified
    pub last_modified: SystemTime,
    
    /// How many times this permission has been used
    pub usage_count: usize,
    
    /// Whether this permission is required for core functionality
    pub required: bool,
}

/// A permission request event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Unique identifier for the request
    pub id: String,
    
    /// Permission being requested
    pub permission_id: String,
    
    /// Reason for the request
    pub reason: String,
    
    /// When the request was made
    pub timestamp: SystemTime,
    
    /// Whether the request was granted
    pub granted: bool,
    
    /// Application context for the request
    pub context: HashMap<String, String>,
}

/// User interaction type for permission requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionInteractionType {
    /// No interaction, use default or saved setting
    None,
    
    /// Display a non-modal notification
    Notification,
    
    /// Show a modal dialog requiring user action
    Modal,
}

/// Callback for permission requests
pub type PermissionCallback = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// Permission Manager
pub struct PermissionManager {
    /// Default permission level for new permissions
    default_level: PermissionLevel,
    
    /// Whether to prompt for permissions interactively
    interactive: bool,
    
    /// Known permissions
    permissions: Arc<RwLock<HashMap<String, Permission>>>,
    
    /// Permission request history
    request_history: Arc<RwLock<Vec<PermissionRequest>>>,
    
    /// Callback for requesting permissions from the user
    permission_callback: Arc<RwLock<Option<PermissionCallback>>>,
    
    /// Lock for file operations
    file_lock: Arc<Mutex<()>>,
}

impl PermissionManager {
    /// Create a new Permission Manager
    pub fn new(
        default_level: PermissionLevel,
        interactive: bool,
    ) -> Result<Self> {
        Ok(Self {
            default_level,
            interactive,
            permissions: Arc::new(RwLock::new(HashMap::new())),
            request_history: Arc::new(RwLock::new(Vec::new())),
            permission_callback: Arc::new(RwLock::new(None)),
            file_lock: Arc::new(Mutex::new(())),
        })
    }
    
    /// Start the permission management service
    pub fn start_service(&self) -> Result<()> {
        // Load permissions from disk
        self.load_permissions()?;
        
        // Initialize default permissions if needed
        self.initialize_default_permissions()?;
        
        info!("Permission management service started with {} permissions", 
            self.permissions.read().unwrap().len());
        
        record_gauge("security.permissions.count", self.permissions.read().unwrap().len() as f64, None);
        record_counter("security.permissions.service_started", 1.0, None);
        
        Ok(())
    }
    
    /// Initialize default permissions
    fn initialize_default_permissions(&self) -> Result<()> {
        let mut permissions = self.permissions.write().unwrap();
        let now = SystemTime::now();
        
        // Define default permissions if not already present
        self.add_permission_internal(
            &mut permissions,
            "storage_access",
            "Storage Access",
            "Allow the application to read and write files on your device",
            PermissionLevel::AskFirstTime,
            "File System",
            true,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "network_access",
            "Network Access",
            "Allow the application to connect to the internet",
            PermissionLevel::AlwaysAllow,
            "Network",
            true,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "offline_mode",
            "Offline Mode",
            "Allow the application to operate without internet connection",
            PermissionLevel::AlwaysAllow,
            "Network",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "telemetry",
            "Usage Telemetry",
            "Allow the application to collect anonymous usage data",
            PermissionLevel::AskFirstTime,
            "Privacy",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "api_access",
            "API Access",
            "Allow the application to connect to cloud services",
            PermissionLevel::AlwaysAllow,
            "Network",
            true,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "clipboard",
            "Clipboard Access",
            "Allow the application to read from and write to the clipboard",
            PermissionLevel::AskFirstTime,
            "Privacy",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "notifications",
            "Show Notifications",
            "Allow the application to show system notifications",
            PermissionLevel::AskFirstTime,
            "System",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "location",
            "Access Location",
            "Allow the application to access your approximate location",
            PermissionLevel::AskEveryTime,
            "Privacy",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "camera",
            "Camera Access",
            "Allow the application to access your camera",
            PermissionLevel::AskEveryTime,
            "Privacy",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "microphone",
            "Microphone Access",
            "Allow the application to access your microphone",
            PermissionLevel::AskEveryTime,
            "Privacy",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "background_tasks",
            "Background Tasks",
            "Allow the application to run tasks in the background",
            PermissionLevel::AskFirstTime,
            "System",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "autostart",
            "Start at Login",
            "Allow the application to start automatically when you log in",
            PermissionLevel::AskFirstTime,
            "System",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "sync",
            "Sync Data",
            "Allow the application to synchronize your data with the cloud",
            PermissionLevel::AskFirstTime,
            "Data",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "e2ee",
            "End-to-End Encryption",
            "Allow the application to encrypt your data before sending it to the cloud",
            PermissionLevel::AlwaysAllow,
            "Security",
            false,
            now,
        )?;
        
        self.add_permission_internal(
            &mut permissions,
            "secure_enclave",
            "Secure Enclave Access",
            "Allow the application to store credentials in your device's secure enclave",
            PermissionLevel::AlwaysAllow,
            "Security",
            false,
            now,
        )?;
        
        // Save permissions to disk
        self.save_permissions()?;
        
        Ok(())
    }
    
    /// Add a permission internally
    fn add_permission_internal(
        &self,
        permissions: &mut HashMap<String, Permission>,
        id: &str,
        name: &str,
        description: &str,
        level: PermissionLevel,
        category: &str,
        required: bool,
        last_modified: SystemTime,
    ) -> Result<()> {
        // Only add if not already present
        if !permissions.contains_key(id) {
            let permission = Permission {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                level,
                category: category.to_string(),
                last_modified,
                usage_count: 0,
                required,
            };
            
            permissions.insert(id.to_string(), permission);
            debug!("Added permission: {}", id);
        }
        
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(
        &self,
        default_level: PermissionLevel,
        interactive: bool,
    ) -> Result<()> {
        // Update settings
        let mut this = unsafe { &mut *(self as *const Self as *mut Self) };
        this.default_level = default_level;
        this.interactive = interactive;
        
        info!("Updated permission manager configuration");
        
        Ok(())
    }
    
    /// Set the callback for requesting permissions
    pub fn set_permission_callback(&self, callback: PermissionCallback) -> Result<()> {
        let mut cb = self.permission_callback.write().unwrap();
        *cb = Some(callback);
        
        Ok(())
    }
    
    /// Load permissions from disk
    fn load_permissions(&self) -> Result<()> {
        let _lock = self.file_lock.lock().unwrap();
        let permissions_path = config_path(PERMISSIONS_FILE);
        
        if !permissions_path.exists() {
            // No permissions file yet
            return Ok(());
        }
        
        let permissions_data = fs::read_to_string(&permissions_path)
            .map_err(|e| format!("Failed to read permissions file: {}", e))?;
        
        let loaded_permissions: HashMap<String, Permission> = serde_json::from_str(&permissions_data)
            .map_err(|e| format!("Failed to parse permissions file: {}", e))?;
        
        // Update permissions
        let mut permissions = self.permissions.write().unwrap();
        *permissions = loaded_permissions;
        
        debug!("Loaded {} permissions", permissions.len());
        
        Ok(())
    }
    
    /// Save permissions to disk
    fn save_permissions(&self) -> Result<()> {
        let _lock = self.file_lock.lock().unwrap();
        let permissions_path = config_path(PERMISSIONS_FILE);
        
        let permissions = self.permissions.read().unwrap();
        
        let permissions_data = serde_json::to_string_pretty(&*permissions)
            .map_err(|e| format!("Failed to serialize permissions: {}", e))?;
        
        fs::write(&permissions_path, permissions_data)
            .map_err(|e| format!("Failed to write permissions file: {}", e))?;
        
        debug!("Saved {} permissions", permissions.len());
        
        Ok(())
    }
    
    /// Add a new permission
    pub fn add_permission(
        &self,
        id: &str,
        name: &str,
        description: &str,
        level: Option<PermissionLevel>,
        category: &str,
        required: bool,
    ) -> Result<()> {
        let mut permissions = self.permissions.write().unwrap();
        
        // Use provided level or default
        let level = level.unwrap_or(self.default_level);
        
        self.add_permission_internal(
            &mut permissions,
            id,
            name,
            description,
            level,
            category,
            required,
            SystemTime::now(),
        )?;
        
        // Save changes
        drop(permissions);
        self.save_permissions()?;
        
        info!("Added permission: {}", id);
        record_counter("security.permissions.added", 1.0, None);
        
        Ok(())
    }
    
    /// Get a permission
    pub fn get_permission(&self, id: &str) -> Result<Permission> {
        let permissions = self.permissions.read().unwrap();
        
        if let Some(permission) = permissions.get(id) {
            Ok(permission.clone())
        } else {
            Err(format!("Permission '{}' not found", id).into())
        }
    }
    
    /// Set permission level
    pub fn set_permission_level(
        &self,
        id: &str,
        level: PermissionLevel,
    ) -> Result<()> {
        let mut permissions = self.permissions.write().unwrap();
        
        if let Some(permission) = permissions.get_mut(id) {
            // Update permission
            permission.level = level;
            permission.last_modified = SystemTime::now();
            
            // Save changes
            drop(permissions);
            self.save_permissions()?;
            
            info!("Updated permission level for {}: {:?}", id, level);
            record_counter("security.permissions.updated", 1.0, None);
            
            Ok(())
        } else {
            Err(format!("Permission '{}' not found", id).into())
        }
    }
    
    /// Get all permissions
    pub fn get_all_permissions(&self) -> Result<Vec<Permission>> {
        let permissions = self.permissions.read().unwrap();
        
        Ok(permissions.values().cloned().collect())
    }
    
    /// Check if a permission is granted
    pub fn check_permission(&self, id: &str) -> Result<bool> {
        let mut permissions = self.permissions.write().unwrap();
        
        if let Some(permission) = permissions.get_mut(id) {
            // Check permission level
            let granted = match permission.level {
                PermissionLevel::AlwaysAllow => true,
                PermissionLevel::NeverAllow => false,
                PermissionLevel::AskFirstTime | PermissionLevel::AskEveryTime => {
                    // Need to request permission interactively
                    false
                }
            };
            
            // Update usage count
            if granted {
                permission.usage_count += 1;
                
                // Save changes periodically
                if permission.usage_count % 10 == 0 {
                    drop(permissions);
                    self.save_permissions()?;
                }
            }
            
            record_counter("security.permissions.checked", 1.0, None);
            
            Ok(granted)
        } else if self.default_level == PermissionLevel::AlwaysAllow {
            // Permission not found, but default is to allow
            warn!("Permission '{}' not found, using default (allow)", id);
            Ok(true)
        } else {
            // Permission not found and default is not to allow
            warn!("Permission '{}' not found, using default (deny)", id);
            Ok(false)
        }
    }
    
    /// Request permission interactively
    pub fn request_permission(
        &self,
        id: &str,
        reason: &str,
    ) -> Result<bool> {
        let mut permissions = self.permissions.write().unwrap();
        
        if let Some(permission) = permissions.get_mut(id) {
            // Check permission level
            let (granted, interaction_type) = match permission.level {
                PermissionLevel::AlwaysAllow => (true, PermissionInteractionType::None),
                PermissionLevel::NeverAllow => (false, PermissionInteractionType::None),
                PermissionLevel::AskFirstTime => {
                    if permission.usage_count > 0 {
                        // Already asked and granted before
                        (true, PermissionInteractionType::None)
                    } else {
                        // Need to ask for the first time
                        (false, PermissionInteractionType::Modal)
                    }
                },
                PermissionLevel::AskEveryTime => {
                    // Always ask
                    (false, PermissionInteractionType::Modal)
                }
            };
            
            if granted {
                // Permission already granted
                permission.usage_count += 1;
                
                // Record the request
                let request = PermissionRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    permission_id: id.to_string(),
                    reason: reason.to_string(),
                    timestamp: SystemTime::now(),
                    granted: true,
                    context: HashMap::new(),
                };
                
                self.request_history.write().unwrap().push(request);
                
                // Save changes periodically
                if permission.usage_count % 10 == 0 {
                    drop(permissions);
                    self.save_permissions()?;
                }
                
                record_counter("security.permissions.granted", 1.0, None);
                
                return Ok(true);
            } else if !self.interactive || interaction_type == PermissionInteractionType::None {
                // Interactive mode disabled or no interaction needed, deny
                
                // Record the request
                let request = PermissionRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    permission_id: id.to_string(),
                    reason: reason.to_string(),
                    timestamp: SystemTime::now(),
                    granted: false,
                    context: HashMap::new(),
                };
                
                self.request_history.write().unwrap().push(request);
                
                record_counter("security.permissions.denied", 1.0, None);
                
                return Ok(false);
            } else {
                // Need to request permission interactively
                let callback = self.permission_callback.read().unwrap();
                
                if let Some(ref cb) = *callback {
                    // Call the callback to request permission
                    let granted = cb(
                        &format!("{}: {}", permission.name, permission.description),
                        reason,
                    );
                    
                    // Record the request
                    let request = PermissionRequest {
                        id: uuid::Uuid::new_v4().to_string(),
                        permission_id: id.to_string(),
                        reason: reason.to_string(),
                        timestamp: SystemTime::now(),
                        granted,
                        context: HashMap::new(),
                    };
                    
                    self.request_history.write().unwrap().push(request);
                    
                    // If granted, update usage count
                    if granted {
                        permission.usage_count += 1;
                        record_counter("security.permissions.granted", 1.0, None);
                    } else {
                        record_counter("security.permissions.denied", 1.0, None);
                    }
                    
                    // If this was AskFirstTime, update level to Always/Never based on response
                    if permission.level == PermissionLevel::AskFirstTime {
                        permission.level = if granted {
                            PermissionLevel::AlwaysAllow
                        } else {
                            PermissionLevel::NeverAllow
                        };
                        
                        permission.last_modified = SystemTime::now();
                    }
                    
                    // Save changes
                    drop(permissions);
                    self.save_permissions()?;
                    
                    return Ok(granted);
                } else {
                    // No callback registered, deny by default
                    warn!("No permission callback registered, denying permission request");
                    
                    // Record the request
                    let request = PermissionRequest {
                        id: uuid::Uuid::new_v4().to_string(),
                        permission_id: id.to_string(),
                        reason: reason.to_string(),
                        timestamp: SystemTime::now(),
                        granted: false,
                        context: HashMap::new(),
                    };
                    
                    self.request_history.write().unwrap().push(request);
                    
                    record_counter("security.permissions.denied", 1.0, None);
                    
                    return Ok(false);
                }
            }
        } else {
            // Permission not found
            warn!("Permission '{}' not found when requesting", id);
            
            // Create a new permission with default level
            drop(permissions);
            self.add_permission(
                id,
                id, // Use ID as name temporarily
                "Dynamically requested permission",
                Some(self.default_level),
                "Dynamic",
                false,
            )?;
            
            // Recursive call now that the permission exists
            self.request_permission(id, reason)
        }
    }
    
    /// Reset a permission to default
    pub fn reset_permission(
        &self,
        id: &str,
    ) -> Result<()> {
        let mut permissions = self.permissions.write().unwrap();
        
        if let Some(permission) = permissions.get_mut(id) {
            // Reset to default level
            permission.level = self.default_level;
            permission.last_modified = SystemTime::now();
            permission.usage_count = 0;
            
            // Save changes
            drop(permissions);
            self.save_permissions()?;
            
            info!("Reset permission: {}", id);
            record_counter("security.permissions.reset", 1.0, None);
            
            Ok(())
        } else {
            Err(format!("Permission '{}' not found", id).into())
        }
    }
    
    /// Reset all permissions to default
    pub fn reset_all_permissions(&self) -> Result<()> {
        let mut permissions = self.permissions.write().unwrap();
        
        // Reset all permissions
        for (_, permission) in permissions.iter_mut() {
            permission.level = self.default_level;
            permission.last_modified = SystemTime::now();
            permission.usage_count = 0;
        }
        
        // Save changes
        drop(permissions);
        self.save_permissions()?;
        
        info!("Reset all permissions to default");
        record_counter("security.permissions.reset_all", 1.0, None);
        
        Ok(())
    }
    
    /// Get permission request history
    pub fn get_request_history(&self) -> Result<Vec<PermissionRequest>> {
        Ok(self.request_history.read().unwrap().clone())
    }
    
    /// Clear permission request history
    pub fn clear_request_history(&self) -> Result<()> {
        self.request_history.write().unwrap().clear();
        
        info!("Cleared permission request history");
        record_counter("security.permissions.history_cleared", 1.0, None);
        
        Ok(())
    }
    
    /// Get permission statistics
    pub fn get_statistics(&self) -> Result<PermissionStatistics> {
        let permissions = self.permissions.read().unwrap();
        let requests = self.request_history.read().unwrap();
        
        // Count by level
        let mut count_by_level = HashMap::new();
        count_by_level.insert(PermissionLevel::AlwaysAllow, 0);
        count_by_level.insert(PermissionLevel::AskFirstTime, 0);
        count_by_level.insert(PermissionLevel::AskEveryTime, 0);
        count_by_level.insert(PermissionLevel::NeverAllow, 0);
        
        for permission in permissions.values() {
            *count_by_level.entry(permission.level).or_insert(0) += 1;
        }
        
        // Count by category
        let mut count_by_category = HashMap::new();
        for permission in permissions.values() {
            *count_by_category.entry(permission.category.clone()).or_insert(0) += 1;
        }
        
        // Count requests
        let mut granted_count = 0;
        let mut denied_count = 0;
        
        for request in requests.iter() {
            if request.granted {
                granted_count += 1;
            } else {
                denied_count += 1;
            }
        }
        
        // Find most used permissions
        let mut usage_counts: Vec<(&String, &Permission)> = permissions.iter().collect();
        usage_counts.sort_by(|a, b| b.1.usage_count.cmp(&a.1.usage_count));
        
        let most_used = usage_counts.iter()
            .take(5)
            .map(|(id, permission)| ((*id).clone(), permission.usage_count))
            .collect();
        
        // Calculate stats
        let stats = PermissionStatistics {
            total_permissions: permissions.len(),
            count_by_level,
            count_by_category,
            total_requests: requests.len(),
            granted_count,
            denied_count,
            most_used_permissions: most_used,
        };
        
        Ok(stats)
    }
    
    /// Export permissions to JSON
    pub fn export_permissions(&self) -> Result<String> {
        let permissions = self.permissions.read().unwrap();
        
        let json = serde_json::to_string_pretty(&*permissions)
            .map_err(|e| format!("Failed to serialize permissions: {}", e))?;
        
        Ok(json)
    }
    
    /// Import permissions from JSON
    pub fn import_permissions(&self, json: &str) -> Result<()> {
        let imported_permissions: HashMap<String, Permission> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse permissions JSON: {}", e))?;
        
        // Update permissions
        let mut permissions = self.permissions.write().unwrap();
        *permissions = imported_permissions;
        
        // Save changes
        drop(permissions);
        self.save_permissions()?;
        
        info!("Imported {} permissions", permissions.len());
        record_counter("security.permissions.imported", 1.0, None);
        
        Ok(())
    }
}

/// Permission statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatistics {
    /// Total number of permissions
    pub total_permissions: usize,
    
    /// Count of permissions by level
    pub count_by_level: HashMap<PermissionLevel, usize>,
    
    /// Count of permissions by category
    pub count_by_category: HashMap<String, usize>,
    
    /// Total number of permission requests
    pub total_requests: usize,
    
    /// Number of granted requests
    pub granted_count: usize,
    
    /// Number of denied requests
    pub denied_count: usize,
    
    /// Most frequently used permissions (id, count)
    pub most_used_permissions: Vec<(String, usize)>,
}

/// Default implementation for the permission callback
pub fn default_permission_callback(permission: &str, reason: &str) -> bool {
    // In a real app, this would show a UI dialog
    // For now, just log and deny
    warn!("Permission request: {} - Reason: {}", permission, reason);
    warn!("No UI available, denying by default");
    
    false
}
