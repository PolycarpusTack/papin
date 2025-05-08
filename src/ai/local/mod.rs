mod inference;
mod models;

use self::inference::InferenceEngine;
use self::models::LocalModelInfo;
use crate::ai::{ModelError, ModelProvider, ModelProviderConfig, ModelStatus, ProviderType};
use crate::models::messages::{ContentType, Message, MessageContent, MessageError, MessageRole};
use crate::models::Model;
use crate::utils::config;
use crate::utils::events::{events, get_event_system};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Local model provider for offline operations
pub struct LocalProvider {
    /// Provider configuration
    config: ModelProviderConfig,
    
    /// Inference engine
    inference_engine: Arc<Mutex<Option<InferenceEngine>>>,
    
    /// Available models
    models: Arc<RwLock<Vec<LocalModelInfo>>>,
    
    /// Active model status
    model_status: Arc<RwLock<HashMap<String, ModelStatus>>>,
    
    /// Active streaming sessions
    active_streams: Arc<Mutex<HashMap<String, mpsc::Sender<Result<Message, MessageError>>>>>,
    
    /// Model download directory
    model_dir: PathBuf,
}

impl LocalProvider {
    /// Create a new local provider
    pub fn new() -> Result<Self, ModelError> {
        // Load configuration
        let config = config::get_config();
        let config_guard = config.lock().unwrap();
        
        // Get model directory
        let model_dir = if let Some(dir) = config_guard.get_string("ai.local.model_dir") {
            PathBuf::from(dir)
        } else {
            // Use default location
            if let Some(proj_dirs) = directories::ProjectDirs::from("com", "claude", "mcp") {
                proj_dirs.data_dir().join("models")
            } else {
                PathBuf::from("models")
            }
        };
        
        // Create model directory if it doesn't exist
        if !model_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&model_dir) {
                warn!("Failed to create model directory: {}", e);
            }
        }
        
        let provider_config = ModelProviderConfig {
            provider_type: ProviderType::Local,
            name: "Local Models".to_string(),
            base_url: "".to_string(),
            api_key: "".to_string(),
            organization_id: None,
            timeout: Duration::from_secs(60),
            default_model: "tinyllama".to_string(),
            fallback_model: None,
            enable_mcp: false,
            enable_streaming: true,
            settings: serde_json::Map::new(),
        };
        
        // Try to initialize inference engine
        let inference_engine = match InferenceEngine::new(&model_dir) {
            Ok(engine) => Some(engine),
            Err(e) => {
                warn!("Failed to initialize inference engine: {:?}", e);
                None
            }
        };
        
        // Discover available local models
        let default_models = vec![
            LocalModelInfo {
                id: "tinyllama".to_string(),
                name: "TinyLlama".to_string(),
                path: model_dir.join("tinyllama.bin"),
                parameters: 1_000_000_000,
                quantization: "q4_0".to_string(),
                context_size: 2048,
                is_downloaded: false,
                download_url: Some("https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0/resolve/main/ggml-model-q4_0.gguf".to_string()),
                model: Model {
                    id: "tinyllama".to_string(),
                    provider: "local".to_string(),
                    name: "TinyLlama 1.1B".to_string(),
                    version: "1.1".to_string(),
                    capabilities: crate::models::ModelCapabilities {
                        vision: false,
                        max_context_length: 2048,
                        functions: false,
                        streaming: true,
                    },
                },
            },
            LocalModelInfo {
                id: "redpajama-mini".to_string(),
                name: "RedPajama Mini".to_string(),
                path: model_dir.join("redpajama-mini.bin"),
                parameters: 1_400_000_000,
                quantization: "q4_0".to_string(),
                context_size: 2048,
                is_downloaded: false,
                download_url: Some("https://huggingface.co/weyaxi/redpajama.cpp/resolve/main/redpajama-mini-q4_0.bin".to_string()),
                model: Model {
                    id: "redpajama-mini".to_string(),
                    provider: "local".to_string(),
                    name: "RedPajama Mini".to_string(),
                    version: "1.0".to_string(),
                    capabilities: crate::models::ModelCapabilities {
                        vision: false,
                        max_context_length: 2048,
                        functions: false,
                        streaming: true,
                    },
                },
            },
        ];
        
        let provider = Self {
            config: provider_config,
            inference_engine: Arc::new(Mutex::new(inference_engine)),
            models: Arc::new(RwLock::new(default_models)),
            model_status: Arc::new(RwLock::new(HashMap::new())),
            active_streams: Arc::new(Mutex::new(HashMap::new())),
            model_dir,
        };
        
        // Update model download status
        provider.update_model_download_status();
        
        Ok(provider)
    }
    
    /// Update the download status of all models
    fn update_model_download_status(&self) {
        let mut models = self.models.write().unwrap();
        for model in models.iter_mut() {
            model.is_downloaded = model.path.exists();
        }
    }
    
    /// Download a model
    pub async fn download_model(&self, model_id: &str) -> Result<(), ModelError> {
        // Find model info
        let model_info = {
            let models = self.models.read().unwrap();
            models
                .iter()
                .find(|m| m.id == model_id)
                .cloned()
                .ok_or(ModelError::InvalidRequest)?
        };
        
        // Check if already downloaded
        if model_info.is_downloaded {
            return Ok(());
        }
        
        // Check if download URL is available
        let download_url = model_info
            .download_url
            .clone()
            .ok_or(ModelError::InvalidRequest)?;
        
        // Update model status
        {
            let mut statuses = self.model_status.write().unwrap();
            statuses.insert(model_id.to_string(), ModelStatus::Loading);
        }
        
        // Create temporary file
        let temp_path = model_info.path.with_extension("download");
        
        // Download model
        let client = reqwest::Client::new();
        let response = client
            .get(&download_url)
            .send()
            .await
            .map_err(|_| ModelError::NetworkError)?;
        
        if !response.status().is_success() {
            return Err(ModelError::NetworkError);
        }
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = model_info.path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!("Failed to create model directory: {}", e);
                    return Err(ModelError::SystemError);
                }
            }
        }
        
        // Save to file
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|_| ModelError::SystemError)?;
        
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|_| ModelError::NetworkError)?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .map_err(|_| ModelError::SystemError)?;
        }
        
        // Rename temp file to final file
        tokio::fs::rename(&temp_path, &model_info.path)
            .await
            .map_err(|_| ModelError::SystemError)?;
        
        // Update model download status
        self.update_model_download_status();
        
        // Update model status
        {
            let mut statuses = self.model_status.write().unwrap();
            statuses.insert(model_id.to_string(), ModelStatus::Available);
        }
        
        Ok(())
    }
    
    /// Load a model into the inference engine
    async fn load_model(&self, model_id: &str) -> Result<(), ModelError> {
        // Find model info
        let model_info = {
            let models = self.models.read().unwrap();
            models
                .iter()
                .find(|m| m.id == model_id)
                .cloned()
                .ok_or(ModelError::InvalidRequest)?
        };
        
        // Check if model is downloaded
        if !model_info.is_downloaded {
            // Try to download the model
            self.download_model(model_id).await?;
        }
        
        // Check if inference engine is initialized
        let mut engine_guard = self.inference_engine.lock().unwrap();
        if engine_guard.is_none() {
            *engine_guard = match InferenceEngine::new(&self.model_dir) {
                Ok(engine) => Some(engine),
                Err(e) => {
                    error!("Failed to initialize inference engine: {:?}", e);
                    return Err(ModelError::SystemError);
                }
            };
        }
        
        // Load model into inference engine
        if let Some(engine) = engine_guard.as_mut() {
            if let Err(e) = engine.load_model(&model_info) {
                error!("Failed to load model {}: {:?}", model_id, e);
                return Err(ModelError::SystemError);
            }
        } else {
            return Err(ModelError::SystemError);
        }
        
        // Update model status
        {
            let mut statuses = self.model_status.write().unwrap();
            statuses.insert(model_id.to_string(), ModelStatus::Available);
        }
        
        Ok(())
    }
    
    /// Process a message with the local model
    async fn process_message(&self, model_id: &str, message: &Message) -> Result<String, ModelError> {
        // Load model if needed
        self.load_model(model_id).await?;
        
        // Extract text from message
        let mut prompt = String::new();
        
        // Process message content
        for part in &message.content.parts {
            match part {
                ContentType::Text { text } => {
                    prompt.push_str(text);
                }
                _ => {
                    // Local models only support text input for now
                    warn!("Local model only supports text input");
                }
            }
        }
        
        // Generate response using inference engine
        let engine_guard = self.inference_engine.lock().unwrap();
        if let Some(engine) = engine_guard.as_ref() {
            match engine.generate(&prompt) {
                Ok(response) => Ok(response),
                Err(e) => {
                    error!("Inference error: {:?}", e);
                    Err(ModelError::SystemError)
                }
            }
        } else {
            Err(ModelError::SystemError)
        }
    }
    
    /// Process a streaming message with the local model
    async fn process_streaming(
        &self,
        model_id: &str,
        message: &Message,
        tx: mpsc::Sender<Result<Message, MessageError>>,
    ) -> Result<(), ModelError> {
        // Load model if needed
        self.load_model(model_id).await?;
        
        // Extract text from message
        let mut prompt = String::new();
        
        // Process message content
        for part in &message.content.parts {
            match part {
                ContentType::Text { text } => {
                    prompt.push_str(text);
                }
                _ => {
                    // Local models only support text input for now
                    warn!("Local model only supports text input");
                }
            }
        }
        
        // Generate streaming response using inference engine
        let engine_guard = self.inference_engine.lock().unwrap();
        if let Some(engine) = engine_guard.as_ref() {
            let mut accumulated_text = String::new();
            let response_id = Uuid::new_v4().to_string();
            
            match engine.generate_streaming(&prompt, 512, |token| {
                // Accumulate text
                accumulated_text.push_str(token);
                
                // Send partial message
                let message = Message {
                    id: response_id.clone(),
                    role: MessageRole::Assistant,
                    content: MessageContent {
                        parts: vec![ContentType::Text {
                            text: accumulated_text.clone(),
                        }],
                    },
                    metadata: Some(HashMap::from([(
                        "model".to_string(),
                        serde_json::to_value(model_id).unwrap(),
                    )])),
                    created_at: SystemTime::now(),
                };
                
                // Send token - ignore errors as the receiver might be dropped
                let _ = tx.blocking_send(Ok(message));
                
                // Continue generating
                true
            }) {
                Ok(_) => {
                    // Send final message with complete text
                    let final_message = Message {
                        id: response_id,
                        role: MessageRole::Assistant,
                        content: MessageContent {
                            parts: vec![ContentType::Text {
                                text: accumulated_text,
                            }],
                        },
                        metadata: Some(HashMap::from([(
                            "model".to_string(),
                            serde_json::to_value(model_id).unwrap(),
                        )])),
                        created_at: SystemTime::now(),
                    };
                    
                    let _ = tx.blocking_send(Ok(final_message));
                    Ok(())
                }
                Err(e) => {
                    error!("Inference error: {:?}", e);
                    Err(ModelError::SystemError)
                }
            }
        } else {
            Err(ModelError::SystemError)
        }
    }
}

