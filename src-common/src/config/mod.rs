mod settings;
mod storage;

use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use settings::Settings;
pub use storage::StorageManager;

/// Global settings instance
static SETTINGS: OnceCell<Arc<Mutex<Settings>>> = OnceCell::new();

/// Global storage manager instance
static STORAGE_MANAGER: OnceCell<Arc<StorageManager>> = OnceCell::new();

/// Get the global settings instance
pub fn get_settings() -> Arc<Mutex<Settings>> {
    SETTINGS.get_or_init(|| {
        Arc::new(Mutex::new(Settings::load().unwrap_or_default()))
    }).clone()
}

/// Get the global storage manager instance
pub fn get_storage_manager() -> Arc<StorageManager> {
    STORAGE_MANAGER.get_or_init(|| {
        Arc::new(StorageManager::new())
    }).clone()
}

/// Get the application config directory
pub fn get_config_dir() -> PathBuf {
    let proj_dirs = directories::ProjectDirs::from("com", "anthropic", "mcp-client")
        .expect("Failed to determine config directory");
    
    let config_dir = proj_dirs.config_dir().to_path_buf();
    
    // Create if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }
    
    config_dir
}

/// Get the application data directory
pub fn get_data_dir() -> PathBuf {
    let proj_dirs = directories::ProjectDirs::from("com", "anthropic", "mcp-client")
        .expect("Failed to determine data directory");
    
    let data_dir = proj_dirs.data_dir().to_path_buf();
    
    // Create if it doesn't exist
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    }
    
    data_dir
}

/// Get a path within the config directory
pub fn config_path(filename: &str) -> PathBuf {
    let mut path = get_config_dir();
    path.push(filename);
    path
}

/// Get a path within the data directory
pub fn data_path(filename: &str) -> PathBuf {
    let mut path = get_data_dir();
    path.push(filename);
    path
}
