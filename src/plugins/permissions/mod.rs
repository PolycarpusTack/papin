use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Permission manager
pub struct PermissionManager {
    /// Permissions of plugins
    plugin_permissions: RwLock<HashMap<String, HashSet<String>>>,
    /// Pending permission requests
    pending_requests: RwLock<HashMap<String, HashSet<String>>>,
    /// Permission settings
    settings: RwLock<PermissionSettings>,
}

/// Permission settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSettings {
    /// Automatically allowed permissions for all plugins
    #[serde(default)]
    pub auto_allowed: HashSet<String>,
    /// Restricted permissions that require confirmation
    #[serde(default)]
    pub restricted: HashSet<String>,
    /// Denied permissions that cannot be used
    #[serde(default)]
    pub denied: HashSet<String>,
    /// Plugin-specific permission overrides
    #[serde(default)]
    pub plugin_overrides: HashMap<String, PluginPermissionOverride>,
}

/// Plugin-specific permission override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissionOverride {
    /// Allowed permissions for this plugin
    #[serde(default)]
    pub allowed: HashSet<String>,
    /// Denied permissions for this plugin
    #[serde(default)]
    pub denied: HashSet<String>,
}

impl Default for PermissionSettings {
    fn default() -> Self {
        let mut auto_allowed = HashSet::new();
        let mut restricted = HashSet::new();
        let mut denied = HashSet::new();
        
        // Set up default permissions
        auto_allowed.insert("conversations:read".to_string());
        auto_allowed.insert("models:read".to_string());
        auto_allowed.insert("ui:display".to_string());
        
        restricted.insert("conversations:write".to_string());
        restricted.insert("models:use".to_string());
        restricted.insert("network:github.com".to_string());
        restricted.insert("ui:interact".to_string());
        restricted.insert("user:preferences".to_string());
        
        denied.insert("network:all".to_string());
        denied.insert("fs:read".to_string());
        denied.insert("fs:write".to_string());
        denied.insert("system:settings".to_string());
        
        Self {
            auto_allowed,
            restricted,
            denied,
            plugin_overrides: HashMap::new(),
        }
    }
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        Self {
            plugin_permissions: RwLock::new(HashMap::new()),
            pending_requests: RwLock::new(HashMap::new()),
            settings: RwLock::new(PermissionSettings::default()),
        }
    }
    
    /// Initialize the permission manager
    pub async fn initialize(&self) -> Result<(), String> {
        // Load settings from disk
        match self.load_settings().await {
            Ok(settings) => {
                // Update settings
                *self.settings.write().await = settings;
                log::info!("Loaded permission settings");
            },
            Err(e) => {
                log::warn!("Failed to load permission settings, using defaults: {}", e);
            }
        }
        
        log::info!("Permission manager initialized");
        Ok(())
    }
    
    /// Load settings from disk
    async fn load_settings(&self) -> Result<PermissionSettings, String> {
        // Get settings file path
        let config_dir = self.get_config_dir()?;
        let settings_path = config_dir.join("permission_settings.json");
        
        // Load settings if file exists
        if settings_path.exists() {
            let content = tokio::fs::read_to_string(&settings_path)
                .await
                .map_err(|e| format!("Failed to read settings file: {}", e))?;
                
            // Parse JSON
            let settings: PermissionSettings = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;
                
            Ok(settings)
        } else {
            // Use defaults if file doesn't exist
            Ok(PermissionSettings::default())
        }
    }
    
    /// Save settings to disk
    async fn save_settings(&self) -> Result<(), String> {
        // Get settings
        let settings = self.settings.read().await;
        
        // Get settings file path
        let config_dir = self.get_config_dir()?;
        
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            tokio::fs::create_dir_all(&config_dir)
                .await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        let settings_path = config_dir.join("permission_settings.json");
        
        // Serialize and save
        let content = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
            
        tokio::fs::write(&settings_path, content)
            .await
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
            
        Ok(())
    }
    
    /// Get config directory
    fn get_config_dir(&self) -> Result<std::path::PathBuf, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "Could not determine home directory".to_string())?;
            
        #[cfg(target_os = "windows")]
        let config_dir = home_dir.join("AppData").join("Roaming").join("mcp").join("plugins");
        
        #[cfg(target_os = "macos")]
        let config_dir = home_dir.join("Library").join("Application Support").join("mcp").join("plugins");
        
        #[cfg(target_os = "linux")]
        let config_dir = home_dir.join(".config").join("mcp").join("plugins");
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let config_dir = home_dir.join(".mcp").join("plugins");
        
        Ok(config_dir)
    }
    
    /// Check if initial permissions for a plugin are allowed
    pub async fn check_initial_permissions(&self, permissions: &[String]) -> Result<bool, String> {
        let settings = self.settings.read().await;
        
        // Check if any permissions are denied
        for permission in permissions {
            if settings.denied.contains(permission) {
                log::warn!("Plugin requested denied permission: {}", permission);
                return Ok(false);
            }
        }
        
        // All permissions must be auto-allowed for initial installation without prompt
        for permission in permissions {
            if !settings.auto_allowed.contains(permission) {
                log::info!("Plugin requires permission that's not auto-allowed: {}", permission);
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Grant permissions to a plugin
    pub async fn grant_permissions(&self, plugin_id: &str, permissions: &[String]) -> Result<(), String> {
        // Get current plugin permissions
        let mut all_permissions = self.plugin_permissions.write().await;
        
        // Get or create permissions for this plugin
        let plugin_perms = all_permissions.entry(plugin_id.to_string())
            .or_insert_with(HashSet::new);
            
        // Add all permissions
        for permission in permissions {
            plugin_perms.insert(permission.clone());
        }
        
        // Check if there were pending requests for these permissions
        let mut pending = self.pending_requests.write().await;
        if let Some(pending_perms) = pending.get_mut(plugin_id) {
            // Remove granted permissions from pending
            for permission in permissions {
                pending_perms.remove(permission);
            }
            
            // Remove entry if empty
            if pending_perms.is_empty() {
                pending.remove(plugin_id);
            }
        }
        
        // Update plugin overrides in settings
        let mut settings = self.settings.write().await;
        let override_entry = settings.plugin_overrides
            .entry(plugin_id.to_string())
            .or_insert_with(PluginPermissionOverride::default);
            
        // Add granted permissions to allowed list
        for permission in permissions {
            override_entry.allowed.insert(permission.clone());
            // Remove from denied if present
            override_entry.denied.remove(permission);
        }
        
        // Save settings
        drop(settings);
        self.save_settings().await?;
        
        log::info!("Granted permissions to plugin {}: {:?}", plugin_id, permissions);
        Ok(())
    }
    
    /// Revoke permissions from a plugin
    pub async fn revoke_permissions(&self, plugin_id: &str, permissions: &[String]) -> Result<(), String> {
        // Get current plugin permissions
        let mut all_permissions = self.plugin_permissions.write().await;
        
        // Remove permissions if the plugin exists
        if let Some(plugin_perms) = all_permissions.get_mut(plugin_id) {
            for permission in permissions {
                plugin_perms.remove(permission);
            }
        }
        
        // Update plugin overrides in settings
        let mut settings = self.settings.write().await;
        if let Some(override_entry) = settings.plugin_overrides.get_mut(plugin_id) {
            // Remove permissions from allowed list
            for permission in permissions {
                override_entry.allowed.remove(permission);
                // Add to denied list
                override_entry.denied.insert(permission.clone());
            }
        } else {
            // Create new override entry
            let mut override_entry = PluginPermissionOverride::default();
            for permission in permissions {
                override_entry.denied.insert(permission.clone());
            }
            settings.plugin_overrides.insert(plugin_id.to_string(), override_entry);
        }
        
        // Save settings
        drop(settings);
        self.save_settings().await?;
        
        log::info!("Revoked permissions from plugin {}: {:?}", plugin_id, permissions);
        Ok(())
    }
    
    /// Check if a plugin has a specific permission
    pub async fn has_permission(&self, plugin_id: &str, permission: &str) -> bool {
        // First check if plugin has this permission explicitly
        let all_permissions = self.plugin_permissions.read().await;
        if let Some(plugin_perms) = all_permissions.get(plugin_id) {
            if plugin_perms.contains(permission) {
                return true;
            }
        }
        
        // Then check if it's an auto-allowed permission
        let settings = self.settings.read().await;
        if settings.auto_allowed.contains(permission) {
            return true;
        }
        
        // Check plugin overrides
        if let Some(override_entry) = settings.plugin_overrides.get(plugin_id) {
            // If explicitly allowed in override
            if override_entry.allowed.contains(permission) {
                return true;
            }
            
            // If explicitly denied in override
            if override_entry.denied.contains(permission) {
                return false;
            }
        }
        
        // If we get here, the plugin doesn't have the permission
        false
    }
    
    /// Request additional permissions for a plugin
    pub async fn request_permissions(&self, plugin_id: &str, permissions: &[String]) -> Result<bool, String> {
        log::info!("Plugin {} requesting permissions: {:?}", plugin_id, permissions);
        
        // Check if any permissions are denied at the system level
        let settings = self.settings.read().await;
        for permission in permissions {
            if settings.denied.contains(permission) {
                log::warn!("Plugin requested denied permission: {}", permission);
                return Ok(false);
            }
        }
        
        // All auto-allowed permissions are granted immediately
        let mut to_grant = Vec::new();
        let mut to_request = Vec::new();
        
        for permission in permissions {
            if settings.auto_allowed.contains(permission) {
                to_grant.push(permission.clone());
            } else {
                to_request.push(permission.clone());
            }
        }
        
        // Grant auto-allowed permissions
        if !to_grant.is_empty() {
            drop(settings); // Release lock before calling grant_permissions
            self.grant_permissions(plugin_id, &to_grant).await?;
        } else {
            drop(settings); // Release lock
        }
        
        // If there are permissions that need user approval
        if !to_request.is_empty() {
            // Add to pending requests
            let mut pending = self.pending_requests.write().await;
            let plugin_pending = pending.entry(plugin_id.to_string())
                .or_insert_with(HashSet::new);
                
            for permission in &to_request {
                plugin_pending.insert(permission.clone());
            }
            
            // TODO: Trigger UI to request user approval
            log::info!("Added pending permission request for plugin {}: {:?}", plugin_id, to_request);
            
            // For now, return false since we need user approval
            return Ok(false);
        }
        
        // All permissions were auto-granted
        Ok(true)
    }
    
    /// Get pending permission requests
    pub async fn get_pending_requests(&self) -> HashMap<String, Vec<String>> {
        let pending = self.pending_requests.read().await;
        
        // Convert to HashMap<String, Vec<String>> for easier serialization
        let mut result = HashMap::new();
        for (plugin_id, permissions) in pending.iter() {
            result.insert(plugin_id.clone(), permissions.iter().cloned().collect());
        }
        
        result
    }
    
    /// Respond to a permission request
    pub async fn respond_to_request(&self, plugin_id: &str, permissions: &[String], approved: bool) -> Result<(), String> {
        if approved {
            // Grant the permissions
            self.grant_permissions(plugin_id, permissions).await?;
        } else {
            // Remove from pending requests
            let mut pending = self.pending_requests.write().await;
            if let Some(plugin_pending) = pending.get_mut(plugin_id) {
                for permission in permissions {
                    plugin_pending.remove(permission);
                }
                
                // Remove entry if empty
                if plugin_pending.is_empty() {
                    pending.remove(plugin_id);
                }
            }
            
            // Add to denied permissions for this plugin
            self.revoke_permissions(plugin_id, permissions).await?;
        }
        
        Ok(())
    }
    
    /// Get all permissions for a plugin
    pub async fn get_plugin_permissions(&self, plugin_id: &str) -> HashSet<String> {
        let all_permissions = self.plugin_permissions.read().await;
        if let Some(plugin_perms) = all_permissions.get(plugin_id) {
            plugin_perms.clone()
        } else {
            HashSet::new()
        }
    }
    
    /// Update permission settings
    pub async fn update_settings(&self, settings: PermissionSettings) -> Result<(), String> {
        // Update settings
        *self.settings.write().await = settings;
        
        // Save to disk
        self.save_settings().await?;
        
        log::info!("Updated permission settings");
        Ok(())
    }
    
    /// Get current permission settings
    pub async fn get_settings(&self) -> PermissionSettings {
        self.settings.read().await.clone()
    }
    
    /// Remove a plugin's permissions
    pub async fn remove_plugin(&self, plugin_id: &str) -> Result<(), String> {
        // Remove from plugin permissions
        let mut all_permissions = self.plugin_permissions.write().await;
        all_permissions.remove(plugin_id);
        
        // Remove from pending requests
        let mut pending = self.pending_requests.write().await;
        pending.remove(plugin_id);
        
        // Remove from plugin overrides in settings
        let mut settings = self.settings.write().await;
        settings.plugin_overrides.remove(plugin_id);
        
        // Save settings
        drop(settings);
        self.save_settings().await?;
        
        log::info!("Removed permissions for plugin {}", plugin_id);
        Ok(())
    }
    
    /// Initialize permissions for a new plugin
    pub async fn initialize_plugin(&self, plugin_id: &str, permissions: &[String]) -> Result<bool, String> {
        log::info!("Initializing permissions for plugin {}: {:?}", plugin_id, permissions);
        
        // Check if all permissions are auto-allowed
        let settings = self.settings.read().await;
        let mut all_auto_allowed = true;
        
        // Check if any permissions are denied
        for permission in permissions {
            if settings.denied.contains(permission) {
                log::warn!("Plugin requested denied permission: {}", permission);
                return Ok(false);
            }
            
            if !settings.auto_allowed.contains(permission) {
                all_auto_allowed = false;
            }
        }
        
        // If all permissions are auto-allowed, grant them
        if all_auto_allowed {
            drop(settings); // Release lock before calling grant_permissions
            self.grant_permissions(plugin_id, permissions).await?;
            return Ok(true);
        }
        
        // Otherwise, we need user approval for some permissions
        drop(settings);
        
        // Add to pending requests
        let mut pending = self.pending_requests.write().await;
        let plugin_pending = pending.entry(plugin_id.to_string())
            .or_insert_with(HashSet::new);
            
        for permission in permissions {
            plugin_pending.insert(permission.clone());
        }
        
        // TODO: Trigger UI to request user approval
        log::info!("Added pending permission request for new plugin {}: {:?}", plugin_id, permissions);
        
        // For now, return false since we need user approval
        Ok(false)
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
