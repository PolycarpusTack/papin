use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::plugins::types::{Plugin, PluginInfo, PluginDetails};

/// Plugin registry
pub struct PluginRegistry {
    /// Installed plugins
    plugins: RwLock<HashMap<String, Plugin>>,
    /// Base directory for plugins
    plugins_dir: RwLock<PathBuf>,
}

/// Plugin registry data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryData {
    /// Plugin metadata
    plugins: HashMap<String, PluginMetadata>,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginMetadata {
    /// Plugin ID
    id: String,
    /// Active status
    active: bool,
    /// Installed timestamp (ISO 8601)
    installed_at: String,
    /// Last updated timestamp (ISO 8601)
    updated_at: String,
    /// Plugin settings
    settings: serde_json::Value,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            plugins_dir: RwLock::new(PathBuf::new()),
        }
    }
    
    /// Initialize the plugin registry
    pub async fn initialize(&self) -> Result<(), String> {
        // Set up plugins directory
        self.setup_plugins_dir().await?;
        
        // Load registry data
        self.load_registry_data().await?;
        
        log::info!("Plugin registry initialized");
        Ok(())
    }
    
    /// Set up plugins directory
    async fn setup_plugins_dir(&self) -> Result<(), String> {
        // Get plugins directory
        let plugins_dir = get_plugins_dir()?;
        
        // Create directory if it doesn't exist
        if !plugins_dir.exists() {
            tokio::fs::create_dir_all(&plugins_dir)
                .await
                .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        }
        
        // Store plugins directory
        *self.plugins_dir.write().await = plugins_dir.clone();
        
        log::info!("Using plugins directory: {}", plugins_dir.display());
        Ok(())
    }
    
    /// Load registry data from disk
    async fn load_registry_data(&self) -> Result<(), String> {
        // Get registry file path
        let plugins_dir = self.plugins_dir.read().await;
        let registry_path = plugins_dir.join("registry.json");
        
        // If registry file doesn't exist, create empty registry
        if !registry_path.exists() {
            log::info!("Registry file doesn't exist, creating empty registry");
            return Ok(());
        }
        
        // Read registry file
        let content = tokio::fs::read_to_string(&registry_path)
            .await
            .map_err(|e| format!("Failed to read registry file: {}", e))?;
            
        // Parse JSON
        let registry_data: RegistryData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse registry JSON: {}", e))?;
            
        log::info!("Loaded registry data with {} plugins", registry_data.plugins.len());
        
        // We don't load plugins here, just initialize the registry
        // Plugins will be loaded by the plugin loader
        
        Ok(())
    }
    
    /// Save registry data to disk
    async fn save_registry_data(&self) -> Result<(), String> {
        // Get registry data
        let mut registry_data = RegistryData {
            plugins: HashMap::new(),
        };
        
        // Get plugins
        let plugins = self.plugins.read().await;
        
        // Convert to registry data
        for (id, plugin) in plugins.iter() {
            let metadata = PluginMetadata {
                id: id.clone(),
                active: plugin.active,
                installed_at: plugin.installed_at.to_rfc3339(),
                updated_at: plugin.updated_at.to_rfc3339(),
                settings: plugin.settings.clone(),
            };
            
            registry_data.plugins.insert(id.clone(), metadata);
        }
        
        // Get registry file path
        let plugins_dir = self.plugins_dir.read().await;
        let registry_path = plugins_dir.join("registry.json");
        
        // Serialize and save
        let content = serde_json::to_string_pretty(&registry_data)
            .map_err(|e| format!("Failed to serialize registry data: {}", e))?;
            
        tokio::fs::write(&registry_path, content)
            .await
            .map_err(|e| format!("Failed to write registry file: {}", e))?;
            
        log::info!("Saved registry data with {} plugins", plugins.len());
        Ok(())
    }
    
    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Plugin) -> Result<PluginInfo, String> {
        let plugin_id = plugin.manifest.name.clone();
        log::info!("Registering plugin: {}", plugin_id);
        
        // Create plugin info
        let plugin_info = PluginInfo {
            id: plugin_id.clone(),
            display_name: plugin.manifest.display_name.clone(),
            version: plugin.manifest.version.clone(),
            description: plugin.manifest.description.clone(),
            author: plugin.manifest.author.clone(),
            active: plugin.active,
            installed_at: plugin.installed_at.to_rfc3339(),
            updated_at: plugin.updated_at.to_rfc3339(),
        };
        
        // Add to plugins
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id, plugin);
        
        // Save registry data
        drop(plugins);
        self.save_registry_data().await?;
        
        Ok(plugin_info)
    }
    
    /// Update a plugin
    pub async fn update_plugin(&self, plugin: Plugin) -> Result<PluginInfo, String> {
        let plugin_id = plugin.manifest.name.clone();
        log::info!("Updating plugin: {}", plugin_id);
        
        // Check if plugin exists
        let mut plugins = self.plugins.write().await;
        if !plugins.contains_key(&plugin_id) {
            return Err(format!("Plugin not found: {}", plugin_id));
        }
        
        // Create plugin info
        let plugin_info = PluginInfo {
            id: plugin_id.clone(),
            display_name: plugin.manifest.display_name.clone(),
            version: plugin.manifest.version.clone(),
            description: plugin.manifest.description.clone(),
            author: plugin.manifest.author.clone(),
            active: plugin.active,
            installed_at: plugin.installed_at.to_rfc3339(),
            updated_at: plugin.updated_at.to_rfc3339(),
        };
        
        // Update plugin
        plugins.insert(plugin_id, plugin);
        
        // Save registry data
        drop(plugins);
        self.save_registry_data().await?;
        
        Ok(plugin_info)
    }
    
    /// Get a plugin
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<Plugin, String> {
        let plugins = self.plugins.read().await;
        
        // Get plugin
        if let Some(plugin) = plugins.get(plugin_id) {
            Ok(plugin.clone())
        } else {
            Err(format!("Plugin not found: {}", plugin_id))
        }
    }
    
    /// Get all plugins
    pub async fn get_all_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().await;
        
        // Convert to plugin info
        plugins.values().map(|plugin| {
            PluginInfo {
                id: plugin.manifest.name.clone(),
                display_name: plugin.manifest.display_name.clone(),
                version: plugin.manifest.version.clone(),
                description: plugin.manifest.description.clone(),
                author: plugin.manifest.author.clone(),
                active: plugin.active,
                installed_at: plugin.installed_at.to_rfc3339(),
                updated_at: plugin.updated_at.to_rfc3339(),
            }
        }).collect()
    }
    
    /// Get plugin count
    pub async fn get_plugin_count(&self) -> usize {
        self.plugins.read().await.len()
    }
    
    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<(), String> {
        log::info!("Uninstalling plugin: {}", plugin_id);
        
        // Check if plugin exists
        let mut plugins = self.plugins.write().await;
        if !plugins.contains_key(plugin_id) {
            return Err(format!("Plugin not found: {}", plugin_id));
        }
        
        // Get plugin directory
        let plugin_dir = self.get_plugin_directory_internal(plugin_id).await?;
        
        // Remove plugin directory
        tokio::fs::remove_dir_all(&plugin_dir)
            .await
            .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
            
        // Remove from registry
        plugins.remove(plugin_id);
        
        // Save registry data
        drop(plugins);
        self.save_registry_data().await?;
        
        Ok(())
    }
    
    /// Set plugin active state
    pub async fn set_plugin_active(&self, plugin_id: &str, active: bool) -> Result<(), String> {
        log::info!("Setting plugin {} active state to {}", plugin_id, active);
        
        // Check if plugin exists
        let mut plugins = self.plugins.write().await;
        let plugin = plugins.get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
            
        // Update active state
        plugin.active = active;
        
        // Save registry data
        drop(plugins);
        self.save_registry_data().await?;
        
        Ok(())
    }
    
    /// Get plugin directory
    pub async fn get_plugin_directory(&self, plugin_id: &str) -> Result<PathBuf, String> {
        self.get_plugin_directory_internal(plugin_id).await
    }
    
    /// Internal method to get plugin directory
    async fn get_plugin_directory_internal(&self, plugin_id: &str) -> Result<PathBuf, String> {
        // Get plugins directory
        let plugins_dir = self.plugins_dir.read().await;
        
        // Get plugin directory
        let plugin_dir = plugins_dir.join(plugin_id);
        
        // Check if directory exists
        if !plugin_dir.exists() {
            return Err(format!("Plugin directory not found: {}", plugin_dir.display()));
        }
        
        Ok(plugin_dir)
    }
    
    /// Get all plugin directories
    pub async fn get_plugin_directories(&self) -> Result<Vec<PathBuf>, String> {
        // Get plugins directory
        let plugins_dir = self.plugins_dir.read().await;
        
        // Read directory entries
        let mut entries = tokio::fs::read_dir(&*plugins_dir)
            .await
            .map_err(|e| format!("Failed to read plugins directory: {}", e))?;
            
        let mut result = Vec::new();
        
        // Collect all directories
        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))? {
                
            let path = entry.path();
            if path.is_dir() {
                result.push(path);
            }
        }
        
        Ok(result)
    }
    
    /// Prepare a directory for a plugin
    pub async fn prepare_plugin_directory(&self, plugin_id: &str) -> Result<PathBuf, String> {
        // Get plugins directory
        let plugins_dir = self.plugins_dir.read().await;
        
        // Get plugin directory
        let plugin_dir = plugins_dir.join(plugin_id);
        
        // Create directory if it doesn't exist
        if plugin_dir.exists() {
            // Remove existing directory
            tokio::fs::remove_dir_all(&plugin_dir)
                .await
                .map_err(|e| format!("Failed to remove existing plugin directory: {}", e))?;
        }
        
        // Create directory
        tokio::fs::create_dir_all(&plugin_dir)
            .await
            .map_err(|e| format!("Failed to create plugin directory: {}", e))?;
            
        Ok(plugin_dir)
    }
    
    /// Get plugin details
    pub async fn get_plugin_details(&self, plugin_id: &str) -> Result<PluginDetails, String> {
        // Get plugin
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
            
        // Create plugin details
        let details = PluginDetails {
            info: PluginInfo {
                id: plugin.manifest.name.clone(),
                display_name: plugin.manifest.display_name.clone(),
                version: plugin.manifest.version.clone(),
                description: plugin.manifest.description.clone(),
                author: plugin.manifest.author.clone(),
                active: plugin.active,
                installed_at: plugin.installed_at.to_rfc3339(),
                updated_at: plugin.updated_at.to_rfc3339(),
            },
            license: plugin.manifest.license.clone(),
            permissions: plugin.manifest.permissions.clone(),
            hooks: plugin.manifest.hooks.clone(),
            settings: plugin.manifest.config.settings.clone(),
            current_settings: plugin.settings.clone(),
        };
        
        Ok(details)
    }
    
    /// Update plugin settings
    pub async fn update_plugin_settings(&self, plugin_id: &str, settings: serde_json::Value) -> Result<(), String> {
        // Check if plugin exists
        let mut plugins = self.plugins.write().await;
        let plugin = plugins.get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
            
        // Update settings
        plugin.settings = settings;
        
        // Save registry data
        drop(plugins);
        self.save_registry_data().await?;
        
        Ok(())
    }
    
    /// Get plugin settings
    pub async fn get_plugin_settings(&self, plugin_id: &str) -> Result<serde_json::Value, String> {
        // Get plugin
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
            
        Ok(plugin.settings.clone())
    }
}

/// Get the plugins directory
fn get_plugins_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
        
    #[cfg(target_os = "windows")]
    let plugins_dir = home_dir.join("AppData").join("Roaming").join("mcp").join("plugins").join("installed");
    
    #[cfg(target_os = "macos")]
    let plugins_dir = home_dir.join("Library").join("Application Support").join("mcp").join("plugins").join("installed");
    
    #[cfg(target_os = "linux")]
    let plugins_dir = home_dir.join(".config").join("mcp").join("plugins").join("installed");
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let plugins_dir = home_dir.join(".mcp").join("plugins").join("installed");
    
    Ok(plugins_dir)
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
