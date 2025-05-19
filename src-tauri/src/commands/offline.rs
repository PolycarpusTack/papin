//! Tauri commands for offline features
//! 
//! This module provides commands for interacting with offline features,
//! including local LLM integration.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use tokio::sync::Mutex;

use crate::offline::llm::{
    LLMManager, LLMConfig, ProviderType,
    provider::{GenerationOptions, ModelInfo, DownloadStatus},
};
use crate::offline::OfflineManager;

/// Application state containing the offline manager
pub struct AppState {
    /// Offline manager
    pub offline_manager: Arc<Mutex<OfflineManager>>,
}

/// Response for offline commands
#[derive(Serialize)]
pub struct CommandResponse<T> {
    /// Whether the operation was successful
    pub success: bool,
    /// Optional error message
    pub error: Option<String>,
    /// Optional data
    pub data: Option<T>,
}

impl<T> CommandResponse<T> {
    /// Create a success response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            error: None,
            data: Some(data),
        }
    }
    
    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(message.into()),
            data: None,
        }
    }
}

/// Request for configuring the LLM
#[derive(Deserialize)]
pub struct ConfigureLLMRequest {
    /// Provider type to use
    pub provider_type: Option<ProviderType>,
    /// Provider-specific configuration as a JSON object
    pub provider_config: Option<serde_json::Value>,
    /// Default model to use
    pub default_model: Option<String>,
}

/// Request for generating text
#[derive(Deserialize)]
pub struct GenerateTextRequest {
    /// Model to use
    pub model_id: Option<String>,
    /// Prompt for generation
    pub prompt: String,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for sampling
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
}

/// Configure the LLM provider
#[command]
pub async fn configure_llm(
    state: State<'_, AppState>,
    request: ConfigureLLMRequest,
) -> CommandResponse<bool> {
    let offline_manager = state.offline_manager.lock().await;
    let mut llm_manager = offline_manager.llm_manager().lock().await;
    
    // Create a new configuration based on the current one and the request
    let mut config = LLMConfig::default();
    
    if let Some(provider_type) = request.provider_type {
        config.provider_type = provider_type;
    }
    
    if let Some(provider_config) = request.provider_config {
        config.provider_config = provider_config;
    }
    
    if let Some(default_model) = request.default_model {
        config.default_model = Some(default_model);
    }
    
    // Reinitialize the manager with the new configuration
    *llm_manager = LLMManager::with_config(config);
    
    match llm_manager.initialize().await {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(format!("Failed to configure LLM: {}", e)),
    }
}

/// List available models
#[command]
pub async fn list_available_models(
    state: State<'_, AppState>,
) -> CommandResponse<Vec<ModelInfo>> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    
    match llm_manager.list_available_models().await {
        Ok(models) => CommandResponse::success(models),
        Err(e) => CommandResponse::error(format!("Failed to list models: {}", e)),
    }
}

/// List downloaded models
#[command]
pub async fn list_downloaded_models(
    state: State<'_, AppState>,
) -> CommandResponse<Vec<ModelInfo>> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    
    match llm_manager.list_downloaded_models().await {
        Ok(models) => CommandResponse::success(models),
        Err(e) => CommandResponse::error(format!("Failed to list models: {}", e)),
    }
}

/// Get model info
#[command]
pub async fn get_model_info(
    state: State<'_, AppState>,
    model_id: String,
) -> CommandResponse<ModelInfo> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    
    match llm_manager.get_model_info(&model_id).await {
        Ok(model) => CommandResponse::success(model),
        Err(e) => CommandResponse::error(format!("Failed to get model info: {}", e)),
    }
}

/// Download a model
#[command]
pub async fn download_model(
    state: State<'_, AppState>,
    model_id: String,
) -> CommandResponse<bool> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    
    match llm_manager.download_model(&model_id).await {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(format!("Failed to download model: {}", e)),
    }
}

