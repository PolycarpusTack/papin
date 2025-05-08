pub mod mcp;

use crate::models::messages::{Message, MessageError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Protocol status information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected to any server
    Disconnected,
    
    /// Currently establishing connection
    Connecting,
    
    /// Connected and authenticated
    Connected,
    
    /// Connection established but authentication failed
    AuthFailed,
    
    /// Connection dropped or experiencing issues
    ConnectionError(String),
}

/// Base trait for all protocol handlers
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Returns the protocol name
    fn protocol_name(&self) -> &'static str;
    
    /// Returns the current connection status
    fn connection_status(&self) -> ConnectionStatus;
    
    /// Establishes connection to the server
    async fn connect(&self) -> Result<(), String>;
    
    /// Disconnects from the server
    async fn disconnect(&self) -> Result<(), String>;
    
    /// Sends a message to the server
    async fn send_message(&self, message: Message) -> Result<(), MessageError>;
    
    /// Receives messages from the server
    async fn receive_messages(&self) -> Result<Vec<Message>, MessageError>;
    
    /// Checks if the handler is connected
    fn is_connected(&self) -> bool {
        matches!(self.connection_status(), ConnectionStatus::Connected)
    }
}

/// Factory for creating protocol handlers
pub trait ProtocolFactory: Send + Sync {
    /// Creates a new protocol handler instance
    fn create_handler(&self) -> Arc<dyn ProtocolHandler>;
    
    /// Returns the protocol name
    fn protocol_name(&self) -> &'static str;
    
    /// Returns the protocol description
    fn protocol_description(&self) -> &'static str;
}

/// Protocol configuration base trait
pub trait ProtocolConfig: Send + Sync {
    /// Validates the configuration
    fn validate(&self) -> Result<(), String>;
}
