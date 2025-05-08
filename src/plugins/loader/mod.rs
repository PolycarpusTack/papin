use std::path::{Path, PathBuf};
use std::io::Read;
use zip::ZipArchive;
use chrono::Utc;
use uuid::Uuid;

use crate::plugins::types::{Plugin, PluginManifest};
use crate::plugins::sandbox::SandboxManager;
use crate::plugins::permissions::PermissionManager;

/// Plugin loader
pub struct PluginLoader {
    /// Sandbox manager reference
    sandbox_manager: Option<SandboxManager>,
    /// Permission manager reference
    permission_manager: Option<PermissionManager>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            sandbox_manager: None,
            permission_manager: None,
        }
    }
    
    /// Initialize the plugin loader
    pub async fn initialize(&mut self, sandbox_manager: &SandboxManager, 
                           permission_manager: &PermissionManager) -> Result<(), String> {
        log::info!("Initializing plugin loader");
        
        // Store references
        self.sandbox_manager = Some(sandbox_manager.clone());
        self.permission_manager = Some(permission_manager.clone());
        
        log::info!("Plugin loader initialized");
        Ok(())
    }
    
    /// Load a plugin from a directory
    pub async fn load_plugin(&self, dir: &Path) -> Result<Plugin, String> {
        log::info!("Loading plugin from directory: {}", dir.display());
        
        // Check if manifest file exists
        let manifest_path = dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(format!("Manifest file not found: {}", manifest_path.display()));
        }
        
        // Read manifest file
        let manifest_content = tokio::fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| format!("Failed to read manifest file: {}", e))?;
            
        // Parse manifest
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;
            
        // Check if main WASM file exists
        let wasm_path = dir.join(&manifest.main);
        if !wasm_path.exists() {
            return Err(format!("Main WASM file not found: {}", wasm_path.display()));
        }
        
        // Create plugin instance
        let plugin = Plugin {
            manifest,
            path: dir.to_path_buf(),
            active: false,
            installed_at: Utc::now(),
            updated_at: Utc::now(),
            settings: serde_json::Value::Object(serde_json::Map::new()),
            instance_id: Uuid::new_v4().to_string(),
        };
        
        log::info!("Loaded plugin: {}", plugin.manifest.name);
        Ok(plugin)
    }
    
    /// Verify a plugin package
    pub async fn verify_plugin(&self, path: &Path) -> Result<PluginManifest, String> {
        log::info!("Verifying plugin package: {}", path.display());
        
        // Check if file exists
        if !path.exists() {
            return Err(format!("Plugin package not found: {}", path.display()));
        }
        
        // Check file extension
        if let Some(ext) = path.extension() {
            if ext != "zip" {
                return Err(format!("Unsupported plugin package format: {}", ext.to_string_lossy()));
            }
        } else {
            return Err("Plugin package has no extension".to_string());
        }
        
        // Open ZIP file
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open plugin package: {}", e))?;
            
        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
            
        // Look for manifest.json
        let mut manifest_file = archive.by_name("manifest.json")
            .map_err(|e| format!("Manifest file not found in plugin package: {}", e))?;
            
        // Read manifest
        let mut manifest_content = String::new();
        manifest_file.read_to_string(&mut manifest_content)
            .map_err(|e| format!("Failed to read manifest file: {}", e))?;
            
        // Parse manifest
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;
            
        // Check if main WASM file exists in package
        if !archive.by_name(&manifest.main).is_ok() {
            return Err(format!("Main WASM file not found in plugin package: {}", manifest.main));
        }
        
        // TODO: Verify signatures and checksums
        
        log::info!("Plugin package verified: {}", manifest.name);
        Ok(manifest)
    }
    
    /// Install a plugin from a package
    pub async fn install_plugin(&self, package_path: &Path, install_dir: &Path) -> Result<(), String> {
        log::info!("Installing plugin from {} to {}", package_path.display(), install_dir.display());
        
        // Check if file exists
        if !package_path.exists() {
            return Err(format!("Plugin package not found: {}", package_path.display()));
        }
        
        // Open ZIP file
        let file = std::fs::File::open(package_path)
            .map_err(|e| format!("Failed to open plugin package: {}", e))?;
            
        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
            
        // Create installation directory
        if !install_dir.exists() {
            tokio::fs::create_dir_all(install_dir)
                .await
                .map_err(|e| format!("Failed to create installation directory: {}", e))?;
        }
        
        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;
                
            let file_path = file.name();
            let output_path = install_dir.join(file_path);
            
            // Create parent directories
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
            }
            
            // Extract file
            if file.is_file() {
                let mut output_file = std::fs::File::create(&output_path)
                    .map_err(|e| format!("Failed to create file: {}", e))?;
                    
                std::io::copy(&mut file, &mut output_file)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
        }
        
        log::info!("Plugin installed successfully to {}", install_dir.display());
        Ok(())
    }
    
    /// Update a plugin
    pub async fn update_plugin(&self, package_path: &Path, install_dir: &Path) -> Result<(), String> {
        log::info!("Updating plugin from {} to {}", package_path.display(), install_dir.display());
        
        // Remove existing files except settings
        let settings_path = install_dir.join("settings.json");
        let has_settings = settings_path.exists();
        
        // Save settings if they exist
        let settings = if has_settings {
            Some(tokio::fs::read_to_string(&settings_path)
                .await
                .map_err(|e| format!("Failed to read settings file: {}", e))?)
        } else {
            None
        };
        
        // Remove all files
        for entry in std::fs::read_dir(install_dir)
            .map_err(|e| format!("Failed to read installation directory: {}", e))? {
                
            let entry = entry
                .map_err(|e| format!("Failed to read directory entry: {}", e))?;
                
            let path = entry.path();
            if path != settings_path {
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                        .map_err(|e| format!("Failed to remove directory: {}", e))?;
                } else {
                    std::fs::remove_file(&path)
                        .map_err(|e| format!("Failed to remove file: {}", e))?;
                }
            }
        }
        
        // Install new version
        self.install_plugin(package_path, install_dir).await?;
        
        // Restore settings
        if let Some(settings_content) = settings {
            tokio::fs::write(&settings_path, settings_content)
                .await
                .map_err(|e| format!("Failed to restore settings file: {}", e))?;
        }
        
        log::info!("Plugin updated successfully");
        Ok(())
    }
    
    /// Activate a plugin
    pub async fn activate_plugin(&self, plugin: &Plugin) -> Result<(), String> {
        log::info!("Activating plugin: {}", plugin.manifest.name);
        
        // Check if sandbox manager is initialized
        let sandbox_manager = self.sandbox_manager.as_ref()
            .ok_or_else(|| "Sandbox manager not initialized".to_string())?;
            
        // Check if permission manager is initialized
        let permission_manager = self.permission_manager.as_ref()
            .ok_or_else(|| "Permission manager not initialized".to_string())?;
            
        // Load plugin into sandbox
        let instance_id = sandbox_manager.load_plugin(plugin, permission_manager).await?;
        
        log::info!("Plugin activated: {} (instance {})", plugin.manifest.name, instance_id);
        Ok(())
    }
    
    /// Deactivate a plugin
    pub async fn deactivate_plugin(&self, plugin: &Plugin) -> Result<(), String> {
        log::info!("Deactivating plugin: {}", plugin.manifest.name);
        
        // Check if sandbox manager is initialized
        let sandbox_manager = self.sandbox_manager.as_ref()
            .ok_or_else(|| "Sandbox manager not initialized".to_string())?;
            
        // Unload plugin from sandbox
        if sandbox_manager.instance_exists(&plugin.instance_id).await {
            sandbox_manager.unload_plugin(&plugin.instance_id).await?;
        }
        
        log::info!("Plugin deactivated: {}", plugin.manifest.name);
        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Needed to implement clone for the loader
impl Clone for PluginLoader {
    fn clone(&self) -> Self {
        Self {
            sandbox_manager: self.sandbox_manager.clone(),
            permission_manager: self.permission_manager.clone(),
        }
    }
}

// Implement clone for SandboxManager
impl Clone for SandboxManager {
    fn clone(&self) -> Self {
        Self::new()
    }
}

// Implement clone for PermissionManager
impl Clone for PermissionManager {
    fn clone(&self) -> Self {
        Self::new()
    }
}
