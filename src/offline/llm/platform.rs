use crate::performance::platform::{get_performance_manager, HardwareCapabilities};
use log::{info, warn, debug};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::utils::locks::SafeLock;

// Enum representing different acceleration backends
#[derive(Debug, Clone, PartialEq)]
pub enum AccelerationBackend {
    CPU,           // CPU-only execution
    CUDA,          // NVIDIA GPU acceleration
    Metal,         // Apple Metal GPU acceleration
    DirectML,      // Windows DirectML 
    OpenCL,        // Cross-platform OpenCL
    None,          // No acceleration
}

// LLM model configuration with hardware-specific optimizations
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model_name: String,
    pub model_path: PathBuf,
    pub context_size: usize,
    pub batch_size: usize,
    pub thread_count: usize,
    pub backend: AccelerationBackend,
    pub quantization: String,    // 4-bit, 8-bit, etc.
    pub memory_limit_mb: u64,
}

// Get platform-appropriate model directories
pub fn get_model_directories() -> Vec<PathBuf> {
    let perf_manager = get_performance_manager();
    let capabilities = perf_manager.get_capabilities();
    
    let mut directories = Vec::new();
    
    match capabilities.platform {
        crate::performance::platform::PlatformType::Windows => {
            // Windows model directories
            directories.push(PathBuf::from(r"C:\Program Files\Papin\models"));
            
            if let Some(appdata) = std::env::var_os("LOCALAPPDATA") {
                let mut path = PathBuf::from(appdata);
                path.push("Papin");
                path.push("models");
                directories.push(path);
            }
        },
        crate::performance::platform::PlatformType::MacOS => {
            // macOS model directories
            let home = std::env::var_os("HOME").unwrap_or_default();
            let mut app_support = PathBuf::from(home);
            app_support.push("Library");
            app_support.push("Application Support");
            app_support.push("Papin");
            app_support.push("models");
            directories.push(app_support);
            
            // Global models
            directories.push(PathBuf::from("/Applications/Papin.app/Contents/Resources/models"));
        },
        crate::performance::platform::PlatformType::Linux => {
            // Linux model directories
            let home = std::env::var_os("HOME").unwrap_or_default();
            
            // User-specific models
            let mut user_models = PathBuf::from(home);
            user_models.push(".local");
            user_models.push("share");
            user_models.push("papin");
            user_models.push("models");
            directories.push(user_models);
            
            // System-wide models
            directories.push(PathBuf::from("/usr/local/share/papin/models"));
            directories.push(PathBuf::from("/usr/share/papin/models"));
        },
        _ => {
            // Default fallback
            if let Some(home) = std::env::var_os("HOME") {
                let mut path = PathBuf::from(home);
                path.push("papin");
                path.push("models");
                directories.push(path);
            }
        }
    }
    
    directories
}

// Detect available acceleration backends
pub fn detect_acceleration_backends() -> Vec<AccelerationBackend> {
    let perf_manager = get_performance_manager();
    let capabilities = perf_manager.get_capabilities();
    
    let mut backends = vec![AccelerationBackend::CPU]; // CPU is always available
    
    match capabilities.platform {
        crate::performance::platform::PlatformType::Windows => {
            // Windows-specific detection
            if capabilities.gpu_info.has_gpu {
                if capabilities.gpu_info.gpu_name.contains("NVIDIA") {
                    backends.push(AccelerationBackend::CUDA);
                }
                // Add DirectML for Windows as it works with most GPUs
                backends.push(AccelerationBackend::DirectML);
                // OpenCL might be available
                backends.push(AccelerationBackend::OpenCL);
            }
        },
        crate::performance::platform::PlatformType::MacOS => {
            // macOS will always have Metal for GPU acceleration
            backends.push(AccelerationBackend::Metal);
        },
        crate::performance::platform::PlatformType::Linux => {
            // Linux GPU detection
            if capabilities.gpu_info.has_gpu {
                if capabilities.gpu_info.gpu_name.contains("NVIDIA") {
                    backends.push(AccelerationBackend::CUDA);
                }
                // OpenCL might be available on Linux
                backends.push(AccelerationBackend::OpenCL);
            }
        },
        _ => {
            // Default - just use CPU
            debug!("No platform-specific acceleration backends detected");
        }
    }
    
    backends
}

