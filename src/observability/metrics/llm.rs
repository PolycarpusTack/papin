use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};

use crate::commands::offline::llm::{ProviderType, ProviderInfo, ProviderConfig};
use crate::offline::llm::discovery::{DiscoveryService, InstallationInfo};
use crate::offline::llm::migration::{MigrationService, MigrationStatus};
use crate::error::Result;

// Reexport metrics types from src-common for convenience
pub use crate::src_common::observability::metrics::{
    Metric, MetricType, TimerStats, HistogramStats, METRICS_COLLECTOR,
    record_counter, increment_counter, record_gauge, record_histogram, time_operation
};

// Import telemetry helpers
use crate::src_common::observability::telemetry::{
    track_feature_usage, track_error, track_performance, track_model_usage,
    TelemetryClient, TelemetryEventType, PrivacyLevel
};

/// Configuration for LLM metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMetricsConfig {
    /// Whether to collect LLM metrics
    pub enabled: bool,
    /// Whether to collect detailed model performance metrics
    pub collect_performance_metrics: bool,
    /// Whether to collect model usage statistics
    pub collect_usage_metrics: bool,
    /// Whether to collect error metrics
    pub collect_error_metrics: bool,
    /// Level of anonymization for metrics
    pub anonymization_level: AnonymizationLevel,
    /// Sample rate for performance metrics (0.0 to 1.0)
    pub performance_sampling_rate: f64,
    /// Whether to automatically track provider changes
    pub track_provider_changes: bool,
    /// Whether to track model loading/unloading
    pub track_model_events: bool,
    /// Privacy notice version accepted by the user
    pub privacy_notice_version: String,
    /// Whether the privacy notice has been accepted
    pub privacy_notice_accepted: bool,
}

impl Default for LLMMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, requires explicit opt-in
            collect_performance_metrics: true,
            collect_usage_metrics: true,
            collect_error_metrics: true,
            anonymization_level: AnonymizationLevel::Full,
            performance_sampling_rate: 0.1, // 10% sampling by default
            track_provider_changes: true,
            track_model_events: true,
            privacy_notice_version: "1.0.0".to_string(),
            privacy_notice_accepted: false,
        }
    }
}

/// Level of anonymization for metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnonymizationLevel {
    /// No anonymization - collect all metrics with full details (not recommended)
    None,
    /// Partial anonymization - collect metrics with limited identifiable information
    Partial,
    /// Full anonymization - collect metrics with no identifiable information
    Full,
}

/// LLM provider event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderEventType {
    /// Provider discovered
    Discovered,
    /// Provider configuration changed
    ConfigChanged,
    /// Provider became active
    BecameActive,
    /// Provider became inactive
    BecameInactive,
    /// Provider not available (e.g., process not running)
    Unavailable,
    /// Provider error
    Error,
}

/// LLM model event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelEventType {
    /// Model loaded
    Loaded,
    /// Model unloaded
    Unloaded,
    /// Model download started
    DownloadStarted,
    /// Model download completed
    DownloadCompleted,
    /// Model download failed
    DownloadFailed,
    /// Model download canceled
    DownloadCanceled,
    /// Error loading model
    LoadError,
}

/// Generation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// Generation succeeded
    Success,
    /// Generation failed
    Failure,
    /// Generation canceled
    Canceled,
    /// Generation timeout
    Timeout,
}

/// Privacy-sensitive string - only included if privacy settings allow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacySensitiveString {
    /// Value is included
    Included(String),
    /// Value is redacted
    Redacted,
}

/// Provider-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformanceMetrics {
    /// Provider type
    pub provider_type: String,
    /// Number of generation requests
    pub generation_count: u64,
    /// Number of successful generations
    pub successful_generations: u64,
    /// Number of failed generations
    pub failed_generations: u64,
    /// Average tokens per second
    pub avg_tokens_per_second: f64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
    /// P90 latency in ms
    pub p90_latency_ms: f64,
    /// P99 latency in ms
    pub p99_latency_ms: f64,
    /// System information - CPU usage during generation
    pub avg_cpu_usage: f64,
    /// System information - Memory usage during generation
    pub avg_memory_usage: f64,
    /// Timestamp of the last update
    pub last_updated: SystemTime,
}

