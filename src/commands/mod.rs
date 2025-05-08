pub mod ai;
pub mod auth;
pub mod chat;
pub mod mcp;

use tauri::Wry;

/// Register all commands with Tauri
pub fn register_commands(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    builder
        .invoke_handler(tauri::generate_handler![
            // Authentication commands
            auth::set_api_key,
            auth::validate_api_key,
            auth::get_organization_id,
            auth::logout,
            
            // Chat commands
            chat::get_available_models,
            chat::create_conversation,
            chat::get_conversation,
            chat::get_conversations,
            chat::delete_conversation,
            chat::get_messages,
            chat::send_message,
            
            // MCP commands
            mcp::connect,
            mcp::disconnect,
            mcp::get_connection_status,
            
            // AI commands
            ai::get_available_models,
            ai::set_network_status,
            ai::send_message,
            ai::stream_message,
            ai::cancel_streaming,
            ai::get_messages,
            ai::create_conversation,
            ai::delete_conversation,
        ])
}
