// src/offline/llm/mod.rs
//! Local LLM support with cross-platform optimizations and integrated model management

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;
use log::{info, warn, error, debug, trace};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};
use std::fs;

// LLM discovery subsystem
pub mod discovery;

// LLM migration subsystem
pub mod migration;

// LLM platform-specific operations
pub mod platform;

// Provider interface and types
pub mod provider;

// Provider implementations
pub mod providers;

// Provider factory
pub mod factory;

// Model management
pub mod models;

use crate::platform::fs::{platform_fs, PlatformFsError, PathExt};
use self::platform::{platform_llm_manager, CpuArchitecture, GpuType, AccelerationInfo};
use self::provider::{Provider, ProviderType, ModelInfo, GenerationOptions, ProviderError, Result, DownloadStatus};
use self::factory::{get_provider_factory, ProviderConfig};
use self::models::{
    get_model_registry, initialize_model_registry, ModelRegistry, LLMModel, ModelRegistryEvent,
    ModelFormat, ModelArchitecture, QuantizationType, ModelCapabilities, DownloadProgress
};

/// Configuration for a local LLM manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Provider type to use
    pub provider_type: ProviderType,
    /// Provider-specific configuration as JSON
    pub provider_config: serde_json::Value,
    /// Default model to use
    pub default_model: Option<String>,
    /// Whether to automatically discover providers
    pub auto_discover: bool,
    /// Whether to monitor for new models
    pub monitor_for_models: bool,
    /// Enable platform-specific optimizations
    pub enable_optimizations: bool,
    /// Maximum disk space for models in bytes (0 for unlimited)
    pub max_disk_space: u64,
    /// Path to model storage directory (if not the default)
    pub model_storage_path: Option<PathBuf>,
    /// Enable model versioning
    pub enable_model_versioning: bool,
    /// Keep download cache for model updates
    pub keep_download_cache: bool,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Ollama,
            provider_config: serde_json::Value::Object(serde_json::Map::new()),
            default_model: None,
            auto_discover: true,
            monitor_for_models: true,
            enable_optimizations: true,
            max_disk_space: 0, // No limit by default
            model_storage_path: None,
            enable_model_versioning: true,
            keep_download_cache: true,
        }
    }
}

/// Manager for local LLM operations
pub struct LLMManager {
    /// Configuration for the LLM manager
    config: LLMConfig,
    /// Active provider
    provider: TokioMutex<Option<Arc<dyn Provider>>>,
    /// Local acceleration info
    acceleration_info: AccelerationInfo,
    /// Model registry
    model_registry: Arc<ModelRegistry>,
    /// Model version tracking
    model_versions: Mutex<HashMap<String, Vec<String>>>,
}

impl LLMManager {
    /// Create a new LLM manager with default configuration
    pub fn new() -> Self {
        Self::with_config(LLMConfig::default())
    }
    
    /// Create a new LLM manager with the specified configuration
    pub fn with_config(config: LLMConfig) -> Self {
        // Get platform-specific manager
        let platform_manager = platform_llm_manager();
        let acceleration_info = platform_manager.get_acceleration_info().clone();
        
        // Get model registry
        let model_registry = get_model_registry();
        
        Self {
            config,
            provider: TokioMutex::new(None),
            acceleration_info,
            model_registry,
            model_versions: Mutex::new(HashMap::new()),
        }
    }
    
