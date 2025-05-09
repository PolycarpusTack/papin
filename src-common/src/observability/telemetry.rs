use std::sync::{Arc, Mutex, RwLock, Once};
use chrono::{DateTime, Utc};
use log::{debug, info, warn, error};
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use uuid::Uuid;
use std::io::ErrorKind;

use crate::observability::logging::LogEntry;
use crate::observability::metrics::Metric;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TelemetryEventType {
    ApplicationStart,
    ApplicationExit,
    FeatureUsage,
    Error,
    Performance,
    UserAction,
    SystemInfo,
    ModelUsage,
    Network,
}

impl TelemetryEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TelemetryEventType::ApplicationStart => "app_start",
            TelemetryEventType::ApplicationExit => "app_exit",
            TelemetryEventType::FeatureUsage => "feature_usage",
            TelemetryEventType::Error => "error",
            TelemetryEventType::Performance => "performance",
            TelemetryEventType::UserAction => "user_action",
            TelemetryEventType::SystemInfo => "system_info",
            TelemetryEventType::ModelUsage => "model_usage",
            TelemetryEventType::Network => "network",
        }
    }
    
    pub fn category(&self) -> &'static str {
        match self {
            TelemetryEventType::ApplicationStart | 
            TelemetryEventType::ApplicationExit => "app_lifecycle",
            TelemetryEventType::FeatureUsage => "feature_usage",
            TelemetryEventType::Error => "errors",
            TelemetryEventType::Performance => "performance",
            TelemetryEventType::UserAction => "user_actions",
            TelemetryEventType::SystemInfo => "system_info",
            TelemetryEventType::ModelUsage => "model_usage",
            TelemetryEventType::Network => "network",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: TelemetryEventType,
    pub name: String,
    pub value: Option<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryBatch {
    pub batch_id: String,
    pub client_id: String,
    pub app_version: String,
    pub platform: String,
    pub events: Vec<TelemetryEvent>,
    pub metrics: Vec<Metric>,
    pub logs: Vec<LogEntry>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub client_id: String,
    pub collection_categories: HashMap<String, bool>,
    pub batch_size: usize,
    pub batch_interval_seconds: u64,
    pub server_url: String,
    pub send_usage_statistics: bool,
    pub privacy_policy_version: String,
    pub privacy_policy_accepted: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, requires explicit opt-in
            client_id: Uuid::new_v4().to_string(),
            collection_categories: [
                ("app_lifecycle".to_string(), true),
                ("feature_usage".to_string(), false),
                ("errors".to_string(), true),
                ("performance".to_string(), false),
                ("user_actions".to_string(), false),
                ("system_info".to_string(), false),
                ("logs".to_string(), false),
                ("model_usage".to_string(), false),
                ("network".to_string(), false),
            ].iter().cloned().collect(),
            batch_size: 100,
            batch_interval_seconds: 60,
            server_url: "https://telemetry.mcp-client.example.com/v1/telemetry".to_string(),
            send_usage_statistics: false,
            privacy_policy_version: "1.0.0".to_string(),
            privacy_policy_accepted: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Minimal,    // Only critical errors and basic usage
    Standard,   // Standard telemetry (default)
    Enhanced,   // Additional usage patterns and performance metrics
    Full,       // Complete telemetry including detailed usage
}

impl PrivacyLevel {
    pub fn to_category_map(&self) -> HashMap<String, bool> {
        match self {
            PrivacyLevel::Minimal => [
                ("app_lifecycle".to_string(), true),
                ("feature_usage".to_string(), false),
                ("errors".to_string(), true),
                ("performance".to_string(), false),
                ("user_actions".to_string(), false),
                ("system_info".to_string(), false),
                ("logs".to_string(), false),
                ("model_usage".to_string(), false),
                ("network".to_string(), false),
            ].iter().cloned().collect(),
            
            PrivacyLevel::Standard => [
                ("app_lifecycle".to_string(), true),
                ("feature_usage".to_string(), true),
                ("errors".to_string(), true),
                ("performance".to_string(), true),
                ("user_actions".to_string(), false),
                ("system_info".to_string(), true),
                ("logs".to_string(), false),
                ("model_usage".to_string(), true),
                ("network".to_string(), true),
            ].iter().cloned().collect(),
            
            PrivacyLevel::Enhanced => [
                ("app_lifecycle".to_string(), true),
                ("feature_usage".to_string(), true),
                ("errors".to_string(), true),
                ("performance".to_string(), true),
                ("user_actions".to_string(), true),
                ("system_info".to_string(), true),
                ("logs".to_string(), false),
                ("model_usage".to_string(), true),
                ("network".to_string(), true),
            ].iter().cloned().collect(),
            
            PrivacyLevel::Full => [
                ("app_lifecycle".to_string(), true),
                ("feature_usage".to_string(), true),
                ("errors".to_string(), true),
                ("performance".to_string(), true),
                ("user_actions".to_string(), true),
                ("system_info".to_string(), true),
                ("logs".to_string(), true),
                ("model_usage".to_string(), true),
                ("network".to_string(), true),
            ].iter().cloned().collect(),
        }
    }
    
    pub fn from_category_map(map: &HashMap<String, bool>) -> Self {
        if *map.get("logs").unwrap_or(&false) {
            return PrivacyLevel::Full;
        }
        
        if *map.get("user_actions").unwrap_or(&false) {
            return PrivacyLevel::Enhanced;
        }
        
        if *map.get("feature_usage").unwrap_or(&false) {
            return PrivacyLevel::Standard;
        }
        
        PrivacyLevel::Minimal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStats {
    pub events_collected: u64,
    pub events_sent: u64,
    pub metrics_collected: u64,
    pub metrics_sent: u64,
    pub logs_collected: u64,
    pub logs_sent: u64,
    pub batches_sent: u64,
    pub batches_failed: u64,
    pub last_batch_sent: Option<DateTime<Utc>>,
}

impl Default for TelemetryStats {
    fn default() -> Self {
        Self {
            events_collected: 0,
            events_sent: 0,
            metrics_collected: 0,
            metrics_sent: 0,
            logs_collected: 0,
            logs_sent: 0,
            batches_sent: 0,
            batches_failed: 0,
            last_batch_sent: None,
        }
    }
}

pub struct TelemetryClient {
    config: Arc<RwLock<TelemetryConfig>>,
    event_buffer: Arc<Mutex<Vec<TelemetryEvent>>>,
    metrics_buffer: Arc<Mutex<Vec<Metric>>>,
    logs_buffer: Arc<Mutex<Vec<LogEntry>>>,
    is_running: Arc<RwLock<bool>>,
    last_flush: Arc<Mutex<Instant>>,
    http_client: HttpClient,
    stats: Arc<RwLock<TelemetryStats>>,
}

impl TelemetryClient {
    fn new(config: TelemetryConfig) -> Self {
        let config = Arc::new(RwLock::new(config));
        let event_buffer = Arc::new(Mutex::new(Vec::new()));
        let metrics_buffer = Arc::new(Mutex::new(Vec::new()));
        let logs_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_running = Arc::new(RwLock::new(false));
        let last_flush = Arc::new(Mutex::new(Instant::now()));
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| HttpClient::new());
        let stats = Arc::new(RwLock::new(TelemetryStats::default()));
        
        Self {
            config,
            event_buffer,
            metrics_buffer,
            logs_buffer,
            is_running,
            last_flush,
            http_client,
            stats,
        }
    }
    
    pub fn start_telemetry_worker(&self) {
        let config = Arc::clone(&self.config);
        let event_buffer = Arc::clone(&self.event_buffer);
        let metrics_buffer = Arc::clone(&self.metrics_buffer);
        let logs_buffer = Arc::clone(&self.logs_buffer);
        let is_running = Arc::clone(&self.is_running);
        let last_flush = Arc::clone(&self.last_flush);
        let http_client = self.http_client.clone();
        let stats = Arc::clone(&self.stats);
        
        // Set running state
        *is_running.write().unwrap() = true;
        
        thread::spawn(move || {
            debug!("Telemetry worker thread started");
            while *is_running.read().unwrap() {
                // Get batch interval from config
                let interval_seconds = {
                    let config = config.read().unwrap();
                    config.batch_interval_seconds
                };
                
                // Sleep for the batch interval
                thread::sleep(Duration::from_secs(interval_seconds));
                
                // Check if telemetry is enabled
                let is_enabled = {
                    let config = config.read().unwrap();
                    config.enabled && config.privacy_policy_accepted
                };
                
                if !is_enabled {
                    continue;
                }
                
                // Check if it's time to flush
                let should_flush = {
                    let last = last_flush.lock().unwrap();
                    last.elapsed() > Duration::from_secs(interval_seconds)
                };
                
                if !should_flush {
                    continue;
                }
                
                // Prepare batch for sending
                let batch = {
                    let config = config.read().unwrap();
                    let mut events = event_buffer.lock().unwrap();
                    let mut metrics = metrics_buffer.lock().unwrap();
                    let mut logs = logs_buffer.lock().unwrap();
                    
                    // Skip if all buffers are empty
                    if events.is_empty() && metrics.is_empty() && logs.is_empty() {
                        continue;
                    }
                    
                    // Update stats
                    {
                        let mut stats_guard = stats.write().unwrap();
                        stats_guard.events_collected += events.len() as u64;
                        stats_guard.metrics_collected += metrics.len() as u64;
                        stats_guard.logs_collected += logs.len() as u64;
                    }
                    
                    // Create batch
                    let batch = TelemetryBatch {
                        batch_id: Uuid::new_v4().to_string(),
                        client_id: config.client_id.clone(),
                        app_version: env!("CARGO_PKG_VERSION").to_string(),
                        platform: std::env::consts::OS.to_string(),
                        events: std::mem::take(events.as_mut()),
                        metrics: std::mem::take(metrics.as_mut()),
                        logs: std::mem::take(logs.as_mut()),
                        timestamp: Utc::now(),
                    };
                    
                    batch
                };
                
                // Send batch to server
                match Self::send_batch(&http_client, &batch, &config.read().unwrap().server_url) {
                    Ok(_) => {
                        // Successfully sent batch
                        let mut last = last_flush.lock().unwrap();
                        *last = Instant::now();
                        
                        // Update stats
                        {
                            let mut stats_guard = stats.write().unwrap();
                            stats_guard.events_sent += batch.events.len() as u64;
                            stats_guard.metrics_sent += batch.metrics.len() as u64;
                            stats_guard.logs_sent += batch.logs.len() as u64;
                            stats_guard.batches_sent += 1;
                            stats_guard.last_batch_sent = Some(Utc::now());
                        }
                        
                        debug!("Successfully sent telemetry batch: events={}, metrics={}, logs={}",
                               batch.events.len(), batch.metrics.len(), batch.logs.len());
                    }
                    Err(error) => {
                        // Failed to send batch, log error
                        warn!("Failed to send telemetry batch: {}", error);
                        
                        // Update stats
                        {
                            let mut stats_guard = stats.write().unwrap();
                            stats_guard.batches_failed += 1;
                        }
                        
                        // Return items to buffers
                        let mut events = event_buffer.lock().unwrap();
                        let mut metrics = metrics_buffer.lock().unwrap();
                        let mut logs = logs_buffer.lock().unwrap();
                        
                        events.extend(batch.events);
                        metrics.extend(batch.metrics);
                        logs.extend(batch.logs);
                    }
                }
            }
            
            debug!("Telemetry worker thread stopped");
        });
    }
    
    pub fn stop_telemetry_worker(&self) {
        let mut is_running = self.is_running.write().unwrap();
        *is_running = false;
    }
    
    async fn send_batch_async(http_client: &HttpClient, batch: &TelemetryBatch, server_url: &str) -> Result<()> {
        // Serialize batch
        let serialized = serde_json::to_string(batch)
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, format!("Failed to serialize batch: {}", e)))?;
        
        // In a real implementation, this would send the data to a telemetry server
        // For this example, we'll just simulate sending
        debug!("Sending telemetry batch: {} bytes to {}", serialized.len(), server_url);
        
        // Simulate network request in dev/test mode
        if cfg!(feature = "dev") || server_url.contains("example.com") {
            // Simulate network delay
            tokio::time::sleep(Duration::from_millis(100)).await;
            return Ok(());
        }
        
        // Real HTTP request in production
        let response = http_client.post(server_url)
            .header("Content-Type", "application/json")
            .body(serialized)
            .send()
            .await
            .map_err(|e| std::io::Error::new(ErrorKind::Other, format!("Failed to send batch: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Server returned error: {}", response.status()),
            ).into());
        }
        
        Ok(())
    }
    
    fn send_batch(http_client: &HttpClient, batch: &TelemetryBatch, server_url: &str) -> Result<()> {
        // Create a runtime for async request
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        
        rt.block_on(Self::send_batch_async(http_client, batch, server_url))
    }
    
    pub fn track_event(&self, event_type: TelemetryEventType, name: &str, value: Option<String>, properties: Option<HashMap<String, String>>) {
        // Check if telemetry is enabled
        let is_enabled = {
            let config = self.config.read().unwrap();
            config.enabled && config.privacy_policy_accepted
        };
        
        if !is_enabled {
            return;
        }
        
        // Check if this category is enabled
        let category_enabled = {
            let config = self.config.read().unwrap();
            *config.collection_categories.get(event_type.category()).unwrap_or(&false)
        };
        
        if !category_enabled {
            return;
        }
        
        // Create event
        let event = TelemetryEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: event_type.clone(),
            name: name.to_string(),
            value,
            properties: properties.unwrap_or_default(),
        };
        
        // Add to buffer
        let mut buffer = self.event_buffer.lock().unwrap();
        buffer.push(event);
        
        // Check if buffer is full
        let batch_size = {
            let config = self.config.read().unwrap();
            config.batch_size
        };
        
        if buffer.len() >= batch_size {
            // Force a flush
            let mut last = self.last_flush.lock().unwrap();
            *last = Instant::now() - Duration::from_secs(3600); // Set to 1 hour ago to force flush
        }
    }
    
    pub fn track_feature_usage(&self, feature_name: &str, properties: Option<HashMap<String, String>>) {
        self.track_event(
            TelemetryEventType::FeatureUsage,
            feature_name,
            None,
            properties,
        );
    }
    
    pub fn track_error(&self, error_name: &str, error_message: &str, properties: Option<HashMap<String, String>>) {
        let mut props = properties.unwrap_or_default();
        props.insert("message".to_string(), error_message.to_string());
        
        self.track_event(
            TelemetryEventType::Error,
            error_name,
            None,
            Some(props),
        );
    }
    
    pub fn track_performance(&self, operation_name: &str, duration_ms: f64, properties: Option<HashMap<String, String>>) {
        self.track_event(
            TelemetryEventType::Performance,
            operation_name,
            Some(duration_ms.to_string()),
            properties,
        );
    }
    
    pub fn track_model_usage(&self, model_name: &str, token_count: u32, properties: Option<HashMap<String, String>>) {
        let mut props = properties.unwrap_or_default();
        props.insert("token_count".to_string(), token_count.to_string());
        
        self.track_event(
            TelemetryEventType::ModelUsage,
            model_name,
            None,
            Some(props),
        );
    }
    
    pub fn send_metrics(&self, metrics: &[Metric]) -> Result<()> {
        // Check if telemetry is enabled
        let is_enabled = {
            let config = self.config.read().unwrap();
            config.enabled && 
            config.privacy_policy_accepted && 
            *config.collection_categories.get("performance").unwrap_or(&false)
        };
        
        if !is_enabled {
            return Ok(());
        }
        
        // Add to buffer
        let mut buffer = self.metrics_buffer.lock().unwrap();
        buffer.extend_from_slice(metrics);
        
        // Check if buffer is full
        let batch_size = {
            let config = self.config.read().unwrap();
            config.batch_size
        };
        
        if buffer.len() >= batch_size {
            // Force a flush
            let mut last = self.last_flush.lock().unwrap();
            *last = Instant::now() - Duration::from_secs(3600); // Set to 1 hour ago to force flush
        }
        
        Ok(())
    }
    
    pub fn send_log(&self, log: &LogEntry) -> Result<()> {
        // Check if telemetry is enabled
        let is_enabled = {
            let config = self.config.read().unwrap();
            config.enabled && 
            config.privacy_policy_accepted && 
            *config.collection_categories.get("logs").unwrap_or(&false)
        };
        
        if !is_enabled {
            return Ok(());
        }
        
        // Add to buffer
        let mut buffer = self.logs_buffer.lock().unwrap();
        buffer.push(log.clone());
        
        // Check if buffer is full
        let batch_size = {
            let config = self.config.read().unwrap();
            config.batch_size
        };
        
        if buffer.len() >= batch_size {
            // Force a flush
            let mut last = self.last_flush.lock().unwrap();
            *last = Instant::now() - Duration::from_secs(3600); // Set to 1 hour ago to force flush
        }
        
        Ok(())
    }
    
    pub fn update_config(&self, new_config: TelemetryConfig) {
        let mut config = self.config.write().unwrap();
        *config = new_config;
    }
    
    pub fn get_config(&self) -> TelemetryConfig {
        let config = self.config.read().unwrap();
        config.clone()
    }
    
    pub fn set_privacy_level(&self, level: PrivacyLevel) {
        let mut config = self.config.write().unwrap();
        config.collection_categories = level.to_category_map();
    }
    
    pub fn get_privacy_level(&self) -> PrivacyLevel {
        let config = self.config.read().unwrap();
        PrivacyLevel::from_category_map(&config.collection_categories)
    }
    
    pub fn accept_privacy_policy(&self, version: &str) {
        let mut config = self.config.write().unwrap();
        config.privacy_policy_version = version.to_string();
        config.privacy_policy_accepted = true;
    }
    
    pub fn is_telemetry_enabled(&self) -> bool {
        let config = self.config.read().unwrap();
        config.enabled && config.privacy_policy_accepted
    }
    
    pub fn delete_telemetry_data(&self) -> Result<()> {
        // Clear local buffers
        {
            let mut events = self.event_buffer.lock().unwrap();
            events.clear();
        }
        
        {
            let mut metrics = self.metrics_buffer.lock().unwrap();
            metrics.clear();
        }
        
        {
            let mut logs = self.logs_buffer.lock().unwrap();
            logs.clear();
        }
        
        // Generate a new client ID
        {
            let mut config = self.config.write().unwrap();
            config.client_id = Uuid::new_v4().to_string();
        }
        
        // Reset stats
        {
            let mut stats = self.stats.write().unwrap();
            *stats = TelemetryStats::default();
        }
        
        // In a real implementation, this would also send a deletion request to the server
        info!("Telemetry data deletion requested - client ID has been reset");
        
        // Send deletion request to server
        if let Ok(server_url) = self.get_deletion_endpoint() {
            let client_id = {
                let config = self.config.read().unwrap();
                config.client_id.clone()
            };
            
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            
            rt.block_on(async {
                let _ = self.http_client.delete(&server_url)
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({ "client_id": client_id }))
                    .send()
                    .await;
            });
        }
        
        Ok(())
    }
    
    fn get_deletion_endpoint(&self) -> Result<String> {
        let config = self.config.read().unwrap();
        let base_url = &config.server_url;
        
        // Append "/delete" to the base URL
        let url = if base_url.ends_with('/') {
            format!("{}delete", base_url)
        } else {
            format!("{}/delete", base_url)
        };
        
        Ok(url)
    }
    
    pub fn get_stats(&self) -> TelemetryStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }
    
    // Singleton instance getter
    pub fn get_instance() -> Arc<Self> {
        static mut INSTANCE: Option<Arc<TelemetryClient>> = None;
        static ONCE: Once = Once::new();
        
        unsafe {
            ONCE.call_once(|| {
                // Create default config
                let config = TelemetryConfig::default();
                
                // Create instance
                let client = TelemetryClient::new(config);
                client.start_telemetry_worker();
                
                INSTANCE = Some(Arc::new(client));
            });
            
            INSTANCE.clone().unwrap()
        }
    }
}

