//! Factory for creating and managing LLM providers
//!
//! This module provides functions for instantiating, checking availability, and
//! managing local LLM providers. It includes a registry of available providers
//! and utilities for determining which providers are currently usable.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::offline::llm::provider::{LocalLLMProvider, LLMProviderError, LLMProviderResult};
use crate::offline::llm::providers::{ollama::OllamaProvider, localai::LocalAIProvider};
use crate::offline::llm::types::{ProviderType, ProviderConfig};

/// The registry for supported LLM providers
static PROVIDER_REGISTRY: Lazy<RwLock<ProviderRegistry>> = Lazy::new(|| {
    RwLock::new(ProviderRegistry::new())
});

/// A registry of all supported LLM providers
#[derive(Debug)]
pub struct ProviderRegistry {
    /// All registered providers
    providers: HashMap<ProviderType, ProviderInfo>,
    /// HTTP client for checking provider availability
    client: Client,
}

/// Information about a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Type of the provider
    pub provider_type: ProviderType,
    /// Display name of the provider
    pub name: String,
    /// Description of the provider
    pub description: String,
    /// Version of the provider
    pub version: String,
    /// Default endpoint URL for the provider
    pub default_endpoint: String,
    /// Whether the provider supports text generation
    pub supports_text_generation: bool,
    /// Whether the provider supports chat completion
    pub supports_chat: bool,
    /// Whether the provider supports embeddings
    pub supports_embeddings: bool,
    /// Whether the provider requires an API key
    pub requires_api_key: bool,
    /// Health check endpoint for the provider
    pub health_check_endpoint: Option<String>,
    /// Current availability status
    #[serde(skip)]
    pub available: bool,
}

/// Result from checking provider availability
#[derive(Debug, Clone)]
pub struct AvailabilityResult {
    /// Whether the provider is available
    pub available: bool,
    /// The detected version if available
    pub version: Option<String>,
    /// Error message if the provider is not available
    pub error: Option<String>,
    /// Response time in milliseconds if available
    pub response_time_ms: Option<u64>,
}

impl ProviderRegistry {
    /// Create a new provider registry with default providers registered
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        let mut registry = Self {
            providers: HashMap::new(),
            client,
        };
        
        // Register default providers
        registry.register_ollama();
        registry.register_local_ai();
        
        registry
    }
    
    /// Register the Ollama provider
    fn register_ollama(&mut self) {
        let info = ProviderInfo {
            provider_type: ProviderType::Ollama,
            name: "Ollama".to_string(),
            description: "Ollama allows you to run open-source large language models locally.".to_string(),
            version: "0.0.0".to_string(), // Will be updated when checked
            default_endpoint: "http://localhost:11434".to_string(),
            supports_text_generation: true,
            supports_chat: true,
            supports_embeddings: false,
            requires_api_key: false,
            health_check_endpoint: Some("/api/version".to_string()),
            available: false,
        };
        
        self.providers.insert(ProviderType::Ollama, info);
    }
    
    /// Register the LocalAI provider
    fn register_local_ai(&mut self) {
        let info = ProviderInfo {
            provider_type: ProviderType::LocalAI,
            name: "LocalAI".to_string(),
            description: "LocalAI is a drop-in replacement for OpenAI, running models locally.".to_string(),
            version: "0.0.0".to_string(), // Will be updated when checked
            default_endpoint: "http://localhost:8080".to_string(),
            supports_text_generation: true,
            supports_chat: true,
            supports_embeddings: true,
            requires_api_key: false,
            health_check_endpoint: Some("/health".to_string()),
            available: false,
        };
        
        self.providers.insert(ProviderType::LocalAI, info);
    }
    
    /// Get provider information by type
    pub fn get_provider_info(&self, provider_type: &ProviderType) -> Option<&ProviderInfo> {
        self.providers.get(provider_type)
    }
    
    /// Get all registered providers
    pub fn get_all_providers(&self) -> Vec<&ProviderInfo> {
        self.providers.values().collect()
    }
    
    /// Get all available providers
    pub fn get_available_providers(&self) -> Vec<&ProviderInfo> {
        self.providers.values().filter(|p| p.available).collect()
    }
    
    /// Check if a provider is registered
    pub fn is_provider_registered(&self, provider_type: &ProviderType) -> bool {
        self.providers.contains_key(provider_type)
    }
    
    /// Check if a provider is available
    pub async fn check_provider_availability(&mut self, provider_type: &ProviderType, endpoint_url: Option<&str>) -> AvailabilityResult {
        let provider = match self.providers.get_mut(provider_type) {
            Some(provider) => provider,
            None => {
                return AvailabilityResult {
                    available: false,
                    version: None,
                    error: Some(format!("Provider type {:?} is not registered", provider_type)),
                    response_time_ms: None,
                };
            }
        };
        
        let base_url = endpoint_url.unwrap_or(&provider.default_endpoint);
        
        if let Some(health_endpoint) = &provider.health_check_endpoint {
            let url = format!("{}{}", base_url, health_endpoint);
            debug!("Checking availability of {} at {}", provider.name, url);
            
            let start_time = std::time::Instant::now();
            let result = match timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
                Ok(Ok(response)) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    
                    if response.status().is_success() {
                        // Try to extract version information
                        let mut version = None;
                        
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                // Check for version field based on provider type
                                match provider_type {
                                    ProviderType::Ollama => {
                                        if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                                            version = Some(v.to_string());
                                            provider.version = v.to_string();
                                        }
                                    },
                                    ProviderType::LocalAI => {
                                        if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                                            version = Some(v.to_string());
                                            provider.version = v.to_string();
                                        }
                                    },
                                    _ => {}
                                }
                            },
                            Err(e) => {
                                debug!("Failed to parse response from {}: {}", provider.name, e);
                            }
                        }
                        
                        // Update availability status
                        provider.available = true;
                        
                        AvailabilityResult {
                            available: true,
                            version,
                            error: None,
                            response_time_ms: Some(response_time),
                        }
                    } else {
                        // Service responded but returned an error
                        provider.available = false;
                        
                        AvailabilityResult {
                            available: false,
                            version: None,
                            error: Some(format!("Provider returned HTTP {}", response.status())),
                            response_time_ms: Some(response_time),
                        }
                    }
                },
                Ok(Err(e)) => {
                    // Connection error
                    provider.available = false;
                    
                    AvailabilityResult {
                        available: false,
                        version: None,
                        error: Some(format!("Connection error: {}", e)),
                        response_time_ms: None,
                    }
                },
                Err(_) => {
                    // Timeout
                    provider.available = false;
                    
                    AvailabilityResult {
                        available: false,
                        version: None,
                        error: Some("Connection timed out".to_string()),
                        response_time_ms: None,
                    }
                }
            };
            
            result
        } else {
            // No health check endpoint defined
            AvailabilityResult {
                available: false,
                version: None,
                error: Some("No health check endpoint defined for this provider".to_string()),
                response_time_ms: None,
            }
        }
    }
    
    /// Check availability of all registered providers
    pub async fn check_all_providers(&mut self) -> HashMap<ProviderType, AvailabilityResult> {
        let mut results = HashMap::new();
        
        for provider_type in self.providers.keys().cloned().collect::<Vec<_>>() {
            let result = self.check_provider_availability(&provider_type, None).await;
            results.insert(provider_type, result);
        }
        
        results
    }
}

