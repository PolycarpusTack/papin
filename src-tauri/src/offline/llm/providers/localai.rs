use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use chrono::Utc;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use reqwest::{Client, StatusCode, header};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::offline::llm::provider::{
    CompletionResponse, CompletionStream, LocalLLMProvider,
    LLMProviderError, LLMProviderResult, ModelConfig, ModelInfo, TokenUsage,
};
use crate::offline::llm::types::{DownloadStatus, GenerationOptions, ProviderType};

// LocalAI API - OpenAI compatibility types

/// OpenAI compatible model object
#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
    #[serde(default)]
    permission: Vec<OpenAIPermission>,
    root: Option<String>,
    parent: Option<String>,
}

/// OpenAI compatible permission object
#[derive(Debug, Deserialize)]
struct OpenAIPermission {
    id: String,
    object: String,
    created: u64,
    allow_create_engine: bool,
    allow_sampling: bool,
    allow_logprobs: bool,
    allow_search_indices: bool,
    allow_view: bool,
    allow_fine_tuning: bool,
    organization: String,
    group: Option<String>,
    is_blocking: bool,
}

/// OpenAI compatible model list response
#[derive(Debug, Deserialize)]
struct OpenAIModelList {
    object: String,
    data: Vec<OpenAIModel>,
}

/// OpenAI compatible completion request
#[derive(Debug, Serialize)]
struct OpenAICompletionRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    best_of: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

/// OpenAI compatible chat message
#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIChatMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// OpenAI compatible chat completion request
#[derive(Debug, Serialize)]
struct OpenAIChatCompletionRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

/// OpenAI compatible chat completion response
#[derive(Debug, Deserialize)]
struct OpenAIChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChatCompletionChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI compatible chat completion choice
#[derive(Debug, Deserialize)]
struct OpenAIChatCompletionChoice {
    index: u32,
    message: OpenAIChatMessage,
    finish_reason: Option<String>,
}

/// OpenAI compatible completion response
#[derive(Debug, Deserialize)]
struct OpenAICompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAICompletionChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI compatible completion choice
#[derive(Debug, Deserialize)]
struct OpenAICompletionChoice {
    text: String,
    index: u32,
    logprobs: Option<serde_json::Value>,
    finish_reason: Option<String>,
}

/// OpenAI compatible usage information
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI compatible chat completion delta
#[derive(Debug, Deserialize)]
struct OpenAIChatCompletionDelta {
    content: Option<String>,
    role: Option<String>,
}

/// OpenAI compatible streaming chat completion chunk
#[derive(Debug, Deserialize)]
struct OpenAIChatCompletionChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChatCompletionStreamChoice>,
}

/// OpenAI compatible streaming chat completion choice
#[derive(Debug, Deserialize)]
struct OpenAIChatCompletionStreamChoice {
    index: u32,
    delta: OpenAIChatCompletionDelta,
    finish_reason: Option<String>,
}

/// OpenAI compatible streaming completion chunk
#[derive(Debug, Deserialize)]
struct OpenAICompletionChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAICompletionStreamChoice>,
}

/// OpenAI compatible streaming completion choice
#[derive(Debug, Deserialize)]
struct OpenAICompletionStreamChoice {
    text: String,
    index: u32,
    logprobs: Option<serde_json::Value>,
    finish_reason: Option<String>,
}

/// LocalAI-specific API types

/// LocalAI model pull request
#[derive(Debug, Serialize)]
struct LocalAIPullRequest {
    name: String,
}

/// LocalAI model pull response
#[derive(Debug, Deserialize)]
struct LocalAIPullResponse {
    status: String,
    error: Option<String>,
}

/// LocalAI model gallery item
#[derive(Debug, Deserialize)]
struct LocalAIGalleryModel {
    name: String,
    description: Option<String>,
    license: Option<String>,
    urls: Vec<String>,
    tags: Option<Vec<String>>,
    parameters: Option<HashMap<String, serde_json::Value>>,
}

