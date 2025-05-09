use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use tokio::time;
use reqwest;
use anyhow::{Result, anyhow, Context};

use crate::error::Error;
use crate::commands::offline::llm::{ProviderType, ProviderInfo, ProviderConfig};

/// Installation status of a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallationStatus {
    /// Provider is installed and available
    Installed {
        /// Where the provider was found
        location: PathBuf,
        /// Provider version
        version: String,
    },
    /// Provider is not installed
    NotInstalled,
    /// Provider is partially installed or in invalid state
    PartiallyInstalled {
        /// Reason for partial installation
        reason: String,
        /// Path to partial installation if available
        location: Option<PathBuf>,
    },
}

/// Installation information for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    /// Provider type
    pub provider_type: ProviderType,
    /// Installation status
    pub status: InstallationStatus,
    /// Last checked timestamp
    pub last_checked: chrono::DateTime<chrono::Utc>,
    /// Is this provider auto-configured
    pub auto_configured: bool,
}

/// Suggestion for installing a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSuggestion {
    /// Provider type
    pub provider_type: ProviderType,
    /// Installation command or instructions
    pub install_command: String,
    /// URL to installation instructions
    pub instructions_url: String,
    /// Brief description of the provider
    pub description: String,
    /// Recommended for these use cases
    pub recommended_for: Vec<String>,
    /// Hardware requirements
    pub hardware_requirements: Option<HardwareRequirements>,
}

/// Hardware requirements for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRequirements {
    /// Minimum RAM in GB
    pub min_ram_gb: u32,
    /// Recommended RAM in GB
    pub recommended_ram_gb: u32,
    /// Minimum free disk space in GB
    pub min_disk_gb: u32,
    /// GPU required
    pub requires_gpu: bool,
    /// Recommended GPU with minimum VRAM in GB
    pub recommended_gpu: Option<String>,
    /// Minimum VRAM in GB
    pub min_vram_gb: Option<u32>,
}

/// Discovery service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Whether to automatically detect providers
    pub auto_detect: bool,
    /// Whether to set detected providers as active
    pub auto_configure: bool,
    /// How often to scan for providers (in seconds)
    pub scan_interval_seconds: u64,
    /// Paths to scan for providers
    pub scan_paths: Vec<PathBuf>,
    /// Whether to show suggestions for missing providers
    pub show_suggestions: bool,
    /// Whether to check for new versions of providers
    pub check_for_updates: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            auto_configure: true,
            scan_interval_seconds: 3600, // 1 hour
            scan_paths: vec![],
            show_suggestions: true,
            check_for_updates: true,
        }
    }
}

/// LLM Provider Discovery Service
pub struct DiscoveryService {
    /// Configuration for the discovery service
    config: Mutex<DiscoveryConfig>,
    /// Detected providers
    installations: Mutex<HashMap<String, InstallationInfo>>,
    /// Provider suggestions
    suggestions: Mutex<Vec<ProviderSuggestion>>,
    /// Whether a scan is currently running
    scanning: Mutex<bool>,
    /// Last scan timestamp
    last_scan: Mutex<Instant>,
    /// Is the background scanner running
    scanner_running: Mutex<bool>,
}

impl DiscoveryService {
    /// Create a new discovery service with default configuration
    pub fn new() -> Self {
        Self {
            config: Mutex::new(DiscoveryConfig::default()),
            installations: Mutex::new(HashMap::new()),
            suggestions: Mutex::new(Vec::new()),
            scanning: Mutex::new(false),
            last_scan: Mutex::new(Instant::now()),
            scanner_running: Mutex::new(false),
        }
    }

