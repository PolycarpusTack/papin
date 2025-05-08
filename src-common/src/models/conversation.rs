use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use super::model::Model;
use super::message::Message;

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
    
    /// Messages in this conversation
    #[serde(default)]
    pub messages: Vec<Message>,
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
            messages: Vec::new(),
        }
    }
    
    /// Set conversation title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = SystemTime::now();
    }
    
    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = SystemTime::now();
    }
    
    /// Calculate conversation age
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0))
    }
    
    /// Get a summary of the conversation
    pub fn summary(&self) -> String {
        let msg_count = self.messages.len();
        let last_update = chrono::DateTime::<chrono::Local>::from(self.updated_at)
            .format("%Y-%m-%d %H:%M")
            .to_string();
            
        format!(
            "{} ({}, {} messages, updated {})",
            self.title,
            self.model.name,
            msg_count,
            last_update
        )
    }
}
