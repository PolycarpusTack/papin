// src/offline/llm/models.rs
//! Model management for offline LLM capabilities

use crate::offline::llm::platform::{get_model_directories, AccelerationBackend};
use crate::performance::platform::{get_performance_manager, HardwareCapabilities};
use crate::platform::fs::{platform_fs, PlatformFsError, PathExt};
use crate::ai::local::models::LocalModelInfo;
use crate::models::Model;
use crate::offline::llm::provider::{ModelInfo, ProviderType, ProviderError, Result};

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::Mutex as TokioMutex;
use log::{info, warn, error, debug, trace};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use std::time::{Duration, Instant, SystemTime};

/// Model file format
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFormat {
    /// GGUF format (successor to GGML)
    GGUF,
    /// GGML format (older format)
    GGML,
    /// ONNX format
    ONNX,
    /// PyTorch format
    PyTorch,
    /// TensorFlow format
    TensorFlow,
    /// Safetensors format
    SafeTensors,
    /// MLC format
    MLC,
    /// Custom format
    Custom(String),
}

impl std::fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFormat::GGUF => write!(f, "GGUF"),
            ModelFormat::GGML => write!(f, "GGML"),
            ModelFormat::ONNX => write!(f, "ONNX"),
            ModelFormat::PyTorch => write!(f, "PyTorch"),
            ModelFormat::TensorFlow => write!(f, "TensorFlow"),
            ModelFormat::SafeTensors => write!(f, "SafeTensors"),
            ModelFormat::MLC => write!(f, "MLC"),
            ModelFormat::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl From<&str> for ModelFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gguf" => ModelFormat::GGUF,
            "ggml" => ModelFormat::GGML,
            "onnx" => ModelFormat::ONNX,
            "pytorch" | "pt" => ModelFormat::PyTorch,
            "tensorflow" | "tf" => ModelFormat::TensorFlow,
            "safetensors" => ModelFormat::SafeTensors,
            "mlc" => ModelFormat::MLC,
            _ => ModelFormat::Custom(s.to_string()),
        }
    }
}

/// Model architecture
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelArchitecture {
    /// Llama architecture
    Llama,
    /// Mistral architecture
    Mistral,
    /// Falcon architecture
    Falcon,
    /// Mamba architecture
    Mamba,
    /// GPT-NeoX architecture
    GPTNeoX,
    /// Phi architecture (Microsoft)
    Phi,
    /// Pythia architecture
    Pythia,
    /// Qwen architecture (Alibaba)
    Qwen,
    /// Custom architecture
    Custom(String),
}

impl std::fmt::Display for ModelArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelArchitecture::Llama => write!(f, "Llama"),
            ModelArchitecture::Mistral => write!(f, "Mistral"),
            ModelArchitecture::Falcon => write!(f, "Falcon"),
            ModelArchitecture::Mamba => write!(f, "Mamba"),
            ModelArchitecture::GPTNeoX => write!(f, "GPT-NeoX"),
            ModelArchitecture::Phi => write!(f, "Phi"),
            ModelArchitecture::Pythia => write!(f, "Pythia"),
            ModelArchitecture::Qwen => write!(f, "Qwen"),
            ModelArchitecture::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl From<&str> for ModelArchitecture {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "llama" => ModelArchitecture::Llama,
            "mistral" => ModelArchitecture::Mistral,
            "falcon" => ModelArchitecture::Falcon,
            "mamba" => ModelArchitecture::Mamba,
            "gpt-neox" | "neox" => ModelArchitecture::GPTNeoX,
            "phi" => ModelArchitecture::Phi,
            "pythia" => ModelArchitecture::Pythia,
            "qwen" => ModelArchitecture::Qwen,
            _ => ModelArchitecture::Custom(s.to_string()),
        }
    }
}

/// Model quantization type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuantizationType {
    /// No quantization (F32)
    None,
    /// 16-bit floating point
    F16,
    /// 8-bit integer
    Int8,
    /// 5-bit integer
    Int5,
    /// 4-bit integer
    Int4,
    /// 3-bit integer
    Int3,
    /// 2-bit integer
    Int2,
    /// Mixed precision quantization
    Mixed,
    /// Custom quantization
    Custom(String),
}

