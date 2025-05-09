mod memory;
mod cache;

pub use memory::{MemoryManager, MemoryLimits, MemoryStats};
pub use cache::{Cache, CacheConfig, CacheStats};

use log::{info, debug, warn};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Window};

/// Manager for all optimizations
pub struct OptimizationManager {
    memory_manager: Arc<MemoryManager>,
    api_cache: Arc<Cache<String, String>>,
    resource_cache: Arc<Cache<String, Vec<u8>>>,
}

impl OptimizationManager {
    /// Create a new optimization manager
    pub fn new() -> Self {
        // Initialize memory manager
        let memory_manager = Arc::new(MemoryManager::new());
        
        // Initialize API cache
        let api_cache_config = CacheConfig {
            max_entries: 1000,
            ttl_seconds: 300, // 5 minutes
            persist: true,
            cache_file: Some("api_cache.json".to_string()),
            enabled: true,
            cleanup_interval_secs: 60,
        };
        let api_cache = Arc::new(Cache::new(api_cache_config));
        
        // Initialize resource cache
        let resource_cache_config = CacheConfig {
            max_entries: 200,
            ttl_seconds: 3600, // 1 hour
            persist: true,
            cache_file: Some("resource_cache.bin".to_string()),
            enabled: true,
            cleanup_interval_secs: 300,
        };
        let resource_cache = Arc::new(Cache::new(resource_cache_config));
        
        Self {
            memory_manager,
            api_cache,
            resource_cache,
        }
    }
    
    /// Start all optimization managers
    pub fn start(&self) {
        // Start memory manager
        self.memory_manager.start();
        
        // Start cache cleanup tasks
        self.api_cache.start_cleanup();
        self.resource_cache.start_cleanup();
        
        // Register memory optimization handlers
        self.register_memory_optimizations();
        
        info!("Optimization manager started");
    }
    
    /// Stop all optimization managers
    pub fn stop(&self) {
        // Stop memory manager
        self.memory_manager.stop();
        
        // Stop cache cleanup tasks
        self.api_cache.stop_cleanup();
        self.resource_cache.stop_cleanup();
        
        info!("Optimization manager stopped");
    }
    
    /// Register memory optimizations with appropriate subsystems
    fn register_memory_optimizations(&self) {
        let memory_manager = self.memory_manager.clone();
        
        // Register cleaner for API cache
        {
            let api_cache = self.api_cache.clone();
            memory_manager.register_cleaner(move || {
                let stats_before = api_cache.get_stats();
                api_cache.clear();
                let stats_after = api_cache.get_stats();
                debug!("API cache cleared: {} entries removed", stats_before.size - stats_after.size);
            });
        }
        
        // Register cleaner for resource cache
        {
            let resource_cache = self.resource_cache.clone();
            memory_manager.register_cleaner(move || {
                let stats_before = resource_cache.get_stats();
                resource_cache.clear();
                let stats_after = resource_cache.get_stats();
                debug!("Resource cache cleared: {} entries removed", stats_before.size - stats_after.size);
            });
        }
    }
    
    /// Register window-related optimizations for Tauri
    pub fn register_window_optimizations(&self, app: &AppHandle) {
        let window = app.get_window("main").unwrap();
        let memory_manager = self.memory_manager.clone();
        
        // Register blur handler to reduce memory usage when window is not focused
        window.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    if !*focused {
                        // Window lost focus, perform memory optimization
                        memory_manager.force_gc(true);
                    }
                }
                tauri::WindowEvent::CloseRequested { .. } => {
                    // Window is closing, perform memory cleanup
                    memory_manager.force_gc(true);
                }
                _ => {}
            }
        });
    }
    
    /// Get the memory manager
    pub fn memory_manager(&self) -> Arc<MemoryManager> {
        self.memory_manager.clone()
    }
    
    /// Get the API cache
    pub fn api_cache(&self) -> Arc<Cache<String, String>> {
        self.api_cache.clone()
    }
    
    /// Get the resource cache
    pub fn resource_cache(&self) -> Arc<Cache<String, Vec<u8>>> {
        self.resource_cache.clone()
    }
}

impl Drop for OptimizationManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_manager_creation() {
        let manager = OptimizationManager::new();
        
        // Check memory manager limits
        let memory_limits = manager.memory_manager().get_limits();
        assert_eq!(memory_limits.max_memory_mb, 1024);
        assert_eq!(memory_limits.threshold_memory_mb, 768);
        
        // Check API cache config
        let api_cache_config = manager.api_cache().get_config();
        assert_eq!(api_cache_config.max_entries, 1000);
        assert_eq!(api_cache_config.ttl_seconds, 300);
        
        // Check resource cache config
        let resource_cache_config = manager.resource_cache().get_config();
        assert_eq!(resource_cache_config.max_entries, 200);
        assert_eq!(resource_cache_config.ttl_seconds, 3600);
    }
}