use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Represents errors that can occur when interacting with a local LLM provider.
#[derive(Error, Debug)]
pub enum LLMProviderError {
    /// Returned when a requested model does not exist
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Returned when there's an error during model download
    #[error("Model download failed: {0}")]
    DownloadFailed(String),
    
    /// Returned when there's an error during text generation
    #[error("Text generation failed: {0}")]
    GenerationFailed(String),
    
    /// Returned when an operation fails due to network issues
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Returned when a provider-specific operation fails
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    /// Returned when the provider is not properly initialized
    #[error("Provider not initialized: {0}")]
    NotInitialized(String),
    
    /// Represents an unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Result type for LLM provider operations
pub type LLMProviderResult<T> = Result<T, LLMProviderError>;

/// Represents the download status of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// Model download has not started
    NotStarted,
    /// Model download is in progress with percentage
    InProgress { percent: f32 },
    /// Model download has completed successfully
    Completed,
    /// Model download has failed with error message
    Failed { reason: String },
    /// Model download has been cancelled
    Cancelled,
}

/// Represents the configuration for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model identifier
    pub id: String,
    /// Configuration parameters for the model
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Represents model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique identifier for the model
    pub id: String,
    /// Display name of the model
    pub name: String,
    /// Model description
    pub description: String,
    /// Model size in bytes
    pub size_bytes: u64,
    /// Whether the model is currently downloaded and available locally
    pub is_downloaded: bool,
    /// Provider-specific model information
    pub provider_metadata: HashMap<String, serde_json::Value>,
}

/// Options for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for sampling (0.0 to 1.0, lower is more deterministic)
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Whether to stream the generated text
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

/// Represents a completion response from a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The generated text
    pub text: String,
    /// Whether the generation was stopped due to reaching max tokens
    pub reached_max_tokens: bool,
    /// Token usage statistics
    pub usage: TokenUsage,
    /// Provider-specific metadata about the generation
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents token usage statistics for a generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

/// Trait for streaming completions
#[async_trait]
pub trait CompletionStream: Send + Sync {
    /// Get the next chunk of the completion
    async fn next_chunk(&mut self) -> Option<LLMProviderResult<String>>;
}

/// Represents a local LLM provider, abstracting different implementations like Ollama, LocalAI, etc.
#[async_trait]
pub trait LocalLLMProvider: Send + Sync {
    /// Initialize the provider with the given configuration
    /// 
    /// # Arguments
    /// 
    /// * `config` - Provider-specific configuration as a JSON value
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn initialize(&mut self, config: serde_json::Value) -> LLMProviderResult<()>;
    
    /// Get provider name
    /// 
    /// # Returns
    /// 
    /// * `&str` - Name of the provider (e.g., "Ollama", "LocalAI")
    fn provider_name(&self) -> &str;
    
    /// List all available models from this provider
    /// 
    /// This will return both downloaded/available models and models that can be downloaded.
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<Vec<ModelInfo>>` - Information about available models
    async fn list_available_models(&self) -> LLMProviderResult<Vec<ModelInfo>>;
    
    /// List models that are currently downloaded and ready to use
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<Vec<ModelInfo>>` - Information about downloaded models
    async fn list_downloaded_models(&self) -> LLMProviderResult<Vec<ModelInfo>>;
    
    /// Download a model by its identifier
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to download
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn download_model(&self, model_id: &str) -> LLMProviderResult<()>;
    
    /// Get the download status of a model
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to check
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<DownloadStatus>` - Current download status
    async fn get_download_status(&self, model_id: &str) -> LLMProviderResult<DownloadStatus>;
    
    /// Cancel an in-progress model download
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model whose download should be cancelled
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn cancel_download(&self, model_id: &str) -> LLMProviderResult<()>;
    
    /// Delete a downloaded model
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to delete
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn delete_model(&self, model_id: &str) -> LLMProviderResult<()>;
    
    /// Check if a model is currently loaded and ready for inference
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to check
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<bool>` - Whether the model is loaded
    async fn is_model_loaded(&self, model_id: &str) -> LLMProviderResult<bool>;
    
    /// Explicitly load a model into memory for faster inference
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to load
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn load_model(&self, model_id: &str) -> LLMProviderResult<()>;
    
    /// Unload a model from memory
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to unload
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn unload_model(&self, model_id: &str) -> LLMProviderResult<()>;
    
    /// Generate text with a model
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to use
    /// * `prompt` - Input prompt for text generation
    /// * `options` - Options for text generation
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<CompletionResponse>` - Generated completion
    async fn generate_text(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<CompletionResponse>;
    
    /// Generate text with streaming response
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model to use
    /// * `prompt` - Input prompt for text generation
    /// * `options` - Options for text generation
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<Box<dyn CompletionStream>>` - Stream of generated text chunks
    async fn generate_text_streaming(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<Box<dyn CompletionStream>>;
    
    /// Get the current configuration for a model
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<ModelConfig>` - Current model configuration
    async fn get_model_config(&self, model_id: &str) -> LLMProviderResult<ModelConfig>;
    
    /// Update the configuration for a model
    /// 
    /// # Arguments
    /// 
    /// * `config` - New configuration for the model
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<()>` - Result indicating success or failure
    async fn update_model_config(&self, config: ModelConfig) -> LLMProviderResult<()>;
    
    /// Get detailed information about a specific model
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Identifier of the model
    /// 
    /// # Returns
    /// 
    /// * `LLMProviderResult<ModelInfo>` - Detailed model information
    async fn get_model_info(&self, model_id: &str) -> LLMProviderResult<ModelInfo>;
}