mod client;
mod config;
mod error;
mod message;
mod protocol;
mod types;
mod websocket;

// Export the key components
pub use client::McpClient;
pub use config::McpConfig;
pub use error::McpError;
pub use message::{McpMessage, McpMessagePayload, McpResponseMessage};
pub use protocol::McpProtocolHandler;
pub use types::*;
pub use websocket::WebSocketClient;

use crate::protocols::{ProtocolFactory, ProtocolHandler};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// MCP Protocol Factory
pub struct McpProtocolFactory {
    config: McpConfig,
}

impl McpProtocolFactory {
    /// Create a new MCP protocol factory with default configuration
    pub fn new() -> Self {
        Self {
            config: McpConfig::default(),
        }
    }
    
    /// Create a new MCP protocol factory with custom configuration
    pub fn with_config(config: McpConfig) -> Self {
        Self { config }
    }
}

impl ProtocolFactory for McpProtocolFactory {
    fn create_handler(&self) -> Arc<dyn ProtocolHandler> {
        Arc::new(McpProtocolHandler::new(self.config.clone()))
    }
    
    fn protocol_name(&self) -> &'static str {
        "Model Context Protocol"
    }
    
    fn protocol_description(&self) -> &'static str {
        "A protocol for communicating with AI models using WebSockets"
    }
}

/// Implementation for McpProtocolFactory
impl Default for McpProtocolFactory {
    fn default() -> Self {
        Self::new()
    }
}
