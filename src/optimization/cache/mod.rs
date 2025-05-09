use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use lru::LruCache;
use log::{debug, info, trace, warn};
use serde::{Serialize, Deserialize};
use tokio::time::{interval, sleep};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Time-to-live for cache entries in seconds
    pub ttl_seconds: u64,
    /// Whether to persist the cache to disk
    pub persist: bool,
    /// Path to the cache file
    pub cache_file: Option<String>,
    /// Whether the cache is enabled
    pub enabled: bool,
    /// Interval in seconds for cache cleanup
    pub cleanup_interval_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 3600, // 1 hour
            persist: true,
            cache_file: None,
            enabled: true,
            cleanup_interval_secs: 300, // 5 minutes
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    /// Cached value
    value: T,
    /// When the entry was created
    created_at: u64,
    /// When the entry expires
    expires_at: u64,
    /// Number of hits
    hits: u64,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Total number of cache evictions
    pub evictions: u64,
    /// Total number of cache invalidations
    pub invalidations: u64,
    /// Current number of entries in the cache
    pub size: usize,
    /// Maximum capacity of the cache
    pub capacity: usize,
    /// Hit ratio (hits / (hits + misses))
    pub hit_ratio: f64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            invalidations: 0,
            size: 0,
            capacity: 0,
            hit_ratio: 0.0,
        }
    }
}

