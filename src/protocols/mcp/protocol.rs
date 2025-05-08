use crate::models::messages::{Message, MessageError};
use crate::protocols::mcp::{McpClient, McpConfig};
use crate::protocols::{ConnectionStatus, ProtocolHandler};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::sync::{Arc, RwLock};

/// MCP Protocol Handler
pub struct McpProtocolHandler {
    /// MCP client
    client: Arc<McpClient>,
    
    /// Current connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Protocol configuration
    config: McpConfig,
}

impl McpProtocolHandler {
    /// Create a new MCP protocol handler
    pub fn new(config: McpConfig) -> Self {
        let status = Arc::new(RwLock::new(ConnectionStatus::Disconnected));
        let client = Arc::new(McpClient::new(config.clone()));
        
        Self {
            client,
            status,
            config,
        }
    }
}

#[async_trait]
impl ProtocolHandler for McpProtocolHandler {
    fn protocol_name(&self) -> &'static str {
        "Model Context Protocol"
    }
    
    fn connection_status(&self) -> ConnectionStatus {
        self.client.status()
    }
    
    async fn connect(&self) -> Result<(), String> {
        match self.client.connect().await {
            Ok(_) => {
                let mut status = self.status.write().unwrap();
                *status = ConnectionStatus::Connected;
                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().unwrap();
                *status = ConnectionStatus::ConnectionError(e.to_string());
                Err(e.to_string())
            }
        }
    }
    
    async fn disconnect(&self) -> Result<(), String> {
        match self.client.disconnect().await {
            Ok(_) => {
                let mut status = self.status.write().unwrap();
                *status = ConnectionStatus::Disconnected;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
    
    async fn send_message(&self, message: Message) -> Result<(), MessageError> {
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        self.client.send(message).await?;
        Ok(())
    }
    
    async fn receive_messages(&self) -> Result<Vec<Message>, MessageError> {
        // This implementation would normally use some kind of subscription or poll
        // mechanism to receive messages from the MCP server.
        // For now, we'll return an empty vec since our implementation primarily
        // uses the send method which handles responses directly.
        
        if !self.is_connected() {
            return Err(MessageError::ConnectionClosed);
        }
        
        // Placeholder - real implementation would check for any received messages
        Ok(Vec::new())
    }
}
