pub mod claude;
pub mod local;
pub mod router;

use crate::models::messages::{Message, MessageError};
use crate::models::Model;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// AI provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    /// Claude AI (Anthropic)
    Claude,
    
    /// Local model
    Local,
    
    /// Custom provider
    Custom,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProviderType::Claude => write!(f, "Claude"),
            ProviderType::Local => write!(f, "Local"),
            ProviderType::Custom => write!(f, "Custom"),
        }
    }
}

/// AI model status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model is available
    Available,
    
    /// Model is being loaded
    Loading,
    
    /// Model is not available
    Unavailable,
    
    /// Error occurred while loading or using the model
    Error(ModelError),
}

/// Model error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelError {
    /// Network error
    NetworkError,
    
    /// Authentication error
    AuthError,
    
    /// Rate limit error
    RateLimitError,
    
    /// Model overloaded
    ModelOverloaded,
    
    /// Context length exceeded
    ContextLengthExceeded,
    
    /// Content filtered
    ContentFiltered,
    
    /// Invalid request
    InvalidRequest,
    
    /// System error
    SystemError,
    
    /// Not implemented
    NotImplemented,
    
    /// Unknown error
    Unknown,
}

/// Model provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderConfig {
    /// Provider type
    pub provider_type: ProviderType,
    
    /// Provider name
    pub name: String,
    
    /// API base URL
    pub base_url: String,
    
    /// API key
    pub api_key: String,
    
    /// Organization ID
    pub organization_id: Option<String>,
    
    /// Request timeout
    pub timeout: Duration,
    
    /// Default model
    pub default_model: String,
    
    /// Fallback model
    pub fallback_model: Option<String>,
    
    /// Enable MCP protocol
    pub enable_mcp: bool,
    
    /// Enable streaming
    pub enable_streaming: bool,
    
    /// Additional settings
    pub settings: serde_json::Map<String, serde_json::Value>,
}

impl Default for ModelProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Claude,
            name: "Claude".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            api_key: String::new(),
            organization_id: None,
            timeout: Duration::from_secs(120),
            default_model: "claude-3-opus-20240229".to_string(),
            fallback_model: Some("claude-3-haiku-20240307".to_string()),
            enable_mcp: true,
            enable_streaming: true,
            settings: serde_json::Map::new(),
        }
    }
}

/// Model callback for streaming responses
pub type ModelCallback = Box<dyn Fn(Message) -> () + Send + Sync>;

/// Base trait for AI model providers
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Get provider type
    fn provider_type(&self) -> ProviderType;
    
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Get provider configuration
    fn config(&self) -> &ModelProviderConfig;
    
    /// Get available models
    async fn available_models(&self) -> Result<Vec<Model>, ModelError>;
    
    /// Check if model is available
    async fn is_available(&self, model_id: &str) -> bool;
    
    /// Get model status
    async fn model_status(&self, model_id: &str) -> ModelStatus;
    
    /// Complete a message (synchronous)
    async fn complete(&self, model_id: &str, message: Message) -> Result<Message, MessageError>;
    
    /// Stream a message (asynchronous)
    async fn stream(&self, model_id: &str, message: Message) 
        -> Result<mpsc::Receiver<Result<Message, MessageError>>, MessageError>;
    
    /// Cancel a streaming message
    async fn cancel_stream(&self, stream_id: &str) -> Result<(), MessageError>;
    
    /// Check if provider supports a feature
    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "streaming" => self.config().enable_streaming,
            "mcp" => self.config().enable_mcp,
            _ => false,
        }
    }
}

/// Get all available model providers
pub fn get_all_providers() -> Vec<Arc<dyn ModelProvider>> {
    let mut providers = Vec::new();
    
    // Claude provider
    if let Ok(claude_provider) = claude::ClaudeProvider::new() {
        providers.push(Arc::new(claude_provider) as Arc<dyn ModelProvider>);
    }
    
    // Local provider
    if let Ok(local_provider) = local::LocalProvider::new() {
        providers.push(Arc::new(local_provider) as Arc<dyn ModelProvider>);
    }
    
    providers
}