impl Drop for TelemetryClient {
    fn drop(&mut self) {
        // Track application exit event
        self.track_event(
            TelemetryEventType::ApplicationExit,
            "application_exit",
            None,
            None,
        );
        
        // Stop worker thread
        self.stop_telemetry_worker();
    }
}

// Initialize telemetry
pub fn init_telemetry() -> Arc<TelemetryClient> {
    let client = TelemetryClient::get_instance();
    
    // Track application start event
    client.track_event(
        TelemetryEventType::ApplicationStart,
        "application_start",
        None,
        None,
    );
    
    // Track system info
    let mut system_info = HashMap::new();
    system_info.insert("os".to_string(), std::env::consts::OS.to_string());
    system_info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    system_info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    
    client.track_event(
        TelemetryEventType::SystemInfo,
        "system_info",
        None,
        Some(system_info),
    );
    
    client
}

// Tauri commands
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_telemetry_config() -> TelemetryConfig {
    let client = TelemetryClient::get_instance();
    client.get_config()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn update_telemetry_config(config: TelemetryConfig) -> Result<()> {
    let client = TelemetryClient::get_instance();
    client.update_config(config);
    Ok(())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn set_privacy_level(level: String) -> Result<()> {
    let client = TelemetryClient::get_instance();
    let privacy_level = match level.as_str() {
        "minimal" => PrivacyLevel::Minimal,
        "standard" => PrivacyLevel::Standard,
        "enhanced" => PrivacyLevel::Enhanced,
        "full" => PrivacyLevel::Full,
        _ => return Err(std::io::Error::new(ErrorKind::InvalidInput, "Invalid privacy level").into()),
    };
    client.set_privacy_level(privacy_level);
    Ok(())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_privacy_level() -> String {
    let client = TelemetryClient::get_instance();
    match client.get_privacy_level() {
        PrivacyLevel::Minimal => "minimal".to_string(),
        PrivacyLevel::Standard => "standard".to_string(),
        PrivacyLevel::Enhanced => "enhanced".to_string(),
        PrivacyLevel::Full => "full".to_string(),
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn delete_telemetry_data() -> Result<()> {
    let client = TelemetryClient::get_instance();
    client.delete_telemetry_data()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_telemetry_stats() -> TelemetryStats {
    let client = TelemetryClient::get_instance();
    client.get_stats()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn accept_privacy_policy(version: String) -> Result<()> {
    let client = TelemetryClient::get_instance();
    client.accept_privacy_policy(&version);
    Ok(())
}

// Helper function to track feature usage
pub fn track_feature_usage(feature_name: &str, properties: Option<HashMap<String, String>>) {
    let client = TelemetryClient::get_instance();
    client.track_feature_usage(feature_name, properties);
}

// Helper function to track errors
pub fn track_error(error_name: &str, error_message: &str, properties: Option<HashMap<String, String>>) {
    let client = TelemetryClient::get_instance();
    client.track_error(error_name, error_message, properties);
}

// Helper function to track performance
pub fn track_performance(operation_name: &str, duration_ms: f64, properties: Option<HashMap<String, String>>) {
    let client = TelemetryClient::get_instance();
    client.track_performance(operation_name, duration_ms, properties);
}

// Helper function to track model usage
pub fn track_model_usage(model_name: &str, token_count: u32, properties: Option<HashMap<String, String>>) {
    let client = TelemetryClient::get_instance();
    client.track_model_usage(model_name, token_count, properties);
}

// Macro for timing and tracking performance
#[macro_export]
macro_rules! track_performance {
    ($name:expr, $block:expr) => {
        {
            let start = std::time::Instant::now();
            let result = $block;
            let duration = start.elapsed();
            $crate::observability::telemetry::track_performance(
                $name, 
                duration.as_secs_f64() * 1000.0,
                None,
            );
            result
        }
    };
    ($name:expr, $properties:expr, $block:expr) => {
        {
            let start = std::time::Instant::now();
            let result = $block;
            let duration = start.elapsed();
            $crate::observability::telemetry::track_performance(
                $name, 
                duration.as_secs_f64() * 1000.0,
                $properties,
            );
            result
        }
    };
}