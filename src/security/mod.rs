// Security and Privacy Module for MCP Client
//
// This module provides comprehensive security and privacy features:
// - End-to-end encryption for data synchronization
// - Secure credential storage using platform-specific secure enclaves
// - Data flow tracking and visualization
// - Granular permission management

pub mod e2ee;
pub mod credentials;
pub mod data_flow;
pub mod permissions;

use std::sync::{Arc, RwLock};
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::observability::metrics::record_counter;
use crate::observability::telemetry::track_feature_usage;

/// Security configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether end-to-end encryption is enabled for sync
    pub e2ee_enabled: bool,
    
    /// Whether to use platform secure enclave for credential storage
    pub use_secure_enclave: bool,
    
    /// Whether to collect and display data flow information
    pub data_flow_tracking_enabled: bool,
    
    /// Default permission level for new features and integrations
    pub default_permission_level: PermissionLevel,
    
    /// Whether to prompt for permission changes
    pub interactive_permissions: bool,
    
    /// Whether to anonymize telemetry data
    pub anonymize_telemetry: bool,
    
    /// Whether to encrypt local storage
    pub encrypt_local_storage: bool,
    
    /// How long to cache credentials in memory (in seconds)
    pub credential_cache_duration: u64,
    
    /// Whether to clear clipboard after sensitive operations
    pub clipboard_security_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            e2ee_enabled: true,
            use_secure_enclave: true,
            data_flow_tracking_enabled: true,
            default_permission_level: PermissionLevel::AskEveryTime,
            interactive_permissions: true,
            anonymize_telemetry: true,
            encrypt_local_storage: true,
            credential_cache_duration: 600, // 10 minutes
            clipboard_security_enabled: true,
        }
    }
}

/// Permission levels for features and capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Always allow this operation without asking
    AlwaysAllow,
    
    /// Ask for confirmation the first time only
    AskFirstTime,
    
    /// Ask for confirmation every time
    AskEveryTime,
    
    /// Never allow this operation
    NeverAllow,
}

/// Data classification levels for privacy controls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    /// Public data that can be shared without restriction
    Public,
    
    /// Personal data that should be handled with care
    Personal,
    
    /// Sensitive data that requires strong privacy controls
    Sensitive,
    
    /// Highly confidential data that should never leave the device
    Confidential,
}

/// Security manager for the MCP client
pub struct SecurityManager {
    /// Configuration
    config: Arc<RwLock<SecurityConfig>>,
    
    /// E2EE Manager for encryption
    e2ee_manager: Arc<RwLock<e2ee::E2EEManager>>,
    
    /// Credential storage manager
    credential_manager: Arc<RwLock<credentials::CredentialManager>>,
    
    /// Data flow tracking manager
    data_flow_manager: Arc<RwLock<data_flow::DataFlowManager>>,
    
    /// Permission manager
    permission_manager: Arc<RwLock<permissions::PermissionManager>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        // Initialize E2EE Manager
        let e2ee_manager = e2ee::E2EEManager::new(config.e2ee_enabled)?;
        
        // Initialize Credential Manager
        let credential_manager = credentials::CredentialManager::new(
            config.use_secure_enclave,
            config.credential_cache_duration,
        )?;
        
        // Initialize Data Flow Manager
        let data_flow_manager = data_flow::DataFlowManager::new(
            config.data_flow_tracking_enabled,
        )?;
        