/// LocalAI model gallery response
#[derive(Debug, Deserialize)]
struct LocalAIGalleryResponse {
    models: Vec<LocalAIGalleryModel>,
}

/// LocalAI health endpoint response
#[derive(Debug, Deserialize)]
struct LocalAIHealthResponse {
    status: String,
    version: Option<String>,
}

/// Download statistics for models
#[derive(Debug, Clone)]
struct ModelDownloadStats {
    start_time: chrono::DateTime<Utc>,
    model_id: String,
    status: DownloadStatus,
    // For non-standard providers like LocalAI we don't have download progress 
    // info, so we use a simple status tracking system
}

impl ModelDownloadStats {
    fn new(model_id: &str) -> Self {
        Self {
            start_time: Utc::now(),
            model_id: model_id.to_string(),
            status: DownloadStatus::InProgress { 
                percent: 0.0,
                bytes_downloaded: None,
                total_bytes: None,
                eta_seconds: None,
                bytes_per_second: None,
            },
        }
    }
    
    fn set_completed(&mut self) {
        let duration = Utc::now() - self.start_time;
        self.status = DownloadStatus::Completed { 
            completed_at: Some(Utc::now().to_rfc3339()),
            duration_seconds: Some(duration.num_milliseconds() as f32 / 1000.0),
        };
    }
    
    fn set_failed(&mut self, reason: &str) {
        self.status = DownloadStatus::Failed { 
            reason: reason.to_string(),
            error_code: None,
            failed_at: Some(Utc::now().to_rfc3339()),
        };
    }
    
    fn set_cancelled(&mut self) {
        self.status = DownloadStatus::Cancelled { 
            cancelled_at: Some(Utc::now().to_rfc3339()),
        };
    }
}

/// LocalAI provider implementation
pub struct LocalAIProvider {
    /// Base URL for the LocalAI API
    base_url: String,
    /// HTTP client for API requests
    client: Client,
    /// API key (if required)
    api_key: Option<String>,
    /// Map of model IDs to download status
    download_status: Arc<RwLock<HashMap<String, DownloadStatus>>>,
    /// Map of model IDs to download statistics
    download_stats: Arc<RwLock<HashMap<String, ModelDownloadStats>>>,
    /// Map of model IDs to model configurations
    model_configs: Arc<RwLock<HashMap<String, ModelConfig>>>,
    /// Whether the provider has been initialized
    initialized: Arc<Mutex<bool>>,
    /// Model capabilities from gallery info
    model_capabilities: Arc<RwLock<HashMap<String, HashMap<String, bool>>>>,
}

impl LocalAIProvider {
    /// Create a new LocalAI provider with the default URL
    pub fn new() -> Self {
        Self::with_base_url("http://localhost:8080")
    }

    /// Create a new LocalAI provider with a custom URL
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key: None,
            download_status: Arc::new(RwLock::new(HashMap::new())),
            download_stats: Arc::new(RwLock::new(HashMap::new())),
            model_configs: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
            model_capabilities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if the provider is initialized
    fn ensure_initialized(&self) -> LLMProviderResult<()> {
        let initialized = *self.initialized.lock().map_err(|e| {
            LLMProviderError::Unexpected(format!("Failed to acquire lock: {}", e))
        })?;

        if !initialized {
            return Err(LLMProviderError::NotInitialized(
                "LocalAI provider not initialized. Call initialize() first.".to_string(),
            ));
        }

        Ok(())
    }
    
    /// Create a request builder with appropriate headers including API key if set
    fn create_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.request(method, url);
        
        // Add API key if available
        if let Some(api_key) = &self.api_key {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
        }
        
