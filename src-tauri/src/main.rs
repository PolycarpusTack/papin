// src-tauri/src/main.rs
//
// Main entry point for the Tauri application

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod notification;
mod system;

use log::{debug, info, warn, error};
use tauri::{Manager, RunEvent, WindowEvent};

fn main() {
    // Initialize logging early to capture any startup errors
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("Starting Papin application");
    
    // Build Tauri application with proper error handling
    let invoke_handler = tauri::generate_handler![
        // Register commands
        commands::offline::get_offline_config,
        commands::offline::update_offline_config,
        commands::offline::get_offline_status,
        commands::offline::go_offline,
        commands::offline::go_online,
        commands::offline::scan_for_llm_providers,
        commands::offline::get_offline_providers,
        commands::offline::get_provider_suggestions,
        
        // LLM Provider commands
        commands::offline::llm::get_all_providers,
        commands::offline::llm::get_all_provider_availability,
        commands::offline::llm::check_provider_availability,
        commands::offline::llm::get_active_provider,
        commands::offline::llm::set_active_provider,
        commands::offline::llm::get_provider_config,
        commands::offline::llm::update_provider_config,
        
        // Model Registry commands
        commands::offline::model_registry::get_all_models,
        commands::offline::model_registry::get_model,
        commands::offline::model_registry::get_installed_models,
        commands::offline::model_registry::get_compatible_models,
        commands::offline::model_registry::update_model_metadata,
        commands::offline::model_registry::download_model,
        commands::offline::model_registry::get_download_status,
        commands::offline::model_registry::cancel_download,
        commands::offline::model_registry::delete_model,
        commands::offline::model_registry::import_model,
        commands::offline::model_registry::export_model,
        commands::offline::model_registry::get_disk_usage,
        commands::offline::model_registry::set_disk_space_limit,
        commands::offline::model_registry::register_model_registry_events,
        
        notification::send_notification,
        system::platform::get_platform_info,
        system::platform::get_platform_name,
        system::platform::show_platform_notification,
        // Performance monitoring commands
        commands::performance::get_current_resource_metrics,
        commands::performance::get_historic_resource_metrics,
        commands::performance::get_hardware_capabilities,
        commands::performance::get_resource_recommendations,
        commands::performance::is_feature_supported,
        commands::performance::get_thread_pool_size,
        commands::performance::get_memory_settings,
    ];
    
    let result = tauri::Builder::default()
        .setup(|app| {
            // Initialize platform-specific features
            if let Err(e) = system::initialize(&app.app_handle()) {
                error!("Failed to initialize platform integration: {}", e);
                // Non-critical, so we continue
            }
            
            // Get platform info
            let platform_info = system::platform::PlatformInfo::current();
            info!("Running on platform: {:?}", platform_info.platform);
            
            // Initialize performance monitoring
            if let Err(e) = commands::performance::register(app) {
                error!("Failed to initialize performance monitoring: {}", e);
                // Non-critical, so we continue
            }
            
            Ok(())
        })
        .invoke_handler(invoke_handler)
        .build(tauri::generate_context!())
        .and_then(|app| {
            // Run the application with custom event handling
            app.run(|app_handle, event| match event {
                RunEvent::WindowEvent { label, event: WindowEvent::CloseRequested { api, .. }, .. } => {
                    if label == "main" {
                        // Get platform integration to handle close request
                        let platform_integration = system::platform::PlatformIntegration::new();
                        let minimize_on_close = true; // This would be loaded from config
                        
                        // If we should minimize instead of closing
                        if !platform_integration.handle_close_requested(app_handle, minimize_on_close) {
                            api.prevent_close();
                        }
                    }
                },
                RunEvent::ExitRequested { api, .. } => {
                    // Perform any cleanup needed before exit
                    info!("Application exit requested");
                },
                _ => {}
            });
            
            Ok(())
        });

    if let Err(e) = result {
        error!("Failed to run Tauri application: {}", e);
        
        // If we're in a GUI context, show an error dialog
        #[cfg(not(debug_assertions))]
        {
            use native_dialog::{MessageDialog, MessageType};
            let _ = MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Papin Application Error")
                .set_text(&format!("The application encountered a critical error: {}", e))
                .show_alert();
        }
        
        // Exit with error code
        std::process::exit(1);
    }
}