    /// Create a new discovery service with custom configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            config: Mutex::new(config),
            installations: Mutex::new(HashMap::new()),
            suggestions: Mutex::new(Vec::new()),
            scanning: Mutex::new(false),
            last_scan: Mutex::new(Instant::now()),
            scanner_running: Mutex::new(false),
        }
    }

    /// Get the current configuration
    pub fn get_config(&self) -> DiscoveryConfig {
        self.config.lock().unwrap().clone()
    }

    /// Update the configuration
    pub fn update_config(&self, config: DiscoveryConfig) {
        *self.config.lock().unwrap() = config;
    }

    /// Get all detected installations
    pub fn get_installations(&self) -> HashMap<String, InstallationInfo> {
        self.installations.lock().unwrap().clone()
    }

    /// Get a specific installation
    pub fn get_installation(&self, provider_type: &ProviderType) -> Option<InstallationInfo> {
        self.installations.lock().unwrap().get(&provider_type.to_string()).cloned()
    }

    /// Get all suggestions
    pub fn get_suggestions(&self) -> Vec<ProviderSuggestion> {
        self.suggestions.lock().unwrap().clone()
    }

    /// Start the background scanner
    pub async fn start_background_scanner(&self) -> Result<()> {
        let mut scanner_running = self.scanner_running.lock().unwrap();
        if *scanner_running {
            return Ok(());
        }
        
        *scanner_running = true;
        
        // Clone Arc for the task
        let service = Arc::new(self.clone());
        
        tokio::spawn(async move {
            loop {
                // Check if scanner should still be running
                {
                    let running = service.scanner_running.lock().unwrap();
                    if !*running {
                        break;
                    }
                }
                
                // Get scan interval from config
                let interval = {
                    let config = service.config.lock().unwrap();
                    Duration::from_secs(config.scan_interval_seconds)
                };
                
                // Scan for providers
                if let Err(e) = service.scan_for_providers().await {
                    error!("Error scanning for providers: {}", e);
                }
                
                // Update suggestions
                if let Err(e) = service.update_suggestions().await {
                    error!("Error updating suggestions: {}", e);
                }
                
                // Sleep until next scan
                time::sleep(interval).await;
            }
        });
        
        Ok(())
    }

    /// Stop the background scanner
    pub fn stop_background_scanner(&self) {
        let mut scanner_running = self.scanner_running.lock().unwrap();
        *scanner_running = false;
    }

    /// Scan for installed providers
    pub async fn scan_for_providers(&self) -> Result<()> {
        // Check if a scan is already running
        {
            let mut scanning = self.scanning.lock().unwrap();
            if *scanning {
                return Ok(());
            }
            *scanning = true;
        }
        
        info!("Scanning for LLM providers...");
        
        // Get a copy of the current config
        let config = self.get_config();
        
        // Skip if auto-detect is disabled
        if !config.auto_detect {
            let mut scanning = self.scanning.lock().unwrap();
            *scanning = false;
            return Ok(());
        }
        
        // Create a new installations map
        let mut new_installations = HashMap::new();
        
        // Scan for Ollama
        if let Ok(ollama_info) = self.detect_ollama().await {
            new_installations.insert(ProviderType::Ollama.to_string(), ollama_info);
        }
        
        // Scan for LocalAI
        if let Ok(localai_info) = self.detect_localai().await {
            new_installations.insert(ProviderType::LocalAI.to_string(), localai_info);
        }
        
        // Scan for LlamaCpp
        if let Ok(llamacpp_info) = self.detect_llamacpp().await {
            new_installations.insert(ProviderType::LlamaCpp.to_string(), llamacpp_info);
        }
        
        // Scan custom paths
        for path in &config.scan_paths {
            if let Ok(custom_providers) = self.scan_custom_path(path).await {
                for (provider_type, info) in custom_providers {
                    new_installations.insert(provider_type, info);
                }
            }
        }
        
        // Update installations
        *self.installations.lock().unwrap() = new_installations;
        
        // Update last scan time
        *self.last_scan.lock().unwrap() = Instant::now();
        
        // Reset scanning flag
        {
            let mut scanning = self.scanning.lock().unwrap();
            *scanning = false;
        }
        
        info!("Provider scan complete");
        Ok(())
    }

    /// Create provider configurations for installed providers
    pub fn create_provider_configs(&self) -> Vec<ProviderConfig> {
        let installations = self.get_installations();
        let mut configs = Vec::new();
        
        for (provider_type_str, info) in installations {
            if let InstallationStatus::Installed { location, version } = info.status {
                let provider_type = match ProviderType::from_string(&provider_type_str) {
                    Ok(pt) => pt,
                    Err(_) => continue,
                };
                
                let config = match provider_type {
                    ProviderType::Ollama => ProviderConfig {
                        provider_type: provider_type_str.clone(),
                        endpoint_url: "http://localhost:11434".to_string(),
                        api_key: None,
                        default_model: Some("llama2".to_string()),
                        enable_advanced_config: false,
                        advanced_config: HashMap::new(),
                    },
                    ProviderType::LocalAI => ProviderConfig {
                        provider_type: provider_type_str.clone(),
                        endpoint_url: "http://localhost:8080".to_string(),
                        api_key: None,
                        default_model: Some("ggml-gpt4all-j".to_string()),
                        enable_advanced_config: false,
                        advanced_config: HashMap::new(),
                    },
                    ProviderType::LlamaCpp => ProviderConfig {
                        provider_type: provider_type_str.clone(),
                        endpoint_url: format!("local://{}", location.to_string_lossy()),
                        api_key: None,
                        default_model: None,
                        enable_advanced_config: false,
                        advanced_config: HashMap::new(),
                    },
                    ProviderType::Custom(name) => ProviderConfig {
                        provider_type: provider_type_str.clone(),
                        endpoint_url: format!("http://localhost:8000/{}", name.to_lowercase()),
                        api_key: None,
                        default_model: None,
                        enable_advanced_config: false,
                        advanced_config: HashMap::new(),
                    },
                };
                
                configs.push(config);
            }
        }
        
        configs
    }

    /// Create provider information for installed providers
    pub fn create_provider_infos(&self) -> Vec<ProviderInfo> {
        let installations = self.get_installations();
        let mut infos = Vec::new();
        
        for (provider_type_str, info) in installations {
            if let InstallationStatus::Installed { location, version } = info.status {
                let provider_type = match ProviderType::from_string(&provider_type_str) {
                    Ok(pt) => pt,
                    Err(_) => continue,
                };
                
                let info = match provider_type {
                    ProviderType::Ollama => ProviderInfo {
                        provider_type: provider_type_str.clone(),
                        name: "Ollama".to_string(),
                        description: "Local model runner for LLama and other models".to_string(),
                        version,
                        default_endpoint: "http://localhost:11434".to_string(),
                        supports_text_generation: true,
                        supports_chat: true,
                        supports_embeddings: true,
                        requires_api_key: false,
                    },
                    ProviderType::LocalAI => ProviderInfo {
                        provider_type: provider_type_str.clone(),
                        name: "LocalAI".to_string(),
                        description: "Self-hosted OpenAI API compatible server".to_string(),
                        version,
                        default_endpoint: "http://localhost:8080".to_string(),
                        supports_text_generation: true,
                        supports_chat: true,
                        supports_embeddings: true,
                        requires_api_key: false,
                    },
                    ProviderType::LlamaCpp => ProviderInfo {
                        provider_type: provider_type_str.clone(),
                        name: "llama.cpp".to_string(),
                        description: "Embedded llama.cpp integration for efficient local inference".to_string(),
                        version,
                        default_endpoint: format!("local://{}", location.to_string_lossy()),
                        supports_text_generation: true,
                        supports_chat: true,
                        supports_embeddings: false,
                        requires_api_key: false,
                    },
                    ProviderType::Custom(name) => ProviderInfo {
                        provider_type: provider_type_str.clone(),
                        name: name.clone(),
                        description: format!("Custom provider: {}", name),
                        version,
                        default_endpoint: format!("http://localhost:8000/{}", name.to_lowercase()),
                        supports_text_generation: true,
                        supports_chat: true,
                        supports_embeddings: false,
                        requires_api_key: false,
                    },
                };
                
                infos.push(info);
            }
        }
        
        infos
    }

    /// Update suggestions for providers
    pub async fn update_suggestions(&self) -> Result<()> {
        let installations = self.get_installations();
        let config = self.get_config();
        
        // Skip if suggestions are disabled
        if !config.show_suggestions {
            return Ok(());
        }
        
        let mut suggestions = Vec::new();
        
        // Add suggestion for Ollama if not installed
        if !installations.contains_key(&ProviderType::Ollama.to_string()) {
            suggestions.push(ProviderSuggestion {
                provider_type: ProviderType::Ollama,
                install_command: if cfg!(target_os = "windows") {
                    "curl https://ollama.ai/download/ollama-windows-amd64.zip -o ollama.zip".to_string()
                } else if cfg!(target_os = "macos") {
                    "curl https://ollama.ai/download/ollama-darwin-amd64 -o ollama".to_string()
                } else {
                    "curl https://ollama.ai/install.sh | sh".to_string()
                },
                instructions_url: "https://ollama.ai/download".to_string(),
                description: "Run LLMs on your local machine. Ollama is a tool that allows you to run large language models locally.".to_string(),
                recommended_for: vec![
                    "Easy setup".to_string(),
                    "Familiar OpenAI-like API".to_string(),
                    "Wide model support".to_string(),
                ],
                hardware_requirements: Some(HardwareRequirements {
                    min_ram_gb: 8,
                    recommended_ram_gb: 16,
                    min_disk_gb: 10,
                    requires_gpu: false,
                    recommended_gpu: Some("NVIDIA with CUDA support".to_string()),
                    min_vram_gb: Some(4),
                }),
            });
        }
        
        // Add suggestion for LocalAI if not installed
        if !installations.contains_key(&ProviderType::LocalAI.to_string()) {
            suggestions.push(ProviderSuggestion {
                provider_type: ProviderType::LocalAI,
                install_command: if cfg!(target_os = "windows") {
                    "docker run -p 8080:8080 localai/localai:latest".to_string()
                } else if cfg!(target_os = "macos") {
                    "docker run -p 8080:8080 localai/localai:latest".to_string()
                } else {
                    "docker run -p 8080:8080 localai/localai:latest".to_string()
                },
                instructions_url: "https://localai.io/basics/getting_started/".to_string(),
                description: "Self-hosted, OpenAI API compatible server that can run with CPU, GPUs, M1/2, or even TPUs.".to_string(),
                recommended_for: vec![
                    "OpenAI API compatibility".to_string(),
                    "Multi-modal models".to_string(),
                    "Advanced customization".to_string(),
                ],
                hardware_requirements: Some(HardwareRequirements {
                    min_ram_gb: 8,
                    recommended_ram_gb: 16,
                    min_disk_gb: 10,
                    requires_gpu: false,
                    recommended_gpu: Some("NVIDIA with CUDA support".to_string()),
                    min_vram_gb: Some(4),
                }),
            });
        }
        
        // Update suggestions
        *self.suggestions.lock().unwrap() = suggestions;
        
        Ok(())
    }

    /// Detect Ollama installation
    async fn detect_ollama(&self) -> Result<InstallationInfo> {
        debug!("Detecting Ollama installation...");
        
        // Platform-specific detection
        let executable_name = if cfg!(target_os = "windows") {
            "ollama.exe"
        } else {
            "ollama"
        };
        
        // Try to find Ollama in PATH
        let output = match which::which(executable_name) {
            Ok(path) => {
                // Found ollama executable
                debug!("Found Ollama executable at {:?}", path);
                
                // Check if Ollama server is running
                match self.check_ollama_server().await {
                    Ok(version) => {
                        // Ollama server is running
                        InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::Installed {
                                location: path.clone(),
                                version,
                            },
                            last_checked: chrono::Utc::now(),
                            auto_configured: true,
                        }
                    },
                    Err(_) => {
                        // Ollama server is not running
                        InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::PartiallyInstalled {
                                reason: "Ollama executable found but server is not running".to_string(),
                                location: Some(path),
                            },
                            last_checked: chrono::Utc::now(),
                            auto_configured: false,
                        }
                    }
                }
            },
            Err(_) => {
                // Try additional platform-specific locations
                if cfg!(target_os = "windows") {
                    let program_files = std::env::var("ProgramFiles").unwrap_or("C:\\Program Files".to_string());
                    let ollama_path = Path::new(&program_files).join("Ollama").join(executable_name);
                    
                    if ollama_path.exists() {
                        match self.check_ollama_server().await {
                            Ok(version) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::Installed {
                                        location: ollama_path.clone(),
                                        version,
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: true,
                                }
                            },
                            Err(_) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::PartiallyInstalled {
                                        reason: "Ollama executable found but server is not running".to_string(),
                                        location: Some(ollama_path),
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: false,
                                }
                            }
                        }
                    } else {
                        // Not found
                        InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::NotInstalled,
                            last_checked: chrono::Utc::now(),
                            auto_configured: false,
                        }
                    }
                } else if cfg!(target_os = "macos") {
                    let ollama_path = Path::new("/Applications/Ollama.app/Contents/MacOS").join(executable_name);
                    
                    if ollama_path.exists() {
                        match self.check_ollama_server().await {
                            Ok(version) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::Installed {
                                        location: ollama_path.clone(),
                                        version,
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: true,
                                }
                            },
                            Err(_) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::PartiallyInstalled {
                                        reason: "Ollama executable found but server is not running".to_string(),
                                        location: Some(ollama_path),
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: false,
                                }
                            }
                        }
                    } else {
                        // Not found
                        InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::NotInstalled,
                            last_checked: chrono::Utc::now(),
                            auto_configured: false,
                        }
                    }
                } else {
                    // Linux - check /usr/local/bin and /usr/bin
                    let usr_local_bin = Path::new("/usr/local/bin").join(executable_name);
                    let usr_bin = Path::new("/usr/bin").join(executable_name);
                    
                    if usr_local_bin.exists() {
                        match self.check_ollama_server().await {
                            Ok(version) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::Installed {
                                        location: usr_local_bin.clone(),
                                        version,
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: true,
                                }
                            },
                            Err(_) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::PartiallyInstalled {
                                        reason: "Ollama executable found but server is not running".to_string(),
                                        location: Some(usr_local_bin),
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: false,
                                }
                            }
                        }
                    } else if usr_bin.exists() {
                        match self.check_ollama_server().await {
                            Ok(version) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::Installed {
                                        location: usr_bin.clone(),
                                        version,
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: true,
                                }
                            },
                            Err(_) => {
                                InstallationInfo {
                                    provider_type: ProviderType::Ollama,
                                    status: InstallationStatus::PartiallyInstalled {
                                        reason: "Ollama executable found but server is not running".to_string(),
                                        location: Some(usr_bin),
                                    },
                                    last_checked: chrono::Utc::now(),
                                    auto_configured: false,
                                }
                            }
                        }
                    } else {
                        // Not found
                        InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::NotInstalled,
                            last_checked: chrono::Utc::now(),
                            auto_configured: false,
                        }
                    }
                }
            }
        };
        
        Ok(output)
    }

    /// Check if Ollama server is running
    async fn check_ollama_server(&self) -> Result<String> {
        debug!("Checking Ollama server...");
        
        // Try to connect to Ollama server
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;
        
        let response = client
            .get("http://localhost:11434/api/version")
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let version = json["version"].as_str().unwrap_or("unknown").to_string();
            Ok(version)
        } else {
            Err(anyhow!("Ollama server returned error: {}", response.status()))
        }
    }

    /// Detect LocalAI installation
    async fn detect_localai(&self) -> Result<InstallationInfo> {
        debug!("Detecting LocalAI installation...");
        
        // LocalAI is often run as a Docker container
        // Try to check if LocalAI is running by connecting to the API
        match self.check_localai_server().await {
            Ok(version) => {
                // LocalAI server is running
                Ok(InstallationInfo {
                    provider_type: ProviderType::LocalAI,
                    status: InstallationStatus::Installed {
                        location: PathBuf::from("/var/lib/docker"), // Approximate, not precise
                        version,
                    },
                    last_checked: chrono::Utc::now(),
                    auto_configured: true,
                })
            },
            Err(_) => {
                // Check if Docker is available and try to find LocalAI image
                if which::which("docker").is_ok() {
                    let output = Command::new("docker")
                        .args(["images", "localai/localai", "--format", "{{.Tag}}"])
                        .output();
                    
                    match output {
                        Ok(output) if !output.stdout.is_empty() => {
                            // LocalAI image found but not running
                            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            Ok(InstallationInfo {
                                provider_type: ProviderType::LocalAI,
                                status: InstallationStatus::PartiallyInstalled {
                                    reason: "LocalAI Docker image found but server is not running".to_string(),
                                    location: Some(PathBuf::from("/var/lib/docker")),
                                },
                                last_checked: chrono::Utc::now(),
                                auto_configured: false,
                            })
                        },
                        _ => {
                            // LocalAI not found
                            Ok(InstallationInfo {
                                provider_type: ProviderType::LocalAI,
                                status: InstallationStatus::NotInstalled,
                                last_checked: chrono::Utc::now(),
                                auto_configured: false,
                            })
                        }
                    }
                } else {
                    // Docker not available
                    Ok(InstallationInfo {
                        provider_type: ProviderType::LocalAI,
                        status: InstallationStatus::NotInstalled,
                        last_checked: chrono::Utc::now(),
                        auto_configured: false,
                    })
                }
            }
        }
    }

    /// Check if LocalAI server is running
    async fn check_localai_server(&self) -> Result<String> {
        debug!("Checking LocalAI server...");
        
        // Try to connect to LocalAI server
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;
        
        let response = client
            .get("http://localhost:8080/version")
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let version = json["version"].as_str().unwrap_or("unknown").to_string();
            Ok(version)
        } else {
            Err(anyhow!("LocalAI server returned error: {}", response.status()))
        }
    }

    /// Detect llama.cpp installation
    async fn detect_llamacpp(&self) -> Result<InstallationInfo> {
        debug!("Detecting llama.cpp installation...");
        
        // Look for llama.cpp in common locations
        let mut possible_locations = Vec::new();
        
        // Check PATH for main executable
        let executable_name = if cfg!(target_os = "windows") {
            "llama-main.exe"
        } else {
            "llama-main"
        };
        
        if let Ok(path) = which::which(executable_name) {
            possible_locations.push(path);
        }
        
        // Check other common names
        let alt_executable_names = if cfg!(target_os = "windows") {
            vec!["llama.exe", "llama-cli.exe", "llm.exe"]
        } else {
            vec!["llama", "llama-cli", "llm"]
        };
        
        for name in alt_executable_names {
            if let Ok(path) = which::which(name) {
                possible_locations.push(path);
            }
        }
        
        // Check common installation directories
        if cfg!(target_os = "windows") {
            let program_files = std::env::var("ProgramFiles").unwrap_or("C:\\Program Files".to_string());
            possible_locations.push(Path::new(&program_files).join("llama.cpp").join(executable_name));
            possible_locations.push(Path::new(&program_files).join("llama.cpp").join("bin").join(executable_name));
            
            // Check home directory
            if let Ok(home) = std::env::var("USERPROFILE") {
                possible_locations.push(Path::new(&home).join("llama.cpp").join(executable_name));
                possible_locations.push(Path::new(&home).join("Downloads").join("llama.cpp").join(executable_name));
                possible_locations.push(Path::new(&home).join("git").join("llama.cpp").join(executable_name));
            }
        } else if cfg!(target_os = "macos") {
            possible_locations.push(Path::new("/Applications/llama.cpp/bin").join(executable_name));
            possible_locations.push(Path::new("/usr/local/bin").join(executable_name));
            
            // Check home directory
            if let Ok(home) = std::env::var("HOME") {
                possible_locations.push(Path::new(&home).join("llama.cpp").join(executable_name));
                possible_locations.push(Path::new(&home).join("Downloads").join("llama.cpp").join(executable_name));
                possible_locations.push(Path::new(&home).join("git").join("llama.cpp").join(executable_name));
            }
        } else {
            // Linux
            possible_locations.push(Path::new("/usr/local/bin").join(executable_name));
            possible_locations.push(Path::new("/usr/bin").join(executable_name));
            possible_locations.push(Path::new("/opt/llama.cpp/bin").join(executable_name));
            
            // Check home directory
            if let Ok(home) = std::env::var("HOME") {
                possible_locations.push(Path::new(&home).join("llama.cpp").join(executable_name));
                possible_locations.push(Path::new(&home).join("Downloads").join("llama.cpp").join(executable_name));
                possible_locations.push(Path::new(&home).join("git").join("llama.cpp").join(executable_name));
            }
        }
        
        // Check if any location exists
        for location in possible_locations {
            if location.exists() {
                // Try to get version
                let version = self.get_llamacpp_version(&location).await?;
                
                return Ok(InstallationInfo {
                    provider_type: ProviderType::LlamaCpp,
                    status: InstallationStatus::Installed {
                        location: location.clone(),
                        version,
                    },
                    last_checked: chrono::Utc::now(),
                    auto_configured: true,
                });
            }
        }
        
        // Not found
        Ok(InstallationInfo {
            provider_type: ProviderType::LlamaCpp,
            status: InstallationStatus::NotInstalled,
            last_checked: chrono::Utc::now(),
            auto_configured: false,
        })
    }

    /// Get llama.cpp version
    async fn get_llamacpp_version(&self, path: &Path) -> Result<String> {
        debug!("Getting llama.cpp version from {:?}...", path);
        
        // Try to run the executable with --version
        let output = Command::new(path)
            .args(["--version"])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                // Parse version from output
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Look for version in stdout or stderr
                let combined = format!("{}{}", stdout, stderr);
                let version_regex = regex::Regex::new(r"version\s+([0-9]+\.[0-9]+\.[0-9]+)").unwrap();
                
                if let Some(captures) = version_regex.captures(&combined) {
                    Ok(captures[1].to_string())
                } else {
                    // Fallback - just return "unknown"
                    Ok("unknown".to_string())
                }
            },
            _ => {
                // Try with -v flag
                let output = Command::new(path)
                    .args(["-v"])
                    .output();
                
                match output {
                    Ok(output) if output.status.success() => {
                        // Parse version from output
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        // Look for version in stdout or stderr
                        let combined = format!("{}{}", stdout, stderr);
                        let version_regex = regex::Regex::new(r"version\s+([0-9]+\.[0-9]+\.[0-9]+)").unwrap();
                        
                        if let Some(captures) = version_regex.captures(&combined) {
                            Ok(captures[1].to_string())
                        } else {
                            // Fallback - just return "unknown"
                            Ok("unknown".to_string())
                        }
                    },
                    _ => {
                        // Couldn't get version
                        Ok("unknown".to_string())
                    }
                }
            }
        }
    }

    /// Scan a custom path for possible providers
    async fn scan_custom_path(&self, path: &Path) -> Result<HashMap<String, InstallationInfo>> {
        debug!("Scanning custom path {:?} for providers...", path);
        
        let mut results = HashMap::new();
        
        // Skip if path doesn't exist
        if !path.exists() {
            return Ok(results);
        }
        
        // Check if path is a directory
        if !path.is_dir() {
            return Ok(results);
        }
        
        // List entries in directory
        let entries = std::fs::read_dir(path)
            .with_context(|| format!("Failed to read directory {:?}", path))?;
        
        // Look for executables and directories that might be providers
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Check if directory name matches a known provider pattern
                let dir_name = entry_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                
                if dir_name.to_lowercase().contains("llm") || 
                   dir_name.to_lowercase().contains("model") || 
                   dir_name.to_lowercase().contains("ai") {
                    // This might be a custom provider
                    // Look for executable files
                    if let Ok(subentries) = std::fs::read_dir(&entry_path) {
                        for subentry in subentries {
                            if let Ok(subentry) = subentry {
                                let subentry_path = subentry.path();
                                
                                if subentry_path.is_file() && self.is_executable(&subentry_path) {
                                    // This might be a provider executable
                                    let provider_name = dir_name.to_string();
                                    let provider_type = ProviderType::Custom(provider_name.clone());
                                    
                                    results.insert(
                                        provider_type.to_string(),
                                        InstallationInfo {
                                            provider_type: provider_type.clone(),
                                            status: InstallationStatus::Installed {
                                                location: entry_path.clone(),
                                                version: "custom".to_string(),
                                            },
                                            last_checked: chrono::Utc::now(),
                                            auto_configured: true,
                                        },
                                    );
                                    
                                    // Only add one provider per directory
                                    break;
                                }
                            }
                        }
                    }
                }
            } else if entry_path.is_file() && self.is_executable(&entry_path) {
                // This might be a provider executable
                let file_name = entry_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                
                if file_name.to_lowercase().contains("llm") || 
                   file_name.to_lowercase().contains("model") || 
                   file_name.to_lowercase().contains("ai") {
                    // Strip extension to get provider name
                    let provider_name = entry_path.file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or(file_name)
                        .to_string();
                    
                    let provider_type = ProviderType::Custom(provider_name.clone());
                    
                    results.insert(
                        provider_type.to_string(),
                        InstallationInfo {
                            provider_type: provider_type.clone(),
                            status: InstallationStatus::Installed {
                                location: entry_path.clone(),
                                version: "custom".to_string(),
                            },
                            last_checked: chrono::Utc::now(),
                            auto_configured: true,
                        },
                    );
                }
            }
        }
        
        Ok(results)
    }

    /// Check if a file is executable
    fn is_executable(&self, path: &Path) -> bool {
        if cfg!(unix) {
            // On Unix, check file permissions
            use std::os::unix::fs::PermissionsExt;
            
            if let Ok(metadata) = std::fs::metadata(path) {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o111 != 0;
            }
            
            false
        } else {
            // On Windows, check file extension
            if let Some(extension) = path.extension() {
                let extension = extension.to_string_lossy().to_lowercase();
                return extension == "exe" || extension == "bat" || extension == "cmd";
            }
            
            false
        }
    }
}

