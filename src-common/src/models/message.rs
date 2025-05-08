use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

use super::tool::ToolCall;

/// Message role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Message content type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ContentType {
    Text { text: String },
    Image { url: String, alt_text: Option<String> },
    ToolCalls { calls: Vec<ToolCall> },
    ToolResults { results: Vec<serde_json::Value> },
}

/// Message content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageContent {
    pub parts: Vec<ContentType>,
}

/// Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: String,
    
    /// Role (user, assistant, system)
    pub role: MessageRole,
    
    /// Content of the message
    pub content: MessageContent,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    
    /// When the message was created
    pub created_at: SystemTime,
}

/// Message error types
#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Message sending timed out after {0:?}")]
    Timeout(Duration),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Rate limited: {0}")]
    RateLimit(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Message {
    /// Create a new text message from a user
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: MessageContent {
                parts: vec![ContentType::Text { text: text.into() }],
            },
            metadata: None,
            created_at: SystemTime::now(),
        }
    }
    
    /// Create a new text message from the assistant
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: MessageContent {
                parts: vec![ContentType::Text { text: text.into() }],
            },
            metadata: None,
            created_at: SystemTime::now(),
        }
    }
    
    /// Create a new system message
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::System,
            content: MessageContent {
                parts: vec![ContentType::Text { text: text.into() }],
            },
            metadata: None,
            created_at: SystemTime::now(),
        }
    }
    
    /// Get the text content of the message
    pub fn text(&self) -> String {
        let mut result = String::new();
        
        for part in &self.content.parts {
            if let ContentType::Text { text } = part {
                result.push_str(text);
            }
        }
        
        result
    }
    
    /// Get a formatted timestamp for the message
    pub fn timestamp(&self) -> String {
        chrono::DateTime::<chrono::Local>::from(self.created_at)
            .format("%H:%M:%S")
            .to_string()
    }
    
    /// Check if this message has tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.content.parts.iter().any(|part| {
            if let ContentType::ToolCalls { .. } = part {
                true
            } else {
                false
            }
        })
    }
}