impl std::fmt::Display for QuantizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantizationType::None => write!(f, "F32"),
            QuantizationType::F16 => write!(f, "F16"),
            QuantizationType::Int8 => write!(f, "INT8"),
            QuantizationType::Int5 => write!(f, "INT5"),
            QuantizationType::Int4 => write!(f, "INT4"),
            QuantizationType::Int3 => write!(f, "INT3"),
            QuantizationType::Int2 => write!(f, "INT2"),
            QuantizationType::Mixed => write!(f, "Mixed"),
            QuantizationType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl From<&str> for QuantizationType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "f32" | "fp32" | "float32" | "none" => QuantizationType::None,
            "f16" | "fp16" | "float16" => QuantizationType::F16,
            "int8" | "i8" | "8bit" | "8-bit" => QuantizationType::Int8,
            "int5" | "i5" | "5bit" | "5-bit" => QuantizationType::Int5,
            "int4" | "i4" | "4bit" | "4-bit" => QuantizationType::Int4,
            "int3" | "i3" | "3bit" | "3-bit" => QuantizationType::Int3,
            "int2" | "i2" | "2bit" | "2-bit" => QuantizationType::Int2,
            "mixed" => QuantizationType::Mixed,
            _ => QuantizationType::Custom(s.to_string()),
        }
    }
}

/// Model capability flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Can generate text
    pub text_generation: bool,
    /// Can generate embeddings
    pub embeddings: bool,
    /// Can process images (vision)
    pub vision: bool,
    /// Can process audio
    pub audio: bool,
    /// Supports chat completions
    pub chat: bool,
    /// Supports function calling
    pub function_calling: bool,
    /// Supports token streaming
    pub streaming: bool,
    /// Has been optimized for coding
    pub code_optimized: bool,
    /// Supports multilingual text
    pub multilingual: bool,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            text_generation: true,
            embeddings: false,
            vision: false,
            audio: false,
            chat: true,
            function_calling: false,
            streaming: true,
            code_optimized: false,
            multilingual: false,
        }
    }
}

/// Model file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    /// File name
    pub filename: String,
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size_bytes: u64,
    /// File format
    pub format: ModelFormat,
    /// SHA-256 hash
    pub sha256: Option<String>,
    /// Last modified time
    pub last_modified: SystemTime,
}

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version string (e.g., "v1.0.0")
    pub version: String,
    /// Release date
    pub release_date: Option<DateTime<Utc>>,
    /// Is this the latest version?
    pub is_latest: bool,
    /// Changes in this version
    pub changes: Vec<String>,
    /// Model files
    pub files: Vec<ModelFile>,
}

