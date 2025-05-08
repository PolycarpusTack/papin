use crate::protocols::ConnectionStatus;
use crate::services::mcp::get_mcp_service;

/// Connect to MCP server
#[tauri::command]
pub async fn connect() -> Result<(), String> {
    get_mcp_service().connect().await
}

/// Disconnect from MCP server
#[tauri::command]
pub async fn disconnect() -> Result<(), String> {
    get_mcp_service().disconnect().await
}

/// Get connection status
#[tauri::command]
pub fn get_connection_status() -> String {
    match get_mcp_service().connection_status() {
        ConnectionStatus::Disconnected => "disconnected".to_string(),
        ConnectionStatus::Connecting => "connecting".to_string(),
        ConnectionStatus::Connected => "connected".to_string(),
        ConnectionStatus::AuthFailed => "auth_failed".to_string(),
        ConnectionStatus::ConnectionError(e) => format!("error: {}", e),
    }
}
