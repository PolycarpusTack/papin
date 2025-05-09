use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use rand::Rng;
use log::{debug, warn, error};

use crate::observability::telemetry::TelemetryClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub sampling_rate: f64,
    pub buffer_size: usize,
    pub flush_interval_secs: u64,
    pub min_log_level: Option<u8>,
    pub log_file_path: Option<String>,
    pub console_logging: Option<bool>,
    pub telemetry_enabled: Option<bool>,
    pub log_telemetry: Option<bool>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            sampling_rate: 0.1, // Sample 10% of metrics by default
            buffer_size: 1000,
            flush_interval_secs: 60,
            min_log_level: Some(2), // Info level
            log_file_path: None,
            console_logging: Some(true),
            telemetry_enabled: Some(false), // Opt-in by default
            log_telemetry: Some(false),
        }
    }
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

// Store historical metrics for dashboard visualization
#[derive(Debug, Clone, Default)]
pub struct MetricsHistory {
    // Store raw metrics in time-series buckets
    pub counters: HashMap<String, Vec<(SystemTime, f64)>>,
    pub gauges: HashMap<String, Vec<(SystemTime, f64)>>,
    pub histograms: HashMap<String, Vec<(SystemTime, f64)>>,
    pub timers: HashMap<String, Vec<(SystemTime, f64)>>,
    
    // Store aggregated statistics
    pub counter_totals: HashMap<String, f64>,
    pub gauge_latest: HashMap<String, f64>,
    pub histogram_stats: HashMap<String, HistogramStats>,
    pub timer_stats: HashMap<String, TimerStats>,
    
    // Limit history size
    max_history_points: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub count: usize,
    pub p50: f64, // Median
    pub p90: f64, // 90th percentile
    pub p99: f64, // 99th percentile
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            avg: 0.0,
            count: 0,
            p50: 0.0,
            p90: 0.0,
            p99: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub count: usize,
    pub p50_ms: f64, // Median
    pub p90_ms: f64, // 90th percentile
    pub p99_ms: f64, // 99th percentile
}

impl Default for TimerStats {
    fn default() -> Self {
        Self {
            min_ms: f64::MAX,
            max_ms: f64::MIN,
            avg_ms: 0.0,
            count: 0,
            p50_ms: 0.0,
            p90_ms: 0.0,
            p99_ms: 0.0,
        }
    }
}