/// Download progress of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Model ID
    pub model_id: String,
    /// Download progress (0.0 to 1.0)
    pub progress: f32,
    /// Bytes downloaded
    pub bytes_downloaded: u64,
    /// Total bytes to download
    pub total_bytes: u64,
    /// Download speed in bytes per second
    pub speed_bps: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: u64,
    /// Download start time
    pub start_time: DateTime<Utc>,
    /// Last update time
    pub last_update: DateTime<Utc>,
    /// Is the download complete?
    pub completed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Model details including metadata and file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModel {
    /// Model ID (unique identifier)
    pub id: String,
    /// Model name (human-readable)
    pub name: String,
    /// Model family (e.g., llama, mistral)
    pub family: String,
    /// Model description
    pub description: String,
    /// Model architecture
    pub architecture: ModelArchitecture,
    /// Model format
    pub format: ModelFormat,
    /// Total parameter count
    pub parameters: u64,
    /// Quantization type
    pub quantization: QuantizationType,
    /// Maximum context length
    pub context_length: usize,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Model license
    pub license: String,
    /// Original source or provider
    pub source: String,
    /// Path to the model directory
    pub path: Option<PathBuf>,
    /// Model files
    pub files: Vec<ModelFile>,
    /// Size in megabytes
    pub size_mb: u64,
    /// Available versions
    pub versions: Vec<ModelVersion>,
    /// Current version
    pub current_version: String,
    /// Remote model repository URL
    pub repository_url: Option<String>,
    /// Model homepage
    pub homepage: Option<String>,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Installation date
    pub installed_date: Option<DateTime<Utc>>,
    /// Is the model installed locally?
    pub installed: bool,
    /// Is the model currently loaded in memory?
    pub loaded: bool,
    /// Download URL
    pub download_url: Option<String>,
    /// Suggested provider for this model
    pub suggested_provider: Option<ProviderType>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl LLMModel {
    /// Create a new model instance
    pub fn new(id: &str, name: &str, architecture: ModelArchitecture) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            family: "".to_string(),
            description: "".to_string(),
            architecture,
            format: ModelFormat::GGUF, // Default to GGUF
            parameters: 0,
            quantization: QuantizationType::None,
            context_length: 4096,
            capabilities: ModelCapabilities::default(),
            license: "".to_string(),
            source: "".to_string(),
            path: None,
            files: Vec::new(),
            size_mb: 0,
            versions: Vec::new(),
            current_version: "1.0.0".to_string(),
            repository_url: None,
            homepage: None,
            last_used: None,
            installed_date: None,
            installed: false,
            loaded: false,
            download_url: None,
            suggested_provider: None,
            metadata: HashMap::new(),
        }
    }

    /// Convert to LocalModelInfo
    pub fn to_local_model_info(&self) -> LocalModelInfo {
        let model = Model {
            id: self.id.clone(),
            provider: self.source.clone(),
            name: self.name.clone(),
            version: self.current_version.clone(),
            capabilities: crate::models::ModelCapabilities {
                vision: self.capabilities.vision,
                max_context_length: self.context_length,
                functions: self.capabilities.function_calling,
                streaming: self.capabilities.streaming,
            },
        };

        LocalModelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            path: self.path.clone().unwrap_or_else(|| PathBuf::new()),
            parameters: self.parameters,
            quantization: self.quantization.to_string(),
            context_size: self.context_length,
            is_downloaded: self.installed,
            download_url: self.download_url.clone(),
            model,
        }
    }

    /// Convert to ModelInfo
    pub fn to_model_info(&self) -> ModelInfo {
        ModelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            size_mb: self.size_mb as usize,
            context_size: self.context_length,
            installed: self.installed,
            download_url: self.download_url.clone(),
            description: self.description.clone(),
            quantization: Some(self.quantization.to_string()),
            architecture: Some(self.architecture.to_string()),
            format: Some(self.format.to_string()),
            metadata: self.metadata.clone(),
        }
    }

    /// Update the model's last used timestamp
    pub fn update_last_used(&mut self) {
        self.last_used = Some(Utc::now());
    }

    /// Calculate disk space used by this model
    pub fn calculate_disk_space(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).sum()
    }

    /// Convert from ModelInfo
    pub fn from_model_info(info: &ModelInfo) -> Self {
        let architecture = info.architecture
            .as_ref()
            .map(|a| ModelArchitecture::from(a.as_str()))
            .unwrap_or(ModelArchitecture::Custom("unknown".to_string()));

        let format = info.format
            .as_ref()
            .map(|f| ModelFormat::from(f.as_str()))
            .unwrap_or(ModelFormat::GGUF);

        let quantization = info.quantization
            .as_ref()
            .map(|q| QuantizationType::from(q.as_str()))
            .unwrap_or(QuantizationType::None);

        Self {
            id: info.id.clone(),
            name: info.name.clone(),
            family: info.metadata.get("family").cloned().unwrap_or_default(),
            description: info.description.clone(),
            architecture,
            format,
            parameters: info.metadata.get("parameters")
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(0),
            quantization,
            context_length: info.context_size,
            capabilities: ModelCapabilities::default(),
            license: info.metadata.get("license").cloned().unwrap_or_default(),
            source: info.metadata.get("source").cloned().unwrap_or_default(),
            path: None,
            files: Vec::new(),
            size_mb: info.size_mb as u64,
            versions: Vec::new(),
            current_version: info.metadata.get("version").cloned().unwrap_or("1.0.0".to_string()),
            repository_url: info.metadata.get("repository").cloned(),
            homepage: info.metadata.get("homepage").cloned(),
            last_used: None,
            installed_date: None,
            installed: info.installed,
            loaded: false,
            download_url: info.download_url.clone(),
            suggested_provider: None,
            metadata: info.metadata.clone(),
        }
    }
    
    /// Save model metadata to a file
    pub fn save_metadata(&self, path: &Path) -> io::Result<()> {
        let metadata_path = path.join("model.json");
        let metadata_str = serde_json::to_string_pretty(self)?;
        let mut file = File::create(metadata_path)?;
        file.write_all(metadata_str.as_bytes())?;
        Ok(())
    }
    
    /// Load model metadata from a file
    pub fn load_metadata(path: &Path) -> io::Result<Self> {
        let metadata_path = path.join("model.json");
        let mut file = File::open(metadata_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let model = serde_json::from_str(&contents)?;
        Ok(model)
    }
    
    /// Check if the model is compatible with the given provider
    pub fn is_compatible_with(&self, provider: &ProviderType) -> bool {
        match provider {
            ProviderType::Ollama => {
                // Ollama supports GGUF and GGML formats
                matches!(self.format, ModelFormat::GGUF | ModelFormat::GGML)
            },
            ProviderType::LocalAI => {
                // LocalAI supports GGUF, GGML, and ONNX
                matches!(self.format, ModelFormat::GGUF | ModelFormat::GGML | ModelFormat::ONNX)
            },
            ProviderType::LlamaCpp => {
                // llama.cpp only supports GGUF (and legacy GGML)
                matches!(self.format, ModelFormat::GGUF | ModelFormat::GGML)
            },
            ProviderType::Custom(_) => {
                // Assume custom providers can handle this model
                true
            },
        }
    }
}

