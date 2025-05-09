use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    extract::Path,
    http::{StatusCode, HeaderValue, header},
    Json,
};
use serde_json::{json, Value};
use tauri::{Manager, Wry, Config};
use std::process::Command;
use std::time::Duration;

// Test server for mock updates
struct TestUpdateServer {
    addr: SocketAddr,
    version: Arc<Mutex<String>>,
}

impl TestUpdateServer {
    async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let version = Arc::new(Mutex::new("1.0.1".to_string()));
        
        let version_clone = version.clone();
        let app = Router::new()
            .route(
                "/:target/:current_version",
                get(move |Path((target, current_version))| {
                    let version = version_clone.clone();
                    async move {
                        let version_str = version.lock().await.clone();
                        if current_version == version_str {
                            return (StatusCode::NO_CONTENT, "No updates available").into_response();
                        }
                        
                        let update_data = json!({
                            "version": version_str,
                            "notes": "Test update",
                            "pub_date": "2025-05-09T12:00:00Z",
                            "url": format!("http://{}:{}/download/{}", addr.ip(), addr.port(), target),
                            "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHNpZ25hdHVyZTogNzI1Y2E5OWQ5MTkwYmM1NQp0cnVzdGVkCmNvbW1lbnQ6IHRpbWVzdGFtcDoxNjUwNTU5NTI4CWZpbGU6TUNQLUNsaWVudF8xLjAuMV94NjRfZW4tVVMubXNpCmhxZkpNWEJFTkRGd0tIWmZBakJQZWZKSC85cW5PampUZzZucG9XNXY0VWhNcUdFWmNXeHRCSUdHUCthTVZ0dTQyM21XUDZSdTJLcXcrWmVqTFhhRWdnPT0K"
                        });
                        
                        let mut response = Json(update_data).into_response();
                        response.headers_mut().insert(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static("application/json"),
                        );
                        
                        response
                    }
                }),
            )
            .route(
                "/download/:target",
                get(move |Path(target)| async move {
                    // In a real test, we would return a mock installer file
                    // For this test, we just return a success response
                    format!("Mock installer for {}", target)
                }),
            );
            
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        
        Self {
            addr,
            version,
        }
    }
    
    async fn set_version(&self, version: &str) {
        let mut v = self.version.lock().await;
        *v = version.to_string();
    }
    
    fn url(&self) -> String {
        format!("http://{}:{}", self.addr.ip(), self.addr.port())
    }
}

// Main test function
#[tokio::test]
async fn test_auto_update() {
    // Start mock update server
    let update_server = TestUpdateServer::new().await;
    
    // Set server to have a newer version
    update_server.set_version("1.0.2").await;
    
    // Create a custom Tauri config for testing
    let mut config = Config::default();
    config.build.dist_dir = "../dist".into();
    config.package.version = "1.0.1".into();
    
    // Override the updater endpoint with our mock server
    config.tauri.updater = Some(tauri::UpdaterConfig {
        active: true,
        dialog: true,
        endpoints: vec![format!("{}/{{{{target}}}}/{{{{current_version}}}}", update_server.url())],
        pubkey: "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEMxNEYzODkyRjVCQjk3NjUKUldRTVVLVldVdXRrNC9WVklSVmorenBFODZIajVhUG16NnRKU2xEZ1JhRk9oNFpyRklBUkFBQUIKCg==".into(),
    });
    
    // Create a test event to track update availability
    let update_available = Arc::new(tokio::sync::Mutex::new(false));
    let update_version = Arc::new(tokio::sync::Mutex::new(String::new()));
    
    // Build Tauri app for testing
    let update_available_clone = update_available.clone();
    let update_version_clone = update_version.clone();
    
    let app = tauri::Builder::default()
        .config(config)
        .setup(move |app| {
            let app_handle = app.handle();
            
            // Listen for update events
            app_handle.listen_global("update-available", move |event| {
                let version = event.payload().unwrap_or("unknown");
                println!("Update available: {}", version);
                
                let update_available = update_available_clone.clone();
                let update_version = update_version_clone.clone();
                let version_str = version.to_string();
                
                tokio::spawn(async move {
                    let mut available = update_available.lock().await;
                    *available = true;
                    
                    let mut version = update_version.lock().await;
                    *version = version_str;
                });
            });
            
            // Initialize the updater
            let updater_state = app.state::<crate::commands::update::UpdaterState>();
            updater_state.initialize(app_handle);
            
            Ok(())
        })
        .build(tauri::test::mock_context())
        .expect("Failed to build test app");
    
    // Get the update manager
    let updater_state = app.state::<crate::commands::update::UpdaterState>();
    let manager = updater_state.get_manager().expect("Failed to get update manager");
    
    // Check for updates
    manager.check_for_updates().await;
    
    // Wait for update check to complete (with timeout)
    let mut attempts = 0;
    while attempts < 10 {
        if *update_available.lock().await {
            break;
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        attempts += 1;
    }
    
    // Assert that update was detected
    assert!(*update_available.lock().await, "Update was not detected");
    assert_eq!(*update_version.lock().await, "1.0.2", "Wrong update version detected");
    
    // Test with same version (no update should be available)
    update_server.set_version("1.0.1").await;
    
    // Reset flags
    {
        let mut available = update_available.lock().await;
        *available = false;
        
        let mut version = update_version.lock().await;
        *version = String::new();
    }
    
    // Check for updates again
    manager.check_for_updates().await;
    
    // Wait for update check to complete (with timeout)
    let mut attempts = 0;
    while attempts < 10 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        attempts += 1;
    }
    
    // Assert that no update was detected
    assert!(!*update_available.lock().await, "Update was incorrectly detected");
    assert_eq!(*update_version.lock().await, "", "Update version should be empty");
    
    println!("Auto-update tests passed successfully!");
}