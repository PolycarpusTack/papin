use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents the various types of local LLM providers that the system supports.
/// 
/// This enum allows the system to identify and instantiate the appropriate
/// provider implementation based on user selection or configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderType {
    /// Ollama provider (https://ollama.ai/)
    /// A lightweight local LLM server that supports various models
    Ollama,
    
    /// LocalAI provider (https://localai.io/)
    /// A drop-in replacement for OpenAI API that runs locally
    LocalAI,
    
    /// llama.cpp provider (direct integration)
    /// Direct integration with the llama.cpp library for maximum performance
    LlamaCpp,
    
    /// Custom provider implementation
    /// For extensibility and third-party providers
    Custom(String),
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::Ollama
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Ollama => write!(f, "Ollama"),
            ProviderType::LocalAI => write!(f, "LocalAI"),
            ProviderType::LlamaCpp => write!(f, "llama.cpp"),
            ProviderType::Custom(name) => write!(f, "Custom ({})", name),
        }
    }
}

/// Configuration options for LLM providers.
/// 
/// This struct contains common configuration options applicable to most providers,
/// as well as provider-specific options that can be passed through the custom_options field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// The type of provider this configuration is for
    pub provider_type: ProviderType,
    
    /// The base URL/endpoint for the provider's API
    pub endpoint_url: String,
    
    /// API key for providers that require authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    
    /// Maximum number of concurrent requests to the provider
    #[serde(default = "default_concurrency")]
    pub max_concurrent_requests: usize,
    
    /// Timeout in seconds for requests to the provider
    #[serde(default = "default_timeout")]
    pub request_timeout_seconds: u64,
    
    /// Whether to enable request retry on failure
    #[serde(default)]
    pub enable_retry: bool,
    
    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    
    /// Backoff factor for retry delays (in seconds)
    #[serde(default = "default_backoff")]
    pub retry_backoff_seconds: f64,
    
    /// Additional provider-specific configuration options
    #[serde(default)]
    pub custom_options: HashMap<String, serde_json::Value>,
}

fn default_concurrency() -> usize { 5 }
fn default_timeout() -> u64 { 30 }
fn default_max_retries() -> usize { 3 }
fn default_backoff() -> f64 { 1.5 }

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig {
            provider_type: ProviderType::default(),
            endpoint_url: "http://localhost:11434".to_string(),
            api_key: None,
            max_concurrent_requests: default_concurrency(),
            request_timeout_seconds: default_timeout(),
            enable_retry: true,
            max_retries: default_max_retries(),
            retry_backoff_seconds: default_backoff(),
            custom_options: HashMap::new(),
        }
    }
}

/// Information about a model available from a provider.
///
/// This struct provides a uniform representation of model information across
/// different providers, allowing the application to display and manage models
/// in a consistent way regardless of the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    /// Unique identifier for the model
    pub id: String,
    
    /// Display name of the model
    pub name: String,
    
    /// Model description
    pub description: String,
    
    /// Provider that supplies this model
    pub provider: ProviderType,
    
    /// Model size in bytes
    pub size_bytes: u64,
    
    /// Whether the model is currently downloaded and available locally
    pub is_downloaded: bool,
    
    /// Whether the model supports text generation
    pub supports_text_generation: bool,
    
    /// Whether the model supports text completion (vs chat)
    pub supports_completion: bool,
    
    /// Whether the model supports chat completion
    pub supports_chat: bool,
    
    /// Whether the model supports embeddings
    pub supports_embeddings: bool,
    
    /// Whether the model supports image generation
    pub supports_image_generation: bool,
    
    /// Quantization level if the model is quantized (e.g., "Q4_K_M")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    
    /// Parameter count of the model (in billions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_count_b: Option<f64>,
    
    /// Context window size (max tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<usize>,
    
    /// The model's family/architecture (e.g., "llama", "mistral", "gpt2")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<String>,
    
    /// Creation date of the model (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    
    /// Tags associated with the model
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// License information for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    
    /// Provider-specific model information that doesn't fit into the standard fields
    #[serde(default)]
    pub provider_specific: HashMap<String, serde_json::Value>,
}

/// Message type for chat-based requests.
///
/// This struct represents a single message in a chat-based interaction,
/// following the format used by many LLM providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// Role of the message sender (system, user, assistant)
    pub role: String,
    
    /// Content of the message
    pub content: String,
    
    /// Optional name of the message sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Options for text generation.