        builder
    }

    /// Convert OpenAI model info to the common ModelInfo format
    fn convert_to_model_info(&self, openai_model: OpenAIModel) -> ModelInfo {
        let mut provider_metadata = HashMap::new();
        let model_id = openai_model.id.clone();
        
        provider_metadata.insert("object".to_string(), serde_json::to_value(openai_model.object).unwrap_or_default());
        provider_metadata.insert("created".to_string(), serde_json::to_value(openai_model.created).unwrap_or_default());
        provider_metadata.insert("owned_by".to_string(), serde_json::to_value(openai_model.owned_by).unwrap_or_default());
        
        if let Some(root) = openai_model.root {
            provider_metadata.insert("root".to_string(), serde_json::to_value(root).unwrap_or_default());
        }
        
        if let Some(parent) = openai_model.parent {
            provider_metadata.insert("parent".to_string(), serde_json::to_value(parent).unwrap_or_default());
        }
        
        // Create tags from model name components
        let tags = model_id.split(':')
            .skip(1)  // Skip the base name
            .filter(|&s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Extract base name without tags
        let base_name = model_id.split(':').next().unwrap_or(&model_id).to_string();
        
        // Determine capabilities from our stored capabilities map
        let mut supports_text_generation = true;  // Assume text generation by default
        let mut supports_completion = true;       // Assume completion by default
        let mut supports_chat = true;             // Assume chat by default
        let mut supports_embeddings = false;      // Don't assume embeddings by default
        let mut supports_image_generation = false; // Don't assume image generation by default
        
        // Look up capabilities if we have them
        if let Ok(capabilities_map) = self.model_capabilities.read() {
            if let Some(capabilities) = capabilities_map.get(&model_id) {
                supports_text_generation = *capabilities.get("text_generation").unwrap_or(&true);
                supports_completion = *capabilities.get("completion").unwrap_or(&true);
                supports_chat = *capabilities.get("chat").unwrap_or(&true);
                supports_embeddings = *capabilities.get("embeddings").unwrap_or(&false);
                supports_image_generation = *capabilities.get("image_generation").unwrap_or(&false);
            }
        }
        
        // Try to infer parameter count from model name
        let param_count = model_id.to_lowercase()
            .replace(&base_name.to_lowercase(), "")
            .replace(':', "")
            .replace('-', "")
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .find(|c: char| c.is_numeric())
            .and_then(|_| {
                // Extract numeric part that might represent parameters
                let numeric_part = model_id.to_lowercase()
                    .chars()
                    .filter(|&c| c.is_digit(10) || c == '.')
                    .collect::<String>();
                
                // Check if it has 'b' suffix for billions
                if model_id.to_lowercase().contains('b') {
                    numeric_part.parse::<f64>().ok()
                } else {
                    None
                }
            });

        // Create model info
        ModelInfo {
            id: model_id.clone(),
            name: base_name,
            description: format!("LocalAI model{}", 
                if let Some(params) = param_count {
                    format!(" ({:.1}B parameters)", params)
                } else {
                    "".to_string()
                }
            ),
            size_bytes: 0, // LocalAI API doesn't provide size info
            is_downloaded: true, // If it's listed, it's available
            provider_metadata,
            provider: ProviderType::LocalAI,
            supports_text_generation,
            supports_completion,
            supports_chat,
            supports_embeddings,
            supports_image_generation,
            quantization: None, // LocalAI API doesn't provide quantization info
            parameter_count_b: param_count,
            context_length: None, // LocalAI API doesn't provide context window info
            model_family: None, // LocalAI API doesn't provide family info
            created_at: Some(chrono::DateTime::from_timestamp(openai_model.created as i64, 0)
                .map_or_else(
                    || Utc::now().to_rfc3339(),
                    |dt| dt.to_rfc3339()
                )),
            tags,
            license: None, // LocalAI API doesn't provide license info directly
        }
    }
    
    /// Fetch model gallery information to determine capabilities
    async fn fetch_model_gallery(&self) -> LLMProviderResult<()> {
        debug!("Fetching model gallery from LocalAI");
        
        let url = format!("{}/api/models/available", self.base_url);
        let response = match self.create_request(reqwest::Method::GET, &url).send().await {
            Ok(response) => response,
            Err(e) => {
                // This is non-critical, so just log and return
                warn!("Failed to fetch model gallery: {}. Some model capabilities may not be detected.", e);
                return Ok(());
            }
        };
        
        if !response.status().is_success() {
            warn!("Failed to fetch model gallery: HTTP {}. Some model capabilities may not be detected.", 
                response.status());
            return Ok(());
        }
        
        // Parse the response
        match response.json::<LocalAIGalleryResponse>().await {
            Ok(gallery) => {
                let mut capabilities_map = self.model_capabilities.write().await;
                
                for model in gallery.models {
                    let mut capabilities = HashMap::new();
                    
                    // Process tags to determine capabilities
                    if let Some(tags) = model.tags {
                        // Check for specific capabilities in tags
                        capabilities.insert("text_generation".to_string(), 
                            tags.iter().any(|t| t.contains("text") || t.contains("llm")));
                        capabilities.insert("chat".to_string(), 
                            tags.iter().any(|t| t.contains("chat")));
                        capabilities.insert("embeddings".to_string(), 
                            tags.iter().any(|t| t.contains("embed")));
                        capabilities.insert("image_generation".to_string(), 
                            tags.iter().any(|t| t.contains("image") || t.contains("diffusion")));
                    }
                    
                    capabilities_map.insert(model.name, capabilities);
                }
                
                debug!("Successfully fetched model gallery with {} models", gallery.models.len());
            },
            Err(e) => {
                warn!("Failed to parse model gallery: {}. Some model capabilities may not be detected.", e);
            }
        }
        
        Ok(())
    }
    
    /// Set download status for a model
    async fn set_download_status(&self, model_id: &str, status: DownloadStatus) {
        let mut status_map = self.download_status.write().await;
        status_map.insert(model_id.to_string(), status);
    }
}

