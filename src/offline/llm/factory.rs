// src/offline/llm/factory.rs
//! Factory for creating LLM providers

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::offline::llm::provider::{Provider, ProviderError, ProviderType, Result};
use crate::offline::llm::providers::ollama::{OllamaProvider, OllamaConfig};
use crate::offline::llm::providers::localai::{LocalAIProvider, LocalAIConfig};

/// Factory singleton for creating and managing LLM providers
pub struct ProviderFactory {
    /// Available providers map
    providers: Mutex<HashMap<ProviderType, Arc<dyn Provider>>>,
    /// Default provider type
    default_provider: Mutex<ProviderType>,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new() -> Self {
        Self {
            providers: Mutex::new(HashMap::new()),
            default_provider: Mutex::new(ProviderType::Ollama),
        }
    }
    
    /// Register a provider with the factory
    pub fn register_provider(&self, provider: Arc<dyn Provider>) {
        let provider_type = provider.get_type();
        
        let mut providers = self.providers.lock().unwrap();
        providers.insert(provider_type, provider);
        
        info!("Registered provider: {}", provider_type);
    }
    
    /// Create and register all known provider types
    pub fn register_default_providers(&self) -> Result<()> {
        // Register Ollama provider
        let ollama_result = OllamaProvider::new();
        if let Ok(provider) = ollama_result {
            self.register_provider(Arc::new(provider));
        } else {
            warn!("Failed to create Ollama provider: {:?}", ollama_result.err());
        }
        
        // Register LocalAI provider
        let localai_result = LocalAIProvider::new();
        if let Ok(provider) = localai_result {
            self.register_provider(Arc::new(provider));
        } else {
            warn!("Failed to create LocalAI provider: {:?}", localai_result.err());
        }
        
        // Set default provider
        self.set_default_provider(ProviderType::Ollama);
        
        Ok(())
    }
    
    /// Get a provider by type
    pub fn get_provider(&self, provider_type: &ProviderType) -> Result<Arc<dyn Provider>> {
        let providers = self.providers.lock().unwrap();
        
        providers.get(provider_type)
            .cloned()
            .ok_or_else(|| ProviderError::ConfigurationError(format!(
                "Provider '{}' not registered", provider_type
            )))
    }
    
    /// Get the default provider
    pub fn get_default_provider(&self) -> Result<Arc<dyn Provider>> {
        let default_type = self.default_provider.lock().unwrap().clone();
        self.get_provider(&default_type)
    }
    
    /// Set the default provider type
    pub fn set_default_provider(&self, provider_type: ProviderType) {
        let mut default = self.default_provider.lock().unwrap();
        *default = provider_type;
    }
    
    /// Create a new provider with custom configuration
    pub fn create_provider(&self, provider_type: ProviderType, config: ProviderConfig) -> Result<Arc<dyn Provider>> {
        let provider: Arc<dyn Provider> = match provider_type {
            ProviderType::Ollama => {
                if let Some(ollama_config) = config.ollama {
                    Arc::new(OllamaProvider::with_config(ollama_config)?)
                } else {
                    Arc::new(OllamaProvider::new()?)
                }
            },
            ProviderType::LocalAI => {
                if let Some(localai_config) = config.localai {
                    Arc::new(LocalAIProvider::with_config(localai_config)?)
                } else {
                    Arc::new(LocalAIProvider::new()?)
                }
            },
            _ => return Err(ProviderError::ConfigurationError(format!(
                "Provider type '{}' not supported for custom configuration", provider_type
            ))),
        };
        
        Ok(provider)
    }
    
    /// Check if a provider is available (can connect)
    pub async fn is_provider_available(&self, provider_type: &ProviderType) -> bool {
        match self.get_provider(provider_type) {
            Ok(provider) => {
                match provider.is_available().await {
                    Ok(available) => available,
                    Err(_) => false,
                }
            },
            Err(_) => false,
        }
    }
    
    /// Get a list of all registered provider types
    pub fn get_registered_providers(&self) -> Vec<ProviderType> {
        let providers = self.providers.lock().unwrap();
        providers.keys().cloned().collect()
    }
    
    /// Get a list of all available (working) providers
    pub async fn get_available_providers(&self) -> Vec<ProviderType> {
        let provider_types = self.get_registered_providers();
        let mut available = Vec::new();
        
        for provider_type in provider_types {
            if self.is_provider_available(&provider_type).await {
                available.push(provider_type);
            }
        }
        
        available
    }
}

// Create a global factory singleton
lazy_static::lazy_static! {
    static ref PROVIDER_FACTORY: ProviderFactory = {
        let factory = ProviderFactory::new();
        // Don't register providers here, as this is called during static initialization
        // We'll register them when first accessed
        factory
    };
}

/// Get the global provider factory
pub fn get_provider_factory() -> &'static ProviderFactory {
    &PROVIDER_FACTORY
}

/// Unified configuration for all provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Configuration for Ollama provider
    pub ollama: Option<OllamaConfig>,
    /// Configuration for LocalAI provider
    pub localai: Option<LocalAIConfig>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            ollama: Some(OllamaConfig::default()),
            localai: Some(LocalAIConfig::default()),
        }
    }
}

/// Initialize the provider factory
pub async fn initialize() -> Result<()> {
    // Register default providers
    let factory = get_provider_factory();
    factory.register_default_providers()?;
    
    // Check which providers are available
    let available = factory.get_available_providers().await;
    
    if available.is_empty() {
        warn!("No LLM providers are available. Offline LLM functionality will be limited.");
    } else {
        info!("Available LLM providers: {:?}", available);
        
        // Set the first available provider as default
        if !available.is_empty() {
            factory.set_default_provider(available[0].clone());
        }
    }
    
    Ok(())
}