impl MetricsHistory {
    pub fn new(max_history_points: usize) -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            timers: HashMap::new(),
            counter_totals: HashMap::new(),
            gauge_latest: HashMap::new(),
            histogram_stats: HashMap::new(),
            timer_stats: HashMap::new(),
            max_history_points,
        }
    }
    
    pub fn add_metric(&mut self, metric: &Metric) {
        match metric.metric_type {
            MetricType::Counter => self.add_counter(metric),
            MetricType::Gauge => self.add_gauge(metric),
            MetricType::Histogram => self.add_histogram(metric),
            MetricType::Timer => self.add_timer(metric),
        }
    }
    
    fn add_counter(&mut self, metric: &Metric) {
        let entry = self.counters.entry(metric.name.clone()).or_insert_with(Vec::new);
        entry.push((metric.timestamp, metric.value));
        
        // Limit history size
        if entry.len() > self.max_history_points {
            entry.remove(0);
        }
        
        // Update totals
        let total = self.counter_totals.entry(metric.name.clone()).or_insert(0.0);
        *total += metric.value;
    }
    
    fn add_gauge(&mut self, metric: &Metric) {
        let entry = self.gauges.entry(metric.name.clone()).or_insert_with(Vec::new);
        entry.push((metric.timestamp, metric.value));
        
        // Limit history size
        if entry.len() > self.max_history_points {
            entry.remove(0);
        }
        
        // Update latest value
        self.gauge_latest.insert(metric.name.clone(), metric.value);
    }
    
    fn add_histogram(&mut self, metric: &Metric) {
        let entry = self.histograms.entry(metric.name.clone()).or_insert_with(Vec::new);
        entry.push((metric.timestamp, metric.value));
        
        // Limit history size
        if entry.len() > self.max_history_points {
            entry.remove(0);
        }
        
        // Update statistics
        let stats = self.histogram_stats.entry(metric.name.clone()).or_insert_with(HistogramStats::default);
        self.update_histogram_stats(stats, metric.value);
    }
    
    fn add_timer(&mut self, metric: &Metric) {
        let entry = self.timers.entry(metric.name.clone()).or_insert_with(Vec::new);
        entry.push((metric.timestamp, metric.value));
        
        // Limit history size
        if entry.len() > self.max_history_points {
            entry.remove(0);
        }
        
        // Update statistics
        let stats = self.timer_stats.entry(metric.name.clone()).or_insert_with(TimerStats::default);
        self.update_timer_stats(stats, metric.value);
    }
    
    fn update_histogram_stats(&mut self, stats: &mut HistogramStats, value: f64) {
        stats.min = stats.min.min(value);
        stats.max = stats.max.max(value);
        
        // Update average
        let new_count = stats.count + 1;
        stats.avg = (stats.avg * stats.count as f64 + value) / new_count as f64;
        stats.count = new_count;
        
        // For true percentiles, we'd need to sort all values
        // This is a simplification
        stats.p50 = stats.avg;
        stats.p90 = stats.max * 0.9;
        stats.p99 = stats.max * 0.99;
    }
    
    fn update_timer_stats(&mut self, stats: &mut TimerStats, value: f64) {
        stats.min_ms = stats.min_ms.min(value);
        stats.max_ms = stats.max_ms.max(value);
        
        // Update average
        let new_count = stats.count + 1;
        stats.avg_ms = (stats.avg_ms * stats.count as f64 + value) / new_count as f64;
        stats.count = new_count;
        
        // For true percentiles, we'd need to sort all values
        // This is a simplification
        stats.p50_ms = stats.avg_ms;
        stats.p90_ms = stats.max_ms * 0.9;
        stats.p99_ms = stats.max_ms * 0.99;
    }
    
    pub fn get_counters_report(&self) -> HashMap<String, f64> {
        self.counter_totals.clone()
    }
    
    pub fn get_gauges_report(&self) -> HashMap<String, f64> {
        self.gauge_latest.clone()
    }
    
    pub fn get_histograms_report(&self) -> HashMap<String, HistogramStats> {
        self.histogram_stats.clone()
    }
    
    pub fn get_timers_report(&self) -> HashMap<String, TimerStats> {
        self.timer_stats.clone()
    }
    
    // Get time series data for a specific metric for dashboard charts
    pub fn get_time_series(&self, name: &str, metric_type: MetricType) -> Vec<(SystemTime, f64)> {
        match metric_type {
            MetricType::Counter => self.counters.get(name).cloned().unwrap_or_default(),
            MetricType::Gauge => self.gauges.get(name).cloned().unwrap_or_default(),
            MetricType::Histogram => self.histograms.get(name).cloned().unwrap_or_default(),
            MetricType::Timer => self.timers.get(name).cloned().unwrap_or_default(),
        }
    }
}

pub struct MetricsCollector {
    // Configuration
    enabled: bool,
    sampling_rate: f64,
    buffer_size: usize,
    flush_interval: Duration,
    
    // Metric storage
    metrics_buffer: Arc<Mutex<Vec<Metric>>>,
    history: Arc<RwLock<MetricsHistory>>,
    
    // Last flush time
    last_flush: Instant,
    
    // Telemetry client for sending metrics
    telemetry_client: Option<Arc<TelemetryClient>>,
}

impl MetricsCollector {
    pub fn new(config: &ObservabilityConfig, telemetry_client: Option<Arc<TelemetryClient>>) -> Self {
        Self {
            enabled: config.metrics_enabled,
            sampling_rate: config.sampling_rate,
            buffer_size: config.buffer_size,
            flush_interval: Duration::from_secs(config.flush_interval_secs),
            metrics_buffer: Arc::new(Mutex::new(Vec::with_capacity(config.buffer_size))),
            history: Arc::new(RwLock::new(MetricsHistory::new(1000))), // Keep last 1000 data points
            last_flush: Instant::now(),
            telemetry_client,
        }
    }
    
    pub fn record_counter(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        if !self.enabled || !self.should_sample() {
            return;
        }
        
        self.record_metric(name, value, MetricType::Counter, tags);
    }
    
