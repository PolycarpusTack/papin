pub mod ai;
pub mod auth;
pub mod chat;
pub mod collaboration;
pub mod mcp;
pub mod offline;
pub mod security;

use tauri::Wry;

/// Register all commands with Tauri
pub fn register_commands(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    // Register each command module
    let builder = builder
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
        ]);
    
    // Register offline commands
    let builder = offline::register_offline_commands(builder);
    
    // Register security commands
    let builder = builder
        .invoke_handler(tauri::generate_handler![
            // Security commands
            security::init_security,
            security::get_security_config,
            security::update_security_config,
            
            // Credentials commands
            security::store_secure_credential,
            security::get_secure_credential,
            security::delete_secure_credential,
            security::list_secure_credentials,
            
            // E2EE commands
            security::encrypt_data,
            security::decrypt_data,
            security::rotate_encryption_keys,
            
            // Permission commands
            security::check_permission_granted,
            security::request_app_permission,
            security::get_all_permissions,
            security::set_permission_level,
            security::reset_permission,
            security::reset_all_permissions,
            security::get_permission_statistics,
            
            // Data flow commands
            security::get_data_flow_graph,
            security::get_recent_data_flow_events,
            security::track_data_flow,
            security::clear_data_flow_events,
            security::get_data_flow_statistics,
            security::search_data_flow_events,
        ]);
    
    builder
}
