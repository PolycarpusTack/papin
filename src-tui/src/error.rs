use std::io;
use mcp_common::error::McpError;
use thiserror::Error;

/// TUI application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("MCP error: {0}")]
    McpError(#[from] McpError),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Application error: {0}")]
    AppError(String),
}

/// Result type for the TUI application
pub type AppResult<T> = Result<T, AppError>;
