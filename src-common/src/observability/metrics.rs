use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use rand::Rng;

use crate::observability::telemetry::TelemetryClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub sampling_rate: f64,
    pub buffer_size: usize,
    pub min_log_level: Option<u8>,
    pub log_file_path: Option<String>,
    pub console_logging: Option<bool>,
    pub telemetry_enabled: Option<bool>,
    pub log_telemetry: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub metric_type: MetricType,
    pub tags: HashMap<String, String>,
    pub timestamp: SystemTime,
}

pub struct MetricsCollector {
    // Configuration
    enabled: bool,
    sampling_rate: f64,
    buffer_size: usize,
    
    // Metric storage
    metrics_buffer: Arc<Mutex<Vec<Metric>>>,
    
    // Last flush time
    last_flush: Instant,
}

impl MetricsCollector {
    pub fn new(config: &ObservabilityConfig) -> Self {
        Self {
            enabled: config.metrics_enabled,
            sampling_rate: config.sampling_rate,
            buffer_size: config.buffer_size,
            metrics_buffer: Arc::new(Mutex::new(Vec::with_capacity(config.buffer_size))),
            last_flush: Instant::now(),
        }
    }
    
    pub fn record_counter(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        if !self.enabled || !self.should_sample() {
            return;
        }
        
        self.record_metric(name, value, MetricType::Counter, tags);
    }
    
    pub fn record_gauge(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        if !self.enabled {
            return;
        }
        
        self.record_metric(name, value, MetricType::Gauge, tags);
    }
    
    pub fn record_histogram(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        if !self.enabled || !self.should_sample() {
            return;
        }
        
        self.record_metric(name, value, MetricType::Histogram, tags);
    }
    
    pub fn time<F, R>(&self, name: &str, tags: HashMap<String, String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if !self.enabled {
            return f();
        }
        
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        self.record_metric(
            name, 
            duration.as_secs_f64() * 1000.0, // Convert to milliseconds
            MetricType::Timer,
            tags,
        );
        
        result
    }
    
    fn record_metric(&self, name: &str, value: f64, metric_type: MetricType, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value,
            metric_type,
            tags,
            timestamp: SystemTime::now(),
        };
        
        let mut metrics_buffer = self.metrics_buffer.lock().unwrap();
        metrics_buffer.push(metric);
        
        if metrics_buffer.len() >= self.buffer_size || self.last_flush.elapsed() > Duration::from_secs(60) {
            self.flush_metrics(&mut metrics_buffer);
        }
    }
    
    fn should_sample(&self) -> bool {
        rand::thread_rng().gen_range(0.0..1.0) <= self.sampling_rate
    }
    
    fn flush_metrics(&self, metrics_buffer: &mut Vec<Metric>) {
        if metrics_buffer.is_empty() {
            return;
        }
        
        // In a real implementation, we would send these metrics to a telemetry backend
        // For this example, we'll just log a count of the metrics being flushed
        println!("Flushing {} metrics", metrics_buffer.len());
        
        // In practice, we would have code like:
        // let telemetry_client = TelemetryClient::get_instance();
        // telemetry_client.send_metrics(&metrics_buffer);
        
        // Clear buffer
        metrics_buffer.clear();
    }
    
    // Add a method to get a snapshot of current metrics for the dashboard
    pub fn get_metrics_snapshot(&self) -> Vec<Metric> {
        let metrics_buffer = self.metrics_buffer.lock().unwrap();
        metrics_buffer.clone()
    }
}

// Create a global metrics collector
lazy_static::lazy_static! {
    pub static ref METRICS_COLLECTOR: Arc<Mutex<Option<MetricsCollector>>> = Arc::new(Mutex::new(None));
}

// Initialize metrics collector
pub fn init_metrics(config: &ObservabilityConfig) {
    let collector = MetricsCollector::new(config);
    let mut global_collector = METRICS_COLLECTOR.lock().unwrap();
    *global_collector = Some(collector);
}

// Helper functions to record metrics
pub fn record_counter(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.lock().unwrap().as_ref() {
        collector.record_counter(name, value, tags.unwrap_or_default());
    }
}

pub fn record_gauge(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.lock().unwrap().as_ref() {
        collector.record_gauge(name, value, tags.unwrap_or_default());
    }
}

pub fn record_histogram(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.lock().unwrap().as_ref() {
        collector.record_histogram(name, value, tags.unwrap_or_default());
    }
}

// Macro for timing blocks of code
#[macro_export]
macro_rules! time_operation {
    ($name:expr, $tags:expr, $block:expr) => {
        {
            if let Some(collector) = $crate::observability::metrics::METRICS_COLLECTOR.lock().unwrap().as_ref() {
                collector.time($name, $tags.unwrap_or_default(), || $block)
            } else {
                $block
            }
        }
    };
}