use crate::commands::error::CommandError;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Runtime};
use std::collections::HashMap;
use log::{info, error};
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use num_cpus;
use crate::utils::safe_lock::SafeLock;

// Types for resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    cpu_usage: f64,
    memory_usage: f64,
    total_memory: u64,
    network_rx: u64,
    network_tx: u64,
    disk_read: u64,
    disk_write: u64,
    battery_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    cpu_cores: usize,
    logical_cores: usize,
    total_memory: u64,
    gpu_info: Option<GpuInfo>,
    platform: String,
    supports_metal: bool,
    supports_directml: bool,
    supports_opencl: bool,
    supports_cuda: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    name: String,
    vendor: String,
    memory: Option<u64>,
}

// Lazily initialized state
lazy_static! {
    static ref METRICS_HISTORY: Arc<RwLock<Vec<ResourceMetrics>>> = 
        Arc::new(RwLock::new(Vec::with_capacity(100)));
    
    static ref HARDWARE_INFO: Arc<RwLock<Option<HardwareCapabilities>>> =
        Arc::new(RwLock::new(None));
}

/// Register performance monitoring with the app
pub fn register<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    info!("Initializing performance monitoring");
    
    // Detect hardware capabilities once at startup
    let capabilities = detect_hardware_capabilities()?;
    
    // Store hardware info
    match HARDWARE_INFO.write() {
        Ok(mut info) => {
            *info = Some(capabilities.clone());
            info!("Detected hardware: {} cores, {} logical cores", 
                capabilities.cpu_cores, capabilities.logical_cores);
        },
        Err(e) => {
            error!("Failed to store hardware info: {}", e);
            return Err(format!("Failed to initialize hardware info: {}", e));
        }
    }
    
    // Start metrics collection thread
    std::thread::spawn(|| {
        collect_metrics_periodically();
    });
    
    Ok(())
}

// Detect hardware capabilities for the current platform
fn detect_hardware_capabilities() -> Result<HardwareCapabilities, String> {
    // Get CPU info
    let logical_cores = num_cpus::get();
    let physical_cores = num_cpus::get_physical();
    
    // Get platform info
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }.to_string();
    
    // Detect GPU capabilities based on platform
    let (gpu_info, supports_metal, supports_directml, supports_opencl, supports_cuda) = 
        detect_gpu_capabilities(&platform)?;
    
    // Get system memory
    let total_memory = get_total_memory()?;
    
    Ok(HardwareCapabilities {
        cpu_cores: physical_cores,
        logical_cores,
        total_memory,
        gpu_info,
        platform,
        supports_metal,
        supports_directml,
        supports_opencl,
        supports_cuda,
    })
}

// Platform-specific GPU detection
fn detect_gpu_capabilities(platform: &str) -> Result<(Option<GpuInfo>, bool, bool, bool, bool), String> {
    // Default values
    let mut gpu_info = None;
    let mut supports_metal = false;
    let mut supports_directml = false;
    let mut supports_opencl = false;
    let mut supports_cuda = false;
    
    match platform {
        "windows" => {
            // DirectML is supported on Windows 10+ with compatible GPUs
            supports_directml = true;
            
            // Windows-specific GPU detection would go here
            // This is simplified for the example
            gpu_info = Some(GpuInfo {
                name: "Generic Windows GPU".to_string(),
                vendor: "Unknown".to_string(),
                memory: None,
            });
        },
        "macos" => {
            // Metal is supported on all modern macOS systems
            supports_metal = true;
            
            // macOS-specific GPU detection would go here
            gpu_info = Some(GpuInfo {
                name: "Generic macOS GPU".to_string(),
                vendor: "Apple".to_string(),
                memory: None,
            });
        },
        "linux" => {
            // Linux might support OpenCL and/or CUDA depending on hardware
            supports_opencl = true;
            
            // Linux-specific GPU detection would go here
            gpu_info = Some(GpuInfo {
                name: "Generic Linux GPU".to_string(),
                vendor: "Unknown".to_string(),
                memory: None,
            });
        },
        _ => {
            return Err(format!("Unsupported platform: {}", platform));
        }
    }
    
    Ok((gpu_info, supports_metal, supports_directml, supports_opencl, supports_cuda))
}

// Get total system memory
fn get_total_memory() -> Result<u64, String> {
    // This is a simplified implementation
    // In a real implementation, use platform-specific APIs
    Ok(8 * 1024 * 1024 * 1024) // 8 GB as a placeholder
}

// Periodically collect metrics
fn collect_metrics_periodically() {
    let interval = std::time::Duration::from_secs(5); // Collect every 5 seconds
    
    loop {
        if let Ok(metrics) = collect_current_metrics() {
            // Store metrics in history
            if let Ok(mut history) = METRICS_HISTORY.write() {
                // Keep history size manageable
                if history.len() >= 100 {
                    history.remove(0);
                }
                history.push(metrics);
            }
        }
        
        std::thread::sleep(interval);
    }
}

// Collect current system metrics
fn collect_current_metrics() -> Result<ResourceMetrics, String> {
    // This is a simplified implementation
    // In a real implementation, use platform-specific APIs
    
    // Placeholder values
    Ok(ResourceMetrics {
        cpu_usage: 10.0,
        memory_usage: 2.0 * 1024.0 * 1024.0 * 1024.0, // 2 GB
        total_memory: 8 * 1024 * 1024 * 1024, // 8 GB
        network_rx: 1024 * 1024, // 1 MB
        network_tx: 512 * 1024, // 512 KB
        disk_read: 2 * 1024 * 1024, // 2 MB
        disk_write: 1 * 1024 * 1024, // 1 MB
        battery_percentage: Some(80.0),
    })
}

