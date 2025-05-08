use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::config_path;
use crate::error::{McpError, McpResult};
use crate::utils::security;

const SETTINGS_FILE: &str = "settings.json";
const API_KEY_FILE: &str = "credentials.enc";

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// API configuration
    pub api: ApiSettings,
    
    /// UI configuration
    pub ui: UiSettings,
    
    /// Model configuration
    pub model: ModelSettings,
}

/// API settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSettings {
    /// API endpoint URL
    pub url: String,
    
    /// Default model to use
    pub model: String,
    
    /// API version
    pub version: String,
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Use dark mode
    pub dark_mode: bool,
    
    /// Font size
    pub font_size: u8,
    
    /// Enable animations
    pub animations: bool,
    
    /// Use system theme
    pub system_theme: bool,
}

/// Model settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    /// Default temperature
    pub temperature: f32,
    
    /// Default max tokens
    pub max_tokens: u32,
    
    /// Default system prompt
    pub system_prompt: Option<String>,
    
    /// Enable streaming
    pub streaming: bool,
}

impl Settings {
    /// Load settings from file
    pub fn load() -> McpResult<Self> {
        let path = config_path(SETTINGS_FILE);
        
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| McpError::Io(e))?;
                
            let settings = serde_json::from_str(&content)
                .map_err(|e| McpError::Serialization(e))?;
                
            Ok(settings)
        } else {
            // Create default settings
            let settings = Self::default();
            settings.save()?;
            Ok(settings)
        }
    }
    
    /// Save settings to file
    pub fn save(&self) -> McpResult<()> {
        let path = config_path(SETTINGS_FILE);
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| McpError::Serialization(e))?;
            
        fs::write(path, content)
            .map_err(|e| McpError::Io(e))?;
            
        Ok(())
    }
    
    /// Get API key (will be decrypted)
    pub fn get_api_key(&self) -> McpResult<Option<String>> {
        let path = config_path(API_KEY_FILE);
        
        if path.exists() {
            let encrypted = fs::read(&path)
                .map_err(|e| McpError::Io(e))?;
                
            let key = security::decrypt(&encrypted)
                .map_err(|e| McpError::Config(format!("Failed to decrypt API key: {}", e)))?;
                
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
    
    /// Set API key (will be encrypted)
    pub fn set_api_key(&self, api_key: &str) -> McpResult<()> {
        let path = config_path(API_KEY_FILE);
        
        let encrypted = security::encrypt(api_key)
            .map_err(|e| McpError::Config(format!("Failed to encrypt API key: {}", e)))?;
            
        fs::write(path, encrypted)
            .map_err(|e| McpError::Io(e))?;
            
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api: ApiSettings {
                url: "wss://api.anthropic.com/v1/messages".to_string(),
                model: "claude-3-sonnet-20240229".to_string(),
                version: "v1".to_string(),
            },
            ui: UiSettings {
                dark_mode: false,
                font_size: 14,
                animations: true,
                system_theme: true,
            },
            model: ModelSettings {
                temperature: 0.7,
                max_tokens: 4096,
                system_prompt: None,
                streaming: true,
            },
        }
    }
}
