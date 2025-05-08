use serde::{Deserialize, Serialize};

/// Information about a model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    /// Model identifier (e.g., "claude-3-opus-20240229")
    pub id: String,
    
    /// Provider name (e.g., "anthropic")
    pub provider: String,
    
    /// User-friendly name (e.g., "Claude 3 Opus")
    pub name: String,
    
    /// Model version
    pub version: String,
    
    /// Model capabilities
    pub capabilities: ModelCapabilities,
}

/// Model capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Can process images
    pub vision: bool,
    
    /// Maximum context length
    pub max_context_length: usize,
    
    /// Supports functions/tools
    pub functions: bool,
    
    /// Supports streamed responses
    pub streaming: bool,
}

/// Implementation for Model
impl Model {
    /// Create a new Claude model
    pub fn claude(variant: &str, version: &str) -> Self {
        let (name, display_name) = match variant {
            "opus" => ("claude-3-opus", "Claude 3 Opus"),
            "sonnet" => ("claude-3-sonnet", "Claude 3 Sonnet"),
            "haiku" => ("claude-3-haiku", "Claude 3 Haiku"),
            _ => (format!("claude-3-{}", variant).as_str(), format!("Claude 3 {}", variant.to_string()).as_str()),
        };
        
        let capabilities = match variant {
            "opus" => ModelCapabilities {
                vision: true,
                max_context_length: 200_000,
                functions: true,
                streaming: true,
            },
            "sonnet" => ModelCapabilities {
                vision: true,
                max_context_length: 180_000,
                functions: true,
                streaming: true,
            },
            "haiku" => ModelCapabilities {
                vision: true,
                max_context_length: 150_000,
                functions: true,
                streaming: true,
            },
            _ => ModelCapabilities {
                vision: false,
                max_context_length: 100_000,
                functions: false,
                streaming: true,
            },
        };
        
        Self {
            id: format!("{}-{}", name, version),
            provider: "anthropic".to_string(),
            name: display_name.to_string(),
            version: version.to_string(),
            capabilities,
        }
    }
    
    /// Get all available Claude models
    pub fn available_claude_models() -> Vec<Self> {
        vec![
            Self::claude("opus", "20240229"),
            Self::claude("sonnet", "20240229"),
            Self::claude("haiku", "20240307"),
        ]
    }
    
    /// Get default Claude model
    pub fn default_claude() -> Self {
        Self::claude("sonnet", "20240229")
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::default_claude()
    }
}
