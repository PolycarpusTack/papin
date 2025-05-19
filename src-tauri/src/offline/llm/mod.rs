//! Local LLM integration module for offline capabilities
//!
//! This module provides support for using locally-hosted Large Language Models
//! for generating text when online services are unavailable.

pub mod provider;
pub mod types;
pub mod providers;
pub mod factory;

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::provider::{
    CompletionResponse, LLMProviderError, 
    LLMProviderResult, LocalLLMProvider, ModelInfo,
};
use self::providers::ollama::OllamaProvider;
use self::types::{ProviderType, ProviderConfig, GenerationOptions, GenerationRequest, GenerationResponse};

/// Errors that can occur in the LLM module
#[derive(Error, Debug)]
pub enum LLMError {
    /// No provider is available
    #[error("No LLM provider available")]
    NoProviderAvailable,
    
    /// Provider-specific error
    #[error("Provider error: {0}")]
    ProviderError(#[from] LLMProviderError),
    
    /// Error communicating with LLM
    #[error("LLM communication error: {0}")]
    CommunicationError(String),
    
    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Result type for LLM operations
pub type LLMResult<T> = Result<T, LLMError>;

/// Configuration for the LLM module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Type of provider to use
    pub provider_type: ProviderType,
    /// Provider-specific configuration
    pub provider_config: serde_json::Value,
    /// Default model to use for generation
    pub default_model: Option<String>,
    /// Whether to enable debug logging
    pub enable_debug: bool,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Ollama,
            provider_config: serde_json::json!({
                "base_url": "http://localhost:11434",
            }),
            default_model: None,
            enable_debug: false,
        }
    }
}

/// LLM manager that handles provider selection and configuration
pub struct LLMManager {
    /// Current configuration
    config: LLMConfig,
    /// Current provider
    provider: Option<Box<dyn LocalLLMProvider>>,
}

impl LLMManager {
    /// Create a new LLM manager with the default configuration
    pub fn new() -> Self {
        Self {
            config: LLMConfig::default(),
            provider: None,
        }
    }
    
    /// Create a new LLM manager with a specific configuration
    pub fn with_config(config: LLMConfig) -> Self {
        Self {
            config,
            provider: None,
        }
    }
    
    /// Initialize the LLM manager using the factory
    pub async fn initialize(&mut self) -> LLMResult<()> {
        let provider_config = ProviderConfig {
            provider_type: self.config.provider_type,
            endpoint_url: match self.config.provider_config.get("base_url") {
                Some(val) => val.as_str().unwrap_or("http://localhost:11434").to_string(),
                None => "http://localhost:11434".to_string(),
            },
            api_key: match self.config.provider_config.get("api_key") {
                Some(val) => val.as_str().map(|s| s.to_string()),
                None => None,
            },
            // Set reasonable defaults for remaining fields
            max_concurrent_requests: 5,
            request_timeout_seconds: 30,
            enable_retry: true,
            max_retries: 3,
            retry_backoff_seconds: 1.5,
            // Convert any remaining custom options
            custom_options: {
                let mut custom = std::collections::HashMap::new();
                if let Some(obj) = self.config.provider_config.as_object() {
                    for (key, val) in obj {
                        if key != "base_url" && key != "api_key" {
                            custom.insert(key.clone(), val.clone());
                        }
                    }
                }
                custom
            },
        };
        
        // Use the factory to create the provider
        match factory::create_provider(&provider_config).await {
            Ok(provider) => {
                self.provider = Some(provider);
                Ok(())
            },
            Err(e) => {
                // Try to use default provider as fallback
                log::warn!("Failed to create configured provider: {}. Trying default provider...", e);
                match factory::create_default_provider().await {
                    Ok(provider) => {
                        self.provider = Some(provider);
                        Ok(())
                    },
                    Err(e) => {
                        log::error!("Failed to create default provider: {}", e);
                        Err(LLMError::ProviderError(e))
                    },
                }
            },
        }
    }
    
    /// Get a reference to the current provider
    pub(crate) fn get_provider(&self) -> LLMResult<&dyn LocalLLMProvider> {
        match &self.provider {
            Some(provider) => Ok(provider.as_ref()),
            None => Err(LLMError::NoProviderAvailable),
        }
    }
    
    /// List available models
    pub async fn list_available_models(&self) -> LLMResult<Vec<ModelInfo>> {
        let provider = self.get_provider()?;
        Ok(provider.list_available_models().await?)
    }
    
    /// List downloaded models
    pub async fn list_downloaded_models(&self) -> LLMResult<Vec<ModelInfo>> {
        let provider = self.get_provider()?;
        Ok(provider.list_downloaded_models().await?)
    }
    
    /// Get model info
    pub async fn get_model_info(&self, model_id: &str) -> LLMResult<ModelInfo> {
        let provider = self.get_provider()?;
        Ok(provider.get_model_info(model_id).await?)
    }
    
    /// Download a model
    pub async fn download_model(&self, model_id: &str) -> LLMResult<()> {
        let provider = self.get_provider()?;
        Ok(provider.download_model(model_id).await?)
    }
    
