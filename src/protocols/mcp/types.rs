use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpMessageType {
    /// Completion request
    CompletionRequest,
    
    /// Completion response
    CompletionResponse,
    
    /// Streaming message
    StreamingMessage,
    
    /// Streaming end notification
    StreamingEnd,
    
    /// Cancel streaming request
    CancelStream,
    
    /// Error message
    Error,
    
    /// Ping message (heartbeat)
    Ping,
    
    /// Pong response to ping
    Pong,
    
    /// Authentication request
    AuthRequest,
    
    /// Authentication response
    AuthResponse,
}

/// MCP message roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpMessageRole {
    /// User message
    User,
    
    /// Assistant (model) message
    Assistant,
    
    /// System message
    System,
    
    /// Tool message
    Tool,
}

impl ToString for McpMessageRole {
    fn to_string(&self) -> String {
        match self {
            McpMessageRole::User => "user".to_string(),
            McpMessageRole::Assistant => "assistant".to_string(),
            McpMessageRole::System => "system".to_string(),
            McpMessageRole::Tool => "tool".to_string(),
        }
    }
}

/// MCP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpErrorCode {
    /// Invalid request format
    InvalidRequest,
    
    /// Authentication failed
    AuthenticationFailed,
    
    /// Authorization failed
    AuthorizationFailed,
    
    /// Rate limit exceeded
    RateLimitExceeded,
    
    /// Model overloaded
    ModelOverloaded,
    
    /// Context length exceeded
    ContextLengthExceeded,
    
    /// Content filtered
    ContentFiltered,
    
    /// Invalid parameters
    InvalidParameters,
    
    /// Server error
    ServerError,
    
    /// Unknown error
    Unknown,
}

/// MCP completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCompletionRequest {
    /// Model to use
    pub model: String,
    
    /// Messages in the conversation
    pub messages: Vec<Value>,
    
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_tokens: Option<u32>,
    
    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Stop sequences
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    
    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    
    /// Streaming ID for tracking streaming responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_id: Option<String>,
}
