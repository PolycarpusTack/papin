use thiserror::Error;

/// MCP client error types
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Message error: {0}")]
    Message(#[from] crate::models::MessageError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Rate limited: {0}")]
    RateLimit(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;