/// Streaming completion implementation for LocalAI using chat completions
pub struct LocalAIChatCompletionStream {
    /// HTTP response stream
    response: reqwest::Response,
    /// Total text accumulated so far
    accumulated_text: String,
    /// Whether we've reached the end of the stream
    completed: bool,
}

impl LocalAIChatCompletionStream {
    /// Parse a chunk from the event stream
    fn parse_chunk(&mut self, chunk: &str) -> LLMProviderResult<Option<String>> {
        // SSE format: "data: {...}\n\n"
        let data = chunk.trim().strip_prefix("data: ").unwrap_or(chunk);
        
        // Check for the "[DONE]" marker
        if data == "[DONE]" {
            self.completed = true;
            return Ok(None);
        }
        
        // Parse the JSON
        match serde_json::from_str::<OpenAIChatCompletionChunk>(data) {
            Ok(chunk_data) => {
                // Extract the text content
                if let Some(choice) = chunk_data.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        self.accumulated_text.push_str(content);
                        return Ok(Some(content.clone()));
                    }
                    
                    // Check for finish reason
                    if choice.finish_reason.is_some() {
                        self.completed = true;
                    }
                }
                
                Ok(None)
            },
            Err(e) => {
                // If we can't parse as chat completion, try as regular completion
                match serde_json::from_str::<OpenAICompletionChunk>(data) {
                    Ok(completion_chunk) => {
                        if let Some(choice) = completion_chunk.choices.first() {
                            self.accumulated_text.push_str(&choice.text);
                            
                            // Check for finish reason
                            if choice.finish_reason.is_some() {
                                self.completed = true;
                            }
                            
                            return Ok(Some(choice.text.clone()));
                        }
                        
                        Ok(None)
                    },
                    Err(e2) => {
                        warn!("Failed to parse chunk as chat or completion: {} / {}", e, e2);
                        Err(LLMProviderError::Unexpected(format!(
                            "Failed to parse streaming response: {}", e
                        )))
                    }
                }
            }
        }
    }
}

#[async_trait]
impl CompletionStream for LocalAIChatCompletionStream {
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
                
