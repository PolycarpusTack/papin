use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// MCP service error
    #[error("Service error: {0}")]
    Service(String),
    
    /// Application logic error
    #[error("Application error: {0}")]
    App(String),
}

/// Convert MCP error to AppError
impl From<mcp_common::error::McpError> for AppError {
    fn from(error: mcp_common::error::McpError) -> Self {
        AppError::Service(error.to_string())
    }
}
