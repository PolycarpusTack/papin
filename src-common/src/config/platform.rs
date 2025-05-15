// src-common/src/config/platform.rs
//
// Cross-platform configuration management system

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::platform::fs::{platform_fs, PlatformFsError, PathExt};

/// Configuration version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigVersion {
    /// Major version (incompatible changes)
    pub major: u32,
    /// Minor version (backwards-compatible changes)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

impl Default for ConfigVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl ConfigVersion {
    /// Create a new config version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &ConfigVersion) -> bool {
        self.major == other.major
    }

    /// Check if this version is newer than another version
    pub fn is_newer_than(&self, other: &ConfigVersion) -> bool {
        if self.major > other.major {
            return true;
        }
        if self.major < other.major {
            return false;
        }
        if self.minor > other.minor {
            return true;
        }
        if self.minor < other.minor {
            return false;
        }
        self.patch > other.patch
    }
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Configuration version
    pub version: ConfigVersion,
    /// Last modified timestamp
    pub last_modified: SystemTime,
    /// Platform where the config was last saved
    pub platform: String,
    /// Application version that last saved the config
    pub app_version: String,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            version: ConfigVersion::default(),
            last_modified: SystemTime::now(),
            platform: platform_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] PlatformFsError),
    
    #[error("Config not found: {0}")]
    NotFound(String),
    
    #[error("Config incompatible: expected v{0}.x.x, found v{1}.{2}.{3}")]
    Incompatible(u32, u32, u32, u32),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Invalid config: {0}")]
    Invalid(String),
}

/// Platform-specific configuration defaults
pub trait PlatformDefaults {
    /// Get a platform-specific default value for a given key
    fn get_default(&self, key: &str) -> Option<serde_json::Value>;
}

/// Windows platform defaults
pub struct WindowsDefaults;

impl PlatformDefaults for WindowsDefaults {
    fn get_default(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "appearance.theme" => Some(serde_json::json!("light")),
            "appearance.use_system_theme" => Some(serde_json::json!(true)),
            "app.minimize_to_tray" => Some(serde_json::json!(true)),
            "app.start_on_boot" => Some(serde_json::json!(false)),
            "networking.proxy.use_system" => Some(serde_json::json!(true)),
            "paths.downloads" => Some(serde_json::json!(default_downloads_dir().to_string_lossy().to_string())),
            _ => None,
        }
    }
}

/// macOS platform defaults
pub struct MacOSDefaults;

impl PlatformDefaults for MacOSDefaults {
    fn get_default(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "appearance.theme" => Some(serde_json::json!("system")),
            "appearance.use_system_theme" => Some(serde_json::json!(true)),
            "app.minimize_to_tray" => Some(serde_json::json!(false)),
            "app.start_on_boot" => Some(serde_json::json!(false)),
            "networking.proxy.use_system" => Some(serde_json::json!(true)),
            "paths.downloads" => Some(serde_json::json!(default_downloads_dir().to_string_lossy().to_string())),
            _ => None,
        }
    }
}

/// Linux platform defaults
pub struct LinuxDefaults;

impl PlatformDefaults for LinuxDefaults {
    fn get_default(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "appearance.theme" => Some(serde_json::json!("dark")),
            "appearance.use_system_theme" => Some(serde_json::json!(true)),
            "app.minimize_to_tray" => Some(serde_json::json!(true)),
            "app.start_on_boot" => Some(serde_json::json!(false)),
            "networking.proxy.use_system" => Some(serde_json::json!(true)),
            "paths.downloads" => Some(serde_json::json!(default_downloads_dir().to_string_lossy().to_string())),
            _ => None,
        }
    }
}

/// Get the default downloads directory
fn default_downloads_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join("Downloads")
    })
}

