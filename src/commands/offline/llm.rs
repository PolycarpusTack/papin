use tauri::{command, AppHandle, Manager, State, Window};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::anyhow;

use crate::offline::llm::{LocalLLM, ModelInfo, LLMConfig, LLMParameters, DownloadStatus};
use crate::error::{Result, Error};

// --------------------------
// Provider-Related Structs
// --------------------------

/// Provider type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    Ollama,
    LocalAI,
    LlamaCpp,
    Custom(String),
}

impl ProviderType {
    pub fn to_string(&self) -> String {
        match self {
            ProviderType::Ollama => "Ollama".to_string(),
            ProviderType::LocalAI => "LocalAI".to_string(),
            ProviderType::LlamaCpp => "LlamaCpp".to_string(),
            ProviderType::Custom(name) => format!("Custom({})", name),
        }
    }

    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "Ollama" => Ok(ProviderType::Ollama),
            "LocalAI" => Ok(ProviderType::LocalAI),
            "LlamaCpp" => Ok(ProviderType::LlamaCpp),
            s if s.starts_with("Custom(") && s.ends_with(")") => {
                let name = s[7..s.len()-1].to_string();
                Ok(ProviderType::Custom(name))
            },
            _ => Err(Error::InvalidInput(format!("Invalid provider type: {}", s))),
        }
    }
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider type identifier
    pub provider_type: String,
    /// Display name
    pub name: String,
    /// Provider description
    pub description: String,
    /// Provider version
    pub version: String,
    /// Default endpoint URL
    pub default_endpoint: String,
    /// Whether the provider supports text generation
    pub supports_text_generation: bool,
    /// Whether the provider supports chat
    pub supports_chat: bool,
    /// Whether the provider supports embeddings
    pub supports_embeddings: bool,
    /// Whether the provider requires an API key
    pub requires_api_key: bool,
}

/// Provider availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityResult {
    /// Whether the provider is available
    pub available: bool,
    /// Provider version if available
    pub version: Option<String>,
    /// Error message if not available
    pub error: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type identifier
    pub provider_type: String,
    /// Endpoint URL for the provider
    pub endpoint_url: String,
    /// API key for the provider (if required)
    pub api_key: Option<String>,
    /// Default model to use
    pub default_model: Option<String>,
    /// Whether to enable advanced configuration
    pub enable_advanced_config: bool,
    /// Advanced configuration options (provider-specific)
    pub advanced_config: HashMap<String, serde_json::Value>,
}

/// Enhanced model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedModelInfo {
    /// Model identifier
    pub id: String,
    /// Model name for display
    pub name: String,
    /// Model description
    pub description: String,
    /// Model size in bytes
    pub size_bytes: usize,
    /// Whether the model is downloaded
    pub is_downloaded: bool,
    /// Provider-specific metadata
    pub provider_metadata: HashMap<String, serde_json::Value>,
    /// Provider type
    pub provider: String,
    /// Whether the model supports text generation
    pub supports_text_generation: bool,
    /// Whether the model supports completion
    pub supports_completion: bool,
    /// Whether the model supports chat
    pub supports_chat: bool,
    /// Whether the model supports embeddings
    pub supports_embeddings: bool,
    /// Whether the model supports image generation
    pub supports_image_generation: bool,
    /// Quantization level (if applicable)
    pub quantization: Option<String>,
    /// Parameter count in billions
    pub parameter_count_b: Option<f32>,
    /// Context length in tokens
    pub context_length: Option<usize>,
    /// Model family/architecture
    pub model_family: Option<String>,
    /// When the model was created
    pub created_at: Option<String>,
    /// Model tags
    pub tags: Vec<String>,
    /// Model license
    pub license: Option<String>,
}

impl From<ModelInfo> for EnhancedModelInfo {
    fn from(info: ModelInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            description: info.description,
            size_bytes: info.size_mb * 1024 * 1024,
            is_downloaded: info.installed,
            provider_metadata: HashMap::new(),
            provider: "LlamaCpp".to_string(), // Default to LlamaCpp for basic ModelInfo
            supports_text_generation: true,
            supports_completion: true,
            supports_chat: true,
            supports_embeddings: false,
            supports_image_generation: false,
            quantization: None,
            parameter_count_b: None,
            context_length: Some(info.context_size),
            model_family: None,
            created_at: None,
            tags: Vec::new(),
            license: None,
        }
    }
}