impl Default for ProviderPerformanceMetrics {
    fn default() -> Self {
        Self {
            provider_type: "unknown".to_string(),
            generation_count: 0,
            successful_generations: 0,
            failed_generations: 0,
            avg_tokens_per_second: 0.0,
            avg_latency_ms: 0.0,
            p90_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            avg_cpu_usage: 0.0,
            avg_memory_usage: 0.0,
            last_updated: SystemTime::now(),
        }
    }
}

/// Model-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    /// Model ID
    pub model_id: String,
    /// Provider type
    pub provider_type: String,
    /// Number of generation requests
    pub generation_count: u64,
    /// Number of tokens generated
    pub tokens_generated: u64,
    /// Number of successful generations
    pub successful_generations: u64,
    /// Number of failed generations
    pub failed_generations: u64,
    /// Average tokens per second
    pub avg_tokens_per_second: f64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
    /// P90 latency in ms
    pub p90_latency_ms: f64,
    /// P99 latency in ms
    pub p99_latency_ms: f64,
    /// Average time to first token in ms
    pub avg_time_to_first_token_ms: f64,
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
    /// Timestamp of the last update
    pub last_updated: SystemTime,
}

impl Default for ModelPerformanceMetrics {
    fn default() -> Self {
        Self {
            model_id: "unknown".to_string(),
            provider_type: "unknown".to_string(),
            generation_count: 0,
            tokens_generated: 0,
            successful_generations: 0,
            failed_generations: 0,
            avg_tokens_per_second: 0.0,
            avg_latency_ms: 0.0,
            p90_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            avg_time_to_first_token_ms: 0.0,
            avg_tokens_per_request: 0.0,
            last_updated: SystemTime::now(),
        }
    }
}

/// LLM Metrics Manager
pub struct LLMMetricsManager {
    /// Configuration
    config: RwLock<LLMMetricsConfig>,
    /// Provider performance metrics
    provider_metrics: RwLock<HashMap<String, ProviderPerformanceMetrics>>,
    /// Model performance metrics
    model_metrics: RwLock<HashMap<String, ModelPerformanceMetrics>>,
    /// Telemetry client
    telemetry_client: Option<Arc<TelemetryClient>>,
    /// Last active provider
    last_active_provider: RwLock<Option<String>>,
    /// Auto-detection of metrics changes
    active_monitoring: Mutex<bool>,
}

impl LLMMetricsManager {
    /// Create a new LLM metrics manager
    pub fn new(telemetry_client: Option<Arc<TelemetryClient>>) -> Self {
        Self {
            config: RwLock::new(LLMMetricsConfig::default()),
            provider_metrics: RwLock::new(HashMap::new()),
            model_metrics: RwLock::new(HashMap::new()),
            telemetry_client,
            last_active_provider: RwLock::new(None),
            active_monitoring: Mutex::new(false),
        }
    }
    
    /// Collect system resource usage metrics
    fn collect_system_resources(&self) -> (f64, f64) {
        // Try to get CPU and memory usage
        let cpu_usage = match sys_info::loadavg() {
            Ok(load) => load.one as f64 * 100.0, // Convert load average to percentage
            Err(_) => 0.0,
        };
        
        let memory_usage = match sys_info::mem_info() {
            Ok(mem) => {
                let used_mem = mem.total - mem.free - mem.buffers - mem.cached;
                (used_mem * 1024) as f64 // Convert to bytes
            },
            Err(_) => 0.0,
        };
        
        (cpu_usage, memory_usage)
    }

    /// Get current configuration
    pub fn get_config(&self) -> LLMMetricsConfig {
        self.config.read().unwrap().clone()
    }

