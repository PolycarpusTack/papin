// src/offline/llm/providers/localai.rs
//! LocalAI provider implementation for local LLM integration

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use log::{debug, error, info, warn};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::offline::llm::provider::{
    DownloadStatus, GenerationOptions, ModelInfo, Provider, ProviderError, ProviderType, Result,
};

/// Configuration for LocalAI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAIConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// Timeout for API requests in seconds
    pub timeout_seconds: u64,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    /// API key (if required)
    pub api_key: Option<String>,
    /// Custom headers to add to requests
    pub headers: HashMap<String, String>,
}

impl Default for LocalAIConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8080".to_string(),
            timeout_seconds: 30,
            verify_ssl: true,
            api_key: None,
            headers: HashMap::new(),
        }
    }
}

/// LocalAI OpenAI-compatible API model response
#[derive(Debug, Clone, Deserialize)]
struct LocalAIModelResponse {
    data: Vec<LocalAIModel>,
}

/// LocalAI OpenAI-compatible API model
#[derive(Debug, Clone, Deserialize)]
struct LocalAIModel {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

/// LocalAI OpenAI-compatible API generation request
#[derive(Debug, Clone, Serialize)]
struct LocalAICompletionRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// LocalAI OpenAI-compatible API chat request
#[derive(Debug, Clone, Serialize)]
struct LocalAIChatRequest {
    model: String,
    messages: Vec<LocalAIChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// LocalAI OpenAI-compatible API chat message
#[derive(Debug, Clone, Serialize)]
struct LocalAIChatMessage {
    role: String,
    content: String,
}

/// LocalAI OpenAI-compatible API chat response
#[derive(Debug, Clone, Deserialize)]
struct LocalAIChatResponse {
    id: String,
    object: String,
    created: u64,
    choices: Vec<LocalAIChatChoice>,
    usage: Option<LocalAIUsage>,
}

/// LocalAI OpenAI-compatible API chat choice
#[derive(Debug, Clone, Deserialize)]
struct LocalAIChatChoice {
    index: u32,
    message: LocalAIChatMessage,
    finish_reason: Option<String>,
}

/// LocalAI OpenAI-compatible API completion response
#[derive(Debug, Clone, Deserialize)]
struct LocalAICompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<LocalAICompletionChoice>,
    usage: Option<LocalAIUsage>,
}

/// LocalAI OpenAI-compatible API completion choice
#[derive(Debug, Clone, Deserialize)]
struct LocalAICompletionChoice {
    text: String,
    index: u32,
    finish_reason: Option<String>,
}

/// LocalAI OpenAI-compatible API usage
#[derive(Debug, Clone, Deserialize)]
struct LocalAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// LocalAI streaming response chunk for chat
#[derive(Debug, Clone, Deserialize)]
struct LocalAIStreamingChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<LocalAIStreamingChoice>,
}

/// LocalAI streaming choice for chat
#[derive(Debug, Clone, Deserialize)]
struct LocalAIStreamingChoice {
    index: u32,
    delta: LocalAIStreamingDelta,
    finish_reason: Option<String>,
}

/// LocalAI streaming delta for chat
#[derive(Debug, Clone, Deserialize)]
struct LocalAIStreamingDelta {
    content: Option<String>,
    role: Option<String>,
}

/// LocalAI provider implementation
pub struct LocalAIProvider {
    /// Provider configuration
    config: LocalAIConfig,
    /// HTTP client
    client: Client,
    /// Active downloads
    downloads: Arc<Mutex<HashMap<String, DownloadStatus>>>,
    /// Base data directory
    data_dir: PathBuf,
    /// Map of model name to model info
    models_cache: Arc<Mutex<HashMap<String, ModelInfo>>>,
}

