use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use log::{info, warn, debug, error};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::performance::platform::{get_performance_manager, HardwareCapabilities, PlatformType};
use crate::utils::locks::{SafeLock, SafeRwLock, LockError};

// Resource monitoring errors
#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Lock acquisition failed: {0}")]
    LockError(String),
    
    #[error("Failed to read system metrics: {0}")]
    MetricsReadError(String),
    
    #[error("Performance threshold exceeded: {0}")]
    ThresholdExceeded(String),
    
    #[error("Resource collection not available on this platform: {0}")]
    PlatformUnsupported(String),
}

// Handle lock errors
impl From<LockError> for MonitoringError {
    fn from(err: LockError) -> Self {
        MonitoringError::LockError(err.to_string())
    }
}

// Resource metrics collected for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub timestamp: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub available_memory_mb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_recv_bytes: u64,
    pub network_sent_bytes: u64,
    pub battery_percent: Option<u8>,
    pub is_on_battery: bool,
    pub is_throttled: bool,
}

impl ResourceMetrics {
    // Create new empty metrics
    pub fn new() -> Self {
        ResourceMetrics {
            timestamp: chrono::Utc::now().timestamp() as u64,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            available_memory_mb: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            network_recv_bytes: 0,
            network_sent_bytes: 0,
            battery_percent: None,
            is_on_battery: false,
            is_throttled: false,
        }
    }
}

// Resource monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    pub cpu_warning_percent: f32,
    pub cpu_critical_percent: f32,
    pub memory_warning_percent: f32,
    pub memory_critical_percent: f32,
    pub disk_warning_mb_per_sec: f32,
    pub battery_warning_percent: u8,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        ResourceThresholds {
            cpu_warning_percent: 80.0,
            cpu_critical_percent: 95.0,
            memory_warning_percent: 85.0,
            memory_critical_percent: 95.0,
            disk_warning_mb_per_sec: 100.0,
            battery_warning_percent: 15,
        }
    }
}