// Tauri commands

#[tauri::command]
pub async fn get_current_resource_metrics() -> Result<ResourceMetrics, CommandError> {
    match collect_current_metrics() {
        Ok(metrics) => Ok(metrics),
        Err(e) => Err(CommandError::InternalError(e)),
    }
}

#[tauri::command]
pub async fn get_historic_resource_metrics(
    limit: Option<usize>
) -> Result<Vec<ResourceMetrics>, CommandError> {
    let limit = limit.unwrap_or(100);
    
    match METRICS_HISTORY.read() {
        Ok(history) => {
            let metrics: Vec<ResourceMetrics> = history.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect();
            Ok(metrics)
        },
        Err(e) => Err(CommandError::InternalError(format!("Failed to read metrics history: {}", e))),
    }
}

#[tauri::command]
pub async fn get_hardware_capabilities() -> Result<HardwareCapabilities, CommandError> {
    match HARDWARE_INFO.read() {
        Ok(info) => {
            if let Some(capabilities) = info.clone() {
                Ok(capabilities)
            } else {
                Err(CommandError::InternalError("Hardware info not initialized".to_string()))
            }
        },
        Err(e) => Err(CommandError::InternalError(format!("Failed to read hardware info: {}", e))),
    }
}

#[tauri::command]
pub async fn get_resource_recommendations() -> Result<HashMap<String, String>, CommandError> {
    let mut recommendations = HashMap::new();
    
    // Get hardware capabilities
    let capabilities = match HARDWARE_INFO.read() {
        Ok(info) => {
            if let Some(capabilities) = info.clone() {
                capabilities
            } else {
                return Err(CommandError::InternalError("Hardware info not initialized".to_string()));
            }
        },
        Err(e) => return Err(CommandError::InternalError(format!("Failed to read hardware info: {}", e))),
    };
    
    // Generate recommendations based on hardware
    if capabilities.logical_cores < 4 {
        recommendations.insert(
            "thread_pool".to_string(),
            "Your CPU has limited cores. Consider reducing the thread pool size.".to_string()
        );
    }
    
    match capabilities.platform.as_str() {
        "windows" => {
            if capabilities.supports_directml {
                recommendations.insert(
                    "acceleration".to_string(),
                    "DirectML acceleration is available. Enable it for faster LLM inference.".to_string()
                );
            }
        },
        "macos" => {
            if capabilities.supports_metal {
                recommendations.insert(
                    "acceleration".to_string(),
                    "Metal acceleration is available. Enable it for faster LLM inference.".to_string()
                );
            }
        },
        "linux" => {
            if capabilities.supports_opencl {
                recommendations.insert(
                    "acceleration".to_string(),
                    "OpenCL acceleration is available. Enable it for faster LLM inference.".to_string()
                );
            }
            if capabilities.supports_cuda {
                recommendations.insert(
                    "acceleration".to_string(),
                    "CUDA acceleration is available. Enable it for optimal performance.".to_string()
                );
            }
        },
        _ => {}
    }
    
    Ok(recommendations)
}

#[tauri::command]
pub async fn is_feature_supported(feature: String) -> Result<bool, CommandError> {
    let capabilities = match HARDWARE_INFO.read() {
        Ok(info) => {
            if let Some(capabilities) = info.clone() {
                capabilities
            } else {
                return Err(CommandError::InternalError("Hardware info not initialized".to_string()));
            }
        },
        Err(e) => return Err(CommandError::InternalError(format!("Failed to read hardware info: {}", e))),
    };
    
    match feature.as_str() {
        "metal" => Ok(capabilities.supports_metal),
        "directml" => Ok(capabilities.supports_directml),
        "opencl" => Ok(capabilities.supports_opencl),
        "cuda" => Ok(capabilities.supports_cuda),
        _ => Err(CommandError::ValidationError(format!("Unknown feature: {}", feature))),
    }
}

#[tauri::command]
pub async fn get_thread_pool_size() -> Result<usize, CommandError> {
    let capabilities = match HARDWARE_INFO.read() {
        Ok(info) => {
            if let Some(capabilities) = info.clone() {
                capabilities
            } else {
                return Err(CommandError::InternalError("Hardware info not initialized".to_string()));
            }
        },
        Err(e) => return Err(CommandError::InternalError(format!("Failed to read hardware info: {}", e))),
    };
    
    // Recommend thread pool size based on available cores
    // Typically use physical cores for compute-intensive tasks
    Ok(capabilities.cpu_cores)
}

#[tauri::command]
pub async fn get_memory_settings() -> Result<HashMap<String, u64>, CommandError> {
    let capabilities = match HARDWARE_INFO.read() {
        Ok(info) => {
            if let Some(capabilities) = info.clone() {
                capabilities
            } else {
                return Err(CommandError::InternalError("Hardware info not initialized".to_string()));
            }
        },
        Err(e) => return Err(CommandError::InternalError(format!("Failed to read hardware info: {}", e))),
    };
    
    let mut settings = HashMap::new();
    
    // Calculate memory limits based on available memory
    let total_memory = capabilities.total_memory;
    
    // Recommend using no more than 40% of memory for LLM
    let llm_limit = (total_memory as f64 * 0.4) as u64;
    settings.insert("llm_memory_limit".to_string(), llm_limit);
    
    // Cache size recommendation
    let cache_size = (total_memory as f64 * 0.1) as u64;
    settings.insert("cache_memory_limit".to_string(), cache_size);
    
    Ok(settings)
}