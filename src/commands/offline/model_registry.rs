// src/commands/offline/model_registry.rs

use tauri::{command, AppHandle, Manager, State, Window};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::offline::llm::models::{
    LLMModel, ModelRegistry, DownloadProgress, ModelFormat, 
    ModelArchitecture, QuantizationType, ModelCapabilities
};
use crate::offline::llm::provider::ProviderType;
use crate::commands::offline::llm::CommandResponse;

// -----------------------------
// Command Response Data Types
// -----------------------------

/// Model information for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Model description
    pub description: String,
    /// Model architecture
    pub architecture: String,
    /// Model format
    pub format: String,
    /// Parameter count in billions
    pub parameter_count: f32,
    /// Quantization type
    pub quantization: String,
    /// Context length
    pub context_length: usize,
    /// Size in megabytes
    pub size_mb: u64,
    /// Download URL if available
    pub download_url: Option<String>,
    /// Model source or creator
    pub source: String,
    /// License information
    pub license: String,
    /// Installation status
    pub installed: bool,
    /// Whether model is currently loaded
    pub loaded: bool,
    /// Last used timestamp
    pub last_used: Option<String>,
    /// Installation date
    pub installed_date: Option<String>,
    /// Suggested provider
    pub suggested_provider: Option<String>,
    /// Model capabilities
    pub capabilities: HashMap<String, bool>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl From<LLMModel> for ModelInfo {
    fn from(model: LLMModel) -> Self {
        // Convert capabilities to map
        let mut capabilities = HashMap::new();
        capabilities.insert("text_generation".to_string(), model.capabilities.text_generation);
        capabilities.insert("embeddings".to_string(), model.capabilities.embeddings);
        capabilities.insert("vision".to_string(), model.capabilities.vision);
        capabilities.insert("audio".to_string(), model.capabilities.audio);
        capabilities.insert("chat".to_string(), model.capabilities.chat);
        capabilities.insert("function_calling".to_string(), model.capabilities.function_calling);
        capabilities.insert("streaming".to_string(), model.capabilities.streaming);
        capabilities.insert("code_optimized".to_string(), model.capabilities.code_optimized);
        capabilities.insert("multilingual".to_string(), model.capabilities.multilingual);

        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            architecture: model.architecture.to_string(),
            format: model.format.to_string(),
            parameter_count: (model.parameters as f32) / 1_000_000_000.0, // Convert to billions
            quantization: model.quantization.to_string(),
            context_length: model.context_length,
            size_mb: model.size_mb,
            download_url: model.download_url,
            source: model.source,
            license: model.license,
            installed: model.installed,
            loaded: model.loaded,
            last_used: model.last_used.map(|d| d.to_rfc3339()),
            installed_date: model.installed_date.map(|d| d.to_rfc3339()),
            suggested_provider: model.suggested_provider.map(|p| p.to_string()),
            capabilities,
            metadata: model.metadata,
        }
    }
}

/// Download progress status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStatus {
    /// Model ID
    pub model_id: String,
    /// Progress as percentage (0-100)
    pub progress_percent: f32,
    /// Bytes downloaded
    pub bytes_downloaded: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Download speed in bytes per second
    pub speed_bps: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: u64,
    /// Is the download complete?
    pub completed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Start time
    pub start_time: String,
    /// Last update time
    pub last_update: String,
}

impl From<DownloadProgress> for DownloadStatus {
    fn from(progress: DownloadProgress) -> Self {
        Self {
            model_id: progress.model_id,
            progress_percent: progress.progress * 100.0,
            bytes_downloaded: progress.bytes_downloaded,
            total_bytes: progress.total_bytes,
            speed_bps: progress.speed_bps,
            eta_seconds: progress.eta_seconds,
            completed: progress.completed,
            error: progress.error,
            start_time: progress.start_time.to_rfc3339(),
            last_update: progress.last_update.to_rfc3339(),
        }
    }
}

/// Disk usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    /// Total disk space used by models in bytes
    pub used_bytes: u64,
    /// Disk space limit in bytes (0 = unlimited)
    pub limit_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Usage percentage (0-100)
    pub usage_percent: f32,
}

// --------------------------
// Registry Commands
// --------------------------

/// Get all models in the registry
#[command]
pub async fn get_all_models(app_handle: AppHandle) -> CommandResponse<Vec<ModelInfo>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    let models = registry.get_all_models();
    let model_infos = models.into_iter().map(ModelInfo::from).collect();
    
    CommandResponse::success(model_infos)
}

/// Get a specific model by ID
#[command]
pub async fn get_model(model_id: String, app_handle: AppHandle) -> CommandResponse<ModelInfo> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    match registry.get_model(&model_id) {
        Ok(model) => CommandResponse::success(ModelInfo::from(model)),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get installed models
#[command]
pub async fn get_installed_models(app_handle: AppHandle) -> CommandResponse<Vec<ModelInfo>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    let models = registry.get_installed_models();
    let model_infos = models.into_iter().map(ModelInfo::from).collect();
    
    CommandResponse::success(model_infos)
}