/// Get a string representing the current platform
fn platform_string() -> String {
    let fs = platform_fs();
    match fs.platform() {
        crate::platform::fs::Platform::Windows => "windows".to_string(),
        crate::platform::fs::Platform::MacOS => "macos".to_string(),
        crate::platform::fs::Platform::Linux => "linux".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Config migration handler type
pub type MigrationHandler = fn(&mut ConfigManager, &ConfigVersion) -> Result<(), ConfigError>;

/// Configuration manager
pub struct ConfigManager {
    config_dir: PathBuf,
    config: RwLock<HashMap<String, serde_json::Value>>,
    metadata: RwLock<ConfigMetadata>,
    filename: String,
    auto_save: bool,
    migrations: Vec<(ConfigVersion, MigrationHandler)>,
    platform_defaults: Box<dyn PlatformDefaults + Send + Sync>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(filename: &str) -> Result<Self, ConfigError> {
        let fs = platform_fs();
        let config_dir = fs.app_data_dir("Papin")?;
        
        // Create platform-specific defaults
        let platform_defaults: Box<dyn PlatformDefaults + Send + Sync> = match fs.platform() {
            crate::platform::fs::Platform::Windows => Box::new(WindowsDefaults),
            crate::platform::fs::Platform::MacOS => Box::new(MacOSDefaults),
            crate::platform::fs::Platform::Linux => Box::new(LinuxDefaults),
            _ => Box::new(LinuxDefaults), // Fallback to Linux defaults
        };
        
        let manager = Self {
            config_dir,
            config: RwLock::new(HashMap::new()),
            metadata: RwLock::new(ConfigMetadata::default()),
            filename: filename.to_string(),
            auto_save: true,
            migrations: Vec::new(),
            platform_defaults,
        };
        
        // Create the config directory if it doesn't exist
        fs.ensure_dir_exists(&manager.config_dir)?;
        
        Ok(manager)
    }
    
    /// Register a migration handler
    pub fn register_migration(&mut self, version: ConfigVersion, handler: MigrationHandler) {
        // Insert sorted by version (newest first)
        let pos = self.migrations.iter()
            .position(|(v, _)| !version.is_newer_than(v))
            .unwrap_or(self.migrations.len());
        
        self.migrations.insert(pos, (version, handler));
    }
    
    /// Set auto-save behavior
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }
    
    /// Load configuration from file
    pub fn load(&self) -> Result<(), ConfigError> {
        let path = self.config_path();
        let fs = platform_fs();
        
        // Check if the file exists
        if !fs.file_exists(&path) {
            // Not an error, just use defaults
            return Ok(());
        }
        
        // Read the file
        let content = fs.read_to_string(&path)?;
        
        // Parse the JSON
        let mut parsed: serde_json::Value = serde_json::from_str(&content)?;
        
        // Extract metadata
        let metadata = if let Some(meta_value) = parsed.get("_metadata") {
            match serde_json::from_value::<ConfigMetadata>(meta_value.clone()) {
                Ok(meta) => meta,
                Err(e) => {
                    return Err(ConfigError::Invalid(format!("Invalid metadata: {}", e)));
                }
            }
        } else {
            // No metadata, assume default version
            ConfigMetadata::default()
        };
        
        // Check if the version is compatible
        let current_version = ConfigVersion::default();
        if !current_version.is_compatible_with(&metadata.version) {
            return Err(ConfigError::Incompatible(
                current_version.major,
                metadata.version.major,
                metadata.version.minor,
                metadata.version.patch,
            ));
        }
        
        // Apply migrations if needed
        if current_version.is_newer_than(&metadata.version) {
            // Find all migrations that need to be applied
            let applicable_migrations: Vec<_> = self.migrations.iter()
                .filter(|(v, _)| v.is_newer_than(&metadata.version) && !v.is_newer_than(&current_version))
                .collect();
            
            if !applicable_migrations.is_empty() {
                // Create a copy of the configuration
                let mut config_copy = self.clone();
                
                // Extract config data (excluding metadata)
                if let Some(obj) = parsed.as_object_mut() {
                    obj.remove("_metadata");
                }
                
                let mut config_data = match serde_json::from_value::<HashMap<String, serde_json::Value>>(parsed.clone()) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(ConfigError::Invalid(format!("Invalid config data: {}", e)));
                    }
                };
                
                // Set the config data
                *config_copy.config.write().unwrap() = config_data.clone();
                
                // Apply migrations in version order (oldest to newest)
                for (version, handler) in applicable_migrations.iter().rev() {
                    log::info!("Applying config migration to v{}.{}.{}", 
                              version.major, version.minor, version.patch);
                    
                    if let Err(e) = handler(&mut config_copy, &metadata.version) {
                        return Err(ConfigError::Migration(format!("Migration to v{}.{}.{} failed: {}", 
                                                               version.major, version.minor, version.patch, e)));
                    }
                }
                
                // Get the migrated config
                config_data = config_copy.config.read().unwrap().clone();
                
                // Save to our config
                *self.config.write().unwrap() = config_data;
                
                // Update metadata
                let mut meta = self.metadata.write().unwrap();
                meta.version = current_version;
                meta.last_modified = SystemTime::now();
                
                // Save the migrated config
                self.save()?;
            } else {
                // No migrations needed, just update metadata
                let mut meta = self.metadata.write().unwrap();
                *meta = metadata;
                meta.version = current_version;
                
                // Extract config data (excluding metadata)
                if let Some(obj) = parsed.as_object_mut() {
                    obj.remove("_metadata");
                }
                
                let config_data = match serde_json::from_value::<HashMap<String, serde_json::Value>>(parsed) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(ConfigError::Invalid(format!("Invalid config data: {}", e)));
                    }
                };
                
                *self.config.write().unwrap() = config_data;
            }
        } else {
            // No migrations needed, just load the config
            let mut meta = self.metadata.write().unwrap();
            *meta = metadata;
            
            // Extract config data (excluding metadata)
            if let Some(obj) = parsed.as_object_mut() {
                obj.remove("_metadata");
            }
            
            let config_data = match serde_json::from_value::<HashMap<String, serde_json::Value>>(parsed) {
                Ok(data) => data,
                Err(e) => {
                    return Err(ConfigError::Invalid(format!("Invalid config data: {}", e)));
                }
            };
            
            *self.config.write().unwrap() = config_data;
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = self.config_path();
        let fs = platform_fs();
        
        // Update metadata
        {
            let mut meta = self.metadata.write().unwrap();
            meta.last_modified = SystemTime::now();
            meta.platform = platform_string();
            meta.app_version = env!("CARGO_PKG_VERSION").to_string();
        }
        
        // Get config data
        let config_data = self.config.read().unwrap().clone();
        let metadata = self.metadata.read().unwrap().clone();
        
        // Combine config and metadata
        let mut combined = serde_json::Map::new();
        combined.insert("_metadata".to_string(), serde_json::to_value(metadata)?);
        
        for (key, value) in config_data {
            combined.insert(key, value);
        }
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&combined)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs.ensure_dir_exists(parent)?;
        }
        
        // Write to file
        fs.write_string(&path, &json)?;
        
        Ok(())
    }
    
    /// Get a value from the configuration
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        // Try to get from config
        if let Some(value) = self.config.read().unwrap().get(key) {
            if let Ok(typed) = serde_json::from_value(value.clone()) {
                return Some(typed);
            }
        }
        
        // Try to get from platform defaults
        if let Some(default) = self.platform_defaults.get_default(key) {
            if let Ok(typed) = serde_json::from_value(default) {
                return Some(typed);
            }
        }
        
        None
    }
    
    /// Get a value or a default
    pub fn get_or<T: for<'de> Deserialize<'de>>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }
    
    /// Get a value as JSON
    pub fn get_json(&self, key: &str) -> Option<serde_json::Value> {
        // Try to get from config
        if let Some(value) = self.config.read().unwrap().get(key) {
            return Some(value.clone());
        }
        
        // Try to get from platform defaults
        self.platform_defaults.get_default(key)
    }
    
    /// Set a value in the configuration
    pub fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), ConfigError> {
        let json_value = serde_json::to_value(value)?;
        
        // Update the config
        {
            let mut config = self.config.write().unwrap();
            config.insert(key.to_string(), json_value);
        }
        
        // Auto-save if enabled
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Set a JSON value in the configuration
    pub fn set_json(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        // Update the config
        {
            let mut config = self.config.write().unwrap();
            config.insert(key.to_string(), value);
        }
        
        // Auto-save if enabled
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Remove a value from the configuration
    pub fn remove(&self, key: &str) -> Result<(), ConfigError> {
        // Update the config
        {
            let mut config = self.config.write().unwrap();
            config.remove(key);
        }
        
        // Auto-save if enabled
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Clear the configuration
    pub fn clear(&self) -> Result<(), ConfigError> {
        // Clear the config
        {
            let mut config = self.config.write().unwrap();
            config.clear();
        }
        
        // Auto-save if enabled
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Check if a key exists in the configuration
    pub fn has_key(&self, key: &str) -> bool {
        self.config.read().unwrap().contains_key(key)
    }
    
    /// Get all keys in the configuration
    pub fn keys(&self) -> Vec<String> {
        self.config.read().unwrap().keys().cloned().collect()
    }
    
    /// Get the configuration path
    fn config_path(&self) -> PathBuf {
        self.config_dir.join(&self.filename)
    }
    
    /// Get the configuration directory
    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }
    
    /// Create a clone of this manager (for migration)
    fn clone(&self) -> Self {
        // Get platform defaults
        let platform_defaults: Box<dyn PlatformDefaults + Send + Sync> = match platform_string().as_str() {
            "windows" => Box::new(WindowsDefaults),
            "macos" => Box::new(MacOSDefaults),
            "linux" => Box::new(LinuxDefaults),
            _ => Box::new(LinuxDefaults), // Fallback to Linux defaults
        };
        
        Self {
            config_dir: self.config_dir.clone(),
            config: RwLock::new(HashMap::new()),
            metadata: RwLock::new(self.metadata.read().unwrap().clone()),
            filename: self.filename.clone(),
            auto_save: false, // Don't auto-save during migration
            migrations: Vec::new(), // Migrations are not needed in the clone
            platform_defaults,
        }
    }
}

/// Configuration manager builder
pub struct ConfigManagerBuilder {
    filename: String,
    auto_save: bool,
    migrations: Vec<(ConfigVersion, MigrationHandler)>,
}

impl ConfigManagerBuilder {
    /// Create a new configuration manager builder
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            auto_save: true,
            migrations: Vec::new(),
        }
    }
    
    /// Set auto-save behavior
    pub fn auto_save(mut self, auto_save: bool) -> Self {
        self.auto_save = auto_save;
        self
    }
    
    /// Register a migration handler
    pub fn with_migration(mut self, version: ConfigVersion, handler: MigrationHandler) -> Self {
        self.migrations.push((version, handler));
        self
    }
    
    /// Build the configuration manager
    pub fn build(self) -> Result<ConfigManager, ConfigError> {
        let mut manager = ConfigManager::new(&self.filename)?;
        
        // Set auto-save
        manager.set_auto_save(self.auto_save);
        
        // Register migrations
        for (version, handler) in self.migrations {
            manager.register_migration(version, handler);
        }
        
        // Load configuration
        if let Err(e) = manager.load() {
            log::warn!("Failed to load configuration: {}", e);
            // Don't propagate the error, just start with defaults
        }
        
        Ok(manager)
    }
}

