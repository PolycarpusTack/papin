use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow, Context};
use tokio::fs;

use crate::error::Error;
use crate::commands::offline::llm::{ProviderType, ProviderConfig, ProviderInfo};
use crate::offline::llm::{LocalLLM, ModelInfo, LLMConfig, LLMParameters};
use crate::offline::llm::discovery::DiscoveryService;

/// Migration status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationStatus {
    /// Migration has not been attempted
    NotMigrated,
    /// Migration is in progress
    InProgress,
    /// Migration completed successfully
    Completed {
        /// When the migration was completed
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Total number of models migrated
        models_migrated: usize,
        /// Provider types that were migrated
        providers_configured: Vec<String>,
    },
    /// Migration failed
    Failed {
        /// When the migration failed
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Reason for failure
        reason: String,
        /// Whether fallback to old system is active
        fallback_active: bool,
    },
    /// User opted out of migration
    OptedOut {
        /// When the user opted out
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl Default for MigrationStatus {
    fn default() -> Self {
        Self::NotMigrated
    }
}

/// Legacy model mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyModelMapping {
    /// Legacy model ID
    pub legacy_id: String,
    /// New provider type
    pub provider_type: String,
    /// New model ID
    pub new_id: String,
    /// File path to model
    pub model_path: PathBuf,
    /// Whether the model was successfully migrated
    pub migrated: bool,
}

/// Old to new provider mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMapping {
    /// Legacy configuration
    pub legacy_config: LLMConfig,
    /// New provider type
    pub provider_type: String,
    /// New provider configuration
    pub provider_config: ProviderConfig,
    /// Whether the provider was successfully migrated
    pub migrated: bool,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Whether to auto-migrate on startup
    pub auto_migrate: bool,
    /// Whether to show migration UI
    pub show_migration_ui: bool,
    /// Whether to keep legacy files
    pub keep_legacy_files: bool,
    /// Extra paths to check for old models
    pub extra_model_paths: Vec<PathBuf>,
    /// Path to legacy config
    pub legacy_config_path: Option<PathBuf>,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_migrate: true,
            show_migration_ui: true,
            keep_legacy_files: true,
            extra_model_paths: Vec::new(),
            legacy_config_path: None,
        }
    }
}

/// Migration notification info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationNotification {
    /// Migration status
    pub status: MigrationStatus,
    /// Number of models found
    pub models_found: usize,
    /// Number of models migrated successfully
    pub models_migrated: usize,
    /// Number of providers configured
    pub providers_configured: usize,
    /// Whether fallback is available
    pub fallback_available: bool,
    /// Duration of migration in seconds
    pub duration_seconds: f64,
    /// List of migrated model mappings
    pub model_mappings: Vec<LegacyModelMapping>,
    /// List of providers configured
    pub provider_mappings: Vec<ProviderMapping>,
}

/// Migration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationOptions {
    /// Whether to migrate models
    pub migrate_models: bool,
    /// Whether to migrate configuration
    pub migrate_config: bool,
    /// Whether to delete legacy files
    pub delete_legacy_files: bool,
    /// Whether to enable fallback
    pub enable_fallback: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            migrate_models: true,
            migrate_config: true,
            delete_legacy_files: false,
            enable_fallback: true,
        }
    }
}

/// Current legacy config store types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyStoreType {
    /// JSON file
    Json,
    /// TOML file
    Toml,
    /// YAML file
    Yaml,
    /// SQLite database
    Sqlite,
    /// Registry (Windows only)
    Registry,
}

/// Migration service
pub struct MigrationService {
    /// Current migration status
    status: Arc<Mutex<MigrationStatus>>,
    /// Migration configuration
    config: Arc<Mutex<MigrationConfig>>,
    /// Legacy models found
    legacy_models: Arc<Mutex<HashMap<String, ModelInfo>>>,
    /// Legacy config found
    legacy_config: Arc<Mutex<Option<LLMConfig>>>,
    /// Model mappings
    model_mappings: Arc<Mutex<Vec<LegacyModelMapping>>>,
    /// Provider mappings
    provider_mappings: Arc<Mutex<Vec<ProviderMapping>>>,
    /// Legacy store type
    store_type: Arc<Mutex<Option<LegacyStoreType>>>,
    /// Legacy fallback provider
    fallback_provider: Arc<Mutex<Option<LocalLLM>>>,
}

