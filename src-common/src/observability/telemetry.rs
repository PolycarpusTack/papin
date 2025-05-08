use std::sync::{Arc, Mutex, Once};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use uuid::Uuid;

use crate::observability::logging::LogEntry;
use crate::observability::metrics::Metric;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryEventType {
    ApplicationStart,
    ApplicationExit,
    FeatureUsage,
    Error,
    Performance,
    UserAction,
    SystemInfo,
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
            ].iter().cloned().collect(),
            batch_size: 100,
            batch_interval_seconds: 60,
            server_url: "https://telemetry.mcp-client.example.com/v1/telemetry".to_string(),
        }
    }
}

pub struct TelemetryClient {
    config: Arc<Mutex<TelemetryConfig>>,
    event_buffer: Arc<Mutex<Vec<TelemetryEvent>>>,
    metrics_buffer: Arc<Mutex<Vec<Metric>>>,
    logs_buffer: Arc<Mutex<Vec<LogEntry>>>,
    is_running: Arc<Mutex<bool>>,
    last_flush: Arc<Mutex<Instant>>,
}

impl TelemetryClient {
    fn new(config: TelemetryConfig) -> Self {
        let config = Arc::new(Mutex::new(config));
        let event_buffer = Arc::new(Mutex::new(Vec::new()));
        let metrics_buffer = Arc::new(Mutex::new(Vec::new()));
        let logs_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_running = Arc::new(Mutex::new(false));
        let last_flush = Arc::new(Mutex::new(Instant::now()));
        
        Self {
            config,
            event_buffer,
            metrics_buffer,
            logs_buffer,
            is_running,
            last_flush,
        }
    }
    
