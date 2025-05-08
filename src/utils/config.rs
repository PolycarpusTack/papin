use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::{error, info};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref CONFIG_INSTANCE: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::new()));
}

/// Configuration manager for the application
pub struct Config {
    /// The loaded configuration data
    data: Value,
    
    /// Path to the config file
    config_path: PathBuf,
    
    /// Whether the config has been modified
    dirty: bool,
}

impl Config {
    /// Create a new config instance
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let data = Self::load_config(&config_path).unwrap_or_else(|_| {
            // Create default config
            let default_config = Self::default_config();
            Self::save_config(&config_path, &default_config).unwrap_or_else(|e| {
                error!("Failed to save default config: {}", e);
            });
            default_config
        });
        
        Config {
            data,
            config_path,
            dirty: false,
        }
    }
    
    /// Get the global config instance
    pub fn global() -> Arc<Mutex<Config>> {
        CONFIG_INSTANCE.clone()
    }
    
    /// Create the default configuration
    fn default_config() -> Value {
        let mut config = Map::new();
        
        // App settings
        config.insert("app_name".to_string(), Value::String("Claude MCP Client".to_string()));
        config.insert("version".to_string(), Value::String("0.1.0".to_string()));
        
        // Feature flags
        config.insert("lazy_loading".to_string(), Value::Bool(true));
        config.insert("plugins_enabled".to_string(), Value::Bool(true));
        config.insert("history_enabled".to_string(), Value::Bool(true));
        config.insert("advanced_ui".to_string(), Value::Bool(true));
        config.insert("experimental_features".to_string(), Value::Bool(false));
        config.insert("analytics_enabled".to_string(), Value::Bool(false));
        
        // API settings
        let mut api = Map::new();
        api.insert("base_url".to_string(), Value::String("https://api.anthropic.com".to_string()));
        api.insert("timeout_ms".to_string(), Value::Number(30000.into()));
        config.insert("api".to_string(), Value::Object(api));
        
        // UI settings
        let mut ui = Map::new();
        ui.insert("theme".to_string(), Value::String("system".to_string()));
        ui.insert("font_size".to_string(), Value::Number(14.into()));
        config.insert("ui".to_string(), Value::Object(ui));
        
        Value::Object(config)
    }
    
    /// Get the path to the config file
    fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "claude", "mcp") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir).unwrap_or_else(|e| {
                error!("Failed to create config directory: {}", e);
            });
            config_dir.join("config.json")
        } else {
            // Fallback to current directory if we can't get the project directories
            PathBuf::from("config.json")
        }
    }
    
    /// Load configuration from a file
    fn load_config(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Err("Config file does not exist".into());
        }
        
        let config_str = fs::read_to_string(path)?;
        let config: Value = serde_json::from_str(&config_str)?;
        Ok(config)
    }
    
    /// Save configuration to a file
    fn save_config(path: &Path, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(config)?;
        fs::write(path, config_str)?;
        Ok(())
    }
    
    /// Get a string value from the config
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_value(key).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
    }
    
    /// Get a boolean value from the config
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_value(key).and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
    }
    
    /// Get a number value from the config
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get_value(key).and_then(|v| match v {
            Value::Number(n) => n.as_f64(),
            _ => None,
        })
    }
    
    /// Get a value from the config using a dotted path (e.g. "api.base_url")
    pub fn get_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    if let Some(value) = map.get(part) {
                        current = value;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        
        Some(current)
    }
    
    /// Set a value in the config using a dotted path
    pub fn set_value(&mut self, path: &str, value: Value) -> Result<(), String> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut self.data;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                match current {
                    Value::Object(map) => {
                        map.insert(part.to_string(), value);
                        self.dirty = true;
                        return Ok(());
                    }
                    _ => return Err(format!("Path {} is not an object", path)),
                }
            } else {
                // Navigate to the next part
                match current {
                    Value::Object(map) => {
                        if !map.contains_key(*part) {
                            map.insert(part.to_string(), Value::Object(Map::new()));
                        }
                        if let Some(next) = map.get_mut(*part) {
                            current = next;
                        } else {
                            return Err(format!("Failed to navigate to {}", part));
                        }
                    }
                    _ => return Err(format!("Path {} is not an object", path)),
                }
            }
        }
        
        Err("Empty path".to_string())
    }
    
    /// Save the config to disk
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.dirty {
            Self::save_config(&self.config_path, &self.data)?;
            self.dirty = false;
            info!("Config saved to {}", self.config_path.display());
        }
        Ok(())
    }
    
    /// Reload the config from disk
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data = Self::load_config(&self.config_path)?;
        self.dirty = false;
        info!("Config reloaded from {}", self.config_path.display());
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions to access the global config instance
pub fn get_config() -> Arc<Mutex<Config>> {
    Config::global()
}

pub fn get_string(key: &str) -> Option<String> {
    let config = Config::global();
    let config = config.lock().unwrap();
    config.get_string(key)
}

pub fn get_bool(key: &str) -> Option<bool> {
    let config = Config::global();
    let config = config.lock().unwrap();
    config.get_bool(key)
}

pub fn get_number(key: &str) -> Option<f64> {
    let config = Config::global();
    let config = config.lock().unwrap();
    config.get_number(key)
}

pub fn set_value(key: &str, value: Value) -> Result<(), String> {
    let config = Config::global();
    let mut config = config.lock().unwrap();
    config.set_value(key, value)
}

pub fn save_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::global();
    let mut config = config.lock().unwrap();
    config.save()
}