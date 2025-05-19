use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::provider::{
    CompletionResponse, CompletionStream, DownloadStatus, GenerationOptions,
    LLMProviderError, LLMProviderResult, LocalLLMProvider, ModelConfig, ModelInfo, TokenUsage,
};

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

/// Ollama provider implementation
pub struct OllamaProvider {
    /// Base URL for the Ollama API
    base_url: String,
    /// HTTP client for API requests
    client: Client,
    /// Map of model IDs to download status
    download_status: Arc<RwLock<HashMap<String, DownloadStatus>>>,
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
            client: Client::new(),
            download_status: Arc::new(RwLock::new(HashMap::new())),
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
        
        if let Some(details) = ollama_info.details {
            provider_metadata.insert("format".to_string(), serde_json::to_value(details.format).unwrap_or_default());
            provider_metadata.insert("family".to_string(), serde_json::to_value(details.family).unwrap_or_default());
            
            if let Some(families) = details.families {
                provider_metadata.insert("families".to_string(), serde_json::to_value(families).unwrap_or_default());
            }
            
            if let Some(param_size) = details.parameter_size {
                provider_metadata.insert("parameter_size".to_string(), serde_json::to_value(param_size).unwrap_or_default());
            }
            
            if let Some(quant_level) = details.quantization_level {
                provider_metadata.insert("quantization_level".to_string(), serde_json::to_value(quant_level).unwrap_or_default());
            }
        }
        
        provider_metadata.insert("digest".to_string(), serde_json::to_value(ollama_info.digest).unwrap_or_default());
        provider_metadata.insert("modified_at".to_string(), serde_json::to_value(ollama_info.modified_at).unwrap_or_default());

        ModelInfo {
            id: ollama_info.name.clone(),
            name: ollama_info.name,
            description: "".to_string(), // Ollama API doesn't provide descriptions
            size_bytes: ollama_info.size,
            is_downloaded,
            provider_metadata,
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
            None => return Err(LLMProviderError::ProviderError(
                "Configuration must be a JSON object".to_string(),
            )),
        };

        // Override base URL if provided
        if let Some(base_url) = config_obj.get("base_url").and_then(|v| v.as_str()) {
            self.base_url = base_url.to_string();
        }

