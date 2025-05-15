pub mod config;
pub mod error;
pub mod models;
pub mod observability;
pub mod platform;
pub mod protocol;
pub mod service;
pub mod utils;

use once_cell::sync::OnceCell;
use std::sync::Arc;

use crate::service::mcp::McpService;

/// Global MCP service instance
static MCP_SERVICE: OnceCell<Arc<McpService>> = OnceCell::new();

/// Initialize the MCP service
pub fn init_mcp_service() -> Arc<McpService> {
    // Create a shared MCP service instance
    let service = Arc::new(McpService::new());
    
    // Store in global cell if not already set
    if let Err(_) = MCP_SERVICE.set(service.clone()) {
        // Already initialized, just return the new instance
    }
    
    service
}

/// Get the global MCP service instance
pub fn get_mcp_service() -> Arc<McpService> {
    MCP_SERVICE.get_or_init(|| Arc::new(McpService::new())).clone()
}
