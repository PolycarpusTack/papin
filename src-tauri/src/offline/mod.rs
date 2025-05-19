//! Offline capabilities module
//!
//! This module provides functionality for offline operation of the MCP client,
//! including local LLM inference, checkpointing, and synchronization mechanisms.

pub mod llm;

use std::sync::Arc;
use std::collections::HashMap;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

use self::llm::{
    LLMManager, create_llm_manager, LLMConfig, LLMError, LLMResult,
    types::{ProviderType, ProviderConfig, GenerationOptions, GenerationRequest},
    factory::{ProviderInfo, AvailabilityResult},
};

/// Errors that can occur in the offline module
#[derive(Error, Debug)]
pub enum OfflineError {
    /// LLM-related error
    #[error("LLM error: {0}")]
    LLMError(#[from] llm::LLMError),
    
    /// Network connectivity error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Checkpoint-related error
    #[error("Checkpoint error: {0}")]
    CheckpointError(String),
    
    /// Synchronization error
    #[error("Sync error: {0}")]
    SyncError(String),
    
    /// Provider-related error
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Result type for offline operations
pub type OfflineResult<T> = Result<T, OfflineError>;

/// Configuration for the offline module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Whether offline mode is enabled
    pub enabled: bool,
    /// Whether to automatically switch to offline mode when network is unavailable
    pub auto_switch: bool,
    /// Configuration for the LLM module
    pub llm_config: LLMProviderConfig,
    /// Maximum size of conversation history to keep in offline mode
    pub max_history_size: usize,
    /// Whether to enable debug logging
    pub enable_debug: bool,
}

/// Configuration for the LLM provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMProviderConfig {
    /// Type of provider to use
    pub provider_type: ProviderType,
    /// Provider endpoint URL
    pub endpoint_url: String,
    /// API key for providers that require authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Default model to use for generation
    pub default_model: Option<String>,
    /// Whether to enable advanced configuration options
    #[serde(default)]
    pub enable_advanced_config: bool,
    /// Advanced provider configuration
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub advanced_config: HashMap<String, serde_json::Value>,
}

impl Default for LLMProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Ollama,
            endpoint_url: "http://localhost:11434".to_string(),
            api_key: None,
            default_model: None,
            enable_advanced_config: false,
            advanced_config: HashMap::new(),
        }
    }
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_switch: true,
            llm_config: LLMProviderConfig::default(),
            max_history_size: 100,
            enable_debug: false,
        }
    }
}

/// Network connectivity status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Network is connected
    Connected,
    /// Network is disconnected
    Disconnected,
    /// Network status is unknown
    Unknown,
}

/// Offline manager that handles offline capabilities
pub struct OfflineManager {
    /// Configuration for the offline module
    config: OfflineConfig,
    /// Current network status
    network_status: NetworkStatus,
    /// LLM manager for local inference
    llm_manager: Arc<Mutex<LLMManager>>,
    /// Last provider availability check results
    provider_availability: HashMap<ProviderType, AvailabilityResult>,
}

impl OfflineManager {
    /// Create a new offline manager with the default configuration
    pub async fn new() -> Self {
        // Initialize the LLM factory
        llm::factory::initialize().await;
        
        // Start with default configuration
        let config = OfflineConfig::default();
        let llm_config = Self::convert_to_llm_config(&config.llm_config);
        
        Self {
            config,
            network_status: NetworkStatus::Unknown,
            llm_manager: create_llm_manager().await,
            provider_availability: HashMap::new(),
        }
    }
    
    /// Create a new offline manager with a specific configuration
    pub async fn with_config(config: OfflineConfig) -> Self {
        // Initialize the LLM factory
        llm::factory::initialize().await;
        
        let mut manager = Self {
            config,
            network_status: NetworkStatus::Unknown,
            llm_manager: create_llm_manager().await,
            provider_availability: HashMap::new(),
        };
        
        // Initialize the LLM manager with the config
        if let Err(e) = manager.initialize_llm().await {
            error!("Failed to initialize LLM manager: {}", e);
        }
        
        manager
    }
    