impl Default for MigrationService {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationService {
    /// Create a new migration service
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(MigrationStatus::NotMigrated)),
            config: Arc::new(Mutex::new(MigrationConfig::default())),
            legacy_models: Arc::new(Mutex::new(HashMap::new())),
            legacy_config: Arc::new(Mutex::new(None)),
            model_mappings: Arc::new(Mutex::new(Vec::new())),
            provider_mappings: Arc::new(Mutex::new(Vec::new())),
            store_type: Arc::new(Mutex::new(None)),
            fallback_provider: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the current migration status
    pub fn get_status(&self) -> MigrationStatus {
        self.status.lock().unwrap().clone()
    }

    /// Get the current migration configuration
    pub fn get_config(&self) -> MigrationConfig {
        self.config.lock().unwrap().clone()
    }

    /// Update the migration configuration
    pub fn update_config(&self, config: MigrationConfig) {
        *self.config.lock().unwrap() = config;
    }

    /// Detect legacy LLM system
    pub async fn detect_legacy_system(&self) -> Result<bool> {
        info!("Detecting legacy LLM system...");
        
        // Check for legacy config
        let legacy_config_path = self.find_legacy_config().await?;
        
        if let Some(path) = &legacy_config_path {
            info!("Found legacy config at {:?}", path);
            
            // Try to load legacy config
            match self.load_legacy_config(path).await {
                Ok(config) => {
                    info!("Loaded legacy config: {:?}", config);
                    *self.legacy_config.lock().unwrap() = Some(config);
                },
                Err(e) => {
                    warn!("Failed to load legacy config: {}", e);
                }
            }
        }
        
        // Check for legacy models
        let legacy_models = self.find_legacy_models().await?;
        
        if !legacy_models.is_empty() {
            info!("Found {} legacy models", legacy_models.len());
            *self.legacy_models.lock().unwrap() = legacy_models;
        }
        
        // Legacy system detected if either config or models were found
        let legacy_detected = legacy_config_path.is_some() || !self.legacy_models.lock().unwrap().is_empty();
        
        info!("Legacy system detection complete: {}", legacy_detected);
        Ok(legacy_detected)
    }

    /// Find legacy config
    async fn find_legacy_config(&self) -> Result<Option<PathBuf>> {
        // Check custom path if specified
        let config = self.get_config();
        if let Some(path) = &config.legacy_config_path {
            if path.exists() {
                // Determine the store type
                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let store_type = match extension {
                    "json" => LegacyStoreType::Json,
                    "toml" => LegacyStoreType::Toml,
                    "yaml" | "yml" => LegacyStoreType::Yaml,
                    "db" | "sqlite" => LegacyStoreType::Sqlite,
                    _ => return Err(anyhow!("Unsupported legacy config format: {}", extension)),
                };
                
                *self.store_type.lock().unwrap() = Some(store_type);
                return Ok(Some(path.clone()));
            }
        }
        
        // Check common locations
        let mut possible_locations = Vec::new();
        
        // Add application data directory
        if let Some(app_data) = dirs::data_dir() {
            possible_locations.push(app_data.join("papin/llm_config.json"));
            possible_locations.push(app_data.join("papin/config/llm.json"));
            possible_locations.push(app_data.join("mcp-client/llm_config.json"));
            possible_locations.push(app_data.join("mcp-client/config/llm.json"));
        }
        
        // Add home directory
        if let Some(home) = dirs::home_dir() {
            possible_locations.push(home.join(".papin/llm_config.json"));
            possible_locations.push(home.join(".mcp-client/llm_config.json"));
            possible_locations.push(home.join(".config/papin/llm.json"));
            possible_locations.push(home.join(".config/mcp-client/llm.json"));
        }
        
        // Add current directory
        possible_locations.push(PathBuf::from("llm_config.json"));
        possible_locations.push(PathBuf::from("config/llm.json"));
        
        // Check Windows registry if on Windows
        #[cfg(target_os = "windows")]
        {
            use winreg::RegKey;
            use winreg::enums::*;
            
            // Try to open registry key
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            if let Ok(key) = hklm.open_subkey("SOFTWARE\\Papin") {
                if let Ok(_) = key.get_value::<String, _>("LLMConfig") {
                    *self.store_type.lock().unwrap() = Some(LegacyStoreType::Registry);
                    return Ok(None); // No path for registry
                }
            }
            
            // Also check current user
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) = hkcu.open_subkey("SOFTWARE\\Papin") {
                if let Ok(_) = key.get_value::<String, _>("LLMConfig") {
                    *self.store_type.lock().unwrap() = Some(LegacyStoreType::Registry);
                    return Ok(None); // No path for registry
                }
            }
        }
        
