use serde::Serialize;
use thiserror::Error;
use crate::models::messages::MessageError;

/// Structured error types for command handlers
#[derive(Debug, Error, Serialize)]
pub enum CommandError {
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error("Input validation error: {0}")]
    ValidationError(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// Allow conversion from service-specific errors to command errors
impl From<MessageError> for CommandError {
    fn from(err: MessageError) -> Self {
        match err {
            MessageError::NotFound(msg) => CommandError::NotFound(msg),
            MessageError::ValidationError(msg) => CommandError::ValidationError(msg),
            MessageError::AuthError(msg) => CommandError::AuthError(msg),
            MessageError::ConfigurationError(msg) => CommandError::ConfigError(msg),
            MessageError::NetworkError(msg) => CommandError::NetworkError(msg),
            MessageError::PermissionDenied(msg) => CommandError::PermissionDenied(msg),
            MessageError::ConnectionClosed => CommandError::NetworkError("Connection closed".to_string()),
            MessageError::TimeoutError(msg) => CommandError::ServiceError(format!("Timeout: {}", msg)),
            MessageError::ProviderError(msg) => CommandError::ServiceError(format!("Provider error: {}", msg)),
            // Generic fallback for other errors
            _ => CommandError::ServiceError(err.to_string()),
        }
    }
}

// Add a convenience method to create a validation error
impl CommandError {
    pub fn validation(message: impl Into<String>) -> Self {
        CommandError::ValidationError(message.into())
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        CommandError::NotFound(message.into())
    }
    
    pub fn internal(message: impl Into<String>) -> Self {
        CommandError::InternalError(message.into())
    }
    
    pub fn auth(message: impl Into<String>) -> Self {
        CommandError::AuthError(message.into())
    }
    
    pub fn config(message: impl Into<String>) -> Self {
        CommandError::ConfigError(message.into())
    }
    
    pub fn network(message: impl Into<String>) -> Self {
        CommandError::NetworkError(message.into())
    }
    
    pub fn permission(message: impl Into<String>) -> Self {
        CommandError::PermissionDenied(message.into())
    }
}