                match self.parse_chunk(chunk_str) {
                    Ok(Some(text)) => Some(Ok(text)),
                    Ok(None) => self.next_chunk().await, // No content in this chunk, try next
                    Err(e) => Some(Err(e)),
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
impl LocalLLMProvider for LocalAIProvider {
    async fn initialize(&mut self, config: serde_json::Value) -> LLMProviderResult<()> {
        // Extract configuration options
        let config_obj = match config.as_object() {
            Some(obj) => obj,
            None => {
                warn!("LocalAI provider configuration is not a JSON object");
                return Err(LLMProviderError::ProviderError(
                    "Configuration must be a JSON object".to_string(),
                ));
            }
        };

        // Override base URL if provided
        if let Some(base_url) = config_obj.get("base_url").and_then(|v| v.as_str()) {
            debug!("Setting LocalAI base URL to: {}", base_url);
            self.base_url = base_url.to_string();
        }
        
        // Set API key if provided
        if let Some(api_key) = config_obj.get("api_key").and_then(|v| v.as_str()) {
            debug!("Setting LocalAI API key");
            self.api_key = Some(api_key.to_string());
        }

        // Test connection to LocalAI API
        info!("Testing connection to LocalAI API at {}", self.base_url);
        let health_url = format!("{}/health", self.base_url);
        
        match self.create_request(reqwest::Method::GET, &health_url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    error!("Failed to connect to LocalAI API: HTTP {}", response.status());
                    return Err(LLMProviderError::ProviderError(format!(
                        "Failed to connect to LocalAI API: HTTP {}", response.status()
                    )));
                }
                
                // Try to parse the health response
                match response.json::<LocalAIHealthResponse>().await {
                    Ok(health_data) => {
                        if let Some(version) = health_data.version {
                            info!("Connected to LocalAI (version: {})", version);
                        } else {
                            info!("Connected to LocalAI (unknown version)");
                        }
                    },
                    Err(e) => {
                        warn!("Connected to LocalAI but couldn't parse health info: {}", e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to connect to LocalAI API: {}", e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to connect to LocalAI API: {}. Make sure LocalAI is running at {}.", 
                    e, self.base_url
                )));
            }
        }
        
        // Fetch model gallery to determine capabilities
        self.fetch_model_gallery().await?;

        // Mark as initialized
        match self.initialized.lock() {
            Ok(mut initialized) => {
                *initialized = true;
                info!("LocalAI provider initialized successfully");
            },
            Err(e) => {
                error!("Failed to acquire lock when initializing LocalAI provider: {}", e);
                return Err(LLMProviderError::Unexpected(format!(
                    "Failed to acquire lock: {}", e
                )));
            }
        }

        Ok(())
    }

    fn provider_name(&self) -> &str {
        "LocalAI"
    }