/// Cache with TTL and statistics
pub struct Cache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    config: Arc<Mutex<CacheConfig>>,
    stats: Arc<Mutex<CacheStats>>,
    running: Arc<Mutex<bool>>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Create a new cache with the specified configuration
    pub fn new(config: CacheConfig) -> Self {
        let cache = Arc::new(RwLock::new(LruCache::new(config.max_entries)));
        let stats = Arc::new(Mutex::new(CacheStats {
            capacity: config.max_entries,
            ..Default::default()
        }));
        
        let instance = Self {
            cache,
            config: Arc::new(Mutex::new(config.clone())),
            stats,
            running: Arc::new(Mutex::new(false)),
        };
        
        // Load from disk if persistence is enabled
        if config.persist && config.cache_file.is_some() {
            instance.load_from_disk();
        }
        
        instance
    }
    
    /// Start the cache cleanup background task
    pub fn start_cleanup(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        
        let cache = self.cache.clone();
        let config = self.config.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let cleanup_interval = {
                let config = config.lock().unwrap();
                Duration::from_secs(config.cleanup_interval_secs)
            };
            
            let mut interval = interval(cleanup_interval);
            
            while *running.lock().unwrap() {
                interval.tick().await;
                
                // Skip if cache is disabled
                if !config.lock().unwrap().enabled {
                    continue;
                }
                
                // Perform cleanup
                Self::cleanup_expired_entries(&cache, &stats);
                
                // Save to disk if persistence is enabled
                let should_persist = {
                    let config = config.lock().unwrap();
                    config.persist && config.cache_file.is_some()
                };
                
                if should_persist {
                    Self::save_to_disk(&cache, &config);
                }
            }
        });
    }
    
    /// Stop the cache cleanup background task
    pub fn stop_cleanup(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
    
    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        if !self.config.lock().unwrap().enabled {
            let mut stats = self.stats.lock().unwrap();
            stats.misses += 1;
            Self::update_hit_ratio(&mut stats);
            return None;
        }
        
        let now = Self::now();
        let mut cache = self.cache.write().unwrap();
        
        if let Some(entry) = cache.get_mut(key) {
            if entry.expires_at > now {
                // Cache hit
                entry.hits += 1;
                
                let mut stats = self.stats.lock().unwrap();
                stats.hits += 1;
                Self::update_hit_ratio(&mut stats);
                
                return Some(entry.value.clone());
            } else {
                // Entry expired, remove it
                cache.pop(key);
                
                let mut stats = self.stats.lock().unwrap();
                stats.evictions += 1;
                stats.misses += 1;
                stats.size = cache.len();
                Self::update_hit_ratio(&mut stats);
            }
        } else {
            // Cache miss
            let mut stats = self.stats.lock().unwrap();
            stats.misses += 1;
            Self::update_hit_ratio(&mut stats);
        }
        
        None
    }
    
    /// Put a value in the cache
    pub fn put(&self, key: K, value: V) {
        if !self.config.lock().unwrap().enabled {
            return;
        }
        
        let now = Self::now();
        let config = self.config.lock().unwrap();
        let ttl = config.ttl_seconds;
        
        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at: now + ttl,
            hits: 0,
        };
        
        let mut cache = self.cache.write().unwrap();
        let old_len = cache.len();
        cache.put(key, entry);
        
        if cache.len() <= old_len && old_len > 0 {
            // An entry was evicted
            let mut stats = self.stats.lock().unwrap();
            stats.evictions += 1;
        }
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.size = cache.len();
    }
    
    /// Put a value in the cache with a custom TTL
    pub fn put_with_ttl(&self, key: K, value: V, ttl_seconds: u64) {
        if !self.config.lock().unwrap().enabled {
            return;
        }
        
        let now = Self::now();
        
        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at: now + ttl_seconds,
            hits: 0,
        };
        
        let mut cache = self.cache.write().unwrap();
        let old_len = cache.len();
        cache.put(key, entry);
        
        if cache.len() <= old_len && old_len > 0 {
            // An entry was evicted
            let mut stats = self.stats.lock().unwrap();
            stats.evictions += 1;
        }
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.size = cache.len();
    }
    
    /// Remove a value from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().unwrap();
        let entry = cache.pop(key);
        
        if entry.is_some() {
            let mut stats = self.stats.lock().unwrap();
            stats.invalidations += 1;
            stats.size = cache.len();
        }
        
        entry.map(|e| e.value)
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let old_size = cache.len();
        cache.clear();
        
        let mut stats = self.stats.lock().unwrap();
        stats.invalidations += old_size as u64;
        stats.size = 0;
    }
    
    /// Check if the cache contains a key
    pub fn contains(&self, key: &K) -> bool {
        if !self.config.lock().unwrap().enabled {
            return false;
        }
        
        let now = Self::now();
        let cache = self.cache.read().unwrap();
        
        if let Some(entry) = cache.peek(key) {
            entry.expires_at > now
        } else {
            false
        }
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Get cache configuration
    pub fn get_config(&self) -> CacheConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// Update cache configuration
    pub fn update_config(&self, config: CacheConfig) {
        let mut current_config = self.config.lock().unwrap();
        let old_max_entries = current_config.max_entries;
        
        // Update config
        *current_config = config;
        
        // Resize cache if necessary
        if current_config.max_entries != old_max_entries {
            let mut cache = self.cache.write().unwrap();
            cache.resize(current_config.max_entries);
            
            let mut stats = self.stats.lock().unwrap();
            stats.capacity = current_config.max_entries;
        }
    }
    
    /// Asynchronously compute a value if not in cache
    pub async fn get_or_compute<F, Fut>(&self, key: K, compute_fn: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = V>,
    {
        // Try to get from cache
        if let Some(value) = self.get(&key) {
            return value;
        }
        
        // Compute the value
        let value = compute_fn().await;
        
        // Store in cache
        self.put(key, value.clone());
        
        value
    }
    
    /// Get the current time in seconds
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
    }
    
    /// Cleanup expired entries
    fn cleanup_expired_entries(
        cache: &Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
        stats: &Arc<Mutex<CacheStats>>,
    ) {
        let now = Self::now();
        let mut expired_keys = Vec::new();
        
        // Find expired keys
        {
            let cache_read = cache.read().unwrap();
            for (key, entry) in cache_read.iter() {
                if entry.expires_at <= now {
                    expired_keys.push(key.clone());
                }
            }
        }
        
        // Remove expired keys
        if !expired_keys.is_empty() {
            let mut cache_write = cache.write().unwrap();
            for key in &expired_keys {
                cache_write.pop(key);
            }
            
            // Update stats
            let mut stats = stats.lock().unwrap();
            stats.evictions += expired_keys.len() as u64;
            stats.size = cache_write.len();
            
            debug!("Cleaned up {} expired cache entries", expired_keys.len());
        }
    }
    
    /// Save cache to disk
    fn save_to_disk(
        cache: &Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
        config: &Arc<Mutex<CacheConfig>>,
    ) {
        let config = config.lock().unwrap();
        if let Some(cache_file) = &config.cache_file {
            let cache_read = cache.read().unwrap();
            
            // Create a serializable representation
            let mut entries = HashMap::new();
            for (key, entry) in cache_read.iter() {
                entries.insert(key.clone(), entry.clone());
            }
            
            // Serialize to JSON
            match serde_json::to_string(&entries) {
                Ok(json) => {
                    // Create parent directories if needed
                    if let Some(parent) = std::path::Path::new(cache_file).parent() {
                        if !parent.exists() {
                            if let Err(e) = std::fs::create_dir_all(parent) {
                                warn!("Failed to create cache directory: {}", e);
                                return;
                            }
                        }
                    }
                    
                    // Write to file
                    if let Err(e) = std::fs::write(cache_file, json) {
                        warn!("Failed to save cache to disk: {}", e);
                    } else {
                        debug!("Cache saved to disk: {}", cache_file);
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize cache: {}", e);
                }
            }
        }
    }
    
    /// Load cache from disk
    fn load_from_disk(&self) {
        let config = self.config.lock().unwrap();
        if let Some(cache_file) = &config.cache_file {
            if std::path::Path::new(cache_file).exists() {
                match std::fs::read_to_string(cache_file) {
                    Ok(json) => {
                        match serde_json::from_str::<HashMap<K, CacheEntry<V>>>(&json) {
                            Ok(entries) => {
                                let now = Self::now();
                                let mut cache = self.cache.write().unwrap();
                                
                                // Reset cache
                                cache.clear();
                                
                                // Load valid entries
                                let mut loaded = 0;
                                let mut expired = 0;
                                
                                for (key, entry) in entries {
                                    if entry.expires_at > now {
                                        cache.put(key, entry);
                                        loaded += 1;
                                    } else {
                                        expired += 1;
                                    }
                                }
                                
                                // Update stats
                                let mut stats = self.stats.lock().unwrap();
                                stats.size = cache.len();
                                
                                info!("Loaded {} cache entries from disk ({} expired)", loaded, expired);
                            }
                            Err(e) => {
                                warn!("Failed to deserialize cache: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read cache file: {}", e);
                    }
                }
            }
        }
    }
    
    /// Update hit ratio
    fn update_hit_ratio(stats: &mut CacheStats) {
        let total = stats.hits + stats.misses;
        if total > 0 {
            stats.hit_ratio = stats.hits as f64 / total as f64;
        } else {
            stats.hit_ratio = 0.0;
        }
    }
}

impl<K, V> Drop for Cache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.stop_cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let config = CacheConfig::default();
        let cache = Cache::<String, String>::new(config);
        
        // Put and get
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        
        // Contains
        assert!(cache.contains(&"key1".to_string()));
        assert!(!cache.contains(&"key2".to_string()));
        
        // Remove
        assert_eq!(cache.remove(&"key1".to_string()), Some("value1".to_string()));
        assert!(!cache.contains(&"key1".to_string()));
        
        // Stats
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.invalidations, 1);
        assert_eq!(stats.size, 0);
    }
    
    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig {
            ttl_seconds: 1, // 1 second TTL
            ..Default::default()
        };
        
        let cache = Cache::<String, String>::new(config);
        
        // Put a value
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_secs(2));
        
        // Value should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
        
        // Stats should show a miss
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.evictions, 1);
    }
    
    #[test]
    fn test_cache_custom_ttl() {
        let config = CacheConfig {
            ttl_seconds: 10, // Default TTL
            ..Default::default()
        };
        
        let cache = Cache::<String, String>::new(config);
        
        // Put a value with custom TTL
        cache.put_with_ttl("key1".to_string(), "value1".to_string(), 1); // 1 second TTL
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_secs(2));
        
        // Value should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
    }
    
    #[test]
    fn test_cache_clear() {
        let config = CacheConfig::default();
        let cache = Cache::<String, String>::new(config);
        
        // Put multiple values
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());
        
        assert_eq!(cache.get_stats().size, 3);
        
        // Clear cache
        cache.clear();
        
        assert_eq!(cache.get_stats().size, 0);
        assert!(!cache.contains(&"key1".to_string()));
        assert!(!cache.contains(&"key2".to_string()));
        assert!(!cache.contains(&"key3".to_string()));
    }
    
    #[tokio::test]
    async fn test_cache_get_or_compute() {
        let config = CacheConfig::default();
        let cache = Cache::<String, String>::new(config);
        
        // Get or compute a value
        let value = cache.get_or_compute("key1".to_string(), || async {
            "computed".to_string()
        }).await;
        
        assert_eq!(value, "computed");
        
        // Second call should hit the cache
        let value = cache.get_or_compute("key1".to_string(), || async {
            "recomputed".to_string()
        }).await;
        
        assert_eq!(value, "computed"); // Still the original value
        
        // Stats should show 2 hits (1 from get_or_compute and 1 from the direct get inside)
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1); // Initial miss when computing
    }
}