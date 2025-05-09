// Secure Credential Storage for MCP Client
//
// This module provides secure credential storage using platform-specific secure enclaves:
// - On Windows: Windows Credential Manager (via wincred)
// - On macOS: Keychain (via keychain-rs)
// - On Linux: Secret Service API (via secret-service-rs)
// - Fallback: Encrypted local storage with obfuscation

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::config::config_path;
use crate::error::Result;
use crate::observability::metrics::{record_counter, record_gauge};
use crate::utils::security::{encrypt, decrypt};

#[cfg(target_os = "windows")]
use windows_credentials::{Credential, CredentialPersistence};

#[cfg(target_os = "macos")]
use keychain::{Keychain, KeychainItem, ItemClass};

#[cfg(all(unix, not(target_os = "macos")))]
use secret_service::{EncryptionType, SecretService, Collection};

// Constants for fallback storage
const CREDENTIALS_FILE: &str = "credentials.enc";
const SERVICE_NAME: &str = "mcp-client";
const MAX_CACHED_CREDENTIALS: usize = 100;

/// Serializable credential entry for fallback storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CredentialEntry {
    /// Key/Name of the credential
    key: String,
    
    /// Encrypted value
    encrypted_value: String,
    
    /// When the credential was last modified
    modified: SystemTime,
}

/// Memory-cached credential for performance
struct CachedCredential {
    /// The decrypted value
    value: String,
    
    /// When the credential was cached
    cached_at: Instant,
    
    /// Whether this credential has been modified and needs to be saved
    modified: bool,
}

/// Credential Manager for the MCP client
pub struct CredentialManager {
    /// Whether to use the platform secure enclave
    use_secure_enclave: bool,
    
    /// How long to cache credentials in memory (in seconds)
    cache_duration: u64,
    
    /// In-memory cache of credentials
    credential_cache: Arc<RwLock<HashMap<String, CachedCredential>>>,
    
    /// Last time credential file was loaded
    last_load: Arc<Mutex<Instant>>,
    
    /// Lock for file operations
    file_lock: Arc<Mutex<()>>,
}

impl CredentialManager {
    /// Create a new credential manager
    pub fn new(use_secure_enclave: bool, cache_duration: u64) -> Result<Self> {
        Ok(Self {
            use_secure_enclave,
            cache_duration,
            credential_cache: Arc::new(RwLock::new(HashMap::new())),
            last_load: Arc::new(Mutex::new(Instant::now())),
            file_lock: Arc::new(Mutex::new(())),
        })
    }
    
    /// Start the credential service
    pub fn start_service(&self) -> Result<()> {
        // Initialize platform-specific keychain access if needed
        if self.use_secure_enclave {
            self.initialize_secure_enclave()?;
        }
        
        // Create credentials file for fallback storage if needed
        if !self.use_secure_enclave {
            let credentials_path = config_path(CREDENTIALS_FILE);
            if !credentials_path.exists() {
                // Create empty credentials file
                fs::write(&credentials_path, encrypt("{}").map_err(|e| format!("Failed to encrypt credentials: {}", e))?)
                    .map_err(|e| format!("Failed to create credentials file: {}", e))?;
                    
                info!("Created fallback credentials file");
            }
        }
        
        // Set up credential expiration monitor
        let cache_duration = self.cache_duration;
        let credential_cache = self.credential_cache.clone();
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(60));
                
                // Check for expired credentials
                let mut cache = credential_cache.write().unwrap();
                let now = Instant::now();
                
                // Find keys to remove
                let expired_keys: Vec<String> = cache.iter()
                    .filter(|(_, cached)| now.duration_since(cached.cached_at) > Duration::from_secs(cache_duration))
                    .filter(|(_, cached)| !cached.modified) // Don't remove modified credentials
                    .map(|(key, _)| key.clone())
                    .collect();
                
                // Remove expired keys
                for key in expired_keys {
                    cache.remove(&key);
                }
                
