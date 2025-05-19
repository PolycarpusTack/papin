use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{info, warn, debug, error};

// Platform-specific CPU feature detection
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub avx: bool,
    pub avx2: bool,
    pub avx512: bool,
    pub sse4: bool,
    pub neon: bool, // For ARM processors
    pub core_count: usize,
}

// Platform-specific memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
}

// Platform-specific GPU information
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub has_gpu: bool,
    pub gpu_name: String,
    pub vram_mb: Option<u64>,
    pub supports_compute: bool,
}

// Platform-specific thermal information
#[derive(Debug, Clone)]
pub struct ThermalInfo {
    pub cpu_temperature: Option<f32>, // Celsius
    pub gpu_temperature: Option<f32>, // Celsius
    pub fan_speed_percent: Option<u8>,
}

// Combined hardware capabilities struct
#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub cpu_features: CpuFeatures,
    pub memory_info: MemoryInfo,
    pub gpu_info: GpuInfo,
    pub thermal_info: ThermalInfo,
    pub platform: PlatformType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformType {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

impl HardwareCapabilities {
    // Create a new HardwareCapabilities instance with platform-specific detection
    pub fn detect() -> Self {
        let platform = detect_platform();
        let cpu_features = detect_cpu_features();
        let memory_info = get_memory_info();
        let gpu_info = detect_gpu_capabilities();
        let thermal_info = get_thermal_info();

        HardwareCapabilities {
            cpu_features,
            memory_info,
            gpu_info,
            thermal_info,
            platform,
        }
    }

    // Determine if the current hardware supports LLM acceleration
    pub fn supports_llm_acceleration(&self) -> bool {
        // Check for minimum requirements for local LLM inference
        let has_sufficient_memory = self.memory_info.total_memory_mb >= 8192; // At least 8GB RAM
        let has_avx = self.cpu_features.avx;
        
        // GPU acceleration check
        let has_gpu_acceleration = self.gpu_info.has_gpu && self.gpu_info.supports_compute;
        
        has_sufficient_memory && (has_avx || has_gpu_acceleration)
    }

    // Get recommended LLM model size based on hardware capabilities
    pub fn recommended_llm_model_size(&self) -> &'static str {
        if !self.supports_llm_acceleration() {
            return "none"; // Hardware doesn't meet minimum requirements
        }

        let memory_gb = self.memory_info.total_memory_mb / 1024;
        
        if self.gpu_info.has_gpu && self.gpu_info.supports_compute {
            if let Some(vram) = self.gpu_info.vram_mb {
                let vram_gb = vram / 1024;
                
                if vram_gb >= 12 {
                    return "large"; // 7B+ parameter models
                } else if vram_gb >= 6 {
                    return "medium"; // 3-7B parameter models
                } else if vram_gb >= 2 {
                    return "small"; // 1-3B parameter models
                }
            }
        }
        
        // CPU-based recommendation
        if memory_gb >= 16 && self.cpu_features.avx2 {
            return "medium"; // 3-7B parameter models
        } else if memory_gb >= 8 && self.cpu_features.avx {
            return "small"; // 1-3B parameter models
        } else {
            return "tiny"; // <1B parameter models
        }
    }

    // Get recommended batch size for LLM inference
    pub fn recommended_batch_size(&self) -> usize {
        if !self.supports_llm_acceleration() {
            return 1; // Minimum if hardware doesn't meet requirements
        }

        let memory_gb = self.memory_info.total_memory_mb / 1024;
        
        if self.gpu_info.has_gpu && self.gpu_info.supports_compute {
            if let Some(vram) = self.gpu_info.vram_mb {
                let vram_gb = vram / 1024;
                
                if vram_gb >= 16 {
                    return 8;
                } else if vram_gb >= 8 {
                    return 4;
                } else if vram_gb >= 4 {
                    return 2;
                }
            }
        }
        
        // CPU-based batch size
        if memory_gb >= 32 {
            return 4;
        } else if memory_gb >= 16 {
            return 2;
        } else {
            return 1;
        }
    }

    // Get recommended thread count for LLM inference
    pub fn recommended_thread_count(&self) -> usize {
        // Start with core count
        let cores = self.cpu_features.core_count;
        
        // Use different thread strategies based on platform
        match self.platform {
            PlatformType::Windows => {
                if cores > 16 {
                    cores - 4  // Leave 4 cores for system on high-core Windows machines
                } else if cores > 8 {
                    cores - 2  // Leave 2 cores for system on mid-range Windows machines
                } else {
                    cores / 2  // Half the cores for low-end machines
                }
            },
            PlatformType::MacOS => {
                // macOS has good thread management, but we'll still be conservative
                if cores > 8 {
                    cores - 2
                } else {
                    cores - 1
                }
            },
            PlatformType::Linux => {
                // Linux is efficient but still needs some headroom
                if cores > 16 {
                    cores - 2
                } else if cores > 8 {
                    cores - 1
                } else {
                    cores
                }
            },
            PlatformType::Unknown => cores / 2, // Be conservative for unknown platforms
        }
    }

    // Get memory limit based on platform and capabilities
    pub fn memory_limit_mb(&self) -> u64 {
        let total = self.memory_info.total_memory_mb;
        let available = self.memory_info.available_memory_mb;
        
        // Different memory strategies based on platform
        match self.platform {
            PlatformType::Windows => {
                // Windows often needs more memory headroom
                let limit = (available as f64 * 0.7) as u64;
                limit.min(total / 2) // Never use more than half of total memory
            },
            PlatformType::MacOS => {
                // macOS memory pressure system works well, can be more aggressive
                let limit = (available as f64 * 0.8) as u64;
                limit.min(total * 3 / 4) // Up to 75% of total memory
            },
            PlatformType::Linux => {
                // Linux can handle memory pressure well
                let limit = (available as f64 * 0.8) as u64;
                limit.min(total * 2 / 3) // Up to 66% of total memory
            },
            PlatformType::Unknown => {
                // Be conservative for unknown platforms
                let limit = (available as f64 * 0.6) as u64;
                limit.min(total / 2)
            },
        }
    }
}