impl LocalAIProvider {
    /// Create a new LocalAI provider with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(LocalAIConfig::default())
    }
    
    /// Create a new LocalAI provider with the specified configuration
    pub fn with_config(config: LocalAIConfig) -> Result<Self> {
        // Create HTTP client with the specified configuration
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .danger_accept_invalid_certs(!config.verify_ssl);
            
        // Add API key if provided
        if let Some(api_key) = &config.api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            let api_key_value = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| ProviderError::ConfigurationError(format!("Invalid API key: {}", e)))?;
            headers.insert(reqwest::header::AUTHORIZATION, api_key_value);
            client_builder = client_builder.default_headers(headers);
        }
        
        let client = client_builder.build()
            .map_err(|e| ProviderError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;
        
        // Determine data directory
        let data_dir = Self::get_data_directory()?;
        
        Ok(Self {
            config,
            client,
            downloads: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
            models_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Get the data directory for LocalAI
    fn get_data_directory() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| ProviderError::ConfigurationError("Failed to determine home directory".into()))?;
        
        // Platform-specific paths
        #[cfg(target_os = "linux")]
        let data_dir = home.join(".local/share/papin/models/localai");
        
        #[cfg(target_os = "macos")]
        let data_dir = home.join("Library/Application Support/com.papin.app/models/localai");
        
        #[cfg(target_os = "windows")]
        let data_dir = home.join("AppData/Local/Papin/models/localai");
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| ProviderError::IoError(e))?;
        
        Ok(data_dir)
    }
    
    /// Get the base API URL
    fn api_url(&self) -> String {
        let mut url = self.config.endpoint.clone();
        if !url.ends_with('/') {
            url.push('/');
        }
        
        url
    }
    
    /// Add authorization header if needed
    fn add_auth_header(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = &self.config.api_key {
            builder.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", api_key))
        } else {
            builder
        }
    }
    
    /// Convert a LocalAI model to our ModelInfo format
    fn convert_model(&self, model: LocalAIModel) -> ModelInfo {
        // Parse model ID to extract details
        let (architecture, quantization) = self.parse_model_name(&model.id);
        
        // Rough size estimate based on model name
        let size_mb = if model.id.contains("7b") {
            4096 // ~4GB
        } else if model.id.contains("13b") {
            8192 // ~8GB
        } else if model.id.contains("70b") {
            40960 // ~40GB
        } else {
            2048 // ~2GB default
        };
        
        // Determine context size based on model family
        let context_size = match &architecture as &str {
            "llama2" | "llama3" => 4096,
            "mistral" => 8192,
            "mpt" => 2048,
            "gpt4all" => 2048,
            _ => 4096, // default
        };
        
        ModelInfo {
            id: model.id.clone(),
            name: model.id.clone(),
            size_mb,
            context_size,
            installed: true,
            download_url: None, // LocalAI handles downloads internally
            description: format!("LocalAI model: {}", model.id),
            quantization: Some(quantization),
            architecture: Some(architecture),
            format: None,
            metadata: HashMap::from([
                ("created".to_string(), model.created.to_string()),
                ("owned_by".to_string(), model.owned_by),
            ]),
        }
    }
    
    /// Parse model name to extract architecture and quantization
    fn parse_model_name(&self, name: &str) -> (String, String) {
        // Examples: llama-2-7b, mixtral-8x7b-instruct-v0.1-q4_0, mpt-7b
        let parts: Vec<&str> = name.split('-').collect();
        let mut arch_parts = Vec::new();
        let mut quantization = "unknown".to_string();
        
        // Extract architecture
        for part in &parts {
            if part.contains("q4") || part.contains("q8") {
                quantization = part.to_string();
            } else if part.contains('b') && part.chars().any(|c| c.is_numeric()) {
                // This part contains the model size (e.g., 7b)
                arch_parts.push(*part);
            } else if *part != "instruct" && *part != "chat" && !part.starts_with("v") {
                arch_parts.push(*part);
            }
        }
        
        let architecture = if arch_parts.is_empty() {
            name.to_string()
        } else {
            arch_parts.join("-")
        };
        
        (architecture, quantization)
    }
    
    /// Start background task to track download progress (Simulate for LocalAI)
    fn track_download_progress(&self, model_id: String) {
        let downloads = self.downloads.clone();
        let client = self.client.clone();
        let endpoint = format!("{}v1/models", self.api_url());
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut last_update = Instant::now();
            let poll_interval = Duration::from_millis(500);
            
            loop {
                // Sleep for the polling interval
                sleep(poll_interval).await;
                
                // Check if download still exists in our tracking map
                {
                    let downloads_lock = downloads.lock().unwrap();
                    if !downloads_lock.contains_key(&model_id) {
                        // Download was cancelled or completed
                        break;
                    }
                }
                
                // Request status from LocalAI
                let mut request = client.get(&endpoint);
                
                // Add authorization if needed
                if let Some(api_key) = &config.api_key {
                    request = request.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", api_key));
                }
                
                match request.send().await {
                    Ok(response) => {
                        if response.status() == StatusCode::OK {
                            // Try to parse the model list
                            if let Ok(model_response) = response.json::<LocalAIModelResponse>().await {
                                if model_response.data.iter().any(|m| m.id == model_id) {
                                    // Model exists, means download is complete
                                    let mut downloads_lock = downloads.lock().unwrap();
                                    if let Some(status) = downloads_lock.get_mut(&model_id) {
                                        status.progress = 1.0;
                                        status.bytes_downloaded = status.total_bytes;
                                        status.complete = true;
                                        status.eta_seconds = 0;
                                    }
                                    break;
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("Failed to check model status: {}", e);
                    }
                }
                
                // Update elapsed time and calculate speed
                let now = Instant::now();
                let elapsed = now.duration_since(last_update).as_secs_f32();
                last_update = now;
                
                // Update download status
                let mut downloads_lock = downloads.lock().unwrap();
                if let Some(status) = downloads_lock.get_mut(&model_id) {
                    // For LocalAI, we can only approximate progress
                    // Increment by a small amount each time
                    status.progress += 0.01;
                    if status.progress > 0.99 {
                        status.progress = 0.99; // Cap at 99% until confirmed complete
                    }
                    
                    // Update time estimates
                    if elapsed > 0.0 {
                        status.bytes_downloaded = (status.progress * status.total_bytes as f32) as usize;
                        status.speed_bps = (status.bytes_downloaded as f32 / elapsed) as usize;
                        let remaining_bytes = status.total_bytes - status.bytes_downloaded;
                        status.eta_seconds = if status.speed_bps > 0 {
                            (remaining_bytes / status.speed_bps) as u64
                        } else {
                            0
                        };
                    }
                } else {
                    // Download was cancelled
                    break;
                }
            }
        });
    }
}