    /// Initialize the manager and load the provider
    pub async fn initialize(&self) -> Result<()> {
        // Initialize model registry with custom path if configured
        if let Some(storage_path) = &self.config.model_storage_path {
            trace!("Setting custom model storage path: {}", storage_path.display());
            // We have to clone the registry to modify it
            if let Ok(registry_as_ref) = Arc::get_mut(&mut self.model_registry.clone()) {
                if let Err(e) = registry_as_ref.set_base_directory(storage_path.clone()) {
                    warn!("Failed to set model storage path: {}", e);
                }
            }
        }
        
        // Set disk space limit if configured
        if self.config.max_disk_space > 0 {
            trace!("Setting disk space limit: {} bytes", self.config.max_disk_space);
            if let Ok(registry_as_ref) = Arc::get_mut(&mut self.model_registry.clone()) {
                registry_as_ref.set_disk_space_limit(self.config.max_disk_space);
            }
        }
        
        // Initialize model registry
        if let Err(e) = initialize_model_registry() {
            warn!("Failed to initialize model registry: {}", e);
            // Continue anyway, as we can still function without the registry
        } else {
            info!("Model registry initialized successfully");
            trace!("Model registry contains {} models", self.model_registry.get_all_models().len());
        }
        
        // Initialize model version tracking
        self.initialize_model_versions();
        
        let factory = get_provider_factory();
        
        // Initialize factory if needed
        if factory.get_registered_providers().is_empty() {
            factory::initialize().await?;
        }
        
        // Get provider from factory
        let provider_result = factory.get_provider(&self.config.provider_type);
        
        match provider_result {
            Ok(provider) => {
                // Check if provider is available
                if provider.is_available().await? {
                    // Set as active provider
                    let mut active_provider = self.provider.lock().await;
                    *active_provider = Some(provider.clone());
                    
                    // Sync models from provider to registry
                    self.sync_provider_models(provider.clone()).await?;
                    
                    info!("Initialized LLM manager with provider: {}", self.config.provider_type);
                    Ok(())
                } else {
                    // Provider is not available, try to auto-discover
                    if self.config.auto_discover {
                        warn!("Provider {} is not available, trying auto-discovery", self.config.provider_type);
                        self.auto_discover_provider().await
                    } else {
                        Err(ProviderError::NotAvailable(format!(
                            "Provider {} is not available", self.config.provider_type
                        )))
                    }
                }
            },
            Err(e) => {
                // Provider not found, try to auto-discover
                if self.config.auto_discover {
                    warn!("Provider {} not found: {}, trying auto-discovery", self.config.provider_type, e);
                    self.auto_discover_provider().await
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Initialize model version tracking from registry
    fn initialize_model_versions(&self) {
        if !self.config.enable_model_versioning {
            return;
        }
        
        let models = self.model_registry.get_all_models();
        let mut version_map = self.model_versions.lock().unwrap();
        
        for model in models {
            let versions: Vec<String> = model.versions
                .iter()
                .map(|v| v.version.clone())
                .collect();
                
            if !versions.is_empty() {
                version_map.insert(model.id.clone(), versions);
                trace!("Loaded version history for model '{}': {:?}", model.id, versions);
            }
        }
        
        trace!("Initialized version tracking for {} models", version_map.len());
    }
    
    /// Sync models from provider to registry
    async fn sync_provider_models(&self, provider: Arc<dyn Provider>) -> Result<()> {
        debug!("Syncing models from provider to registry");
        // Get models from provider
        let provider_models = provider.list_available_models().await?;
        
        let provider_type = provider.get_type();
        trace!("Found {} models from provider {}", provider_models.len(), provider_type);
        
        // Add to registry
        for provider_model in provider_models {
            let mut model = LLMModel::from_model_info(&provider_model);
            
            // Set suggested provider
            model.suggested_provider = Some(provider_type.clone());
            
            // Check if model is already in registry
            let update_existing = if let Ok(existing) = self.model_registry.get_model(&model.id) {
                // Update existing model with new info but preserve path, installed status, etc.
                if existing.installed {
                    model.installed = true;
                    model.path = existing.path.clone();
                    model.installed_date = existing.installed_date;
                    model.last_used = existing.last_used;
                }
                
                // Preserve version history
                if !existing.versions.is_empty() {
                    model.versions = existing.versions.clone();
                }
                
                true
            } else {
                false
            };
            
            // Add or update the model in registry
            if let Err(e) = self.model_registry.add_model(model) {
                warn!("Failed to add model '{}' to registry: {}", provider_model.id, e);
            } else if update_existing {
                trace!("Updated existing model '{}' in registry", provider_model.id);
            } else {
                trace!("Added new model '{}' to registry", provider_model.id);
            }
        }
        
        debug!("Completed syncing models to registry");
        Ok(())
    }
    
    /// Automatically discover and set an available provider
    async fn auto_discover_provider(&self) -> Result<()> {
        info!("Auto-discovering available LLM providers");
        let factory = get_provider_factory();
        let available = factory.get_available_providers().await;
        
        if available.is_empty() {
            return Err(ProviderError::NotAvailable(
                "No LLM providers available".to_string()
            ));
        }
        
        trace!("Found {} available providers: {:?}", available.len(), available);
        
        // Use the first available provider
        let provider = factory.get_provider(&available[0])?;
        
        // Set as active provider
        let mut active_provider = self.provider.lock().await;
        *active_provider = Some(provider.clone());
        
        // Sync models from provider to registry
        self.sync_provider_models(provider.clone()).await?;
        
        info!("Auto-discovered provider: {}", available[0]);
        Ok(())
    }
    
    /// Get the active provider
    pub async fn get_provider(&self) -> Result<Arc<dyn Provider>> {
        let provider = self.provider.lock().await;
        
        provider.clone().ok_or_else(|| ProviderError::ConfigurationError(
            "No active provider. Call initialize() first.".to_string()
        ))
    }
    
    /// Set the provider type
    pub async fn set_provider(&self, provider_type: ProviderType) -> Result<()> {
        info!("Setting active provider to: {}", provider_type);
        let factory = get_provider_factory();
        let provider = factory.get_provider(&provider_type)?;
        
        // Check if provider is available
        if provider.is_available().await? {
            // Set as active provider
            let mut active_provider = self.provider.lock().await;
            *active_provider = Some(provider.clone());
            
            // Sync models from provider to registry
            self.sync_provider_models(provider.clone()).await?;
            
            info!("Changed provider to: {}", provider_type);
            Ok(())
        } else {
            Err(ProviderError::NotAvailable(format!(
                "Provider {} is not available", provider_type
            )))
        }
    }
    
    /// Get available provider types
    pub async fn get_available_providers(&self) -> Result<Vec<ProviderType>> {
        let factory = get_provider_factory();
        Ok(factory.get_available_providers().await)
    }
    
    /// List available models from both registry and provider
    pub async fn list_available_models(&self) -> Result<Vec<ModelInfo>> {
        trace!("Listing available models");
        // First get models from registry
        let registry_models = self.model_registry.get_all_models();
        let mut registry_model_ids = HashSet::new();
        let mut models = Vec::new();
        
        // Convert registry models to ModelInfo
        for model in registry_models {
            registry_model_ids.insert(model.id.clone());
            models.push(model.to_model_info());
        }
        
        trace!("Found {} models in registry", models.len());
        
        // Then try to get additional models from provider
        if let Ok(provider) = self.get_provider().await {
            match provider.list_available_models().await {
                Ok(provider_models) => {
                    trace!("Found {} models from provider", provider_models.len());
                    
                    // Add provider models that aren't in registry
                    for model in provider_models {
                        if !registry_model_ids.contains(&model.id) {
                            models.push(model.clone());
                            
                            // Add to registry for future use
                            let llm_model = LLMModel::from_model_info(&model);
                            if let Err(e) = self.model_registry.add_model(llm_model) {
                                warn!("Failed to add model '{}' to registry: {}", model.id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get models from provider: {}", e);
                    // Continue with registry models only
                }
            }
        }
        
        info!("Found total of {} available models", models.len());
        Ok(models)
    }
    
    /// List downloaded/installed models
    pub async fn list_downloaded_models(&self) -> Result<Vec<ModelInfo>> {
        trace!("Listing downloaded models");
        // First get installed models from registry
        let registry_models = self.model_registry.get_installed_models();
        let mut registry_model_ids = HashSet::new();
        let mut models = Vec::new();
        
        // Convert registry models to ModelInfo
        for model in registry_models {
            registry_model_ids.insert(model.id.clone());
            models.push(model.to_model_info());
        }
        
        trace!("Found {} installed models in registry", models.len());
        
        // Then try to get additional downloaded models from provider
        if models.is_empty() {
            if let Ok(provider) = self.get_provider().await {
                match provider.list_downloaded_models().await {
                    Ok(provider_models) => {
                        trace!("Found {} downloaded models from provider", provider_models.len());
                        
                        // Add all provider models and update registry
                        for model in &provider_models {
                            // Update registry
                            let mut llm_model = LLMModel::from_model_info(model);
                            llm_model.installed = true;
                            if let Err(e) = self.model_registry.add_model(llm_model) {
                                warn!("Failed to update model '{}' in registry: {}", model.id, e);
                            }
                        }
                        
                        models = provider_models;
                    }
                    Err(e) => {
                        warn!("Failed to get downloaded models from provider: {}", e);
                        // Continue with registry models only
                    }
                }
            }
        }
        
        info!("Found total of {} downloaded models", models.len());
        Ok(models)
    }
    
    /// Get model info from registry or provider
    pub async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        trace!("Getting info for model: {}", model_id);
        // First try the registry
        match self.model_registry.get_model(model_id) {
            Ok(model) => {
                trace!("Found model in registry");
                return Ok(model.to_model_info());
            },
            Err(e) => {
                trace!("Model not found in registry: {}, trying provider", e);
                // Fall back to provider
                if let Ok(provider) = self.get_provider().await {
                    match provider.get_model_info(model_id).await {
                        Ok(model_info) => {
                            trace!("Found model through provider");
                            
                            // Add to registry for future use
                            let llm_model = LLMModel::from_model_info(&model_info);
                            if let Err(e) = self.model_registry.add_model(llm_model) {
                                warn!("Failed to add model '{}' to registry: {}", model_id, e);
                            }
                            
                            Ok(model_info)
                        },
                        Err(e) => {
                            error!("Model '{}' not found in provider: {}", model_id, e);
                            Err(e)
                        }
                    }
                } else {
                    error!("No provider available to get model info");
                    Err(ProviderError::ConfigurationError(
                        "No provider available".to_string()
                    ))
                }
            }
        }
    }
    
    /// Download a model using the provider, updating the registry
    pub async fn download_model(&self, model_id: &str) -> Result<()> {
        info!("Starting download of model: {}", model_id);
        
        // Check disk space limit
        if self.config.max_disk_space > 0 {
            // Get model info for size estimation
            let model_info = self.get_model_info(model_id).await?;
            let estimated_size = (model_info.size_mb as u64) * 1024 * 1024; // Convert MB to bytes
            
            // Check if we have enough space or need to free up space
            if !self.model_registry.has_space_for(estimated_size) {
                info!("Insufficient disk space. Attempting to free up {} bytes", estimated_size);
                match self.model_registry.free_up_space(estimated_size) {
                    Ok(freed) => {
                        if freed < estimated_size {
                            return Err(ProviderError::DownloadError(
                                format!("Could not free up enough disk space. Needed {} bytes, freed {} bytes",
                                        estimated_size, freed)
                            ));
                        }
                        info!("Freed up {} bytes of disk space", freed);
                    },
                    Err(e) => {
                        warn!("Failed to free disk space: {}", e);
                        return Err(ProviderError::DownloadError(
                            format!("Failed to free disk space: {}", e)
                        ));
                    }
                }
            }
        }
        
        let provider = self.get_provider().await?;
        
        // Get model info to know download URL
        let model_info = match self.get_model_info(model_id).await {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get model info for download: {}", e);
                return Err(e);
            }
        };
        
        // Register download with the model registry
        if let Some(url) = &model_info.download_url {
            trace!("Registering download with model registry. URL: {}", url);
            if let Err(e) = self.model_registry.start_download(model_id, url, &provider.get_type()) {
                warn!("Failed to register download with model registry: {}", e);
                // Continue with download even if registry fails
            }
        } else {
            warn!("Model '{}' has no download URL", model_id);
        }
        
        // If model versioning is enabled, store the current version
        if self.config.enable_model_versioning {
            trace!("Checking for existing version of model");
            let maybe_update = self.is_model_update(model_id, &model_info).await;
            
            if let Some(true) = maybe_update {
                info!("Download is an update to an existing model version");
                self.track_model_version(model_id, &model_info).await;
            }
        }
        
        // Start download with provider
        trace!("Starting model download through provider");
        let result = provider.download_model(model_id).await;
        
        match &result {
            Ok(_) => {
                info!("Model '{}' downloaded successfully", model_id);
                // Get model path from provider
                if let Ok(status) = provider.get_download_status(model_id).await {
                    if let Err(e) = self.model_registry.complete_download(model_id, &status.target_path) {
                        warn!("Failed to complete download in model registry: {}", e);
                    } else {
                        trace!("Download completed in registry. Path: {}", status.target_path.display());
                    }
                }
            },
            Err(e) => {
                error!("Failed to download model '{}': {}", model_id, e);
                // Register failure with model registry
                if let Err(reg_err) = self.model_registry.fail_download(model_id, &e.to_string()) {
                    warn!("Failed to register download failure with model registry: {}", reg_err);
                }
            }
        }
        
        result
    }
    
    /// Check if a model download is an update to an existing version
    async fn is_model_update(&self, model_id: &str, model_info: &ModelInfo) -> Option<bool> {
        // Check if model exists in registry
        if let Ok(existing_model) = self.model_registry.get_model(model_id) {
            // Check if the versions differ
            let current_version = existing_model.current_version;
            let new_version = model_info.metadata.get("version")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
                
            trace!("Model version comparison - Current: {}, New: {}", current_version, new_version);
            return Some(current_version != new_version);
        }
        
        None
    }
    
    /// Track a model version during update
    async fn track_model_version(&self, model_id: &str, model_info: &ModelInfo) {
        // Get existing versions
        let mut versions = self.model_versions.lock().unwrap();
        let model_versions = versions.entry(model_id.to_string()).or_insert_with(Vec::new);
        
        // Get new version from metadata
        let new_version = model_info.metadata.get("version")
            .cloned()
            .unwrap_or_else(|| format!("v{}", chrono::Utc::now().format("%Y%m%d%H%M%S")));
            
        // Add to version history if not already present
        if !model_versions.contains(&new_version) {
            model_versions.push(new_version.clone());
            info!("Added version '{}' to history for model '{}'", new_version, model_id);
        }
        
        // Update model in registry with new version info
        if let Ok(mut model) = self.model_registry.get_model(model_id) {
            // Update current version
            model.current_version = new_version.clone();
            
            // Add to version history if not exists
            let version_exists = model.versions.iter()
                .any(|v| v.version == new_version);
                
            if !version_exists {
                // Create new version entry
                let new_version_entry = models::ModelVersion {
                    version: new_version,
                    release_date: Some(chrono::Utc::now()),
                    is_latest: true,
                    changes: Vec::new(), // No change info available
                    files: Vec::new(),   // Files will be updated after download
                };
                
                // Mark previous latest version as not latest
                for version in &mut model.versions {
                    version.is_latest = false;
                }
                
                model.versions.push(new_version_entry);
            }
            
            // Update model in registry
            if let Err(e) = self.model_registry.add_model(model) {
                warn!("Failed to update model version info in registry: {}", e);
            }
        }
    }
    
    /// Get download status from registry or provider
    pub async fn get_download_status(&self, model_id: &str) -> Result<DownloadStatus> {
        trace!("Getting download status for model: {}", model_id);
        // First check the registry
        match self.model_registry.get_download_progress(model_id) {
            Ok(progress) => {
                trace!("Found download status in registry: {}% complete", progress.progress * 100.0);
                // Convert to DownloadStatus
                Ok(DownloadStatus {
                    model_id: progress.model_id,
                    progress: progress.progress,
                    bytes_downloaded: progress.bytes_downloaded as usize,
                    total_bytes: progress.total_bytes as usize,
                    speed_bps: progress.speed_bps as usize,
                    eta_seconds: progress.eta_seconds,
                    complete: progress.completed,
                    error: progress.error,
                    target_path: self.model_registry.get_new_model_path(&progress.model_id),
                })
            },
            Err(e) => {
                trace!("Download status not found in registry: {}, trying provider", e);
                // Fall back to provider
                if let Ok(provider) = self.get_provider().await {
                    match provider.get_download_status(model_id).await {
                        Ok(status) => {
                            trace!("Found download status from provider: {}% complete", status.progress * 100.0);
                            
                            // Update registry if possible
                            let registry_progress = models::DownloadProgress {
                                model_id: status.model_id.clone(),
                                progress: status.progress,
                                bytes_downloaded: status.bytes_downloaded as u64,
                                total_bytes: status.total_bytes as u64,
                                speed_bps: status.speed_bps as u64,
                                eta_seconds: status.eta_seconds,
                                start_time: chrono::Utc::now(),
                                last_update: chrono::Utc::now(),
                                completed: status.complete,
                                error: status.error.clone(),
                            };
                            
                            if let Err(e) = self.model_registry.update_download_progress(model_id, registry_progress) {
                                warn!("Failed to update download progress in registry: {}", e);
                            }
                            
                            Ok(status)
                        },
                        Err(e) => {
                            error!("Failed to get download status from provider: {}", e);
                            Err(e)
                        }
                    }
                } else {
                    error!("No provider available to get download status");
                    Err(ProviderError::ConfigurationError(
                        "No provider available".to_string()
                    ))
                }
            }
        }
    }
    
    /// Check if a model is loaded in memory
    pub async fn is_model_loaded(&self, model_id: &str) -> Result<bool> {
        trace!("Checking if model '{}' is loaded", model_id);
        // First check the registry
        match self.model_registry.is_model_loaded(model_id) {
            Ok(loaded) => {
                trace!("Model loaded status from registry: {}", loaded);
                Ok(loaded)
            },
            Err(e) => {
                trace!("Model load status not found in registry: {}, trying provider", e);
                // Fall back to provider
                if let Ok(provider) = self.get_provider().await {
                    match provider.is_model_loaded(model_id).await {
                        Ok(loaded) => {
                            trace!("Model loaded status from provider: {}", loaded);
                            
                            // Update registry
                            if let Err(e) = self.model_registry.set_model_loaded(model_id, loaded) {
                                warn!("Failed to update model loaded state in registry: {}", e);
                            }
                            
                            Ok(loaded)
                        },
                        Err(e) => {
                            error!("Failed to check if model is loaded from provider: {}", e);
                            Err(e)
                        }
                    }
                } else {
                    error!("No provider available to check if model is loaded");
                    Err(ProviderError::ConfigurationError(
                        "No provider available".to_string()
                    ))
                }
            }
        }
    }
    
    /// Load a model into memory
    pub async fn load_model(&self, model_id: &str) -> Result<()> {
        info!("Loading model: {}", model_id);
        
        // Check if model exists in registry
        if let Err(e) = self.model_registry.get_model(model_id) {
            warn!("Model '{}' not found in registry: {}", model_id, e);
            // Continue loading with provider, which might know about it
        }
        
        // Load model using provider
        if let Ok(provider) = self.get_provider().await {
            let result = provider.load_model(model_id).await;
            
            match &result {
                Ok(_) => {
                    info!("Model '{}' loaded successfully", model_id);
                    // Update registry
                    if let Err(e) = self.model_registry.set_model_loaded(model_id, true) {
                        warn!("Failed to update model loaded state in registry: {}", e);
                    }
                    
                    // Update last used timestamp
                    if let Err(e) = self.model_registry.update_model(model_id, |model| {
                        model.update_last_used();
                    }) {
                        warn!("Failed to update last used timestamp: {}", e);
                    }
                },
                Err(e) => {
                    error!("Failed to load model '{}': {}", model_id, e);
                }
            }
            
            result
        } else {
            error!("No provider available to load model");
            Err(ProviderError::ConfigurationError(
                "No provider available".to_string()
            ))
        }
    }
    
    /// Unload a model from memory
    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        info!("Unloading model: {}", model_id);
        
        if let Ok(provider) = self.get_provider().await {
            let result = provider.unload_model(model_id).await;
            
            match &result {
                Ok(_) => {
                    info!("Model '{}' unloaded successfully", model_id);
                    // Update registry
                    if let Err(e) = self.model_registry.set_model_loaded(model_id, false) {
                        warn!("Failed to update model loaded state in registry: {}", e);
                    }
                },
                Err(e) => {
                    error!("Failed to unload model '{}': {}", model_id, e);
                }
            }
            
            result
        } else {
            error!("No provider available to unload model");
            Err(ProviderError::ConfigurationError(
                "No provider available".to_string()
            ))
        }
    }
    
    /// Generate text with a model
    pub async fn generate_text(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<String> {
        trace!("Generating text with model: {}", model_id);
        let resolved_model_id = self.resolve_model_id(model_id).await?;
        let provider = self.get_provider().await?;
        
        let options = options.unwrap_or_default();
        
        // Apply platform-specific optimizations if enabled
        let options = if self.config.enable_optimizations {
            self.optimize_generation_options(options)
        } else {
            options
        };
        
        // Check if model is loaded, load if necessary
        if !self.is_model_loaded(&resolved_model_id).await.unwrap_or(false) {
            debug!("Model '{}' not loaded, loading now", resolved_model_id);
            if let Err(e) = self.load_model(&resolved_model_id).await {
                warn!("Failed to load model: {}", e);
                // Continue anyway, as the provider might load it automatically
            }
        }
        
        // Update last used timestamp
        if let Err(e) = self.model_registry.update_model(&resolved_model_id, |model| {
            model.update_last_used();
        }) {
            trace!("Failed to update last used timestamp: {}", e);
        }
        
        trace!("Calling provider to generate text");
        provider.generate_text(&resolved_model_id, prompt, options).await
    }
    
    /// Generate text with streaming
    pub async fn generate_text_streaming<F>(
        &self,
        model_id: &str,
        prompt: &str,
        options: Option<GenerationOptions>,
        callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        trace!("Generating streaming text with model: {}", model_id);
        let resolved_model_id = self.resolve_model_id(model_id).await?;
        let provider = self.get_provider().await?;
        
        let mut options = options.unwrap_or_default();
        
        // Force streaming mode
        options.stream = true;
        
        // Apply platform-specific optimizations if enabled
        let options = if self.config.enable_optimizations {
            self.optimize_generation_options(options)
        } else {
            options
        };
        
        // Check if model is loaded, load if necessary
        if !self.is_model_loaded(&resolved_model_id).await.unwrap_or(false) {
            debug!("Model '{}' not loaded, loading now", resolved_model_id);
            if let Err(e) = self.load_model(&resolved_model_id).await {
                warn!("Failed to load model: {}", e);
                // Continue anyway, as the provider might load it automatically
            }
        }
        
        // Update last used timestamp
        if let Err(e) = self.model_registry.update_model(&resolved_model_id, |model| {
            model.update_last_used();
        }) {
            trace!("Failed to update last used timestamp: {}", e);
        }
        
        trace!("Calling provider to generate streaming text");
        provider.generate_text_streaming(&resolved_model_id, prompt, options, callback).await
    }
    
    /// Resolve model ID (empty string to default model)
    async fn resolve_model_id(&self, model_id: &str) -> Result<String> {
        if !model_id.is_empty() {
            return Ok(model_id.to_string());
        }
        
        // Use default model if specified
        if let Some(default_model) = &self.config.default_model {
            trace!("Using default model: {}", default_model);
            return Ok(default_model.clone());
        }
        
        // Try to find an installed model from registry
        let installed_models = self.model_registry.get_installed_models();
        if !installed_models.is_empty() {
            trace!("Using first installed model: {}", installed_models[0].id);
            return Ok(installed_models[0].id.clone());
        }
        
        // Try to find any model from provider
        if let Ok(provider) = self.get_provider().await {
            match provider.list_downloaded_models().await {
                Ok(models) => {
                    if !models.is_empty() {
                        trace!("Using first provider model: {}", models[0].id);
                        return Ok(models[0].id.clone());
                    }
                },
                Err(e) => {
                    warn!("Failed to get models from provider: {}", e);
                }
            }
        }
        
        error!("No models available and no default model specified");
        Err(ProviderError::ModelNotFound(
            "No models available and no default model specified".to_string()
        ))
    }
    
    /// Optimize generation options based on platform capabilities
    fn optimize_generation_options(&self, mut options: GenerationOptions) -> GenerationOptions {
        // Adjust batch size based on available memory
        if self.acceleration_info.gpu_type != GpuType::None {
            // Increase batch size for GPU inference
            if let Some(vram) = self.acceleration_info.gpu_memory_mb {
                // Set batch size based on VRAM
                if vram > 8192 {
                    // High-end GPU, use larger batch
                    options.additional_params.insert("batch_size".to_string(), 
                        serde_json::Value::Number(serde_json::Number::from(32)));
                } else if vram > 4096 {
                    // Mid-range GPU
                    options.additional_params.insert("batch_size".to_string(), 
                        serde_json::Value::Number(serde_json::Number::from(16)));
                }
            }
        } else {
            // CPU inference optimizations
            if self.acceleration_info.cpu_features.contains(&"avx2".to_string()) {
                // Modern CPU with AVX2, can handle decent batch sizes
                options.additional_params.insert("batch_size".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(8)));
            } else if self.acceleration_info.cpu_features.contains(&"avx".to_string()) {
                // Older CPU with basic AVX
                options.additional_params.insert("batch_size".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(4)));
            } else {
                // Conservative batch size for basic CPUs
                options.additional_params.insert("batch_size".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(2)));
            }
        }
        
        options
    }
    
    /// Get hardware acceleration info
    pub fn get_acceleration_info(&self) -> &AccelerationInfo {
        &self.acceleration_info
    }
    
    /// Set the default model
    pub async fn set_default_model(&self, model_id: &str) -> Result<()> {
        info!("Setting default model to: {}", model_id);
        
        // Check if model exists in registry or provider
        self.get_model_info(model_id).await?;
        
        // Update configuration
        // TODO: Persist this change to a configuration file
        
        Ok(())
    }
    
    /// Get model registry
    pub fn get_model_registry(&self) -> Arc<ModelRegistry> {
        self.model_registry.clone()
    }
    
    /// Subscribe to model events
    pub fn subscribe_to_model_events(&self) -> tokio::sync::broadcast::Receiver<ModelRegistryEvent> {
        self.model_registry.subscribe()
    }
    
    /// Import a model from a path
    pub async fn import_model(&self, source_path: &std::path::Path, model_id: &str) -> Result<()> {
        info!("Importing model from '{}' as '{}'", source_path.display(), model_id);
        
        // Check if model already exists
        match self.model_registry.get_model(model_id) {
            Ok(_) => {
                // Model exists, check if we should overwrite
                warn!("Model '{}' already exists in registry", model_id);
                return Err(ProviderError::ConfigurationError(
                    format!("Model '{}' already exists. Use a different ID or delete the existing model first.",
                           model_id)
                ));
            },
            Err(_) => {
                // Model doesn't exist, proceed with import
            }
        }
        
        // Check if source path exists
        if !source_path.exists() {
            return Err(ProviderError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Source path '{}' does not exist", source_path.display())
                )
            ));
        }
        
        // Check disk space limit
        if self.config.max_disk_space > 0 {
            let size = self.get_directory_size(source_path)?;
            
            // Check if we have enough space or need to free up space
            if !self.model_registry.has_space_for(size) {
                info!("Insufficient disk space. Attempting to free up {} bytes", size);
                match self.model_registry.free_up_space(size) {
                    Ok(freed) => {
                        if freed < size {
                            return Err(ProviderError::DownloadError(
                                format!("Could not free up enough disk space. Needed {} bytes, freed {} bytes",
                                        size, freed)
                            ));
                        }
                        info!("Freed up {} bytes of disk space", freed);
                    },
                    Err(e) => {
                        warn!("Failed to free disk space: {}", e);
                        return Err(ProviderError::DownloadError(
                            format!("Failed to free disk space: {}", e)
                        ));
                    }
                }
            }
        }
        
        // Import the model
        self.model_registry.import_model(source_path, model_id)
    }
    
    /// Helper to calculate directory size
    fn get_directory_size(&self, path: &std::path::Path) -> Result<u64> {
        let mut size = 0;
        
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                size += metadata.len();
            }
        } else if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        size += metadata.len();
                    }
                } else if path.is_dir() {
                    size += self.get_directory_size(&path)?;
                }
            }
        }
        
        Ok(size)
    }
    
    /// Export a model to a path
    pub async fn export_model(&self, model_id: &str, dest_path: &std::path::Path) -> Result<()> {
        info!("Exporting model '{}' to '{}'", model_id, dest_path.display());
        
        // Check if model exists
        self.model_registry.get_model(model_id)?;
        
        // Check if destination directory exists
        if !dest_path.exists() {
            if let Err(e) = fs::create_dir_all(dest_path) {
                return Err(ProviderError::IoError(e));
            }
        }
        
        // Export the model
        self.model_registry.export_model(model_id, dest_path)
    }
    
    /// Delete a model
    pub async fn delete_model(&self, model_id: &str) -> Result<()> {
        info!("Deleting model: {}", model_id);
        
        // Try to unload model first if it's loaded
        if self.is_model_loaded(model_id).await.unwrap_or(false) {
            info!("Model is loaded, unloading first");
            if let Err(e) = self.unload_model(model_id).await {
                warn!("Failed to unload model: {}", e);
                // Continue with deletion anyway
            }
        }
        
        // Try to delete with provider
        if let Ok(provider) = self.get_provider().await {
            if let Err(e) = provider.delete_model(model_id).await {
                warn!("Failed to delete model from provider: {}", e);
                // Continue with registry deletion
            }
        }
        
        // Get model path from registry
        let model_path = match self.model_registry.get_model_path(model_id) {
            Ok(path) => Some(path),
            Err(e) => {
                warn!("Failed to get model path from registry: {}", e);
                None
            }
        };
        
        // Remove from registry
        if let Err(e) = self.model_registry.remove_model(model_id) {
            warn!("Failed to remove model from registry: {}", e);
            // Continue with directory deletion
        }
        
        // Delete model files
        if let Some(path) = model_path {
            if path.exists() {
                info!("Deleting model files at: {}", path.display());
                if let Err(e) = fs::remove_dir_all(&path) {
                    warn!("Failed to delete model directory: {}", e);
                    return Err(ProviderError::IoError(e));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get model versions
    pub async fn get_model_versions(&self, model_id: &str) -> Result<Vec<String>> {
        trace!("Getting version history for model: {}", model_id);
        
        // Check if model exists
        self.model_registry.get_model(model_id)?;
        
        // Get versions from cache
        let versions = self.model_versions.lock().unwrap();
        if let Some(version_list) = versions.get(model_id) {
            trace!("Found {} versions in cache", version_list.len());
            return Ok(version_list.clone());
        }
        
        // Try to get versions from registry model
        let model = self.model_registry.get_model(model_id)?;
        let model_versions: Vec<String> = model.versions
            .iter()
            .map(|v| v.version.clone())
            .collect();
            
        if !model_versions.is_empty() {
            trace!("Found {} versions in model", model_versions.len());
            return Ok(model_versions);
        }
        
        // No versions found, return empty list
        trace!("No versions found for model");
        Ok(Vec::new())
    }
    
    /// Switch to a specific model version
    pub async fn switch_model_version(&self, model_id: &str, version: &str) -> Result<()> {
        info!("Switching model '{}' to version '{}'", model_id, version);
        
        // Check if model exists
        let model = self.model_registry.get_model(model_id)?;
        
        // Check if version exists
        let version_exists = model.versions.iter()
            .any(|v| v.version == version);
            
        if !version_exists {
            return Err(ProviderError::ModelNotFound(
                format!("Version '{}' not found for model '{}'", version, model_id)
            ));
        }
        
        // This would typically involve:
        // 1. Unloading the current model
        // 2. Potentially downloading the specific version
        // 3. Updating the model registry
        // 4. Loading the new version
        
        // For now, we'll just update the registry to mark this as the current version
        self.model_registry.update_model(model_id, |model| {
            model.current_version = version.to_string();
            
            // Update is_latest flags
            for v in &mut model.versions {
                v.is_latest = v.version == version;
            }
        })?;
        
        // If the model is loaded, reload it
        if self.is_model_loaded(model_id).await? {
            info!("Model is loaded, reloading to apply version change");
            self.unload_model(model_id).await?;
            self.load_model(model_id).await?;
        }
        
        Ok(())
    }
    
    /// Calculate disk space used by models
    pub fn calculate_disk_usage(&self) -> u64 {
        trace!("Calculating total disk usage for models");
        self.model_registry.calculate_disk_usage()
    }
    
    /// Free up disk space by removing unused models
    pub async fn free_up_disk_space(&self, needed_bytes: u64) -> Result<u64> {
        info!("Attempting to free up {} bytes of disk space", needed_bytes);
        self.model_registry.free_up_space(needed_bytes)
    }
    
    /// Set disk space limit for models
    pub fn set_disk_space_limit(&self, limit_bytes: u64) -> Result<()> {
        info!("Setting disk space limit to {} bytes", limit_bytes);
        
        if let Ok(registry_as_ref) = Arc::get_mut(&mut self.model_registry.clone()) {
            registry_as_ref.set_disk_space_limit(limit_bytes);
            Ok(())
        } else {
            warn!("Failed to set disk space limit, registry is in use");
            Err(ProviderError::ConfigurationError(
                "Failed to update registry, it is in use by other threads".to_string()
            ))
        }
    }
    
    /// Search for models matching a query
    pub async fn search_models(&self, query: &str) -> Result<Vec<ModelInfo>> {
        trace!("Searching for models matching: {}", query);
        
        // Get all models from registry
        let all_models = self.model_registry.get_all_models();
        let query = query.to_lowercase();
        
        // Filter models by the query
        let matching_models: Vec<ModelInfo> = all_models.into_iter()
            .filter(|model| {
                // Check various fields for matches
                model.id.to_lowercase().contains(&query) ||
                model.name.to_lowercase().contains(&query) ||
                model.description.to_lowercase().contains(&query) ||
                model.architecture.to_string().to_lowercase().contains(&query) ||
                model.metadata.values().any(|v| v.to_lowercase().contains(&query))
            })
            .map(|model| model.to_model_info())
            .collect();
            
        trace!("Found {} models matching query", matching_models.len());
        Ok(matching_models)
    }
    
    /// Get models compatible with a specific provider
    pub async fn get_compatible_models(&self, provider_type: &ProviderType) -> Result<Vec<ModelInfo>> {
        trace!("Finding models compatible with provider: {}", provider_type);
        
        let compatible_models = self.model_registry.get_compatible_models(provider_type);
        
        trace!("Found {} compatible models", compatible_models.len());
        Ok(compatible_models.into_iter().map(|m| m.to_model_info()).collect())
    }
    
    /// Get cached download status across all models
    pub async fn get_all_downloads(&self) -> HashMap<String, DownloadStatus> {
        trace!("Getting status of all downloads");
        
        let mut result = HashMap::new();
        let downloads = self.model_registry.get_all_download_progress();
        
        for (model_id, progress) in downloads {
            // Convert to DownloadStatus
            let status = DownloadStatus {
                model_id: progress.model_id.clone(),
                progress: progress.progress,
                bytes_downloaded: progress.bytes_downloaded as usize,
                total_bytes: progress.total_bytes as usize,
                speed_bps: progress.speed_bps as usize,
                eta_seconds: progress.eta_seconds,
                complete: progress.completed,
                error: progress.error,
                target_path: self.model_registry.get_new_model_path(&progress.model_id),
            };
            
            result.insert(model_id, status);
        }
        
        result
    }
    
    /// Cancel an ongoing model download
    pub async fn cancel_download(&self, model_id: &str) -> Result<()> {
        info!("Canceling download of model: {}", model_id);
        
        // Try to cancel with provider
        if let Ok(provider) = self.get_provider().await {
            if let Err(e) = provider.cancel_download(model_id).await {
                warn!("Failed to cancel download with provider: {}", e);
                // Continue with registry update
            }
        }
        
        // Update registry with cancellation
        if let Err(e) = self.model_registry.fail_download(model_id, "Download canceled by user") {
            warn!("Failed to update registry for canceled download: {}", e);
        }
        
        Ok(())
    }
}

// Add a method to ModelRegistry to get all download progress
impl ModelRegistry {
    /// Get all download progress
    pub fn get_all_download_progress(&self) -> HashMap<String, DownloadProgress> {
        let downloads = self.downloads.read().unwrap();
        downloads.clone()
    }
}

/// Factory function to create a LLM manager
pub fn create_llm_manager() -> LLMManager {
    LLMManager::new()
}

/// Factory function to create a LLM manager with custom configuration
pub fn create_llm_manager_with_config(config: LLMConfig) -> LLMManager {
    LLMManager::with_config(config)
}