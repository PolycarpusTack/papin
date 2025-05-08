use crate::ai::{get_all_providers, ModelError, ModelProvider, ModelProviderConfig, ModelStatus, ProviderType};
use crate::models::messages::{Message, MessageError};
use crate::models::Model;
use crate::utils::config;
use crate::utils::events::{events, get_event_system};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

/// Model router for switching between providers
pub struct ModelRouter {
    /// Available providers
    providers: RwLock<Vec<Arc<dyn ModelProvider>>>,
    
    /// Provider selection strategy
    strategy: RouterStrategy,
    
    /// Network status
    network_status: Arc<RwLock<NetworkStatus>>,
    
    /// Model routing rules
    routing_rules: Arc<RwLock<HashMap<String, RoutingRule>>>,
    
    /// Default provider
    default_provider: Arc<RwLock<Option<Arc<dyn ModelProvider>>>>,
    
    /// Fallback provider
    fallback_provider: Arc<RwLock<Option<Arc<dyn ModelProvider>>>>,
}

/// Network status for determining connection availability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Network is connected
    Connected,
    
    /// Network is disconnected
    Disconnected,
    
    /// Network connection is unstable
    Unstable,
    
    /// Network status is unknown
    Unknown,
}

/// Provider selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterStrategy {
    /// Prefer online providers (default)
    PreferOnline,
    
    /// Prefer local providers
    PreferLocal,
    
    /// Use only online providers
    OnlineOnly,
    
    /// Use only local providers
    LocalOnly,
    
    /// Round-robin between providers
    RoundRobin,
    
    /// Use specific routing rules
    RulesBased,
}

/// Routing rule for model selection
#[derive(Debug, Clone)]
pub struct RoutingRule {
    /// Model ID pattern to match
    pub model_pattern: String,
    
    /// Provider type to use
    pub provider_type: ProviderType,
    
    /// Fallback provider type
    pub fallback_provider_type: Option<ProviderType>,
    
    /// Fallback model ID
    pub fallback_model_id: Option<String>,
    
    /// Timeout before falling back
    pub timeout: Duration,
    
    /// Whether to fail if provider is unavailable
    pub fail_if_unavailable: bool,
}

impl ModelRouter {
    /// Create a new model router
    pub fn new() -> Self {
        // Get all available providers
        let providers = get_all_providers();
        
        // Set default and fallback providers
        let default_provider = providers
            .iter()
            .find(|p| p.provider_type() == ProviderType::Claude)
            .cloned();
        
        let fallback_provider = providers
            .iter()
            .find(|p| p.provider_type() == ProviderType::Local)
            .cloned();
        
        Self {
            providers: RwLock::new(providers),
            strategy: RouterStrategy::PreferOnline,
            network_status: Arc::new(RwLock::new(NetworkStatus::Unknown)),
            routing_rules: Arc::new(RwLock::new(HashMap::new())),
            default_provider: Arc::new(RwLock::new(default_provider)),
            fallback_provider: Arc::new(RwLock::new(fallback_provider)),
        }
    }
    
    /// Set router strategy
    pub fn set_strategy(&self, strategy: RouterStrategy) {
        self.strategy = strategy;
    }
    
    /// Set network status
    pub fn set_network_status(&self, status: NetworkStatus) {
        let mut network_status = self.network_status.write().unwrap();
        *network_status = status;
        
        // Notify status change
        get_event_system().emit(
            events::NETWORK_STATUS_CHANGED,
            serde_json::json!({
                "status": match status {
                    NetworkStatus::Connected => "connected",
                    NetworkStatus::Disconnected => "disconnected",
                    NetworkStatus::Unstable => "unstable",
                    NetworkStatus::Unknown => "unknown",
                }
            }),
        );
    }
    
    /// Add a routing rule
    pub fn add_routing_rule(&self, model_pattern: &str, rule: RoutingRule) {
        let mut rules = self.routing_rules.write().unwrap();
        rules.insert(model_pattern.to_string(), rule);
    }
    
    /// Remove a routing rule
    pub fn remove_routing_rule(&self, model_pattern: &str) {
        let mut rules = self.routing_rules.write().unwrap();
        rules.remove(model_pattern);
    }
    
    /// Get all available providers
    pub fn get_providers(&self) -> Vec<Arc<dyn ModelProvider>> {
        self.providers.read().unwrap().clone()
    }
    
    /// Get provider by type
    pub fn get_provider_by_type(&self, provider_type: ProviderType) -> Option<Arc<dyn ModelProvider>> {
        self.providers
            .read()
            .unwrap()
            .iter()
            .find(|p| p.provider_type() == provider_type)
            .cloned()
    }
    
    /// Check if network is available
    pub fn is_network_available(&self) -> bool {
        let status = self.network_status.read().unwrap();
        *status == NetworkStatus::Connected || *status == NetworkStatus::Unstable
    }
    
