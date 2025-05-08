#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod feature_flags;
mod models;
mod protocols;
mod services;
mod shell_loader;
mod utils;

use env_logger::Env;
use log::{error, info};
use std::sync::{Arc, Mutex};
use tauri::{Manager, WindowBuilder, WindowUrl};
use tokio::runtime::Runtime;

use crate::feature_flags::{FeatureFlags, FeatureManager};
use crate::shell_loader::{launch_with_fast_shell, ShellLoader};
use crate::utils::config::Config;

// Global runtime handle for async operations
lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime");
}

// Global feature manager
lazy_static::lazy_static! {
    static ref FEATURE_MANAGER: Arc<Mutex<FeatureManager>> = Arc::new(Mutex::new(FeatureManager::from_env()));
}

#[tauri::command]
async fn get_app_info() -> Result<serde_json::Value, String> {
    let config = Config::global();
    let config = config.lock().unwrap();
    
    let app_name = config.get_string("app_name").unwrap_or_else(|| "Claude MCP Client".to_string());
    let version = config.get_string("version").unwrap_or_else(|| "0.1.0".to_string());
    
    let info = serde_json::json!({
        "name": app_name,
        "version": version,
        "platform": "linux"
    });
    
    Ok(info)
}

#[tauri::command]
async fn get_enabled_features() -> Result<Vec<String>, String> {
    let feature_manager = FEATURE_MANAGER.lock().unwrap();
    let flags = feature_manager.flags();
    
    let mut enabled_features = Vec::new();
    
    if flags.contains(FeatureFlags::EXPERIMENTAL) {
        enabled_features.push("experimental".to_string());
    }
    
    if flags.contains(FeatureFlags::DEV_FEATURES) {
        enabled_features.push("dev".to_string());
    }
    
    if flags.contains(FeatureFlags::LAZY_LOAD) {
        enabled_features.push("lazy_load".to_string());
    }
    
    if flags.contains(FeatureFlags::PLUGINS) {
        enabled_features.push("plugins".to_string());
    }
    
    if flags.contains(FeatureFlags::HISTORY) {
        enabled_features.push("history".to_string());
    }
    
    if flags.contains(FeatureFlags::ADVANCED_UI) {
        enabled_features.push("advanced_ui".to_string());
    }
    
    if flags.contains(FeatureFlags::ANALYTICS) {
        enabled_features.push("analytics".to_string());
    }
    
    if flags.contains(FeatureFlags::AUTO_UPDATE) {
        enabled_features.push("auto_update".to_string());
    }
    
    Ok(enabled_features)
}

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting Claude MCP Client");
    
    // Load config
    let config = Config::global();
    
    // Build Tauri application
    let mut builder = tauri::Builder::default();
    
    // Register commands
    builder = commands::register_commands(builder);
    
    builder
        .setup(|app| {
            // Get the main window or create it
            let window = app.get_window("main").unwrap_or_else(|| {
                WindowBuilder::new(
                    app,
                    "main".to_string(),
                    WindowUrl::App("index.html".into())
                )
                .title("Claude MCP")
                .inner_size(1200.0, 800.0)
                .visible(false) // Keep window hidden until shell is ready
                .build()
                .expect("Failed to create main window")
            });
            
            // Store app handle in state
            let app_handle = app.handle();
            app.manage(Arc::new(Mutex::new(app_handle)));
            
            // Start shell loader (this happens in Tokio runtime)
            RUNTIME.spawn(async move {
                let config_lock = config.lock().unwrap();
                let shell_loader = launch_with_fast_shell(window, &config_lock).await;
                
                // Log startup time
                if let Some(elapsed) = shell_loader.elapsed() {
                    info!("Shell ready in {}ms", elapsed.as_millis());
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_enabled_features,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Tauri application");
}