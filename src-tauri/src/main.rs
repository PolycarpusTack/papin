#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod monitoring;
mod services;
mod offline;

use std::collections::HashMap;
use std::sync::Once;
use std::time::Instant;
use chrono::Utc;

// Import observability modules
use mcp_client::observability::{
    metrics::{init_metrics, ObservabilityConfig as MetricsConfig, record_counter},
    logging::{init_logger, LogLevel},
    telemetry::{TelemetryClient, TelemetryEventType},
    canary::CANARY_SERVICE,
};

// Import feature flags
use mcp_client::feature_flags::{
    FEATURE_FLAG_MANAGER, FeatureFlag, RolloutStrategy,
    CANARY_GROUP_ALPHA, CANARY_GROUP_BETA, CANARY_GROUP_EARLY_ACCESS,
    FLAG_ADVANCED_TELEMETRY, FLAG_PERFORMANCE_DASHBOARD, FLAG_DEBUG_LOGGING,
    FLAG_RESOURCE_MONITORING, FLAG_CRASH_REPORTING
};

// Import monitoring tools
use monitoring::resources::RESOURCE_MONITOR;

// For startup performance tracking
static mut APP_START_TIME: Option<Instant> = None;
static START_TIME_INIT: Once = Once::new();

fn main() {
    // Record application start time
    unsafe {
        START_TIME_INIT.call_once(|| {
            APP_START_TIME = Some(Instant::now());
        });
    }

    // Initialize observability configuration
    let app_data_dir = app_dirs::app_dir(
        app_dirs::AppDataType::UserData,
        &app_info::AppInfo {
            name: "mcp-client",
            author: "acme",
        },
        "logs",
    )
    .expect("Failed to get app data directory");
    
    let log_file_path = app_data_dir.join("mcp-client.log");
    
    // Configure metrics
    let metrics_config = MetricsConfig {
        metrics_enabled: true,
        sampling_rate: 0.1, // 10% sampling rate
        buffer_size: 100,
        min_log_level: Some(LogLevel::Info as u8),
        log_file_path: Some(log_file_path.to_str().unwrap().to_string()),
        console_logging: Some(true),
        telemetry_enabled: Some(false), // Start with telemetry disabled, require opt-in
        log_telemetry: Some(false),
    };
    
    // Initialize metrics system
    init_metrics(&metrics_config);
    
    // Initialize logger
    init_logger(&metrics_config);
    
    // Log application start
    log_info!("main", "MCP Client starting up");
    
    // Get telemetry client
    let telemetry_client = TelemetryClient::get_instance();
    
    // Register shutdown handler
    register_shutdown_handler(telemetry_client.clone());
    
    // Initialize feature flags
    initialize_feature_flags();
    
    // Track startup as a telemetry event
    std::thread::spawn(move || {
        telemetry_client.track_event(
            TelemetryEventType::ApplicationStart,
            "app_start",
            None,
            Some(HashMap::from([
                ("version".to_string(), env!("CARGO_PKG_VERSION").to_string()),
                ("os".to_string(), std::env::consts::OS.to_string()),
            ])),
        );
    });

    // Build Tauri application
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Resource monitoring commands
            monitoring::resources::get_resource_metrics,
            monitoring::resources::get_system_info,
            monitoring::resources::update_resource_metrics,
            monitoring::resources::get_uptime,
            monitoring::resources::report_startup_time,
            monitoring::resources::report_frame_rate,
            monitoring::resources::report_resource_metrics,
            monitoring::resources::report_page_metrics,
            
            // Logging commands
            mcp_client::observability::logging::get_recent_logs,
            mcp_client::observability::logging::export_logs,
            
            // Telemetry commands
            mcp_client::observability::telemetry::get_telemetry_config,
            mcp_client::observability::telemetry::update_telemetry_config,
            mcp_client::observability::telemetry::delete_telemetry_data,
            
            // Feature flag commands
            mcp_client::feature_flags::get_feature_flags,
            mcp_client::feature_flags::toggle_feature_flag,
            mcp_client::feature_flags::create_feature_flag,
            mcp_client::feature_flags::delete_feature_flag,
            
            // Canary release commands
            mcp_client::observability::canary::get_canary_groups,
            mcp_client::observability::canary::get_user_canary_group,
            mcp_client::observability::canary::opt_into_canary_group,
            mcp_client::observability::canary::opt_out_of_canary_group,
            mcp_client::observability::canary::toggle_canary_group,
            mcp_client::observability::canary::update_canary_percentage,
            mcp_client::observability::canary::get_canary_metrics,
            mcp_client::observability::canary::promote_canary_feature,
            mcp_client::observability::canary::rollback_canary_feature,
            mcp_client::observability::canary::create_canary_feature,
            mcp_client::observability::canary::toggle_canary_feature,
            
            // Offline LLM commands
            commands::offline::configure_llm,
            commands::offline::list_available_models,
            commands::offline::list_downloaded_models,
            commands::offline::get_model_info,
            commands::offline::download_model,
            commands::offline::get_download_status,
            commands::offline::is_model_loaded,
            commands::offline::load_model,
            commands::offline::delete_model,
            commands::offline::generate_text,
            commands::offline::check_network,
            commands::offline::get_offline_status,
            commands::offline::set_offline_mode
        ])
        .setup(|app| {
            // Register offline commands
            if let Err(e) = commands::offline::register_commands(app) {
                log_error!("main", "Failed to register offline commands: {}", e);
            }
        
            // Start resource monitor if feature is enabled
            if mcp_client::feature_enabled!(FLAG_RESOURCE_MONITORING) {
                let monitor = RESOURCE_MONITOR.lock().unwrap();
                monitor.start(1000); // Update every second
            }
            
            // Store app init time in window for frontend to access
            #[cfg(feature = "frontend")]
            {
                unsafe {
                    if let Some(start_time) = APP_START_TIME {
                        let elapsed = start_time.elapsed().as_millis() as f64;
                        
                        // Record startup time metric
                        let mut tags = HashMap::new();
                        tags.insert("type".to_string(), "backend".to_string());
                        record_counter("startup_time", elapsed, Some(tags));
                        
                        // Add to window for frontend to access
                        app.set_window_limits(app.get_window("main").unwrap(), None, None);
                        let window = app.get_window("main").unwrap();
                        window.eval(&format!("window.__APP_INIT_TIME__ = {}", elapsed)).unwrap();
                    }
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Initialize feature flags
fn initialize_feature_flags() {
    let feature_flags = vec![
        FeatureFlag {
            id: FLAG_ADVANCED_TELEMETRY.to_string(),
            name: "Advanced Telemetry".to_string(),
            description: "Enable advanced telemetry collection for detailed performance insights".to_string(),
            enabled: true,
            rollout_strategy: RolloutStrategy::CanaryGroup(CANARY_GROUP_ALPHA.to_string(), 0.5),
            dependencies: vec![],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        },
        FeatureFlag {
            id: FLAG_PERFORMANCE_DASHBOARD.to_string(),
            name: "Performance Dashboard".to_string(),
            description: "Access to the performance monitoring dashboard".to_string(),
            enabled: true,
            rollout_strategy: RolloutStrategy::CanaryGroup(CANARY_GROUP_BETA.to_string(),
            1.0),
            dependencies: vec![],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        },
        FeatureFlag {
            id: FLAG_DEBUG_LOGGING.to_string(),
            name: "Debug Logging".to_string(),
            description: "Enable verbose debug logging for troubleshooting".to_string(),
            enabled: true,
            rollout_strategy: RolloutStrategy::CanaryGroup(CANARY_GROUP_ALPHA.to_string(), 1.0),
            dependencies: vec![],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        },
        FeatureFlag {
            id: FLAG_RESOURCE_MONITORING.to_string(),
            name: "Resource Monitoring".to_string(),
            description: "Monitor system resource usage for the application".to_string(),
            enabled: true,
            rollout_strategy: RolloutStrategy::AllUsers,
            dependencies: vec![],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        },
        FeatureFlag {
            id: FLAG_CRASH_REPORTING.to_string(),
            name: "Crash Reporting".to_string(),
            description: "Automatically send crash reports for analysis".to_string(),
            enabled: true,
            rollout_strategy: RolloutStrategy::PercentageRollout(0.5),
            dependencies: vec![],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        },
    ];
    
    FEATURE_FLAG_MANAGER.load_flags(feature_flags);
    
    // Add features to canary groups
    let canary_service = CANARY_SERVICE.clone();
    let mut groups = canary_service.get_canary_groups();
    
    // Update alpha group
    if let Some(alpha_group) = groups.iter_mut().find(|g| g.name == CANARY_GROUP_ALPHA) {
        alpha_group.active_features = vec![
            FLAG_ADVANCED_TELEMETRY.to_string(),
            FLAG_DEBUG_LOGGING.to_string(),
        ];
    }
    
    // Update beta group
    if let Some(beta_group) = groups.iter_mut().find(|g| g.name == CANARY_GROUP_BETA) {
        beta_group.active_features = vec![
            FLAG_PERFORMANCE_DASHBOARD.to_string(),
        ];
    }
}

// Register shutdown handler
fn register_shutdown_handler(telemetry_client: Arc<TelemetryClient>) {
    ctrlc::set_handler(move || {
        // Log application exit
        log_info!("main", "MCP Client shutting down");
        
        // Track exit event
        telemetry_client.track_event(
            TelemetryEventType::ApplicationExit,
            "app_exit",
            None,
            None,
        );
        
        // Give telemetry a chance to send final batch
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Exit application
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
}