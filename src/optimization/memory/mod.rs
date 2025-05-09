use log::{debug, info, warn};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tokio::time::interval;

/// Memory usage limits and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum memory usage in MB before aggressive cleanup
    pub max_memory_mb: usize,
    /// Memory threshold in MB for regular cleanup
    pub threshold_memory_mb: usize,
    /// Maximum LLM context size in tokens
    pub max_context_tokens: usize,
    /// Enable memory tracking and optimization
    pub enabled: bool,
    /// Interval in seconds for memory usage check
    pub check_interval_secs: u64,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024, // 1GB
            threshold_memory_mb: 768, // 768MB
            max_context_tokens: 8192,
            enabled: true,
            check_interval_secs: 30,
        }
    }
}

/// Memory utilization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_usage_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_usage_bytes: usize,
    /// System total memory in bytes
    pub total_memory_bytes: usize,
    /// Last garbage collection time
    pub last_gc_time: Option<String>,
    /// Number of garbage collections performed
    pub gc_count: usize,
    /// Current LLM context size in tokens
    pub current_context_tokens: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            current_usage_bytes: 0,
            peak_usage_bytes: 0,
            total_memory_bytes: 0,
            last_gc_time: None,
            gc_count: 0,
            current_context_tokens: 0,
        }
    }
}

/// Memory manager for optimizing application memory usage
pub struct MemoryManager {
    limits: Arc<Mutex<MemoryLimits>>,
    stats: Arc<Mutex<MemoryStats>>,
    running: Arc<AtomicUsize>,
    registered_cleaners: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        let stats = Arc::new(Mutex::new(MemoryStats::default()));
        
        // Initialize system memory info
        if let Ok(sys_info) = sys_info::mem_info() {
            let mut stats = stats.lock().unwrap();
            stats.total_memory_bytes = sys_info.total as usize * 1024;
        }
        
        Self {
            limits: Arc::new(Mutex::new(MemoryLimits::default())),
            stats,
            running: Arc::new(AtomicUsize::new(0)),
            registered_cleaners: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Start memory monitoring
    pub fn start(&self) {
        // Only start if not already running
        if self.running.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return;
        }
        
        let limits = self.limits.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        let cleaners = self.registered_cleaners.clone();
        
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(
                limits.lock().unwrap().check_interval_secs
            ));
            
