use serde::{Serialize, Deserialize};

use crate::plugins::types::{PluginInfo, PluginDetails, RepositoryPlugin};

/// UI state for plugin management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerUIState {
    /// Currently installed plugins
    pub installed_plugins: Vec<PluginInfo>,
    /// Available plugins for installation
    pub available_plugins: Vec<RepositoryPlugin>,
    /// Currently selected plugin
    pub selected_plugin: Option<String>,
    /// Current tab
    pub current_tab: PluginManagerTab,
    /// Search query
    pub search_query: String,
    /// Loading state
    pub loading: bool,
    /// Error message
    pub error: Option<String>,
}

/// Plugin manager tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginManagerTab {
    /// Installed plugins
    Installed,
    /// Available plugins
    Available,
    /// Repositories
    Repositories,
    /// Settings
    Settings,
}

/// Plugin details view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDetailsViewState {
    /// Plugin details
    pub plugin: PluginDetails,
    /// Current tab
    pub current_tab: PluginDetailsTab,
    /// Loading state
    pub loading: bool,
    /// Error message
    pub error: Option<String>,
}

/// Plugin details tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginDetailsTab {
    /// Overview
    Overview,
    /// Settings
    Settings,
    /// Permissions
    Permissions,
    /// Logs
    Logs,
}

/// Plugin installation view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallViewState {
    /// Plugin to install
    pub plugin: RepositoryPlugin,
    /// Installation progress
    pub progress: f32,
    /// Installation step
    pub step: PluginInstallStep,
    /// Error message
    pub error: Option<String>,
}

/// Plugin installation steps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginInstallStep {
    /// Downloading
    Downloading,
    /// Verifying
    Verifying,
    /// Installing
    Installing,
    /// Configuring
    Configuring,
    /// Complete
    Complete,
    /// Failed
    Failed,
}

/// Permission request view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequestViewState {
    /// Plugin name
    pub plugin_name: String,
    /// Plugin ID
    pub plugin_id: String,
    /// Requested permissions
    pub permissions: Vec<String>,
    /// Permission descriptions
    pub permission_descriptions: Vec<String>,
    /// Reason for request
    pub reason: String,
}

/// Plugin management commands for Tauri
#[tauri::command]
pub async fn get_installed_plugins() -> Result<Vec<PluginInfo>, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get installed plugins
    Ok(plugin_manager.get_installed_plugins().await)
}

#[tauri::command]
pub async fn get_available_plugins(query: &str) -> Result<Vec<RepositoryPlugin>, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Search for plugins
    let plugins = plugin_manager.search_plugins(query).await?;
    
    // Convert to repository plugins
    let mut available = Vec::new();
    
    // For each plugin, check if it's already installed
    for plugin in plugins {
        // Check if plugin is already installed
        let installed = plugin_manager.get_installed_plugins().await
            .iter()
            .any(|p| p.id == plugin.id);
            
        if !installed {
            // Get plugin details from discovery
            if let Some(repo_plugin) = plugin_manager.discovery.get_plugin(&plugin.id).await {
                available.push(repo_plugin);
            }
        }
    }
    
    Ok(available)
}

#[tauri::command]
pub async fn install_plugin(plugin_id: &str) -> Result<PluginInfo, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Get plugin details
    let plugin = match plugin_manager.discovery.get_plugin(plugin_id).await {
        Some(plugin) => plugin,
        None => return Err(format!("Plugin not found: {}", plugin_id)),
    };
    
    // Download plugin
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temporary directory: {}", e))?;
        
    let download_path = temp_dir.path().join("plugin.zip");
    
    // Download the plugin
    let response = reqwest::get(&plugin.download_url)
        .await
        .map_err(|e| format!("Failed to download plugin: {}", e))?;
        
    if !response.status().is_success() {
        return Err(format!("Failed to download plugin: {}", response.status()));
    }
    
    // Save to file
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
        
    tokio::fs::write(&download_path, &bytes)
        .await
        .map_err(|e| format!("Failed to write plugin file: {}", e))?;
        
    // Install the plugin
    let plugin_info = plugin_manager.install_plugin(&download_path).await?;
    
    // Activate the plugin
    plugin_manager.activate_plugin(&plugin_id).await?;
    
    Ok(plugin_info)
}

#[tauri::command]
pub async fn uninstall_plugin(plugin_id: &str) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Uninstall the plugin
    plugin_manager.uninstall_plugin(plugin_id).await
}

#[tauri::command]
pub async fn activate_plugin(plugin_id: &str) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Activate the plugin
    plugin_manager.activate_plugin(plugin_id).await
}