    /// Select provider for a model
    pub fn select_provider_for_model(&self, model_id: &str) -> Option<(Arc<dyn ModelProvider>, String)> {
        // Check routing rules first
        let rules = self.routing_rules.read().unwrap();
        for (pattern, rule) in rules.iter() {
            if model_id.starts_with(pattern) {
                // Find provider of the specified type
                if let Some(provider) = self.get_provider_by_type(rule.provider_type) {
                    return Some((provider, model_id.to_string()));
                }
                
                // If provider not found and fallback is specified, try fallback
                if let Some(fallback_type) = rule.fallback_provider_type {
                    if let Some(fallback_provider) = self.get_provider_by_type(fallback_type) {
                        // Use fallback model ID if specified, otherwise use original
                        let fallback_model_id = rule.fallback_model_id.clone().unwrap_or_else(|| model_id.to_string());
                        return Some((fallback_provider, fallback_model_id));
                    }
                }
                
                // No provider found, return None
                return None;
            }
        }
        
        // If no rule matched, use strategy-based selection
        match self.strategy {
            RouterStrategy::PreferOnline => {
                // Check if network is available
                if self.is_network_available() {
                    // Try to find a cloud provider that supports this model
                    for provider in self.providers.read().unwrap().iter() {
                        if provider.provider_type() != ProviderType::Local {
                            return Some((provider.clone(), model_id.to_string()));
                        }
                    }
                }
                
                // Fallback to local provider
                if let Some(fallback) = self.fallback_provider.read().unwrap().clone() {
                    return Some((fallback, model_id.to_string()));
                }
            }
            RouterStrategy::PreferLocal => {
                // Try to find a local provider first
                for provider in self.providers.read().unwrap().iter() {
                    if provider.provider_type() == ProviderType::Local {
                        return Some((provider.clone(), model_id.to_string()));
                    }
                }
                
                // Fallback to cloud provider if network is available
                if self.is_network_available() {
                    if let Some(default) = self.default_provider.read().unwrap().clone() {
                        return Some((default, model_id.to_string()));
                    }
                }
            }
            RouterStrategy::OnlineOnly => {
                // Only use cloud providers
                if self.is_network_available() {
                    for provider in self.providers.read().unwrap().iter() {
                        if provider.provider_type() != ProviderType::Local {
                            return Some((provider.clone(), model_id.to_string()));
                        }
                    }
                }
            }
            RouterStrategy::LocalOnly => {
                // Only use local providers
                for provider in self.providers.read().unwrap().iter() {
                    if provider.provider_type() == ProviderType::Local {
                        return Some((provider.clone(), model_id.to_string()));
                    }
                }
            }
            RouterStrategy::RoundRobin => {
                // Simple round-robin: just use the default provider for now
                // In a real implementation, this would rotate between providers
                if let Some(default) = self.default_provider.read().unwrap().clone() {
                    return Some((default, model_id.to_string()));
                }
            }
            RouterStrategy::RulesBased => {
                // We already checked rules, so this is a fallback
                if let Some(default) = self.default_provider.read().unwrap().clone() {
                    return Some((default, model_id.to_string()));
                }
            }
        }
        
        // No suitable provider found
        None
    }
    
    /// Get available models from all providers
    pub async fn get_available_models(&self) -> Vec<Model> {
        let mut models = Vec::new();
        
        for provider in self.providers.read().unwrap().iter() {
            match provider.available_models().await {
                Ok(provider_models) => {
                    models.extend(provider_models);
                }
                Err(e) => {
                    warn!("Failed to get models from provider {}: {:?}", provider.name(), e);
                }
            }
        }
        
        models
    }
    
    /// Complete a message with the appropriate model
    pub async fn complete(&self, model_id: &str, message: Message) -> Result<Message, MessageError> {
        // Select provider
        let (provider, final_model_id) = self
            .select_provider_for_model(model_id)
            .ok_or_else(|| MessageError::ProtocolError(format!("No provider found for model {}", model_id)))?;
        
        // Complete with selected provider
        provider.complete(&final_model_id, message).await
    }
    
    /// Stream a message with the appropriate model
    pub async fn stream(
        &self,
        model_id: &str,
        message: Message,
    ) -> Result<mpsc::Receiver<Result<Message, MessageError>>, MessageError> {
        // Select provider
        let (provider, final_model_id) = self
            .select_provider_for_model(model_id)
            .ok_or_else(|| MessageError::ProtocolError(format!("No provider found for model {}", model_id)))?;
        
        // Stream with selected provider
        provider.stream(&final_model_id, message).await
    }
    
    /// Cancel a streaming message
    pub async fn cancel_stream(&self, stream_id: &str) -> Result<(), MessageError> {
        // Try cancelling with all providers
        // In a real implementation, we would track which provider is handling which stream
        for provider in self.providers.read().unwrap().iter() {
            let _ = provider.cancel_stream(stream_id).await;
        }
        
        Ok(())
    }
}

/// Global model router instance
static MODEL_ROUTER: once_cell::sync::OnceCell<ModelRouter> = once_cell::sync::OnceCell::new();

/// Get the global model router instance
pub fn get_model_router() -> &'static ModelRouter {
    MODEL_ROUTER.get_or_init(|| ModelRouter::new())
}