// Get optimal LLM configuration based on hardware capabilities
pub fn get_optimal_llm_config(model_name: &str) -> LlmConfig {
    let perf_manager = get_performance_manager();
    let capabilities = perf_manager.get_capabilities();
    
    // Get model directories and find the model
    let model_dirs = get_model_directories();
    let mut model_path = None;
    
    for dir in &model_dirs {
        let path = dir.join(model_name);
        if path.exists() {
            model_path = Some(path);
            break;
        }
    }
    
    let model_path = model_path.unwrap_or_else(|| {
        warn!("Model {} not found in known directories, using first directory as default", model_name);
        model_dirs.first().unwrap().join(model_name)
    });
    
    // Determine optimal batch size
    let batch_size = capabilities.recommended_batch_size();
    
    // Determine thread count
    let thread_count = capabilities.recommended_thread_count();
    
    // Determine context size based on available memory
    let memory_mb = capabilities.memory_limit_mb();
    let context_size = if memory_mb > 16384 {
        8192  // 8K context for high-memory systems
    } else if memory_mb > 8192 {
        4096  // 4K context for mid-range systems
    } else {
        2048  // 2K context for low-memory systems
    };
    
    // Determine backend
    let backends = detect_acceleration_backends();
    let backend = if backends.contains(&AccelerationBackend::CUDA) {
        AccelerationBackend::CUDA
    } else if backends.contains(&AccelerationBackend::Metal) {
        AccelerationBackend::Metal
    } else if backends.contains(&AccelerationBackend::DirectML) {
        AccelerationBackend::DirectML
    } else if backends.contains(&AccelerationBackend::OpenCL) {
        AccelerationBackend::OpenCL
    } else {
        AccelerationBackend::CPU
    };
    
    // Determine quantization based on hardware
    let quantization = if backend == AccelerationBackend::CPU {
        if capabilities.cpu_features.avx2 {
            "int8"  // Use 8-bit quantization for AVX2 CPUs
        } else {
            "float16"  // Use FP16 for CPUs without AVX2
        }
    } else {
        // GPU backends typically support int4 or int8
        "int4"  // Use 4-bit quantization for GPUs for better performance
    };
    
    // Calculate memory limit (in MB)
    // For GPUs, use VRAM if available, otherwise use system RAM
    let memory_limit_mb = if backend != AccelerationBackend::CPU 
                         && backend != AccelerationBackend::None 
                         && capabilities.gpu_info.has_gpu {
        if let Some(vram) = capabilities.gpu_info.vram_mb {
            // Leave some headroom for system
            (vram as f64 * 0.8) as u64
        } else {
            // Use system RAM if VRAM not known
            memory_mb / 2
        }
    } else {
        // For CPU, use a portion of system RAM
        memory_mb / 2
    };
    
    LlmConfig {
        model_name: model_name.to_string(),
        model_path,
        context_size,
        batch_size,
        thread_count,
        backend,
        quantization: quantization.to_string(),
        memory_limit_mb,
    }
}

// LLM model manager with platform optimizations
pub struct LlmModelManager {
    available_models: Mutex<Vec<String>>,
    current_config: Mutex<Option<LlmConfig>>,
}

impl LlmModelManager {
    pub fn new() -> Self {
        LlmModelManager {
            available_models: Mutex::new(Vec::new()),
            current_config: Mutex::new(None),
        }
    }
    
    // Scan for available models
    pub fn scan_for_models(&self) -> Vec<String> {
        let model_dirs = get_model_directories();
        let mut available = Vec::new();
        
        for dir in model_dirs {
            if !dir.exists() {
                continue;
            }
            
            match std::fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Some(name) = path.file_name() {
                                if let Some(name_str) = name.to_str() {
                                    available.push(name_str.to_string());
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    warn!("Failed to read model directory {:?}: {}", dir, e);
                }
            }
        }
        
        // Use safe_lock instead of unwrap
        match self.available_models.safe_lock() {
            Ok(mut models) => {
                *models = available.clone();
            },
            Err(e) => {
                error!("Failed to acquire lock on available_models: {}", e);
            }
        }
        
        available
    }
    
    // Get available models
    pub fn get_available_models(&self) -> Vec<String> {
        match self.available_models.safe_lock() {
            Ok(models) => models.clone(),
            Err(e) => {
                error!("Failed to acquire lock on available_models: {}", e);
                Vec::new() // Return empty vector instead of panicking
            }
        }
    }
    
    // Load and configure a model optimally for the current hardware
    pub fn load_model(&self, model_name: &str) -> Result<LlmConfig, String> {
        // Check if model is available
        let models = self.get_available_models();
        if models.is_empty() {
            self.scan_for_models();
        }
        
        let models = self.get_available_models();
        if !models.contains(&model_name.to_string()) {
            return Err(format!("Model '{}' not available locally", model_name));
        }
        
        // Get optimal configuration
        let config = get_optimal_llm_config(model_name);
        
        // Set current config - use safe_lock instead of unwrap
        match self.current_config.safe_lock() {
            Ok(mut current) => {
                *current = Some(config.clone());
                Ok(config)
            },
            Err(e) => {
                let error_msg = format!("Failed to acquire lock on current_config: {}", e);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }
    
    // Get current model configuration
    pub fn get_current_config(&self) -> Option<LlmConfig> {
        match self.current_config.safe_lock() {
            Ok(current) => current.clone(),
            Err(e) => {
                error!("Failed to acquire lock on current_config: {}", e);
                None // Return None instead of panicking
            }
        }
    }
    
    // Check if the hardware supports local LLM inference
    pub fn supports_local_inference(&self) -> bool {
        let perf_manager = get_performance_manager();
        perf_manager.meets_requirements("offline_llm")
    }
    
    // Get recommended model size based on hardware
    pub fn get_recommended_model_size(&self) -> String {
        let perf_manager = get_performance_manager();
        let capabilities = perf_manager.get_capabilities();
        
        capabilities.recommended_llm_model_size().to_string()
    }
}

// Global LLM model manager instance
lazy_static::lazy_static! {
    static ref LLM_MODEL_MANAGER: Arc<LlmModelManager> = Arc::new(LlmModelManager::new());
}

// Access the global LLM model manager
pub fn get_llm_model_manager() -> Arc<LlmModelManager> {
    LLM_MODEL_MANAGER.clone()
}