// Platform-specific resource monitor
pub struct ResourceMonitor {
    current_metrics: RwLock<ResourceMetrics>,
    historic_metrics: Mutex<Vec<ResourceMetrics>>,
    thresholds: RwLock<ResourceThresholds>,
    last_update: Mutex<Instant>,
    update_interval: Duration,
    max_historic_entries: usize,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        ResourceMonitor {
            current_metrics: RwLock::new(ResourceMetrics::new()),
            historic_metrics: Mutex::new(Vec::with_capacity(100)),
            thresholds: RwLock::new(ResourceThresholds::default()),
            last_update: Mutex::new(Instant::now()),
            update_interval: Duration::from_secs(5),
            max_historic_entries: 100,
        }
    }
    
    // Set custom thresholds
    pub fn set_thresholds(&self, thresholds: ResourceThresholds) -> Result<(), MonitoringError> {
        let mut guard = self.thresholds.safe_write()?;
        *guard = thresholds;
        Ok(())
    }
    
    // Get current thresholds
    pub fn get_thresholds(&self) -> Result<ResourceThresholds, MonitoringError> {
        let guard = self.thresholds.safe_read()?;
        Ok(guard.clone())
    }
    
    // Update resource metrics
    pub fn update_metrics(&self) -> Result<(), MonitoringError> {
        let mut last_update = self.last_update.safe_lock()?;
        
        let now = Instant::now();
        if now.duration_since(*last_update) < self.update_interval {
            // Too soon to update again
            return Ok(());
        }
        
        *last_update = now;
        
        // Get platform-specific metrics
        let metrics = self.collect_platform_metrics()?;
        
        // Update current metrics
        {
            let mut current = self.current_metrics.safe_write()?;
            *current = metrics.clone();
        }
        
        // Add to historic metrics
        {
            let mut historic = self.historic_metrics.safe_lock()?;
            
            historic.push(metrics);
            
            // Maintain maximum size
            if historic.len() > self.max_historic_entries {
                historic.remove(0);
            }
        }
        
        // Check for threshold violations
        self.check_thresholds()?;
        
        Ok(())
    }
    
    // Get current metrics
    pub fn get_current_metrics(&self) -> Result<ResourceMetrics, MonitoringError> {
        let current = self.current_metrics.safe_read()?;
        Ok(current.clone())
    }
    
    // Get historic metrics
    pub fn get_historic_metrics(&self) -> Result<Vec<ResourceMetrics>, MonitoringError> {
        let historic = self.historic_metrics.safe_lock()?;
        Ok(historic.clone())
    }
    
    // Collect platform-specific metrics
    fn collect_platform_metrics(&self) -> Result<ResourceMetrics, MonitoringError> {
        let perf_manager = get_performance_manager();
        let capabilities = perf_manager.get_capabilities();
        
        let mut metrics = ResourceMetrics::new();
        
        match capabilities.platform {
            PlatformType::Windows => {
                // Windows-specific metric collection
                self.collect_windows_metrics(&mut metrics)?;
            },
            PlatformType::MacOS => {
                // macOS-specific metric collection
                self.collect_macos_metrics(&mut metrics)?;
            },
            PlatformType::Linux => {
                // Linux-specific metric collection
                self.collect_linux_metrics(&mut metrics)?;
            },
            _ => {
                warn!("Platform not specifically supported for detailed metrics");
                // Use generic metrics for unknown platforms
                self.collect_generic_metrics(&mut metrics)?;
            }
        }
        
        Ok(metrics)
    }
    
    // Windows-specific metric collection
    #[cfg(target_os = "windows")]
    fn collect_windows_metrics(&self, metrics: &mut ResourceMetrics) -> Result<(), MonitoringError> {
        // This would use Windows Performance Counters, WMI, etc.
        // For simplicity in this example, we'll use placeholder values
        metrics.cpu_usage_percent = 30.0;
        metrics.memory_usage_mb = 4096;
        metrics.available_memory_mb = 12288;
        metrics.disk_read_bytes = 1024 * 1024 * 10;  // 10 MB
        metrics.disk_write_bytes = 1024 * 1024 * 5;   // 5 MB
        metrics.network_recv_bytes = 1024 * 1024 * 2; // 2 MB
        metrics.network_sent_bytes = 1024 * 1024 * 1; // 1 MB
        
        // Battery status
        metrics.battery_percent = Some(75);
        metrics.is_on_battery = false;
        metrics.is_throttled = false;
        
        Ok(())
    }
    
    // macOS-specific metric collection
    #[cfg(target_os = "macos")]
    fn collect_macos_metrics(&self, metrics: &mut ResourceMetrics) -> Result<(), MonitoringError> {
        // This would use sysctl, IOKit, etc.
        // For simplicity in this example, we'll use placeholder values
        metrics.cpu_usage_percent = 25.0;
        metrics.memory_usage_mb = 8192;
        metrics.available_memory_mb = 8192;
        metrics.disk_read_bytes = 1024 * 1024 * 8;   // 8 MB
        metrics.disk_write_bytes = 1024 * 1024 * 3;  // 3 MB
        metrics.network_recv_bytes = 1024 * 1024 * 3; // 3 MB
        metrics.network_sent_bytes = 1024 * 1024 * 2; // 2 MB
        
        // Battery status
        metrics.battery_percent = Some(80);
        metrics.is_on_battery = true;
        metrics.is_throttled = false;
        
        Ok(())
    }
    
    // Linux-specific metric collection
    #[cfg(target_os = "linux")]
    fn collect_linux_metrics(&self, metrics: &mut ResourceMetrics) -> Result<(), MonitoringError> {
        // This would use /proc filesystem, sysinfo, etc.
        // For simplicity in this example, we'll use placeholder values
        metrics.cpu_usage_percent = 20.0;
        metrics.memory_usage_mb = 4096;
        metrics.available_memory_mb = 12288;
        metrics.disk_read_bytes = 1024 * 1024 * 12;  // 12 MB
        metrics.disk_write_bytes = 1024 * 1024 * 6;   // 6 MB
        metrics.network_recv_bytes = 1024 * 1024 * 4; // 4 MB
        metrics.network_sent_bytes = 1024 * 1024 * 2; // 2 MB
        
        // Battery status (might not be available on all Linux systems)
        metrics.battery_percent = Some(65);
        metrics.is_on_battery = false;
        metrics.is_throttled = false;
        
        Ok(())
    }
    
    // Generic/fallback metric collection
    fn collect_generic_metrics(&self, metrics: &mut ResourceMetrics) -> Result<(), MonitoringError> {
        // Use very basic metrics that should be available on most platforms
        metrics.cpu_usage_percent = 50.0; // Conservative estimate
        metrics.memory_usage_mb = 4096;   // Placeholder
        metrics.available_memory_mb = 4096; // Placeholder
        
        // Other metrics left at default 0 values
        
        Ok(())
    }
    
    // Check for threshold violations
    fn check_thresholds(&self) -> Result<(), MonitoringError> {
        let metrics = self.get_current_metrics()?;
        let thresholds = self.get_thresholds()?;
        
        // Check CPU usage
        if metrics.cpu_usage_percent >= thresholds.cpu_critical_percent {
            error!("Critical CPU usage: {}%", metrics.cpu_usage_percent);
            // Here we could trigger an event, alert, etc.
            return Err(MonitoringError::ThresholdExceeded(
                format!("CPU usage critical: {}%", metrics.cpu_usage_percent)
            ));
        } else if metrics.cpu_usage_percent >= thresholds.cpu_warning_percent {
            warn!("High CPU usage: {}%", metrics.cpu_usage_percent);
        }
        
        // Check memory usage
        let total_memory = metrics.memory_usage_mb + metrics.available_memory_mb;
        let memory_percent = (metrics.memory_usage_mb as f32 / total_memory as f32) * 100.0;
        
        if memory_percent >= thresholds.memory_critical_percent {
            error!("Critical memory usage: {}%", memory_percent);
            return Err(MonitoringError::ThresholdExceeded(
                format!("Memory usage critical: {}%", memory_percent)
            ));
        } else if memory_percent >= thresholds.memory_warning_percent {
            warn!("High memory usage: {}%", memory_percent);
        }
        
        // Check battery if available
        if let Some(battery) = metrics.battery_percent {
            if battery <= thresholds.battery_warning_percent && metrics.is_on_battery {
                warn!("Low battery: {}%", battery);
            }
        }
        
        Ok(())
    }
    
    // Get resource recommendations for current state
    pub fn get_recommendations(&self) -> Result<Vec<String>, MonitoringError> {
        let metrics = self.get_current_metrics()?;
        let mut recommendations = Vec::new();
        
        // CPU recommendations
        if metrics.cpu_usage_percent >= 80.0 {
            recommendations.push("Consider closing unused applications to reduce CPU load.".to_string());
            recommendations.push("Reduce the number of concurrent tasks.".to_string());
        }
        
        // Memory recommendations
        let total_memory = metrics.memory_usage_mb + metrics.available_memory_mb;
        let memory_percent = (metrics.memory_usage_mb as f32 / total_memory as f32) * 100.0;
        
        if memory_percent >= 85.0 {
            recommendations.push("Free up memory by closing unused applications.".to_string());
            recommendations.push("Consider reducing LLM model size or context length.".to_string());
        }
        
        // Battery recommendations
        if let Some(battery) = metrics.battery_percent {
            if battery <= 20 && metrics.is_on_battery {
                recommendations.push("Connect to power to avoid interruption.".to_string());
                if battery <= 10 {
                    recommendations.push("Enable power saving mode to extend battery life.".to_string());
                }
            }
        }
        
        // Platform-specific recommendations
        let perf_manager = get_performance_manager();
        let capabilities = perf_manager.get_capabilities();
        
        match capabilities.platform {
            PlatformType::Windows => {
                if metrics.cpu_usage_percent >= 70.0 {
                    recommendations.push("Check Windows Task Manager for resource-intensive applications.".to_string());
                }
            },
            PlatformType::MacOS => {
                if metrics.is_throttled {
                    recommendations.push("Your Mac may be thermal throttling. Ensure adequate ventilation.".to_string());
                }
            },
            PlatformType::Linux => {
                if memory_percent >= 90.0 {
                    recommendations.push("Linux systems may experience OOM killer at high memory usage.".to_string());
                }
            },
            _ => {}
        }
        
        Ok(recommendations)
    }
}

// Global resource monitor instance
lazy_static::lazy_static! {
    static ref RESOURCE_MONITOR: Arc<ResourceMonitor> = Arc::new(ResourceMonitor::new());
}

// Access the global resource monitor
pub fn get_resource_monitor() -> Arc<ResourceMonitor> {
    RESOURCE_MONITOR.clone()
}