                // Update metric for cached credentials
                record_gauge("security.credentials.cached_count", cache.len() as f64, None);
            }
        });
        
        info!("Credential service started in {} mode", 
            if self.use_secure_enclave { "secure enclave" } else { "fallback" });
        
        record_counter("security.credentials.service_started", 1.0, None);
        
        Ok(())
    }
    
    /// Initialize platform-specific secure enclave access
    fn initialize_secure_enclave(&self) -> Result<()> {
        if !self.use_secure_enclave {
            return Ok(());
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows doesn't need initialization for the Credential Manager
            debug!("Initialized Windows Credential Manager");
        }
        
        #[cfg(target_os = "macos")]
        {
            // Test keychain access
            match Keychain::default() {
                Ok(_) => debug!("Initialized macOS Keychain"),
                Err(e) => {
                    warn!("Failed to access macOS Keychain: {}", e);
                    return Err(format!("Failed to access macOS Keychain: {}", e).into());
                }
            }
        }
        
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Test Secret Service access
            match SecretService::new(EncryptionType::Dh) {
                Ok(_) => debug!("Initialized Secret Service API"),
                Err(e) => {
                    warn!("Failed to access Secret Service API: {}", e);
                    return Err(format!("Failed to access Secret Service API: {}", e).into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&self, use_secure_enclave: bool, cache_duration: u64) -> Result<()> {
        // If switching to secure enclave mode, initialize it
        if use_secure_enclave && !self.use_secure_enclave {
            self.initialize_secure_enclave()?;
        }
        
        // Update credentials storage if mode changed
        if use_secure_enclave != self.use_secure_enclave {
            // Get all credentials from current storage
            let credentials = self.get_all_credentials()?;
            
            // Update enclave setting
            let mut this = unsafe { &mut *(self as *const Self as *mut Self) };
            this.use_secure_enclave = use_secure_enclave;
            
            // Store all credentials in new storage
            for (key, value) in credentials {
                self.store_credential_internal(&key, &value)?;
            }
            
            info!("Migrated {} credentials to {} storage", 
                credentials.len(),
                if use_secure_enclave { "secure enclave" } else { "fallback" });
        }
        
        // Update cache duration (affects the cache cleanup thread)
        let mut this = unsafe { &mut *(self as *const Self as *mut Self) };
        this.cache_duration = cache_duration;
        
        Ok(())
    }
    
    /// Get all credential keys and values
    fn get_all_credentials(&self) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        
        if self.use_secure_enclave {
            // Platform-specific implementation to get all credentials
            #[cfg(target_os = "windows")]
            {
                // Windows doesn't have a good API for listing all credentials
                // We'll need to have stored a list of keys separately
                warn!("Windows Credential Manager doesn't support listing all credentials");
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS Keychain
                let keychain = Keychain::default()
                    .map_err(|e| format!("Failed to access keychain: {}", e))?;
                
                // We need to have stored a list of keys separately
                warn!("macOS Keychain doesn't support listing all credentials");
            }
            
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Secret Service API
                let ss = SecretService::new(EncryptionType::Dh)
                    .map_err(|e| format!("Failed to access Secret Service: {}", e))?;
                
                let collection = ss.get_default_collection()
                    .map_err(|e| format!("Failed to get default collection: {}", e))?;
                
                collection.unlock()
                    .map_err(|e| format!("Failed to unlock collection: {}", e))?;
                
                let items = collection.get_all_items()
                    .map_err(|e| format!("Failed to get items: {}", e))?;
                
                for item in items {
                    // Only get items for our service
                    let attributes = item.get_attributes()
                        .map_err(|e| format!("Failed to get item attributes: {}", e))?;
                    
                    let service = attributes.get("service");
                    if service != Some(&SERVICE_NAME.to_string()) {
                        continue;
                    }
                    
                    if let Some(key) = attributes.get("key") {
                        let secret = item.get_secret()
                            .map_err(|e| format!("Failed to get secret: {}", e))?;
                        
                        let value = String::from_utf8(secret)
                            .map_err(|_| "Invalid UTF-8 in secret".to_string())?;
                        
                        result.insert(key.clone(), value);
                    }
                }
            }
        } else {
            // Fallback storage
            let credentials_path = config_path(CREDENTIALS_FILE);
            
            if credentials_path.exists() {
                let encrypted_data = fs::read_to_string(&credentials_path)
                    .map_err(|e| format!("Failed to read credentials file: {}", e))?;
                
                let decrypted_data = decrypt(encrypted_data.as_bytes())
                    .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;
                
                let entries: HashMap<String, CredentialEntry> = serde_json::from_str(&decrypted_data)
                    .map_err(|e| format!("Failed to parse credentials: {}", e))?;
                
                for (_, entry) in entries {
                    let decrypted = decrypt(entry.encrypted_value.as_bytes())
                        .map_err(|e| format!("Failed to decrypt credential value: {}", e))?;
                    
                    result.insert(entry.key, decrypted);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Store a credential securely
    pub fn store_credential(&self, key: &str, value: &str) -> Result<()> {
        // Update cache
        {
            let mut cache = self.credential_cache.write().unwrap();
            
            // Check cache size limit
            if !cache.contains_key(key) && cache.len() >= MAX_CACHED_CREDENTIALS {
                // Find the oldest non-modified credential to remove
                if let Some(oldest_key) = cache.iter()
                    .filter(|(_, cached)| !cached.modified)
                    .min_by_key(|(_, cached)| cached.cached_at)
                    .map(|(key, _)| key.clone())
                {
                    cache.remove(&oldest_key);
                }
            }
            
            cache.insert(key.to_string(), CachedCredential {
                value: value.to_string(),
                cached_at: Instant::now(),
                modified: true,
            });
        }
        
        // Store in platform-specific secure storage
        self.store_credential_internal(key, value)?;
        
        // Mark as no longer modified in cache
        {
            let mut cache = self.credential_cache.write().unwrap();
            if let Some(cached) = cache.get_mut(key) {
                cached.modified = false;
            }
        }
        
        record_counter("security.credentials.stored", 1.0, None);
        
        Ok(())
    }
    
    /// Internal implementation of credential storage
    fn store_credential_internal(&self, key: &str, value: &str) -> Result<()> {
        if self.use_secure_enclave {
            // Platform-specific implementation
            #[cfg(target_os = "windows")]
            {
                // Windows Credential Manager
                let credential = Credential {
                    target_name: format!("{}:{}", SERVICE_NAME, key),
                    generic_password: value.as_bytes().to_vec(),
                    ..Credential::default()
                };
                
                credential.write()
                    .map_err(|e| format!("Failed to write to Windows Credential Manager: {}", e))?;
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS Keychain
                let keychain = Keychain::default()
                    .map_err(|e| format!("Failed to access keychain: {}", e))?;
                
                // Check if item exists
                let item_res = keychain.find_internet_password(
                    Some(SERVICE_NAME),
                    Some(key),
                    None,
                    None,
                    0,
                    ItemClass::InternetPassword,
                    None,
                );
                
                match item_res {
                    Ok(mut item) => {
                        // Update existing item
                        item.set_password(value)
                            .map_err(|e| format!("Failed to update keychain item: {}", e))?;
                    },
                    Err(_) => {
                        // Create new item
                        keychain.add_internet_password(
                            SERVICE_NAME,
                            key,
                            "",  // path
                            "",  // server
                            0,   // port
                            None, // auth type
                            None, // protocol
                            value,
                        ).map_err(|e| format!("Failed to add keychain item: {}", e))?;
                    }
                }
            }
            
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Secret Service API
                let ss = SecretService::new(EncryptionType::Dh)
                    .map_err(|e| format!("Failed to access Secret Service: {}", e))?;
                
                let collection = ss.get_default_collection()
                    .map_err(|e| format!("Failed to get default collection: {}", e))?;
                
                collection.unlock()
                    .map_err(|e| format!("Failed to unlock collection: {}", e))?;
                
                // Build attributes
                let mut attributes = HashMap::new();
                attributes.insert("service".to_string(), SERVICE_NAME.to_string());
                attributes.insert("key".to_string(), key.to_string());
                
                // Search for existing item
                let search = collection.search_items(attributes.clone())
                    .map_err(|e| format!("Failed to search for items: {}", e))?;
                
                if let Some(item) = search.first() {
                    // Update existing item
                    item.set_secret(value.as_bytes())
                        .map_err(|e| format!("Failed to update secret: {}", e))?;
                } else {
                    // Create new item
                    collection.create_item(
                        format!("{} - {}", SERVICE_NAME, key),
                        attributes,
                        value.as_bytes(),
                        true, // replace if exists
                        "text/plain",
                    ).map_err(|e| format!("Failed to create secret: {}", e))?;
                }
            }
        } else {
            // Fallback to encrypted file
            let _lock = self.file_lock.lock().unwrap();
            let credentials_path = config_path(CREDENTIALS_FILE);
            
            // Encrypt the value
            let encrypted_value = encrypt(value.as_bytes())
                .map_err(|e| format!("Failed to encrypt value: {}", e))?;
            
            // Read current credentials
            let mut entries: HashMap<String, CredentialEntry> = if credentials_path.exists() {
                let encrypted_data = fs::read_to_string(&credentials_path)
                    .map_err(|e| format!("Failed to read credentials file: {}", e))?;
                
                let decrypted_data = decrypt(encrypted_data.as_bytes())
                    .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;
                
                serde_json::from_str(&decrypted_data)
                    .map_err(|e| format!("Failed to parse credentials: {}", e))?
            } else {
                HashMap::new()
            };
            
            // Add or update entry
            entries.insert(key.to_string(), CredentialEntry {
                key: key.to_string(),
                encrypted_value: String::from_utf8(encrypted_value)
                    .map_err(|_| "Invalid UTF-8 in encrypted value".to_string())?,
                modified: SystemTime::now(),
            });
            
            // Write back to file
            let serialized = serde_json::to_string(&entries)
                .map_err(|e| format!("Failed to serialize credentials: {}", e))?;
            
            let encrypted_data = encrypt(serialized.as_bytes())
                .map_err(|e| format!("Failed to encrypt credentials: {}", e))?;
            
            fs::write(&credentials_path, encrypted_data)
                .map_err(|e| format!("Failed to write credentials file: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Retrieve a credential
    pub fn get_credential(&self, key: &str) -> Result<String> {
        // Check cache first
        {
            let cache = self.credential_cache.read().unwrap();
            if let Some(cached) = cache.get(key) {
                if cached.modified || cached.cached_at.elapsed() < Duration::from_secs(self.cache_duration) {
                    return Ok(cached.value.clone());
                }
            }
        }
        
        // Not in cache or expired, fetch from storage
        let value = self.get_credential_internal(key)?;
        
        // Update cache
        {
            let mut cache = self.credential_cache.write().unwrap();
            
            // Check cache size limit
            if !cache.contains_key(key) && cache.len() >= MAX_CACHED_CREDENTIALS {
                // Find the oldest non-modified credential to remove
                if let Some(oldest_key) = cache.iter()
                    .filter(|(_, cached)| !cached.modified)
                    .min_by_key(|(_, cached)| cached.cached_at)
                    .map(|(key, _)| key.clone())
                {
                    cache.remove(&oldest_key);
                }
            }
            
            cache.insert(key.to_string(), CachedCredential {
                value: value.clone(),
                cached_at: Instant::now(),
                modified: false,
            });
        }
        
        record_counter("security.credentials.retrieved", 1.0, None);
        
        Ok(value)
    }
    
    /// Internal implementation of credential retrieval
    fn get_credential_internal(&self, key: &str) -> Result<String> {
        if self.use_secure_enclave {
            // Platform-specific implementation
            #[cfg(target_os = "windows")]
            {
                // Windows Credential Manager
                let credential = Credential::get(
                    &format!("{}:{}", SERVICE_NAME, key),
                    false, // all_credentials
                ).map_err(|e| format!("Failed to read from Windows Credential Manager: {}", e))?;
                
                Ok(String::from_utf8(credential.generic_password)
                    .map_err(|_| "Invalid UTF-8 in credential".to_string())?)
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS Keychain
                let keychain = Keychain::default()
                    .map_err(|e| format!("Failed to access keychain: {}", e))?;
                
                let (_item, password) = keychain.find_internet_password(
                    Some(SERVICE_NAME),
                    Some(key),
                    None,
                    None,
                    0,
                    ItemClass::InternetPassword,
                    None,
                ).map_err(|e| format!("Failed to find keychain item: {}", e))?;
                
                Ok(password)
            }
            
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Secret Service API
                let ss = SecretService::new(EncryptionType::Dh)
                    .map_err(|e| format!("Failed to access Secret Service: {}", e))?;
                
                let collection = ss.get_default_collection()
                    .map_err(|e| format!("Failed to get default collection: {}", e))?;
                
                collection.unlock()
                    .map_err(|e| format!("Failed to unlock collection: {}", e))?;
                
                // Build attributes
                let mut attributes = HashMap::new();
                attributes.insert("service".to_string(), SERVICE_NAME.to_string());
                attributes.insert("key".to_string(), key.to_string());
                
                // Search for existing item
                let search = collection.search_items(attributes)
                    .map_err(|e| format!("Failed to search for items: {}", e))?;
                
                if let Some(item) = search.first() {
                    let secret = item.get_secret()
                        .map_err(|e| format!("Failed to get secret: {}", e))?;
                    
                    Ok(String::from_utf8(secret)
                        .map_err(|_| "Invalid UTF-8 in secret".to_string())?)
                } else {
                    Err(format!("Credential '{}' not found", key).into())
                }
            }
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
            {
                Err("Secure enclave not supported on this platform".into())
            }
        } else {
            // Fallback to encrypted file
            let credentials_path = config_path(CREDENTIALS_FILE);
            
            if !credentials_path.exists() {
                return Err(format!("Credential '{}' not found", key).into());
            }
            
            let encrypted_data = fs::read_to_string(&credentials_path)
                .map_err(|e| format!("Failed to read credentials file: {}", e))?;
            
            let decrypted_data = decrypt(encrypted_data.as_bytes())
                .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;
            
            let entries: HashMap<String, CredentialEntry> = serde_json::from_str(&decrypted_data)
                .map_err(|e| format!("Failed to parse credentials: {}", e))?;
            
            if let Some(entry) = entries.get(key) {
                let decrypted = decrypt(entry.encrypted_value.as_bytes())
                    .map_err(|e| format!("Failed to decrypt credential value: {}", e))?;
                
                Ok(decrypted)
            } else {
                Err(format!("Credential '{}' not found", key).into())
            }
        }
    }
    
    /// Delete a credential
    pub fn delete_credential(&self, key: &str) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.credential_cache.write().unwrap();
            cache.remove(key);
        }
        
        if self.use_secure_enclave {
            // Platform-specific implementation
            #[cfg(target_os = "windows")]
            {
                // Windows Credential Manager
                Credential::delete(
                    &format!("{}:{}", SERVICE_NAME, key),
                    false, // all_credentials
                ).map_err(|e| format!("Failed to delete from Windows Credential Manager: {}", e))?;
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS Keychain
                let keychain = Keychain::default()
                    .map_err(|e| format!("Failed to access keychain: {}", e))?;
                
                let (item, _) = keychain.find_internet_password(
                    Some(SERVICE_NAME),
                    Some(key),
                    None,
                    None,
                    0,
                    ItemClass::InternetPassword,
                    None,
                ).map_err(|e| format!("Failed to find keychain item: {}", e))?;
                
                item.delete()
                    .map_err(|e| format!("Failed to delete keychain item: {}", e))?;
            }
            
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Secret Service API
                let ss = SecretService::new(EncryptionType::Dh)
                    .map_err(|e| format!("Failed to access Secret Service: {}", e))?;
                
                let collection = ss.get_default_collection()
                    .map_err(|e| format!("Failed to get default collection: {}", e))?;
                
                collection.unlock()
                    .map_err(|e| format!("Failed to unlock collection: {}", e))?;
                
                // Build attributes
                let mut attributes = HashMap::new();
                attributes.insert("service".to_string(), SERVICE_NAME.to_string());
                attributes.insert("key".to_string(), key.to_string());
                
                // Search for existing item
                let search = collection.search_items(attributes)
                    .map_err(|e| format!("Failed to search for items: {}", e))?;
                
                if let Some(item) = search.first() {
                    item.delete()
                        .map_err(|e| format!("Failed to delete secret: {}", e))?;
                } else {
                    return Err(format!("Credential '{}' not found", key).into());
                }
            }
        } else {
            // Fallback to encrypted file
            let _lock = self.file_lock.lock().unwrap();
            let credentials_path = config_path(CREDENTIALS_FILE);
            
            if !credentials_path.exists() {
                return Err(format!("Credential '{}' not found", key).into());
            }
            
            let encrypted_data = fs::read_to_string(&credentials_path)
                .map_err(|e| format!("Failed to read credentials file: {}", e))?;
            
            let decrypted_data = decrypt(encrypted_data.as_bytes())
                .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;
            
            let mut entries: HashMap<String, CredentialEntry> = serde_json::from_str(&decrypted_data)
                .map_err(|e| format!("Failed to parse credentials: {}", e))?;
            
            if entries.remove(key).is_none() {
                return Err(format!("Credential '{}' not found", key).into());
            }
            
            // Write back to file
            let serialized = serde_json::to_string(&entries)
                .map_err(|e| format!("Failed to serialize credentials: {}", e))?;
            
            let encrypted_data = encrypt(serialized.as_bytes())
                .map_err(|e| format!("Failed to encrypt credentials: {}", e))?;
            
            fs::write(&credentials_path, encrypted_data)
                .map_err(|e| format!("Failed to write credentials file: {}", e))?;
        }
        
        record_counter("security.credentials.deleted", 1.0, None);
        
        Ok(())
    }
    
    /// List all credential keys
    pub fn list_credential_keys(&self) -> Result<Vec<String>> {
        if self.use_secure_enclave {
            // Platform-specific implementation
            #[cfg(target_os = "windows")]
            {
                // Windows doesn't have a good API for listing all credentials
                warn!("Windows Credential Manager doesn't support listing all credentials");
                return Ok(Vec::new());
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS Keychain doesn't have a good API for listing all items of a certain kind
                warn!("macOS Keychain doesn't support listing all credentials");
                return Ok(Vec::new());
            }
            
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Secret Service API
                let ss = SecretService::new(EncryptionType::Dh)
                    .map_err(|e| format!("Failed to access Secret Service: {}", e))?;
                
                let collection = ss.get_default_collection()
                    .map_err(|e| format!("Failed to get default collection: {}", e))?;
                
                collection.unlock()
                    .map_err(|e| format!("Failed to unlock collection: {}", e))?;
                
                // Build attributes for search
                let mut attributes = HashMap::new();
                attributes.insert("service".to_string(), SERVICE_NAME.to_string());
                
                // Search for items
                let search = collection.search_items(attributes)
                    .map_err(|e| format!("Failed to search for items: {}", e))?;
                
                let mut keys = Vec::new();
                for item in search {
                    let item_attrs = item.get_attributes()
                        .map_err(|e| format!("Failed to get item attributes: {}", e))?;
                    
                    if let Some(key) = item_attrs.get("key") {
                        keys.push(key.clone());
                    }
                }
                
                return Ok(keys);
            }
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
            {
                warn!("Secure enclave not supported on this platform");
                return Ok(Vec::new());
            }
        } else {
            // Fallback to encrypted file
            let credentials_path = config_path(CREDENTIALS_FILE);
            
            if !credentials_path.exists() {
                return Ok(Vec::new());
            }
            
            let encrypted_data = fs::read_to_string(&credentials_path)
                .map_err(|e| format!("Failed to read credentials file: {}", e))?;
            
            let decrypted_data = decrypt(encrypted_data.as_bytes())
                .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;
            
            let entries: HashMap<String, CredentialEntry> = serde_json::from_str(&decrypted_data)
                .map_err(|e| format!("Failed to parse credentials: {}", e))?;
            
            Ok(entries.keys().cloned().collect())
        }
    }
    
    /// Check if a credential exists
    pub fn has_credential(&self, key: &str) -> Result<bool> {
        // Check cache first
        {
            let cache = self.credential_cache.read().unwrap();
            if cache.contains_key(key) {
                return Ok(true);
            }
        }
        
        if self.use_secure_enclave {
            // Platform-specific implementation
            #[cfg(target_os = "windows")]
            {
                // Windows Credential Manager
                match Credential::get(
                    &format!("{}:{}", SERVICE_NAME, key),
                    false, // all_credentials
                ) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS Keychain
                let keychain = Keychain::default()
                    .map_err(|e| format!("Failed to access keychain: {}", e))?;
                
                match keychain.find_internet_password(
                    Some(SERVICE_NAME),
                    Some(key),
                    None,
                    None,
                    0,
                    ItemClass::InternetPassword,
                    None,
                ) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Secret Service API
                let ss = SecretService::new(EncryptionType::Dh)
                    .map_err(|e| format!("Failed to access Secret Service: {}", e))?;
                
                let collection = ss.get_default_collection()
                    .map_err(|e| format!("Failed to get default collection: {}", e))?;
                
                collection.unlock()
                    .map_err(|e| format!("Failed to unlock collection: {}", e))?;
                
                // Build attributes
                let mut attributes = HashMap::new();
                attributes.insert("service".to_string(), SERVICE_NAME.to_string());
                attributes.insert("key".to_string(), key.to_string());
                
                // Search for existing item
                let search = collection.search_items(attributes)
                    .map_err(|e| format!("Failed to search for items: {}", e))?;
                
                Ok(!search.is_empty())
            }
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
            {
                Err("Secure enclave not supported on this platform".into())
            }
        } else {
            // Fallback to encrypted file
            let credentials_path = config_path(CREDENTIALS_FILE);
            
            if !credentials_path.exists() {
                return Ok(false);
            }
            
            let encrypted_data = fs::read_to_string(&credentials_path)
                .map_err(|e| format!("Failed to read credentials file: {}", e))?;
            
            let decrypted_data = decrypt(encrypted_data.as_bytes())
                .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;
            
            let entries: HashMap<String, CredentialEntry> = serde_json::from_str(&decrypted_data)
                .map_err(|e| format!("Failed to parse credentials: {}", e))?;
            
            Ok(entries.contains_key(key))
        }
    }
}