/// Get download status
#[command]
pub async fn get_download_status(
    state: State<'_, AppState>,
    model_id: String,
) -> CommandResponse<DownloadStatus> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    let provider = match llm_manager.get_provider() {
        Ok(p) => p,
        Err(e) => return CommandResponse::error(format!("Failed to get provider: {}", e)),
    };
    
    match provider.get_download_status(&model_id).await {
        Ok(status) => CommandResponse::success(status),
        Err(e) => CommandResponse::error(format!("Failed to get download status: {}", e)),
    }
}

/// Check if a model is loaded
#[command]
pub async fn is_model_loaded(
    state: State<'_, AppState>,
    model_id: String,
) -> CommandResponse<bool> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    
    match llm_manager.is_model_loaded(&model_id).await {
        Ok(loaded) => CommandResponse::success(loaded),
        Err(e) => CommandResponse::error(format!("Failed to check if model is loaded: {}", e)),
    }
}

/// Load a model
#[command]
pub async fn load_model(
    state: State<'_, AppState>,
    model_id: String,
) -> CommandResponse<bool> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    
    match llm_manager.load_model(&model_id).await {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(format!("Failed to load model: {}", e)),
    }
}

/// Delete a model
#[command]
pub async fn delete_model(
    state: State<'_, AppState>,
    model_id: String,
) -> CommandResponse<bool> {
    let offline_manager = state.offline_manager.lock().await;
    let llm_manager = offline_manager.llm_manager().lock().await;
    let provider = match llm_manager.get_provider() {
        Ok(p) => p,
        Err(e) => return CommandResponse::error(format!("Failed to get provider: {}", e)),
    };
    
    match provider.delete_model(&model_id).await {
        Ok(_) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(format!("Failed to delete model: {}", e)),
    }
}

/// Generate text
#[command]
pub async fn generate_text(
    state: State<'_, AppState>,
    request: GenerateTextRequest,
) -> CommandResponse<String> {
    let offline_manager = state.offline_manager.lock().await;
    
    // Create options from the request
    let options = GenerationOptions {
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: false,
        additional_params: Default::default(),
    };
    
    // Generate text
    match offline_manager.generate_text(
        &request.prompt,
        request.model_id.as_deref(),
    ).await {
        Ok(text) => CommandResponse::success(text),
        Err(e) => CommandResponse::error(format!("Failed to generate text: {}", e)),
    }
}

/// Check network connectivity
#[command]
pub async fn check_network(
    state: State<'_, AppState>,
) -> CommandResponse<bool> {
    let mut offline_manager = state.offline_manager.lock().await;
    
    match offline_manager.check_network().await {
        Ok(status) => {
            use crate::offline::NetworkStatus;
            let is_connected = status == NetworkStatus::Connected;
            CommandResponse::success(is_connected)
        },
        Err(e) => CommandResponse::error(format!("Failed to check network: {}", e)),
    }
}

/// Get offline mode status
#[command]
pub async fn get_offline_status(
    state: State<'_, AppState>,
) -> CommandResponse<bool> {
    let offline_manager = state.offline_manager.lock().await;
    CommandResponse::success(offline_manager.is_enabled())
}

/// Set offline mode
#[command]
pub async fn set_offline_mode(
    state: State<'_, AppState>,
    enabled: bool,
) -> CommandResponse<bool> {
    let mut offline_manager = state.offline_manager.lock().await;
    offline_manager.set_enabled(enabled);
    CommandResponse::success(enabled)
}

/// Initialize the offline manager
pub async fn init_offline_manager() -> Arc<Mutex<OfflineManager>> {
    crate::offline::create_offline_manager().await
}

/// Register all commands with Tauri
pub fn register_commands(app: &mut tauri::App) -> Result<(), tauri::Error> {
    let offline_manager = tauri::async_runtime::block_on(init_offline_manager());
    
    app.manage(AppState {
        offline_manager,
    });
    
    Ok(())
}