/// Model registry event types
#[derive(Debug, Clone)]
pub enum ModelRegistryEvent {
    /// Model added to registry
    Added(String),
    /// Model updated in registry
    Updated(String),
    /// Model removed from registry
    Removed(String),
    /// Model download started
    DownloadStarted(String),
    /// Model download progress
    DownloadProgress(DownloadProgress),
    /// Model download completed
    DownloadCompleted(String),
    /// Model download failed
    DownloadFailed(String, String),
    /// Model loaded into memory
    Loaded(String),
    /// Model unloaded from memory
    Unloaded(String),
}

/// Model registry for managing LLM models
pub struct ModelRegistry {
    /// Map of model ID to model
    models: RwLock<HashMap<String, LLMModel>>,
    /// Download status of models
    downloads: RwLock<HashMap<String, DownloadProgress>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<ModelRegistryEvent>,
    /// Base directory for model storage
    base_dir: PathBuf,
    /// Disk space limit in bytes (0 for unlimited)
    disk_space_limit: u64,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        
        let base_dir = if !get_model_directories().is_empty() {
            get_model_directories()[0].clone()
        } else {
            // Fallback to a default directory
            let mut default_dir = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("papin")
                .join("models");
                
            // Ensure directory exists
            if let Err(e) = fs::create_dir_all(&default_dir) {
                error!("Failed to create model directory: {}", e);
                default_dir = PathBuf::from("models");
            }
            
            default_dir
        };
        
