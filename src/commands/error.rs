use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum CommandError {
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error("Input validation error: {0}")]
    ValidationError(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

// Implementation for converting common error types
impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::InternalError(format!("IO error: {}", err))
    }
}

// Assuming MessageError is defined elsewhere in your project
impl From<crate::models::messages::MessageError> for CommandError {
    fn from(err: crate::models::messages::MessageError) -> Self {
        match err {
            // Map specific error types appropriately - adjust these to match your actual error types
            crate::models::messages::MessageError::ConnectionClosed => {
                CommandError::ConnectionError("Connection closed".to_string())
            },
            crate::models::messages::MessageError::AuthError(msg) => {
                CommandError::AuthError(msg)
            },
            // Add other specific mappings as needed
            _ => CommandError::ServiceError(err.to_string()),
        }
    }
}