// Integration test for the auto-update functionality

use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

use crate::feature_flags::FeatureFlags;
use crate::updater::{init_updater, UpdateStatus};

#[tokio::test]
async fn test_updater_initialization() {
    // Create a mock Tauri app for testing
    let context = tauri::generate_context!();
    let app = tauri::Builder::default()
        .build(context)
        .expect("Failed to build app");
    let app_handle = app.handle();
    
    // Enable auto-update feature flag
    let mut feature_flags = FeatureFlags::empty();
    feature_flags |= FeatureFlags::AUTO_UPDATE;
    
    // Initialize the updater
    init_updater(app_handle.clone(), feature_flags).expect("Failed to initialize updater");
    
    // Allow some time for initialization
    sleep(Duration::from_millis(500)).await;
    
    // Get update info
    let update_info: serde_json::Value = app_handle
        .invoke_handler()
        .invoke_raw("getUpdateInfo", "null".into())
        .expect("Failed to get update info");
    
    // Verify that the updater is initialized correctly
    let status = update_info["status"].as_str().unwrap();
    assert!(
        status == "Checking" || status == "UpToDate",
        "Expected status to be Checking or UpToDate, got {}",
        status
    );
}

#[tokio::test]
async fn test_updater_disabled() {
    // Create a mock Tauri app for testing
    let context = tauri::generate_context!();
    let app = tauri::Builder::default()
        .build(context)
        .expect("Failed to build app");
    let app_handle = app.handle();
    
    // Disable auto-update feature flag
    let feature_flags = FeatureFlags::empty();
    
    // Initialize the updater
    init_updater(app_handle.clone(), feature_flags).expect("Failed to initialize updater");
    
    // Allow some time for initialization
    sleep(Duration::from_millis(500)).await;
    
    // Get update info
    let update_info: serde_json::Value = app_handle
        .invoke_handler()
        .invoke_raw("getUpdateInfo", "null".into())
        .expect("Failed to get update info");
    
    // Verify that the updater is disabled
    let status = update_info["status"].as_str().unwrap();
    assert_eq!(status, "Disabled", "Expected status to be Disabled");
    
    // Enable updates
    app_handle
        .invoke_handler()
        .invoke_raw("setUpdateEnabled", "true".into())
        .expect("Failed to enable updates");
    
    // Allow some time for the change to take effect
    sleep(Duration::from_millis(500)).await;
    
    // Get update info again
    let update_info: serde_json::Value = app_handle
        .invoke_handler()
        .invoke_raw("getUpdateInfo", "null".into())
        .expect("Failed to get update info");
    
    // Verify that the updater is now enabled
    let status = update_info["status"].as_str().unwrap();
    assert!(
        status == "Checking" || status == "UpToDate",
        "Expected status to be Checking or UpToDate, got {}",
        status
    );
}

#[tokio::test]
async fn test_check_interval() {
    // Create a mock Tauri app for testing
    let context = tauri::generate_context!();
    let app = tauri::Builder::default()
        .build(context)
        .expect("Failed to build app");
    let app_handle = app.handle();
    
    // Enable auto-update feature flag
    let mut feature_flags = FeatureFlags::empty();
    feature_flags |= FeatureFlags::AUTO_UPDATE;
    
    // Initialize the updater
    init_updater(app_handle.clone(), feature_flags).expect("Failed to initialize updater");
    
    // Set update check interval to 2 hours
    app_handle
        .invoke_handler()
        .invoke_raw("setUpdateCheckInterval", "2".into())
        .expect("Failed to set update check interval");
    
    // Try to set an invalid interval
    let result = app_handle
        .invoke_handler()
        .invoke_raw("setUpdateCheckInterval", "0".into());
    
    // Verify that setting an invalid interval fails
    assert!(result.is_err(), "Expected setting invalid interval to fail");
}
