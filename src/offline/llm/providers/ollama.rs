// src/offline/llm/providers/ollama.rs
//! Ollama provider implementation for local LLM integration

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

/// Configuration for Ollama provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// Timeout for API requests in seconds
    pub timeout_seconds: u64,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    /// Custom headers to add to requests
    pub headers: HashMap<String, String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            verify_ssl: true,
            headers: HashMap::new(),
        }
    }
}

/// Ollama API model response
#[derive(Debug, Clone, Deserialize)]
struct OllamaModelResponse {
    models: Vec<OllamaModel>,
}

/// Ollama API model info
#[derive(Debug, Clone, Deserialize)]
struct OllamaModel {
    name: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: Option<OllamaModelDetails>,
}

/// Ollama API model details
#[derive(Debug, Clone, Deserialize)]
struct OllamaModelDetails {
    format: Option<String>,
    family: Option<String>,
    parameter_size: Option<String>,
    quantization_level: Option<String>,
}

/// Ollama API generation request
#[derive(Debug, Clone, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<bool>,
    options: Option<OllamaOptions>,
}

/// Ollama API generation options
#[derive(Debug, Clone, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// Ollama API generation response
#[derive(Debug, Clone, Deserialize)]
struct OllamaGenerateResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    context: Option<Vec<u32>>,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_duration: Option<u64>,
    eval_count: Option<u64>,
    eval_duration: Option<u64>,
}

/// Ollama API pull request
#[derive(Debug, Clone, Serialize)]
struct OllamaPullRequest {
    name: String,
    insecure: Option<bool>,
    stream: Option<bool>,
}

/// Ollama API pull response
#[derive(Debug, Clone, Deserialize)]
struct OllamaPullResponse {
    status: String,
    digest: Option<String>,
    total: Option<u64>,
    completed: Option<u64>,
}

/// Ollama provider implementation
pub struct OllamaProvider {
    /// Provider configuration
    config: OllamaConfig,
    /// HTTP client
    client: Client,
    /// Active downloads
    downloads: Arc<Mutex<HashMap<String, DownloadStatus>>>,
    /// Base data directory
    data_dir: PathBuf,
}