#[async_trait]
impl Provider for LocalAIProvider {
    fn get_type(&self) -> ProviderType {
        ProviderType::LocalAI
    }
    
    fn get_name(&self) -> String {
        "LocalAI".to_string()
    }
    
    fn get_description(&self) -> String {
        "LocalAI provider for running local LLMs with OpenAI-compatible API".to_string()
    }
    
    async fn get_version(&self) -> Result<String> {
        // LocalAI doesn't have a direct version endpoint, try to connect and return a generic version
        if self.is_available().await? {
            Ok("LocalAI OpenAI-compatible API".to_string())
        } else {
            Err(ProviderError::NotAvailable("LocalAI is not available".into()))
        }
    }
    
    async fn is_available(&self) -> Result<bool> {
        let url = format!("{}v1/models", self.api_url());
        
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);
        
        match request.send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Failed to connect to LocalAI: {}", e);
                Ok(false)
            }
        }
    }
    
    async fn list_available_models(&self) -> Result<Vec<ModelInfo>> {
        self.list_downloaded_models().await
    }
    
    async fn list_downloaded_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}v1/models", self.api_url());
        
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);
        
        let response = request
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to LocalAI: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiError(format!(
                "Failed to list models: HTTP {}", 
                response.status()
            )));
        }
        
        let model_response: LocalAIModelResponse = response.json()
            .await
            .map_err(|e| ProviderError::SerializationError(e))?;
        
        let models = model_response.data.into_iter()
            .map(|model| self.convert_model(model))
            .collect::<Vec<_>>();
        
        // Update cache
        {
            let mut cache = self.models_cache.lock().unwrap();
            for model in &models {
                cache.insert(model.id.clone(), model.clone());
            }
        }
        
        Ok(models)
    }
    
    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        // Check cache first
        {
            let cache = self.models_cache.lock().unwrap();
            if let Some(model) = cache.get(model_id) {
                return Ok(model.clone());
            }
        }
        
        // If not in cache, list all models and find the one we want
        let models = self.list_downloaded_models().await?;
        
        models.into_iter()
            .find(|model| model.id == model_id)
            .ok_or_else(|| ProviderError::ModelNotFound(format!("Model '{}' not found", model_id)))
    }
    
    async fn download_model(&self, model_id: &str) -> Result<()> {
        // LocalAI doesn't have a standard model download API
        // This is a placeholder for API-based downloads
        
        // Check if download is already in progress
        {
            let downloads = self.downloads.lock().unwrap();
            if downloads.contains_key(model_id) {
                return Err(ProviderError::DownloadError(format!(
                    "Download for model '{}' already in progress", 
                    model_id
                )));
            }
        }
        
        // Create download status
        let status = DownloadStatus {
            model_id: model_id.to_string(),
            progress: 0.0,
            bytes_downloaded: 0,
            total_bytes: 1024 * 1024 * 1024, // 1GB placeholder
            speed_bps: 0,
            eta_seconds: 0,
            complete: false,
            error: None,
            target_path: self.data_dir.join(format!("{}.bin", model_id)),
        };
        
        // Add to downloads map
        {
            let mut downloads = self.downloads.lock().unwrap();
            downloads.insert(model_id.to_string(), status);
        }
        
        // Start tracking download progress
        self.track_download_progress(model_id.to_string());
        
        // Return immediately, as we're simulating the download
        // In a real implementation, we would initiate the download through LocalAI's API
        Ok(())
    }
    
    async fn cancel_download(&self, model_id: &str) -> Result<()> {
        // Remove from downloads map to stop tracking
        let mut downloads = self.downloads.lock().unwrap();
        if downloads.remove(model_id).is_none() {
            return Err(ProviderError::DownloadError(format!(
                "No download in progress for model '{}'", 
                model_id
            )));
        }
        
        // LocalAI doesn't have a direct way to cancel downloads
        Ok(())
    }
    
    async fn get_download_status(&self, model_id: &str) -> Result<DownloadStatus> {
        let downloads = self.downloads.lock().unwrap();
        downloads.get(model_id)
            .cloned()
            .ok_or_else(|| ProviderError::DownloadError(format!(
                "No download in progress for model '{}'", 
                model_id
            )))
    }
    
    async fn delete_model(&self, model_id: &str) -> Result<()> {
        // LocalAI doesn't provide a standard API for deleting models
        // This would generally be handled by the LocalAI administrator
        
        Err(ProviderError::ApiError(
            "Model deletion is not supported by the LocalAI API. Please remove the model manually.".to_string()
        ))
    }
    
    async fn generate_text(
        &self, 
        model_id: &str, 
        prompt: &str, 
        options: GenerationOptions
    ) -> Result<String> {
        // Determine whether to use chat or completions API based on model name
        if model_id.contains("gpt") || model_id.contains("chat") {
            self.generate_chat(model_id, prompt, options).await
        } else {
            self.generate_completion(model_id, prompt, options).await
        }
    }
    
    async fn generate_text_streaming<F>(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        // Determine whether to use chat or completions API based on model name
        if model_id.contains("gpt") || model_id.contains("chat") {
            self.generate_chat_streaming(model_id, prompt, options, callback).await
        } else {
            // Default to chat streaming even for completion models (more reliable)
            self.generate_chat_streaming(model_id, prompt, options, callback).await
        }
    }
    
    async fn is_model_loaded(&self, model_id: &str) -> Result<bool> {
        // LocalAI loads models on demand
        // We'll check if the model exists in the list
        let url = format!("{}v1/models", self.api_url());
        
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);
        
        let response = request
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to LocalAI: {}", e)))?;
        
        if !response.status().is_success() {
            return Ok(false);
        }
        
        let model_response: LocalAIModelResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e))?;
        
        Ok(model_response.data.iter().any(|m| m.id == model_id))
    }
    
    async fn load_model(&self, model_id: &str) -> Result<()> {
        // LocalAI loads models on demand
        // We can make a small request to ensure it's loaded
        let options = GenerationOptions {
            max_tokens: Some(1),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        // Generate a single token to load the model
        self.generate_text(model_id, " ", options).await?;
        
        Ok(())
    }
    
    async fn unload_model(&self, model_id: &str) -> Result<()> {
        // LocalAI doesn't provide a direct way to unload models
        Ok(())
    }
    
    async fn get_system_info(&self) -> Result<HashMap<String, String>> {
        // LocalAI doesn't provide a standard API for system info
        let mut info = HashMap::new();
        info.insert("provider".to_string(), "LocalAI".to_string());
        info.insert("endpoint".to_string(), self.api_url());
        
        // Add version if available
        if let Ok(version) = self.get_version().await {
            info.insert("version".to_string(), version);
        }
        
        Ok(info)
    }
}

