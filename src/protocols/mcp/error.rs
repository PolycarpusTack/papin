use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// MCP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    
    /// Connection error
    ConnectionError,
    
    /// Connection closed
    ConnectionClosed,
    
    /// Timeout
    Timeout,
    
    /// Unknown error
    Unknown,
}

impl fmt::Display for McpErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpErrorCode::InvalidRequest => write!(f, "invalid_request"),
            McpErrorCode::AuthenticationFailed => write!(f, "authentication_failed"),
            McpErrorCode::AuthorizationFailed => write!(f, "authorization_failed"),
            McpErrorCode::RateLimitExceeded => write!(f, "rate_limit_exceeded"),
            McpErrorCode::ModelOverloaded => write!(f, "model_overloaded"),
            McpErrorCode::ContextLengthExceeded => write!(f, "context_length_exceeded"),
            McpErrorCode::ContentFiltered => write!(f, "content_filtered"),
            McpErrorCode::InvalidParameters => write!(f, "invalid_parameters"),
            McpErrorCode::ServerError => write!(f, "server_error"),
            McpErrorCode::ConnectionError => write!(f, "connection_error"),
            McpErrorCode::ConnectionClosed => write!(f, "connection_closed"),
            McpErrorCode::Timeout => write!(f, "timeout"),
            McpErrorCode::Unknown => write!(f, "unknown"),
        }
    }
}

/// MCP error type
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Model overloaded: {0}")]
    ModelOverloaded(String),
    
    #[error("Context length exceeded: {0}")]
    ContextLengthExceeded(String),
    
    #[error("Content filtered: {0}")]
    ContentFiltered(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Request timed out after {0:?}")]
    Timeout(Duration),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::SerializationError(err.to_string())
    }
}

impl From<McpErrorCode> for McpError {
    fn from(code: McpErrorCode) -> Self {
        match code {
            McpErrorCode::InvalidRequest => McpError::InvalidRequest("Invalid request".to_string()),
            McpErrorCode::AuthenticationFailed => {
                McpError::AuthenticationFailed("Authentication failed".to_string())
            }
            McpErrorCode::AuthorizationFailed => {
                McpError::AuthorizationFailed("Authorization failed".to_string())
            }
            McpErrorCode::RateLimitExceeded => {
                McpError::RateLimitExceeded("Rate limit exceeded".to_string())
            }
            McpErrorCode::ModelOverloaded => {
                McpError::ModelOverloaded("Model is currently overloaded".to_string())
            }
            McpErrorCode::ContextLengthExceeded => {
                McpError::ContextLengthExceeded("Context length exceeded".to_string())
            }
            McpErrorCode::ContentFiltered => {
                McpError::ContentFiltered("Content was filtered".to_string())
            }
            McpErrorCode::InvalidParameters => {
                McpError::InvalidParameters("Invalid parameters".to_string())
            }
            McpErrorCode::ServerError => McpError::ServerError("Server error".to_string()),
            McpErrorCode::ConnectionError => {
                McpError::ConnectionError("Connection error".to_string())
            }
            McpErrorCode::ConnectionClosed => McpError::ConnectionClosed,
            McpErrorCode::Timeout => McpError::Timeout(Duration::from_secs(60)),
            McpErrorCode::Unknown => McpError::Unknown("Unknown error".to_string()),
        }
    }
}