        // Test connection to Ollama API
        let health_url = format!("{}/api/health", self.base_url);
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(LLMProviderError::ProviderError(format!(
                        "Failed to connect to Ollama API: HTTP {}", response.status()
                    )));
                }
            },
            Err(e) => {
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to connect to Ollama API: {}", e
                )));
            }
        }

        // Mark as initialized
        let mut initialized = self.initialized.lock().map_err(|e| {
            LLMProviderError::Unexpected(format!("Failed to acquire lock: {}", e))
        })?;
        *initialized = true;

        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Ollama"
    }

    async fn list_available_models(&self) -> LLMProviderResult<Vec<ModelInfo>> {
        self.ensure_initialized()?;

        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| {
            LLMProviderError::NetworkError(format!("Failed to fetch models: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to fetch models: HTTP {}", response.status()
            )));
        }

        let models_response: OllamaListModelsResponse = response.json().await.map_err(|e| {
            LLMProviderError::ProviderError(format!("Failed to parse models response: {}", e))
        })?;

        let models = models_response.models.into_iter()
            .map(|m| self.convert_to_model_info(m, true))
            .collect();

        Ok(models)
    }

    async fn list_downloaded_models(&self) -> LLMProviderResult<Vec<ModelInfo>> {
        // For Ollama, all listed models are downloaded models
        self.list_available_models().await
    }

    async fn download_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;

        // Update download status
        {
            let mut status_map = self.download_status.write().await;
            status_map.insert(model_id.to_string(), DownloadStatus::InProgress { percent: 0.0 });
        }

        // Construct the pull request
        let url = format!("{}/api/pull", self.base_url);
        let request = serde_json::json!({
            "name": model_id,
            "stream": true,
        });

        // Start download in a separate task
        let model_id = model_id.to_string();
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let download_status = self.download_status.clone();

        tokio::spawn(async move {
            let result = client.post(&url).json(&request).send().await;
            
            match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        let mut status_map = download_status.write().await;
                        status_map.insert(model_id, DownloadStatus::Failed { 
                            reason: format!("HTTP error: {}", response.status()) 
                        });
                        return;
                    }

                    // Stream the response to track progress
                    let mut stream = response.bytes_stream();
                    while let Some(chunk_result) = futures_util::StreamExt::next(&mut stream).await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Ok(text) = std::str::from_utf8(&chunk) {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                                        // Extract progress information if available
                                        if let Some(completed) = json.get("completed").and_then(|v| v.as_f64()) {
                                            if let Some(total) = json.get("total").and_then(|v| v.as_f64()) {
                                                if total > 0.0 {
                                                    let percent = (completed / total * 100.0) as f32;
                                                    let mut status_map = download_status.write().await;
                                                    status_map.insert(model_id.clone(), DownloadStatus::InProgress { percent });
                                                }
                                            }
                                        }
                                        
                                        // Check for completion
                                        if json.get("status").and_then(|v| v.as_str()) == Some("success") {
                                            let mut status_map = download_status.write().await;
                                            status_map.insert(model_id.clone(), DownloadStatus::Completed);
                                            break;
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                let mut status_map = download_status.write().await;
                                status_map.insert(model_id.clone(), DownloadStatus::Failed { 
                                    reason: format!("Failed to read response: {}", e) 
                                });
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    let mut status_map = download_status.write().await;
                    status_map.insert(model_id, DownloadStatus::Failed { 
                        reason: format!("Network error: {}", e) 
                    });
                }
            }
        });

        Ok(())
    }

    async fn get_download_status(&self, model_id: &str) -> LLMProviderResult<DownloadStatus> {
        self.ensure_initialized()?;

        let status_map = self.download_status.read().await;
        match status_map.get(model_id) {
            Some(status) => Ok(status.clone()),
            None => {
                // If no status is recorded, check if the model exists already
                let models = self.list_downloaded_models().await?;
                if models.iter().any(|m| m.id == model_id) {
                    Ok(DownloadStatus::Completed)
                } else {
                    Ok(DownloadStatus::NotStarted)
                }
            }
        }
    }

    async fn cancel_download(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;

        // Ollama doesn't have a direct API to cancel downloads
        // We can only update our status to indicate cancellation
        let mut status_map = self.download_status.write().await;
        if let Some(status) = status_map.get(model_id) {
            match status {
                DownloadStatus::InProgress { .. } => {
                    status_map.insert(model_id.to_string(), DownloadStatus::Cancelled);
                    Ok(())
                },
                _ => Err(LLMProviderError::ProviderError(
                    "Model is not currently downloading".to_string()
                )),
            }
        } else {
            Err(LLMProviderError::ModelNotFound(model_id.to_string()))
        }
    }

    async fn delete_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;

        let url = format!("{}/api/delete", self.base_url);
        let request = serde_json::json!({
            "name": model_id,
        });

        let response = self.client.delete(&url).json(&request).send().await.map_err(|e| {
            LLMProviderError::NetworkError(format!("Failed to delete model: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to delete model: HTTP {}", response.status()
            )));
        }

        // Remove status and config for the deleted model
        {
            let mut status_map = self.download_status.write().await;
            status_map.remove(model_id);
        }
        {
            let mut config_map = self.model_configs.write().await;
            config_map.remove(model_id);
        }

        Ok(())
    }

    async fn is_model_loaded(&self, model_id: &str) -> LLMProviderResult<bool> {
        self.ensure_initialized()?;

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
                if response.status().is_success() {
                    Ok(true)
                } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                    Ok(false)
                } else {
                    Err(LLMProviderError::ProviderError(format!(
                        "Failed to check model status: HTTP {}", response.status()
                    )))
                }
            },
            Err(e) => Err(LLMProviderError::NetworkError(format!(
                "Failed to check model status: {}", e
            ))),
        }
    }

    async fn load_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;

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

        let response = self.client.post(&url).json(&request).send().await.map_err(|e| {
            LLMProviderError::NetworkError(format!("Failed to load model: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to load model: HTTP {}", response.status()
            )));
        }

        Ok(())
    }

    async fn unload_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        
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
        
        // Add any additional parameters
        for (key, value) in options.additional_params {
            ollama_options.insert(key, value);
        }

        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaGenerationRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(ollama_options),
        };

        // Send the request
        let response = self.client.post(&url).json(&request).send().await.map_err(|e| {
            LLMProviderError::NetworkError(format!("Failed to generate text: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to generate text: HTTP {}", response.status()
            )));
        }

        // Parse the response
        let ollama_response: OllamaGenerationResponse = response.json().await.map_err(|e| {
            LLMProviderError::ProviderError(format!("Failed to parse generation response: {}", e))
        })?;

        // Create the completion response
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

        let usage = TokenUsage {
            // Ollama doesn't provide token counts, so we have to estimate
            prompt_tokens: (prompt.len() / 4) as u32, // Very rough estimate
            completion_tokens: (ollama_response.response.len() / 4) as u32, // Very rough estimate
            total_tokens: ((prompt.len() + ollama_response.response.len()) / 4) as u32,
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
        
        // Add any additional parameters
        for (key, value) in options.additional_params {
            ollama_options.insert(key, value);
        }

        let url = format!("{}/api/generate", self.base_url);
        let request = OllamaGenerationRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options: Some(ollama_options),
        };

        // Send the request
        let response = self.client.post(&url).json(&request).send().await.map_err(|e| {
            LLMProviderError::NetworkError(format!("Failed to generate text: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to generate text: HTTP {}", response.status()
            )));
        }

        // Create the streaming response handler
        Ok(Box::new(OllamaCompletionStream {
            response,
            accumulated_text: String::new(),
            completed: false,
        }))
    }

    async fn get_model_config(&self, model_id: &str) -> LLMProviderResult<ModelConfig> {
        self.ensure_initialized()?;

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

        // Store the config in our cache
        let mut config_map = self.model_configs.write().await;
        config_map.insert(config.id.clone(), config);

        // Note: Ollama doesn't have a direct API to update model config
        // We're just storing it locally for now
        Ok(())
    }

    async fn get_model_info(&self, model_id: &str) -> LLMProviderResult<ModelInfo> {
        self.ensure_initialized()?;

        // Get all models and find the one we want
        let models = self.list_available_models().await?;
        for model in models {
            if model.id == model_id {
                return Ok(model);
            }
        }

        Err(LLMProviderError::ModelNotFound(model_id.to_string()))
    }
}