/// Enhanced download status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDownloadStatus {
    /// Status type 
    pub status: String,
    /// Not started status (empty object)
    pub NotStarted: Option<HashMap<String, serde_json::Value>>,
    /// In progress status
    pub InProgress: Option<InProgressStatus>,
    /// Completed status
    pub Completed: Option<CompletedStatus>,
    /// Failed status
    pub Failed: Option<FailedStatus>,
    /// Cancelled status
    pub Cancelled: Option<CancelledStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InProgressStatus {
    /// Download progress as percentage
    pub percent: f32,
    /// Bytes downloaded
    pub bytes_downloaded: Option<usize>,
    /// Total bytes to download
    pub total_bytes: Option<usize>,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Download speed in bytes per second
    pub bytes_per_second: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedStatus {
    /// When the download completed
    pub completed_at: Option<String>,
    /// Total download duration in seconds
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedStatus {
    /// Reason for failure
    pub reason: String,
    /// Error code if available
    pub error_code: Option<String>,
    /// When the download failed
    pub failed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledStatus {
    /// When the download was cancelled
    pub cancelled_at: Option<String>,
}

impl From<DownloadStatus> for EnhancedDownloadStatus {
    fn from(status: DownloadStatus) -> Self {
        if status.complete {
            Self {
                status: "Completed".to_string(),
                NotStarted: None,
                InProgress: None,
                Completed: Some(CompletedStatus {
                    completed_at: Some(chrono::Utc::now().to_rfc3339()),
                    duration_seconds: None,
                }),
                Failed: None,
                Cancelled: None,
            }
        } else if let Some(error) = status.error {
            Self {
                status: "Failed".to_string(),
                NotStarted: None,
                InProgress: None,
                Completed: None,
                Failed: Some(FailedStatus {
                    reason: error,
                    error_code: None,
                    failed_at: Some(chrono::Utc::now().to_rfc3339()),
                }),
                Cancelled: None,
            }
        } else {
            Self {
                status: "InProgress".to_string(),
                NotStarted: None,
                InProgress: Some(InProgressStatus {
                    percent: status.progress * 100.0,
                    bytes_downloaded: Some(status.bytes_downloaded),
                    total_bytes: Some(status.total_bytes),
                    eta_seconds: Some(status.eta_seconds),
                    bytes_per_second: Some(status.speed_bps),
                }),
                Completed: None,
                Failed: None,
                Cancelled: None,
            }
        }
    }
}

/// Command response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    /// Whether the command was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error: Option<String>,
    /// Response data if successful
    pub data: Option<T>,
}

impl<T> CommandResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            error: None,
            data: Some(data),
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            error: Some(message.to_string()),
            data: None,
        }
    }
}

// --------------------------
// Provider Manager
// --------------------------

/// Provider manager state
#[derive(Debug)]
pub struct ProviderManager {
    /// Available providers
    providers: Mutex<HashMap<String, ProviderInfo>>,
    /// Provider availability status
    availability: Mutex<HashMap<String, AvailabilityResult>>,
    /// Active provider
    active_provider: Mutex<Option<ProviderType>>,
    /// Provider configurations
    configs: Mutex<HashMap<String, ProviderConfig>>,
    /// LLM instances for each provider
    llm_instances: Mutex<HashMap<String, Arc<LocalLLM>>>,
}

impl Default for ProviderManager {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Add default providers
        providers.insert(
            "Ollama".to_string(),
            ProviderInfo {
                provider_type: "Ollama".to_string(),
                name: "Ollama".to_string(),
                description: "Local model runner for LLama and other models".to_string(),
                version: "1.0.0".to_string(),
                default_endpoint: "http://localhost:11434".to_string(),
                supports_text_generation: true,
                supports_chat: true,
                supports_embeddings: true,
                requires_api_key: false,
            },
        );
        
        providers.insert(
            "LocalAI".to_string(),
            ProviderInfo {
                provider_type: "LocalAI".to_string(),
                name: "LocalAI".to_string(),
                description: "Self-hosted OpenAI API compatible server".to_string(),
                version: "1.0.0".to_string(),
                default_endpoint: "http://localhost:8080".to_string(),
                supports_text_generation: true,
                supports_chat: true,
                supports_embeddings: true,
                requires_api_key: false,
            },
        );
        
        providers.insert(
            "LlamaCpp".to_string(),
            ProviderInfo {
                provider_type: "LlamaCpp".to_string(),
                name: "llama.cpp".to_string(),
                description: "Embedded llama.cpp integration for efficient local inference".to_string(),
                version: "1.0.0".to_string(),
                default_endpoint: "local://models".to_string(),
                supports_text_generation: true,
                supports_chat: true,
                supports_embeddings: false,
                requires_api_key: false,
            },
        );
        
        // Create default configs for each provider
        let mut configs = HashMap::new();
        for (provider_type, info) in &providers {
            configs.insert(
                provider_type.clone(),
                ProviderConfig {
                    provider_type: provider_type.clone(),
                    endpoint_url: info.default_endpoint.clone(),
                    api_key: None,
                    default_model: None,
                    enable_advanced_config: false,
                    advanced_config: HashMap::new(),
                },
            );
        }
        
        // Create LLM instances
        let mut llm_instances = HashMap::new();
        llm_instances.insert("LlamaCpp".to_string(), Arc::new(LocalLLM::new_manager()));
        
        Self {
            providers: Mutex::new(providers),
            availability: Mutex::new(HashMap::new()),
            active_provider: Mutex::new(Some(ProviderType::LlamaCpp)),
            configs: Mutex::new(configs),
            llm_instances: Mutex::new(llm_instances),
        }
    }
}

impl ProviderManager {
    /// Get all available providers
    pub fn get_all_providers(&self) -> Result<Vec<ProviderInfo>> {
        let providers = self.providers.lock().unwrap();
        Ok(providers.values().cloned().collect())
    }
    
    /// Get a specific provider
    pub fn get_provider(&self, provider_type: &str) -> Result<ProviderInfo> {
        let providers = self.providers.lock().unwrap();
        
        providers.get(provider_type)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Provider {} not found", provider_type)))
    }
    
