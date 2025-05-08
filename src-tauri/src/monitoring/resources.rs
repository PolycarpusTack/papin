use sysinfo::{System, SystemExt, ProcessExt, CpuExt};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, Instant};
use crate::observability::metrics;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceMetrics {
    pub timestamp: u64,
    pub cpu: f32,
    pub memory: u64,  // in bytes
    pub fps: Option<f32>,
    pub message_count: u32,
    pub api_latency: f64,  // in milliseconds
    pub api_calls: u32,
}

pub struct ResourceMonitor {
    system: Arc<Mutex<System>>,
    metrics_history: Arc<Mutex<Vec<ResourceMetrics>>>,
    max_history_size: usize,
    pid: u32,
    is_running: Arc<Mutex<bool>>,
    start_time: Instant,
}

impl ResourceMonitor {
    pub fn new(pid: u32, max_history_size: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let system = Arc::new(Mutex::new(system));
        let metrics_history = Arc::new(Mutex::new(Vec::with_capacity(max_history_size)));
        let is_running = Arc::new(Mutex::new(false));
        
        Self {
            system,
            metrics_history,
            max_history_size,
            pid,
            is_running,
            start_time: Instant::now(),
        }
    }
    
    pub fn start(&self, interval_ms: u64) {
        let system = Arc::clone(&self.system);
        let metrics_history = Arc::clone(&self.metrics_history);
        let is_running = Arc::clone(&self.is_running);
        let pid = self.pid;
        let max_history_size = self.max_history_size;
        
        // Set running state
        *is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            while *is_running.lock().unwrap() {
                // Refresh system info
                let mut system = system.lock().unwrap();
                system.refresh_all();
                
                // Get process info (our own process)
                let process = system.process(sysinfo::Pid::from(pid));
                
                if let Some(process) = process {
                    // Calculate CPU usage (as percentage)
                    let cpu_usage = process.cpu_usage();
                    
                    // Get memory usage (in bytes)
                    let memory_usage = process.memory();
                    
                    // Get current timestamp
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    // Create metrics record
                    let metrics = ResourceMetrics {
                        timestamp,
                        cpu: cpu_usage,
                        memory: memory_usage,
                        fps: None, // Will be updated from the UI
                        message_count: 0, // Will be updated from the message service
                        api_latency: 0.0, // Will be updated from the API service
                        api_calls: 0, // Will be updated from the API service
                    };
                    
                    // Record these metrics for telemetry as well
                    let mut tags = std::collections::HashMap::new();
                    tags.insert("pid".to_string(), pid.to_string());
                    metrics::record_gauge("cpu_usage", cpu_usage as f64, Some(tags.clone()));
                    metrics::record_gauge("memory_usage", memory_usage as f64, Some(tags));
                    
                    // Add to history
                    let mut history = metrics_history.lock().unwrap();
                    history.push(metrics);
                    
                    // Trim history if needed
                    if history.len() > max_history_size {
                        history.remove(0);
                    }
                }
                
                // Sleep for the specified interval
                drop(system);
                thread::sleep(Duration::from_millis(interval_ms));
            }
        });
    }
    
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
    }
    
    pub fn get_metrics(&self, time_range_ms: Option<u64>) -> Vec<ResourceMetrics> {
        let history = self.metrics_history.lock().unwrap();
        
        if let Some(time_range) = time_range_ms {
            // Calculate cutoff timestamp
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
                
            let cutoff = if now > time_range { now - time_range } else { 0 };
            
            // Filter history by timestamp
            history.iter()
                .filter(|metrics| metrics.timestamp >= cutoff)
                .cloned()
                .collect()
        } else {
            // Return all history
            history.clone()
        }
    }
    
    pub fn update_metrics(&self, 
        fps: Option<f32>, 
        message_count: Option<u32>, 
        api_latency: Option<f64>, 
        api_calls: Option<u32>
    ) {
        let mut history = self.metrics_history.lock().unwrap();
        
        if let Some(latest) = history.last_mut() {
            if let Some(fps_value) = fps {
                latest.fps = Some(fps_value);
            }
            
            if let Some(message_count_value) = message_count {
                latest.message_count = message_count_value;
            }
            
            if let Some(api_latency_value) = api_latency {
                latest.api_latency = api_latency_value;
            }
            
            if let Some(api_calls_value) = api_calls {
                latest.api_calls = api_calls_value;
            }
        }
    }
    
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn get_current_memory_usage(&self) -> u64 {
        let system = self.system.lock().unwrap();
        if let Some(process) = system.process(sysinfo::Pid::from(self.pid)) {
            process.memory()
        } else {
            0
        }
    }
    
    pub fn get_current_cpu_usage(&self) -> f32 {
        let system = self.system.lock().unwrap();
        if let Some(process) = system.process(sysinfo::Pid::from(self.pid)) {
            process.cpu_usage()
        } else {
            0.0
        }
    }
    
    pub fn get_system_info(&self) -> SystemInfo {
        let system = self.system.lock().unwrap();
        
        SystemInfo {
            total_memory: system.total_memory(),
            used_memory: system.used_memory(),
            total_swap: system.total_swap(),
            used_swap: system.used_swap(),
            cpu_count: system.cpus().len() as u32,
            system_name: system.name().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: system.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            os_version: system.os_version().unwrap_or_else(|| "Unknown".to_string()),
            host_name: system.host_name().unwrap_or_else(|| "Unknown".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_count: u32,
    pub system_name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
}

// Create a global resource monitor
lazy_static::lazy_static! {
    pub static ref RESOURCE_MONITOR: Mutex<ResourceMonitor> = {
        let pid = std::process::id();
        let monitor = ResourceMonitor::new(pid, 10000); // Store up to 10000 data points
        monitor.start(1000); // Update every second
        Mutex::new(monitor)
    };
}

// Tauri commands
#[tauri::command]
pub fn get_resource_metrics(time_range: Option<u64>) -> Vec<ResourceMetrics> {
    let monitor = RESOURCE_MONITOR.lock().unwrap();
    monitor.get_metrics(time_range)
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let monitor = RESOURCE_MONITOR.lock().unwrap();
    monitor.get_system_info()
}

#[tauri::command]
pub fn update_resource_metrics(
    fps: Option<f32>, 
    message_count: Option<u32>, 
    api_latency: Option<f64>, 
    api_calls: Option<u32>
) {
    let monitor = RESOURCE_MONITOR.lock().unwrap();
    monitor.update_metrics(fps, message_count, api_latency, api_calls);
}

#[tauri::command]
pub fn get_uptime() -> u64 {
    let monitor = RESOURCE_MONITOR.lock().unwrap();
    monitor.get_uptime().as_secs()
}

#[tauri::command]
pub fn report_startup_time(time_ms: f64) {
    let mut tags = std::collections::HashMap::new();
    tags.insert("type".to_string(), "frontend".to_string());
    metrics::record_histogram("startup_time", time_ms, Some(tags));
    
    println!("Frontend startup time: {:.2}ms", time_ms);
}

#[tauri::command]
pub fn report_frame_rate(fps: f32) {
    let monitor = RESOURCE_MONITOR.lock().unwrap();
    monitor.update_metrics(Some(fps), None, None, None);
    
    let mut tags = std::collections::HashMap::new();
    tags.insert("type".to_string(), "ui".to_string());
    metrics::record_gauge("fps", fps as f64, Some(tags));
}

#[tauri::command]
pub fn report_resource_metrics(
    resource_type: String,
    url: String,
    duration: f64,
    size: u64
) {
    let mut tags = std::collections::HashMap::new();
    tags.insert("type".to_string(), resource_type.clone());
    tags.insert("url".to_string(), url);
    
    metrics::record_histogram("resource_load_time", duration, Some(tags.clone()));
    metrics::record_histogram("resource_size", size as f64, Some(tags));
}

#[tauri::command]
pub fn report_page_metrics(
    load_time: f64,
    dom_content_loaded: f64,
    first_paint: f64
) {
    let mut tags = std::collections::HashMap::new();
    tags.insert("type".to_string(), "page".to_string());
    
    metrics::record_histogram("page_load_time", load_time, Some(tags.clone()));
    metrics::record_histogram("dom_content_loaded", dom_content_loaded, Some(tags.clone()));
    metrics::record_histogram("first_paint", first_paint, Some(tags));
}