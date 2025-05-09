pub mod ai;
pub mod auth;
pub mod chat;
pub mod collaboration;
pub mod mcp;
pub mod offline;
pub mod security;
pub mod llm_metrics;

// Offline commands
pub mod offline {
    pub mod llm;
    
    // Re-export the existing module
    pub use super::offline::*;
}

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
    
    // Register offline LLM provider commands
    let builder = builder
        .invoke_handler(tauri::generate_handler![
            // Provider management
            offline::llm::get_all_providers,
            offline::llm::get_all_provider_availability,
            offline::llm::check_provider_availability,
            offline::llm::add_custom_provider,
            offline::llm::remove_custom_provider,
            offline::llm::get_active_provider,
            offline::llm::set_active_provider,
            offline::llm::get_provider_config,
            offline::llm::update_provider_config,
            
            // Model management
            offline::llm::list_available_models,
            offline::llm::list_downloaded_models,
            offline::llm::get_download_status,
            offline::llm::download_model,
            offline::llm::cancel_download,
            offline::llm::delete_model,
            
            // Text generation
            offline::llm::generate_text,
            
            // Provider discovery
            offline::llm::scan_for_providers,
            offline::llm::get_discovery_status,
            offline::llm::get_provider_suggestions,
            offline::llm::get_discovery_config,
            offline::llm::update_discovery_config,
            offline::llm::auto_configure_providers,
            
            // Provider migration
            offline::llm::check_legacy_system,
            offline::llm::get_migration_status,
            offline::llm::run_migration,
            offline::llm::get_migration_config,
            offline::llm::update_migration_config,
            offline::llm::opt_out_of_migration,
            offline::llm::get_model_mappings,
            offline::llm::get_provider_mappings,
        ]);
    
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
        
    // Register LLM metrics commands
    let builder = builder
        .invoke_handler(tauri::generate_handler![
            // LLM metrics commands
            llm_metrics::get_llm_provider_metrics,
            llm_metrics::get_llm_model_metrics,
            llm_metrics::get_active_llm_provider,
            llm_metrics::get_default_llm_model,
            llm_metrics::get_llm_metrics_enabled,
            llm_metrics::get_llm_metrics_config,
            llm_metrics::update_llm_metrics_config,
            llm_metrics::accept_llm_metrics_privacy_notice,
            llm_metrics::reset_llm_metrics,
        ]);
    
    builder
}