    pub fn start_telemetry_worker(&self) {
        let config = Arc::clone(&self.config);
        let event_buffer = Arc::clone(&self.event_buffer);
        let metrics_buffer = Arc::clone(&self.metrics_buffer);
        let logs_buffer = Arc::clone(&self.logs_buffer);
        let is_running = Arc::clone(&self.is_running);
        let last_flush = Arc::clone(&self.last_flush);
        
        // Set running state
        *is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            while *is_running.lock().unwrap() {
                // Get batch interval from config
                let interval_seconds = {
                    let config = config.lock().unwrap();
                    config.batch_interval_seconds
                };
                
                // Sleep for the batch interval
                thread::sleep(Duration::from_secs(interval_seconds));
                
                // Check if telemetry is enabled
                let is_enabled = {
                    let config = config.lock().unwrap();
                    config.enabled
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
                    let config = config.lock().unwrap();
                    let mut events = event_buffer.lock().unwrap();
                    let mut metrics = metrics_buffer.lock().unwrap();
                    let mut logs = logs_buffer.lock().unwrap();
                    
                    // Skip if all buffers are empty
                    if events.is_empty() && metrics.is_empty() && logs.is_empty() {
                        continue;
                    }
                    
                    // Create batch
                    let batch = TelemetryBatch {
                        batch_id: Uuid::new_v4().to_string(),
                        client_id: config.client_id.clone(),
                        app_version: env!("CARGO_PKG_VERSION").to_string(),
                        events: std::mem::take(events.as_mut()),
                        metrics: std::mem::take(metrics.as_mut()),
                        logs: std::mem::take(logs.as_mut()),
                        timestamp: Utc::now(),
                    };
                    
                    batch
                };
                
                // Send batch to server
                match Self::send_batch(&batch) {
                    Ok(_) => {
                        // Successfully sent batch
                        let mut last = last_flush.lock().unwrap();
                        *last = Instant::now();
                    }
                    Err(error) => {
                        // Failed to send batch, log error
                        eprintln!("Failed to send telemetry batch: {}", error);
                        
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
        });
    }
    
    pub fn stop_telemetry_worker(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
    }
    
    fn send_batch(batch: &TelemetryBatch) -> Result<(), String> {
        // Serialize batch
        let serialized = serde_json::to_string(batch)
            .map_err(|e| format!("Failed to serialize batch: {}", e))?;
        
        // In a real implementation, this would send the data to a telemetry server
        // For this example, we'll just simulate sending
        println!("Sending telemetry batch: {} bytes", serialized.len());
        
        // Simulate network delay
        thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }
    
    pub fn track_event(&self, event_type: TelemetryEventType, name: &str, value: Option<String>, properties: Option<HashMap<String, String>>) {
        // Check if telemetry is enabled
        let is_enabled = {
            let config = self.config.lock().unwrap();
            config.enabled
        };
        
        if !is_enabled {
            return;
        }
        
        // Check if this category is enabled
        let category_enabled = {
            let config = self.config.lock().unwrap();
            let category = match event_type {
                TelemetryEventType::ApplicationStart | TelemetryEventType::ApplicationExit => "app_lifecycle",
                TelemetryEventType::FeatureUsage => "feature_usage",
                TelemetryEventType::Error => "errors",
                TelemetryEventType::Performance => "performance",
                TelemetryEventType::UserAction => "user_actions",
                TelemetryEventType::SystemInfo => "system_info",
            };
            
            *config.collection_categories.get(category).unwrap_or(&false)
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
            let config = self.config.lock().unwrap();
            config.batch_size
        };
        
        if buffer.len() >= batch_size {
            // Force a flush
            let mut last = self.last_flush.lock().unwrap();
            *last = Instant::now() - Duration::from_secs(3600); // Set to 1 hour ago to force flush
        }
    }
    
    pub fn send_metrics(&self, metrics: &[Metric]) {
        // Check if telemetry is enabled
        let is_enabled = {
            let config = self.config.lock().unwrap();
            config.enabled && *config.collection_categories.get("performance").unwrap_or(&false)
        };
        
        if !is_enabled {
            return;
        }
        
        // Add to buffer
        let mut buffer = self.metrics_buffer.lock().unwrap();
        buffer.extend_from_slice(metrics);
        
        // Check if buffer is full
        let batch_size = {
            let config = self.config.lock().unwrap();
            config.batch_size
        };
        
        if buffer.len() >= batch_size {
            // Force a flush
            let mut last = self.last_flush.lock().unwrap();
            *last = Instant::now() - Duration::from_secs(3600); // Set to 1 hour ago to force flush
        }
    }
    
    pub fn send_log(&self, log: &LogEntry) {
        // Check if telemetry is enabled
        let is_enabled = {
            let config = self.config.lock().unwrap();
            config.enabled && *config.collection_categories.get("logs").unwrap_or(&false)
        };
        
        if !is_enabled {
            return;
        }
        
        // Add to buffer
        let mut buffer = self.logs_buffer.lock().unwrap();
        buffer.push(log.clone());
        
        // Check if buffer is full
        let batch_size = {
            let config = self.config.lock().unwrap();
            config.batch_size
        };
        
        if buffer.len() >= batch_size {
            // Force a flush
            let mut last = self.last_flush.lock().unwrap();
            *last = Instant::now() - Duration::from_secs(3600); // Set to 1 hour ago to force flush
        }
    }
    
    pub fn update_config(&self, new_config: TelemetryConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
    }
    
    pub fn get_config(&self) -> TelemetryConfig {
        let config = self.config.lock().unwrap();
        config.clone()
    }
    
    pub fn delete_telemetry_data(&self) -> Result<(), String> {
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
            let mut config = self.config.lock().unwrap();
            config.client_id = Uuid::new_v4().to_string();
        }
        
        // In a real implementation, this would also send a deletion request to the server
        println!("Telemetry data deletion requested - client ID has been reset");
        
        Ok(())
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
        self.stop_telemetry_worker();
    }
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
pub fn update_telemetry_config(config: TelemetryConfig) -> Result<(), String> {
    let client = TelemetryClient::get_instance();
    client.update_config(config);
    Ok(())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn delete_telemetry_data() -> Result<(), String> {
    let client = TelemetryClient::get_instance();
    client.delete_telemetry_data()
}