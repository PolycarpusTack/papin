// Security Commands Module
//
// This module provides Tauri commands for interacting with the security system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;
use crate::security::{
    SecurityConfig,
    DataClassification,
    PermissionLevel,
    init_security_manager,
    get_security_manager,
    check_permission,
    request_permission,
    store_credential,
    get_credential,
    encrypt_for_sync,
    decrypt_from_sync,
};
use crate::security::permissions::PermissionStatistics;
use crate::security::data_flow::{DataFlowGraph, DataFlowEvent, DataFlowStatistics};

// Security system configuration
#[tauri::command]
pub async fn init_security(config: Option<SecurityConfig>) -> Result<()> {
    init_security_manager(config)
}

#[tauri::command]
pub async fn get_security_config() -> Result<SecurityConfig> {
    let manager = get_security_manager()?;
    Ok(manager.get_config())
}

#[tauri::command]
pub async fn update_security_config(config: SecurityConfig) -> Result<()> {
    let manager = get_security_manager()?;
    manager.update_config(config)
}

// Credentials management commands

#[tauri::command]
pub async fn store_secure_credential(key: String, value: String) -> Result<()> {
    store_credential(&key, &value)
}

#[tauri::command]
pub async fn get_secure_credential(key: String) -> Result<String> {
    get_credential(&key)
}

#[tauri::command]
pub async fn delete_secure_credential(key: String) -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_credential_manager().read().unwrap().delete_credential(&key)
}

#[tauri::command]
pub async fn list_secure_credentials() -> Result<Vec<String>> {
    let manager = get_security_manager()?;
    manager.get_credential_manager().read().unwrap().list_credential_keys()
}

// End-to-end encryption commands

#[tauri::command]
pub async fn encrypt_data(data: Vec<u8>) -> Result<Vec<u8>> {
    encrypt_for_sync(&data)
}

#[tauri::command]
pub async fn decrypt_data(data: Vec<u8>) -> Result<Vec<u8>> {
    decrypt_from_sync(&data)
}

#[tauri::command]
pub async fn rotate_encryption_keys() -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_e2ee_manager().read().unwrap().rotate_keys()
}

// Permission management commands

#[tauri::command]
pub async fn check_permission_granted(permission: String) -> Result<bool> {
    check_permission(&permission)
}

#[tauri::command]
pub async fn request_app_permission(permission: String, reason: String) -> Result<bool> {
    request_permission(&permission, &reason)
}

#[tauri::command]
pub async fn get_all_permissions() -> Result<Vec<serde_json::Value>> {
    let manager = get_security_manager()?;
    let permissions = manager.get_permission_manager().read().unwrap().get_all_permissions()?;
    
    // Convert to generic JSON values for frontend
    let json_permissions = permissions.into_iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect();
        
    Ok(json_permissions)
}

#[tauri::command]
pub async fn set_permission_level(id: String, level: PermissionLevel) -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_permission_manager().read().unwrap().set_permission_level(&id, level)
}

#[tauri::command]
pub async fn reset_permission(id: String) -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_permission_manager().read().unwrap().reset_permission(&id)
}

#[tauri::command]
pub async fn reset_all_permissions() -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_permission_manager().read().unwrap().reset_all_permissions()
}

#[tauri::command]
pub async fn get_permission_statistics() -> Result<PermissionStatistics> {
    let manager = get_security_manager()?;
    manager.get_permission_manager().read().unwrap().get_statistics()
}

// Data flow tracking commands

#[tauri::command]
pub async fn get_data_flow_graph() -> Result<DataFlowGraph> {
    let manager = get_security_manager()?;
    manager.get_data_flow_manager().read().unwrap().get_data_flow_graph()
}

#[tauri::command]
pub async fn get_recent_data_flow_events(limit: Option<usize>) -> Result<Vec<DataFlowEvent>> {
    let manager = get_security_manager()?;
    manager.get_data_flow_manager().read().unwrap().get_recent_events(limit)
}

#[tauri::command]
pub async fn track_data_flow(
    operation: String,
    data_item: String,
    classification: DataClassification,
    destination: String,
) -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_data_flow_manager().read().unwrap().track_data_flow(
        &operation,
        &data_item,
        classification,
        &destination,
    )
}

#[tauri::command]
pub async fn clear_data_flow_events() -> Result<()> {
    let manager = get_security_manager()?;
    manager.get_data_flow_manager().read().unwrap().clear_events()
}

#[tauri::command]
pub async fn get_data_flow_statistics() -> Result<DataFlowStatistics> {
    let manager = get_security_manager()?;
    manager.get_data_flow_manager().read().unwrap().get_statistics()
}

#[tauri::command]
pub async fn search_data_flow_events(
    data_item: Option<String>,
    classification: Option<DataClassification>,
    source: Option<String>,
    destination: Option<String>,
    operation: Option<String>,
) -> Result<Vec<DataFlowEvent>> {
    let manager = get_security_manager()?;
    manager.get_data_flow_manager().read().unwrap().search_events(
        data_item.as_deref(),
        classification,
        source.as_deref(),
        destination.as_deref(),
        operation.as_deref(),
        None,
        None,
    )
}