    /// Add a custom provider
    pub fn add_provider(&self, provider: ProviderInfo) -> Result<()> {
        let mut providers = self.providers.lock().unwrap();
        
        // Check if provider type is valid
        if provider.provider_type.starts_with("Custom(") && provider.provider_type.ends_with(")") {
            providers.insert(provider.provider_type.clone(), provider.clone());
            
            // Add default config
            let mut configs = self.configs.lock().unwrap();
            configs.insert(
                provider.provider_type.clone(),
                ProviderConfig {
                    provider_type: provider.provider_type.clone(),
                    endpoint_url: provider.default_endpoint.clone(),
                    api_key: None,
                    default_model: None,
                    enable_advanced_config: false,
                    advanced_config: HashMap::new(),
                },
            );
            
            Ok(())
        } else {
            Err(Error::InvalidInput("Custom provider type must start with 'Custom(' and end with ')'".to_string()))
        }
    }
    
    /// Remove a custom provider
    pub fn remove_provider(&self, provider_type: &str) -> Result<()> {
        let mut providers = self.providers.lock().unwrap();
        
        // Only allow removing custom providers
        if provider_type.starts_with("Custom(") && provider_type.ends_with(")") {
            if providers.remove(provider_type).is_some() {
                // Remove config
                let mut configs = self.configs.lock().unwrap();
                configs.remove(provider_type);
                
                // Remove LLM instance if exists
                let mut llm_instances = self.llm_instances.lock().unwrap();
                llm_instances.remove(provider_type);
                
                Ok(())
            } else {
                Err(Error::NotFound(format!("Provider {} not found", provider_type)))
            }
        } else {
            Err(Error::InvalidInput("Only custom providers can be removed".to_string()))
        }
    }
    
    /// Check availability of all providers
    pub fn check_all_providers(&self) -> Result<HashMap<String, AvailabilityResult>> {
        let providers = self.providers.lock().unwrap();
        let configs = self.configs.lock().unwrap();
        let mut availability = self.availability.lock().unwrap();
        
        for (provider_type, provider) in providers.iter() {
            if let Some(config) = configs.get(provider_type) {
                // Check provider availability
                let start_time = std::time::Instant::now();
                let available = match provider_type.as_str() {
                    "Ollama" => self.check_ollama(&config.endpoint_url),
                    "LocalAI" => self.check_localai(&config.endpoint_url),
                    "LlamaCpp" => self.check_llamacpp(),
                    _ if provider_type.starts_with("Custom(") => self.check_custom(&config.endpoint_url),
                    _ => Err(anyhow!("Unsupported provider type")),
                };
                
                let elapsed = start_time.elapsed();
                
                // Update availability
                match available {
                    Ok(version) => {
                        availability.insert(
                            provider_type.clone(),
                            AvailabilityResult {
                                available: true,
                                version: Some(version),
                                error: None,
                                response_time_ms: Some(elapsed.as_millis() as u64),
                            },
                        );
                    }
                    Err(e) => {
                        availability.insert(
                            provider_type.clone(),
                            AvailabilityResult {
                                available: false,
                                version: None,
                                error: Some(e.to_string()),
                                response_time_ms: Some(elapsed.as_millis() as u64),
                            },
                        );
                    }
                }
            }
        }
        
        Ok(availability.clone())
    }
    
    /// Check availability of a specific provider
    pub fn check_provider(&self, provider_type: &str) -> Result<AvailabilityResult> {
        let configs = self.configs.lock().unwrap();
        let mut availability = self.availability.lock().unwrap();
        
        let config = configs.get(provider_type)
            .ok_or_else(|| Error::NotFound(format!("Provider config for {} not found", provider_type)))?;
        
        // Check provider availability
        let start_time = std::time::Instant::now();
        let available = match provider_type {
            "Ollama" => self.check_ollama(&config.endpoint_url),
            "LocalAI" => self.check_localai(&config.endpoint_url),
            "LlamaCpp" => self.check_llamacpp(),
            _ if provider_type.starts_with("Custom(") => self.check_custom(&config.endpoint_url),
            _ => Err(anyhow!("Unsupported provider type")),
        };
        
        let elapsed = start_time.elapsed();
        
        // Update availability
        let result = match available {
            Ok(version) => {
                AvailabilityResult {
                    available: true,
                    version: Some(version),
                    error: None,
                    response_time_ms: Some(elapsed.as_millis() as u64),
                }
            }
            Err(e) => {
                AvailabilityResult {
                    available: false,
                    version: None,
                    error: Some(e.to_string()),
                    response_time_ms: Some(elapsed.as_millis() as u64),
                }
            }
        };
        
        // Store result
        availability.insert(provider_type.to_string(), result.clone());
        
        Ok(result)
    }
    
