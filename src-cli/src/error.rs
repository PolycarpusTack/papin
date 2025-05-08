use mcp_common::error::McpError;
use thiserror::Error;
use anyhow::Result;

/// CLI error type
#[derive(Error, Debug)]
pub enum CliError {
    #[error("MCP error: {0}")]
    McpError(#[from] McpError),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Input error: {0}")]
    InputError(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// Convert any error to a CliError
pub fn to_cli_error<E: std::error::Error>(e: E) -> CliError {
    CliError::Unknown(e.to_string())
}
