use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use chrono::Utc;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::offline::llm::provider::{
    CompletionResponse, CompletionStream, LocalLLMProvider,
    LLMProviderError, LLMProviderResult, ModelConfig, ModelInfo, TokenUsage,
};
use crate::offline::llm::types::{DownloadStatus, GenerationOptions, ProviderType};

/// Ollama API response for listing models
#[derive(Debug, Deserialize)]
struct OllamaListModelsResponse {
    models: Vec<OllamaModelInfo>,
}

/// Ollama API model information
#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: Option<OllamaModelDetails>,
}

/// Ollama API model details
#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    format: String,
    family: String,
    families: Option<Vec<String>>,
    parameter_size: Option<String>,
    quantization_level: Option<String>,
}

/// Ollama API generation request
#[derive(Debug, Serialize)]
struct OllamaGenerationRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<HashMap<String, serde_json::Value>>,
}

/// Ollama API generation response
#[derive(Debug, Deserialize)]
struct OllamaGenerationResponse {
    model: String,
    response: String,
    done: bool,
    context: Option<Vec<u32>>,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_duration: Option<u64>,
    eval_count: Option<u32>,
    eval_duration: Option<u64>,
}

/// Ollama API pull request
#[derive(Debug, Serialize)]
struct OllamaPullRequest {
    name: String,
    stream: bool,
    insecure: Option<bool>,
}

/// Ollama API pull response
#[derive(Debug, Deserialize)]
struct OllamaPullResponse {
    status: String,
    digest: Option<String>,
    total: Option<u64>,
    completed: Option<u64>,
    error: Option<String>,
}

/// Ollama API delete request
#[derive(Debug, Serialize)]
struct OllamaDeleteRequest {
    name: String,
}

/// Statistics for a running download
#[derive(Debug, Clone)]
struct DownloadStats {
    /// Start time of the download
    start_time: chrono::DateTime<Utc>,
    /// Last update time
    last_update: chrono::DateTime<Utc>,
    /// Total bytes to download
    total_bytes: Option<u64>,
    /// Bytes downloaded so far
    bytes_downloaded: Option<u64>,
    /// Download speed in bytes per second
    bytes_per_second: Option<f32>,
    /// Sample of recent download speeds for averaging
    speed_samples: Vec<f32>,
}

impl DownloadStats {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            start_time: now,
            last_update: now,
            total_bytes: None,
            bytes_downloaded: None,
            bytes_per_second: None,
            speed_samples: Vec::with_capacity(10),
        }
    }
    
    fn update(&mut self, completed: Option<u64>, total: Option<u64>) {
        let now = Utc::now();
        let time_elapsed = (now - self.last_update).num_milliseconds() as f32 / 1000.0;
        
        // Update bytes
        if let Some(total) = total {
            self.total_bytes = Some(total);
        }
        
        // Calculate speed if we have new completed data
        if let Some(completed) = completed {
            if let Some(old_completed) = self.bytes_downloaded {
                if time_elapsed > 0.0 && completed > old_completed {
                    let bytes_delta = (completed - old_completed) as f32;
                    let speed = bytes_delta / time_elapsed;
                    
                    // Add to speed samples, keeping a fixed window
                    self.speed_samples.push(speed);
                    if self.speed_samples.len() > 10 {
                        self.speed_samples.remove(0);
                    }
                    
                    // Calculate average speed
                    let avg_speed = self.speed_samples.iter().sum::<f32>() / self.speed_samples.len() as f32;
                    self.bytes_per_second = Some(avg_speed);
                }
            }
            
            self.bytes_downloaded = Some(completed);
        }
        
        self.last_update = now;
    }
    
    fn eta_seconds(&self) -> Option<f32> {
        if let (Some(total), Some(completed), Some(speed)) = (self.total_bytes, self.bytes_downloaded, self.bytes_per_second) {
            if speed > 0.0 && completed < total {
                let remaining_bytes = (total - completed) as f32;
                return Some(remaining_bytes / speed);
            }
        }
        None
    }
    
    fn to_download_status(&self) -> DownloadStatus {
        let total = self.total_bytes;
        let completed = self.bytes_downloaded;
        
        if let (Some(total), Some(completed)) = (total, completed) {
            if total > 0 {
                let percent = (completed as f32 / total as f32) * 100.0;
                return DownloadStatus::InProgress {
                    percent,
                    bytes_downloaded: completed,
                    total_bytes: total,
                    eta_seconds: self.eta_seconds(),
                    bytes_per_second: self.bytes_per_second,
                };
            }
        }
        
        // Default to 0% if we don't have enough info
        DownloadStatus::InProgress {
            percent: 0.0,
            bytes_downloaded: completed,
            total_bytes: total,
            eta_seconds: None,
            bytes_per_second: None,
        }
    }
}