    // Check Ollama availability
    fn check_ollama(&self, endpoint: &str) -> anyhow::Result<String> {
        // This is a simplified check - in a real implementation,
        // you would make an HTTP request to the Ollama API
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            // Simulate HTTP request
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            if endpoint.contains("localhost") || endpoint.contains("127.0.0.1") {
                Ok("0.1.0".to_string())
            } else {
                Err(anyhow!("Could not connect to Ollama endpoint"))
            }
        } else {
            Err(anyhow!("Invalid Ollama endpoint URL"))
        }
    }
    
    // Check LocalAI availability
    fn check_localai(&self, endpoint: &str) -> anyhow::Result<String> {
        // Similar to check_ollama, but for LocalAI
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            // Simulate HTTP request
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            if endpoint.contains("localhost") || endpoint.contains("127.0.0.1") {
                Ok("1.0.0".to_string())
            } else {
                Err(anyhow!("Could not connect to LocalAI endpoint"))
            }
        } else {
            Err(anyhow!("Invalid LocalAI endpoint URL"))
        }
    }
    
    // Check LlamaCpp availability
    fn check_llamacpp(&self) -> anyhow::Result<String> {
        // Check if any models are available
        let llm_instances = self.llm_instances.lock().unwrap();
        
        if let Some(llm) = llm_instances.get("LlamaCpp") {
            if llm.list_models().is_empty() {
                Err(anyhow!("No models available for llama.cpp"))
            } else {
                Ok("0.2.0".to_string())
            }
        } else {
            Err(anyhow!("llama.cpp instance not found"))
        }
    }
    
    // Check custom provider availability
    fn check_custom(&self, endpoint: &str) -> anyhow::Result<String> {
        // Basic check for custom providers
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            // Simulate HTTP request
            std::thread::sleep(std::time::Duration::from_millis(150));
            
            if endpoint.contains("localhost") || endpoint.contains("127.0.0.1") {
                Ok("1.0.0".to_string())
            } else {
                Err(anyhow!("Could not connect to custom provider endpoint"))
            }
        } else {
            Err(anyhow!("Invalid custom provider endpoint URL"))
        }
    }
    
    /// Get the active provider type
    pub fn get_active_provider(&self) -> Result<ProviderType> {
        let active_provider = self.active_provider.lock().unwrap();
        
        active_provider.clone()
            .ok_or_else(|| Error::NotInitialized("No active provider set".to_string()))
    }
    
    /// Set the active provider
    pub fn set_active_provider(&self, provider_type: &str) -> Result<()> {
        let providers = self.providers.lock().unwrap();
        
        if providers.contains_key(provider_type) {
            let provider_type = ProviderType::from_string(provider_type)?;
            *self.active_provider.lock().unwrap() = Some(provider_type);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Provider {} not found", provider_type)))
        }
    }
    
    /// Get the configuration for a provider
    pub fn get_provider_config(&self, provider_type: &str) -> Result<ProviderConfig> {
        let configs = self.configs.lock().unwrap();
        
        configs.get(provider_type)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Provider config for {} not found", provider_type)))
    }
    
    /// Update the configuration for a provider
    pub fn update_provider_config(&self, config: ProviderConfig) -> Result<()> {
        let mut configs = self.configs.lock().unwrap();
        
        if configs.contains_key(&config.provider_type) {
            configs.insert(config.provider_type.clone(), config);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Provider {} not found", config.provider_type)))
        }
    }
    
    /// List all available models for a provider
    pub fn list_available_models(&self, provider_type: &str) -> Result<Vec<EnhancedModelInfo>> {
        match provider_type {
            "LlamaCpp" => {
                let llm_instances = self.llm_instances.lock().unwrap();
                
                if let Some(llm) = llm_instances.get("LlamaCpp") {
                    let models = llm.list_models();
                    Ok(models.into_iter().map(EnhancedModelInfo::from).collect())
                } else {
                    Err(Error::NotInitialized("LlamaCpp instance not found".to_string()))
                }
            },
            "Ollama" => {
                // Simulate fetching models from Ollama API
                self.simulate_ollama_models()
            },
            "LocalAI" => {
                // Simulate fetching models from LocalAI API
                self.simulate_localai_models()
            },
            _ if provider_type.starts_with("Custom(") => {
                // Simulate fetching models from custom provider
                self.simulate_custom_models(provider_type)
            },
            _ => Err(Error::InvalidInput(format!("Unsupported provider type: {}", provider_type))),
        }
    }
    
    // Simulate fetching models from Ollama
    fn simulate_ollama_models(&self) -> Result<Vec<EnhancedModelInfo>> {
        // This would be an API call in a real implementation
        let models = vec![
            EnhancedModelInfo {
                id: "llama2".to_string(),
                name: "Llama 2".to_string(),
                description: "Open-source LLM with 7B parameters".to_string(),
                size_bytes: 4 * 1024 * 1024 * 1024, // 4GB
                is_downloaded: true,
                provider_metadata: HashMap::new(),
                provider: "Ollama".to_string(),
                supports_text_generation: true,
                supports_completion: true,
                supports_chat: true,
                supports_embeddings: false,
                supports_image_generation: false,
                quantization: Some("Q4_K_M".to_string()),
                parameter_count_b: Some(7.0),
                context_length: Some(4096),
                model_family: Some("Llama".to_string()),
                created_at: Some("2023-07-18T00:00:00Z".to_string()),
                tags: vec!["llama".to_string(), "meta".to_string()],
                license: Some("Meta License".to_string()),
            },
            EnhancedModelInfo {
                id: "mistral".to_string(),
                name: "Mistral".to_string(),
                description: "Mistral 7B model with excellent performance".to_string(),
                size_bytes: 5 * 1024 * 1024 * 1024, // 5GB
                is_downloaded: false,
                provider_metadata: HashMap::new(),
                provider: "Ollama".to_string(),
                supports_text_generation: true,
                supports_completion: true,
                supports_chat: true,
                supports_embeddings: false,
                supports_image_generation: false,
                quantization: Some("Q4_K_M".to_string()),
                parameter_count_b: Some(7.0),
                context_length: Some(8192),
                model_family: Some("Mistral".to_string()),
                created_at: Some("2023-09-15T00:00:00Z".to_string()),
                tags: vec!["mistral".to_string(), "mistral-ai".to_string()],
                license: Some("Apache 2.0".to_string()),
            },
        ];
        
        Ok(models)
    }
    
    // Simulate fetching models from LocalAI
    fn simulate_localai_models(&self) -> Result<Vec<EnhancedModelInfo>> {
        // This would be an API call in a real implementation
        let models = vec![
            EnhancedModelInfo {
                id: "ggml-gpt4all-j".to_string(),
                name: "GPT4All-J".to_string(),
                description: "Compact and efficient GPT4All-J model".to_string(),
                size_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                is_downloaded: true,
                provider_metadata: HashMap::new(),
                provider: "LocalAI".to_string(),
                supports_text_generation: true,
                supports_completion: true,
                supports_chat: true,
                supports_embeddings: false,
                supports_image_generation: false,
                quantization: Some("Q4_0".to_string()),
                parameter_count_b: Some(6.0),
                context_length: Some(2048),
                model_family: Some("GPT".to_string()),
                created_at: Some("2023-04-28T00:00:00Z".to_string()),
                tags: vec!["gpt".to_string(), "compact".to_string()],
                license: Some("MIT".to_string()),
            },
            EnhancedModelInfo {
                id: "ggml-vicuna-13b-1.1".to_string(),
                name: "Vicuna 13B".to_string(),
                description: "Vicuna 13B fine-tuned model".to_string(),
                size_bytes: 8 * 1024 * 1024 * 1024, // 8GB
                is_downloaded: false,
                provider_metadata: HashMap::new(),
                provider: "LocalAI".to_string(),
                supports_text_generation: true,
                supports_completion: true,
                supports_chat: true,
                supports_embeddings: false,
                supports_image_generation: false,
                quantization: Some("Q4_0".to_string()),
                parameter_count_b: Some(13.0),
                context_length: Some(4096),
                model_family: Some("Vicuna".to_string()),
                created_at: Some("2023-05-10T00:00:00Z".to_string()),
                tags: vec!["vicuna".to_string(), "large".to_string()],
                license: Some("License-to-use".to_string()),
            },
        ];
        
        Ok(models)
    }
    
    // Simulate fetching models from custom provider
    fn simulate_custom_models(&self, provider_type: &str) -> Result<Vec<EnhancedModelInfo>> {
        // This would be an API call in a real implementation
        let models = vec![
            EnhancedModelInfo {
                id: format!("{}-model1", provider_type),
                name: "Custom Model 1".to_string(),
                description: "A custom model for demonstration".to_string(),
                size_bytes: 1 * 1024 * 1024 * 1024, // 1GB
                is_downloaded: true,
                provider_metadata: HashMap::new(),
                provider: provider_type.to_string(),
                supports_text_generation: true,
                supports_completion: true,
                supports_chat: true,
                supports_embeddings: true,
                supports_image_generation: false,
                quantization: None,
                parameter_count_b: Some(2.0),
                context_length: Some(4096),
                model_family: Some("Custom".to_string()),
                created_at: Some("2023-12-01T00:00:00Z".to_string()),
                tags: vec!["custom".to_string(), "demo".to_string()],
                license: Some("Proprietary".to_string()),
            },
        ];
        
        Ok(models)
    }
    
    /// List all downloaded models for a provider
    pub fn list_downloaded_models(&self, provider_type: &str) -> Result<Vec<EnhancedModelInfo>> {
        // Get all models and filter for downloaded ones
        let all_models = self.list_available_models(provider_type)?;
        let downloaded = all_models.into_iter().filter(|m| m.is_downloaded).collect();
        Ok(downloaded)
    }
    
    /// Get the download status for a model
    pub fn get_download_status(&self, model_id: &str) -> Result<EnhancedDownloadStatus> {
        // For simplicity, we'll just use the LlamaCpp provider here
        let llm_instances = self.llm_instances.lock().unwrap();
        
        if let Some(llm) = llm_instances.get("LlamaCpp") {
            let download_id = format!("download_{}", model_id);
            
            if let Some(status) = llm.get_download_status(&download_id) {
                Ok(EnhancedDownloadStatus::from(status))
            } else {
                // If no download status, assume not started
                Ok(EnhancedDownloadStatus {
                    status: "NotStarted".to_string(),
                    NotStarted: Some(HashMap::new()),
                    InProgress: None,
                    Completed: None,
                    Failed: None,
                    Cancelled: None,
                })
            }
        } else {
            Err(Error::NotInitialized("LlamaCpp instance not found".to_string()))
        }
    }
    
    /// Download a model
    pub fn download_model(&self, provider_type: &str, model_id: &str) -> Result<EnhancedDownloadStatus> {
        match provider_type {
            "LlamaCpp" => {
                let llm_instances = self.llm_instances.lock().unwrap();
                
                if let Some(llm) = llm_instances.get("LlamaCpp") {
                    match llm.download_model(model_id) {
                        Ok(download_id) => {
                            if let Some(status) = llm.get_download_status(&download_id) {
                                Ok(EnhancedDownloadStatus::from(status))
                            } else {
                                Err(Error::Internal("Failed to get download status".to_string()))
                            }
                        },
                        Err(e) => Err(Error::Internal(e)),
                    }
                } else {
                    Err(Error::NotInitialized("LlamaCpp instance not found".to_string()))
                }
            },
            "Ollama" | "LocalAI" | _ if provider_type.starts_with("Custom(") => {
                // Simulate starting a download
                Ok(EnhancedDownloadStatus {
                    status: "InProgress".to_string(),
                    NotStarted: None,
                    InProgress: Some(InProgressStatus {
                        percent: 0.0,
                        bytes_downloaded: Some(0),
                        total_bytes: Some(1 * 1024 * 1024 * 1024), // 1GB
                        eta_seconds: Some(600), // 10 minutes
                        bytes_per_second: Some(2 * 1024 * 1024), // 2MB/s
                    }),
                    Completed: None,
                    Failed: None,
                    Cancelled: None,
                })
            },
            _ => Err(Error::InvalidInput(format!("Unsupported provider type: {}", provider_type))),
        }
    }
    
    /// Cancel a model download
    pub fn cancel_download(&self, provider_type: &str, model_id: &str) -> Result<bool> {
        match provider_type {
            "LlamaCpp" => {
                let llm_instances = self.llm_instances.lock().unwrap();
                
                if let Some(llm) = llm_instances.get("LlamaCpp") {
                    let download_id = format!("download_{}", model_id);
                    match llm.cancel_download(&download_id) {
                        Ok(_) => Ok(true),
                        Err(e) => Err(Error::Internal(e)),
                    }
                } else {
                    Err(Error::NotInitialized("LlamaCpp instance not found".to_string()))
                }
            },
            "Ollama" | "LocalAI" | _ if provider_type.starts_with("Custom(") => {
                // Simulate cancelling a download
                Ok(true)
            },
            _ => Err(Error::InvalidInput(format!("Unsupported provider type: {}", provider_type))),
        }
    }
    
    /// Delete a model
    pub fn delete_model(&self, provider_type: &str, model_id: &str) -> Result<bool> {
        // In a real implementation, you would call the appropriate provider API
        // For now, just return success
        Ok(true)
    }
    
    /// Generate text using the active provider
    pub fn generate_text(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let active_provider = self.get_active_provider()?;
        
        match active_provider {
            ProviderType::LlamaCpp => {
                let llm_instances = self.llm_instances.lock().unwrap();
                
                if let Some(llm) = llm_instances.get("LlamaCpp") {
                    Ok(llm.generate(prompt, max_tokens))
                } else {
                    Err(Error::NotInitialized("LlamaCpp instance not found".to_string()))
                }
            },
            ProviderType::Ollama => {
                // Simulate Ollama text generation
                std::thread::sleep(std::time::Duration::from_millis(500));
                Ok(format!("Generated text from Ollama: {}", prompt))
            },
            ProviderType::LocalAI => {
                // Simulate LocalAI text generation
                std::thread::sleep(std::time::Duration::from_millis(700));
                Ok(format!("Generated text from LocalAI: {}", prompt))
            },
            ProviderType::Custom(name) => {
                // Simulate custom provider text generation
                std::thread::sleep(std::time::Duration::from_millis(600));
                Ok(format!("Generated text from custom provider {}: {}", name, prompt))
            },
        }
    }
}

