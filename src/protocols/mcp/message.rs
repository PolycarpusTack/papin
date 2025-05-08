use crate::protocols::mcp::types::{McpCompletionRequest, McpErrorCode, McpMessageType};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Main MCP message structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpMessage {
    /// Unique message identifier
    pub id: String,
    
    /// Protocol version
    pub version: String,
    
    /// Message type
    #[serde(rename = "type")]
    pub type_: McpMessageType,
    
    /// Message payload (depends on type)
    #[serde(flatten)]
    pub payload: McpMessagePayload,
}

/// MCP message payload - varies based on message type
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpMessagePayload {
    /// Completion request payload
    CompletionRequest(McpCompletionRequest),
    
    /// Completion response payload
    #[serde(rename_all = "snake_case")]
    CompletionResponse {
        /// Completion response data
        response: Value,
    },
    
    /// Streaming message payload
    #[serde(rename_all = "snake_case")]
    StreamingMessage {
        /// Streaming ID associated with this message
        streaming_id: String,
        
        /// Chunk of text or data
        chunk: Value,
        
        /// Is this chunk final?
        is_final: bool,
    },
    
    /// Streaming end notification
    #[serde(rename_all = "snake_case")]
    StreamingEnd {
        /// Streaming ID that has ended
        streaming_id: String,
    },
    
    /// Cancel streaming request
    #[serde(rename_all = "snake_case")]
    CancelStream {
        /// Streaming ID to cancel
        streaming_id: String,
    },
    
    /// Error message payload
    #[serde(rename_all = "snake_case")]
    Error {
        /// Request ID that caused the error
        request_id: String,
        
        /// Error code
        code: McpErrorCode,
        
        /// Human-readable error message
        message: String,
        
        /// Additional error details (optional)
        details: Option<Value>,
    },
    
    /// Ping message (heartbeat)
    Ping {},
    
    /// Pong response to ping
    Pong {},
    
    /// Authentication request
    #[serde(rename_all = "snake_case")]
    AuthRequest {
        /// API key
        api_key: String,
        
        /// Organization ID (optional)
        organization_id: Option<String>,
    },
    
    /// Authentication response
    #[serde(rename_all = "snake_case")]
    AuthResponse {
        /// Authentication successful?
        success: bool,
        
        /// Session ID
        session_id: Option<String>,
    },
}

/// Response message types we can receive
#[derive(Debug)]
pub enum McpResponseMessage {
    /// Completion response
    Completion(McpMessage),
    
    /// Streaming message
    Streaming(McpMessage),
    
    /// Authentication response
    Auth(McpMessage),
    
    /// Error response
    Error(McpMessage),
}