/// Get models compatible with a provider
#[command]
pub async fn get_compatible_models(
    provider: String,
    app_handle: AppHandle
) -> CommandResponse<Vec<ModelInfo>> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Convert provider string to ProviderType
    let provider_type = match ProviderType::from_string(&provider) {
        Ok(pt) => pt,
        Err(e) => return CommandResponse::error(&e.to_string()),
    };
    
    let models = registry.get_compatible_models(&provider_type);
    let model_infos = models.into_iter().map(ModelInfo::from).collect();
    
    CommandResponse::success(model_infos)
}

/// Update model metadata
#[command]
pub async fn update_model_metadata(
    model_id: String, 
    updates: HashMap<String, String>,
    app_handle: AppHandle
) -> CommandResponse<ModelInfo> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    let result = registry.update_model(&model_id, |model| {
        // Update basic fields if provided
        if let Some(name) = updates.get("name") {
            model.name = name.clone();
        }
        
        if let Some(description) = updates.get("description") {
            model.description = description.clone();
        }
        
        if let Some(license) = updates.get("license") {
            model.license = license.clone();
        }
        
        // Update metadata
        for (key, value) in &updates {
            if key != "name" && key != "description" && key != "license" {
                model.metadata.insert(key.clone(), value.clone());
            }
        }
    });
    
    match result {
        Ok(()) => {
            // Get updated model
            match registry.get_model(&model_id) {
                Ok(model) => CommandResponse::success(ModelInfo::from(model)),
                Err(e) => CommandResponse::error(&e.to_string()),
            }
        },
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Download a model
#[command]
pub async fn download_model(
    model_id: String,
    url: String,
    provider: String,
    app_handle: AppHandle
) -> CommandResponse<DownloadStatus> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Convert provider string to ProviderType
    let provider_type = match ProviderType::from_string(&provider) {
        Ok(pt) => pt,
        Err(e) => return CommandResponse::error(&e.to_string()),
    };
    
    // Start download process
    match registry.start_download(&model_id, &url, &provider_type) {
        Ok(()) => {
            // Return initial download status
            match registry.get_download_progress(&model_id) {
                Ok(progress) => CommandResponse::success(DownloadStatus::from(progress)),
                Err(e) => CommandResponse::error(&e.to_string()),
            }
        },
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get download status
#[command]
pub async fn get_download_status(
    model_id: String,
    app_handle: AppHandle
) -> CommandResponse<DownloadStatus> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    match registry.get_download_progress(&model_id) {
        Ok(progress) => CommandResponse::success(DownloadStatus::from(progress)),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Cancel a download
#[command]
pub async fn cancel_download(
    model_id: String,
    app_handle: AppHandle
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    
    // We'll need a custom implementation since the registry itself doesn't
    // handle cancellation but delegates to the provider
    match llm_manager.cancel_model_download(&model_id) {
        Ok(()) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e),
    }
}

/// Delete a model
#[command]
pub async fn delete_model(
    model_id: String,
    app_handle: AppHandle
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Get model first to check if it exists and get its path
    let model = match registry.get_model(&model_id) {
        Ok(model) => model,
        Err(e) => return CommandResponse::error(&e.to_string()),
    };
    
    // Model must be installed to delete
    if !model.installed {
        return CommandResponse::error("Model is not installed");
    }
    
    // Cannot delete a loaded model
    if model.loaded {
        return CommandResponse::error("Cannot delete a loaded model");
    }
    
    // Get model path
    let path = match model.path {
        Some(path) => path,
        None => return CommandResponse::error("Model has no path"),
    };
    
    // Delete model files
    if let Err(e) = std::fs::remove_dir_all(&path) {
        return CommandResponse::error(&format!("Failed to delete model files: {}", e));
    }
    
    // Update model in registry
    match registry.update_model(&model_id, |model| {
        model.installed = false;
        model.path = None;
    }) {
        Ok(()) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Import a model from an external path
#[command]
pub async fn import_model(
    source_path: String,
    model_id: String,
    app_handle: AppHandle
) -> CommandResponse<ModelInfo> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Convert string path to PathBuf
    let source = PathBuf::from(source_path);
    
    match registry.import_model(&source, &model_id) {
        Ok(()) => {
            // Get the imported model
            match registry.get_model(&model_id) {
                Ok(model) => CommandResponse::success(ModelInfo::from(model)),
                Err(e) => CommandResponse::error(&e.to_string()),
            }
        },
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Export a model to an external path
#[command]
pub async fn export_model(
    model_id: String,
    destination_path: String,
    app_handle: AppHandle
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Convert string path to PathBuf
    let destination = PathBuf::from(destination_path);
    
    match registry.export_model(&model_id, &destination) {
        Ok(()) => CommandResponse::success(true),
        Err(e) => CommandResponse::error(&e.to_string()),
    }
}

/// Get disk usage information
#[command]
pub async fn get_disk_usage(app_handle: AppHandle) -> CommandResponse<DiskUsageInfo> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Calculate disk usage
    let used_bytes = registry.calculate_disk_usage();
    
    // Get disk space limit (this is a property of the registry)
    // Assume we can access it via a function like get_disk_space_limit(), which we'd need to add
    let limit_bytes = 0; // Unlimited by default
    
    // Calculate remaining space
    let available_bytes = if limit_bytes == 0 {
        u64::MAX // Effectively unlimited
    } else {
        limit_bytes.saturating_sub(used_bytes)
    };
    
    // Calculate usage percentage
    let usage_percent = if limit_bytes == 0 {
        0.0 // Can't calculate percentage of unlimited
    } else {
        (used_bytes as f32 / limit_bytes as f32) * 100.0
    };
    
    let disk_usage = DiskUsageInfo {
        used_bytes,
        limit_bytes,
        available_bytes,
        usage_percent,
    };
    
    CommandResponse::success(disk_usage)
}

/// Set disk space limit
#[command]
pub async fn set_disk_space_limit(
    limit_bytes: u64,
    app_handle: AppHandle
) -> CommandResponse<DiskUsageInfo> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    
    // Update disk space limit
    llm_manager.set_model_disk_space_limit(limit_bytes);
    
    // Get updated disk usage info
    get_disk_usage(app_handle).await
}

/// Register model registry event listener
/// This sets up an event emitter that will send model registry events to the frontend
#[command]
pub async fn register_model_registry_events(
    window: Window,
    app_handle: AppHandle
) -> CommandResponse<bool> {
    let offline_manager = app_handle.state::<Arc<crate::offline::OfflineManager>>();
    let llm_manager = offline_manager.get_llm_manager();
    let registry = llm_manager.get_model_registry();
    
    // Create a new subscriber
    let mut rx = registry.subscribe();
    
    // Clone window for the task
    let window_clone = window.clone();
    
    // Spawn a background task to listen for events
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Convert event to a serializable format
            let event_json = match event {
                crate::offline::llm::models::ModelRegistryEvent::Added(id) => {
                    serde_json::json!({
                        "type": "added",
                        "modelId": id
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::Updated(id) => {
                    serde_json::json!({
                        "type": "updated",
                        "modelId": id
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::Removed(id) => {
                    serde_json::json!({
                        "type": "removed",
                        "modelId": id
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::DownloadStarted(id) => {
                    serde_json::json!({
                        "type": "downloadStarted",
                        "modelId": id
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::DownloadProgress(progress) => {
                    serde_json::json!({
                        "type": "downloadProgress",
                        "modelId": progress.model_id,
                        "progress": DownloadStatus::from(progress)
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::DownloadCompleted(id) => {
                    serde_json::json!({
                        "type": "downloadCompleted",
                        "modelId": id
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::DownloadFailed(id, error) => {
                    serde_json::json!({
                        "type": "downloadFailed",
                        "modelId": id,
                        "error": error
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::Loaded(id) => {
                    serde_json::json!({
                        "type": "loaded",
                        "modelId": id
                    })
                },
                crate::offline::llm::models::ModelRegistryEvent::Unloaded(id) => {
                    serde_json::json!({
                        "type": "unloaded",
                        "modelId": id
                    })
                },
            };
            
            // Emit event to the frontend
            if let Err(e) = window_clone.emit("model-registry-event", event_json) {
                error!("Failed to emit model registry event: {}", e);
            }
        }
    });
    
    CommandResponse::success(true)
}

// --------------------------
// Module Registration
// --------------------------

/// Generate list of command names 
pub fn init_commands() -> Vec<(&'static str, Box<dyn Fn() + Send + Sync + 'static>)> {
    vec![
        ("get_all_models", Box::new(|| {})),
        ("get_model", Box::new(|| {})),
        ("get_installed_models", Box::new(|| {})),
        ("get_compatible_models", Box::new(|| {})),
        ("update_model_metadata", Box::new(|| {})),
        ("download_model", Box::new(|| {})),
        ("get_download_status", Box::new(|| {})),
        ("cancel_download", Box::new(|| {})),
        ("delete_model", Box::new(|| {})),
        ("import_model", Box::new(|| {})),
        ("export_model", Box::new(|| {})),
        ("get_disk_usage", Box::new(|| {})),
        ("set_disk_space_limit", Box::new(|| {})),
        ("register_model_registry_events", Box::new(|| {})),
    ]
}