#[tauri::command]
pub async fn deactivate_plugin(plugin_id: &str) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Deactivate the plugin
    plugin_manager.deactivate_plugin(plugin_id).await
}

#[tauri::command]
pub async fn get_plugin_details(plugin_id: &str) -> Result<PluginDetails, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get plugin details
    plugin_manager.get_plugin_details(plugin_id).await
}

#[tauri::command]
pub async fn update_plugin_settings(plugin_id: &str, settings: serde_json::Value) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Update plugin settings
    plugin_manager.update_plugin_settings(plugin_id, settings).await
}

#[tauri::command]
pub async fn install_local_plugin(path: &str) -> Result<PluginInfo, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Install the plugin
    let plugin_info = plugin_manager.install_plugin(&std::path::Path::new(path)).await?;
    
    // Activate the plugin
    plugin_manager.activate_plugin(&plugin_info.id).await?;
    
    Ok(plugin_info)
}

#[tauri::command]
pub async fn update_plugin(plugin_id: &str, path: &str) -> Result<PluginInfo, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let mut plugin_manager = plugin_manager.write().await;
    
    // Update the plugin
    plugin_manager.update_plugin(plugin_id, &std::path::Path::new(path)).await
}

#[tauri::command]
pub async fn get_pending_permission_requests() -> Result<HashMap<String, Vec<String>>, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get permission manager
    let permission_manager = plugin_manager.permission_manager();
    
    // Get pending requests
    Ok(permission_manager.get_pending_requests().await)
}

#[tauri::command]
pub async fn respond_to_permission_request(plugin_id: &str, permissions: Vec<String>, approved: bool) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get permission manager
    let permission_manager = plugin_manager.permission_manager();
    
    // Respond to request
    permission_manager.respond_to_request(plugin_id, &permissions, approved).await
}

#[tauri::command]
pub async fn get_repositories() -> Result<Vec<crate::plugins::discovery::PluginRepository>, String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get discovery
    let discovery = plugin_manager.discovery();
    
    // Get repositories
    Ok(discovery.get_repositories().await)
}

#[tauri::command]
pub async fn add_repository(repo: crate::plugins::discovery::PluginRepository) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get discovery
    let discovery = plugin_manager.discovery();
    
    // Add repository
    discovery.add_repository(repo).await
}

#[tauri::command]
pub async fn remove_repository(name: &str) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get discovery
    let discovery = plugin_manager.discovery();
    
    // Remove repository
    discovery.remove_repository(name).await
}

#[tauri::command]
pub async fn set_repository_enabled(name: &str, enabled: bool) -> Result<(), String> {
    // Get plugin manager
    let plugin_manager = crate::plugins::get_plugin_manager();
    let plugin_manager = plugin_manager.read().await;
    
    // Get discovery
    let discovery = plugin_manager.discovery();
    
    // Set repository enabled
    discovery.set_repository_enabled(name, enabled).await
}

/// Implement accessor methods for the plugin manager
impl crate::plugins::PluginManager {
    /// Get permission manager
    pub fn permission_manager(&self) -> &crate::plugins::permissions::PermissionManager {
        &self.permission_manager
    }
    
    /// Get discovery
    pub fn discovery(&self) -> &crate::plugins::discovery::PluginDiscovery {
        &self.discovery
    }
}

/// Register plugin management commands
pub fn register_commands(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.register_async_command("get_installed_plugins", get_installed_plugins);
    app.register_async_command("get_available_plugins", get_available_plugins);
    app.register_async_command("install_plugin", install_plugin);
    app.register_async_command("uninstall_plugin", uninstall_plugin);
    app.register_async_command("activate_plugin", activate_plugin);
    app.register_async_command("deactivate_plugin", deactivate_plugin);
    app.register_async_command("get_plugin_details", get_plugin_details);
    app.register_async_command("update_plugin_settings", update_plugin_settings);
    app.register_async_command("install_local_plugin", install_local_plugin);
    app.register_async_command("update_plugin", update_plugin);
    app.register_async_command("get_pending_permission_requests", get_pending_permission_requests);
    app.register_async_command("respond_to_permission_request", respond_to_permission_request);
    app.register_async_command("get_repositories", get_repositories);
    app.register_async_command("add_repository", add_repository);
    app.register_async_command("remove_repository", remove_repository);
    app.register_async_command("set_repository_enabled", set_repository_enabled);
    
    Ok(())
}

// Add missing imports
use std::collections::HashMap;
use crate::plugins::permissions::PermissionManager;
use crate::plugins::discovery::PluginDiscovery;
