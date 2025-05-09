use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use log::{debug, error};

use crate::observability::metrics::llm::{
    ProviderPerformanceMetrics, ModelPerformanceMetrics,
    get_provider_metrics, get_model_metrics, LLMMetricsConfig,
    get_llm_metrics_manager
};
use crate::error::Result;
use crate::commands::offline::llm::get_active_provider_type;

// Make provider and model metrics serializable for Tauri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProviderMetrics(pub HashMap<String, ProviderPerformanceMetrics>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableModelMetrics(pub HashMap<String, ModelPerformanceMetrics>);

/// Get LLM provider metrics
#[tauri::command]
pub fn get_llm_provider_metrics() -> Result<HashMap<String, ProviderPerformanceMetrics>> {
    debug!("Getting LLM provider metrics");
    
    match get_provider_metrics() {
        Some(metrics) => Ok(metrics),
        None => Ok(HashMap::new()),
    }
}

/// Get LLM model metrics
#[tauri::command]
pub fn get_llm_model_metrics() -> Result<HashMap<String, ModelPerformanceMetrics>> {
    debug!("Getting LLM model metrics");
    
    match get_model_metrics() {
        Some(metrics) => Ok(metrics),
        None => Ok(HashMap::new()),
    }
}

/// Get active LLM provider
#[tauri::command]
pub fn get_active_llm_provider() -> Result<String> {
    debug!("Getting active LLM provider");
    
    match get_active_provider_type() {
        Ok(provider) => Ok(provider),
        Err(_) => Ok("None".to_string()),
    }
}

/// Get default LLM model
#[tauri::command]
pub fn get_default_llm_model() -> Result<String> {
    debug!("Getting default LLM model");
    
    // Get from the offline config
    match crate::commands::offline::get_offline_config() {
        Ok(config) => {
            if let Some(default_model) = config.llm_config.default_model {
                Ok(default_model)
            } else {
                Ok("None".to_string())
            }
        },
        Err(_) => Ok("None".to_string()),
    }
}

/// Check if LLM metrics collection is enabled
#[tauri::command]
pub fn get_llm_metrics_enabled() -> Result<bool> {
    debug!("Checking if LLM metrics collection is enabled");
    
    match get_llm_metrics_manager() {
        Some(manager) => Ok(manager.is_enabled()),
        None => Ok(false),
    }
}

/// Get current LLM metrics configuration
#[tauri::command]
pub fn get_llm_metrics_config() -> Result<LLMMetricsConfig> {
    debug!("Getting LLM metrics configuration");
    
    match get_llm_metrics_manager() {
        Some(manager) => Ok(manager.get_config()),
        None => Ok(LLMMetricsConfig::default()),
    }
}

/// Update LLM metrics configuration
#[tauri::command]
pub fn update_llm_metrics_config(config: LLMMetricsConfig) -> Result<bool> {
    debug!("Updating LLM metrics configuration");
    
    match get_llm_metrics_manager() {
        Some(manager) => {
            manager.update_config(config);
            Ok(true)
        },
        None => {
            error!("Cannot update LLM metrics config: manager not initialized");
            Ok(false)
        },
    }
}

/// Accept privacy notice for metrics collection
#[tauri::command]
pub fn accept_llm_metrics_privacy_notice(version: String) -> Result<bool> {
    debug!("Accepting LLM metrics privacy notice: {}", version);
    
    match get_llm_metrics_manager() {
        Some(manager) => {
            manager.accept_privacy_notice(&version);
            Ok(true)
        },
        None => {
            error!("Cannot accept privacy notice: manager not initialized");
            Ok(false)
        },
    }
}

/// Reset all LLM metrics (for testing/debugging)
#[tauri::command]
pub fn reset_llm_metrics() -> Result<bool> {
    debug!("Resetting LLM metrics");
    
    // Simply reinitialize the metrics manager
    match crate::src_common::observability::telemetry::get_telemetry_client() {
        Some(client) => {
            let _ = crate::observability::metrics::llm::init_llm_metrics(Some(client));
            Ok(true)
        },
        None => {
            let _ = crate::observability::metrics::llm::init_llm_metrics(None);
            Ok(true)
        },
    }
}