/// Get a reference to the provider registry
pub async fn get_provider_registry() -> tokio::sync::RwLockReadGuard<'static, ProviderRegistry> {
    PROVIDER_REGISTRY.read().await
}

/// Get a mutable reference to the provider registry
pub async fn get_provider_registry_mut() -> tokio::sync::RwLockWriteGuard<'static, ProviderRegistry> {
    PROVIDER_REGISTRY.write().await
}

/// Check availability of all registered providers
pub async fn check_all_providers() -> HashMap<ProviderType, AvailabilityResult> {
    let mut registry = get_provider_registry_mut().await;
    registry.check_all_providers().await
}

/// Check if a specific provider is available
pub async fn is_provider_available(provider_type: ProviderType, endpoint_url: Option<&str>) -> bool {
    let mut registry = get_provider_registry_mut().await;
    registry.check_provider_availability(&provider_type, endpoint_url).await.available
}

/// Create a provider instance based on configuration
pub async fn create_provider(config: &ProviderConfig) -> LLMProviderResult<Box<dyn LocalLLMProvider>> {
    let provider_type = &config.provider_type;
    
    // Check if provider is registered
    let registry = get_provider_registry().await;
    if !registry.is_provider_registered(provider_type) {
        error!("Provider type {:?} is not registered", provider_type);
        return Err(LLMProviderError::ProviderError(
            format!("Provider type {:?} is not registered", provider_type)
        ));
    }
    drop(registry);
    
    // Check if provider is available
    let endpoint_url = Some(&config.endpoint_url);
    if !is_provider_available(config.provider_type, endpoint_url).await {
        let error_message = match config.provider_type {
            ProviderType::Ollama => format!(
                "Ollama is not available at {}. Make sure Ollama is installed and running.",
                config.endpoint_url
            ),
            ProviderType::LocalAI => format!(
                "LocalAI is not available at {}. Make sure LocalAI is installed and running.",
                config.endpoint_url
            ),
            ProviderType::LlamaCpp => format!(
                "llama.cpp is not available. Make sure it's properly set up.",
            ),
            ProviderType::Custom(ref name) => format!(
                "Custom provider '{}' is not available at {}.",
                name, config.endpoint_url
            ),
        };
        
        warn!("{}", error_message);
        return Err(LLMProviderError::ProviderError(error_message));
    }
    
    // Create provider based on type
    let mut provider: Box<dyn LocalLLMProvider> = match config.provider_type {
        ProviderType::Ollama => {
            debug!("Creating Ollama provider with endpoint {}", config.endpoint_url);
            Box::new(OllamaProvider::with_base_url(&config.endpoint_url))
        },
        ProviderType::LocalAI => {
            debug!("Creating LocalAI provider with endpoint {}", config.endpoint_url);
            Box::new(LocalAIProvider::with_base_url(&config.endpoint_url))
        },
        ProviderType::LlamaCpp => {
            debug!("llama.cpp provider not implemented yet, falling back to Ollama");
            warn!("llama.cpp provider requested but not implemented, falling back to Ollama");
            Box::new(OllamaProvider::with_base_url(&config.endpoint_url))
        },
        ProviderType::Custom(_) => {
            debug!("Custom provider not implemented yet, falling back to Ollama");
            warn!("Custom provider requested but not implemented, falling back to Ollama");
            Box::new(OllamaProvider::with_base_url(&config.endpoint_url))
        },
    };
    
    // Initialize the provider
    debug!("Initializing provider with config: {:?}", config);
    
    // Create initialization config
    let mut init_config = serde_json::Map::new();
    
    // Add endpoint URL
    init_config.insert("base_url".to_string(), serde_json::Value::String(config.endpoint_url.clone()));
    
    // Add API key if present
    if let Some(api_key) = &config.api_key {
        init_config.insert("api_key".to_string(), serde_json::Value::String(api_key.clone()));
    }
    
    // Add timeout settings
    init_config.insert(
        "request_timeout_seconds".to_string(), 
        serde_json::Value::Number(serde_json::Number::from(config.request_timeout_seconds))
    );
    
    // Add any custom options
    for (key, value) in &config.custom_options {
        init_config.insert(key.clone(), value.clone());
    }
    
    // Initialize provider
    match provider.initialize(serde_json::Value::Object(init_config)).await {
        Ok(_) => {
            info!("Successfully initialized {:?} provider", config.provider_type);
            Ok(provider)
        },
        Err(e) => {
            error!("Failed to initialize {:?} provider: {}", config.provider_type, e);
            Err(e)
        },
    }
}