// Global config manager instance
lazy_static::lazy_static! {
    static ref CONFIG_MANAGERS: RwLock<HashMap<String, Arc<ConfigManager>>> = RwLock::new(HashMap::new());
}

/// Get a configuration manager for a specific file
pub fn get_config_manager(filename: &str) -> Result<Arc<ConfigManager>, ConfigError> {
    let mut managers = CONFIG_MANAGERS.write().unwrap();
    
    if let Some(manager) = managers.get(filename) {
        return Ok(manager.clone());
    }
    
    let manager = Arc::new(ConfigManager::new(filename)?);
    
    // Try to load configuration
    if let Err(e) = manager.load() {
        log::warn!("Failed to load configuration from {}: {}", filename, e);
        // Don't propagate the error, just start with defaults
    }
    
    managers.insert(filename.to_string(), manager.clone());
    
    Ok(manager)
}

/// Clear all configuration managers
pub fn clear_config_managers() {
    let mut managers = CONFIG_MANAGERS.write().unwrap();
    managers.clear();
}

/// Example migration handler for v1.1.0
pub fn migrate_to_v1_1_0(manager: &mut ConfigManager, from_version: &ConfigVersion) -> Result<(), ConfigError> {
    // This is just an example - customize for your actual migrations
    log::info!("Migrating configuration from v{}.{}.{} to v1.1.0", 
               from_version.major, from_version.minor, from_version.patch);
    
    // Example: Rename a configuration key
    if let Some(value) = manager.get_json("old_key") {
        manager.set_json("new_key", value)?;
        manager.remove("old_key")?;
    }
    
    // Example: Change the format of a value
    if let Some(old_value) = manager.get::<String>("some_setting") {
        let new_value = format!("updated:{}", old_value);
        manager.set("some_setting", new_value)?;
    }
    
    Ok(())
}