// --------------------------
// Tauri Commands
// --------------------------

/// Get all available providers
#[command]
pub async fn get_all_providers(
    state: State<'_, ProviderManager>,
) -> CommandResponse<Vec<ProviderInfo>> {
    match state.get_all_providers() {
        Ok(providers) => CommandResponse::success(providers),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Check availability of all providers
#[command]
pub async fn get_all_provider_availability(
    state: State<'_, ProviderManager>,
) -> CommandResponse<HashMap<String, AvailabilityResult>> {
    match state.check_all_providers() {
        Ok(availability) => CommandResponse::success(availability),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Check availability of a specific provider
#[command]
pub async fn check_provider_availability(
    provider_type: String,
    state: State<'_, ProviderManager>,
) -> CommandResponse<AvailabilityResult> {
    match state.check_provider(&provider_type) {
        Ok(availability) => CommandResponse::success(availability),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Add a custom provider
#[command]
pub async fn add_custom_provider(
    provider: ProviderInfo,
    state: State<'_, ProviderManager>,
) -> CommandResponse<bool> {
    match state.add_provider(provider) {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Remove a custom provider
#[command]
pub async fn remove_custom_provider(
    provider_type: String,
    state: State<'_, ProviderManager>,
) -> CommandResponse<bool> {
    match state.remove_provider(&provider_type) {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get the active provider
#[command]
pub async fn get_active_provider(
    state: State<'_, ProviderManager>,
) -> CommandResponse<String> {
    match state.get_active_provider() {
        Ok(provider_type) => CommandResponse::success(provider_type.to_string()),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Set the active provider
#[command]
pub async fn set_active_provider(
    provider_type: String,
    state: State<'_, ProviderManager>,
) -> CommandResponse<bool> {
    match state.set_active_provider(&provider_type) {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get the configuration for a provider
#[command]
pub async fn get_provider_config(
    provider_type: String,
    state: State<'_, ProviderManager>,
) -> CommandResponse<ProviderConfig> {
    match state.get_provider_config(&provider_type) {
        Ok(config) => CommandResponse::success(config),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Update the configuration for a provider
#[command]
pub async fn update_provider_config(
    config: ProviderConfig,
    state: State<'_, ProviderManager>,
) -> CommandResponse<bool> {
    match state.update_provider_config(config) {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// List available models for a provider
#[command]
pub async fn list_available_models(
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<Vec<EnhancedModelInfo>> {
    // Use active provider if none specified
    let provider_type = match provider_type {
        Some(pt) => pt,
        None => match state.get_active_provider() {
            Ok(pt) => pt.to_string(),
            Err(e) => return CommandResponse::error(&e.to_string()),
        },
    };
    
    match state.list_available_models(&provider_type) {
        Ok(models) => CommandResponse::success(models),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// List downloaded models for a provider
#[command]
pub async fn list_downloaded_models(
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<Vec<EnhancedModelInfo>> {
    // Use active provider if none specified
    let provider_type = match provider_type {
        Some(pt) => pt,
        None => match state.get_active_provider() {
            Ok(pt) => pt.to_string(),
            Err(e) => return CommandResponse::error(&e.to_string()),
        },
    };
    
    match state.list_downloaded_models(&provider_type) {
        Ok(models) => CommandResponse::success(models),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get download status for a model
#[command]
pub async fn get_download_status(
    model_id: String,
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<EnhancedDownloadStatus> {
    match state.get_download_status(&model_id) {
        Ok(status) => CommandResponse::success(status),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Download a model
#[command]
pub async fn download_model(
    model_id: String,
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<EnhancedDownloadStatus> {
    // Use active provider if none specified
    let provider_type = match provider_type {
        Some(pt) => pt,
        None => match state.get_active_provider() {
            Ok(pt) => pt.to_string(),
            Err(e) => return CommandResponse::error(&e.to_string()),
        },
    };
    
    match state.download_model(&provider_type, &model_id) {
        Ok(status) => CommandResponse::success(status),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Cancel a model download
#[command]
pub async fn cancel_download(
    model_id: String,
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<bool> {
    // Use active provider if none specified
    let provider_type = match provider_type {
        Some(pt) => pt,
        None => match state.get_active_provider() {
            Ok(pt) => pt.to_string(),
            Err(e) => return CommandResponse::error(&e.to_string()),
        },
    };
    
    match state.cancel_download(&provider_type, &model_id) {
        Ok(success) => CommandResponse::success(success),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Delete a model
#[command]
pub async fn delete_model(
    model_id: String,
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<bool> {
    // Use active provider if none specified
    let provider_type = match provider_type {
        Some(pt) => pt,
        None => match state.get_active_provider() {
            Ok(pt) => pt.to_string(),
            Err(e) => return CommandResponse::error(&e.to_string()),
        },
    };
    
    match state.delete_model(&provider_type, &model_id) {
        Ok(success) => CommandResponse::success(success),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Generate text using a model
#[command]
pub async fn generate_text(
    prompt: String,
    max_tokens: Option<usize>,
    model_id: Option<String>,
    provider_type: Option<String>,
    state: State<'_, ProviderManager>,
) -> CommandResponse<String> {
    // Default max tokens if not specified
    let max_tokens = max_tokens.unwrap_or(1024);
    
    match state.generate_text(&prompt, max_tokens) {
        Ok(text) => CommandResponse::success(text),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Register all LLM commands
pub fn register_commands(app: &mut tauri::App) -> Result<()> {
    // Create and manage the provider manager
    let provider_manager = ProviderManager::default();
    app.manage(provider_manager);
    
    Ok(())
}

// --------------------------
// Provider Discovery Commands
// --------------------------

/// Scan for LLM providers
#[command]
pub async fn scan_for_providers(
    app_handle: AppHandle,
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    
    match offline_manager.scan_for_llm_providers().await {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e),
    }
}

/// Get LLM provider discovery status
#[command]
pub async fn get_discovery_status(
    app_handle: AppHandle,
) -> CommandResponse<HashMap<String, crate::offline::llm::discovery::InstallationInfo>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let discovery = offline_manager.get_llm_discovery();
    
    let installations = discovery.get_installations();
    CommandResponse::success(installations)
}

/// Get LLM provider suggestions
#[command]
pub async fn get_provider_suggestions(
    app_handle: AppHandle,
) -> CommandResponse<Vec<crate::offline::llm::discovery::ProviderSuggestion>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let suggestions = offline_manager.get_provider_suggestions();
    
    CommandResponse::success(suggestions)
}

/// Get LLM provider discovery configuration
#[command]
pub async fn get_discovery_config(
    app_handle: AppHandle,
) -> CommandResponse<crate::offline::llm::discovery::DiscoveryConfig> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let discovery = offline_manager.get_llm_discovery();
    
    let config = discovery.get_config();
    CommandResponse::success(config)
}

/// Update LLM provider discovery configuration
#[command]
pub async fn update_discovery_config(
    config: crate::offline::llm::discovery::DiscoveryConfig,
    app_handle: AppHandle,
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let discovery = offline_manager.get_llm_discovery();
    
    discovery.update_config(config);
    CommandResponse::success(true)
}

/// Auto-configure providers based on discovery
#[command]
pub async fn auto_configure_providers(
    app_handle: AppHandle,
    state: State<'_, ProviderManager>,
) -> CommandResponse<Vec<ProviderConfig>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    
    // Get provider configs from discovery
    let configs = offline_manager.get_provider_configs();
    
    // Update provider manager with discovered configs
    for config in &configs {
        if let Err(e) = state.update_provider_config(config.clone()) {
            warn!("Failed to configure provider {}: {}", config.provider_type, e);
        }
    }
    
    CommandResponse::success(configs)
}

// --------------------------
// Migration Commands
// --------------------------

/// Check for legacy LLM system
#[command]
pub async fn check_legacy_system(
    app_handle: AppHandle,
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    match migration.detect_legacy_system().await {
        Ok(detected) => CommandResponse::success(detected),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get migration status
#[command]
pub async fn get_migration_status(
    app_handle: AppHandle,
) -> CommandResponse<crate::offline::llm::migration::MigrationStatus> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    let status = migration.get_status();
    CommandResponse::success(status)
}

/// Run migration
#[command]
pub async fn run_migration(
    options: crate::offline::llm::migration::MigrationOptions,
    app_handle: AppHandle,
) -> CommandResponse<crate::offline::llm::migration::MigrationNotification> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    
    match offline_manager.run_llm_migration(options).await {
        Ok(notification) => CommandResponse::success(notification),
        Err(e) => CommandResponse::error(&e),
    }
}

/// Get migration configuration
#[command]
pub async fn get_migration_config(
    app_handle: AppHandle,
) -> CommandResponse<crate::offline::llm::migration::MigrationConfig> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    let config = migration.get_config();
    CommandResponse::success(config)
}

/// Update migration configuration
#[command]
pub async fn update_migration_config(
    config: crate::offline::llm::migration::MigrationConfig,
    app_handle: AppHandle,
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    migration.update_config(config);
    CommandResponse::success(true)
}

/// Opt out of migration
#[command]
pub async fn opt_out_of_migration(
    app_handle: AppHandle,
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    migration.opt_out();
    CommandResponse::success(true)
}

/// Get model mappings
#[command]
pub async fn get_model_mappings(
    app_handle: AppHandle,
) -> CommandResponse<Vec<crate::offline::llm::migration::LegacyModelMapping>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    let mappings = migration.get_model_mappings();
    CommandResponse::success(mappings)
}

/// Get provider mappings
#[command]
pub async fn get_provider_mappings(
    app_handle: AppHandle,
) -> CommandResponse<Vec<crate::offline::llm::migration::ProviderMapping>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let migration = offline_manager.get_llm_migration();
    
    let mappings = migration.get_provider_mappings();
    CommandResponse::success(mappings)
}

// Generate Tauri command handler
pub fn init_commands() -> Vec<(&'static str, Box<dyn Fn() + Send + Sync + 'static>)> {
    vec![
        ("get_all_providers", Box::new(|| {})),
        ("get_all_provider_availability", Box::new(|| {})),
        ("check_provider_availability", Box::new(|| {})),
        ("add_custom_provider", Box::new(|| {})),
        ("remove_custom_provider", Box::new(|| {})),
        ("get_active_provider", Box::new(|| {})),
        ("set_active_provider", Box::new(|| {})),
        ("get_provider_config", Box::new(|| {})),
        ("update_provider_config", Box::new(|| {})),
        ("list_available_models", Box::new(|| {})),
        ("list_downloaded_models", Box::new(|| {})),
        ("get_download_status", Box::new(|| {})),
        ("download_model", Box::new(|| {})),
        ("cancel_download", Box::new(|| {})),
        ("delete_model", Box::new(|| {})),
        ("generate_text", Box::new(|| {})),
        ("scan_for_providers", Box::new(|| {})),
        ("get_discovery_status", Box::new(|| {})),
        ("get_provider_suggestions", Box::new(|| {})),
        ("get_discovery_config", Box::new(|| {})),
        ("update_discovery_config", Box::new(|| {})),
        ("auto_configure_providers", Box::new(|| {})),
        ("check_legacy_system", Box::new(|| {})),
        ("get_migration_status", Box::new(|| {})),
        ("run_migration", Box::new(|| {})),
        ("get_migration_config", Box::new(|| {})),
        ("update_migration_config", Box::new(|| {})),
        ("opt_out_of_migration", Box::new(|| {})),
        ("get_model_mappings", Box::new(|| {})),
        ("get_provider_mappings", Box::new(|| {})),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_type_conversion() {
        let ollama = ProviderType::Ollama;
        let str_value = ollama.to_string();
        assert_eq!(str_value, "Ollama");
        
        let parsed = ProviderType::from_string(&str_value).unwrap();
        assert!(matches!(parsed, ProviderType::Ollama));
        
        let custom = ProviderType::Custom("MyProvider".to_string());
        let str_value = custom.to_string();
        assert_eq!(str_value, "Custom(MyProvider)");
        
        let parsed = ProviderType::from_string(&str_value).unwrap();
        assert!(matches!(parsed, ProviderType::Custom(name) if name == "MyProvider"));
    }
    
    #[test]
    fn test_provider_manager() {
        let manager = ProviderManager::default();
        
        // Check initial providers
        let providers = manager.get_all_providers().unwrap();
        assert_eq!(providers.len(), 3);
        
        // Check provider types
        let provider_types: Vec<String> = providers.iter().map(|p| p.provider_type.clone()).collect();
        assert!(provider_types.contains(&"Ollama".to_string()));
        assert!(provider_types.contains(&"LocalAI".to_string()));
        assert!(provider_types.contains(&"LlamaCpp".to_string()));
        
        // Check active provider
        let active = manager.get_active_provider().unwrap();
        assert!(matches!(active, ProviderType::LlamaCpp));
    }
    
    #[test]
    fn test_download_status_conversion() {
        let status = DownloadStatus {
            model_id: "test".to_string(),
            progress: 0.5,
            bytes_downloaded: 500,
            total_bytes: 1000,
            speed_bps: 100,
            eta_seconds: 5,
            complete: false,
            error: None,
        };
        
        let enhanced = EnhancedDownloadStatus::from(status);
        assert_eq!(enhanced.status, "InProgress");
        assert!(enhanced.InProgress.is_some());
        
        let in_progress = enhanced.InProgress.unwrap();
        assert_eq!(in_progress.percent, 50.0);
        assert_eq!(in_progress.bytes_downloaded, Some(500));
        assert_eq!(in_progress.total_bytes, Some(1000));
        assert_eq!(in_progress.eta_seconds, Some(5));
        assert_eq!(in_progress.bytes_per_second, Some(100));
    }
}