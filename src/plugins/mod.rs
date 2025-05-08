pub mod registry;
pub mod loader;
pub mod permissions;
pub mod sandbox;
pub mod discovery;
pub mod ui;
pub mod types;
pub mod hooks;

use std::sync::Arc;
use once_cell::sync::OnceCell;
use tokio::sync::RwLock;

use registry::PluginRegistry;
use loader::PluginLoader;
use permissions::PermissionManager;
use sandbox::SandboxManager;
use discovery::PluginDiscovery;

/// Global plugin manager instance
static PLUGIN_MANAGER: OnceCell<Arc<RwLock<PluginManager>>> = OnceCell::new();

/// Main plugin management system
pub struct PluginManager {
    /// Registry of all installed plugins
    registry: PluginRegistry,
    /// Loader for loading plugins
    loader: PluginLoader,
    /// Permission manager
    permission_manager: PermissionManager,
    /// Sandbox manager
    sandbox_manager: SandboxManager,
    /// Plugin discovery
    discovery: PluginDiscovery,
    /// Enabled state
    enabled: bool,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
            loader: PluginLoader::new(),
            permission_manager: PermissionManager::new(),
            sandbox_manager: SandboxManager::new(),
            discovery: PluginDiscovery::new(),
            enabled: true,
        }
    }
    
    /// Initialize the plugin manager
    pub async fn initialize(&mut self) -> Result<(), String> {
        // Skip initialization if plugins are disabled
        if !crate::feature_flags::FeatureManager::default().is_enabled(crate::feature_flags::FeatureFlags::PLUGINS) {
            log::info!("Plugins are disabled in feature flags, skipping initialization");
            self.enabled = false;
            return Ok(());
        }
        
        log::info!("Initializing plugin manager");
        
        // Initialize components
        self.permission_manager.initialize().await?;
        self.sandbox_manager.initialize().await?;
        self.loader.initialize(&self.sandbox_manager, &self.permission_manager).await?;
        self.registry.initialize().await?;
        self.discovery.initialize().await?;
        
        // Load installed plugins
        self.load_installed_plugins().await?;
        
        log::info!("Plugin manager initialized successfully");
        Ok(())
    }
    
    /// Load all installed plugins
    async fn load_installed_plugins(&mut self) -> Result<(), String> {
        log::info!("Loading installed plugins");
        
        // Get plugin directories
        let plugin_dirs = self.registry.get_plugin_directories().await?;
        
        // Load each plugin
        for dir in plugin_dirs {
            match self.loader.load_plugin(&dir).await {
                Ok(plugin) => {
                    log::info!("Loaded plugin: {}", plugin.manifest.name);
                    self.registry.register_plugin(plugin).await?;
                }
                Err(e) => {
                    log::error!("Failed to load plugin from directory {}: {}", dir.display(), e);
                }
            }
        }
        
        log::info!("Loaded {} plugins", self.registry.get_plugin_count().await);
        Ok(())
    }
    
    /// Get a list of all installed plugins
    pub async fn get_installed_plugins(&self) -> Vec<types::PluginInfo> {
        self.registry.get_all_plugins().await
    }
    
    /// Install a plugin from a given path
    pub async fn install_plugin(&mut self, path: &std::path::Path) -> Result<types::PluginInfo, String> {
        log::info!("Installing plugin from: {}", path.display());
        
        // Verify the plugin
        let manifest = self.loader.verify_plugin(path).await?;
        
        // Check permissions
        let permissions = manifest.permissions.clone();
        if !self.permission_manager.check_initial_permissions(&permissions).await? {
            return Err(format!("Plugin requires permissions that are not allowed for initial installation"));
        }
        
        // Install the plugin
        let install_dir = self.registry.prepare_plugin_directory(&manifest.name).await?;
        self.loader.install_plugin(path, &install_dir).await?;
        
        // Load the plugin
        let plugin = self.loader.load_plugin(&install_dir).await?;
        
        // Register the plugin
        let plugin_info = self.registry.register_plugin(plugin).await?;
        
        log::info!("Plugin installed successfully: {}", manifest.name);
        Ok(plugin_info)
    }
    
    /// Uninstall a plugin by ID
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        log::info!("Uninstalling plugin: {}", plugin_id);
        
        // Deactivate the plugin first
        self.deactivate_plugin(plugin_id).await?;
        
        // Uninstall from registry
        self.registry.uninstall_plugin(plugin_id).await?;
        
        log::info!("Plugin uninstalled successfully: {}", plugin_id);
        Ok(())
    }
    
    /// Activate a plugin by ID
    pub async fn activate_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        log::info!("Activating plugin: {}", plugin_id);
        
        // Get the plugin
        let plugin = self.registry.get_plugin(plugin_id).await?;
        
        // Activate the plugin
        self.loader.activate_plugin(&plugin).await?;
        
        // Update the registry
        self.registry.set_plugin_active(plugin_id, true).await?;
        
        log::info!("Plugin activated successfully: {}", plugin_id);
        Ok(())
    }
    
    /// Deactivate a plugin by ID
    pub async fn deactivate_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        log::info!("Deactivating plugin: {}", plugin_id);
        
        // Get the plugin
        let plugin = self.registry.get_plugin(plugin_id).await?;
        
        // Deactivate the plugin
        self.loader.deactivate_plugin(&plugin).await?;
        
        // Update the registry
        self.registry.set_plugin_active(plugin_id, false).await?;
        
        log::info!("Plugin deactivated successfully: {}", plugin_id);
        Ok(())
    }
    
    /// Update a plugin by ID
    pub async fn update_plugin(&mut self, plugin_id: &str, path: &std::path::Path) -> Result<types::PluginInfo, String> {
        log::info!("Updating plugin: {} from path: {}", plugin_id, path.display());
        
        // Deactivate the plugin first
        self.deactivate_plugin(plugin_id).await?;
        
        // Verify the plugin
        let manifest = self.loader.verify_plugin(path).await?;
        
        // Make sure the ID matches
        if manifest.name != plugin_id {
            return Err(format!("Plugin ID mismatch: expected {}, got {}", plugin_id, manifest.name));
        }
        
        // Get the plugin directory
        let plugin_dir = self.registry.get_plugin_directory(plugin_id).await?;
        
        // Update the plugin
        self.loader.update_plugin(path, &plugin_dir).await?;
        
        // Reload the plugin
        let plugin = self.loader.load_plugin(&plugin_dir).await?;
        
        // Update the registry
        let plugin_info = self.registry.update_plugin(plugin).await?;
        
        // Reactivate the plugin if it was active
        if plugin_info.active {
            self.activate_plugin(plugin_id).await?;
        }
        
        log::info!("Plugin updated successfully: {}", plugin_id);
        Ok(plugin_info)
    }
    
    /// Search for available plugins
    pub async fn search_plugins(&self, query: &str) -> Result<Vec<types::PluginInfo>, String> {
        self.discovery.search_plugins(query).await
    }
    
    /// Get details about a plugin
    pub async fn get_plugin_details(&self, plugin_id: &str) -> Result<types::PluginDetails, String> {
        self.registry.get_plugin_details(plugin_id).await
    }
    
    /// Update plugin settings
    pub async fn update_plugin_settings(&mut self, plugin_id: &str, settings: serde_json::Value) -> Result<(), String> {
        self.registry.update_plugin_settings(plugin_id, settings).await
    }
    
    /// Get plugin settings
    pub async fn get_plugin_settings(&self, plugin_id: &str) -> Result<serde_json::Value, String> {
        self.registry.get_plugin_settings(plugin_id).await
    }
    
    /// Request additional permissions for a plugin
    pub async fn request_permissions(&mut self, plugin_id: &str, permissions: &[String]) -> Result<bool, String> {
        self.permission_manager.request_permissions(plugin_id, permissions).await
    }
    
    /// Check if plugin system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the global plugin manager
pub async fn init_plugin_manager() -> Arc<RwLock<PluginManager>> {
    let manager = Arc::new(RwLock::new(PluginManager::new()));
    
    if let Err(_) = PLUGIN_MANAGER.set(manager.clone()) {
        // Already initialized
    }
    
    // Initialize the manager
    manager.write().await.initialize().await.unwrap_or_else(|e| {
        log::error!("Failed to initialize plugin manager: {}", e);
    });
    
    manager
}

/// Get the global plugin manager
pub fn get_plugin_manager() -> Arc<RwLock<PluginManager>> {
    PLUGIN_MANAGER.get_or_init(|| {
        // Create a new manager - this should only happen if init_plugin_manager wasn't called
        log::warn!("Plugin manager not properly initialized, creating a new instance");
        Arc::new(RwLock::new(PluginManager::new()))
    }).clone()
}
