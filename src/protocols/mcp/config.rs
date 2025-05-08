use crate::protocols::ProtocolConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// MCP protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// WebSocket URL for MCP server
    pub url: String,
    
    /// API key for authentication
    pub api_key: String,
    
    /// Organization ID (if applicable)
    pub organization_id: Option<String>,
    
    /// Default model to use
    pub model: String,
    
    /// Default system prompt
    pub system_prompt: Option<String>,
    
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Whether to reconnect automatically
    pub auto_reconnect: bool,
    
    /// Maximum reconnection attempts
    pub max_reconnect_attempts: u32,
    
    /// Reconnection backoff delay
    pub reconnect_backoff: Duration,
}

impl ProtocolConfig for McpConfig {
    fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("WebSocket URL cannot be empty".to_string());
        }
        
        if self.api_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        
        if self.model.is_empty() {
            return Err("Model ID cannot be empty".to_string());
        }
        
        Ok(())
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            url: "wss://api.anthropic.com/v1/messages".to_string(),
            api_key: String::new(),
            organization_id: None,
            model: "claude-3-opus-20240229".to_string(),
            system_prompt: None,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(120),
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_backoff: Duration::from_secs(2),
        }
    }
}

impl McpConfig {
    /// Create a new configuration with the given API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let mut config = Self::default();
        config.api_key = api_key.into();
        config
    }
    
    /// Set the WebSocket URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }
    
    /// Set the organization ID
    pub fn with_organization_id(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }
    
    /// Set the default model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
    
    /// Set the default system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
    
    /// Configure connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }
    
    /// Configure request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
    
    /// Configure auto-reconnect behavior
    pub fn with_auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
        self
    }
    
    /// Configure max reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }
    
    /// Configure reconnection backoff delay
    pub fn with_reconnect_backoff(mut self, backoff: Duration) -> Self {
        self.reconnect_backoff = backoff;
        self
    }
}
