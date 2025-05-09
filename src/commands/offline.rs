use tauri::{command, AppHandle, Manager, State, Window};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};
use std::sync::Arc;

use crate::offline::{self, ConnectivityStatus, OfflineConfig, OfflineStats};
use crate::models::messages::{Message, Conversation};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl OfflineResponse {
    pub fn success(message: &str, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data,
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Check if offline mode is active
#[command]
pub async fn is_offline_mode_active() -> Result<bool> {
    Ok(offline::is_offline())
}

/// Get the current connectivity status
#[command]
pub async fn get_connectivity_status() -> Result<ConnectivityStatus> {
    Ok(offline::get_connectivity_status())
}

/// Enable offline mode
#[command]
pub async fn enable_offline_mode() -> Result<OfflineResponse> {
    match offline::enable_offline_mode() {
        Ok(_) => Ok(OfflineResponse::success("Offline mode enabled", None)),
        Err(e) => Ok(OfflineResponse::error(&format!("Failed to enable offline mode: {}", e))),
    }
}

/// Disable offline mode
#[command]
pub async fn disable_offline_mode() -> Result<OfflineResponse> {
    match offline::disable_offline_mode() {
        Ok(_) => Ok(OfflineResponse::success("Offline mode disabled", None)),
        Err(e) => Ok(OfflineResponse::error(&format!("Failed to disable offline mode: {}", e))),
    }
}

/// Process a message in offline mode
#[command]
pub async fn process_message_offline(message: Message, conversation: Conversation) -> Result<Message> {
    offline::process_message(&message, &conversation)
}

/// Sync offline changes to the cloud
#[command]
pub async fn sync_offline_changes() -> Result<OfflineResponse> {
    match offline::sync_changes() {
        Ok(_) => Ok(OfflineResponse::success("Changes synced successfully", None)),
        Err(e) => Ok(OfflineResponse::error(&format!("Failed to sync changes: {}", e))),
    }
}

/// Get the number of pending sync items
#[command]
pub async fn get_pending_sync_count() -> Result<usize> {
    offline::get_pending_sync_count()
}

/// Get the offline configuration
#[command]
pub async fn get_offline_config() -> Result<OfflineConfig> {
    offline::get_config()
}

/// Update the offline configuration
#[command]
pub async fn update_offline_config(config: OfflineConfig) -> Result<OfflineResponse> {
    match offline::update_config(config) {
        Ok(_) => Ok(OfflineResponse::success("Configuration updated", None)),
        Err(e) => Ok(OfflineResponse::error(&format!("Failed to update configuration: {}", e))),
    }
}

/// Get stats about offline mode
#[command]
pub async fn get_offline_stats() -> Result<OfflineStats> {
    offline::get_stats()
}

/// Get available local models
#[command]
pub async fn get_available_local_models() -> Result<Vec<String>> {
    match offline::get_config() {
        Ok(config) => {
            let llm_engine = crate::offline::llm::LLMEngine::new(
                &config.models_directory,
                &config.default_model,
            )?;
            llm_engine.get_available_models()
        },
        Err(e) => Err(e),
    }
}

/// Register all offline commands with Tauri
pub fn register_offline_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        is_offline_mode_active,
        get_connectivity_status,
        enable_offline_mode,
        disable_offline_mode,
        process_message_offline,
        sync_offline_changes,
        get_pending_sync_count,
        get_offline_config,
        update_offline_config,
        get_offline_stats,
        get_available_local_models,
    ])
}
