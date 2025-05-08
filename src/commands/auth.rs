use crate::services::auth::get_auth_service;
use tauri::State;

/// Set API key
#[tauri::command]
pub async fn set_api_key(api_key: String) -> Result<(), String> {
    get_auth_service().set_api_key(api_key)
}

/// Validate API key
#[tauri::command]
pub async fn validate_api_key() -> Result<bool, String> {
    get_auth_service().validate_api_key().await
}

/// Get organization ID
#[tauri::command]
pub fn get_organization_id() -> Option<String> {
    get_auth_service().get_organization_id()
}

/// Logout
#[tauri::command]
pub fn logout() -> Result<(), String> {
    get_auth_service().logout()
}
