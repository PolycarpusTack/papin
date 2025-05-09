use mcp_client::auto_update::{UpdateManager, UpdaterConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;
use mockall::predicate::*;
use mockall::mock;

// Mock the HTTP client for testing
mock! {
    pub HttpClient {
        fn get(&self, url: String) -> Result<Vec<u8>, String>;
        fn post(&self, url: String, body: Vec<u8>) -> Result<Vec<u8>, String>;
    }
}

#[tokio::test]
async fn test_updater_check_with_update_available() {
    // Create mock HTTP client
    let mut mock_client = MockHttpClient::new();
    mock_client
        .expect_get()
        .with(eq("https://update.mcp-client.com/win64/1.0.0".to_string()))
        .returning(|_| {
            Ok(r#"{
                "version": "1.0.1",
                "notes": "Test update",
                "pub_date": "2025-05-10T12:00:00Z",
                "url": "https://update.mcp-client.com/download/mcp-client-1.0.1.msi",
                "signature": "test-signature"
            }"#.as_bytes().to_vec())
        });

    // Create update manager with custom config
    let config = UpdaterConfig {
        enabled: true,
        check_interval: 24,
        last_check: None,
        auto_download: true,
        auto_install: false,
    };

    // Create app event tracker
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let events_clone = events.clone();

    // Create mock app handle
    let app_handle = MockAppHandle::new(move |event, payload| {
        let mut events = events_clone.lock().unwrap();
        events.push(format!("{}:{}", event, payload));
    });

    // Create update manager
    let manager = UpdateManager::new_with_client(app_handle, mock_client);
    manager.update_config(config).await;

    // Check for updates
    manager.check_for_updates().await;

    // Wait for events to be processed
    time::sleep(Duration::from_millis(100)).await;

    // Verify that events were emitted
    let events = events.lock().unwrap();
    assert!(events.iter().any(|e| e.starts_with("update-available")));
    assert!(events.iter().any(|e| e.contains("1.0.1")));
}

#[tokio::test]
async fn test_updater_check_no_update() {
    // Create mock HTTP client
    let mut mock_client = MockHttpClient::new();
    mock_client
        .expect_get()
        .with(eq("https://update.mcp-client.com/win64/1.0.1".to_string()))
        .returning(|_| {
            Ok(r#"{
                "version": "1.0.1",
                "notes": "No update available",
                "pub_date": "2025-05-10T12:00:00Z"
            }"#.as_bytes().to_vec())
        });

    // Create update manager with custom config
    let config = UpdaterConfig {
        enabled: true,
        check_interval: 24,
        last_check: None,
        auto_download: true,
        auto_install: false,
    };

    // Create app event tracker
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let events_clone = events.clone();

    // Create mock app handle
    let app_handle = MockAppHandle::new(move |event, payload| {
        let mut events = events_clone.lock().unwrap();
        events.push(format!("{}:{}", event, payload));
    });

    // Create update manager with version 1.0.1 (same as server)
    let manager = UpdateManager::new_with_client_and_version(app_handle, mock_client, "1.0.1".to_string());
    manager.update_config(config).await;

    // Check for updates
    manager.check_for_updates().await;

    // Wait for events to be processed
    time::sleep(Duration::from_millis(100)).await;

    // Verify that no update events were emitted
    let events = events.lock().unwrap();
    assert!(!events.iter().any(|e| e.starts_with("update-available")));
}

#[tokio::test]
async fn test_updater_with_auto_install() {
    // Create mock HTTP client
    let mut mock_client = MockHttpClient::new();
    mock_client
        .expect_get()
        .with(eq("https://update.mcp-client.com/win64/1.0.0".to_string()))
        .returning(|_| {
            Ok(r#"{
                "version": "1.0.1",
                "notes": "Test update",
                "pub_date": "2025-05-10T12:00:00Z",
                "url": "https://update.mcp-client.com/download/mcp-client-1.0.1.msi",
                "signature": "test-signature"
            }"#.as_bytes().to_vec())
        });

    // Mock download response
    mock_client
        .expect_get()
        .with(eq("https://update.mcp-client.com/download/mcp-client-1.0.1.msi".to_string()))
        .returning(|_| {
            Ok(vec![0, 1, 2, 3, 4]) // Mock installer bytes
        });

    // Create update manager with auto-install enabled
    let config = UpdaterConfig {
        enabled: true,
        check_interval: 24,
        last_check: None,
        auto_download: true,
        auto_install: true,
    };

    // Create app event tracker
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let install_called = Arc::new(Mutex::new(false));
    let events_clone = events.clone();
    let install_called_clone = install_called.clone();

    // Create mock app handle with install capability
    let app_handle = MockAppHandleWithInstall::new(
        move |event, payload| {
            let mut events = events_clone.lock().unwrap();
            events.push(format!("{}:{}", event, payload));
        },
        move || {
            let mut called = install_called_clone.lock().unwrap();
            *called = true;
            Ok(())
        }
    );

    // Create update manager
    let manager = UpdateManager::new_with_client(app_handle, mock_client);
    manager.update_config(config).await;

    // Check for updates
    manager.check_for_updates().await;

    // Wait for events to be processed
    time::sleep(Duration::from_millis(100)).await;

    // Verify that events were emitted and install was called
    let events = events.lock().unwrap();
    assert!(events.iter().any(|e| e.starts_with("update-available")));
    
    let install_called = install_called.lock().unwrap();
    assert!(*install_called);
}

// Mock app handle
struct MockAppHandle {
    event_handler: Box<dyn Fn(&str, &str) + Send + Sync>,
}

impl MockAppHandle {
    pub fn new<F>(event_handler: F) -> Self 
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        Self {
            event_handler: Box::new(event_handler),
        }
    }

    pub fn emit_all(&self, event: &str, payload: &str) -> Result<(), String> {
        (self.event_handler)(event, payload);
        Ok(())
    }
}

// Mock app handle with install capability
struct MockAppHandleWithInstall {
    event_handler: Box<dyn Fn(&str, &str) + Send + Sync>,
    install_handler: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
}

impl MockAppHandleWithInstall {
    pub fn new<F, I>(event_handler: F, install_handler: I) -> Self 
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
        I: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            event_handler: Box::new(event_handler),
            install_handler: Box::new(install_handler),
        }
    }

    pub fn emit_all(&self, event: &str, payload: &str) -> Result<(), String> {
        (self.event_handler)(event, payload);
        Ok(())
    }

    pub fn install(&self) -> Result<(), String> {
        (self.install_handler)()
    }
}