///
/// This struct contains parameters that control the behavior of text generation,
/// including creativity, response length, and other settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationOptions {
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Temperature parameter (0.0 to 2.0, higher = more creative)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Top-p sampling parameter (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Top-k sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// List of token IDs to never generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_token_ids: Option<Vec<u32>>,
    
    /// List of sequences to stop generation when encountered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    
    /// Whether to stream the response token-by-token
    #[serde(default)]
    pub stream: bool,
    
    /// Whether to return logprobs for generated tokens
    #[serde(default)]
    pub logprobs: bool,
    
    /// Number of top logprobs to return per token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    
    /// Seed for deterministic generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    
    /// Additional provider-specific parameters
    #[serde(default)]
    pub additional_params: HashMap<String, serde_json::Value>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(1024),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_token_ids: None,
            stop_sequences: None,
            stream: false,
            logprobs: false,
            top_logprobs: None,
            seed: None,
            additional_params: HashMap::new(),
        }
    }
}

/// Request for text generation.
///
/// This struct contains all the information needed for a text generation request,
/// including the prompt, model, and generation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationRequest {
    /// ID of the model to use
    pub model_id: String,
    
    /// Type of generation request (text, chat, etc.)
    pub request_type: GenerationRequestType,
    
    /// Generation options
    #[serde(default)]
    pub options: GenerationOptions,
}

/// Type of generation request.
///
/// This enum represents the different types of generation requests that can be made,
/// such as text completion, chat, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum GenerationRequestType {
    /// Text completion with a single prompt
    #[serde(rename_all = "camelCase")]
    TextCompletion {
        /// The prompt text to complete
        prompt: String,
    },
    
    /// Chat completion with a sequence of messages
    #[serde(rename_all = "camelCase")]
    ChatCompletion {
        /// The chat messages
        messages: Vec<ChatMessage>,
    },
}

/// Token usage information.
///
/// This struct tracks token usage for billing and rate limiting purposes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    #[serde(default)]
    pub prompt_tokens: u32,
    
    /// Number of tokens in the completion
    #[serde(default)]
    pub completion_tokens: u32,
    
    /// Total number of tokens used
    #[serde(default)]
    pub total_tokens: u32,
}

/// Response from text generation.
///
/// This struct contains the generated text and metadata from the generation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationResponse {
    /// The generated text
    pub text: String,
    
    /// Type of the response (text, chat)
    pub response_type: ResponseType,
    
    /// Token usage information
    #[serde(default)]
    pub usage: TokenUsage,
    
    /// Whether the generation was stopped due to reaching max tokens
    #[serde(default)]
    pub truncated: bool,
    
    /// Reason for stopping generation (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    
    /// ID of the model that generated the text
    pub model_id: String,
    
    /// Creation timestamp (ISO 8601 format)
    pub created_at: String,
    
    /// Provider-specific metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of response.
///
/// This enum represents the different types of responses that can be generated,
/// matching the request types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseType {
    /// Text completion response
    TextCompletion,
    
    /// Chat completion response
    ChatCompletion,
}

/// Status of a model download.
///
/// This enum tracks the status and progress of model downloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum DownloadStatus {
    /// Model download has not started
    NotStarted,
    
    /// Model download is in progress
    #[serde(rename_all = "camelCase")]
    InProgress {
        /// Download progress percentage (0.0 to 100.0)
        percent: f32,
        
        /// Downloaded bytes
        bytes_downloaded: Option<u64>,
        
        /// Total bytes to download
        total_bytes: Option<u64>,
        
        /// Estimated time remaining in seconds
        eta_seconds: Option<f32>,
        
        /// Download speed in bytes per second
        bytes_per_second: Option<f32>,
    },
    
    /// Model download has completed successfully
    Completed {
        /// Timestamp when download completed (ISO 8601 format)
        completed_at: Option<String>,
        
        /// Time taken to download in seconds
        duration_seconds: Option<f32>,
    },
    
    /// Model download has failed
    #[serde(rename_all = "camelCase")]
    Failed {
        /// Reason for failure
        reason: String,
        
        /// Error code (if available)
        error_code: Option<String>,
        
        /// Timestamp when failure occurred (ISO 8601 format)
        failed_at: Option<String>,
    },
    
    /// Model download has been cancelled
    Cancelled {
        /// Timestamp when download was cancelled (ISO 8601 format)
        cancelled_at: Option<String>,
    },
}

/// Error types for LLM provider operations.
///
/// This enum represents the different types of errors that can occur
/// when interacting with LLM providers.
#[derive(Error, Debug)]
pub enum ProviderError {
    /// The requested model was not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// The model download failed
    #[error("Model download failed: {0}")]
    DownloadFailed(String),
    
    /// The text generation failed
    #[error("Text generation failed: {0}")]
    GenerationFailed(String),
    
    /// A network error occurred
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    /// Provider not initialized properly
    #[error("Provider not initialized: {0}")]
    NotInitialized(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;
