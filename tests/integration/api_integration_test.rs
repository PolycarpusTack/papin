use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tauri::{Manager, Runtime, Wry};
use mcp_client::commands::{api, update, optimization, offline};
use serde_json::json;

// Test the API commands integration with the backend
#[tokio::test]
async fn test_api_integration() {
    // Build a test Tauri application
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Register commands
            api::register_commands(app)?;
            update::register_commands(app)?;
            optimization::register_commands(app)?;
            offline::register_commands(app)?;
            
            Ok(())
        })
        .build()
        .expect("Failed to build test app");
    
    // Initialize the API client
    let result = api::init_api_client(
        app.app_handle(), 
        json!({
            "baseUrl": "https://api.mcp-client.test",
            "timeout": 30000,
            "retries": 3
        })
    );
    
    assert!(result.is_ok());
    
    // Set up a mock HTTP server
    let server = mockito::Server::new();
    let mock_url = server.url();
    
    // Mock API response
    let _m = server.mock("GET", "/api/v1/models")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "models": [
                {
                    "id": "model1",
                    "name": "Test Model 1",
                    "description": "A test model"
                },
                {
                    "id": "model2",
                    "name": "Test Model 2",
                    "description": "Another test model"
                }
            ]
        }"#)
        .create();
    
    // Update API base URL to use mock server
    let result = api::update_api_config(
        app.app_handle(), 
        json!({
            "baseUrl": mock_url,
            "timeout": 30000,
            "retries": 3
        })
    );
    
    assert!(result.is_ok());
    
    // Make API request
    let response = api::fetch_models(app.app_handle());
    
    // Verify response
    assert!(response.is_ok());
    let models = response.unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0]["id"], "model1");
    assert_eq!(models[1]["id"], "model2");
}

// Test offline fallback for API requests
#[tokio::test]
async fn test_api_offline_fallback() {
    // Build a test Tauri application
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Register commands
            api::register_commands(app)?;
            update::register_commands(app)?;
            optimization::register_commands(app)?;
            offline::register_commands(app)?;
            
            Ok(())
        })
        .build()
        .expect("Failed to build test app");
    
    // Initialize the API client
    let _ = api::init_api_client(
        app.app_handle(), 
        json!({
            "baseUrl": "https://nonexistent-api.mcp-client.test",
            "timeout": 1000, // Short timeout for testing
            "retries": 1
        })
    );
    
    // Initialize offline manager
    let _ = offline::init_offline_manager(app.app_handle());
    
    // Enable offline fallback
    let _ = offline::update_offline_config(
        app.app_handle(),
        json!({
            "enabled": true,
            "auto_switch": true,
            "use_local_llm": true
        })
    );
    
    // Make API request (should fail and trigger offline mode)
    let response = api::fetch_models(app.app_handle());
    
    // Wait for offline switch to complete
    time::sleep(Duration::from_millis(500)).await;
    
    // Verify offline mode was activated
    let status = offline::get_offline_status(app.app_handle());
    assert!(status.is_ok());
    assert_eq!(status.unwrap(), "Offline");
    
    // Try the same request again (should use offline cache/local model)
    let response2 = api::fetch_models(app.app_handle());
    
    // Verify response from offline mode
    assert!(response2.is_ok());
}

// Test API performance with caching
#[tokio::test]
async fn test_api_performance_with_caching() {
    // Build a test Tauri application
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Register commands
            api::register_commands(app)?;
            update::register_commands(app)?;
            optimization::register_commands(app)?;
            
            Ok(())
        })
        .build()
        .expect("Failed to build test app");
    
    // Initialize the API client
    let _ = api::init_api_client(
        app.app_handle(), 
        json!({
            "baseUrl": "https://api.mcp-client.test",
            "timeout": 30000,
            "retries": 3
        })
    );
    
    // Initialize optimization manager
    let _ = optimization::init_optimizations(app.app_handle());
    
    // Set up a mock HTTP server
    let server = mockito::Server::new();
    let mock_url = server.url();
    
    // Mock API response with a delay
    let _m = server.mock("GET", "/api/v1/models")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "models": [
                {
                    "id": "model1",
                    "name": "Test Model 1",
                    "description": "A test model"
                }
            ]
        }"#)
        .with_delay(Duration::from_millis(500)) // Simulate network delay
        .create();
    
    // Update API base URL to use mock server
    let _ = api::update_api_config(
        app.app_handle(), 
        json!({
            "baseUrl": mock_url,
            "timeout": 30000,
            "retries": 3
        })
    );
    
    // Enable API caching
    let _ = optimization::update_api_cache_config(
        app.app_handle(),
        json!({
            "enabled": true,
            "maxEntries": 100,
            "ttlSeconds": 60
        })
    );
    
    // Make first API request (should hit the network)
    let start = std::time::Instant::now();
    let response1 = api::fetch_models(app.app_handle());
    let duration1 = start.elapsed();
    
    assert!(response1.is_ok());
    
    // Make second API request (should use cache)
    let start = std::time::Instant::now();
    let response2 = api::fetch_models(app.app_handle());
    let duration2 = start.elapsed();
    
    assert!(response2.is_ok());
    
    // Verify second request was faster due to caching
    assert!(duration2 < duration1);
    
    // Check cache stats
    let stats = optimization::get_api_cache_stats(app.app_handle());
    assert!(stats.is_ok());
    let stats = stats.unwrap();
    assert_eq!(stats["hits"], 1);
}