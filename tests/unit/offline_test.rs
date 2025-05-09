use mcp_client::offline::{
    OfflineManager, OfflineStatus, OfflineConfig,
    llm::LocalLLM,
    checkpointing::CheckpointManager,
    sync::{SyncManager, SyncConfig, SyncResolutionStrategy}
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

#[test]
fn test_offline_mode_switching() {
    // Create offline manager
    let manager = OfflineManager::new();
    
    // Verify initial state
    assert_eq!(manager.get_status(), OfflineStatus::Online);
    
    // Switch to offline mode
    let result = manager.go_offline();
    assert!(result.is_ok());
    assert_eq!(manager.get_status(), OfflineStatus::Offline);
    
    // Try to switch to offline again (should fail)
    let result = manager.go_offline();
    assert!(result.is_err());
    
    // Switch back to online mode
    // Note: This might fail if real network checks are performed
    let _ = manager.go_online();
}

#[test]
fn test_offline_configuration() {
    // Create offline manager
    let manager = OfflineManager::new();
    
    // Create custom config
    let config = OfflineConfig {
        enabled: true,
        auto_switch: false,
        use_local_llm: true,
        connectivity_check_interval: 60,
        network_timeout_ms: 10000,
        max_checkpoints: 5,
        sync: SyncConfig {
            enabled: true,
            interval_seconds: 600,
            device_id: "test_device".to_string(),
            auto_sync: false,
            default_resolution: SyncResolutionStrategy::UseLocal,
            sync_on_startup: false,
            sync_on_shutdown: false,
        },
    };
    
    // Update config
    manager.update_config(config.clone());
    
    // Verify config was updated
    let updated_config = manager.get_config();
    assert_eq!(updated_config.auto_switch, false);
    assert_eq!(updated_config.connectivity_check_interval, 60);
    assert_eq!(updated_config.sync.interval_seconds, 600);
    assert_eq!(updated_config.sync.auto_sync, false);
}

#[test]
fn test_local_llm() {
    // Create LLM
    let llm = LocalLLM::new("test", 1024, 100);
    
    // Generate text
    let input = "This is a test input";
    let output = llm.generate(input, 10);
    
    // Verify output was generated
    assert!(!output.is_empty());
    assert!(output.contains("Generated"));
    
    // Test context limit
    let large_input = "test ".repeat(1000);
    let output = llm.generate(&large_input, 10);
    
    // Verify context limit is enforced
    assert!(output.contains("too long"));
}

#[tokio::test]
async fn test_checkpoint_save_load() {
    // Create temporary directory for testing
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create checkpoint manager
    let manager = CheckpointManager::new()
        .with_base_path(temp_dir.path())
        .with_max_checkpoints(3);
    
    // Create test data
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    data.insert("key2".to_string(), "value2".to_string());
    
    // Save checkpoint
    let checkpoint_id = manager.save_checkpoint("test", &data);
    
    // Load checkpoint
    let loaded: Option<HashMap<String, String>> = manager.load_checkpoint(&checkpoint_id);
    
    // Verify data was loaded correctly
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.get("key1"), Some(&"value1".to_string()));
    assert_eq!(loaded.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_sync_conflict_resolution() {
    // Create sync manager
    let manager = SyncManager::new();
    
    // Create local and remote changes with conflict
    let mut local_changes = HashMap::new();
    local_changes.insert("key1".to_string(), "local_value".to_string());
    
    let mut remote_changes = HashMap::new();
    remote_changes.insert("key1".to_string(), "remote_value".to_string());
    
    // Perform sync (default resolution is UseRemote)
    let result = SyncManager::sync(local_changes, remote_changes);
    
    // Verify conflict was detected and resolved
    assert!(result.success);
    assert_eq!(result.conflicts.len(), 1);
    
    let conflict = &result.conflicts[0];
    assert_eq!(conflict.key, "key1");
    assert_eq!(conflict.resolution, SyncResolutionStrategy::UseRemote);
    assert_eq!(conflict.resolved_value, Some("remote_value".to_string()));
}