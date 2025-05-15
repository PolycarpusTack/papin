use std::sync::Arc;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};
use thiserror::Error;

// Re-export types from the monitoring module
use crate::common::monitoring::platform::{
    ResourceMetrics, ResourceThresholds, MonitoringError
};

// Import performance-related modules
use crate::common::monitoring::platform::get_resource_monitor;
use crate::common::performance::platform::get_performance_manager;

// Error types for performance commands
#[derive(Error, Debug, Serialize)]
pub enum PerformanceCommandError {
    #[error("Monitoring error: {0}")]
    MonitoringError(String),
    
    #[error("Performance manager error: {0}")]
    PerformanceError(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

// Implement From for the original error types to convert them
impl From<MonitoringError> for PerformanceCommandError {
    fn from(err: MonitoringError) -> Self {
        PerformanceCommandError::MonitoringError(err.to_string())
    }
}

// Hardware capabilities structure for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilitiesResponse {
    pub platform: String,
    pub cpu_features: CpuFeaturesResponse,
    pub memory_info: MemoryInfoResponse,
    pub gpu_info: GpuInfoResponse,
    pub supports_llm_acceleration: bool,
    pub recommended_model_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuFeaturesResponse {
    pub core_count: usize,
    pub avx: bool,
    pub avx2: bool,
    pub neon: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfoResponse {
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfoResponse {
    pub has_gpu: bool,
    pub gpu_name: String,
    pub vram_mb: Option<u64>,
    pub supports_compute: bool,
}

// Get current resource metrics
#[tauri::command]
pub async fn get_current_resource_metrics() -> Result<ResourceMetrics, PerformanceCommandError> {
    let monitor = get_resource_monitor();
    
    // Trigger an update of the metrics before returning them
    monitor.update_metrics()?;
    
    // Get and return the updated metrics
    let metrics = monitor.get_current_metrics()?;
    Ok(metrics)
}

// Get historic resource metrics
#[tauri::command]
pub async fn get_historic_resource_metrics() -> Result<Vec<ResourceMetrics>, PerformanceCommandError> {
    let monitor = get_resource_monitor();
    let metrics = monitor.get_historic_metrics()?;
    Ok(metrics)
}

// Get hardware capabilities
#[tauri::command]
pub async fn get_hardware_capabilities() -> Result<HardwareCapabilitiesResponse, PerformanceCommandError> {
    let perf_manager = get_performance_manager();
    let capabilities = perf_manager.get_capabilities();
    
    // Convert to the response structure for the frontend
    let response = HardwareCapabilitiesResponse {
        platform: format!("{:?}", capabilities.platform),
        cpu_features: CpuFeaturesResponse {
            core_count: capabilities.cpu_features.core_count,
            avx: capabilities.cpu_features.avx,
            avx2: capabilities.cpu_features.avx2,
            neon: capabilities.cpu_features.neon,
        },
        memory_info: MemoryInfoResponse {
            total_memory_mb: capabilities.memory_info.total_memory_mb,
            available_memory_mb: capabilities.memory_info.available_memory_mb,
        },
        gpu_info: GpuInfoResponse {
            has_gpu: capabilities.gpu_info.has_gpu,
            gpu_name: capabilities.gpu_info.gpu_name.clone(),
            vram_mb: capabilities.gpu_info.vram_mb,
            supports_compute: capabilities.gpu_info.supports_compute,
        },
        supports_llm_acceleration: capabilities.supports_llm_acceleration(),
        recommended_model_size: capabilities.recommended_llm_model_size().to_string(),
    };
    
    Ok(response)
}

// Get resource recommendations
#[tauri::command]
pub async fn get_resource_recommendations() -> Result<Vec<String>, PerformanceCommandError> {
    let monitor = get_resource_monitor();
    let recommendations = monitor.get_recommendations()?;
    Ok(recommendations)
}

// Set resource thresholds
#[tauri::command]
pub async fn set_resource_thresholds(thresholds: ResourceThresholds) -> Result<(), PerformanceCommandError> {
    let monitor = get_resource_monitor();
    monitor.set_thresholds(thresholds)?;
    Ok(())
}

// Check if a feature is supported by current hardware
#[tauri::command]
pub async fn is_feature_supported(feature: String) -> Result<bool, PerformanceCommandError> {
    let perf_manager = get_performance_manager();
    let supported = perf_manager.meets_requirements(&feature);
    Ok(supported)
}

// Get optimal thread pool size for current hardware
#[tauri::command]
pub async fn get_thread_pool_size() -> Result<usize, PerformanceCommandError> {
    let perf_manager = get_performance_manager();
    let size = perf_manager.get_thread_pool_size();
    Ok(size)
}

// Get memory settings
#[tauri::command]
pub async fn get_memory_settings() -> Result<(u64, u64), PerformanceCommandError> {
    let perf_manager = get_performance_manager();
    let (max, cache) = perf_manager.get_memory_settings();
    Ok((max, cache))
}

// Simulate high resource usage for testing
#[tauri::command]
pub async fn simulate_high_resource_usage() -> Result<(), PerformanceCommandError> {
    use tokio::task;
    
    // Create a CPU-intensive task
    let cpu_task = task::spawn(async {
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(10) {
            // CPU intensive operation
            for _ in 0..1000000 {
                let mut x = 0;
                for i in 0..1000 {
                    x += i;
                }
            }
        }
    });
    
    // Create a memory-intensive task
    let memory_task = task::spawn(async {
        // Allocate a large vector
        let mut data = Vec::with_capacity(1024 * 1024 * 100); // 100 MB
        for i in 0..data.capacity() {
            data.push(i as u8);
        }
        
        // Hold it for 10 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    });
    
    // Wait for both tasks to complete
    let _ = cpu_task.await;
    let _ = memory_task.await;
    
    Ok(())
}

// Register all performance-related commands with Tauri
pub fn register(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("Registering performance commands");
    
    // Ensure the resource monitor is initialized
    let monitor = get_resource_monitor();
    monitor.update_metrics().map_err(|e| {
        error!("Failed to initialize resource monitor: {}", e);
        Box::<dyn std::error::Error>::from(format!("Resource monitor init failed: {}", e))
    })?;
    
    // Commands are automatically registered via the #[tauri::command] macro
    Ok(())
}
