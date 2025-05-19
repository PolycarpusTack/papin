// src/offline/llm/provider.rs
//! Provider trait and related types for local LLM integration

use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// Types of supported LLM providers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    /// Ollama provider (https://ollama.ai)
    Ollama,
    /// LocalAI provider (https://github.com/go-skynet/LocalAI)
    LocalAI,
    /// llama.cpp direct integration
    LlamaCpp,
    /// Custom provider implementation
    Custom(String),
}

impl Display for ProviderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::Ollama => write!(f, "Ollama"),
            ProviderType::LocalAI => write!(f, "LocalAI"),
            ProviderType::LlamaCpp => write!(f, "LlamaCpp"),
            ProviderType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::Ollama
    }
}

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique identifier for the model
    pub id: String,
    /// Human-readable name of the model
    pub name: String,
    /// Size of the model in megabytes
    pub size_mb: usize,
    /// Model context size in tokens
    pub context_size: usize,
    /// Whether the model is installed locally
    pub installed: bool,
    /// URL to download the model (if available)
    pub download_url: Option<String>,
    /// Description of the model
    pub description: String,
    /// Quantization level (e.g., 4-bit, 8-bit)
    pub quantization: Option<String>,
    /// Model architecture (e.g., llama, mistral, falcon)
    pub architecture: Option<String>,
    /// Model format (e.g., gguf, ggml)
    pub format: Option<String>,
    /// Provider-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Status of a model download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStatus {
    /// Model identifier
    pub model_id: String,
    /// Download progress (0.0 to 1.0)
    pub progress: f32,
    /// Bytes downloaded
    pub bytes_downloaded: usize,
    /// Total bytes to download
    pub total_bytes: usize,
    /// Download speed in bytes per second
    pub speed_bps: usize,
    /// Estimated time remaining in seconds
    pub eta_seconds: u64,
    /// Whether the download is complete
    pub complete: bool,
    /// Error message if download failed
    pub error: Option<String>,
    /// Download target path
    pub target_path: PathBuf,
}

/// Generation options for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for sampling (higher = more random)
    pub temperature: Option<f32>,
    /// Top-p sampling value (nucleus sampling)
    pub top_p: Option<f32>,
    /// Whether to stream the response token by token
    pub stream: bool,
    /// Additional provider-specific parameters
    pub additional_params: HashMap<String, serde_json::Value>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(1024),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stream: false,
            additional_params: HashMap::new(),
        }
    }
}

/// Error type for provider operations
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Provider API error
    #[error("API error: {0}")]
    ApiError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Model download error
    #[error("Model download error: {0}")]
    DownloadError(String),
    
    /// Model generation error
    #[error("Model generation error: {0}")]
    GenerationError(String),
    
    /// Provider configuration error
    #[error("Provider configuration error: {0}")]
    ConfigurationError(String),
    
    /// Provider not available
    #[error("Provider not available: {0}")]
    NotAvailable(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Request error
    #[error("Request error: {0}")]
    RequestError(String),
}

/// Result type for provider operations
pub type Result<T> = std::result::Result<T, ProviderError>;

/// Trait for LLM providers
/// 
/// This trait defines the interface for interacting with local LLM providers.
/// Implementations of this trait should handle the specifics of communicating
/// with a particular LLM backend (e.g., Ollama, LocalAI).
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider type
    fn get_type(&self) -> ProviderType;
    
    /// Get the provider name
    fn get_name(&self) -> String;
    
    /// Get the provider description
    fn get_description(&self) -> String;
    
    /// Get the provider version
    async fn get_version(&self) -> Result<String>;
    
    /// Check if the provider is available
    async fn is_available(&self) -> Result<bool>;
    
    /// List all available models (both installed and not installed)
    async fn list_available_models(&self) -> Result<Vec<ModelInfo>>;
    
    /// List models that are downloaded/installed
    async fn list_downloaded_models(&self) -> Result<Vec<ModelInfo>>;
    
    /// Get information about a specific model
    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo>;
    
    /// Download a model
    async fn download_model(&self, model_id: &str) -> Result<()>;
    
    /// Cancel a model download
    async fn cancel_download(&self, model_id: &str) -> Result<()>;
    
    /// Get the status of a model download
    async fn get_download_status(&self, model_id: &str) -> Result<DownloadStatus>;
    
    /// Delete a downloaded model
    async fn delete_model(&self, model_id: &str) -> Result<()>;
    
    /// Generate text using a specified model
    async fn generate_text(
        &self, 
        model_id: &str, 
        prompt: &str, 
        options: GenerationOptions
    ) -> Result<String>;
    
    /// Generate text with streaming
    async fn generate_text_streaming<F>(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
        callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> bool + Send + 'static;
    
    /// Check if a model is loaded in memory
    async fn is_model_loaded(&self, model_id: &str) -> Result<bool>;
    
    /// Load a model into memory
    async fn load_model(&self, model_id: &str) -> Result<()>;
    
    /// Unload a model from memory
    async fn unload_model(&self, model_id: &str) -> Result<()>;
    
    /// Get system information (available RAM, VRAM, etc.)
    async fn get_system_info(&self) -> Result<HashMap<String, String>>;
}