// src-tauri/src/system/mod.rs
//
// System integration module

pub mod platform;

use tauri::{Invoke, Runtime};

// Function to register system commands with Tauri
pub fn register_commands<R: Runtime>() -> Vec<Invoke<R>> {
    vec![
        platform::get_platform_info::invoke,
        platform::get_platform_name::invoke,
        platform::show_platform_notification::invoke,
    ]
}

// System integration initialization
pub fn initialize<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    // Create and initialize the platform integration
    let platform_integration = platform::PlatformIntegration::new();
    platform_integration.initialize(app)?;
    
    Ok(())
}