        Self {
            models: RwLock::new(HashMap::new()),
            downloads: RwLock::new(HashMap::new()),
            event_tx: tx,
            base_dir,
            disk_space_limit: 0, // No limit by default
        }
    }
    
    /// Initialize the registry and scan for models
    pub fn initialize(&self) -> Result<()> {
        self.scan_for_models()?;
        Ok(())
    }
    
    /// Set the base directory for model storage
    pub fn set_base_directory(&mut self, dir: PathBuf) -> io::Result<()> {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        self.base_dir = dir;
        Ok(())
    }
    
    /// Set disk space limit
    pub fn set_disk_space_limit(&mut self, limit_bytes: u64) {
        self.disk_space_limit = limit_bytes;
    }
    
    /// Subscribe to model registry events
    pub fn subscribe(&self) -> broadcast::Receiver<ModelRegistryEvent> {
        self.event_tx.subscribe()
    }
    
    /// Add a model to the registry
    pub fn add_model(&self, model: LLMModel) -> Result<()> {
        let mut models = self.models.write().unwrap();
        
        // Add or update the model
        models.insert(model.id.clone(), model.clone());
        
        // Emit event
        let _ = self.event_tx.send(ModelRegistryEvent::Added(model.id));
        
        Ok(())
    }
    
    /// Remove a model from the registry
    pub fn remove_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.models.write().unwrap();
        
        if models.remove(model_id).is_some() {
            // Emit event
            let _ = self.event_tx.send(ModelRegistryEvent::Removed(model_id.to_string()));
            Ok(())
        } else {
            Err(ProviderError::ModelNotFound(format!("Model '{}' not found in registry", model_id)))
        }
    }
    
    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Result<LLMModel> {
        let models = self.models.read().unwrap();
        
        models.get(model_id)
            .cloned()
            .ok_or_else(|| ProviderError::ModelNotFound(format!("Model '{}' not found in registry", model_id)))
    }
    
    /// Get all models
    pub fn get_all_models(&self) -> Vec<LLMModel> {
        let models = self.models.read().unwrap();
        models.values().cloned().collect()
    }
    
    /// Get installed models
    pub fn get_installed_models(&self) -> Vec<LLMModel> {
        let models = self.models.read().unwrap();
        models.values()
            .filter(|m| m.installed)
            .cloned()
            .collect()
    }
    
    /// Get models compatible with a specific provider
    pub fn get_compatible_models(&self, provider_type: &ProviderType) -> Vec<LLMModel> {
        let models = self.models.read().unwrap();
        models.values()
            .filter(|m| m.is_compatible_with(provider_type))
            .cloned()
            .collect()
    }
    
    /// Update model information
    pub fn update_model(&self, model_id: &str, update_fn: impl FnOnce(&mut LLMModel)) -> Result<()> {
        let mut models = self.models.write().unwrap();
        
        if let Some(model) = models.get_mut(model_id) {
            update_fn(model);
            
            // Save updated metadata if the model is installed
            if model.installed {
                if let Some(path) = &model.path {
                    if let Err(e) = model.save_metadata(path) {
                        warn!("Failed to save model metadata for '{}': {}", model_id, e);
                    }
                }
            }
            
            // Emit event
            let _ = self.event_tx.send(ModelRegistryEvent::Updated(model_id.to_string()));
            
            Ok(())
        } else {
            Err(ProviderError::ModelNotFound(format!("Model '{}' not found in registry", model_id)))
        }
    }
    
    /// Scan directories for models
    pub fn scan_for_models(&self) -> Result<()> {
        let model_dirs = get_model_directories();
        let mut added_models = HashSet::new();
        
        for dir in model_dirs {
            if !dir.exists() {
                if let Err(e) = fs::create_dir_all(&dir) {
                    warn!("Failed to create model directory {}: {}", dir.display(), e);
                    continue;
                }
            }
            
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        
                        if path.is_dir() {
                            // Check for model.json file
                            let metadata_path = path.join("model.json");
                            
                            if metadata_path.exists() {
                                match LLMModel::load_metadata(&path) {
                                    Ok(mut model) => {
                                        // Update path and installed status
                                        model.path = Some(path.clone());
                                        model.installed = true;
                                        
                                        // Update file list and calculate size
                                        self.update_model_files(&mut model)?;
                                        
                                        // Add to registry
                                        let mut models = self.models.write().unwrap();
                                        models.insert(model.id.clone(), model.clone());
                                        added_models.insert(model.id.clone());
                                    },
                                    Err(e) => {
                                        warn!("Failed to load model metadata from {}: {}", 
                                              metadata_path.display(), e);
                                    }
                                }
                            } else {
                                // Try to infer model info from directory structure
                                if let Some(model) = self.infer_model_from_directory(&path)? {
                                    // Add to registry
                                    let mut models = self.models.write().unwrap();
                                    models.insert(model.id.clone(), model.clone());
                                    added_models.insert(model.id.clone());
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    warn!("Failed to read model directory {}: {}", dir.display(), e);
                }
            }
        }
        
        // Log what we found
        info!("Scanned for models: found {} installed models", added_models.len());
        
        Ok(())
    }
    
    /// Infer model information from directory structure
    fn infer_model_from_directory(&self, dir: &Path) -> Result<Option<LLMModel>> {
        // Get directory name as potential model ID
        let model_id = dir.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        // Look for model files with common extensions
        let mut model_files = Vec::new();
        let mut total_size = 0;
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() {
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                        
                    // Check for common model file extensions
                    if ["gguf", "ggml", "bin", "pt", "onnx", "model"].contains(&extension.as_str()) {
                        if let Ok(metadata) = fs::metadata(&path) {
                            let file_size = metadata.len();
                            total_size += file_size;
                            
                            let file_format = match extension.as_str() {
                                "gguf" => ModelFormat::GGUF,
                                "ggml" => ModelFormat::GGML,
                                "pt" => ModelFormat::PyTorch,
                                "onnx" => ModelFormat::ONNX,
                                _ => ModelFormat::Custom(extension.clone()),
                            };
                            
                            let model_file = ModelFile {
                                filename: path.file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                path: path.clone(),
                                size_bytes: file_size,
                                format: file_format,
                                sha256: None,
                                last_modified: metadata.modified().unwrap_or_else(|_| SystemTime::now()),
                            };
                            
                            model_files.push(model_file);
                        }
                    }
                }
            }
        }
        
        // If we found model files, create a model
        if !model_files.is_empty() {
            // Try to determine format from files
            let format = model_files.first()
                .map(|f| f.format.clone())
                .unwrap_or(ModelFormat::GGUF);
                
            // Create model
            let mut model = LLMModel::new(
                &model_id,
                &model_id,
                ModelArchitecture::Custom("unknown".to_string()),
            );
            
            model.path = Some(dir.to_path_buf());
            model.installed = true;
            model.format = format;
            model.files = model_files;
            model.size_mb = total_size / 1_048_576; // Convert bytes to MB
            model.installed_date = Some(Utc::now());
            
            // Attempt to save metadata
            if let Err(e) = model.save_metadata(dir) {
                warn!("Failed to save inferred model metadata: {}", e);
            }
            
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
    
    /// Update model files and calculate size
    fn update_model_files(&self, model: &mut LLMModel) -> Result<()> {
        let model_dir = match &model.path {
            Some(path) => path,
            None => return Ok(()),
        };
        
        if !model_dir.exists() {
            return Ok(());
        }
        
        let mut model_files = Vec::new();
        let mut total_size = 0;
        
        if let Ok(entries) = fs::read_dir(model_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() && path.file_name().map_or(false, |name| name != "model.json") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        let file_size = metadata.len();
                        total_size += file_size;
                        
                        let extension = path.extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                            
                        let file_format = match extension.as_str() {
                            "gguf" => ModelFormat::GGUF,
                            "ggml" => ModelFormat::GGML,
                            "pt" => ModelFormat::PyTorch,
                            "onnx" => ModelFormat::ONNX,
                            _ => ModelFormat::Custom(extension),
                        };
                        
                        let model_file = ModelFile {
                            filename: path.file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            path: path.clone(),
                            size_bytes: file_size,
                            format: file_format,
                            sha256: None,
                            last_modified: metadata.modified().unwrap_or_else(|_| SystemTime::now()),
                        };
                        
                        model_files.push(model_file);
                    }
                }
            }
        }
        
        model.files = model_files;
        model.size_mb = total_size / 1_048_576; // Convert bytes to MB
        
        Ok(())
    }
    
    /// Start model download
    pub fn start_download(&self, model_id: &str, url: &str, provider: &ProviderType) -> Result<()> {
        // Check if model exists in registry
        let mut model_exists = false;
        let model_name: String;
        
        {
            let models = self.models.read().unwrap();
            if let Some(model) = models.get(model_id) {
                model_exists = true;
                model_name = model.name.clone();
            } else {
                model_name = model_id.to_string();
            }
        }
        
        // If model doesn't exist, create a stub entry
        if !model_exists {
            let model = LLMModel::new(
                model_id,
                &model_name,
                ModelArchitecture::Custom("unknown".to_string()),
            );
            
            self.add_model(model)?;
        }
        
        // Create download progress
        let progress = DownloadProgress {
            model_id: model_id.to_string(),
            progress: 0.0,
            bytes_downloaded: 0,
            total_bytes: 0,
            speed_bps: 0,
            eta_seconds: 0,
            start_time: Utc::now(),
            last_update: Utc::now(),
            completed: false,
            error: None,
        };
        
        // Add to downloads
        {
            let mut downloads = self.downloads.write().unwrap();
            downloads.insert(model_id.to_string(), progress.clone());
        }
        
        // Emit event
        let _ = self.event_tx.send(ModelRegistryEvent::DownloadStarted(model_id.to_string()));
        let _ = self.event_tx.send(ModelRegistryEvent::DownloadProgress(progress));
        
        Ok(())
    }
    
    /// Update download progress
    pub fn update_download_progress(&self, model_id: &str, progress: DownloadProgress) -> Result<()> {
        let mut downloads = self.downloads.write().unwrap();
        
        downloads.insert(model_id.to_string(), progress.clone());
        
        // Emit event
        let _ = self.event_tx.send(ModelRegistryEvent::DownloadProgress(progress));
        
        Ok(())
    }
    
    /// Complete download
    pub fn complete_download(&self, model_id: &str, path: &Path) -> Result<()> {
        let mut downloads = self.downloads.write().unwrap();
        
        // Update download status
        if let Some(progress) = downloads.get_mut(model_id) {
            progress.completed = true;
            progress.progress = 1.0;
            progress.last_update = Utc::now();
        }
        
        // Update model information
        self.update_model(model_id, |model| {
            model.installed = true;
            model.path = Some(path.to_path_buf());
            model.installed_date = Some(Utc::now());
            
            // Update file list and size
            if let Err(e) = self.update_model_files(model) {
                warn!("Failed to update model files: {}", e);
            }
            
            // Save metadata
            if let Err(e) = model.save_metadata(path) {
                warn!("Failed to save model metadata: {}", e);
            }
        })?;
        
        // Emit event
        let _ = self.event_tx.send(ModelRegistryEvent::DownloadCompleted(model_id.to_string()));
        
        Ok(())
    }
    
    /// Fail download
    pub fn fail_download(&self, model_id: &str, error: &str) -> Result<()> {
        let mut downloads = self.downloads.write().unwrap();
        
        // Update download status
        if let Some(progress) = downloads.get_mut(model_id) {
            progress.completed = true;
            progress.error = Some(error.to_string());
            progress.last_update = Utc::now();
        }
        
        // Emit event
        let _ = self.event_tx.send(ModelRegistryEvent::DownloadFailed(
            model_id.to_string(),
            error.to_string(),
        ));
        
        Ok(())
    }
    
    /// Get download progress
    pub fn get_download_progress(&self, model_id: &str) -> Result<DownloadProgress> {
        let downloads = self.downloads.read().unwrap();
        
        downloads.get(model_id)
            .cloned()
            .ok_or_else(|| ProviderError::DownloadError(format!("No download in progress for model '{}'", model_id)))
    }
    
    /// Check if model is loaded
    pub fn is_model_loaded(&self, model_id: &str) -> Result<bool> {
        let models = self.models.read().unwrap();
        
        models.get(model_id)
            .map(|model| model.loaded)
            .ok_or_else(|| ProviderError::ModelNotFound(format!("Model '{}' not found in registry", model_id)))
    }
    
    /// Set model loaded state
    pub fn set_model_loaded(&self, model_id: &str, loaded: bool) -> Result<()> {
        self.update_model(model_id, |model| {
            model.loaded = loaded;
            
            if loaded {
                model.update_last_used();
                
                // Emit loaded event
                let _ = self.event_tx.send(ModelRegistryEvent::Loaded(model_id.to_string()));
            } else {
                // Emit unloaded event
                let _ = self.event_tx.send(ModelRegistryEvent::Unloaded(model_id.to_string()));
            }
        })
    }
    
    /// Get model storage path
    pub fn get_model_path(&self, model_id: &str) -> Result<PathBuf> {
        let model = self.get_model(model_id)?;
        
        model.path.clone()
            .ok_or_else(|| ProviderError::ModelNotFound(format!("Model '{}' has no path", model_id)))
    }
    
    /// Get new model path (for download)
    pub fn get_new_model_path(&self, model_id: &str) -> PathBuf {
        let model_dir = self.base_dir.join(model_id);
        
        // Ensure directory exists
        if let Err(e) = fs::create_dir_all(&model_dir) {
            error!("Failed to create model directory {}: {}", model_dir.display(), e);
        }
        
        model_dir
    }
    
    /// Calculate total disk space used by models
    pub fn calculate_disk_usage(&self) -> u64 {
        let models = self.models.read().unwrap();
        
        models.values()
            .filter(|m| m.installed)
            .map(|m| m.calculate_disk_space())
            .sum()
    }
    
    /// Check if there's enough space for a new model
    pub fn has_space_for(&self, size_bytes: u64) -> bool {
        // If no limit, always return true
        if self.disk_space_limit == 0 {
            return true;
        }
        
        let current_usage = self.calculate_disk_usage();
        
        current_usage + size_bytes <= self.disk_space_limit
    }
    
    /// Delete least recently used models to free up space
    pub fn free_up_space(&self, needed_bytes: u64) -> Result<u64> {
        // If no limit, nothing to do
        if self.disk_space_limit == 0 {
            return Ok(0);
        }
        
        let current_usage = self.calculate_disk_usage();
        
        // If we already have enough space, nothing to do
        if self.disk_space_limit >= current_usage + needed_bytes {
            return Ok(0);
        }
        
        // Calculate how much space we need to free
        let to_free = (current_usage + needed_bytes) - self.disk_space_limit;
        
        // Get models sorted by last used time
        let mut models: Vec<LLMModel>;
        {
            let model_map = self.models.read().unwrap();
            models = model_map.values()
                .filter(|m| m.installed && !m.loaded) // Only consider installed and not loaded models
                .cloned()
                .collect();
        }
        
        // Sort by last used (oldest first)
        models.sort_by(|a, b| {
            a.last_used.unwrap_or_else(|| Utc::now())
                .cmp(&b.last_used.unwrap_or_else(|| Utc::now()))
        });
        
        let mut freed = 0u64;
        let mut removed_models = Vec::new();
        
        // Remove models until we've freed enough space
        for model in models {
            let model_size = model.calculate_disk_space();
            
            // Skip if removing would exceed what we need to free
            if freed > to_free && model_size > (freed - to_free) {
                continue;
            }
            
            // Try to remove the model files
            if let Some(path) = &model.path {
                if let Err(e) = fs::remove_dir_all(path) {
                    warn!("Failed to remove model directory {}: {}", path.display(), e);
                    continue;
                }
            }
            
            freed += model_size;
            removed_models.push(model.id.clone());
            
            if freed >= to_free {
                break;
            }
        }
        
        // Update registry
        for model_id in removed_models {
            self.update_model(&model_id, |model| {
                model.installed = false;
                model.path = None;
            })?;
        }
        
        Ok(freed)
    }
    
    /// Import a model from an external directory
    pub fn import_model(&self, source_path: &Path, model_id: &str) -> Result<()> {
        if !source_path.exists() {
            return Err(ProviderError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path {} does not exist", source_path.display()),
            )));
        }
        
        // Create destination directory
        let dest_path = self.get_new_model_path(model_id);
        
        if dest_path.exists() {
            return Err(ProviderError::IoError(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Destination path {} already exists", dest_path.display()),
            )));
        }
        
        fs::create_dir_all(&dest_path)?;
        
        // Copy files
        if source_path.is_dir() {
            // Copy all files in directory
            if let Ok(entries) = fs::read_dir(source_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    if path.is_file() {
                        let filename = path.file_name().unwrap_or_default();
                        let dest_file = dest_path.join(filename);
                        
                        fs::copy(&path, &dest_file)?;
                    }
                }
            }
        } else if source_path.is_file() {
            // Copy single file
            let filename = source_path.file_name().unwrap_or_default();
            let dest_file = dest_path.join(filename);
            
            fs::copy(source_path, &dest_file)?;
        }
        
        // Create or update model
        let model = if let Ok(existing) = self.get_model(model_id) {
            // Update existing model
            let mut model = existing;
            model.installed = true;
            model.path = Some(dest_path.clone());
            model.installed_date = Some(Utc::now());
            model
        } else {
            // Create new model
            let mut model = LLMModel::new(
                model_id,
                model_id,
                ModelArchitecture::Custom("unknown".to_string()),
            );
            model.installed = true;
            model.path = Some(dest_path.clone());
            model.installed_date = Some(Utc::now());
            model
        };
        
        // Update file list and size
        self.update_model_files(&mut model)?;
        
        // Save metadata
        model.save_metadata(&dest_path)?;
        
        // Add to registry
        self.add_model(model)?;
        
        Ok(())
    }
    
    /// Export a model to an external directory
    pub fn export_model(&self, model_id: &str, dest_path: &Path) -> Result<()> {
        let model = self.get_model(model_id)?;
        
        if !model.installed {
            return Err(ProviderError::ModelNotFound(format!("Model '{}' is not installed", model_id)));
        }
        
        let source_path = model.path.as_ref().ok_or_else(|| {
            ProviderError::ModelNotFound(format!("Model '{}' has no path", model_id))
        })?;
        
        if !source_path.exists() {
            return Err(ProviderError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path {} does not exist", source_path.display()),
            )));
        }
        
        // Create destination directory
        if !dest_path.exists() {
            fs::create_dir_all(dest_path)?;
        }
        
        // Copy files
        if let Ok(entries) = fs::read_dir(source_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() && path.file_name().map_or(false, |name| name != "model.json") {
                    let filename = path.file_name().unwrap_or_default();
                    let dest_file = dest_path.join(filename);
                    
                    fs::copy(&path, &dest_file)?;
                }
            }
        }
        
        // Export metadata
        model.save_metadata(dest_path)?;
        
        Ok(())
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global model registry instance
lazy_static::lazy_static! {
    static ref MODEL_REGISTRY: Arc<ModelRegistry> = Arc::new(ModelRegistry::new());
}

/// Get the global model registry
pub fn get_model_registry() -> Arc<ModelRegistry> {
    MODEL_REGISTRY.clone()
}

/// Initialize the model registry
pub fn initialize_model_registry() -> Result<()> {
    MODEL_REGISTRY.initialize()
}