    pub fn increment_counter(&self, name: &str, tags: HashMap<String, String>) {
        self.record_counter(name, 1.0, tags);
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
        
        // Update metrics history
        {
            let mut history = self.history.write().unwrap();
            history.add_metric(&metric);
        }
        
        // Add to buffer for sending to backend
        let mut metrics_buffer = self.metrics_buffer.lock().unwrap();
        metrics_buffer.push(metric);
        
        if metrics_buffer.len() >= self.buffer_size || self.last_flush.elapsed() > self.flush_interval {
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
        
        let metrics_count = metrics_buffer.len();
        debug!("Flushing {} metrics", metrics_count);
        
        // Send metrics to telemetry service if enabled
        if let Some(client) = &self.telemetry_client {
            if client.is_telemetry_enabled() {
                match client.send_metrics(metrics_buffer) {
                    Ok(_) => debug!("Successfully sent {} metrics to telemetry", metrics_count),
                    Err(e) => warn!("Failed to send metrics to telemetry: {}", e),
                }
            }
        }
        
        // Clear buffer
        metrics_buffer.clear();
    }
    
    // Get a snapshot of current metrics for the dashboard
    pub fn get_metrics_snapshot(&self) -> Vec<Metric> {
        let metrics_buffer = self.metrics_buffer.lock().unwrap();
        metrics_buffer.clone()
    }
    
    // Get metrics history for dashboard visualizations
    pub fn get_history(&self) -> Arc<RwLock<MetricsHistory>> {
        self.history.clone()
    }
    
    // Get timers report
    pub fn get_timers_report(&self) -> HashMap<String, TimerStats> {
        let history = self.history.read().unwrap();
        history.get_timers_report()
    }
    
    // Get counters report
    pub fn get_counters_report(&self) -> HashMap<String, f64> {
        let history = self.history.read().unwrap();
        history.get_counters_report()
    }
    
    // Get gauges report
    pub fn get_gauges_report(&self) -> HashMap<String, f64> {
        let history = self.history.read().unwrap();
        history.get_gauges_report()
    }
    
    // Get histograms report
    pub fn get_histograms_report(&self) -> HashMap<String, HistogramStats> {
        let history = self.history.read().unwrap();
        history.get_histograms_report()
    }
}

// Create a global metrics collector
lazy_static::lazy_static! {
    pub static ref METRICS_COLLECTOR: Arc<RwLock<Option<MetricsCollector>>> = Arc::new(RwLock::new(None));
}

// Initialize metrics collector
pub fn init_metrics(config: &ObservabilityConfig, telemetry_client: Option<Arc<TelemetryClient>>) {
    let collector = MetricsCollector::new(config, telemetry_client);
    let mut global_collector = METRICS_COLLECTOR.write().unwrap();
    *global_collector = Some(collector);
    debug!("Metrics collector initialized with sampling rate: {}", config.sampling_rate);
}

// Helper functions to record metrics
pub fn record_counter(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.read().unwrap().as_ref() {
        collector.record_counter(name, value, tags.unwrap_or_default());
    }
}

pub fn increment_counter(name: &str, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.read().unwrap().as_ref() {
        collector.increment_counter(name, tags.unwrap_or_default());
    }
}

pub fn record_gauge(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.read().unwrap().as_ref() {
        collector.record_gauge(name, value, tags.unwrap_or_default());
    }
}

pub fn record_histogram(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Some(collector) = METRICS_COLLECTOR.read().unwrap().as_ref() {
        collector.record_histogram(name, value, tags.unwrap_or_default());
    }
}

pub fn time_operation<F, R>(name: &str, tags: Option<HashMap<String, String>>, f: F) -> R
where
    F: FnOnce() -> R,
{
    if let Some(collector) = METRICS_COLLECTOR.read().unwrap().as_ref() {
        collector.time(name, tags.unwrap_or_default(), f)
    } else {
        f()
    }
}

// Get metrics reports for dashboard
pub fn get_timers_report() -> Option<HashMap<String, TimerStats>> {
    METRICS_COLLECTOR.read().unwrap().as_ref().map(|collector| collector.get_timers_report())
}

pub fn get_counters_report() -> Option<HashMap<String, f64>> {
    METRICS_COLLECTOR.read().unwrap().as_ref().map(|collector| collector.get_counters_report())
}

pub fn get_gauges_report() -> Option<HashMap<String, f64>> {
    METRICS_COLLECTOR.read().unwrap().as_ref().map(|collector| collector.get_gauges_report())
}

pub fn get_histograms_report() -> Option<HashMap<String, HistogramStats>> {
    METRICS_COLLECTOR.read().unwrap().as_ref().map(|collector| collector.get_histograms_report())
}

// Macro for timing blocks of code
#[macro_export]
macro_rules! time_operation {
    ($name:expr, $tags:expr, $block:expr) => {
        {
            $crate::observability::metrics::time_operation($name, $tags, || $block)
        }
    };
}