impl Clone for DiscoveryService {
    fn clone(&self) -> Self {
        Self {
            config: Mutex::new(self.config.lock().unwrap().clone()),
            installations: Mutex::new(self.installations.lock().unwrap().clone()),
            suggestions: Mutex::new(self.suggestions.lock().unwrap().clone()),
            scanning: Mutex::new(*self.scanning.lock().unwrap()),
            last_scan: Mutex::new(*self.last_scan.lock().unwrap()),
            scanner_running: Mutex::new(*self.scanner_running.lock().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_discovery_service_creation() {
        let service = DiscoveryService::new();
        
        // Check default config
        let config = service.get_config();
        assert!(config.auto_detect);
        assert!(config.auto_configure);
        assert_eq!(config.scan_interval_seconds, 3600);
        
        // Initially no installations
        let installations = service.get_installations();
        assert!(installations.is_empty());
        
        // Initially no suggestions
        let suggestions = service.get_suggestions();
        assert!(suggestions.is_empty());
    }
    
    #[tokio::test]
    async fn test_custom_config() {
        let config = DiscoveryConfig {
            auto_detect: false,
            auto_configure: false,
            scan_interval_seconds: 120,
            scan_paths: vec![PathBuf::from("/tmp")],
            show_suggestions: false,
            check_for_updates: false,
        };
        
        let service = DiscoveryService::with_config(config.clone());
        
        // Check config values
        let retrieved_config = service.get_config();
        assert_eq!(retrieved_config.auto_detect, config.auto_detect);
        assert_eq!(retrieved_config.auto_configure, config.auto_configure);
        assert_eq!(retrieved_config.scan_interval_seconds, config.scan_interval_seconds);
        assert_eq!(retrieved_config.scan_paths, config.scan_paths);
        assert_eq!(retrieved_config.show_suggestions, config.show_suggestions);
        assert_eq!(retrieved_config.check_for_updates, config.check_for_updates);
    }
    
    #[tokio::test]
    async fn test_update_config() {
        let service = DiscoveryService::new();
        
        // Update config
        let new_config = DiscoveryConfig {
            auto_detect: false,
            auto_configure: false,
            scan_interval_seconds: 120,
            scan_paths: vec![PathBuf::from("/tmp")],
            show_suggestions: false,
            check_for_updates: false,
        };
        
        service.update_config(new_config.clone());
        
        // Check config values
        let retrieved_config = service.get_config();
        assert_eq!(retrieved_config.auto_detect, new_config.auto_detect);
        assert_eq!(retrieved_config.auto_configure, new_config.auto_configure);
        assert_eq!(retrieved_config.scan_interval_seconds, new_config.scan_interval_seconds);
        assert_eq!(retrieved_config.scan_paths, new_config.scan_paths);
        assert_eq!(retrieved_config.show_suggestions, new_config.show_suggestions);
        assert_eq!(retrieved_config.check_for_updates, new_config.check_for_updates);
    }
    
    // Note: More comprehensive tests would be added in a real implementation
    // to test the actual detection functionality, but those would require
    // mocking the file system and network requests.
}

// Public API functions

/// Initialize the discovery service
pub async fn init_discovery() -> Arc<DiscoveryService> {
    let service = Arc::new(DiscoveryService::new());
    
    // Perform initial scan
    if let Err(e) = service.scan_for_providers().await {
        error!("Error during initial provider scan: {}", e);
    }
    
    // Update suggestions
    if let Err(e) = service.update_suggestions().await {
        error!("Error updating provider suggestions: {}", e);
    }
    
    // Start background scanner
    if let Err(e) = service.start_background_scanner().await {
        error!("Error starting background scanner: {}", e);
    }
    
    service
}

/// Scan for providers
pub async fn scan_for_providers(service: &Arc<DiscoveryService>) -> Result<()> {
    service.scan_for_providers().await
}

/// Get provider configurations
pub fn get_provider_configs(service: &Arc<DiscoveryService>) -> Vec<ProviderConfig> {
    service.create_provider_configs()
}

/// Get provider information
pub fn get_provider_infos(service: &Arc<DiscoveryService>) -> Vec<ProviderInfo> {
    service.create_provider_infos()
}

/// Get provider suggestions
pub fn get_provider_suggestions(service: &Arc<DiscoveryService>) -> Vec<ProviderSuggestion> {
    service.get_suggestions()
}

/// Get provider installations
pub fn get_provider_installations(service: &Arc<DiscoveryService>) -> HashMap<String, InstallationInfo> {
    service.get_installations()
}

/// Get discovery service configuration
pub fn get_discovery_config(service: &Arc<DiscoveryService>) -> DiscoveryConfig {
    service.get_config()
}

/// Update discovery service configuration
pub fn update_discovery_config(service: &Arc<DiscoveryService>, config: DiscoveryConfig) {
    service.update_config(config);
}