        // Initialize Permission Manager
        let permission_manager = permissions::PermissionManager::new(
            config.default_permission_level,
            config.interactive_permissions,
        )?;
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            e2ee_manager: Arc::new(RwLock::new(e2ee_manager)),
            credential_manager: Arc::new(RwLock::new(credential_manager)),
            data_flow_manager: Arc::new(RwLock::new(data_flow_manager)),
            permission_manager: Arc::new(RwLock::new(permission_manager)),
        })
    }
    
    /// Start security services
    pub fn start_services(&self) -> Result<()> {
        // Start E2EE service
        self.e2ee_manager.read().unwrap().start_service()?;
        
        // Start credential management service
        self.credential_manager.read().unwrap().start_service()?;
        
        // Start data flow tracking service
        self.data_flow_manager.read().unwrap().start_service()?;
        
        // Start permission management service
        self.permission_manager.read().unwrap().start_service()?;
        
        info!("Security services started");
        record_counter("security.services.started", 1.0, None);
        
        Ok(())
    }
    
    /// Get the E2EE manager
    pub fn get_e2ee_manager(&self) -> Arc<RwLock<e2ee::E2EEManager>> {
        self.e2ee_manager.clone()
    }
    
    /// Get the credential manager
    pub fn get_credential_manager(&self) -> Arc<RwLock<credentials::CredentialManager>> {
        self.credential_manager.clone()
    }
    
    /// Get the data flow manager
    pub fn get_data_flow_manager(&self) -> Arc<RwLock<data_flow::DataFlowManager>> {
        self.data_flow_manager.clone()
    }
    
    /// Get the permission manager
    pub fn get_permission_manager(&self) -> Arc<RwLock<permissions::PermissionManager>> {
        self.permission_manager.clone()
    }
    
    /// Check if a permission is granted
    pub fn check_permission(&self, permission: &str) -> Result<bool> {
        self.permission_manager.read().unwrap().check_permission(permission)
    }
    
    /// Request permission for an operation
    pub fn request_permission(&self, permission: &str, reason: &str) -> Result<bool> {
        let granted = self.permission_manager.write().unwrap().request_permission(permission, reason)?;
        
        // Track permission request
        let mut properties = std::collections::HashMap::new();
        properties.insert("permission".to_string(), permission.to_string());
        properties.insert("granted".to_string(), granted.to_string());
        track_feature_usage("security.permission_request", Some(properties));
        
        Ok(granted)
    }
    
    /// Store a credential securely
    pub fn store_credential(&self, key: &str, value: &str) -> Result<()> {
        self.credential_manager.write().unwrap().store_credential(key, value)?;
        
        // Track data flow
        if self.config.read().unwrap().data_flow_tracking_enabled {
            self.data_flow_manager.write().unwrap().track_data_flow(
                "credential_storage",
                key,
                DataClassification::Sensitive,
                "secure_enclave",
            )?;
        }
        
        // Track credential storage
        record_counter("security.credential.stored", 1.0, None);
        
        Ok(())
    }
    
    /// Retrieve a credential
    pub fn get_credential(&self, key: &str) -> Result<String> {
        let credential = self.credential_manager.write().unwrap().get_credential(key)?;
        
        // Track data flow
        if self.config.read().unwrap().data_flow_tracking_enabled {
            self.data_flow_manager.write().unwrap().track_data_flow(
                "credential_retrieval",
                key,
                DataClassification::Sensitive,
                "application",
            )?;
        }
        
        // Track credential retrieval
        record_counter("security.credential.retrieved", 1.0, None);
        
        Ok(credential)
    }
    
    /// Encrypt data for synchronization
    pub fn encrypt_for_sync(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encrypted = self.e2ee_manager.read().unwrap().encrypt_data(data)?;
        
        // Track encryption operation
        record_counter("security.e2ee.encrypt", 1.0, None);
        
        Ok(encrypted)
    }
    
    /// Decrypt data from synchronization
    pub fn decrypt_from_sync(&self, data: &[u8]) -> Result<Vec<u8>> {
        let decrypted = self.e2ee_manager.read().unwrap().decrypt_data(data)?;
        
        // Track decryption operation
        record_counter("security.e2ee.decrypt", 1.0, None);
        
        Ok(decrypted)
    }
    
    /// Update security configuration
    pub fn update_config(&self, new_config: SecurityConfig) -> Result<()> {
        // Update managers with new config values
        self.e2ee_manager.write().unwrap().set_enabled(new_config.e2ee_enabled)?;
        self.credential_manager.write().unwrap().update_config(
            new_config.use_secure_enclave,
            new_config.credential_cache_duration,
        )?;
        self.data_flow_manager.write().unwrap().set_enabled(
            new_config.data_flow_tracking_enabled,
        )?;
        self.permission_manager.write().unwrap().update_config(
            new_config.default_permission_level,
            new_config.interactive_permissions,
        )?;
        
        // Update main config
        *self.config.write().unwrap() = new_config;
        
        info!("Security configuration updated");
        
        Ok(())
    }
    
    /// Get the current security configuration
    pub fn get_config(&self) -> SecurityConfig {
        self.config.read().unwrap().clone()
    }
}

// Global security manager
lazy_static::lazy_static! {
    pub static ref SECURITY_MANAGER: Arc<RwLock<Option<SecurityManager>>> = Arc::new(RwLock::new(None));
}

/// Initialize the security manager
pub fn init_security_manager(config: Option<SecurityConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    
    let manager = SecurityManager::new(config)?;
    manager.start_services()?;
    
    let mut global_manager = SECURITY_MANAGER.write().unwrap();
    *global_manager = Some(manager);
    
    info!("Security manager initialized");
    
    Ok(())
}

/// Get reference to the security manager
pub fn get_security_manager() -> Result<Arc<SecurityManager>> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => Ok(Arc::new(manager.clone())),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}

// Helper functions for common security operations

/// Check if a permission is granted
pub fn check_permission(permission: &str) -> Result<bool> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => manager.check_permission(permission),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}

/// Request permission for an operation
pub fn request_permission(permission: &str, reason: &str) -> Result<bool> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => manager.request_permission(permission, reason),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}

/// Store a credential securely
pub fn store_credential(key: &str, value: &str) -> Result<()> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => manager.store_credential(key, value),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}

/// Retrieve a credential
pub fn get_credential(key: &str) -> Result<String> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => manager.get_credential(key),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}

/// Encrypt data for synchronization
pub fn encrypt_for_sync(data: &[u8]) -> Result<Vec<u8>> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => manager.encrypt_for_sync(data),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}

/// Decrypt data from synchronization
pub fn decrypt_from_sync(data: &[u8]) -> Result<Vec<u8>> {
    match SECURITY_MANAGER.read().unwrap().as_ref() {
        Some(manager) => manager.decrypt_from_sync(data),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Security manager not initialized",
        ).into()),
    }
}
