// src/offline/llm/discovery.rs
//! Discovery service for local LLM providers

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use tokio::time;
use reqwest;
use tokio::sync::Mutex as TokioMutex;

use crate::platform::fs::platform_fs;
use super::provider::{Provider, ProviderType, ProviderError, Result};
use super::factory::{get_provider_factory, ProviderConfig};
use super::providers::ollama::OllamaConfig;
use super::providers::localai::LocalAIConfig;

/// Installation status of a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub last_checked: String,
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
    /// Whether the provider is detected
    pub detected: bool,
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
            scan_paths: Vec::new(),
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
    installations: TokioMutex<HashMap<String, InstallationInfo>>,
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
        let service = Self {
            config: Mutex::new(DiscoveryConfig::default()),
            installations: TokioMutex::new(HashMap::new()),
            suggestions: Mutex::new(Vec::new()),
            scanning: Mutex::new(false),
            last_scan: Mutex::new(Instant::now()),
            scanner_running: Mutex::new(false),
        };
        
        // Initialize suggestions
        service.init_suggestions();
        
        service
    }

    /// Create a new discovery service with custom configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        let service = Self {
            config: Mutex::new(config),
            installations: TokioMutex::new(HashMap::new()),
            suggestions: Mutex::new(Vec::new()),
            scanning: Mutex::new(false),
            last_scan: Mutex::new(Instant::now()),
            scanner_running: Mutex::new(false),
        };
        
        // Initialize suggestions
        service.init_suggestions();
        
        service
    }
    
    /// Initialize default suggestions
    fn init_suggestions(&self) {
        let mut suggestions = Vec::new();
        
        // Add suggestion for Ollama
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
            detected: false,
        });
        
        // Add suggestion for LocalAI
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
            detected: false,
        });
        
        // Add suggestion for LlamaCpp
        suggestions.push(ProviderSuggestion {
            provider_type: ProviderType::LlamaCpp,
            install_command: if cfg!(target_os = "windows") {
                "git clone https://github.com/ggerganov/llama.cpp.git && cd llama.cpp && mkdir build && cd build && cmake .. && cmake --build . --config Release".to_string()
            } else if cfg!(target_os = "macos") {
                "git clone https://github.com/ggerganov/llama.cpp.git && cd llama.cpp && make".to_string()
            } else {
                "git clone https://github.com/ggerganov/llama.cpp.git && cd llama.cpp && make".to_string()
            },
            instructions_url: "https://github.com/ggerganov/llama.cpp".to_string(),
            description: "Port of Facebook's LLaMA model in C/C++ for efficient CPU and GPU inference.".to_string(),
            recommended_for: vec![
                "High performance".to_string(),
                "Low resource usage".to_string(),
                "Fine-grained control".to_string(),
            ],
            hardware_requirements: Some(HardwareRequirements {
                min_ram_gb: 4,
                recommended_ram_gb: 8,
                min_disk_gb: 5,
                requires_gpu: false,
                recommended_gpu: Some("NVIDIA with CUDA support".to_string()),
                min_vram_gb: Some(4),
            }),
            detected: false,
        });
        
        // Set suggestions
        *self.suggestions.lock().unwrap() = suggestions;
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
    pub async fn get_installations(&self) -> HashMap<String, InstallationInfo> {
        self.installations.lock().await.clone()
    }

    /// Get a specific installation
    pub async fn get_installation(&self, provider_type: &ProviderType) -> Option<InstallationInfo> {
        self.installations.lock().await.get(&provider_type.to_string()).cloned()
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
        
        // Create clones for the task
        let config = self.config.clone();
        let installations = self.installations.clone();
        let suggestions = self.suggestions.clone();
        let scanning = self.scanning.clone();
        let last_scan = self.last_scan.clone();
        let scanner_running = self.scanner_running.clone();
        
        tokio::spawn(async move {
            loop {
                // Check if scanner should still be running
                {
                    let running = scanner_running.lock().unwrap();
                    if !*running {
                        break;
                    }
                }
                
                // Get scan interval from config
                let interval = {
                    let config = config.lock().unwrap();
                    Duration::from_secs(config.scan_interval_seconds)
                };
                
                // Scan for providers
                Self::scan_providers_internal(
                    &config,
                    &installations,
                    &suggestions,
                    &scanning,
                    &last_scan
                ).await;
                
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
        
        Self::scan_providers_internal(
            &self.config,
            &self.installations,
            &self.suggestions,
            &self.scanning,
            &self.last_scan
        ).await;
        
        Ok(())
    }
    
    /// Internal implementation of provider scanning
    async fn scan_providers_internal(
        config: &Mutex<DiscoveryConfig>,
        installations: &TokioMutex<HashMap<String, InstallationInfo>>,
        suggestions: &Mutex<Vec<ProviderSuggestion>>,
        scanning: &Mutex<bool>,
        last_scan: &Mutex<Instant>,
    ) {
        info!("Scanning for LLM providers...");
        
        // Get a copy of the current config
        let config_copy = {
            let config = config.lock().unwrap();
            config.clone()
        };
        
        // Skip if auto-detect is disabled
        if !config_copy.auto_detect {
            let mut scanning_lock = scanning.lock().unwrap();
            *scanning_lock = false;
            return;
        }
        
        // Create a new installations map
        let mut new_installations = HashMap::new();
        
        // Scan for Ollama
        match Self::detect_ollama().await {
            Ok(info) => {
                new_installations.insert(ProviderType::Ollama.to_string(), info);
            },
            Err(e) => {
                warn!("Error detecting Ollama: {}", e);
            }
        }
        
        // Scan for LocalAI
        match Self::detect_localai().await {
            Ok(info) => {
                new_installations.insert(ProviderType::LocalAI.to_string(), info);
            },
            Err(e) => {
                warn!("Error detecting LocalAI: {}", e);
            }
        }
        
        // Scan for LlamaCpp
        match Self::detect_llamacpp().await {
            Ok(info) => {
                new_installations.insert(ProviderType::LlamaCpp.to_string(), info);
            },
            Err(e) => {
                warn!("Error detecting LlamaCpp: {}", e);
            }
        }
        
        // Scan custom paths
        for path in &config_copy.scan_paths {
            match Self::scan_custom_path(path).await {
                Ok(custom_providers) => {
                    for (provider_type, info) in custom_providers {
                        new_installations.insert(provider_type, info);
                    }
                },
                Err(e) => {
                    warn!("Error scanning custom path {}: {}", path.display(), e);
                }
            }
        }
        
        // Auto-configure providers if enabled
        if config_copy.auto_configure {
            Self::auto_configure_providers(&new_installations).await;
        }
        
        // Update installations
        {
            let mut installations_lock = installations.lock().await;
            *installations_lock = new_installations.clone();
        }
        
        // Update suggestions detection status
        {
            let mut suggestions_lock = suggestions.lock().unwrap();
            for suggestion in suggestions_lock.iter_mut() {
                let provider_str = suggestion.provider_type.to_string();
                suggestion.detected = new_installations.contains_key(&provider_str) &&
                    matches!(new_installations[&provider_str].status, InstallationStatus::Installed { .. });
            }
        }
        
        // Update last scan time
        {
            let mut last_scan_lock = last_scan.lock().unwrap();
            *last_scan_lock = Instant::now();
        }
        
        // Reset scanning flag
        {
            let mut scanning_lock = scanning.lock().unwrap();
            *scanning_lock = false;
        }
        
        info!("Provider scan complete");
    }

    /// Detect Ollama provider
    async fn detect_ollama() -> Result<InstallationInfo> {
        debug!("Detecting Ollama installation...");
        
        // Check if Ollama is running by connecting to API
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| ProviderError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;
        
        let response = client
            .get("http://localhost:11434/api/tags")
            .send()
            .await;
            
        match response {
            Ok(resp) if resp.status().is_success() => {
                // Try to parse response to get version
                let version = match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(models) = json.get("models").and_then(|v| v.as_array()) {
                            format!("Ollama (with {} models)", models.len())
                        } else {
                            "Ollama (running)".to_string()
                        }
                    },
                    Err(_) => "Ollama (running)".to_string(),
                };
                
                // Find executable path
                let executable_path = Self::find_ollama_executable().await
                    .unwrap_or_else(|| PathBuf::from("unknown"));
                
                Ok(InstallationInfo {
                    provider_type: ProviderType::Ollama,
                    status: InstallationStatus::Installed {
                        location: executable_path,
                        version,
                    },
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    auto_configured: true,
                })
            },
            _ => {
                // Ollama is not running, check if executable exists
                match Self::find_ollama_executable().await {
                    Some(path) => {
                        Ok(InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::PartiallyInstalled {
                                reason: "Ollama is installed but not running".to_string(),
                                location: Some(path),
                            },
                            last_checked: chrono::Utc::now().to_rfc3339(),
                            auto_configured: false,
                        })
                    },
                    None => {
                        Ok(InstallationInfo {
                            provider_type: ProviderType::Ollama,
                            status: InstallationStatus::NotInstalled,
                            last_checked: chrono::Utc::now().to_rfc3339(),
                            auto_configured: false,
                        })
                    }
                }
            }
        }
    }
    
    /// Find Ollama executable
    async fn find_ollama_executable() -> Option<PathBuf> {
        // Platform-specific executable name
        let executable_name = if cfg!(target_os = "windows") {
            "ollama.exe"
        } else {
            "ollama"
        };
        
        // Try to find in PATH
        if let Ok(path) = which::which(executable_name) {
            return Some(path);
        }
        
        // Check platform-specific common locations
        let common_locations = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from(r"C:\Program Files\Ollama").join(executable_name),
                PathBuf::from(r"C:\Program Files (x86)\Ollama").join(executable_name),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                PathBuf::from("/Applications/Ollama.app/Contents/MacOS").join(executable_name),
                PathBuf::from("/usr/local/bin").join(executable_name),
                PathBuf::from("/opt/homebrew/bin").join(executable_name),
            ]
        } else {
            vec![
                PathBuf::from("/usr/local/bin").join(executable_name),
                PathBuf::from("/usr/bin").join(executable_name),
                PathBuf::from("/opt/ollama").join(executable_name),
            ]
        };
        
        // Check all locations
        for path in common_locations {
            if path.exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    /// Detect LocalAI provider
    async fn detect_localai() -> Result<InstallationInfo> {
        debug!("Detecting LocalAI installation...");
        
        // Check common ports for LocalAI
        let ports = [8080, 8000, 8081, 8765];
        
        // Try connecting to possible LocalAI instances
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| ProviderError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;
            
        for port in ports {
            let url = format!("http://localhost:{}/v1/models", port);
            let response = client.get(&url).send().await;
            
            if let Ok(resp) = response {
                if resp.status().is_success() {
                    // Try to parse response
                    let version = match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(data) = json.get("data").and_then(|v| v.as_array()) {
                                format!("LocalAI (with {} models, port {})", data.len(), port)
                            } else {
                                format!("LocalAI (port {})", port)
                            }
                        },
                        Err(_) => format!("LocalAI (port {})", port),
                    };
                    
                    return Ok(InstallationInfo {
                        provider_type: ProviderType::LocalAI,
                        status: InstallationStatus::Installed {
                            location: PathBuf::from(format!("http://localhost:{}", port)),
                            version,
                        },
                        last_checked: chrono::Utc::now().to_rfc3339(),
                        auto_configured: true,
                    });
                }
            }
        }
        
        // Check if Docker is available and has LocalAI image
        if which::which("docker").is_ok() {
            let output = Command::new("docker")
                .args(["images", "localai/localai", "--format", "{{.Tag}}"])
                .output();
                
            match output {
                Ok(output) if !output.stdout.is_empty() => {
                    // LocalAI image exists but not running
                    return Ok(InstallationInfo {
                        provider_type: ProviderType::LocalAI,
                        status: InstallationStatus::PartiallyInstalled {
                            reason: "LocalAI Docker image found but server is not running".to_string(),
                            location: Some(PathBuf::from("/var/lib/docker")),
                        },
                        last_checked: chrono::Utc::now().to_rfc3339(),
                        auto_configured: false,
                    });
                },
                _ => {}
            }
        }
        
        // Not found
        Ok(InstallationInfo {
            provider_type: ProviderType::LocalAI,
            status: InstallationStatus::NotInstalled,
            last_checked: chrono::Utc::now().to_rfc3339(),
            auto_configured: false,
        })
    }
    
    /// Detect llama.cpp provider
    async fn detect_llamacpp() -> Result<InstallationInfo> {
        debug!("Detecting llama.cpp installation...");
        
        // Platform-specific executable names
        let possible_names = if cfg!(target_os = "windows") {
            vec!["llama.exe", "llama-server.exe", "llama-main.exe", "server.exe"]
        } else {
            vec!["llama", "llama-server", "llama-main", "server"]
        };
        
        // Try to find in PATH
        for name in &possible_names {
            if let Ok(path) = which::which(name) {
                // Try to get version
                let version = Self::get_llamacpp_version(&path).await
                    .unwrap_or_else(|_| "unknown".to_string());
                
                return Ok(InstallationInfo {
                    provider_type: ProviderType::LlamaCpp,
                    status: InstallationStatus::Installed {
                        location: path,
                        version,
                    },
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    auto_configured: true,
                });
            }
        }
        
        // Check common locations
        let common_locations = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from(r"C:\Program Files\llama.cpp").join("llama.exe"),
                PathBuf::from(r"C:\Program Files (x86)\llama.cpp").join("llama.exe"),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                PathBuf::from("/Applications/llama.cpp").join("llama"),
                PathBuf::from("/usr/local/bin/llama"),
                PathBuf::from("/opt/homebrew/bin/llama"),
            ]
        } else {
            vec![
                PathBuf::from("/usr/local/bin/llama"),
                PathBuf::from("/usr/bin/llama"),
                PathBuf::from("/opt/llama.cpp/llama"),
            ]
        };
        
        // Check all locations
        for path in common_locations {
            if path.exists() {
                // Try to get version
                let version = Self::get_llamacpp_version(&path).await
                    .unwrap_or_else(|_| "unknown".to_string());
                
                return Ok(InstallationInfo {
                    provider_type: ProviderType::LlamaCpp,
                    status: InstallationStatus::Installed {
                        location: path,
                        version,
                    },
                    last_checked: chrono::Utc::now().to_rfc3339(),
                    auto_configured: true,
                });
            }
        }
        
        // Not found
        Ok(InstallationInfo {
            provider_type: ProviderType::LlamaCpp,
            status: InstallationStatus::NotInstalled,
            last_checked: chrono::Utc::now().to_rfc3339(),
            auto_configured: false,
        })
    }
    
    /// Get llama.cpp version
    async fn get_llamacpp_version(path: &Path) -> Result<String> {
        // Try various version flags
        let version_flags = [
            &["--version"][..],
            &["-v"][..],
            &["--help"][..], // Some versions only show version in help
        ];
        
        for flags in &version_flags {
            let output = Command::new(path)
                .args(flags)
                .output();
                
            if let Ok(output) = output {
                // Check both stdout and stderr
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}{}", stdout, stderr);
                
                // Try to extract version with regex
                let version_patterns = [
                    r"version\s+([0-9]+\.[0-9]+\.[0-9]+)",
                    r"v([0-9]+\.[0-9]+\.[0-9]+)",
                    r"llama\.cpp\s+([0-9]+\.[0-9]+\.[0-9]+)",
                ];
                
                for pattern in &version_patterns {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if let Some(captures) = regex.captures(&combined) {
                            if let Some(version) = captures.get(1) {
                                return Ok(format!("llama.cpp {}", version.as_str()));
                            }
                        }
                    }
                }
                
                // If we found output but no version, just return a generic version
                if !combined.is_empty() {
                    return Ok("llama.cpp (unknown version)".to_string());
                }
            }
        }
        
        // No version info found
        Err(ProviderError::ConfigurationError("Failed to determine llama.cpp version".into()))
    }
    
    /// Scan a custom path for LLM providers
    async fn scan_custom_path(path: &Path) -> Result<HashMap<String, InstallationInfo>> {
        let mut results = HashMap::new();
        
        if !path.exists() || !path.is_dir() {
            return Ok(results);
        }
        
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => return Err(ProviderError::IoError(e)),
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Check if directory contains LLM-related files
                if let Ok(subentries) = std::fs::read_dir(&entry_path) {
                    let has_llm_files = subentries
                        .filter_map(Result::ok)
                        .any(|e| {
                            let name = e.file_name().to_string_lossy().to_lowercase();
                            name.contains("llm") || name.contains("model") || 
                            name.contains("weight") || name.contains("gguf") || 
                            name.contains("ggml")
                        });
                    
                    if has_llm_files {
                        let provider_name = entry_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                            
                        let provider_type = ProviderType::Custom(provider_name.clone());
                        
                        results.insert(
                            provider_type.to_string(),
                            InstallationInfo {
                                provider_type,
                                status: InstallationStatus::Installed {
                                    location: entry_path.clone(),
                                    version: "custom".to_string(),
                                },
                                last_checked: chrono::Utc::now().to_rfc3339(),
                                auto_configured: false,
                            },
                        );
                    }
                }
            } else if entry_path.is_file() && Self::is_executable(&entry_path) {
                // Check if executable name is LLM-related
                let name = entry_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                    
                if name.contains("llm") || name.contains("model") || 
                   name.contains("ai") || name.contains("llama") {
                    let provider_name = entry_path
                        .file_stem()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                        
                    let provider_type = ProviderType::Custom(provider_name.clone());
                    
                    results.insert(
                        provider_type.to_string(),
                        InstallationInfo {
                            provider_type,
                            status: InstallationStatus::Installed {
                                location: entry_path.clone(),
                                version: "custom".to_string(),
                            },
                            last_checked: chrono::Utc::now().to_rfc3339(),
                            auto_configured: false,
                        },
                    );
                }
            }
        }
        
        Ok(results)
    }
    
    /// Auto-configure detected providers
    async fn auto_configure_providers(installations: &HashMap<String, InstallationInfo>) {
        let factory = get_provider_factory();
        
        // Auto-configure Ollama
        if let Some(info) = installations.get(&ProviderType::Ollama.to_string()) {
            if let InstallationStatus::Installed { .. } = info.status {
                let config = OllamaConfig::default();
                
                let provider_result = factory.create_provider(
                    ProviderType::Ollama,
                    ProviderConfig {
                        ollama: Some(config),
                        ..Default::default()
                    }
                );
                
                if let Ok(provider) = provider_result {
                    factory.register_provider(provider);
                    info!("Auto-configured Ollama provider");
                }
            }
        }
        
        // Auto-configure LocalAI
        if let Some(info) = installations.get(&ProviderType::LocalAI.to_string()) {
            if let InstallationStatus::Installed { ref location, .. } = info.status {
                // Extract port from location path if it's a URL
                let location_str = location.to_string_lossy();
                let port = if location_str.starts_with("http://") {
                    location_str
                        .strip_prefix("http://localhost:")
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(8080)
                } else {
                    8080 // Default port
                };
                
                let mut config = LocalAIConfig::default();
                config.endpoint = format!("http://localhost:{}", port);
                
                let provider_result = factory.create_provider(
                    ProviderType::LocalAI,
                    ProviderConfig {
                        localai: Some(config),
                        ..Default::default()
                    }
                );
                
                if let Ok(provider) = provider_result {
                    factory.register_provider(provider);
                    info!("Auto-configured LocalAI provider with endpoint: {}", format!("http://localhost:{}", port));
                }
            }
        }
    }
    
    /// Check if a file is executable
    fn is_executable(path: &Path) -> bool {
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
    
    /// Create provider infos for UI
    pub fn create_provider_infos(&self) -> Vec<super::factory::ProviderInfo> {
        let factory = get_provider_factory();
        let mut provider_infos = Vec::new();
        
        // Get detected providers
        let suggestions = self.suggestions.lock().unwrap();
        
        for suggestion in suggestions.iter() {
            let detected = suggestion.detected;
            
            // Create basic info
            let info = super::factory::ProviderInfo {
                provider_type: suggestion.provider_type.clone(),
                name: match &suggestion.provider_type {
                    ProviderType::Ollama => "Ollama".to_string(),
                    ProviderType::LocalAI => "LocalAI".to_string(),
                    ProviderType::LlamaCpp => "llama.cpp".to_string(),
                    ProviderType::Custom(name) => name.clone(),
                },
                description: suggestion.description.clone(),
                detected,
                installation_url: Some(suggestion.instructions_url.clone()),
                recommended: suggestion.recommended_for.contains(&"Easy setup".to_string()),
            };
            
            provider_infos.push(info);
        }
        
        provider_infos
    }
}

/// Run LLM provider discovery
pub async fn run_discovery() -> Result<()> {
    let service = DiscoveryService::new();
    service.scan_for_providers().await?;
    Ok(())
}