    /// Check if a model is loaded
    pub async fn is_model_loaded(&self, model_id: &str) -> LLMResult<bool> {
        let provider = self.get_provider()?;
        Ok(provider.is_model_loaded(model_id).await?)
    }
    
    /// Load a model
    pub async fn load_model(&self, model_id: &str) -> LLMResult<()> {
        let provider = self.get_provider()?;
        Ok(provider.load_model(model_id).await?)
    }
    
    /// Generate text with a model
    pub async fn generate_text(
        &self,
        model_id: Option<&str>,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> LLMResult<CompletionResponse> {
        let provider = self.get_provider()?;
        
        // Determine which model to use
        let model_id = match model_id {
            Some(id) => id,
            None => match &self.config.default_model {
                Some(id) => id,
                None => return Err(LLMError::Unexpected("No model specified and no default model configured".to_string())),
            },
        };
        
        // Determine options to use
        let options = options.unwrap_or_default();
        
        // Generate text
        Ok(provider.generate_text(model_id, prompt, options).await?)
    }
    
    /// Process a generation request
    pub async fn process_request(&self, request: GenerationRequest) -> LLMResult<GenerationResponse> {
        let provider = self.get_provider()?;
        
        match &request.request_type {
            types::GenerationRequestType::TextCompletion { prompt } => {
                let completion = provider.generate_text(&request.model_id, prompt, request.options).await?;
                
                // Convert to GenerationResponse
                let created_at = chrono::Utc::now().to_rfc3339();
                let response = GenerationResponse {
                    text: completion.text,
                    response_type: types::ResponseType::TextCompletion,
                    usage: types::TokenUsage {
                        prompt_tokens: completion.usage.prompt_tokens,
                        completion_tokens: completion.usage.completion_tokens,
                        total_tokens: completion.usage.total_tokens,
                    },
                    truncated: completion.reached_max_tokens,
                    finish_reason: if completion.reached_max_tokens {
                        Some("length".to_string())
                    } else {
                        Some("stop".to_string())
                    },
                    model_id: request.model_id,
                    created_at,
                    metadata: completion.metadata,
                };
                
                Ok(response)
            },
            types::GenerationRequestType::ChatCompletion { messages } => {
                // Convert chat messages to a prompt
                let mut prompt = String::new();
                for message in messages {
                    match message.role.as_str() {
                        "system" => {
                            prompt.push_str(&format!("[SYSTEM]: {}\n\n", message.content));
                        },
                        "user" => {
                            prompt.push_str(&format!("[USER]: {}\n\n", message.content));
                        },
                        "assistant" => {
                            prompt.push_str(&format!("[ASSISTANT]: {}\n\n", message.content));
                        },
                        _ => {
                            prompt.push_str(&format!("[{}]: {}\n\n", message.role, message.content));
                        },
                    }
                }
                prompt.push_str("[ASSISTANT]: ");
                
                // Generate completion
                let completion = provider.generate_text(&request.model_id, &prompt, request.options).await?;
                
                // Convert to GenerationResponse
                let created_at = chrono::Utc::now().to_rfc3339();
                let response = GenerationResponse {
                    text: completion.text,
                    response_type: types::ResponseType::ChatCompletion,
                    usage: types::TokenUsage {
                        prompt_tokens: completion.usage.prompt_tokens,
                        completion_tokens: completion.usage.completion_tokens,
                        total_tokens: completion.usage.total_tokens,
                    },
                    truncated: completion.reached_max_tokens,
                    finish_reason: if completion.reached_max_tokens {
                        Some("length".to_string())
                    } else {
                        Some("stop".to_string())
                    },
                    model_id: request.model_id,
                    created_at,
                    metadata: completion.metadata,
                };
                
                Ok(response)
            },
        }
    }
    
    /// Get available providers
    pub async fn get_available_providers() -> LLMResult<Vec<factory::ProviderInfo>> {
        match factory::get_available_providers().await {
            info if !info.is_empty() => Ok(info),
            _ => {
                log::warn!("No LLM providers available");
                Err(LLMError::NoProviderAvailable)
            }
        }
    }
    
    /// Check if a provider is available
    pub async fn is_provider_available(provider_type: ProviderType, endpoint_url: Option<&str>) -> bool {
        factory::is_provider_available(provider_type, endpoint_url).await
    }
}

/// Create a shared LLM manager with the default configuration
pub async fn create_llm_manager() -> Arc<tokio::sync::Mutex<LLMManager>> {
    // Initialize the factory
    factory::initialize().await;
    
    let mut manager = LLMManager::new();
    if let Err(e) = manager.initialize().await {
        log::error!("Failed to initialize LLM manager: {}", e);
    }
    
    Arc::new(tokio::sync::Mutex::new(manager))
}

/// Get a list of all available providers
pub async fn get_available_providers() -> Vec<factory::ProviderInfo> {
    factory::get_available_providers().await
}

/// Check if a provider is available
pub async fn is_provider_available(provider_type: ProviderType, endpoint_url: Option<&str>) -> bool {
    factory::is_provider_available(provider_type, endpoint_url).await
}