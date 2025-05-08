pub mod messages;

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Represents a conversation with a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique conversation identifier
    pub id: String,
    
    /// User-friendly title
    pub title: String,
    
    /// When the conversation was created
    pub created_at: SystemTime,
    
    /// When the conversation was last modified
    pub updated_at: SystemTime,
    
    /// Model used for this conversation
    pub model: Model,
    
    /// Conversation metadata
    pub metadata: serde_json::Value,
}

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

/// Implementation for Conversation
impl Conversation {
    /// Create a new conversation
    pub fn new(title: impl Into<String>, model: Model) -> Self {
        let now = SystemTime::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            model,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Set conversation title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = SystemTime::now();
    }
    
    /// Calculate conversation age
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0))
    }
}

/// Implementation for Model
impl Model {
    /// Create a new Claude model
    pub fn claude(variant: &str, version: &str) -> Self {
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
            id: format!("claude-3-{}-{}", variant, version),
            provider: "anthropic".to_string(),
            name: format!("Claude 3 {}", variant.to_string()),
            version: version.to_string(),
            capabilities,
        }
    }
}