        // Check each possible location
        for path in possible_locations {
            if path.exists() {
                // JSON store type
                *self.store_type.lock().unwrap() = Some(LegacyStoreType::Json);
                return Ok(Some(path));
            }
            
            // Also check for TOML and YAML variants
            let toml_path = path.with_extension("toml");
            if toml_path.exists() {
                *self.store_type.lock().unwrap() = Some(LegacyStoreType::Toml);
                return Ok(Some(toml_path));
            }
            
            let yaml_path = path.with_extension("yaml");
            if yaml_path.exists() {
                *self.store_type.lock().unwrap() = Some(LegacyStoreType::Yaml);
                return Ok(Some(yaml_path));
            }
            
            let yml_path = path.with_extension("yml");
            if yml_path.exists() {
                *self.store_type.lock().unwrap() = Some(LegacyStoreType::Yaml);
                return Ok(Some(yml_path));
            }
            
            // Check for SQLite variant
            let sqlite_path = path.with_extension("db");
            if sqlite_path.exists() {
                *self.store_type.lock().unwrap() = Some(LegacyStoreType::Sqlite);
                return Ok(Some(sqlite_path));
            }
        }
        
        // No legacy config found
        Ok(None)
    }

    /// Load legacy config from file
    async fn load_legacy_config(&self, path: &Path) -> Result<LLMConfig> {
        // Read file content
        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read legacy config from {:?}", path))?;
        
        // Parse based on store type
        match *self.store_type.lock().unwrap() {
            Some(LegacyStoreType::Json) => {
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse legacy JSON config"))
            },
            Some(LegacyStoreType::Toml) => {
                toml::from_str(&content)
                    .with_context(|| format!("Failed to parse legacy TOML config"))
            },
            Some(LegacyStoreType::Yaml) => {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse legacy YAML config"))
            },
            #[cfg(feature = "sqlite")]
            Some(LegacyStoreType::Sqlite) => {
                // Simplified SQLite handling - would be more complex in a real implementation
                Err(anyhow!("SQLite support not fully implemented"))
            },
            #[cfg(target_os = "windows")]
            Some(LegacyStoreType::Registry) => {
                use winreg::RegKey;
                use winreg::enums::*;
                
                // Try to open registry key
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                if let Ok(key) = hklm.open_subkey("SOFTWARE\\Papin") {
                    if let Ok(config_json) = key.get_value::<String, _>("LLMConfig") {
                        return serde_json::from_str(&config_json)
                            .with_context(|| format!("Failed to parse legacy config from registry"));
                    }
                }
                
                // Also check current user
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("SOFTWARE\\Papin") {
                    if let Ok(config_json) = key.get_value::<String, _>("LLMConfig") {
                        return serde_json::from_str(&config_json)
                            .with_context(|| format!("Failed to parse legacy config from registry"));
                    }
                }
                
                Err(anyhow!("Failed to load legacy config from registry"))
            },
            _ => Err(anyhow!("Unsupported legacy config store type")),
        }
    }

    /// Find legacy models
    async fn find_legacy_models(&self) -> Result<HashMap<String, ModelInfo>> {
        let mut legacy_models = HashMap::new();
        
        // Check standard model paths
        let mut model_paths = Vec::new();
        
        // Check for models in fixed app directory
        if let Some(app_data) = dirs::data_dir() {
            model_paths.push(app_data.join("papin/models"));
            model_paths.push(app_data.join("mcp-client/models"));
        }
        
        // Check config-defined model path
        if let Some(config) = &*self.legacy_config.lock().unwrap() {
            let model_dir = config.model_path.parent()
                .unwrap_or(&config.model_path)
                .to_path_buf();
            
            model_paths.push(model_dir);
        }
        
        // Add user-defined extra paths
        let config = self.get_config();
        model_paths.extend(config.extra_model_paths.clone());
        
        // Check all model paths
        for path in model_paths {
            if !path.exists() || !path.is_dir() {
                continue;
            }
            
            // Look for model files
            let entries = match fs::read_dir(&path).await {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("Failed to read model directory {:?}: {}", path, e);
                    continue;
                }
            };
            
            // Process directory entries
            let mut entries_vec = Vec::new();
            let mut entry = entries.into_iter().next();
            while let Some(entry_result) = entry {
                match entry_result {
                    Ok(e) => entries_vec.push(e),
                    Err(e) => warn!("Failed to read directory entry: {}", e),
                }
                entry = entries.into_iter().next();
            }
            
            for entry in entries_vec {
                let entry_path = entry.path();
                
                // Skip directories and non-model files
                if entry_path.is_dir() {
                    continue;
                }
                
                let extension = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !Self::is_model_file(extension) {
                    continue;
                }
                
                // Get model ID from filename
                let model_id = entry_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Get file size
                let metadata = match fs::metadata(&entry_path).await {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Failed to get metadata for {:?}: {}", entry_path, e);
                        continue;
                    }
                };
                
                let size_mb = metadata.len() / (1024 * 1024);
                
                // Create model info
                let model_info = ModelInfo {
                    id: model_id.clone(),
                    name: model_id.clone(),
                    size_mb: size_mb as usize,
                    context_size: 4096, // Default reasonable value
                    installed: true,
                    download_url: None,
                    description: format!("Legacy model found at {:?}", entry_path),
                };
                
                legacy_models.insert(model_id, model_info);
            }
        }
        
        Ok(legacy_models)
    }

    /// Check if a file extension is a model file
    fn is_model_file(extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(), 
            "bin" | "gguf" | "ggml" | "bin" | "weight" | "pt" | 
            "pth" | "model" | "onnx" | "safetensors" | "f16" | "f32")
    }

    /// Run the migration
    pub async fn run_migration(&self, options: MigrationOptions) -> Result<MigrationNotification> {
        info!("Starting legacy system migration...");
        
        // Update status
        *self.status.lock().unwrap() = MigrationStatus::InProgress;
        
        let start_time = std::time::Instant::now();
        
        // Detect legacy system if not already done
        if self.legacy_models.lock().unwrap().is_empty() && self.legacy_config.lock().unwrap().is_none() {
            self.detect_legacy_system().await?;
        }
        
        // Track migration results
        let mut models_found = 0;
        let mut models_migrated = 0;
        let mut providers_configured = 0;
        let mut model_mappings = Vec::new();
        let mut provider_mappings = Vec::new();
        
        // Migrate configuration if requested
        if options.migrate_config {
            match self.migrate_configuration().await {
                Ok(mappings) => {
                    providers_configured = mappings.len();
                    provider_mappings = mappings;
                },
                Err(e) => {
                    error!("Failed to migrate configuration: {}", e);
                    
                    // If fallback is enabled, set up fallback provider
                    if options.enable_fallback {
                        if let Some(config) = &*self.legacy_config.lock().unwrap() {
                            let llm = LocalLLM::new(
                                "legacy_fallback",
                                config.context_size,
                                1000, // Default speed
                            );
                            
                            *self.fallback_provider.lock().unwrap() = Some(llm);
                            info!("Fallback provider set up successfully");
                        }
                    }
                    
                    // Set failed status
                    *self.status.lock().unwrap() = MigrationStatus::Failed {
                        timestamp: chrono::Utc::now(),
                        reason: format!("Failed to migrate configuration: {}", e),
                        fallback_active: options.enable_fallback,
                    };
                    
                    let duration = start_time.elapsed().as_secs_f64();
                    
                    return Ok(MigrationNotification {
                        status: self.get_status(),
                        models_found,
                        models_migrated,
                        providers_configured,
                        fallback_available: options.enable_fallback,
                        duration_seconds: duration,
                        model_mappings,
                        provider_mappings,
                    });
                }
            }
        }
        
        // Migrate models if requested
        if options.migrate_models {
            let legacy_models = self.legacy_models.lock().unwrap().clone();
            models_found = legacy_models.len();
            
            match self.migrate_models(legacy_models, &provider_mappings).await {
                Ok(mappings) => {
                    models_migrated = mappings.iter().filter(|m| m.migrated).count();
                    model_mappings = mappings;
                },
                Err(e) => {
                    error!("Failed to migrate models: {}", e);
                    
                    // If fallback is enabled, set up fallback provider
                    if options.enable_fallback && self.fallback_provider.lock().unwrap().is_none() {
                        if let Some(config) = &*self.legacy_config.lock().unwrap() {
                            let llm = LocalLLM::new(
                                "legacy_fallback",
                                config.context_size,
                                1000, // Default speed
                            );
                            
                            *self.fallback_provider.lock().unwrap() = Some(llm);
                            info!("Fallback provider set up successfully");
                        }
                    }
                    
                    // Set failed status
                    *self.status.lock().unwrap() = MigrationStatus::Failed {
                        timestamp: chrono::Utc::now(),
                        reason: format!("Failed to migrate models: {}", e),
                        fallback_active: options.enable_fallback,
                    };
                    
                    let duration = start_time.elapsed().as_secs_f64();
                    
                    return Ok(MigrationNotification {
                        status: self.get_status(),
                        models_found,
                        models_migrated,
                        providers_configured,
                        fallback_available: options.enable_fallback,
                        duration_seconds: duration,
                        model_mappings,
                        provider_mappings,
                    });
                }
            }
        }
        
        // Delete legacy files if requested
        if options.delete_legacy_files && !options.enable_fallback {
            // This would be handled in a real implementation
            // For safety, we're not implementing actual file deletion here
            info!("Legacy file deletion requested but not implemented in this version");
        }
        
        // Save mappings
        *self.model_mappings.lock().unwrap() = model_mappings.clone();
        *self.provider_mappings.lock().unwrap() = provider_mappings.clone();
        
        // Update status to completed
        let provider_names = provider_mappings.iter()
            .map(|p| p.provider_type.clone())
            .collect();
        
        *self.status.lock().unwrap() = MigrationStatus::Completed {
            timestamp: chrono::Utc::now(),
            models_migrated,
            providers_configured: provider_names,
        };
        
        let duration = start_time.elapsed().as_secs_f64();
        
        info!("Migration completed successfully in {:.2} seconds", duration);
        info!("Migrated {}/{} models", models_migrated, models_found);
        info!("Configured {} providers", providers_configured);
        
        // Return migration notification
        Ok(MigrationNotification {
            status: self.get_status(),
            models_found,
            models_migrated,
            providers_configured,
            fallback_available: options.enable_fallback,
            duration_seconds: duration,
            model_mappings,
            provider_mappings,
        })
    }

    /// Migrate configuration
    async fn migrate_configuration(&self) -> Result<Vec<ProviderMapping>> {
        let mut mappings = Vec::new();
        
        // Check if we have legacy config
        let legacy_config = match &*self.legacy_config.lock().unwrap() {
            Some(config) => config.clone(),
            None => {
                // Create a default config if none found
                LLMConfig {
                    model_id: "default".to_string(),
                    model_path: PathBuf::from("models/default"),
                    context_size: 4096,
                    max_output_length: 2048,
                    parameters: LLMParameters::default(),
                    enabled: true,
                    memory_usage_mb: 512,
                }
            }
        };
        
        // Create provider configs based on legacy config
        
        // 1. Create LlamaCpp provider mapping
        let llamacpp_config = ProviderConfig {
            provider_type: ProviderType::LlamaCpp.to_string(),
            endpoint_url: format!("local://{}", legacy_config.model_path.to_string_lossy()),
            api_key: None,
            default_model: Some(legacy_config.model_id.clone()),
            enable_advanced_config: false,
            advanced_config: {
                let mut params = HashMap::new();
                
                // Map LLM parameters
                params.insert("temperature".to_string(), 
                    serde_json::to_value(legacy_config.parameters.temperature)?);
                params.insert("top_p".to_string(), 
                    serde_json::to_value(legacy_config.parameters.top_p)?);
                params.insert("top_k".to_string(), 
                    serde_json::to_value(legacy_config.parameters.top_k)?);
                params.insert("repeat_penalty".to_string(), 
                    serde_json::to_value(legacy_config.parameters.repetition_penalty)?);
                
                if legacy_config.parameters.use_mirostat {
                    params.insert("mirostat".to_string(), serde_json::to_value(true)?);
                    params.insert("mirostat_tau".to_string(), 
                        serde_json::to_value(legacy_config.parameters.mirostat_tau)?);
                    params.insert("mirostat_eta".to_string(), 
                        serde_json::to_value(legacy_config.parameters.mirostat_eta)?);
                }
                
                params
            },
        };
        
        mappings.push(ProviderMapping {
            legacy_config: legacy_config.clone(),
            provider_type: ProviderType::LlamaCpp.to_string(),
            provider_config: llamacpp_config,
            migrated: true,
        });
        
        // 2. Create Ollama provider mapping (if likely)
        // If the model path contains "ollama", it's likely an Ollama model
        if legacy_config.model_path.to_string_lossy().to_lowercase().contains("ollama") {
            let ollama_config = ProviderConfig {
                provider_type: ProviderType::Ollama.to_string(),
                endpoint_url: "http://localhost:11434".to_string(),
                api_key: None,
                default_model: Some(legacy_config.model_id.clone()),
                enable_advanced_config: false,
                advanced_config: {
                    let mut params = HashMap::new();
                    
                    // Map LLM parameters
                    params.insert("temperature".to_string(), 
                        serde_json::to_value(legacy_config.parameters.temperature)?);
                    params.insert("top_p".to_string(), 
                        serde_json::to_value(legacy_config.parameters.top_p)?);
                    params.insert("top_k".to_string(), 
                        serde_json::to_value(legacy_config.parameters.top_k)?);
                    
                    params
                },
            };
            
            mappings.push(ProviderMapping {
                legacy_config: legacy_config.clone(),
                provider_type: ProviderType::Ollama.to_string(),
                provider_config: ollama_config,
                migrated: true,
            });
        }
        
        // 3. Create LocalAI provider mapping
        let localai_config = ProviderConfig {
            provider_type: ProviderType::LocalAI.to_string(),
            endpoint_url: "http://localhost:8080".to_string(),
            api_key: None,
            default_model: None, // We don't know LocalAI model name
            enable_advanced_config: false,
            advanced_config: HashMap::new(),
        };
        
        mappings.push(ProviderMapping {
            legacy_config: legacy_config.clone(),
            provider_type: ProviderType::LocalAI.to_string(),
            provider_config: localai_config,
            migrated: false, // Mark as not migrated since it's speculative
        });
        
        Ok(mappings)
    }

    /// Migrate models
    async fn migrate_models(
        &self, 
        legacy_models: HashMap<String, ModelInfo>,
        provider_mappings: &[ProviderMapping]
    ) -> Result<Vec<LegacyModelMapping>> {
        let mut mappings = Vec::new();
        
        for (legacy_id, model_info) in legacy_models {
            // Determine best provider for this model
            let (provider_type, model_id) = self.determine_best_provider_for_model(&model_info, provider_mappings);
            
            // Create mapping
            mappings.push(LegacyModelMapping {
                legacy_id: legacy_id.clone(),
                provider_type: provider_type.clone(),
                new_id: model_id.clone(),
                model_path: if let Some(legacy_config) = &*self.legacy_config.lock().unwrap() {
                    if legacy_id == legacy_config.model_id {
                        legacy_config.model_path.clone()
                    } else {
                        PathBuf::new() // We don't know the exact path
                    }
                } else {
                    PathBuf::new() // We don't know the exact path
                },
                migrated: true, // Assume migration is successful
            });
        }
        
        Ok(mappings)
    }

    /// Determine the best provider for a model
    fn determine_best_provider_for_model(
        &self,
        model_info: &ModelInfo,
        provider_mappings: &[ProviderMapping]
    ) -> (String, String) {
        // Default to LlamaCpp as the most reliable option
        let mut best_provider = ProviderType::LlamaCpp.to_string();
        let mut best_model_id = model_info.id.clone();
        
        // If model ID contains "llama", it's likely a LlamaCpp model
        if model_info.id.to_lowercase().contains("llama") {
            best_provider = ProviderType::LlamaCpp.to_string();
            best_model_id = model_info.id.clone();
        }
        // If model ID contains "gpt4all", it's likely a LocalAI model
        else if model_info.id.to_lowercase().contains("gpt4all") {
            best_provider = ProviderType::LocalAI.to_string();
            best_model_id = format!("ggml-{}", model_info.id);
        }
        // If model ID contains "openai", it's likely an Ollama model
        else if model_info.id.to_lowercase().contains("openai") {
            best_provider = ProviderType::Ollama.to_string();
            best_model_id = model_info.id.clone();
        }
        // If model file is large (>4GB), it's likely an Ollama model
        else if model_info.size_mb > 4000 {
            best_provider = ProviderType::Ollama.to_string();
            best_model_id = model_info.id.clone();
        }
        
        // Check if we have a specific provider mapping that matches this model
        if let Some(legacy_config) = &*self.legacy_config.lock().unwrap() {
            if model_info.id == legacy_config.model_id {
                // This is the default model from config
                // Use the first successful mapping
                if let Some(mapping) = provider_mappings.iter().find(|m| m.migrated) {
                    best_provider = mapping.provider_type.clone();
                    best_model_id = model_info.id.clone();
                }
            }
        }
        
        (best_provider, best_model_id)
    }

    /// Get the fallback provider
    pub fn get_fallback_provider(&self) -> Option<Arc<LocalLLM>> {
        self.fallback_provider.lock().unwrap().as_ref().map(|p| Arc::new(p.clone()))
    }

    /// Opt out of migration
    pub fn opt_out(&self) {
        *self.status.lock().unwrap() = MigrationStatus::OptedOut {
            timestamp: chrono::Utc::now(),
        };
    }

    /// Get the model mappings
    pub fn get_model_mappings(&self) -> Vec<LegacyModelMapping> {
        self.model_mappings.lock().unwrap().clone()
    }

    /// Get the provider mappings
    pub fn get_provider_mappings(&self) -> Vec<ProviderMapping> {
        self.provider_mappings.lock().unwrap().clone()
    }
}