    /// Convert LLMProviderConfig to LLMConfig
    fn convert_to_llm_config(provider_config: &LLMProviderConfig) -> LLMConfig {
        // Create a JSON object for the provider config
        let mut config_obj = serde_json::Map::new();
        
        // Add base URL
        config_obj.insert("base_url".to_string(), serde_json::Value::String(provider_config.endpoint_url.clone()));
        
        // Add API key if present
        if let Some(api_key) = &provider_config.api_key {
            config_obj.insert("api_key".to_string(), serde_json::Value::String(api_key.clone()));
        }
        
        // Add any advanced configuration options
        if provider_config.enable_advanced_config {
            for (key, value) in &provider_config.advanced_config {
                config_obj.insert(key.clone(), value.clone());
            }
        }
        
        LLMConfig {
            provider_type: provider_config.provider_type,
            provider_config: serde_json::Value::Object(config_obj),
            default_model: provider_config.default_model.clone(),
            enable_debug: false,
        }
    }
    
    /// Convert LLMProviderConfig to ProviderConfig
    fn convert_to_provider_config(provider_config: &LLMProviderConfig) -> ProviderConfig {
        let mut custom_options = HashMap::new();
        
        if provider_config.enable_advanced_config {
            for (key, value) in &provider_config.advanced_config {
                custom_options.insert(key.clone(), value.clone());
            }
        }
        
        ProviderConfig {
            provider_type: provider_config.provider_type,
            endpoint_url: provider_config.endpoint_url.clone(),
            api_key: provider_config.api_key.clone(),
            max_concurrent_requests: 5,  // Default value
            request_timeout_seconds: 30, // Default value
            enable_retry: true,          // Default value
            max_retries: 3,              // Default value
            retry_backoff_seconds: 1.5,  // Default value
            custom_options,
        }
    }
    
    /// Initialize the LLM manager
    async fn initialize_llm(&mut self) -> OfflineResult<()> {
        info!("Initializing LLM manager with provider {:?}", self.config.llm_config.provider_type);
        
        // Check provider availability first
        self.check_provider_availability().await?;
        
        // Convert the provider config to LLMConfig
        let llm_config = Self::convert_to_llm_config(&self.config.llm_config);
        
        // Initialize the LLM manager
        let mut llm_manager = self.llm_manager.lock().await;
        *llm_manager = LLMManager::with_config(llm_config);
        
        match llm_manager.initialize().await {
            Ok(()) => {
                info!("Successfully initialized LLM manager");
                Ok(())
            },
            Err(e) => {
                // Try to use default provider as fallback
                error!("Failed to initialize LLM manager: {}. Trying default provider...", e);
                
                // Get available providers
                match llm::get_available_providers().await {
                    providers if !providers.is_empty() => {
                        let available_provider = &providers[0];
                        warn!(
                            "Using available provider {:?} as fallback",
                            available_provider.provider_type
                        );
                        
                        // Update the configuration
                        self.config.llm_config.provider_type = available_provider.provider_type;
                        self.config.llm_config.endpoint_url = available_provider.default_endpoint.clone();
                        
                        // Create new config
                        let fallback_config = Self::convert_to_llm_config(&self.config.llm_config);
                        
                        // Reinitialize
                        *llm_manager = LLMManager::with_config(fallback_config);
                        match llm_manager.initialize().await {
                            Ok(()) => {
                                info!("Successfully initialized LLM manager with fallback provider");
                                Ok(())
                            },
                            Err(e) => {
                                error!("Failed to initialize LLM manager with fallback provider: {}", e);
                                Err(OfflineError::LLMError(e))
                            },
                        }
                    },
                    _ => {
                        error!("No LLM providers available");
                        Err(OfflineError::ProviderError("No LLM providers available".to_string()))
                    },
                }
            },
        }
    }
    
