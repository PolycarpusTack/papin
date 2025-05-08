use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Error type for message-related operations
#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Message timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Message role (user, assistant, system)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Message content type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum ContentType {
    #[serde(rename = "text")]
    Text { text: String },
    
    #[serde(rename = "image")]
    Image { url: String, media_type: String },
    
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_call_id: String,
        result: String,
    },
}

/// Message content that can contain multiple parts (text, images, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageContent {
    pub parts: Vec<ContentType>,
}

/// Base message structure used for MCP communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: String,
    
    /// Message role (user, assistant, etc.)
    pub role: MessageRole,
    
    /// Message content (can be multipart)
    pub content: MessageContent,
    
    /// Optional metadata key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    
    /// Message creation timestamp
    #[serde(with = "time_serde")]
    pub created_at: SystemTime,
}

/// Conversation message for tracking conversation history
#[derive(Debug, Clone)]
pub struct ConversationMessage {
    /// The underlying message
    pub message: Message,
    
    /// References to any parent messages (for threading)
    pub parent_ids: Vec<String>,
    
    /// If message was streamed, time when streaming completed
    pub completed_at: Option<SystemTime>,
    
    /// If message is being streamed, partial content to display
    pub partial_content: Option<String>,
    
    /// Message status
    pub status: MessageStatus,
}

/// Status of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    /// Message has been sent and we're waiting for response
    Sending,
    
    /// Message is currently being streamed (receiving)
    Streaming,
    
    /// Message has been sent and received completely
    Complete,
    
    /// Message sending or receiving failed
    Failed,
    
    /// Message is canceled
    Cancelled,
}

/// Implementation for Message
impl Message {
    /// Create a new user message with text content
    pub fn new_user_text(text: impl Into<String>) -> Self {
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
    
    /// Create a new system message with text content
    pub fn new_system_text(text: impl Into<String>) -> Self {
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
    
    /// Add metadata to a message
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.into(), value.into());
        self
    }
    
    /// Get text content if message contains only text
    pub fn text_content(&self) -> Option<&str> {
        if self.content.parts.len() == 1 {
            match &self.content.parts[0] {
                ContentType::Text { text } => Some(text),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Time serialization helpers for serde
mod time_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = time
            .duration_since(UNIX_EPOCH)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?
            .as_secs();
        serializer.serialize_u64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(timestamp))
    }
}