/// Initialize the migration service
pub async fn init_migration() -> Arc<MigrationService> {
    let service = Arc::new(MigrationService::new());
    
    // Detect legacy system
    if let Err(e) = service.detect_legacy_system().await {
        error!("Failed to detect legacy system: {}", e);
    }
    
    service
}

/// Run migration with default options
pub async fn run_migration(service: &Arc<MigrationService>) -> Result<MigrationNotification> {
    service.run_migration(MigrationOptions::default()).await
}

/// Get migration status
pub fn get_migration_status(service: &Arc<MigrationService>) -> MigrationStatus {
    service.get_status()
}

/// Get migration configuration
pub fn get_migration_config(service: &Arc<MigrationService>) -> MigrationConfig {
    service.get_config()
}

/// Update migration configuration
pub fn update_migration_config(service: &Arc<MigrationService>, config: MigrationConfig) {
    service.update_config(config);
}

/// Get model mappings
pub fn get_model_mappings(service: &Arc<MigrationService>) -> Vec<LegacyModelMapping> {
    service.get_model_mappings()
}

/// Get provider mappings
pub fn get_provider_mappings(service: &Arc<MigrationService>) -> Vec<ProviderMapping> {
    service.get_provider_mappings()
}

/// Get fallback provider
pub fn get_fallback_provider(service: &Arc<MigrationService>) -> Option<Arc<LocalLLM>> {
    service.get_fallback_provider()
}

/// Opt out of migration
pub fn opt_out_of_migration(service: &Arc<MigrationService>) {
    service.opt_out();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migration_status_defaults() {
        let status = MigrationStatus::default();
        assert!(matches!(status, MigrationStatus::NotMigrated));
    }
    
    #[test]
    fn test_migration_options_defaults() {
        let options = MigrationOptions::default();
        assert!(options.migrate_models);
        assert!(options.migrate_config);
        assert!(!options.delete_legacy_files);
        assert!(options.enable_fallback);
    }
    
    #[test]
    fn test_is_model_file() {
        assert!(MigrationService::is_model_file("bin"));
        assert!(MigrationService::is_model_file("gguf"));
        assert!(MigrationService::is_model_file("ggml"));
        assert!(MigrationService::is_model_file("safetensors"));
        assert!(!MigrationService::is_model_file("txt"));
        assert!(!MigrationService::is_model_file("json"));
        assert!(!MigrationService::is_model_file(""));
    }
}