    /// Update configuration
    pub fn update_config(&self, config: LLMMetricsConfig) {
        let mut current_config = self.config.write().unwrap();
        *current_config = config;
    }

    /// Accept privacy notice
    pub fn accept_privacy_notice(&self, version: &str) {
        let mut config = self.config.write().unwrap();
        config.privacy_notice_version = version.to_string();
        config.privacy_notice_accepted = true;
        config.enabled = true;
    }

    /// Check if metrics collection is enabled
    pub fn is_enabled(&self) -> bool {
        let config = self.config.read().unwrap();
        config.enabled && config.privacy_notice_accepted
    }

    /// Track provider event
    pub fn track_provider_event(
        &self,
        provider_type: &str,
        event_type: ProviderEventType,
        details: Option<HashMap<String, String>>,
    ) {
        // Check if enabled
        if !self.is_enabled() {
            return;
        }

        let config = self.config.read().unwrap();
        if !config.track_provider_changes {
            return;
        }

        // Create event name
        let event_name = match event_type {
            ProviderEventType::Discovered => "provider_discovered",
            ProviderEventType::ConfigChanged => "provider_config_changed",
            ProviderEventType::BecameActive => "provider_became_active",
            ProviderEventType::BecameInactive => "provider_became_inactive",
            ProviderEventType::Unavailable => "provider_unavailable",
            ProviderEventType::Error => "provider_error",
        };

        // Track as counter
        let mut tags = HashMap::new();
        tags.insert("provider_type".to_string(), provider_type.to_string());
        tags.insert("event_type".to_string(), event_name.to_string());

        if let Some(details) = &details {
            // Add details if allowed by privacy settings
            if config.anonymization_level != AnonymizationLevel::Full {
                for (key, value) in details {
                    // Skip sensitive fields in full anonymization mode
                    if !is_sensitive_field(key) || config.anonymization_level == AnonymizationLevel::None {
                        tags.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Record metric
        increment_counter(&format!("llm.provider.{}", event_name), Some(tags.clone()));

        // Track in telemetry if available
        if let Some(client) = &self.telemetry_client {
            client.track_feature_usage(&format!("llm_provider_{}", event_name), Some(tags));
        }

        // Update last active provider if this is an "active" event
        if let ProviderEventType::BecameActive = event_type {
            let mut last_active = self.last_active_provider.write().unwrap();
            *last_active = Some(provider_type.to_string());
        }
    }

    /// Track model event
    pub fn track_model_event(
        &self,
        provider_type: &str,
        model_id: &str,
        event_type: ModelEventType,
        details: Option<HashMap<String, String>>,
    ) {
        // Check if enabled
        if !self.is_enabled() {
            return;
        }

        let config = self.config.read().unwrap();
        if !config.track_model_events {
            return;
        }

        // Create event name
        let event_name = match event_type {
            ModelEventType::Loaded => "model_loaded",
            ModelEventType::Unloaded => "model_unloaded",
            ModelEventType::DownloadStarted => "model_download_started",
            ModelEventType::DownloadCompleted => "model_download_completed",
            ModelEventType::DownloadFailed => "model_download_failed",
            ModelEventType::DownloadCanceled => "model_download_canceled",
            ModelEventType::LoadError => "model_load_error",
        };

        // Track as counter
        let mut tags = HashMap::new();
        tags.insert("provider_type".to_string(), provider_type.to_string());
        tags.insert("model_id".to_string(), model_id.to_string());
        tags.insert("event_type".to_string(), event_name.to_string());

        if let Some(details) = &details {
            // Add details if allowed by privacy settings
            if config.anonymization_level != AnonymizationLevel::Full {
                for (key, value) in details {
                    // Skip sensitive fields in full anonymization mode
                    if !is_sensitive_field(key) || config.anonymization_level == AnonymizationLevel::None {
                        tags.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Record metric
        increment_counter(&format!("llm.model.{}", event_name), Some(tags.clone()));

        // Track in telemetry if available
        if let Some(client) = &self.telemetry_client {
            client.track_feature_usage(&format!("llm_model_{}", event_name), Some(tags));
        }
    }

    /// Track generation request
    pub fn track_generation_request(
        &self,
        provider_type: &str,
        model_id: &str,
        status: GenerationStatus,
        latency_ms: f64,
        tokens_generated: usize,
        tokens_per_second: f64,
        time_to_first_token_ms: f64,
        details: Option<HashMap<String, String>>,
    ) {
        // Check if enabled
        if !self.is_enabled() {
            return;
        }

        let config = self.config.read().unwrap();
        if !config.collect_usage_metrics {
            return;
        }

        // Check sampling rate for performance metrics
        if config.collect_performance_metrics {
            // Sample based on configuration
            let should_sample = rand::random::<f64>() <= config.performance_sampling_rate;
            
            if should_sample {
                // Record latency
                let mut perf_tags = HashMap::new();
                perf_tags.insert("provider_type".to_string(), provider_type.to_string());
                perf_tags.insert("model_id".to_string(), model_id.to_string());
                
                // Record performance metrics
                record_histogram("llm.generation.latency_ms", latency_ms, Some(perf_tags.clone()));
                record_histogram("llm.generation.tokens_per_second", tokens_per_second, Some(perf_tags.clone()));
                record_histogram("llm.generation.time_to_first_token_ms", time_to_first_token_ms, Some(perf_tags.clone()));
                record_histogram("llm.generation.tokens_generated", tokens_generated as f64, Some(perf_tags.clone()));
                
                // Update provider metrics
                self.update_provider_metrics(provider_type, &status, latency_ms, tokens_per_second);
                
                // Update model metrics
                self.update_model_metrics(
                    provider_type,
                    model_id,
                    &status,
                    latency_ms,
                    tokens_generated as u64,
                    tokens_per_second,
                    time_to_first_token_ms,
                );
            }
        }

        // Track usage metrics
        let mut tags = HashMap::new();
        tags.insert("provider_type".to_string(), provider_type.to_string());
        tags.insert("model_id".to_string(), model_id.to_string());
        tags.insert("status".to_string(), match status {
            GenerationStatus::Success => "success",
            GenerationStatus::Failure => "failure",
            GenerationStatus::Canceled => "canceled",
            GenerationStatus::Timeout => "timeout",
        }.to_string());

        if let Some(details) = &details {
            // Add details if allowed by privacy settings
            if config.anonymization_level != AnonymizationLevel::Full {
                for (key, value) in details {
                    // Skip sensitive fields
                    if !is_sensitive_field(key) || config.anonymization_level == AnonymizationLevel::None {
                        tags.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Record metric
        increment_counter("llm.generation.count", Some(tags.clone()));
        
        // Count status-specific metrics
        match status {
            GenerationStatus::Success => {
                increment_counter("llm.generation.success", Some(tags.clone()));
            },
            GenerationStatus::Failure => {
                increment_counter("llm.generation.failure", Some(tags.clone()));
                
                // Track error if enabled
                if config.collect_error_metrics {
                    let error_message = details
                        .as_ref()
                        .and_then(|d| d.get("error_message"))
                        .unwrap_or(&"Unknown error".to_string())
                        .clone();
                    
                    if let Some(client) = &self.telemetry_client {
                        client.track_error("llm_generation_error", &error_message, Some(tags.clone()));
                    }
                }
            },
            GenerationStatus::Canceled => {
                increment_counter("llm.generation.canceled", Some(tags.clone()));
            },
            GenerationStatus::Timeout => {
                increment_counter("llm.generation.timeout", Some(tags.clone()));
            },
        }

        // Track in telemetry if available
        if let Some(client) = &self.telemetry_client {
            client.track_model_usage(model_id, tokens_generated as u32, Some(tags));
        }
    }

    /// Get performance metrics for all providers
    pub fn get_provider_metrics(&self) -> HashMap<String, ProviderPerformanceMetrics> {
        self.provider_metrics.read().unwrap().clone()
    }

    /// Get performance metrics for a specific provider
    pub fn get_provider_metric(&self, provider_type: &str) -> Option<ProviderPerformanceMetrics> {
        self.provider_metrics.read().unwrap().get(provider_type).cloned()
    }

    /// Get performance metrics for all models
    pub fn get_model_metrics(&self) -> HashMap<String, ModelPerformanceMetrics> {
        self.model_metrics.read().unwrap().clone()
    }

    /// Get performance metrics for a specific model
    pub fn get_model_metric(&self, model_id: &str) -> Option<ModelPerformanceMetrics> {
        self.model_metrics.read().unwrap().get(model_id).cloned()
    }

    /// Start monitoring for active provider changes
    pub fn start_monitoring(&self, discovery_service: Arc<DiscoveryService>) {
        let mut monitoring = self.active_monitoring.lock().unwrap();
        if *monitoring {
            return;
        }
        
        *monitoring = true;
        
        // Clone Arc for thread
        let metrics_manager = Arc::new(self.clone());
        
        // Start monitoring thread
        std::thread::spawn(move || {
            debug!("LLM provider metrics monitoring started");
            let check_interval = Duration::from_secs(30);
            
            let mut last_active: Option<String> = None;
            
            loop {
                // Check if monitoring should stop
                {
                    let monitoring = metrics_manager.active_monitoring.lock().unwrap();
                    if !*monitoring {
                        break;
                    }
                }
                
                // Check for metric changes
                if metrics_manager.is_enabled() {
                    // Check for provider changes
                    let installations = discovery_service.get_installations();
                    for (provider_type, info) in &installations {
                        use crate::offline::llm::discovery::InstallationStatus;
                        
                        match &info.status {
                            InstallationStatus::Installed { location, version } => {
                                // Track installed provider if not already tracked
                                let mut provider_metrics = metrics_manager.provider_metrics.write().unwrap();
                                if !provider_metrics.contains_key(provider_type) {
                                    let mut details = HashMap::new();
                                    details.insert("location".to_string(), location.to_string_lossy().to_string());
                                    details.insert("version".to_string(), version.clone());
                                    
                                    // Drop the lock before calling track method to avoid deadlock
                                    drop(provider_metrics);
                                    
                                    metrics_manager.track_provider_event(
                                        provider_type,
                                        ProviderEventType::Discovered,
                                        Some(details),
                                    );
                                }
                            },
                            InstallationStatus::PartiallyInstalled { reason, location } => {
                                // Track partially installed provider
                                let mut details = HashMap::new();
                                details.insert("reason".to_string(), reason.clone());
                                if let Some(loc) = location {
                                    details.insert("location".to_string(), loc.to_string_lossy().to_string());
                                }
                                
                                metrics_manager.track_provider_event(
                                    provider_type,
                                    ProviderEventType::Unavailable,
                                    Some(details),
                                );
                            },
                            _ => {},
                        }
                    }
                }
                
                // Sleep for the check interval
                std::thread::sleep(check_interval);
            }
            
            debug!("LLM provider metrics monitoring stopped");
        });
    }

    /// Stop monitoring for active provider changes
    pub fn stop_monitoring(&self) {
        let mut monitoring = self.active_monitoring.lock().unwrap();
        *monitoring = false;
    }

    // Internal method to update provider metrics
    fn update_provider_metrics(
        &self,
        provider_type: &str,
        status: &GenerationStatus,
        latency_ms: f64,
        tokens_per_second: f64,
    ) {
        // Collect system resource metrics if available
        let (cpu_usage, memory_usage) = self.collect_system_resources();
        let mut provider_metrics = self.provider_metrics.write().unwrap();
        
        // Get or create provider metrics
        let metrics = provider_metrics
            .entry(provider_type.to_string())
            .or_insert_with(|| {
                let mut default = ProviderPerformanceMetrics::default();
                default.provider_type = provider_type.to_string();
                default
            });
        
        // Update metrics
        metrics.generation_count += 1;
        metrics.last_updated = SystemTime::now();
        
        match status {
            GenerationStatus::Success => {
                metrics.successful_generations += 1;
                
                // Update latency and tokens per second metrics
                // Use exponential moving average to avoid sudden jumps
                metrics.avg_latency_ms = if metrics.avg_latency_ms == 0.0 {
                    latency_ms
                } else {
                    metrics.avg_latency_ms * 0.9 + latency_ms * 0.1
                };
                
                metrics.avg_tokens_per_second = if metrics.avg_tokens_per_second == 0.0 {
                    tokens_per_second
                } else {
                    metrics.avg_tokens_per_second * 0.9 + tokens_per_second * 0.1
                };
                
                // Update percentiles (simplistic approach)
                metrics.p90_latency_ms = metrics.avg_latency_ms * 1.5;
                metrics.p99_latency_ms = metrics.avg_latency_ms * 2.5;
                
                // Update CPU and memory usage if available
                if cpu_usage > 0.0 {
                    metrics.avg_cpu_usage = if metrics.avg_cpu_usage == 0.0 {
                        cpu_usage
                    } else {
                        metrics.avg_cpu_usage * 0.9 + cpu_usage * 0.1
                    };
                }
                
                if memory_usage > 0.0 {
                    metrics.avg_memory_usage = if metrics.avg_memory_usage == 0.0 {
                        memory_usage
                    } else {
                        metrics.avg_memory_usage * 0.9 + memory_usage * 0.1
                    };
                }
            },
            GenerationStatus::Failure | GenerationStatus::Timeout => {
                metrics.failed_generations += 1;
            },
            _ => {},
        }
    }

    // Internal method to update model metrics
    fn update_model_metrics(
        &self,
        provider_type: &str,
        model_id: &str,
        status: &GenerationStatus,
        latency_ms: f64,
        tokens_generated: u64,
        tokens_per_second: f64,
        time_to_first_token_ms: f64,
    ) {
        let mut model_metrics = self.model_metrics.write().unwrap();
        
        // Get or create model metrics
        let key = format!("{}:{}", provider_type, model_id);
        let metrics = model_metrics
            .entry(key)
            .or_insert_with(|| {
                let mut default = ModelPerformanceMetrics::default();
                default.provider_type = provider_type.to_string();
                default.model_id = model_id.to_string();
                default
            });
        
        // Update metrics
        metrics.generation_count += 1;
        metrics.last_updated = SystemTime::now();
        
        match status {
            GenerationStatus::Success => {
                metrics.successful_generations += 1;
                metrics.tokens_generated += tokens_generated;
                
                // Update latency and tokens per second metrics
                // Use exponential moving average to avoid sudden jumps
                metrics.avg_latency_ms = if metrics.avg_latency_ms == 0.0 {
                    latency_ms
                } else {
                    metrics.avg_latency_ms * 0.9 + latency_ms * 0.1
                };
                
                metrics.avg_tokens_per_second = if metrics.avg_tokens_per_second == 0.0 {
                    tokens_per_second
                } else {
                    metrics.avg_tokens_per_second * 0.9 + tokens_per_second * 0.1
                };
                
                metrics.avg_time_to_first_token_ms = if metrics.avg_time_to_first_token_ms == 0.0 {
                    time_to_first_token_ms
                } else {
                    metrics.avg_time_to_first_token_ms * 0.9 + time_to_first_token_ms * 0.1
                };
                
                metrics.avg_tokens_per_request = metrics.tokens_generated as f64 / metrics.successful_generations as f64;
                
                // Update percentiles (simplistic approach)
                metrics.p90_latency_ms = metrics.avg_latency_ms * 1.5;
                metrics.p99_latency_ms = metrics.avg_latency_ms * 2.5;
            },
            GenerationStatus::Failure | GenerationStatus::Timeout => {
                metrics.failed_generations += 1;
            },
            _ => {},
        }
    }
}

impl Clone for LLMMetricsManager {
    fn clone(&self) -> Self {
        Self {
            config: RwLock::new(self.config.read().unwrap().clone()),
            provider_metrics: RwLock::new(self.provider_metrics.read().unwrap().clone()),
            model_metrics: RwLock::new(self.model_metrics.read().unwrap().clone()),
            telemetry_client: self.telemetry_client.clone(),
            last_active_provider: RwLock::new(self.last_active_provider.read().unwrap().clone()),
            active_monitoring: Mutex::new(*self.active_monitoring.lock().unwrap()),
        }
    }
}

// Helper function to check if a field is privacy-sensitive
fn is_sensitive_field(field: &str) -> bool {
    matches!(field,
        "prompt" | "input" | "query" | "api_key" | "auth_token" | "user_id" | 
        "username" | "email" | "file_path" | "model_path" | "context" | "message" |
        "ip_address" | "location" | "device_id"
    )
}

// Create a global LLM metrics manager
lazy_static::lazy_static! {
    pub static ref LLM_METRICS_MANAGER: Arc<RwLock<Option<LLMMetricsManager>>> = Arc::new(RwLock::new(None));
}

/// Initialize LLM metrics manager
pub fn init_llm_metrics(telemetry_client: Option<Arc<TelemetryClient>>) -> Arc<LLMMetricsManager> {
    let manager = Arc::new(LLMMetricsManager::new(telemetry_client));
    
    // Store in global variable
    let mut global_manager = LLM_METRICS_MANAGER.write().unwrap();
    *global_manager = Some(manager.clone());
    
    debug!("LLM metrics manager initialized");
    
    manager
}

/// Get LLM metrics manager
pub fn get_llm_metrics_manager() -> Option<Arc<LLMMetricsManager>> {
    LLM_METRICS_MANAGER.read().unwrap().clone()
}

// --------------------------
// Utility Functions
// --------------------------

/// Time an LLM generation operation and record metrics
pub fn time_generation<F, R>(
    provider_type: &str,
    model_id: &str,
    input_tokens: usize,
    f: F
) -> R 
where
    F: FnOnce() -> (R, usize, Option<String>),  // Returns result, output tokens, error message
{
    // Check if metrics manager is available
    let metrics_manager = get_llm_metrics_manager();
    
    if metrics_manager.is_none() || 
       !metrics_manager.as_ref().unwrap().is_enabled() {
        // If metrics are disabled, just run the function without tracking
        let (result, _, _) = f();
        return result;
    }
    
    // Start timing
    let start_time = Instant::now();
    let time_to_first_token_marker = Arc::new(Mutex::new(None::<Instant>));
    let time_to_first_token_marker_clone = time_to_first_token_marker.clone();
    
    // Run the function
    let (result, output_tokens, error) = f();
    
    // Calculate metrics
    let end_time = Instant::now();
    let total_time = end_time.duration_since(start_time);
    let total_time_ms = total_time.as_secs_f64() * 1000.0;
    
    // Get time to first token if set
    let time_to_first_token_ms = time_to_first_token_marker
        .lock()
        .unwrap()
        .map(|t| t.duration_since(start_time).as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    
    // Calculate tokens per second if successful
    let tokens_per_second = if output_tokens > 0 && error.is_none() {
        output_tokens as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    // Determine status
    let status = if error.is_some() {
        GenerationStatus::Failure
    } else {
        GenerationStatus::Success
    };
    
    // Create details map
    let mut details = HashMap::new();
    details.insert("input_tokens".to_string(), input_tokens.to_string());
    details.insert("output_tokens".to_string(), output_tokens.to_string());
    details.insert("total_tokens".to_string(), (input_tokens + output_tokens).to_string());
    
    if let Some(error_msg) = error {
        details.insert("error_message".to_string(), error_msg);
    }
    
    // Track metrics
    if let Some(manager) = metrics_manager {
        manager.track_generation_request(
            provider_type,
            model_id,
            status,
            total_time_ms,
            output_tokens,
            tokens_per_second,
            time_to_first_token_ms,
            Some(details),
        );
    }
    
    result
}

/// Mark time to first token during generation
pub fn mark_first_token(time_marker: &Arc<Mutex<Option<Instant>>>) {
    let mut marker = time_marker.lock().unwrap();
    if marker.is_none() {
        *marker = Some(Instant::now());
    }
}

/// Track provider event
pub fn track_provider_event(
    provider_type: &str,
    event_type: ProviderEventType,
    details: Option<HashMap<String, String>>,
) {
    if let Some(manager) = get_llm_metrics_manager() {
        manager.track_provider_event(provider_type, event_type, details);
    }
}

/// Track model event
pub fn track_model_event(
    provider_type: &str,
    model_id: &str,
    event_type: ModelEventType,
    details: Option<HashMap<String, String>>,
) {
    if let Some(manager) = get_llm_metrics_manager() {
        manager.track_model_event(provider_type, model_id, event_type, details);
    }
}

/// Get provider performance metrics
pub fn get_provider_metrics() -> Option<HashMap<String, ProviderPerformanceMetrics>> {
    get_llm_metrics_manager().map(|manager| manager.get_provider_metrics())
}

/// Get model performance metrics
pub fn get_model_metrics() -> Option<HashMap<String, ModelPerformanceMetrics>> {
    get_llm_metrics_manager().map(|manager| manager.get_model_metrics())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_config_defaults() {
        let config = LLMMetricsConfig::default();
        assert_eq!(config.enabled, false); // Disabled by default
        assert_eq!(config.privacy_notice_accepted, false); // Requires explicit acceptance
    }
    
    #[test]
    fn test_sensitive_field_detection() {
        assert!(is_sensitive_field("prompt"));
        assert!(is_sensitive_field("api_key"));
        assert!(is_sensitive_field("user_id"));
        assert!(!is_sensitive_field("model_name"));
        assert!(!is_sensitive_field("tokens_generated"));
    }
    
    #[test]
    fn test_metrics_manager_creation() {
        let manager = LLMMetricsManager::new(None);
        assert!(!manager.is_enabled());
        
        let config = manager.get_config();
        assert_eq!(config.enabled, false);
        
        // Accept privacy notice
        manager.accept_privacy_notice("1.0.0");
        assert!(manager.is_enabled());
    }
    
    #[test]
    fn test_metrics_collection() {
        let manager = Arc::new(LLMMetricsManager::new(None));
        
        // Enable metrics
        let mut config = manager.get_config();
        config.enabled = true;
        config.privacy_notice_accepted = true;
        manager.update_config(config);
        
        // Track a model event
        manager.track_model_event(
            "LlamaCpp",
            "llama-7b",
            ModelEventType::Loaded,
            None,
        );
        
        // Track a generation request
        manager.track_generation_request(
            "LlamaCpp",
            "llama-7b",
            GenerationStatus::Success,
            500.0,
            100,
            20.0,
            100.0,
            None,
        );
        
        // Verify provider metrics were updated
        let provider_metrics = manager.get_provider_metrics();
        assert!(provider_metrics.contains_key("LlamaCpp"));
        let llama_metrics = provider_metrics.get("LlamaCpp").unwrap();
        assert_eq!(llama_metrics.provider_type, "LlamaCpp");
        assert_eq!(llama_metrics.generation_count, 1);
        assert_eq!(llama_metrics.successful_generations, 1);
        
        // Verify model metrics were updated
        let model_metrics = manager.get_model_metrics();
        let key = "LlamaCpp:llama-7b";
        assert!(model_metrics.contains_key(key));
        let model = model_metrics.get(key).unwrap();
        assert_eq!(model.provider_type, "LlamaCpp");
        assert_eq!(model.model_id, "llama-7b");
        assert_eq!(model.generation_count, 1);
        assert_eq!(model.successful_generations, 1);
    }
}