impl OllamaProvider {
    /// Create a new Ollama provider with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(OllamaConfig::default())
    }
    
    /// Create a new Ollama provider with the specified configuration
    pub fn with_config(config: OllamaConfig) -> Result<Self> {
        // Create HTTP client with the specified configuration
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()
            .map_err(|e| ProviderError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;
        
        // Determine data directory
        let data_dir = Self::get_data_directory()?;
        
        Ok(Self {
            config,
            client,
            downloads: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
        })
    }
    
    /// Get the data directory for Ollama
    fn get_data_directory() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| ProviderError::ConfigurationError("Failed to determine home directory".into()))?;
        
        // Platform-specific paths
        #[cfg(target_os = "linux")]
        let data_dir = home.join(".local/share/papin/models/ollama");
        
        #[cfg(target_os = "macos")]
        let data_dir = home.join("Library/Application Support/com.papin.app/models/ollama");
        
        #[cfg(target_os = "windows")]
        let data_dir = home.join("AppData/Local/Papin/models/ollama");
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| ProviderError::IoError(e))?;
        
        Ok(data_dir)
    }
    
    /// Get the base API URL
    fn api_url(&self) -> String {
        self.config.endpoint.clone()
    }
    
    /// Convert an Ollama model to our ModelInfo format
    fn convert_model(&self, model: OllamaModel) -> ModelInfo {
        // Extract quantization and model architecture from model name
        let (architecture, quantization) = self.parse_model_name(&model.name);
        
        // Determine model size in MB (convert from bytes)
        let size_mb = (model.size / (1024 * 1024)) as usize;
        
        // Determine context size based on model family
        let context_size = match &architecture as &str {
            "llama2" | "llama3" => 4096,
            "mistral" => 8192,
            "phi" => 2048,
            "gemma" => 8192,
            _ => 4096, // default
        };
        
        ModelInfo {
            id: model.name.clone(),
            name: model.name.clone(),
            size_mb,
            context_size,
            installed: true,
            download_url: None, // Ollama handles downloads internally
            description: format!("Ollama model: {}", model.name),
            quantization: Some(quantization),
            architecture: Some(architecture),
            format: model.details.as_ref().and_then(|d| d.format.clone()),
            metadata: HashMap::from([
                ("modified_at".to_string(), model.modified_at),
                ("digest".to_string(), model.digest),
            ]),
        }
    }
    
    /// Parse model name to extract architecture and quantization
    fn parse_model_name(&self, name: &str) -> (String, String) {
        // Examples: llama2, llama2:7b, llama2:7b-q4_0, mistral:7b-instruct-q8_0
        let parts: Vec<&str> = name.split(':').collect();
        let base_name = parts[0].to_string();
        
        if parts.len() == 1 {
            return (base_name, "unknown".to_string());
        }
        
        let variant = parts[1];
        let variant_parts: Vec<&str> = variant.split('-').collect();
        
        // Extract size/variant (e.g., 7b, 13b)
        let architecture = if variant_parts.len() > 0 && !variant_parts[0].contains("q") {
            format!("{}-{}", base_name, variant_parts[0])
        } else {
            base_name
        };
        
        // Extract quantization (e.g., q4_0, q8_0)
        let quantization = variant_parts.iter()
            .find(|part| part.starts_with("q"))
            .map(|q| q.to_string())
            .unwrap_or_else(|| "f16".to_string()); // Default to f16 if no quantization specified
        
        (architecture, quantization)
    }
    
    /// Start background task to track download progress
    fn track_download_progress(&self, model_id: String) {
        let downloads = self.downloads.clone();
        let client = self.client.clone();
        let endpoint = format!("{}/api/show", self.api_url());

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
                
                // Request status from Ollama
                match client.post(&endpoint)
                    .json(&serde_json::json!({ "name": model_id }))
                    .send()
                    .await 
                {
                    Ok(response) => {
                        if response.status() == StatusCode::OK {
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
                    // For Ollama, we can only approximate progress
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
impl Provider for OllamaProvider {
    fn get_type(&self) -> ProviderType {
        ProviderType::Ollama
    }
    
    fn get_name(&self) -> String {
        "Ollama".to_string()
    }
    
    fn get_description(&self) -> String {
        "Ollama provider for running local LLMs".to_string()
    }
    
    async fn get_version(&self) -> Result<String> {
        // Ollama doesn't have a direct version endpoint, try to connect and return a generic version
        if self.is_available().await? {
            Ok("Ollama API".to_string())
        } else {
            Err(ProviderError::NotAvailable("Ollama is not available".into()))
        }
    }
    
    async fn is_available(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.api_url());
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Failed to connect to Ollama: {}", e);
                Ok(false)
            }
        }
    }
    
    async fn list_available_models(&self) -> Result<Vec<ModelInfo>> {
        // Ollama doesn't differentiate between available and downloaded models
        // Both are accessed through the same API endpoint
        self.list_downloaded_models().await
    }
    
    async fn list_downloaded_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.api_url());
        
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiError(format!(
                "Failed to list models: HTTP {}", 
                response.status()
            )));
        }
        
        let model_response: OllamaModelResponse = response.json()
            .await
            .map_err(|e| ProviderError::SerializationError(e))?;
        
        let models = model_response.models.into_iter()
            .map(|model| self.convert_model(model))
            .collect();
        
        Ok(models)
    }
    
    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        // For Ollama, we need to list all models and find the one we want
        let models = self.list_downloaded_models().await?;
        
        models.into_iter()
            .find(|model| model.id == model_id)
            .ok_or_else(|| ProviderError::ModelNotFound(format!("Model '{}' not found", model_id)))
    }
    
    async fn download_model(&self, model_id: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.api_url());
        
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
            total_bytes: 0, // We don't know the size yet
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
        
        // Start download using Ollama API
        let request = OllamaPullRequest {
            name: model_id.to_string(),
            insecure: Some(!self.config.verify_ssl),
            stream: Some(false), // We'll track progress ourselves
        };
        
        // Start the download in a new task
        let client = self.client.clone();
        let url_clone = url.clone();
        let model_id_clone = model_id.to_string();
        let downloads = self.downloads.clone();
        
        tokio::spawn(async move {
            match client.post(&url_clone)
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    if !response.status().is_success() {
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        let mut downloads = downloads.lock().unwrap();
                        if let Some(status) = downloads.get_mut(&model_id_clone) {
                            status.error = Some(format!("Failed to download model: HTTP {} - {}", response.status(), error_text));
                        }
                        downloads.remove(&model_id_clone);
                        return;
                    }
                    
                    // Ollama handles the download internally, we'll poll for status
                    // This is handled by tracking logic, just update the status with estimated size
                    let mut downloads = downloads.lock().unwrap();
                    if let Some(status) = downloads.get_mut(&model_id_clone) {
                        // Rough size estimate based on model name
                        let estimated_size = if model_id_clone.contains("7b") {
                            4096 // ~4GB
                        } else if model_id_clone.contains("13b") {
                            8192 // ~8GB
                        } else if model_id_clone.contains("70b") {
                            40960 // ~40GB
                        } else {
                            2048 // ~2GB default
                        };
                        
                        status.total_bytes = estimated_size * 1024 * 1024;
                    }
                },
                Err(e) => {
                    let mut downloads = downloads.lock().unwrap();
                    if let Some(status) = downloads.get_mut(&model_id_clone) {
                        status.error = Some(format!("Failed to download model: {}", e));
                    }
                    downloads.remove(&model_id_clone);
                }
            }
        });
        
        // Start tracking download progress
        self.track_download_progress(model_id.to_string());
        
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
        
        // Ollama doesn't have a direct way to cancel downloads
        // We can only stop tracking it on our side
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
        let url = format!("{}/api/delete", self.api_url());
        
        let response = self.client.delete(&url)
            .json(&serde_json::json!({ "name": model_id }))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiError(format!(
                "Failed to delete model: HTTP {}", 
                response.status()
            )));
        }
        
        Ok(())
    }
    
    async fn generate_text(
        &self, 
        model_id: &str, 
        prompt: &str, 
        options: GenerationOptions
    ) -> Result<String> {
        let url = format!("{}/api/generate", self.api_url());
        
        // Create options for Ollama API
        let ollama_options = OllamaOptions {
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.additional_params.get("top_k")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            num_predict: options.max_tokens.map(|v| v as u32),
            stop: options.additional_params.get("stop")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                }),
        };
        
        // Create request
        let request = OllamaGenerateRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            system: options.additional_params.get("system")
                .and_then(|v| v.as_str())
                .map(String::from),
            template: options.additional_params.get("template")
                .and_then(|v| v.as_str())
                .map(String::from),
            context: None,
            stream: Some(false),
            raw: Some(false),
            options: Some(ollama_options),
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::GenerationError(format!(
                "Failed to generate text: HTTP {}", 
                response.status()
            )));
        }
        
        let generate_response: OllamaGenerateResponse = response.json()
            .await
            .map_err(|e| ProviderError::SerializationError(e))?;
        
        Ok(generate_response.response)
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
        let url = format!("{}/api/generate", self.api_url());
        
        // Create options for Ollama API
        let ollama_options = OllamaOptions {
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.additional_params.get("top_k")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            num_predict: options.max_tokens.map(|v| v as u32),
            stop: options.additional_params.get("stop")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                }),
        };
        
        // Create request
        let request = OllamaGenerateRequest {
            model: model_id.to_string(),
            prompt: prompt.to_string(),
            system: options.additional_params.get("system")
                .and_then(|v| v.as_str())
                .map(String::from),
            template: options.additional_params.get("template")
                .and_then(|v| v.as_str())
                .map(String::from),
            context: None,
            stream: Some(true),
            raw: Some(false),
            options: Some(ollama_options),
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;
        
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
                if let Some(end) = buffer[pos..].windows(1).position(|w| w[0] == b'\n') {
                    let end = pos + end;
                    if end > pos {
                        // Extract the JSON object
                        let json_bytes = &buffer[pos..end];
                        pos = end + 1; // Move past the newline
                        
                        if let Ok(json_str) = std::str::from_utf8(json_bytes) {
                            // Parse the JSON
                            if let Ok(response) = serde_json::from_str::<OllamaGenerateResponse>(json_str) {
                                // Call the callback with the generated text
                                if !callback(response.response) {
                                    // Callback returned false, stop generation
                                    return Ok(());
                                }
                                
                                // Check if generation is done
                                if response.done {
                                    return Ok(());
                                }
                            }
                        }
                    }
                } else {
                    // No complete JSON object found, leave the rest for the next iteration
                    buffer = buffer[pos..].to_vec();
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn is_model_loaded(&self, model_id: &str) -> Result<bool> {
        // Ollama doesn't provide a direct way to check if a model is loaded
        // We'll simply check if the model exists
        let url = format!("{}/api/show", self.api_url());
        
        let response = self.client.post(&url)
            .json(&serde_json::json!({ "name": model_id }))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;
        
        Ok(response.status().is_success())
    }
    
    async fn load_model(&self, model_id: &str) -> Result<()> {
        // Ollama automatically loads models when they're used
        // We can make a small request to ensure it's loaded
        let url = format!("{}/api/generate", self.api_url());
        
        // Create a minimal request to load the model
        let request = OllamaGenerateRequest {
            model: model_id.to_string(),
            prompt: " ".to_string(), // Empty prompt to just load the model
            system: None,
            template: None,
            context: None,
            stream: Some(false),
            raw: Some(false),
            options: Some(OllamaOptions {
                num_predict: Some(1), // Only generate 1 token
                temperature: Some(0.0),
                top_p: None,
                top_k: None,
                stop: None,
            }),
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::GenerationError(format!(
                "Failed to load model: HTTP {}", 
                response.status()
            )));
        }
        
        Ok(())
    }
    
    async fn unload_model(&self, model_id: &str) -> Result<()> {
        // Ollama doesn't provide a direct way to unload a model
        // Models are automatically unloaded based on memory pressure
        Ok(())
    }
    
    async fn get_system_info(&self) -> Result<HashMap<String, String>> {
        // Ollama doesn't provide a direct way to get system info
        // We'll return some basic information
        let mut info = HashMap::new();
        info.insert("provider".to_string(), "Ollama".to_string());
        info.insert("endpoint".to_string(), self.api_url());
        
        // Add version if available
        if let Ok(version) = self.get_version().await {
            info.insert("version".to_string(), version);
        }
        
        Ok(info)
    }
}