/// Example migration handler for v2.0.0
pub fn migrate_to_v2_0_0(manager: &mut ConfigManager, from_version: &ConfigVersion) -> Result<(), ConfigError> {
    // This is just an example - customize for your actual migrations
    log::info!("Migrating configuration from v{}.{}.{} to v2.0.0", 
               from_version.major, from_version.minor, from_version.patch);
    
    // Example: Restructure configuration
    if let Some(old_value) = manager.get::<String>("theme") {
        manager.set("appearance.theme", old_value)?;
        manager.remove("theme")?;
    }
    
    // Example: Convert a string to an object
    if let Some(old_path) = manager.get::<String>("download_path") {
        let downloads = serde_json::json!({
            "path": old_path,
            "ask_before_download": true,
        });
        manager.set_json("downloads", downloads)?;
        manager.remove("download_path")?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_version_compatibility() {
        let v1_0_0 = ConfigVersion::new(1, 0, 0);
        let v1_1_0 = ConfigVersion::new(1, 1, 0);
        let v1_1_1 = ConfigVersion::new(1, 1, 1);
        let v2_0_0 = ConfigVersion::new(2, 0, 0);
        
        // Test compatibility
        assert!(v1_0_0.is_compatible_with(&v1_0_0));
        assert!(v1_0_0.is_compatible_with(&v1_1_0));
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
        
        // Test newer than
        assert!(!v1_0_0.is_newer_than(&v1_0_0));
        assert!(v1_1_0.is_newer_than(&v1_0_0));
        assert!(v1_1_1.is_newer_than(&v1_1_0));
        assert!(v2_0_0.is_newer_than(&v1_1_1));
        assert!(!v1_1_1.is_newer_than(&v2_0_0));
    }
    
    #[test]
    fn test_basic_config_operations() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config");
        
        // Create a config manager
        let mut manager = ConfigManager::new("test_config.json").unwrap();
        
        // Override config directory for testing
        manager.config_dir = config_path;
        
        // Set some values
        manager.set("test_string", "Hello, world!").unwrap();
        manager.set("test_number", 42).unwrap();
        manager.set("test_boolean", true).unwrap();
        
        // Get the values
        assert_eq!(manager.get::<String>("test_string").unwrap(), "Hello, world!");
        assert_eq!(manager.get::<i32>("test_number").unwrap(), 42);
        assert_eq!(manager.get::<bool>("test_boolean").unwrap(), true);
        
        // Check if keys exist
        assert!(manager.has_key("test_string"));
        assert!(manager.has_key("test_number"));
        assert!(manager.has_key("test_boolean"));
        assert!(!manager.has_key("nonexistent"));
        
        // Get keys
        let keys = manager.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"test_string".to_string()));
        assert!(keys.contains(&"test_number".to_string()));
        assert!(keys.contains(&"test_boolean".to_string()));
        
        // Remove a value
        manager.remove("test_boolean").unwrap();
        assert!(!manager.has_key("test_boolean"));
        
        // Clear the config
        manager.clear().unwrap();
        assert_eq!(manager.keys().len(), 0);
    }
    
    #[test]
    fn test_config_save_load() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config");
        
        // Create a config manager
        let mut manager = ConfigManager::new("test_config.json").unwrap();
        
        // Override config directory for testing
        manager.config_dir = config_path.clone();
        
        // Set some values
        manager.set("test_string", "Hello, world!").unwrap();
        manager.set("test_number", 42).unwrap();
        
        // Save the config
        manager.save().unwrap();
        
        // Create a new manager with the same config file
        let mut manager2 = ConfigManager::new("test_config.json").unwrap();
        manager2.config_dir = config_path;
        
        // Load the config
        manager2.load().unwrap();
        
        // Check if the values were loaded correctly
        assert_eq!(manager2.get::<String>("test_string").unwrap(), "Hello, world!");
        assert_eq!(manager2.get::<i32>("test_number").unwrap(), 42);
    }
    
    #[test]
    fn test_platform_defaults() {
        // Get the current platform
        let platform = platform_string();
        
        // Create defaults for the platform
        let defaults: Box<dyn PlatformDefaults + Send + Sync> = match platform.as_str() {
            "windows" => Box::new(WindowsDefaults),
            "macos" => Box::new(MacOSDefaults),
            "linux" => Box::new(LinuxDefaults),
            _ => Box::new(LinuxDefaults), // Fallback to Linux defaults
        };
        
        // Test platform-specific defaults
        match platform.as_str() {
            "windows" => {
                assert_eq!(defaults.get_default("appearance.theme").unwrap(), serde_json::json!("light"));
                assert_eq!(defaults.get_default("app.minimize_to_tray").unwrap(), serde_json::json!(true));
            },
            "macos" => {
                assert_eq!(defaults.get_default("appearance.theme").unwrap(), serde_json::json!("system"));
                assert_eq!(defaults.get_default("app.minimize_to_tray").unwrap(), serde_json::json!(false));
            },
            "linux" => {
                assert_eq!(defaults.get_default("appearance.theme").unwrap(), serde_json::json!("dark"));
                assert_eq!(defaults.get_default("app.minimize_to_tray").unwrap(), serde_json::json!(true));
            },
            _ => {
                // Unknown platform, should fall back to Linux defaults
                assert_eq!(defaults.get_default("appearance.theme").unwrap(), serde_json::json!("dark"));
                assert_eq!(defaults.get_default("app.minimize_to_tray").unwrap(), serde_json::json!(true));
            },
        }
    }
    
    #[test]
    fn test_migration() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config");
        
        // Create a migration handler
        fn test_migration_handler(manager: &mut ConfigManager, _: &ConfigVersion) -> Result<(), ConfigError> {
            // Change the format of the test_string value
            if let Some(old_value) = manager.get::<String>("test_string") {
                let new_value = format!("migrated:{}", old_value);
                manager.set("test_string", new_value)?;
            }
            
            // Add a new value
            manager.set("migration_added", true)?;
            
            Ok(())
        }
        
        // Create a config manager with an older version
        let mut manager = ConfigManager::new("test_config.json").unwrap();
        manager.config_dir = config_path.clone();
        
        // Set the version to an older one
        {
            let mut meta = manager.metadata.write().unwrap();
            meta.version = ConfigVersion::new(1, 0, 0);
        }
        
        // Set some values
        manager.set("test_string", "Hello, world!").unwrap();
        
        // Save the config
        manager.save().unwrap();
        
        // Create a new manager with migration
        let mut builder = ConfigManagerBuilder::new("test_config.json");
        builder = builder.with_migration(ConfigVersion::new(1, 1, 0), test_migration_handler);
        
        let mut manager2 = builder.build().unwrap();
        manager2.config_dir = config_path;
        
        // Load the config (should trigger migration)
        manager2.load().unwrap();
        
        // Check if the migration was applied
        assert_eq!(manager2.get::<String>("test_string").unwrap(), "migrated:Hello, world!");
        assert_eq!(manager2.get::<bool>("migration_added").unwrap(), true);
        
        // Check if the version was updated
        let meta = manager2.metadata.read().unwrap();
        assert_eq!(meta.version.major, 1);
        assert_eq!(meta.version.minor, 0); // Version is set to current (default), which is 1.0.0
        assert_eq!(meta.version.patch, 0);
    }
}