            while running.load(Ordering::SeqCst) == 1 {
                check_interval.tick().await;
                
                if !limits.lock().unwrap().enabled {
                    continue;
                }
                
                // Update memory stats
                Self::update_memory_stats(&stats);
                
                // Check if memory usage exceeds limits
                let should_cleanup = {
                    let limits = limits.lock().unwrap();
                    let stats = stats.lock().unwrap();
                    
                    let current_mb = stats.current_usage_bytes / (1024 * 1024);
                    current_mb >= limits.threshold_memory_mb
                };
                
                if should_cleanup {
                    let aggressive = {
                        let limits = limits.lock().unwrap();
                        let stats = stats.lock().unwrap();
                        
                        let current_mb = stats.current_usage_bytes / (1024 * 1024);
                        current_mb >= limits.max_memory_mb
                    };
                    
                    Self::perform_cleanup(&cleaners, aggressive, &stats);
                }
            }
        });
    }
    
    /// Stop memory monitoring
    pub fn stop(&self) {
        self.running.store(0, Ordering::SeqCst);
    }
    
    /// Register a cleanup function
    pub fn register_cleaner<F>(&self, cleaner: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut cleaners = self.registered_cleaners.lock().unwrap();
        cleaners.push(Box::new(cleaner));
    }
    
    /// Update memory limits
    pub fn update_limits(&self, limits: MemoryLimits) {
        let mut current_limits = self.limits.lock().unwrap();
        *current_limits = limits;
    }
    
    /// Get current memory limits
    pub fn get_limits(&self) -> MemoryLimits {
        self.limits.lock().unwrap().clone()
    }
    
    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Update context token count
    pub fn update_context_tokens(&self, token_count: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.current_context_tokens = token_count;
        
        // Check if token count exceeds limits
        let limits = self.limits.lock().unwrap();
        if token_count > limits.max_context_tokens {
            warn!("LLM context size ({} tokens) exceeds limit ({} tokens)",
                token_count, limits.max_context_tokens);
        }
    }
    
    /// Request immediate garbage collection
    pub fn force_gc(&self, aggressive: bool) {
        Self::perform_cleanup(&self.registered_cleaners, aggressive, &self.stats);
    }
    
    /// Update memory statistics
    fn update_memory_stats(stats: &Arc<Mutex<MemoryStats>>) {
        if let Ok(sys_mem) = sys_info::mem_info() {
            let mut stats = stats.lock().unwrap();
            
            // Calculate current memory usage
            let used_mem = sys_mem.total - sys_mem.free - sys_mem.avail;
            stats.current_usage_bytes = used_mem as usize * 1024;
            
            // Update peak memory usage
            if stats.current_usage_bytes > stats.peak_usage_bytes {
                stats.peak_usage_bytes = stats.current_usage_bytes;
            }
            
            // Update total memory if needed
            if stats.total_memory_bytes == 0 {
                stats.total_memory_bytes = sys_mem.total as usize * 1024;
            }
        }
    }
    
    /// Perform memory cleanup
    fn perform_cleanup(
        cleaners: &Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
        aggressive: bool,
        stats: &Arc<Mutex<MemoryStats>>,
    ) {
        let now = Instant::now();
        
        // Run JavaScript garbage collection
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            
            // Call JavaScript garbage collection
            let _ = js_sys::eval("if (window.gc) { window.gc(); }");
            
            if aggressive {
                // Multiple GC calls in aggressive mode
                for _ in 0..3 {
                    let _ = js_sys::eval("if (window.gc) { window.gc(); }");
                }
            }
        }
        
        // Call registered cleanup functions
        let cleaners = cleaners.lock().unwrap();
        for cleaner in cleaners.iter() {
            cleaner();
        }
        
        // Force Rust garbage collection
        #[cfg(feature = "mimalloc")]
        unsafe {
            mimalloc_sys::mi_collect(aggressive as _);
        }
        
        // Update stats
        {
            let mut stats = stats.lock().unwrap();
            stats.gc_count += 1;
            stats.last_gc_time = Some(chrono::Local::now().to_rfc3339());
        }
        
        // Update memory stats after cleanup
        Self::update_memory_stats(stats);
        
        let elapsed = now.elapsed();
        if aggressive {
            info!("Completed aggressive memory cleanup in {:?}", elapsed);
        } else {
            debug!("Completed regular memory cleanup in {:?}", elapsed);
        }
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    #[test]
    fn test_memory_limits_default() {
        let limits = MemoryLimits::default();
        assert_eq!(limits.max_memory_mb, 1024);
        assert_eq!(limits.threshold_memory_mb, 768);
        assert_eq!(limits.max_context_tokens, 8192);
        assert!(limits.enabled);
        assert_eq!(limits.check_interval_secs, 30);
    }
    
    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.current_usage_bytes, 0);
        assert_eq!(stats.peak_usage_bytes, 0);
        assert_eq!(stats.total_memory_bytes, 0);
        assert_eq!(stats.gc_count, 0);
        assert_eq!(stats.current_context_tokens, 0);
        assert!(stats.last_gc_time.is_none());
    }
    
    #[test]
    fn test_memory_manager_cleaner_registration() {
        let manager = MemoryManager::new();
        let called = Arc::new(AtomicBool::new(false));
        
        {
            let called = called.clone();
            manager.register_cleaner(move || {
                called.store(true, Ordering::SeqCst);
            });
        }
        
        manager.force_gc(false);
        assert!(called.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_memory_manager_update_limits() {
        let manager = MemoryManager::new();
        
        let new_limits = MemoryLimits {
            max_memory_mb: 2048,
            threshold_memory_mb: 1536,
            max_context_tokens: 16384,
            enabled: false,
            check_interval_secs: 60,
        };
        
        manager.update_limits(new_limits.clone());
        let limits = manager.get_limits();
        
        assert_eq!(limits.max_memory_mb, 2048);
        assert_eq!(limits.threshold_memory_mb, 1536);
        assert_eq!(limits.max_context_tokens, 16384);
        assert!(!limits.enabled);
        assert_eq!(limits.check_interval_secs, 60);
    }
    
    #[test]
    fn test_memory_manager_context_tokens() {
        let manager = MemoryManager::new();
        
        manager.update_context_tokens(1000);
        let stats = manager.get_stats();
        
        assert_eq!(stats.current_context_tokens, 1000);
    }
}