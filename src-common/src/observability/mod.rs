// Observability module for the MCP client
//
// This module contains components for monitoring, logging, telemetry,
// and canary release management.

pub mod metrics;
pub mod logging;
pub mod telemetry;
pub mod canary;

use serde::{Serialize, Deserialize};

// Shared configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub enabled: bool,
    pub telemetry: TelemetryConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub canary_release: CanaryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub server_url: String,
    pub client_id: Option<String>,
    pub batch_interval_seconds: u64,
    pub collection_categories: Vec<String>,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub min_level: logging::LogLevel,
    pub console_enabled: bool,
    pub file_enabled: bool,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub log_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub collection_interval_seconds: u64,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub enabled: bool,
    pub group: Option<String>,
    pub opt_in_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyLevel {
    Minimal,    // Only essential data (errors, crashes)
    Basic,      // Basic usage data (features used, performance metrics)
    Standard,   // More detailed usage and performance data
    Extended,   // Detailed telemetry including user flows and patterns
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            telemetry: TelemetryConfig {
                enabled: false, // Disabled by default, requires opt-in
                server_url: "https://telemetry.mcp-client.example.com/v1/telemetry".to_string(),
                client_id: None, // Will be generated on first run
                batch_interval_seconds: 300, // 5 minutes
                collection_categories: vec![
                    "errors".to_string(),
                    "performance".to_string(),
                ],
                privacy_level: PrivacyLevel::Basic,
            },
            logging: LoggingConfig {
                min_level: logging::LogLevel::Info,
                console_enabled: true,
                file_enabled: true,
                max_file_size_mb: 10,
                max_files: 5,
                log_dir: None, // Will be set to default app data directory
            },
            metrics: MetricsConfig {
                enabled: true,
                sampling_rate: 0.1, // 10% sampling
                collection_interval_seconds: 60,
                buffer_size: 100,
            },
            canary_release: CanaryConfig {
                enabled: true,
                group: None, // No canary group by default
                opt_in_percentage: 0.05, // 5% of users will be asked to opt in
            },
        }
    }
}

// Initialize all observability systems
pub fn init(config: &ObservabilityConfig) {
    // Initialize logging
    let logging_config = metrics::ObservabilityConfig {
        metrics_enabled: config.metrics.enabled,
        sampling_rate: config.metrics.sampling_rate,
        buffer_size: config.metrics.buffer_size,
        min_log_level: Some(config.logging.min_level as u8),
        log_file_path: config.logging.log_dir.clone(),
        console_logging: Some(config.logging.console_enabled),
        telemetry_enabled: Some(config.telemetry.enabled),
        log_telemetry: Some(config.logging.file_enabled),
    };
    
    // Initialize logging first for other components to use
    logging::init_logger(&logging_config);
    
    // Initialize metrics
    metrics::init_metrics(&logging_config);
    
    // Log initialization
    log_info!("observability", "Observability systems initialized");
}