    /// Check the availability of the current provider
    pub async fn check_provider_availability(&mut self) -> OfflineResult<bool> {
        let provider_type = self.config.llm_config.provider_type;
        let endpoint_url = Some(self.config.llm_config.endpoint_url.as_str());
        
        debug!("Checking availability of provider {:?}", provider_type);
        
        // Use the factory to check availability
        let is_available = llm::is_provider_available(provider_type, endpoint_url).await;
        
        if is_available {
            debug!("Provider {:?} is available", provider_type);
        } else {
            warn!("Provider {:?} is not available", provider_type);
        }
        
        // Get detailed availability results
        let mut registry = llm::factory::get_provider_registry_mut().await;
        let result = registry.check_provider_availability(&provider_type, endpoint_url).await;
        self.provider_availability.insert(provider_type, result.clone());
        
        Ok(is_available)
    }
    
    /// Get availability status for all providers
    pub async fn get_all_provider_availability(&mut self) -> OfflineResult<HashMap<ProviderType, AvailabilityResult>> {
        debug!("Checking availability of all providers");
        
        // Use the factory to check all providers
        let results = llm::factory::check_all_providers().await;
        
        // Store results
        self.provider_availability = results.clone();
        
        Ok(results)
    }
    
    /// Get a list of all available providers
    pub async fn get_available_providers(&mut self) -> OfflineResult<Vec<ProviderInfo>> {
        debug!("Getting list of available providers");
        
        // Use the factory to get available providers
        let providers = llm::get_available_providers().await;
        
        Ok(providers)
    }
    
    /// Get a list of all registered providers
    pub async fn get_all_providers(&self) -> OfflineResult<Vec<ProviderInfo>> {
        debug!("Getting list of all registered providers");
        
        // Use the factory to get all providers
        let providers = llm::factory::get_all_providers().await;
        
        Ok(providers)
    }
    
    /// Change the provider type and reconfigure
    pub async fn change_provider(&mut self, provider_type: ProviderType, endpoint_url: &str) -> OfflineResult<()> {
        debug!("Changing provider to {:?} at {}", provider_type, endpoint_url);
        
        // Check if the provider is available
        let is_available = llm::is_provider_available(provider_type, Some(endpoint_url)).await;
        
        if !is_available {
            warn!("Provider {:?} is not available at {}", provider_type, endpoint_url);
            return Err(OfflineError::ProviderError(format!(
                "Provider {:?} is not available at {}",
                provider_type, endpoint_url
            )));
        }
        
        // Update the configuration
        self.config.llm_config.provider_type = provider_type;
        self.config.llm_config.endpoint_url = endpoint_url.to_string();
        
        // Reinitialize the LLM manager
        self.initialize_llm().await
    }
    
    /// Get the current provider configuration
    pub fn get_provider_config(&self) -> LLMProviderConfig {
        self.config.llm_config.clone()
    }
    
    /// Update the provider configuration
    pub async fn update_provider_config(&mut self, config: LLMProviderConfig) -> OfflineResult<()> {
        debug!("Updating provider configuration: {:?}", config);
        
        // Check if the configuration changed
        if self.config.llm_config == config {
            debug!("Provider configuration unchanged");
            return Ok(());
        }
        
        // Update the configuration
        self.config.llm_config = config;
        
        // Reinitialize the LLM manager
        self.initialize_llm().await
    }
    