// Platform detection
fn detect_platform() -> PlatformType {
    #[cfg(target_os = "windows")]
    return PlatformType::Windows;
    
    #[cfg(target_os = "macos")]
    return PlatformType::MacOS;
    
    #[cfg(target_os = "linux")]
    return PlatformType::Linux;
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return PlatformType::Unknown;
}

// Platform-specific CPU feature detection
fn detect_cpu_features() -> CpuFeatures {
    #[cfg(target_arch = "x86_64")]
    {
        // This would use actual CPU feature detection like is_x86_feature_detected!
        // For now, we'll create a placeholder implementation
        let avx = true; // Placeholder
        let avx2 = true; // Placeholder
        let avx512 = false; // Placeholder
        let sse4 = true; // Placeholder
        
        #[cfg(target_os = "windows")]
        let core_count = {
            // On Windows, use GetSystemInfo or similar
            // For this example, we'll use num_cpus crate's approach
            let cores = num_cpus::get();
            debug!("Detected {} CPU cores on Windows", cores);
            cores
        };
        
        #[cfg(target_os = "macos")]
        let core_count = {
            // On macOS, use sysctl
            let cores = num_cpus::get();
            debug!("Detected {} CPU cores on macOS", cores);
            cores
        };
        
        #[cfg(target_os = "linux")]
        let core_count = {
            // On Linux, check /proc/cpuinfo
            let cores = num_cpus::get();
            debug!("Detected {} CPU cores on Linux", cores);
            cores
        };
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let core_count = {
            let cores = num_cpus::get();
            debug!("Detected {} CPU cores on unknown platform", cores);
            cores
        };
        
        CpuFeatures {
            avx,
            avx2,
            avx512,
            sse4,
            neon: false, // x86_64 doesn't have NEON
            core_count,
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 specific detection
        let neon = true; // Most ARM64 processors have NEON
        
        #[cfg(target_os = "macos")]
        let core_count = {
            // Apple Silicon specific detection
            let cores = num_cpus::get();
            debug!("Detected {} CPU cores on Apple Silicon", cores);
            cores
        };
        
        #[cfg(not(target_os = "macos"))]
        let core_count = {
            // Other ARM platforms
            let cores = num_cpus::get();
            debug!("Detected {} CPU cores on ARM", cores);
            cores
        };
        
        CpuFeatures {
            avx: false, // ARM doesn't have AVX
            avx2: false,
            avx512: false,
            sse4: false,
            neon,
            core_count,
        }
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // Default for other architectures
        debug!("Unsupported CPU architecture for feature detection");
        CpuFeatures {
            avx: false,
            avx2: false,
            avx512: false,
            sse4: false,
            neon: false,
            core_count: num_cpus::get(),
        }
    }
}

// Platform-specific memory information retrieval
fn get_memory_info() -> MemoryInfo {
    // Placeholder implementation - would use platform-specific APIs
    #[cfg(target_os = "windows")]
    {
        // On Windows, use GlobalMemoryStatusEx
        // Placeholder values
        let total_memory_mb = 16384; // 16GB
        let available_memory_mb = 8192; // 8GB
        debug!("Memory info on Windows: {}MB total, {}MB available", 
               total_memory_mb, available_memory_mb);
        
        MemoryInfo {
            total_memory_mb,
            available_memory_mb,
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, use sysctl
        // Placeholder values
        let total_memory_mb = 16384; // 16GB
        let available_memory_mb = 8192; // 8GB
        debug!("Memory info on macOS: {}MB total, {}MB available", 
               total_memory_mb, available_memory_mb);
        
        MemoryInfo {
            total_memory_mb,
            available_memory_mb,
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // On Linux, use /proc/meminfo
        // Placeholder values
        let total_memory_mb = 16384; // 16GB
        let available_memory_mb = 8192; // 8GB
        debug!("Memory info on Linux: {}MB total, {}MB available", 
               total_memory_mb, available_memory_mb);
        
        MemoryInfo {
            total_memory_mb,
            available_memory_mb,
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Default for other platforms
        let total_memory_mb = 8192; // Assume 8GB
        let available_memory_mb = 4096; // Assume 4GB
        debug!("Memory info on unknown platform: assuming {}MB total, {}MB available", 
               total_memory_mb, available_memory_mb);
        
        MemoryInfo {
            total_memory_mb,
            available_memory_mb,
        }
    }
}

// Platform-specific GPU detection
fn detect_gpu_capabilities() -> GpuInfo {
    // Placeholder implementation - would use platform-specific APIs
    #[cfg(target_os = "windows")]
    {
        // On Windows, use DXGI or similar
        debug!("Detecting GPU on Windows");
        GpuInfo {
            has_gpu: true,
            gpu_name: "Generic GPU".to_string(),
            vram_mb: Some(4096), // 4GB
            supports_compute: true,
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, use Metal or similar
        debug!("Detecting GPU on macOS");
        GpuInfo {
            has_gpu: true,
            gpu_name: "Apple GPU".to_string(),
            vram_mb: None, // Shared memory on many Apple devices
            supports_compute: true,
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // On Linux, check for OpenCL or CUDA
        debug!("Detecting GPU on Linux");
        GpuInfo {
            has_gpu: true,
            gpu_name: "Generic Linux GPU".to_string(),
            vram_mb: Some(2048), // 2GB
            supports_compute: true,
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Default for other platforms
        debug!("GPU detection not implemented for this platform");
        GpuInfo {
            has_gpu: false,
            gpu_name: "Unknown".to_string(),
            vram_mb: None,
            supports_compute: false,
        }
    }
}

// Platform-specific thermal information
fn get_thermal_info() -> ThermalInfo {
    // Placeholder implementation - would use platform-specific APIs
    #[cfg(target_os = "windows")]
    {
        // On Windows, possibly use WMI
        debug!("Getting thermal info on Windows");
        ThermalInfo {
            cpu_temperature: Some(45.0),
            gpu_temperature: Some(55.0),
            fan_speed_percent: Some(30),
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, use SMC
        debug!("Getting thermal info on macOS");
        ThermalInfo {
            cpu_temperature: Some(40.0),
            gpu_temperature: None, // Integrated GPU might not report separately
            fan_speed_percent: Some(25),
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // On Linux, check for sensors
        debug!("Getting thermal info on Linux");
        ThermalInfo {
            cpu_temperature: Some(50.0),
            gpu_temperature: Some(60.0),
            fan_speed_percent: None, // Might not be accessible
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Default for other platforms
        debug!("Thermal info not implemented for this platform");
        ThermalInfo {
            cpu_temperature: None,
            gpu_temperature: None,
            fan_speed_percent: None,
        }
    }
}

// Platform-optimized performance manager
pub struct PerformanceManager {
    hardware_capabilities: HardwareCapabilities,
    last_adaptive_check: Mutex<Instant>,
    adaptive_check_interval: Duration,
}

impl PerformanceManager {
    pub fn new() -> Self {
        let hardware_capabilities = HardwareCapabilities::detect();
        
        // Log detected capabilities
        info!("Detected hardware capabilities: {:?}", hardware_capabilities);
        
        PerformanceManager {
            hardware_capabilities,
            last_adaptive_check: Mutex::new(Instant::now()),
            adaptive_check_interval: Duration::from_secs(60), // Check every minute
        }
    }
    
    // Get hardware capabilities
    pub fn get_capabilities(&self) -> &HardwareCapabilities {
        &self.hardware_capabilities
    }
    
    // Check if hardware meets minimum requirements for a feature
    pub fn meets_requirements(&self, feature: &str) -> bool {
        match feature {
            "offline_llm" => self.hardware_capabilities.supports_llm_acceleration(),
            "realtime_transcription" => {
                // Check if hardware can handle real-time audio transcription
                let has_sufficient_cpu = self.hardware_capabilities.cpu_features.core_count >= 4;
                has_sufficient_cpu
            },
            "advanced_ui" => {
                // Check if hardware can handle advanced UI features
                self.hardware_capabilities.gpu_info.has_gpu
            },
            _ => {
                warn!("Unknown feature requirement check: {}", feature);
                false
            }
        }
    }
    
    // Get appropriate thread pool size based on current system state
    pub fn get_thread_pool_size(&self) -> usize {
        let base_threads = self.hardware_capabilities.recommended_thread_count();
        
        // Check thermal state and adjust if needed
        if let Some(cpu_temp) = self.hardware_capabilities.thermal_info.cpu_temperature {
            if cpu_temp > 80.0 {
                // CPU is very hot, reduce threads
                return (base_threads / 2).max(1);
            } else if cpu_temp > 70.0 {
                // CPU is hot, slightly reduce threads
                return (base_threads * 3 / 4).max(1);
            }
        }
        
        base_threads
    }
    
    // Check if adaptive optimizations should run and update parameters if needed
    pub fn update_adaptive_optimizations(&self) -> bool {
        use crate::utils::locks::SafeLock;
        
        let mutex_result = self.last_adaptive_check.safe_lock();
        
        if let Ok(mut last_check) = mutex_result {
            let now = Instant::now();
            
            if now.duration_since(*last_check) >= self.adaptive_check_interval {
                *last_check = now;
                
                // Perform adaptive checks
                // This would typically refresh memory info, thermal info, etc.
                
                true
            } else {
                false
            }
        } else {
            // If we can't acquire the lock, log the error but don't panic
            error!("Failed to acquire lock for adaptive optimization check: {:?}", mutex_result.err());
            false
        }
    }
    
    // Get platform-specific optimal batch size for bulk operations
    pub fn get_optimal_batch_size(&self, operation_type: &str) -> usize {
        match operation_type {
            "message_processing" => {
                match self.hardware_capabilities.platform {
                    PlatformType::Windows => 100,
                    PlatformType::MacOS => 200,   // macOS has better I/O scheduling
                    PlatformType::Linux => 150,
                    PlatformType::Unknown => 50,  // Conservative default
                }
            },
            "file_operations" => {
                match self.hardware_capabilities.platform {
                    PlatformType::Windows => 50,  // Windows tends to lock files
                    PlatformType::MacOS => 100,
                    PlatformType::Linux => 200,   // Linux has better file handling
                    PlatformType::Unknown => 25,  // Conservative default
                }
            },
            _ => 50, // Default batch size
        }
    }
    
    // Get platform-optimized memory settings (in MB)
    pub fn get_memory_settings(&self) -> (u64, u64) {
        let max_memory = self.hardware_capabilities.memory_limit_mb();
        let cache_memory = max_memory / 4; // 25% for cache
        
        (max_memory, cache_memory)
    }
}

// Global performance manager instance
lazy_static::lazy_static! {
    static ref PERFORMANCE_MANAGER: Arc<PerformanceManager> = Arc::new(PerformanceManager::new());
}

// Access the global performance manager
pub fn get_performance_manager() -> Arc<PerformanceManager> {
    PERFORMANCE_MANAGER.clone()
}

// Re-export important types
pub use self::{
    CpuFeatures,
    MemoryInfo,
    GpuInfo,
    ThermalInfo,
    HardwareCapabilities,
    PlatformType,
    PerformanceManager,
};
