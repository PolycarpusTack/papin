use crate::ai::{ModelError, ModelProviderConfig};
use crate::models::messages::{Message, MessageError};
use crate::protocols::mcp::{McpClient, McpConfig, McpError};
use crate::utils::events::{events, get_event_system};
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Claude MCP client
#[derive(Clone)]
pub struct ClaudeMcpClient {
    /// MCP client
    client: Arc<McpClient>,
    
    /// Provider configuration
    config: ModelProviderConfig,
}

impl ClaudeMcpClient {
    /// Create a new Claude MCP client
    pub fn new(provider_config: &ModelProviderConfig) -> Result<Self, ModelError> {
        // Create MCP configuration
        let mcp_config = McpConfig::with_api_key(provider_config.api_key.clone())
            .with_url("wss://api.anthropic.com/v1/messages")
            .with_model(provider_config.default_model.clone());
        
        // Create MCP client
        let client = match McpClient::new(mcp_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to create MCP client: {:?}", e);
                return Err(ModelError::SystemError);
            }
        };
        
        Ok(Self {
            client: Arc::new(client),
            config: provider_config.clone(),
        })
    }
    
    /// Connect to MCP server
    pub async fn connect(&self) -> Result<(), ModelError> {
        match self.client.connect().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to connect to MCP server: {}", e);
                Err(ModelError::NetworkError)
            }
        }
    }
    
    /// Disconnect from MCP server
    pub async fn disconnect(&self) -> Result<(), ModelError> {
        match self.client.disconnect().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to disconnect from MCP server: {}", e);
                Err(ModelError::SystemError)
            }
        }
    }
    
    /// Complete a message
    pub async fn complete(&self, model_id: &str, message: Message) -> Result<Message, MessageError> {
        // Check if connected
        if !self.client.is_connected() {
            // Try to connect
            match self.connect().await {
                Ok(_) => {}
                Err(e) => {
                    return Err(MessageError::NetworkError(format!(
                        "Failed to connect to MCP server: {:?}",
                        e
                    )));
                }
            }
        }
        
        // Send message with model ID in metadata
        let mut message_with_metadata = message;
        
        if message_with_metadata.metadata.is_none() {
            message_with_metadata.metadata = Some(std::collections::HashMap::new());
        }
        
        if let Some(metadata) = &mut message_with_metadata.metadata {
            metadata.insert(
                "model".to_string(),
                serde_json::to_value(model_id).unwrap(),
            );
        }
        
        // Send through MCP client
        self.client.send(message_with_metadata).await
    }
    
    /// Stream a message
    pub async fn stream(
        &self,
        model_id: &str,
        message: Message,
    ) -> Result<mpsc::Receiver<Result<Message, MessageError>>, MessageError> {
        // Check if connected
        if !self.client.is_connected() {
            // Try to connect
            match self.connect().await {
                Ok(_) => {}
                Err(e) => {
                    return Err(MessageError::NetworkError(format!(
                        "Failed to connect to MCP server: {:?}",
                        e
                    )));
                }
            }
        }
        
        // Send message with model ID in metadata
        let mut message_with_metadata = message;
        
        if message_with_metadata.metadata.is_none() {
            message_with_metadata.metadata = Some(std::collections::HashMap::new());
        }
        
        if let Some(metadata) = &mut message_with_metadata.metadata {
            metadata.insert(
                "model".to_string(),
                serde_json::to_value(model_id).unwrap(),
            );
        }
        
        // Stream through MCP client
        self.client.stream(message_with_metadata).await
    }
}