    /// Check if offline mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Enable or disable offline mode
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }
    
    /// Get the current network status
    pub fn get_network_status(&self) -> NetworkStatus {
        self.network_status
    }
    
    /// Update the network status
    pub fn update_network_status(&mut self, status: NetworkStatus) {
        self.network_status = status;
        
        // If auto-switch is enabled, update offline mode based on network status
        if self.config.auto_switch {
            match status {
                NetworkStatus::Connected => self.config.enabled = false,
                NetworkStatus::Disconnected => self.config.enabled = true,
                NetworkStatus::Unknown => {}, // Don't change anything
            }
        }
    }
    
    /// Update the configuration
    pub async fn update_config(&mut self, config: OfflineConfig) -> OfflineResult<()> {
        debug!("Updating offline configuration");
        
        // Check if LLM config changed
        let llm_config_changed = self.config.llm_config != config.llm_config;
        
        // Update the config
        self.config = config;
        
        // Reinitialize the LLM manager if needed
        if llm_config_changed {
            self.initialize_llm().await?;
        }
        
        Ok(())
    }
    
    /// Get a reference to the LLM manager
    pub fn llm_manager(&self) -> &Arc<Mutex<LLMManager>> {
        &self.llm_manager
    }
    
    /// Check if network is available
    pub async fn check_network(&mut self) -> OfflineResult<NetworkStatus> {
        debug!("Checking network connectivity");
        
        // Try to make a simple HTTP request to check connectivity
        let client = reqwest::Client::new();
        
        match client.get("https://anthropic.com")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await 
        {
            Ok(_) => {
                debug!("Network is connected");
                self.update_network_status(NetworkStatus::Connected);
                Ok(NetworkStatus::Connected)
            },
            Err(e) => {
                debug!("Network is disconnected: {}", e);
                self.update_network_status(NetworkStatus::Disconnected);
                Ok(NetworkStatus::Disconnected)
            },
        }
    }
    
    /// Generate text using a local LLM (backward compatible)
    pub async fn generate_text(
        &self,
        prompt: &str,
        model_id: Option<&str>,
    ) -> OfflineResult<String> {
        debug!("Generating text with prompt: {:?}", prompt);
        let llm_manager = self.llm_manager.lock().await;
        
        let response = llm_manager.generate_text(model_id, prompt, None).await?;
        
        Ok(response.text)
    }
    
    /// Generate text with options
    pub async fn generate_text_with_options(
        &self,
        prompt: &str,
        model_id: Option<&str>,
        options: GenerationOptions,
    ) -> OfflineResult<String> {
        debug!("Generating text with options");
        let llm_manager = self.llm_manager.lock().await;
        
        let response = llm_manager.generate_text(model_id, prompt, Some(options)).await?;
        
        Ok(response.text)
    }
    
    /// Process a generation request
    pub async fn process_request(&self, request: GenerationRequest) -> OfflineResult<String> {
        debug!("Processing generation request");
        let llm_manager = self.llm_manager.lock().await;
        
        let response = llm_manager.process_request(request).await?;
        
        Ok(response.text)
    }
    
    /// List available models from the current provider
    pub async fn list_available_models(&self) -> OfflineResult<Vec<llm::provider::ModelInfo>> {
        debug!("Listing available models");
        let llm_manager = self.llm_manager.lock().await;
        
        match llm_manager.list_available_models().await {
            Ok(models) => Ok(models),
            Err(e) => Err(OfflineError::LLMError(e)),
        }
    }
    
    /// List downloaded models from the current provider
    pub async fn list_downloaded_models(&self) -> OfflineResult<Vec<llm::provider::ModelInfo>> {
        debug!("Listing downloaded models");
        let llm_manager = self.llm_manager.lock().await;
        
        match llm_manager.list_downloaded_models().await {
            Ok(models) => Ok(models),
            Err(e) => Err(OfflineError::LLMError(e)),
        }
    }
    
    /// Get model info
    pub async fn get_model_info(&self, model_id: &str) -> OfflineResult<llm::provider::ModelInfo> {
        debug!("Getting model info for {}", model_id);
        let llm_manager = self.llm_manager.lock().await;
        
        match llm_manager.get_model_info(model_id).await {
            Ok(info) => Ok(info),
            Err(e) => Err(OfflineError::LLMError(e)),
        }
    }
    
    /// Download a model
    pub async fn download_model(&self, model_id: &str) -> OfflineResult<()> {
        debug!("Downloading model {}", model_id);
        let llm_manager = self.llm_manager.lock().await;
        
        match llm_manager.download_model(model_id).await {
            Ok(()) => Ok(()),
            Err(e) => Err(OfflineError::LLMError(e)),
        }
    }
}

/// Create a shared offline manager with the default configuration
pub async fn create_offline_manager() -> Arc<Mutex<OfflineManager>> {
    Arc::new(Mutex::new(OfflineManager::new().await))
}