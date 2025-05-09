use mcp_client::optimization::{
    MemoryManager, MemoryLimits, Cache, CacheConfig, OptimizationManager
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[test]
fn test_memory_manager_cleanup() {
    // Create memory manager with custom limits
    let limits = MemoryLimits {
        max_memory_mb: 100,
        threshold_memory_mb: 50,
        max_context_tokens: 1000,
        enabled: true,
        check_interval_secs: 1,
    };

    let manager = MemoryManager::new();
    manager.update_limits(limits);

    // Simulate usage by updating token count
    manager.update_context_tokens(500);

    // Verify token count was updated
    let stats = manager.get_stats();
    assert_eq!(stats.current_context_tokens, 500);

    // Force cleanup
    manager.force_gc(false);

    // Verify cleanup was performed
    let stats_after = manager.get_stats();
    assert!(stats_after.gc_count > 0);
    assert!(stats_after.last_gc_time.is_some());
}

#[tokio::test]
async fn test_cache_ttl() {
    // Create cache with short TTL
    let config = CacheConfig {
        max_entries: 10,
        ttl_seconds: 1, // 1 second TTL
        persist: false,
        cache_file: None,
        enabled: true,
        cleanup_interval_secs: 1,
    };

    let cache: Cache<String, String> = Cache::new(config);
    cache.start_cleanup();

    // Add an item
    cache.put("key1".to_string(), "value1".to_string());

    // Verify item exists
    assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

    // Wait for TTL to expire
    time::sleep(Duration::from_secs(2)).await;

    // Verify item was removed
    assert_eq!(cache.get(&"key1".to_string()), None);

    // Stop cleanup task
    cache.stop_cleanup();
}

#[tokio::test]
async fn test_cache_get_or_compute() {
    // Create cache
    let config = CacheConfig::default();
    let cache: Cache<String, String> = Cache::new(config);

    // Define computation counter
    let computation_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let computation_count_clone = computation_count.clone();

    // Get or compute (first call - should compute)
    let value = cache.get_or_compute("key1".to_string(), || async move {
        computation_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        "computed_value".to_string()
    }).await;

    assert_eq!(value, "computed_value");
    assert_eq!(computation_count.load(std::sync::atomic::Ordering::SeqCst), 1);

    // Get or compute again (should use cache)
    let value = cache.get_or_compute("key1".to_string(), || async move {
        computation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        "different_value".to_string()
    }).await;

    assert_eq!(value, "computed_value"); // Should still be the original value
    assert_eq!(computation_count.load(std::sync::atomic::Ordering::SeqCst), 1); // Count shouldn't increase
}

#[test]
fn test_optimization_manager() {
    // Create optimization manager
    let manager = OptimizationManager::new();
    manager.start();

    // Get components
    let memory_manager = manager.memory_manager();
    let api_cache = manager.api_cache();
    let resource_cache = manager.resource_cache();

    // Verify components are initialized
    assert!(memory_manager.get_limits().enabled);
    assert!(api_cache.get_config().enabled);
    assert!(resource_cache.get_config().enabled);

    // Add some items to cache
    api_cache.put("test_key".to_string(), "test_value".to_string());
    assert_eq!(api_cache.get(&"test_key".to_string()), Some("test_value".to_string()));

    // Force garbage collection (should clear caches)
    memory_manager.force_gc(true);

    // Verify caches were cleared
    assert_eq!(api_cache.get(&"test_key".to_string()), None);

    // Stop optimization manager
    manager.stop();
}