/// Ollama provider implementation
pub struct OllamaProvider {
    /// Base URL for the Ollama API
    base_url: String,
    /// HTTP client for API requests
    client: Client,
    /// Map of model IDs to download status
    download_status: Arc<RwLock<HashMap<String, DownloadStatus>>>,
    /// Map of model IDs to download statistics
    download_stats: Arc<RwLock<HashMap<String, DownloadStats>>>,
    /// Map of model IDs to model configurations
    model_configs: Arc<RwLock<HashMap<String, ModelConfig>>>,
    /// Whether the provider has been initialized
    initialized: Arc<Mutex<bool>>,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the default URL
    pub fn new() -> Self {
        Self::with_base_url("http://localhost:11434")
    }

    /// Create a new Ollama provider with a custom URL
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
            download_status: Arc::new(RwLock::new(HashMap::new())),
            download_stats: Arc::new(RwLock::new(HashMap::new())),
            model_configs: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Check if the provider is initialized
    fn ensure_initialized(&self) -> LLMProviderResult<()> {
        let initialized = *self.initialized.lock().map_err(|e| {
            LLMProviderError::Unexpected(format!("Failed to acquire lock: {}", e))
        })?;

        if !initialized {
            return Err(LLMProviderError::NotInitialized(
                "Ollama provider not initialized. Call initialize() first.".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert Ollama model info to the common ModelInfo format
    fn convert_to_model_info(&self, ollama_info: OllamaModelInfo, is_downloaded: bool) -> ModelInfo {
        let mut provider_metadata = HashMap::new();
        let mut parameter_count = None;
        let mut quant_level = None;
        let mut model_family = None;
        
        if let Some(details) = ollama_info.details {
            provider_metadata.insert("format".to_string(), serde_json::to_value(details.format).unwrap_or_default());
            
            // Extract the model family
            model_family = Some(details.family.clone());
            provider_metadata.insert("family".to_string(), serde_json::to_value(details.family).unwrap_or_default());
            
            if let Some(families) = details.families {
                provider_metadata.insert("families".to_string(), serde_json::to_value(families).unwrap_or_default());
            }
            
            // Extract parameter count
            if let Some(param_size) = details.parameter_size {
                provider_metadata.insert("parameter_size".to_string(), serde_json::to_value(&param_size).unwrap_or_default());
                
                // Parse the parameter size (usually in format like "7B")
                if let Some(size_str) = param_size.trim_end_matches('B').parse::<f64>().ok() {
                    parameter_count = Some(size_str);
                }
            }
            
            // Extract quantization level
            if let Some(q_level) = details.quantization_level {
                provider_metadata.insert("quantization_level".to_string(), serde_json::to_value(&q_level).unwrap_or_default());
                quant_level = Some(q_level);
            }
        }
        
        provider_metadata.insert("digest".to_string(), serde_json::to_value(ollama_info.digest).unwrap_or_default());
        provider_metadata.insert("modified_at".to_string(), serde_json::to_value(ollama_info.modified_at).unwrap_or_default());

        // Create tags from model name components
        let tags = ollama_info.name.split(':')
            .skip(1)  // Skip the base name
            .filter(|&s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Extract base name without tags
        let base_name = ollama_info.name.split(':').next().unwrap_or(&ollama_info.name).to_string();
        
        // Determine what capabilities the model has
        // For Ollama, we'll assume all models support text generation and chat
        let supports_text_generation = true;
        let supports_completion = true;
        let supports_chat = true;
        let supports_embeddings = false;  // Ollama doesn't support embeddings yet
        let supports_image_generation = false;  // Ollama doesn't support image generation yet

        ModelInfo {
            id: ollama_info.name.clone(),
            name: format!("{}{}", 
                base_name,
                if !quant_level.is_none() { 
                    format!(" ({})", quant_level.unwrap_or_default()) 
                } else { 
                    "".to_string() 
                }
            ),
            description: format!("{}{} model", 
                if let Some(params) = parameter_count {
                    format!("{:.1}B parameter ", params)
                } else {
                    "".to_string()
                },
                model_family.unwrap_or_else(|| "LLM".to_string())
            ),
            size_bytes: ollama_info.size,
            is_downloaded,
            provider_metadata,
            provider: ProviderType::Ollama,
            supports_text_generation,
            supports_completion,
            supports_chat,
            supports_embeddings,
            supports_image_generation,
            quantization: quant_level,
            parameter_count_b: parameter_count,
            context_length: None,  // Ollama doesn't provide this info
            model_family,
            created_at: None,  // Ollama doesn't provide this info
            tags,
            license: None,  // Ollama doesn't provide this info
        }
    }
    
    /// Set the download status for a model
    async fn set_download_status(&self, model_id: &str, status: DownloadStatus) {
        let mut status_map = self.download_status.write().await;
        status_map.insert(model_id.to_string(), status);
    }

    /// Cancel any running download for a model
    async fn _cleanup_download_tasks(&self, model_id: &str) {
        // Remove from status maps
        {
            let mut status_map = self.download_status.write().await;
            status_map.remove(model_id);
        }
        {
            let mut stats_map = self.download_stats.write().await;
            stats_map.remove(model_id);
        }
    }
}

/// Ollama implementation of CompletionStream
pub struct OllamaCompletionStream {
    /// HTTP response stream
    response: reqwest::Response,
    /// Total completion text accumulated so far
    accumulated_text: String,
    /// Whether we've reached the end of the stream
    completed: bool,
}

#[async_trait]
impl CompletionStream for OllamaCompletionStream {
    async fn next_chunk(&mut self) -> Option<LLMProviderResult<String>> {
        if self.completed {
            return None;
        }

        match self.response.chunk().await {
            Ok(Some(chunk)) => {
                let chunk_str = match std::str::from_utf8(&chunk) {
                    Ok(s) => s,
                    Err(e) => return Some(Err(LLMProviderError::Unexpected(format!(
                        "Failed to decode response chunk: {}", e
                    )))),
                };

                // Parse the JSON response
                match serde_json::from_str::<OllamaGenerationResponse>(chunk_str) {
                    Ok(response) => {
                        self.accumulated_text.push_str(&response.response);
                        
                        if response.done {
                            self.completed = true;
                        }
                        
                        Some(Ok(response.response))
                    },
                    Err(e) => Some(Err(LLMProviderError::Unexpected(format!(
                        "Failed to parse response JSON: {}", e
                    )))),
                }
            },
            Ok(None) => {
                self.completed = true;
                None
            },
            Err(e) => Some(Err(LLMProviderError::NetworkError(format!(
                "Failed to get response chunk: {}", e
            )))),
        }
    }
}

#[async_trait]
impl LocalLLMProvider for OllamaProvider {
    async fn initialize(&mut self, config: serde_json::Value) -> LLMProviderResult<()> {
        // Extract configuration options
        let config_obj = match config.as_object() {
            Some(obj) => obj,
            None => {
                warn!("Ollama provider configuration is not a JSON object");
                return Err(LLMProviderError::ProviderError(
                    "Configuration must be a JSON object".to_string(),
                ));
            }
        };

        // Override base URL if provided
        if let Some(base_url) = config_obj.get("base_url").and_then(|v| v.as_str()) {
            debug!("Setting Ollama base URL to: {}", base_url);
            self.base_url = base_url.to_string();
        }

        // Test connection to Ollama API
        info!("Testing connection to Ollama API at {}", self.base_url);
        let health_url = format!("{}/api/version", self.base_url);
        
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    error!("Failed to connect to Ollama API: HTTP {}", response.status());
                    return Err(LLMProviderError::ProviderError(format!(
                        "Failed to connect to Ollama API: HTTP {}", response.status()
                    )));
                }
                
                // Try to parse the version response
                match response.json::<serde_json::Value>().await {
                    Ok(version_data) => {
                        if let Some(version) = version_data.get("version").and_then(|v| v.as_str()) {
                            info!("Connected to Ollama (version: {})", version);
                        } else {
                            info!("Connected to Ollama (unknown version)");
                        }
                    },
                    Err(e) => {
                        warn!("Connected to Ollama but couldn't parse version info: {}", e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to connect to Ollama API: {}", e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to connect to Ollama API: {}. Make sure Ollama is running at {}.", 
                    e, self.base_url
                )));
            }
        }

        // Mark as initialized
        match self.initialized.lock() {
            Ok(mut initialized) => {
                *initialized = true;
                info!("Ollama provider initialized successfully");
            },
            Err(e) => {
                error!("Failed to acquire lock when initializing Ollama provider: {}", e);
                return Err(LLMProviderError::Unexpected(format!(
                    "Failed to acquire lock: {}", e
                )));
            }
        }

        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Ollama"
    }

    async fn list_available_models(&self) -> LLMProviderResult<Vec<ModelInfo>> {
        self.ensure_initialized()?;
        debug!("Listing available models from Ollama");

        let url = format!("{}/api/tags", self.base_url);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to fetch models from Ollama: {}", e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to fetch models: {}", e
                )));
            }
        };

        if !response.status().is_success() {
            error!("Failed to fetch models from Ollama: HTTP {}", response.status());
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to fetch models: HTTP {}", response.status()
            )));
        }

        let models_response: OllamaListModelsResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse models response from Ollama: {}", e);
                return Err(LLMProviderError::ProviderError(format!(
                    "Failed to parse models response: {}", e
                )));
            }
        };

        let models = models_response.models.into_iter()
            .map(|m| self.convert_to_model_info(m, true))
            .collect::<Vec<_>>();

        info!("Found {} models from Ollama", models.len());
        Ok(models)
    }

    async fn list_downloaded_models(&self) -> LLMProviderResult<Vec<ModelInfo>> {
        // For Ollama, all listed models are downloaded models
        debug!("Listing downloaded models from Ollama");
        self.list_available_models().await
    }

    async fn download_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        info!("Starting download of model '{}' from Ollama", model_id);

        // Update download status
        self.set_download_status(model_id, DownloadStatus::InProgress { 
            percent: 0.0,
            bytes_downloaded: None,
            total_bytes: None,
            eta_seconds: None,
            bytes_per_second: None,
        }).await;
        
        // Initialize download stats
        {
            let mut stats_map = self.download_stats.write().await;
            stats_map.insert(model_id.to_string(), DownloadStats::new());
        }

        // Construct the pull request
        let url = format!("{}/api/pull", self.base_url);
        let request = OllamaPullRequest {
            name: model_id.to_string(),
            stream: true,
            insecure: None,
        };

        // Start download in a separate task
        let model_id = model_id.to_string();
        let client = self.client.clone();
        let download_status = self.download_status.clone();
        let download_stats = self.download_stats.clone();

        tokio::spawn(async move {
            debug!("Sending pull request for model '{}'", model_id);
            let result = client.post(&url).json(&request).send().await;
            
            match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        error!("Failed to download model '{}': HTTP {}", model_id, response.status());
                        let mut status_map = download_status.write().await;
                        status_map.insert(model_id, DownloadStatus::Failed { 
                            reason: format!("HTTP error: {}", response.status()),
                            error_code: Some(response.status().to_string()),
                            failed_at: Some(Utc::now().to_rfc3339()),
                        });
                        return;
                    }

                    // Stream the response to track progress
                    debug!("Streaming download response for model '{}'", model_id);
                    let mut stream = response.bytes_stream();
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Ok(text) = std::str::from_utf8(&chunk) {
                                    match serde_json::from_str::<OllamaPullResponse>(text) {
                                        Ok(pull_response) => {
                                            match pull_response.status.as_str() {
                                                "pulling manifest" => {
                                                    debug!("Pulling manifest for model '{}'", model_id);
                                                },
                                                "pulling layers" => {
                                                    // Extract progress information if available
                                                    let total = pull_response.total;
                                                    let completed = pull_response.completed;
                                                    
                                                    // Update stats
                                                    {
                                                        let mut stats_map = download_stats.write().await;
                                                        if let Some(stats) = stats_map.get_mut(&model_id) {
                                                            stats.update(completed, total);
                                                            
                                                            // Update status from stats
                                                            let status = stats.to_download_status();
                                                            let mut status_map = download_status.write().await;
                                                            status_map.insert(model_id.clone(), status);
                                                        }
                                                    }
                                                },
                                                "success" => {
                                                    info!("Successfully downloaded model '{}'", model_id);
                                                    let mut status_map = download_status.write().await;
                                                    status_map.insert(model_id.clone(), DownloadStatus::Completed { 
                                                        completed_at: Some(Utc::now().to_rfc3339()),
                                                        duration_seconds: {
                                                            let stats_map = download_stats.read().await;
                                                            stats_map.get(&model_id).map(|stats| {
                                                                let duration = Utc::now() - stats.start_time;
                                                                Some(duration.num_milliseconds() as f32 / 1000.0)
                                                            }).flatten()
                                                        }
                                                    });
                                                    break;
                                                },
                                                "error" => {
                                                    let error_msg = pull_response.error.unwrap_or_else(|| "Unknown error".to_string());
                                                    error!("Error downloading model '{}': {}", model_id, error_msg);
                                                    let mut status_map = download_status.write().await;
                                                    status_map.insert(model_id.clone(), DownloadStatus::Failed { 
                                                        reason: error_msg,
                                                        error_code: None,
                                                        failed_at: Some(Utc::now().to_rfc3339()),
                                                    });
                                                    break;
                                                },
                                                _ => {
                                                    debug!("Unhandled pull status for model '{}': {}", model_id, pull_response.status);
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            warn!("Failed to parse Ollama pull response for model '{}': {}", model_id, e);
                                            warn!("Response text: {}", text);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Failed to read response chunk for model '{}': {}", model_id, e);
                                let mut status_map = download_status.write().await;
                                status_map.insert(model_id.clone(), DownloadStatus::Failed { 
                                    reason: format!("Failed to read response: {}", e),
                                    error_code: None,
                                    failed_at: Some(Utc::now().to_rfc3339()),
                                });
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Network error while downloading model '{}': {}", model_id, e);
                    let mut status_map = download_status.write().await;
                    status_map.insert(model_id, DownloadStatus::Failed { 
                        reason: format!("Network error: {}", e),
                        error_code: None,
                        failed_at: Some(Utc::now().to_rfc3339()),
                    });
                }
            }
        });

        Ok(())
    }

    async fn get_download_status(&self, model_id: &str) -> LLMProviderResult<DownloadStatus> {
        self.ensure_initialized()?;
        debug!("Getting download status for model '{}'", model_id);

        let status_map = self.download_status.read().await;
        match status_map.get(model_id) {
            Some(status) => Ok(status.clone()),
            None => {
                // If no status is recorded, check if the model exists already
                debug!("No download status found for model '{}', checking if it exists", model_id);
                let models = self.list_downloaded_models().await?;
                if models.iter().any(|m| m.id == model_id) {
                    debug!("Model '{}' is already downloaded", model_id);
                    Ok(DownloadStatus::Completed {
                        completed_at: None,
                        duration_seconds: None,
                    })
                } else {
                    debug!("Model '{}' is not downloaded", model_id);
                    Ok(DownloadStatus::NotStarted)
                }
            }
        }
    }

    async fn cancel_download(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        debug!("Cancelling download of model '{}'", model_id);

        // Ollama doesn't have a direct API to cancel downloads
        // We can only update our status to indicate cancellation
        let mut status_map = self.download_status.write().await;
        match status_map.get(model_id) {
            Some(status) => {
                match status {
                    DownloadStatus::InProgress { .. } => {
                        info!("Marking download of model '{}' as cancelled", model_id);
                        status_map.insert(model_id.to_string(), DownloadStatus::Cancelled {
                            cancelled_at: Some(Utc::now().to_rfc3339()),
                        });
                        Ok(())
                    },
                    _ => {
                        warn!("Cannot cancel download of model '{}' as it is not in progress", model_id);
                        Err(LLMProviderError::ProviderError(
                            "Model is not currently downloading".to_string()
                        ))
                    },
                }
            }
            None => {
                warn!("Cannot cancel download of model '{}' as it is not found", model_id);
                Err(LLMProviderError::ModelNotFound(model_id.to_string()))
            }
        }
    }

    async fn delete_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        info!("Deleting model '{}'", model_id);

        let url = format!("{}/api/delete", self.base_url);
        let request = OllamaDeleteRequest {
            name: model_id.to_string(),
        };

        let response = match self.client.delete(&url).json(&request).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to delete model '{}': {}", model_id, e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to delete model: {}", e
                )));
            }
        };

        match response.status() {
            StatusCode::OK => {
                info!("Successfully deleted model '{}'", model_id);
                
                // Remove status and config for the deleted model
                {
                    let mut status_map = self.download_status.write().await;
                    status_map.remove(model_id);
                }
                {
                    let mut stats_map = self.download_stats.write().await;
                    stats_map.remove(model_id);
                }
                {
                    let mut config_map = self.model_configs.write().await;
                    config_map.remove(model_id);
                }
                
                Ok(())
            },
            StatusCode::NOT_FOUND => {
                warn!("Model '{}' not found for deletion", model_id);
                Err(LLMProviderError::ModelNotFound(model_id.to_string()))
            },
            _ => {
                error!("Failed to delete model '{}': HTTP {}", model_id, response.status());
                Err(LLMProviderError::ProviderError(format!(
                    "Failed to delete model: HTTP {}", response.status()
                )))
            }
        }
    }

    async fn is_model_loaded(&self, model_id: &str) -> LLMProviderResult<bool> {
        self.ensure_initialized()?;
        debug!("Checking if model '{}' is loaded", model_id);

        // Ollama doesn't have a direct API to check if a model is loaded
        // We can send a simple generation request to see if it responds quickly
        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaGenerationRequest {
            model: model_id.to_string(),
            prompt: "".to_string(),
            stream: false,
            options: Some(HashMap::from([
                ("num_predict".to_string(), serde_json::json!(1)),
            ])),
        };

        match self.client.post(&url).json(&request).send().await {
            Ok(response) => {
                match response.status() {
                    StatusCode::OK => {
                        debug!("Model '{}' is loaded", model_id);
                        Ok(true)
                    },
                    StatusCode::NOT_FOUND => {
                        debug!("Model '{}' is not loaded", model_id);
                        Ok(false)
                    },
                    _ => {
                        warn!("Failed to check if model '{}' is loaded: HTTP {}", model_id, response.status());
                        Err(LLMProviderError::ProviderError(format!(
                            "Failed to check model status: HTTP {}", response.status()
                        )))
                    }
                }
            },
            Err(e) => {
                error!("Failed to check if model '{}' is loaded: {}", model_id, e);
                Err(LLMProviderError::NetworkError(format!(
                    "Failed to check model status: {}", e
                )))
            },
        }
    }

    async fn load_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        info!("Loading model '{}'", model_id);

        // Ollama automatically loads models when used, but we can "warm up" the model
        // by sending a trivial request
        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaGenerationRequest {
            model: model_id.to_string(),
            prompt: "".to_string(),
            stream: false,
            options: Some(HashMap::from([
                ("num_predict".to_string(), serde_json::json!(1)),
            ])),
        };

        let response = match self.client.post(&url).json(&request).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to load model '{}': {}", model_id, e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to load model: {}", e
                )));
            }
        };

        if !response.status().is_success() {
            error!("Failed to load model '{}': HTTP {}", model_id, response.status());
            if response.status() == StatusCode::NOT_FOUND {
                return Err(LLMProviderError::ModelNotFound(model_id.to_string()));
            } else {
                return Err(LLMProviderError::ProviderError(format!(
                    "Failed to load model: HTTP {}", response.status()
                )));
            }
        }

        info!("Successfully loaded model '{}'", model_id);
        Ok(())
    }

    async fn unload_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        debug!("Unloading model '{}' (no-op in Ollama)", model_id);
        
        // Ollama doesn't have a direct API to unload models
        // Models are automatically unloaded when not used for a while
        Ok(())
    }

    async fn generate_text(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<CompletionResponse> {
        self.ensure_initialized()?;
        debug!("Generating text with model '{}'", model_id);

        // Prepare the generation request
        let mut ollama_options = HashMap::new();
        
        if let Some(max_tokens) = options.max_tokens {
            ollama_options.insert("num_predict".to_string(), serde_json::json!(max_tokens));
        }
        
        if let Some(temperature) = options.temperature {
            ollama_options.insert("temperature".to_string(), serde_json::json!(temperature));
        }
        
        if let Some(top_p) = options.top_p {
            ollama_options.insert("top_p".to_string(), serde_json::json!(top_p));
        }
        
        if let Some(top_k) = options.top_k {
            ollama_options.insert("top_k".to_string(), serde_json::json!(top_k));
        }
        
        if let Some(fp) = options.frequency_penalty {
            ollama_options.insert("frequency_penalty".to_string(), serde_json::json!(fp));
        }
        
        if let Some(pp) = options.presence_penalty {
            ollama_options.insert("presence_penalty".to_string(), serde_json::json!(pp));
        }
        
        if let Some(seed) = options.seed {
            ollama_options.insert("seed".to_string(), serde_json::json!(seed));
        }
        
        if let Some(stop_sequences) = &options.stop_sequences {
            if !stop_sequences.is_empty() {
                ollama_options.insert("stop".to_string(), serde_json::json!(stop_sequences));
            }
        }
        
        // Add any additional parameters
        for (key, value) in options.additional_params {
            ollama_options.insert(key, value);
        }

        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaGenerationRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: if ollama_options.is_empty() { None } else { Some(ollama_options) },
        };

        // Send the request
        debug!("Sending generation request to Ollama for model '{}'", model_id);
        let response = match self.client.post(&url).json(&request).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to generate text with model '{}': {}", model_id, e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to generate text: {}", e
                )));
            }
        };

        if !response.status().is_success() {
            error!("Failed to generate text with model '{}': HTTP {}", model_id, response.status());
            match response.status() {
                StatusCode::NOT_FOUND => {
                    return Err(LLMProviderError::ModelNotFound(model_id.to_string()));
                },
                StatusCode::BAD_REQUEST => {
                    // Try to extract error message from response
                    let error_text = match response.text().await {
                        Ok(text) => {
                            match serde_json::from_str::<serde_json::Value>(&text) {
                                Ok(json) => {
                                    json.get("error").and_then(|e| e.as_str())
                                        .unwrap_or(&text).to_string()
                                },
                                Err(_) => text,
                            }
                        },
                        Err(_) => "Bad request".to_string(),
                    };
                    return Err(LLMProviderError::GenerationFailed(error_text));
                },
                _ => {
                    return Err(LLMProviderError::ProviderError(format!(
                        "Failed to generate text: HTTP {}", response.status()
                    )));
                }
            }
        }

        // Parse the response
        let ollama_response: OllamaGenerationResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse generation response for model '{}': {}", model_id, e);
                return Err(LLMProviderError::ProviderError(format!(
                    "Failed to parse generation response: {}", e
                )));
            }
        };

        // Create the completion response
        debug!("Successfully generated text with model '{}'", model_id);
        let mut metadata = HashMap::new();
        
        if let Some(context) = &ollama_response.context {
            metadata.insert("context".to_string(), serde_json::to_value(context).unwrap_or_default());
        }
        
        if let Some(total_duration) = ollama_response.total_duration {
            metadata.insert("total_duration_ms".to_string(), serde_json::json!(total_duration));
        }
        
        if let Some(load_duration) = ollama_response.load_duration {
            metadata.insert("load_duration_ms".to_string(), serde_json::json!(load_duration));
        }
        
        if let Some(prompt_eval_duration) = ollama_response.prompt_eval_duration {
            metadata.insert("prompt_eval_duration_ms".to_string(), serde_json::json!(prompt_eval_duration));
        }
        
        if let Some(eval_count) = ollama_response.eval_count {
            metadata.insert("eval_count".to_string(), serde_json::json!(eval_count));
        }
        
        if let Some(eval_duration) = ollama_response.eval_duration {
            metadata.insert("eval_duration_ms".to_string(), serde_json::json!(eval_duration));
        }

        // Ollama doesn't provide token counts, so we have to estimate
        let prompt_tokens = (prompt.len() / 4) as u32; // Very rough estimate
        let completion_tokens = (ollama_response.response.len() / 4) as u32; // Very rough estimate
        
        let usage = TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };

        Ok(CompletionResponse {
            text: ollama_response.response,
            reached_max_tokens: false, // Ollama doesn't indicate this
            usage,
            metadata,
        })
    }

    async fn generate_text_streaming(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<Box<dyn CompletionStream>> {
        self.ensure_initialized()?;
        debug!("Generating text (streaming) with model '{}'", model_id);

        // Prepare the generation request with streaming enabled
        let mut ollama_options = HashMap::new();
        
        if let Some(max_tokens) = options.max_tokens {
            ollama_options.insert("num_predict".to_string(), serde_json::json!(max_tokens));
        }
        
        if let Some(temperature) = options.temperature {
            ollama_options.insert("temperature".to_string(), serde_json::json!(temperature));
        }
        
        if let Some(top_p) = options.top_p {
            ollama_options.insert("top_p".to_string(), serde_json::json!(top_p));
        }
        
        if let Some(top_k) = options.top_k {
            ollama_options.insert("top_k".to_string(), serde_json::json!(top_k));
        }
        
        if let Some(fp) = options.frequency_penalty {
            ollama_options.insert("frequency_penalty".to_string(), serde_json::json!(fp));
        }
        
        if let Some(pp) = options.presence_penalty {
            ollama_options.insert("presence_penalty".to_string(), serde_json::json!(pp));
        }
        
        if let Some(seed) = options.seed {
            ollama_options.insert("seed".to_string(), serde_json::json!(seed));
        }
        
        if let Some(stop_sequences) = &options.stop_sequences {
            if !stop_sequences.is_empty() {
                ollama_options.insert("stop".to_string(), serde_json::json!(stop_sequences));
            }
        }
        
        // Add any additional parameters
        for (key, value) in options.additional_params {
            ollama_options.insert(key, value);
        }

        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaGenerationRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options: if ollama_options.is_empty() { None } else { Some(ollama_options) },
        };

        // Send the request
        debug!("Sending streaming generation request to Ollama for model '{}'", model_id);
        let response = match self.client.post(&url).json(&request).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to generate streaming text with model '{}': {}", model_id, e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to generate text: {}", e
                )));
            }
        };

        if !response.status().is_success() {
            error!("Failed to generate streaming text with model '{}': HTTP {}", model_id, response.status());
            match response.status() {
                StatusCode::NOT_FOUND => {
                    return Err(LLMProviderError::ModelNotFound(model_id.to_string()));
                },
                StatusCode::BAD_REQUEST => {
                    // Try to extract error message from response
                    let error_text = match response.text().await {
                        Ok(text) => {
                            match serde_json::from_str::<serde_json::Value>(&text) {
                                Ok(json) => {
                                    json.get("error").and_then(|e| e.as_str())
                                        .unwrap_or(&text).to_string()
                                },
                                Err(_) => text,
                            }
                        },
                        Err(_) => "Bad request".to_string(),
                    };
                    return Err(LLMProviderError::GenerationFailed(error_text));
                },
                _ => {
                    return Err(LLMProviderError::ProviderError(format!(
                        "Failed to generate text: HTTP {}", response.status()
                    )));
                }
            }
        }

        // Create the streaming response handler
        debug!("Successfully created streaming response for model '{}'", model_id);
        Ok(Box::new(OllamaCompletionStream {
            response,
            accumulated_text: String::new(),
            completed: false,
        }))
    }

    async fn get_model_config(&self, model_id: &str) -> LLMProviderResult<ModelConfig> {
        self.ensure_initialized()?;
        debug!("Getting configuration for model '{}'", model_id);

        // Check if we have a cached config
        let config_map = self.model_configs.read().await;
        if let Some(config) = config_map.get(model_id) {
            return Ok(config.clone());
        }

        // Ollama doesn't have a direct API to get model config
        // We'll return a default config
        Ok(ModelConfig {
            id: model_id.to_string(),
            parameters: HashMap::new(),
        })
    }

    async fn update_model_config(&self, config: ModelConfig) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        debug!("Updating configuration for model '{}'", config.id);

        // Store the config in our cache
        let mut config_map = self.model_configs.write().await;
        config_map.insert(config.id.clone(), config);

        // Note: Ollama doesn't have a direct API to update model config
        // We're just storing it locally for now
        Ok(())
    }

    async fn get_model_info(&self, model_id: &str) -> LLMProviderResult<ModelInfo> {
        self.ensure_initialized()?;
        debug!("Getting information for model '{}'", model_id);

        // Get all models and find the one we want
        let models = self.list_available_models().await?;
        for model in models {
            if model.id == model_id {
                return Ok(model);
            }
        }

        error!("Model '{}' not found", model_id);
        Err(LLMProviderError::ModelNotFound(model_id.to_string()))
    }
}