#[async_trait]
impl ModelProvider for LocalProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Local
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModelProviderConfig {
        &self.config
    }
    
    async fn available_models(&self) -> Result<Vec<Model>, ModelError> {
        // Return cached model list
        let models = self.models.read().unwrap();
        let result = models.iter().map(|m| m.model.clone()).collect();
        Ok(result)
    }
    
    async fn is_available(&self, model_id: &str) -> bool {
        // Check if model exists in available models
        let models = self.models.read().unwrap();
        let model = models.iter().find(|m| m.id == model_id);
        
        match model {
            Some(model_info) => model_info.is_downloaded,
            None => false,
        }
    }
    
    async fn model_status(&self, model_id: &str) -> ModelStatus {
        // Check cached status
        {
            let statuses = self.model_status.read().unwrap();
            if let Some(status) = statuses.get(model_id) {
                return *status;
            }
        }
        
        // Find model
        let models = self.models.read().unwrap();
        let model = models.iter().find(|m| m.id == model_id);
        
        match model {
            Some(model_info) => {
                let status = if model_info.is_downloaded {
                    ModelStatus::Available
                } else {
                    ModelStatus::Unavailable
                };
                
                // Update cache
                {
                    let mut statuses = self.model_status.write().unwrap();
                    statuses.insert(model_id.to_string(), status);
                }
                
                status
            }
            None => {
                // Update cache
                {
                    let mut statuses = self.model_status.write().unwrap();
                    statuses.insert(model_id.to_string(), ModelStatus::Unavailable);
                }
                
                ModelStatus::Unavailable
            }
        }
    }
    
    async fn complete(&self, model_id: &str, message: Message) -> Result<Message, MessageError> {
        // Check if model is available
        if !self.is_available(model_id).await {
            let downloadable = {
                let models = self.models.read().unwrap();
                models
                    .iter()
                    .find(|m| m.id == model_id)
                    .map(|m| m.download_url.is_some())
                    .unwrap_or(false)
            };
            
            if downloadable {
                // Try to download the model
                match self.download_model(model_id).await {
                    Ok(_) => {
                        // Continue with completion
                    }
                    Err(e) => {
                        return Err(MessageError::ProtocolError(format!(
                            "Failed to download model {}: {:?}",
                            model_id, e
                        )));
                    }
                }
            } else {
                return Err(MessageError::ProtocolError(format!(
                    "Model {} is not available",
                    model_id
                )));
            }
        }
        
        // Process message
        match self.process_message(model_id, &message).await {
            Ok(response_text) => {
                // Create response message
                let response = Message {
                    id: Uuid::new_v4().to_string(),
                    role: MessageRole::Assistant,
                    content: MessageContent {
                        parts: vec![ContentType::Text {
                            text: response_text,
                        }],
                    },
                    metadata: Some(HashMap::from([(
                        "model".to_string(),
                        serde_json::to_value(model_id).unwrap(),
                    )])),
                    created_at: SystemTime::now(),
                };
                
                Ok(response)
            }
            Err(e) => Err(MessageError::ProtocolError(format!(
                "Failed to process message: {:?}",
                e
            ))),
        }
    }
    
    async fn stream(
        &self,
        model_id: &str,
        message: Message,
    ) -> Result<mpsc::Receiver<Result<Message, MessageError>>, MessageError> {
        // Check if model is available
        if !self.is_available(model_id).await {
            let downloadable = {
                let models = self.models.read().unwrap();
                models
                    .iter()
                    .find(|m| m.id == model_id)
                    .map(|m| m.download_url.is_some())
                    .unwrap_or(false)
            };
            
            if downloadable {
                // Try to download the model
                match self.download_model(model_id).await {
                    Ok(_) => {
                        // Continue with streaming
                    }
                    Err(e) => {
                        return Err(MessageError::ProtocolError(format!(
                            "Failed to download model {}: {:?}",
                            model_id, e
                        )));
                    }
                }
            } else {
                return Err(MessageError::ProtocolError(format!(
                    "Model {} is not available",
                    model_id
                )));
            }
        }
        
        // Create streaming channel
        let (tx, rx) = mpsc::channel(32);
        let stream_id = Uuid::new_v4().to_string();
        
        // Store streaming channel
        {
            let mut streams = self.active_streams.lock().unwrap();
            streams.insert(stream_id.clone(), tx.clone());
        }
        
        // Start streaming in background
        let self_clone = self.clone();
        let message_clone = message.clone();
        let model_id_clone = model_id.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = self_clone.process_streaming(&model_id_clone, &message_clone, tx.clone()).await {
                // Send error
                let _ = tx
                    .send(Err(MessageError::ProtocolError(format!(
                        "Streaming error: {:?}",
                        e
                    ))))
                    .await;
            }
            
            // Remove streaming channel when done
            let mut streams = self_clone.active_streams.lock().unwrap();
            streams.remove(&stream_id);
        });
        
        Ok(rx)
    }
    
    async fn cancel_stream(&self, stream_id: &str) -> Result<(), MessageError> {
        // Find streaming channel
        let tx_opt = {
            let mut streams = self.active_streams.lock().unwrap();
            streams.remove(stream_id)
        };
        
        if tx_opt.is_some() {
            // Channel is dropped, stream will be cancelled when the task notices
            Ok(())
        } else {
            Err(MessageError::Unknown(format!(
                "Stream {} not found",
                stream_id
            )))
        }
    }
}

impl Clone for LocalProvider {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            inference_engine: self.inference_engine.clone(),
            models: self.models.clone(),
            model_status: self.model_status.clone(),
            active_streams: self.active_streams.clone(),
            model_dir: self.model_dir.clone(),
        }
    }
}