    async fn list_available_models(&self) -> LLMProviderResult<Vec<ModelInfo>> {
        self.ensure_initialized()?;
        debug!("Listing available models from LocalAI");

        let url = format!("{}/v1/models", self.base_url);
        let response = match self.create_request(reqwest::Method::GET, &url).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to fetch models from LocalAI: {}", e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to fetch models: {}", e
                )));
            }
        };

        if !response.status().is_success() {
            error!("Failed to fetch models from LocalAI: HTTP {}", response.status());
            return Err(LLMProviderError::ProviderError(format!(
                "Failed to fetch models: HTTP {}", response.status()
            )));
        }

        // Parse the response
        let models_response: OpenAIModelList = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse models response from LocalAI: {}", e);
                return Err(LLMProviderError::ProviderError(format!(
                    "Failed to parse models response: {}", e
                )));
            }
        };

        // Convert to our model format
        let models = models_response.data.into_iter()
            .map(|m| self.convert_to_model_info(m))
            .collect::<Vec<_>>();

        info!("Found {} models from LocalAI", models.len());
        Ok(models)
    }

    async fn list_downloaded_models(&self) -> LLMProviderResult<Vec<ModelInfo>> {
        // For LocalAI, all listed models are already downloaded and ready to use
        self.list_available_models().await
    }

    async fn download_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        info!("Starting download of model '{}' from LocalAI", model_id);

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
            stats_map.insert(model_id.to_string(), ModelDownloadStats::new(model_id));
        }

        // Construct the pull request
        let url = format!("{}/api/models/apply", self.base_url);
        let request = LocalAIPullRequest {
            name: model_id.to_string(),
        };

        // Start download in a separate task
        let model_id = model_id.to_string();
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let download_status = self.download_status.clone();
        let download_stats = self.download_stats.clone();

        tokio::spawn(async move {
            let mut request_builder = client.post(&url);
            
            // Add API key if available
            if let Some(api_key) = &api_key {
                request_builder = request_builder.header(header::AUTHORIZATION, format!("Bearer {}", api_key));
            }
            
            // Send the request
            let result = request_builder.json(&request).send().await;
            
            match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        error!("Failed to download model '{}': HTTP {}", model_id, response.status());
                        
                        // Update status to failed
                        let mut stats_map = download_stats.write().await;
                        if let Some(stats) = stats_map.get_mut(&model_id) {
                            stats.set_failed(&format!("HTTP error: {}", response.status()));
                            // Update the download status
                            let mut status_map = download_status.write().await;
                            status_map.insert(model_id.clone(), stats.status.clone());
                        }
                        return;
                    }

                    // Parse the response
                    match response.json::<LocalAIPullResponse>().await {
                        Ok(pull_response) => {
                            if pull_response.status == "success" {
                                info!("Successfully downloaded model '{}'", model_id);
                                
                                // Update status to completed
                                let mut stats_map = download_stats.write().await;
                                if let Some(stats) = stats_map.get_mut(&model_id) {
                                    stats.set_completed();
                                    // Update the download status
                                    let mut status_map = download_status.write().await;
                                    status_map.insert(model_id.clone(), stats.status.clone());
                                }
                            } else {
                                let error_msg = pull_response.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Error downloading model '{}': {}", model_id, error_msg);
                                
                                // Update status to failed
                                let mut stats_map = download_stats.write().await;
                                if let Some(stats) = stats_map.get_mut(&model_id) {
                                    stats.set_failed(&error_msg);
                                    // Update the download status
                                    let mut status_map = download_status.write().await;
                                    status_map.insert(model_id.clone(), stats.status.clone());
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to parse pull response for model '{}': {}", model_id, e);
                            
                            // Update status to failed
                            let mut stats_map = download_stats.write().await;
                            if let Some(stats) = stats_map.get_mut(&model_id) {
                                stats.set_failed(&format!("Failed to parse response: {}", e));
                                // Update the download status
                                let mut status_map = download_status.write().await;
                                status_map.insert(model_id.clone(), stats.status.clone());
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Network error while downloading model '{}': {}", model_id, e);
                    
                    // Update status to failed
                    let mut stats_map = download_stats.write().await;
                    if let Some(stats) = stats_map.get_mut(&model_id) {
                        stats.set_failed(&format!("Network error: {}", e));
                        // Update the download status
                        let mut status_map = download_status.write().await;
                        status_map.insert(model_id.clone(), stats.status.clone());
                    }
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

        // LocalAI doesn't have a direct API to cancel downloads
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
                        
                        // Update stats
                        let mut stats_map = self.download_stats.write().await;
                        if let Some(stats) = stats_map.get_mut(model_id) {
                            stats.set_cancelled();
                        }
                        
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

        // LocalAI doesn't have a convenient API to delete models
        // We'll try the undocumented endpoint
        let url = format!("{}/api/models/delete", self.base_url);
        let request = serde_json::json!({
            "name": model_id,
        });

        let response = match self.create_request(reqwest::Method::POST, &url).json(&request).send().await {
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
                
                // Try to parse error message
                let error_text = match response.text().await {
                    Ok(text) => text,
                    Err(_) => format!("HTTP {}", response.status()),
                };
                
                Err(LLMProviderError::ProviderError(format!(
                    "Failed to delete model: {}", error_text
                )))
            }
        }
    }

    async fn is_model_loaded(&self, model_id: &str) -> LLMProviderResult<bool> {
        self.ensure_initialized()?;
        debug!("Checking if model '{}' is loaded", model_id);

        // LocalAI doesn't explicitly distinguish between available and loaded models
        // If a model is listed, we'll assume it's loaded or can be loaded immediately
        let models = self.list_available_models().await?;
        Ok(models.iter().any(|m| m.id == model_id))
    }

    async fn load_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        info!("Loading model '{}'", model_id);

        // LocalAI automatically loads models when used
        // Check if the model exists
        if self.is_model_loaded(model_id).await? {
            Ok(())
        } else {
            Err(LLMProviderError::ModelNotFound(model_id.to_string()))
        }
    }

    async fn unload_model(&self, model_id: &str) -> LLMProviderResult<()> {
        self.ensure_initialized()?;
        debug!("Unloading model '{}' (no-op in LocalAI)", model_id);
        
        // LocalAI doesn't have a direct API to unload models
        // Models are automatically unloaded when not used
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

        // Check if we should use chat completions or text completions
        // Try chat completions first, then fall back to text completions if needed
        match self.generate_chat_completion(model_id, prompt, options.clone()).await {
            Ok(completion) => Ok(completion),
            Err(e) => {
                debug!("Chat completion failed: {}. Trying text completion...", e);
                self.generate_text_completion(model_id, prompt, options).await
            }
        }
    }
    
    async fn generate_text_streaming(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<Box<dyn CompletionStream>> {
        self.ensure_initialized()?;
        debug!("Generating streaming text with model '{}'", model_id);

        // Prepare the chat request with streaming enabled
        let mut messages = Vec::new();
        messages.push(OpenAIChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            name: None,
        });
        
        let request = OpenAIChatCompletionRequest {
            model: model_id.to_string(),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stop: options.stop_sequences,
            n: Some(1),
            stream: Some(true),
            seed: options.seed,
            user: None,
        };
        
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        // Send the request
        let response = match self.create_request(reqwest::Method::POST, &url)
            .json(&request)
            .send()
            .await 
        {
            Ok(response) => {
                if !response.status().is_success() {
                    let error_text = match response.text().await {
                        Ok(text) => text,
                        Err(_) => format!("HTTP {}", response.status()),
                    };
                    
                    error!("Failed to generate streaming chat completion: {}", error_text);
                    return Err(LLMProviderError::GenerationFailed(error_text));
                }
                
                response
            },
            Err(e) => {
                error!("Network error while generating streaming chat completion: {}", e);
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to generate text: {}", e
                )));
            }
        };
        
        // Create streaming handler
        Ok(Box::new(LocalAIChatCompletionStream {
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

        // LocalAI doesn't have a direct API to get model config
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

        // LocalAI doesn't have a direct API to update model config
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

impl LocalAIProvider {
    /// Generate text using the chat completions endpoint
    async fn generate_chat_completion(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<CompletionResponse> {
        debug!("Generating chat completion with model '{}'", model_id);
        
        // Prepare the chat request
        let mut messages = Vec::new();
        messages.push(OpenAIChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            name: None,
        });
        
        let request = OpenAIChatCompletionRequest {
            model: model_id.to_string(),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stop: options.stop_sequences,
            n: Some(1),
            stream: None,
            seed: options.seed,
            user: None,
        };
        
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        // Send the request
        let response = match self.create_request(reqwest::Method::POST, &url)
            .json(&request)
            .send()
            .await 
        {
            Ok(response) => {
                if !response.status().is_success() {
                    let error_text = match response.text().await {
                        Ok(text) => text,
                        Err(_) => format!("HTTP {}", response.status()),
                    };
                    
                    return Err(LLMProviderError::GenerationFailed(error_text));
                }
                
                response
            },
            Err(e) => {
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to generate text: {}", e
                )));
            }
        };
        
        // Parse the response
        let chat_response: OpenAIChatCompletionResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                return Err(LLMProviderError::ProviderError(format!(
                    "Failed to parse generation response: {}", e
                )));
            }
        };
        
        // Extract the completion text
        let choice = match chat_response.choices.first() {
            Some(choice) => choice,
            None => {
                return Err(LLMProviderError::ProviderError(
                    "No completion choices returned".to_string()
                ));
            }
        };
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::to_value(chat_response.id).unwrap_or_default());
        metadata.insert("object".to_string(), serde_json::to_value(chat_response.object).unwrap_or_default());
        metadata.insert("created".to_string(), serde_json::to_value(chat_response.created).unwrap_or_default());
        
        if let Some(finish_reason) = &choice.finish_reason {
            metadata.insert("finish_reason".to_string(), serde_json::to_value(finish_reason).unwrap_or_default());
        }
        
        // Create usage info
        let usage = chat_response.usage.unwrap_or(OpenAIUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });
        
        // Create the completion response
        Ok(CompletionResponse {
            text: choice.message.content.clone(),
            reached_max_tokens: choice.finish_reason.as_deref() == Some("length"),
            usage: TokenUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
            metadata,
        })
    }
    
    /// Generate text using the completions endpoint
    async fn generate_text_completion(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
    ) -> LLMProviderResult<CompletionResponse> {
        debug!("Generating text completion with model '{}'", model_id);
        
        // Prepare the completion request
        let request = OpenAICompletionRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stop: options.stop_sequences,
            n: Some(1),
            logprobs: None,
            best_of: None,
            echo: None,
            seed: options.seed,
            stream: None,
            user: None,
        };
        
        let url = format!("{}/v1/completions", self.base_url);
        
        // Send the request
        let response = match self.create_request(reqwest::Method::POST, &url)
            .json(&request)
            .send()
            .await 
        {
            Ok(response) => {
                if !response.status().is_success() {
                    let error_text = match response.text().await {
                        Ok(text) => text,
                        Err(_) => format!("HTTP {}", response.status()),
                    };
                    
                    return Err(LLMProviderError::GenerationFailed(error_text));
                }
                
                response
            },
            Err(e) => {
                return Err(LLMProviderError::NetworkError(format!(
                    "Failed to generate text: {}", e
                )));
            }
        };
        
        // Parse the response
        let completion_response: OpenAICompletionResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                return Err(LLMProviderError::ProviderError(format!(
                    "Failed to parse generation response: {}", e
                )));
            }
        };
        
        // Extract the completion text
        let choice = match completion_response.choices.first() {
            Some(choice) => choice,
            None => {
                return Err(LLMProviderError::ProviderError(
                    "No completion choices returned".to_string()
                ));
            }
        };
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), serde_json::to_value(completion_response.id).unwrap_or_default());
        metadata.insert("object".to_string(), serde_json::to_value(completion_response.object).unwrap_or_default());
        metadata.insert("created".to_string(), serde_json::to_value(completion_response.created).unwrap_or_default());
        
        if let Some(finish_reason) = &choice.finish_reason {
            metadata.insert("finish_reason".to_string(), serde_json::to_value(finish_reason).unwrap_or_default());
        }
        
        if let Some(logprobs) = &choice.logprobs {
            metadata.insert("logprobs".to_string(), logprobs.clone());
        }
        
        // Create usage info
        let usage = completion_response.usage.unwrap_or(OpenAIUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });
        
        // Create the completion response
        Ok(CompletionResponse {
            text: choice.text.clone(),
            reached_max_tokens: choice.finish_reason.as_deref() == Some("length"),
            usage: TokenUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
            metadata,
        })
    }
}