/// Create a provider with default configuration
pub async fn create_default_provider() -> LLMProviderResult<Box<dyn LocalLLMProvider>> {
    // Check which providers are available
    let results = check_all_providers().await;
    
    // Try to find an available provider
    let available_provider = results.iter()
        .filter(|(_, result)| result.available)
        .map(|(provider_type, _)| provider_type)
        .next();
    
    match available_provider {
        Some(provider_type) => {
            let registry = get_provider_registry().await;
            let info = registry.get_provider_info(provider_type)
                .expect("Provider info should exist for available provider");
            
            // Create configuration for the available provider
            let config = ProviderConfig {
                provider_type: *provider_type,
                endpoint_url: info.default_endpoint.clone(),
                api_key: None,
                max_concurrent_requests: 5,
                request_timeout_seconds: 30,
                enable_retry: true,
                max_retries: 3,
                retry_backoff_seconds: 1.5,
                custom_options: HashMap::new(),
            };
            
            create_provider(&config).await
        },
        None => {
            // No providers available
            // Try Ollama as a default fallback
            warn!("No providers available, trying Ollama as fallback");
            let config = ProviderConfig {
                provider_type: ProviderType::Ollama,
                endpoint_url: "http://localhost:11434".to_string(),
                api_key: None,
                max_concurrent_requests: 5,
                request_timeout_seconds: 30,
                enable_retry: true,
                max_retries: 3,
                retry_backoff_seconds: 1.5,
                custom_options: HashMap::new(),
            };
            
            create_provider(&config).await
        },
    }
}

/// Get a list of all available providers
pub async fn get_available_providers() -> Vec<ProviderInfo> {
    let mut registry = get_provider_registry_mut().await;
    let results = registry.check_all_providers().await;
    
    // Filter for available providers and get their info
    registry.get_available_providers().cloned().collect()
}

/// Get information about all registered providers
pub async fn get_all_providers() -> Vec<ProviderInfo> {
    let registry = get_provider_registry().await;
    registry.get_all_providers().cloned().collect()
}

/// Initialize the factory and check all providers
pub async fn initialize() {
    info!("Initializing LLM provider factory");
    
    // Force initialization of the registry
    let _ = PROVIDER_REGISTRY.clone();
    
    // Check all providers
    let results = check_all_providers().await;
    
    // Log available providers
    let available_count = results.values().filter(|r| r.available).count();
    info!("Found {} available LLM providers", available_count);
    
    for (provider_type, result) in results {
        if result.available {
            info!(
                "Provider {:?} is available (version: {})",
                provider_type,
                result.version.unwrap_or_else(|| "unknown".to_string())
            );
        } else {
            if let Some(error) = result.error {
                debug!("Provider {:?} is not available: {}", provider_type, error);
            } else {
                debug!("Provider {:?} is not available", provider_type);
            }
        }
    }
}