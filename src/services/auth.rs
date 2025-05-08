use crate::services::api::{get_api_service, ApiError};
use crate::utils::config;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Authentication service for handling API keys and tokens
pub struct AuthService {
    /// Current API key
    api_key: Arc<RwLock<String>>,
    
    /// Current session token
    session_token: Arc<RwLock<Option<String>>>,
    
    /// Session expiration time
    session_expiry: Arc<RwLock<Option<SystemTime>>>,
    
    /// Organization ID (for multi-org accounts)
    organization_id: Arc<RwLock<Option<String>>>,
    
    /// Is authenticated
    is_authenticated: Arc<RwLock<bool>>,
}

/// API key validation response
#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyValidation {
    valid: bool,
    expires_at: Option<String>,
    organization_id: Option<String>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new() -> Self {
        // Load configuration
        let config = config::get_config();
        let config_guard = config.lock().unwrap();
        
        let api_key = config_guard
            .get_string("api.key")
            .unwrap_or_else(|| String::new());
        
        Self {
            api_key: Arc::new(RwLock::new(api_key)),
            session_token: Arc::new(RwLock::new(None)),
            session_expiry: Arc::new(RwLock::new(None)),
            organization_id: Arc::new(RwLock::new(None)),
            is_authenticated: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Set API key
    pub fn set_api_key(&self, api_key: String) -> Result<(), String> {
        let mut api_key_guard = self.api_key.write().unwrap();
        *api_key_guard = api_key;
        
        // Reset authentication state
        {
            let mut auth_guard = self.is_authenticated.write().unwrap();
            *auth_guard = false;
        }
        {
            let mut token_guard = self.session_token.write().unwrap();
            *token_guard = None;
        }
        {
            let mut expiry_guard = self.session_expiry.write().unwrap();
            *expiry_guard = None;
        }
        
        // Save to config
        config::set_value("api.key", serde_json::Value::String(api_key_guard.clone()))
            .map_err(|e| e.to_string())?;
        
        // Save config to disk
        config::save_config().map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// Get current API key
    pub fn get_api_key(&self) -> String {
        self.api_key.read().unwrap().clone()
    }
    
    /// Validate API key
    pub async fn validate_api_key(&self) -> Result<bool, String> {
        let api_key = self.api_key.read().unwrap().clone();
        
        if api_key.is_empty() {
            return Err("API key is empty".to_string());
        }
        
        // In a real implementation, we would call the API to validate the key
        // For now, we'll simulate a successful validation if the key is non-empty
        
        // Simulated API key validation
        let validation = ApiKeyValidation {
            valid: true,
            expires_at: Some("2025-12-31T00:00:00Z".to_string()),
            organization_id: Some("org_123456".to_string()),
        };
        
        // Update state based on validation
        if validation.valid {
            // Set organization ID if provided
            if let Some(org_id) = validation.organization_id {
                let mut org_guard = self.organization_id.write().unwrap();
                *org_guard = Some(org_id);
            }
            
            // Mark as authenticated
            {
                let mut auth_guard = self.is_authenticated.write().unwrap();
                *auth_guard = true;
            }
            
            Ok(true)
        } else {
            // Mark as not authenticated
            {
                let mut auth_guard = self.is_authenticated.write().unwrap();
                *auth_guard = false;
            }
            
            Ok(false)
        }
    }
    
    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        // Check if we have a valid session token
        if let Some(expiry) = *self.session_expiry.read().unwrap() {
            if expiry > SystemTime::now() {
                return true;
            }
        }
        
        // Otherwise check the general authentication state
        *self.is_authenticated.read().unwrap()
    }
    
    /// Get organization ID
    pub fn get_organization_id(&self) -> Option<String> {
        self.organization_id.read().unwrap().clone()
    }
    
    /// Set organization ID
    pub fn set_organization_id(&self, organization_id: Option<String>) {
        let mut org_guard = self.organization_id.write().unwrap();
        *org_guard = organization_id;
    }
    
    /// Logout and clear credentials
    pub fn logout(&self) -> Result<(), String> {
        // Clear authentication state
        {
            let mut auth_guard = self.is_authenticated.write().unwrap();
            *auth_guard = false;
        }
        {
            let mut token_guard = self.session_token.write().unwrap();
            *token_guard = None;
        }
        {
            let mut expiry_guard = self.session_expiry.write().unwrap();
            *expiry_guard = None;
        }
        
        // We don't clear the API key from memory
        // but we could clear it from config if desired
        
        Ok(())
    }
}

/// Global authentication service instance
static AUTH_SERVICE: once_cell::sync::OnceCell<AuthService> = once_cell::sync::OnceCell::new();

/// Get the global authentication service instance
pub fn get_auth_service() -> &'static AuthService {
    AUTH_SERVICE.get_or_init(|| AuthService::new())
}