impl LocalAIProvider {
    /// Generate text using the chat completions API
    async fn generate_chat(
        &self, 
        model_id: &str, 
        prompt: &str, 
        options: GenerationOptions
    ) -> Result<String> {
        let url = format!("{}v1/chat/completions", self.api_url());
        
        // Create chat message
        let messages = vec![
            LocalAIChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];
        
        // Create request
        let request = LocalAIChatRequest {
            model: model_id.to_string(),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            n: Some(1),
            stream: Some(false),
            stop: options.additional_params.get("stop")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                }),
        };
        
        let request_builder = self.client.post(&url).json(&request);
        let request_builder = self.add_auth_header(request_builder);
        
        let response = request_builder
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to LocalAI: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::GenerationError(format!(
                "Failed to generate text: HTTP {}", 
                response.status()
            )));
        }
        
        let chat_response: LocalAIChatResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e))?;
        
        if chat_response.choices.is_empty() {
            return Err(ProviderError::GenerationError("No choices returned".into()));
        }
        
        Ok(chat_response.choices[0].message.content.clone())
    }
    
    /// Generate text using the completions API
    async fn generate_completion(
        &self, 
        model_id: &str, 
        prompt: &str, 
        options: GenerationOptions
    ) -> Result<String> {
        let url = format!("{}v1/completions", self.api_url());
        
        // Create request
        let request = LocalAICompletionRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            n: Some(1),
            stream: Some(false),
            stop: options.additional_params.get("stop")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                }),
        };
        
        let request_builder = self.client.post(&url).json(&request);
        let request_builder = self.add_auth_header(request_builder);
        
        let response = request_builder
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to LocalAI: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::GenerationError(format!(
                "Failed to generate text: HTTP {}", 
                response.status()
            )));
        }
        
        let completion_response: LocalAICompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e))?;
        
        if completion_response.choices.is_empty() {
            return Err(ProviderError::GenerationError("No choices returned".into()));
        }
        
        Ok(completion_response.choices[0].text.clone())
    }
    
    /// Generate text using the chat completions API with streaming
    async fn generate_chat_streaming<F>(
        &self,
        model_id: &str,
        prompt: &str,
        options: GenerationOptions,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        let url = format!("{}v1/chat/completions", self.api_url());
        
        // Create chat message
        let messages = vec![
            LocalAIChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];
        
        // Create request
        let request = LocalAIChatRequest {
            model: model_id.to_string(),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            n: Some(1),
            stream: Some(true),
            stop: options.additional_params.get("stop")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                }),
        };
        
        let request_builder = self.client.post(&url).json(&request);
        let request_builder = self.add_auth_header(request_builder);
        
        let response = request_builder
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to LocalAI: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::GenerationError(format!(
                "Failed to generate text: HTTP {}", 
                response.status()
            )));
        }
        
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        
        // Process the stream
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                ProviderError::GenerationError(format!("Error reading stream: {}", e))
            })?;
            
            buffer.extend_from_slice(&chunk);
            
            // Process complete JSON objects from the buffer
            let mut pos = 0;
            while pos < buffer.len() {
                // Find the end of the current JSON object
                if let Some(end) = buffer[pos..].windows(6).position(|w| w == b"data: ".as_slice()) {
                    let start = pos;
                    pos = pos + end + 6; // Move to the start of the data
                    
                    // Find the end of this data chunk
                    if let Some(end) = buffer[pos..].windows(1).position(|w| w[0] == b'\n') {
                        let end = pos + end;
                        let json_bytes = &buffer[pos..end];
                        pos = end + 1; // Move past the newline
                        
                        if let Ok(json_str) = std::str::from_utf8(json_bytes) {
                            // Handle "[DONE]" marker
                            if json_str == "[DONE]" {
                                return Ok(());
                            }
                            
                            // Parse the JSON
                            if let Ok(chunk) = serde_json::from_str::<LocalAIStreamingChunk>(json_str) {
                                for choice in chunk.choices {
                                    if let Some(content) = choice.delta.content {
                                        if !callback(content) {
                                            // Callback returned false, stop generation
                                            return Ok(());
                                        }
                                    }
                                    
                                    if choice.finish_reason.is_some() {
                                        // Generation complete
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    } else {
                        // Incomplete data, wait for more
                        buffer = buffer[start..].to_vec();
                        break;
                    }
                } else {
                    // No data marker found, look for next complete JSON object
                    if let Some(end) = buffer[pos..].windows(1).position(|w| w[0] == b'\n') {
                        pos += end + 1; // Move past this line
                    } else {
                        // No complete line, wait for more data
                        buffer = buffer[pos..].to_vec();
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
}