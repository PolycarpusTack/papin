use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use reqwest;

use crate::plugins::types::{RepositoryPlugin, PluginInfo};

/// Plugin discovery
pub struct PluginDiscovery {
    /// Plugin repositories
    repositories: RwLock<Vec<PluginRepository>>,
    /// Cached plugins from repositories
    cached_plugins: RwLock<HashMap<String, RepositoryPlugin>>,
    /// Last cache update
    last_cache_update: RwLock<Option<chrono::DateTime<chrono::Utc>>>,
}

/// Plugin repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepository {
    /// Repository name
    pub name: String,
    /// Repository URL
    pub url: String,
    /// Repository type
    pub repo_type: RepositoryType,
    /// Enabled state
    pub enabled: bool,
}

/// Repository type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepositoryType {
    /// Official repository
    Official,
    /// Community repository
    Community,
    /// Local repository
    Local,
}

impl PluginDiscovery {
    /// Create a new plugin discovery
    pub fn new() -> Self {
        Self {
            repositories: RwLock::new(Vec::new()),
            cached_plugins: RwLock::new(HashMap::new()),
            last_cache_update: RwLock::new(None),
        }
    }
    
    /// Initialize the plugin discovery
    pub async fn initialize(&self) -> Result<(), String> {
        log::info!("Initializing plugin discovery");
        
        // Set up default repositories
        let mut repositories = self.repositories.write().await;
        
        // If no repositories are configured, add defaults
        if repositories.is_empty() {
            repositories.push(PluginRepository {
                name: "Official Repository".to_string(),
                url: "https://plugins.claudemcp.io/api/plugins".to_string(),
                repo_type: RepositoryType::Official,
                enabled: true,
            });
            
            repositories.push(PluginRepository {
                name: "Community Repository".to_string(),
                url: "https://community.claudemcp.io/api/plugins".to_string(),
                repo_type: RepositoryType::Community,
                enabled: true,
            });
        }
        
        log::info!("Plugin discovery initialized with {} repositories", repositories.len());
        
        // Update plugin cache
        drop(repositories);
        match self.update_cache().await {
            Ok(_) => {
                log::info!("Plugin cache updated");
            }
            Err(e) => {
                log::warn!("Failed to update plugin cache: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Update plugin cache
    pub async fn update_cache(&self) -> Result<(), String> {
        log::info!("Updating plugin cache");
        
        let repositories = self.repositories.read().await;
        let mut cached_plugins = self.cached_plugins.write().await;
        let mut last_cache_update = self.last_cache_update.write().await;
        
        // Clear cache
        cached_plugins.clear();
        
        // Fetch plugins from each repository
        for repo in repositories.iter().filter(|r| r.enabled) {
            match self.fetch_repository_plugins(repo).await {
                Ok(plugins) => {
                    // Add to cache
                    for plugin in plugins {
                        cached_plugins.insert(plugin.id.clone(), plugin);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch plugins from repository {}: {}", repo.name, e);
                }
            }
        }
        
        // Update last cache update
        *last_cache_update = Some(chrono::Utc::now());
        
        log::info!("Plugin cache updated with {} plugins", cached_plugins.len());
        Ok(())
    }
    
    /// Fetch plugins from a repository
    async fn fetch_repository_plugins(&self, repo: &PluginRepository) -> Result<Vec<RepositoryPlugin>, String> {
        match repo.repo_type {
            RepositoryType::Official | RepositoryType::Community => {
                // Fetch from remote repository
                self.fetch_remote_repository_plugins(repo).await
            }
            RepositoryType::Local => {
                // Fetch from local repository
                self.fetch_local_repository_plugins(repo).await
            }
        }
    }
    
    /// Fetch plugins from a remote repository
    async fn fetch_remote_repository_plugins(&self, repo: &PluginRepository) -> Result<Vec<RepositoryPlugin>, String> {
        log::info!("Fetching plugins from remote repository: {}", repo.name);
        
        // Create HTTP client
        let client = reqwest::Client::new();
        
        // Send request
        let response = client.get(&repo.url)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;
            
        // Check status
        if !response.status().is_success() {
            return Err(format!("Request failed with status: {}", response.status()));
        }
        
        // Parse response
        let plugins: Vec<RepositoryPlugin> = response.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
            
        log::info!("Fetched {} plugins from remote repository: {}", plugins.len(), repo.name);
        Ok(plugins)
    }
    
    /// Fetch plugins from a local repository
    async fn fetch_local_repository_plugins(&self, repo: &PluginRepository) -> Result<Vec<RepositoryPlugin>, String> {
        log::info!("Fetching plugins from local repository: {}", repo.name);
        
        // Parse local repository path
        let repo_path = std::path::Path::new(&repo.url);
        
        // Check if directory exists
        if !repo_path.exists() || !repo_path.is_dir() {
            return Err(format!("Local repository directory not found: {}", repo.url));
        }
        
        // Read directory entries
        let mut entries = tokio::fs::read_dir(repo_path)
            .await
            .map_err(|e| format!("Failed to read repository directory: {}", e))?;
            
        let mut plugins = Vec::new();
        
        // Process each entry
        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))? {
                
            let path = entry.path();
            
            // Check if it's a directory
            if path.is_dir() {
                continue;
            }
            
            // Check file extension
            if let Some(ext) = path.extension() {
                if ext != "zip" {
                    continue;
                }
            } else {
                continue;
            }
            
            // Try to read plugin metadata
            match self.read_local_plugin_metadata(&path).await {
                Ok(plugin) => {
                    plugins.push(plugin);
                }
                Err(e) => {
                    log::warn!("Failed to read plugin metadata from {}: {}", path.display(), e);
                }
            }
        }
        
        log::info!("Fetched {} plugins from local repository: {}", plugins.len(), repo.name);
        Ok(plugins)
    }
    
    /// Read plugin metadata from a local file
    async fn read_local_plugin_metadata(&self, path: &std::path::Path) -> Result<RepositoryPlugin, String> {
        // Open ZIP file
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open plugin package: {}", e))?;
            
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
            
        // Look for manifest.json
        let mut manifest_file = archive.by_name("manifest.json")
            .map_err(|e| format!("Manifest file not found in plugin package: {}", e))?;
            
        // Read manifest
        let mut manifest_content = String::new();
        std::io::Read::read_to_string(&mut manifest_file, &mut manifest_content)
            .map_err(|e| format!("Failed to read manifest file: {}", e))?;
            
        // Parse manifest
        let manifest: crate::plugins::types::PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;
            
        // Create repository plugin
        let plugin = RepositoryPlugin {
            id: manifest.name.clone(),
            display_name: manifest.display_name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            author: manifest.author.clone(),
            downloads: 0,
            rating: 0.0,
            repo_url: path.to_string_lossy().to_string(),
            download_url: path.to_string_lossy().to_string(),
            screenshots: Vec::new(),
            permissions: manifest.permissions.clone(),
        };
        
        Ok(plugin)
    }
    
    /// Search for plugins
    pub async fn search_plugins(&self, query: &str) -> Result<Vec<PluginInfo>, String> {
        log::info!("Searching for plugins with query: {}", query);
        
        // Check if cache is stale (older than 1 hour)
        let last_update = self.last_cache_update.read().await;
        let now = chrono::Utc::now();
        let is_stale = match *last_update {
            Some(timestamp) => {
                (now - timestamp).num_seconds() > 3600
            }
            None => true,
        };
        
        // Update cache if stale
        drop(last_update);
        if is_stale {
            match self.update_cache().await {
                Ok(_) => {
                    log::info!("Plugin cache updated");
                }
                Err(e) => {
                    log::warn!("Failed to update plugin cache: {}", e);
                }
            }
        }
        
        // Get cached plugins
        let cached_plugins = self.cached_plugins.read().await;
        
        // Filter by query
        let query = query.to_lowercase();
        let results: Vec<PluginInfo> = cached_plugins.values()
            .filter(|plugin| {
                // Match on name, display name, or description
                plugin.id.to_lowercase().contains(&query) ||
                plugin.display_name.to_lowercase().contains(&query) ||
                plugin.description.to_lowercase().contains(&query)
            })
            .map(|plugin| {
                // Convert to plugin info
                PluginInfo {
                    id: plugin.id.clone(),
                    display_name: plugin.display_name.clone(),
                    version: plugin.version.clone(),
                    description: plugin.description.clone(),
                    author: plugin.author.clone(),
                    active: false,
                    installed_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                }
            })
            .collect();
            
        log::info!("Found {} plugins matching query: {}", results.len(), query);
        Ok(results)
    }
    
    /// Add a repository
    pub async fn add_repository(&self, repo: PluginRepository) -> Result<(), String> {
        log::info!("Adding repository: {}", repo.name);
        
        // Check if repository with same name exists
        let mut repositories = self.repositories.write().await;
        if repositories.iter().any(|r| r.name == repo.name) {
            return Err(format!("Repository with name {} already exists", repo.name));
        }
        
        // Add repository
        repositories.push(repo);
        
        // Update plugin cache
        drop(repositories);
        self.update_cache().await?;
        
        Ok(())
    }
    
    /// Remove a repository
    pub async fn remove_repository(&self, name: &str) -> Result<(), String> {
        log::info!("Removing repository: {}", name);
        
        // Remove repository
        let mut repositories = self.repositories.write().await;
        let pos = repositories.iter().position(|r| r.name == name);
        
        if let Some(idx) = pos {
            repositories.remove(idx);
            
            // Update plugin cache
            drop(repositories);
            self.update_cache().await?;
            
            Ok(())
        } else {
            Err(format!("Repository {} not found", name))
        }
    }
    
    /// Enable or disable a repository
    pub async fn set_repository_enabled(&self, name: &str, enabled: bool) -> Result<(), String> {
        log::info!("Setting repository {} enabled state to {}", name, enabled);
        
        // Find repository
        let mut repositories = self.repositories.write().await;
        let pos = repositories.iter().position(|r| r.name == name);
        
        if let Some(idx) = pos {
            repositories[idx].enabled = enabled;
            
            // Update plugin cache
            drop(repositories);
            self.update_cache().await?;
            
            Ok(())
        } else {
            Err(format!("Repository {} not found", name))
        }
    }
    
    /// Get all repositories
    pub async fn get_repositories(&self) -> Vec<PluginRepository> {
        self.repositories.read().await.clone()
    }
    
    /// Get a specific plugin by ID
    pub async fn get_plugin(&self, id: &str) -> Option<RepositoryPlugin> {
        let cached_plugins = self.cached_plugins.read().await;
        cached_plugins.get(id).cloned()
    }
    
    /// Get all available plugins
    pub async fn get_all_plugins(&self) -> Vec<RepositoryPlugin> {
        let cached_plugins = self.cached_plugins.read().await;
        cached_plugins.values().cloned().collect()
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
