# Developer Documentation: Local LLM Architecture

This document provides comprehensive technical information for developers who want to understand, extend, or modify the local LLM integration in the MCP Client.

## Architecture Overview

The local LLM system is designed with a provider-based architecture that allows for flexibility, extensibility, and easy integration of different LLM backends.

### Key Components

![Architecture Diagram](../assets/images/llm_architecture.png)

1. **Provider Interface**: A common interface that all LLM providers must implement
2. **Provider Manager**: Central component that manages provider configurations and selection
3. **Discovery Service**: Auto-detection and configuration of installed providers
4. **Provider Implementations**: Specific implementations for each supported provider (Ollama, LocalAI, llama.cpp, Custom)
5. **Migration System**: Handles migration from legacy LLM systems
6. **Metrics Collection**: Performance monitoring and telemetry
7. **Frontend Components**: UI elements for configuring and using local LLMs

### Design Principles

The architecture follows these key principles:

- **Abstraction**: Common interface hiding implementation details
- **Loose Coupling**: Components interact through well-defined interfaces
- **Extensibility**: Easy addition of new provider types
- **Configuration**: Flexible configuration options for each provider
- **Auto-discovery**: Automatic detection of installed providers
- **Fallback**: Graceful degradation when providers are unavailable

## Core Components

### Provider Interface

The core of the system is the `Provider` trait, which defines the contract that all LLM providers must implement:

```rust
pub trait Provider: Send + Sync {
    /// Get provider type
    fn get_type(&self) -> ProviderType;
    
    /// Get provider name
    fn get_name(&self) -> String;
    
    /// Get provider description
    fn get_description(&self) -> String;
    
    /// Get provider version
    fn get_version(&self) -> String;
    
    /// Check if provider is available
    fn is_available(&self) -> Result<bool>;
    
    /// List available models
    fn list_available_models(&self) -> Result<Vec<ModelInfo>>;
    
    /// List downloaded models
    fn list_downloaded_models(&self) -> Result<Vec<ModelInfo>>;
    
    /// Download a model
    fn download_model(&self, model_id: &str) -> Result<()>;
    
    /// Cancel a model download
    fn cancel_download(&self, model_id: &str) -> Result<()>;
    
    /// Get download status
    fn get_download_status(&self, model_id: &str) -> Result<DownloadStatus>;
    
    /// Delete a model
    fn delete_model(&self, model_id: &str) -> Result<()>;
    
    /// Generate text using a specified model
    fn generate_text(&self, request: GenerationRequest) -> Result<GenerationResponse>;
}
```

This interface ensures that all providers offer a consistent set of capabilities regardless of their underlying implementation.

### Provider Types

The system defines a `ProviderType` enum to identify different types of providers:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    Ollama,
    LocalAI,
    LlamaCpp,
    Custom(String),
}
```

Custom providers can be dynamically added by specifying a name in the `Custom` variant.

### Provider Manager

The `ProviderManager` is the central component that manages provider configurations and handles requests:

```rust
pub struct ProviderManager {
    /// Available providers
    providers: Mutex<HashMap<String, ProviderInfo>>,
    /// Provider availability status
    availability: Mutex<HashMap<String, AvailabilityResult>>,
    /// Active provider
    active_provider: Mutex<Option<ProviderType>>,
    /// Provider configurations
    configs: Mutex<HashMap<String, ProviderConfig>>,
    /// LLM instances for each provider
    llm_instances: Mutex<HashMap<String, Arc<LocalLLM>>>,
}
```

Key responsibilities:
- Managing provider registration and availability
- Storing and retrieving provider configurations
- Selecting the active provider
- Routing text generation requests to the appropriate provider
- Managing model downloads and status

### Discovery Service

The `DiscoveryService` is responsible for detecting installed LLM providers:

```rust
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
```

Key capabilities:
- Scanning the system for installed providers
- Creating provider configurations for detected providers
- Suggesting providers that could be installed
- Background scanning to detect new or updated providers

### Migration System

The `MigrationService` handles migration from legacy LLM systems:

```rust
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
```

This service:
- Detects legacy LLM systems
- Maps legacy configurations to new provider configurations
- Migrates models from legacy to new system
- Provides fallback options when migration fails

## Implementation Details

### Provider Implementations

Each provider implementation must satisfy the `Provider` trait. Here are the key details for each built-in provider:

#### Ollama Provider

The Ollama provider integrates with the [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md):

- **API Endpoints**:
  - List models: `GET http://localhost:11434/api/tags`
  - Pull model: `POST http://localhost:11434/api/pull`
  - Generate: `POST http://localhost:11434/api/generate`

- **Model Handling**:
  - Models are identified by their tag (e.g., `llama2:7b`)
  - Model download is managed through the Ollama API
  - Model files are stored in Ollama's own directory structure

#### LocalAI Provider

The LocalAI provider integrates with the [LocalAI API](https://github.com/go-skynet/LocalAI/blob/master/docs/openai-compatibility.md), which is compatible with the OpenAI API:

- **API Endpoints**:
  - List models: `GET http://localhost:8080/models`
  - Completion: `POST http://localhost:8080/v1/completions`
  - Chat: `POST http://localhost:8080/v1/chat/completions`

- **Model Handling**:
  - Models must be manually placed in LocalAI's model directory
  - The provider detects available models from the API
  - Model files are not directly managed by the provider

#### llama.cpp Provider

The llama.cpp provider interacts with the [llama.cpp server API](https://github.com/ggerganov/llama.cpp/blob/master/examples/server/README.md):

- **API Endpoints**:
  - Completion: `POST http://localhost:8000/completion`
  - Info: `GET http://localhost:8000/info`

- **Model Handling**:
  - Models are loaded directly by the llama.cpp server
  - Only one model can be loaded at a time
  - Model switching requires restarting the server

#### Custom Provider

The Custom provider allows for integration with any compatible LLM API:

- **Configuration**:
  - Custom endpoint URL
  - Optional API key
  - Provider-specific parameters in advanced config

- **Implementation**:
  - Adapts to the API of the custom service
  - Provides flexible request and response mapping

### Data Structures

#### Provider Configuration

Provider configurations are stored in the `ProviderConfig` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type identifier
    pub provider_type: String,
    /// Endpoint URL for the provider
    pub endpoint_url: String,
    /// API key for the provider (if required)
    pub api_key: Option<String>,
    /// Default model to use
    pub default_model: Option<String>,
    /// Whether to enable advanced configuration
    pub enable_advanced_config: bool,
    /// Advanced configuration options (provider-specific)
    pub advanced_config: HashMap<String, serde_json::Value>,
}
```

This configuration is stored in the application's settings and can be updated through the UI.

#### Model Information

Information about models is represented by the `EnhancedModelInfo` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedModelInfo {
    /// Model identifier
    pub id: String,
    /// Model name for display
    pub name: String,
    /// Model description
    pub description: String,
    /// Model size in bytes
    pub size_bytes: usize,
    /// Whether the model is downloaded
    pub is_downloaded: bool,
    /// Provider-specific metadata
    pub provider_metadata: HashMap<String, serde_json::Value>,
    /// Provider type
    pub provider: String,
    /// Whether the model supports text generation
    pub supports_text_generation: bool,
    /// Whether the model supports completion
    pub supports_completion: bool,
    /// Whether the model supports chat
    pub supports_chat: bool,
    /// Whether the model supports embeddings
    pub supports_embeddings: bool,
    /// Whether the model supports image generation
    pub supports_image_generation: bool,
    /// Quantization level (if applicable)
    pub quantization: Option<String>,
    /// Parameter count in billions
    pub parameter_count_b: Option<f32>,
    /// Context length in tokens
    pub context_length: Option<usize>,
    /// Model family/architecture
    pub model_family: Option<String>,
    /// When the model was created
    pub created_at: Option<String>,
    /// Model tags
    pub tags: Vec<String>,
    /// Model license
    pub license: Option<String>,
}
```

This structure provides comprehensive information about models, including capabilities, technical details, and metadata.

#### Download Status

The status of model downloads is tracked with the `EnhancedDownloadStatus` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDownloadStatus {
    /// Status type 
    pub status: String,
    /// Not started status (empty object)
    pub NotStarted: Option<HashMap<String, serde_json::Value>>,
    /// In progress status
    pub InProgress: Option<InProgressStatus>,
    /// Completed status
    pub Completed: Option<CompletedStatus>,
    /// Failed status
    pub Failed: Option<FailedStatus>,
    /// Cancelled status
    pub Cancelled: Option<CancelledStatus>,
}
```

This structure allows for detailed tracking of download progress, errors, and completions.

### Command API

The system exposes several Tauri commands for interacting with the LLM providers:

#### Provider Management Commands

- `get_all_providers`: Get all available providers
- `get_all_provider_availability`: Check availability of all providers
- `check_provider_availability`: Check availability of a specific provider
- `get_active_provider`: Get the active provider
- `set_active_provider`: Set the active provider
- `get_provider_config`: Get the configuration for a provider
- `update_provider_config`: Update the configuration for a provider

#### Model Management Commands

- `list_available_models`: List available models for a provider
- `list_downloaded_models`: List downloaded models for a provider
- `get_download_status`: Get download status for a model
- `download_model`: Download a model
- `cancel_download`: Cancel a model download
- `delete_model`: Delete a model

#### Text Generation Commands

- `generate_text`: Generate text using a model
- `stream_text`: Stream text generation (for UI responsiveness)

#### Discovery Commands

- `scan_for_providers`: Scan for LLM providers
- `get_discovery_status`: Get provider discovery status
- `get_provider_suggestions`: Get suggestions for providers to install
- `get_discovery_config`: Get discovery service configuration
- `update_discovery_config`: Update discovery service configuration
- `auto_configure_providers`: Auto-configure detected providers

#### Migration Commands

- `check_legacy_system`: Check for legacy LLM system
- `get_migration_status`: Get migration status
- `run_migration`: Run migration
- `get_migration_config`: Get migration configuration
- `update_migration_config`: Update migration configuration
- `opt_out_of_migration`: Opt out of migration

## Frontend Integration

### Offline Settings Component

The primary UI for configuring local LLMs is the `OfflineSettings` component:

```tsx
const OfflineSettings: React.FC = () => {
  // State for offline mode
  const [isOfflineMode, setIsOfflineMode] = useState<boolean>(false);
  const [autoSwitchMode, setAutoSwitchMode] = useState<boolean>(true);
  
  // State for providers
  const [availableProviders, setAvailableProviders] = useState<ProviderInfo[]>([]);
  const [selectedProviderType, setSelectedProviderType] = useState<string>('Ollama');
  
  // State for models
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [downloadedModels, setDownloadedModels] = useState<ModelInfo[]>([]);
  
  // Additional state and functions...
  
  return (
    // UI implementation...
  );
};
```

This component:
- Displays offline mode settings
- Shows provider selection and configuration
- Lists available and downloaded models
- Handles model downloads and management
- Displays model information and status

### LLM Performance Dashboard

The `LLMPerformanceDashboard` component displays metrics about local LLM performance:

```tsx
const LLMPerformanceDashboard: React.FC = () => {
  // State for metrics
  const [metrics, setMetrics] = useState<LLMMetrics | null>(null);
  
  // Fetch metrics on mount and periodically
  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        const response = await invoke('get_llm_metrics');
        if (response.success && response.data) {
          setMetrics(response.data);
        }
      } catch (error) {
        console.error('Failed to fetch metrics:', error);
      }
    };
    
    fetchMetrics();
    const interval = setInterval(fetchMetrics, 5000);
    
    return () => clearInterval(interval);
  }, []);
  
  return (
    // UI implementation with charts and stats...
  );
};
```

This dashboard shows:
- Generation throughput and latency
- Model-specific performance stats
- Resource usage metrics
- Success and error rates

## Extension Guide

### Adding a New Provider

To add a new LLM provider:

1. **Create Provider Implementation**:

```rust
pub struct MyProvider {
    endpoint: String,
    api_key: Option<String>,
    http_client: reqwest::Client,
}

impl Provider for MyProvider {
    fn get_type(&self) -> ProviderType {
        ProviderType::Custom("MyProvider".to_string())
    }
    
    fn get_name(&self) -> String {
        "My Custom Provider".to_string()
    }
    
    // Implement remaining methods...
}
```

2. **Register the Provider**:

```rust
// In provider_manager.rs
impl ProviderManager {
    pub fn register_providers(&mut self) {
        // Register built-in providers
        self.register_provider(Box::new(OllamaProvider::new()));
        self.register_provider(Box::new(LocalAIProvider::new()));
        self.register_provider(Box::new(LlamaCppProvider::new()));
        
        // Register your custom provider
        self.register_provider(Box::new(MyProvider::new()));
    }
}
```

3. **Add Provider Configuration UI**:

Update the `OfflineSettings.tsx` component to include configuration options for your provider:

```tsx
{selectedProviderType === 'MyProvider' && (
  <>
    <FormControl fullWidth sx={{ mb: 2 }}>
      <FormLabel>My Provider Endpoint</FormLabel>
      <TextField
        value={providerEndpoint}
        onChange={(e) => setProviderEndpoint(e.target.value)}
        placeholder="http://localhost:9000"
      />
    </FormControl>
    
    {/* Additional configuration fields */}
  </>
)}
```

4. **Update Documentation**:

Add information about your provider to the documentation, including:
- Installation instructions
- Configuration details
- Supported models
- Performance characteristics
- Troubleshooting tips

### Custom Model Formats

To support custom model formats:

1. **Update the Model Information Structure**:

```rust
pub struct ModelInfo {
    // Existing fields...
    
    // Add custom format information
    pub format: Option<String>,
    pub format_version: Option<String>,
    pub format_specific_data: Option<HashMap<String, serde_json::Value>>,
}
```

2. **Implement Format-Specific Handling**:

```rust
impl MyProvider {
    fn handle_custom_format(&self, model_id: &str, format: &str) -> Result<()> {
        match format {
            "my-format" => {
                // Format-specific handling
                Ok(())
            },
            _ => Err(anyhow!("Unsupported format: {}", format)),
        }
    }
}
```

### Metrics Collection

To collect additional metrics for your provider:

1. **Define Custom Metrics**:

```rust
pub struct MyProviderMetrics {
    pub inference_time_ms: f64,
    pub memory_usage_mb: f64,
    pub tokens_per_second: f64,
    pub custom_metric: f64,
}
```

2. **Implement Metrics Collection**:

```rust
impl MyProvider {
    fn collect_metrics(&self, start: Instant, tokens: usize) -> MyProviderMetrics {
        let elapsed = start.elapsed();
        let inference_time_ms = elapsed.as_millis() as f64;
        let tokens_per_second = tokens as f64 / (inference_time_ms / 1000.0);
        
        MyProviderMetrics {
            inference_time_ms,
            memory_usage_mb: self.measure_memory_usage(),
            tokens_per_second,
            custom_metric: self.get_custom_metric(),
        }
    }
    
    fn measure_memory_usage(&self) -> f64 {
        // Implementation-specific memory measurement
        0.0
    }
    
    fn get_custom_metric(&self) -> f64 {
        // Custom metric measurement
        0.0
    }
}
```

3. **Register Metrics**:

```rust
// In metrics.rs
pub fn register_provider_metrics(
    registry: &mut MetricsRegistry,
    provider_type: &str,
    metrics: &MyProviderMetrics,
) {
    registry.register_gauge(
        format!("llm.{}.inference_time_ms", provider_type),
        metrics.inference_time_ms,
    );
    
    // Register other metrics...
}
```

## Testing

### Unit Testing

When implementing a new provider, ensure comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_creation() {
        let provider = MyProvider::new(
            "http://localhost:9000".to_string(),
            None,
        );
        
        assert_eq!(provider.get_type().to_string(), "Custom(MyProvider)");
        assert_eq!(provider.get_name(), "My Custom Provider");
    }
    
    #[tokio::test]
    async fn test_provider_availability() {
        let provider = MyProvider::new(
            "http://localhost:9000".to_string(),
            None,
        );
        
        // Mock HTTP client for testing
        let mock_client = MockHttpClient::new()
            .with_response("/info", json!({"version": "1.0.0"}));
        provider.set_http_client(mock_client);
        
        let result = provider.is_available().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    // Additional tests...
}
```

### Integration Testing

Verify the integration with the provider manager:

```rust
#[tokio::test]
async fn test_provider_registration() {
    let mut manager = ProviderManager::default();
    
    // Register custom provider
    manager.register_provider(Box::new(MyProvider::new(
        "http://localhost:9000".to_string(),
        None,
    )));
    
    // Verify provider is registered
    let providers = manager.get_all_providers().await.unwrap();
    assert!(providers.iter().any(|p| p.provider_type == "Custom(MyProvider)"));
}
```

## Performance Considerations

When implementing a provider, consider these performance aspects:

1. **Memory Management**:
   - Use streaming responses where possible
   - Minimize copying of large data buffers
   - Consider memory usage during model loading and inference

2. **Concurrency**:
   - Implement non-blocking API calls
   - Handle multiple concurrent requests efficiently
   - Use appropriate thread pools for CPU-bound operations

3. **Timeouts and Error Handling**:
   - Implement timeouts for API calls
   - Provide clear error messages
   - Handle network failures gracefully

4. **Resource Management**:
   - Clean up resources when no longer needed
   - Release memory when models are unloaded
   - Implement proper shutdown sequences

## Security Considerations

When implementing providers, consider these security aspects:

1. **API Key Handling**:
   - Store API keys securely
   - Avoid logging API keys
   - Use environment variables where appropriate

2. **Network Security**:
   - Validate endpoint URLs
   - Use HTTPS for external endpoints
   - Implement proper certificate validation

3. **Input Validation**:
   - Validate all user inputs
   - Sanitize prompts where necessary
   - Implement appropriate content filtering

4. **Local File Access**:
   - Validate file paths
   - Prevent path traversal attacks
   - Use appropriate file permissions

## Conclusion

The provider-based architecture of the local LLM integration in MCP Client allows for flexible and extensible integration of different LLM backends. By following this documentation, developers can understand the existing implementation, extend it with new providers, and implement custom functionality.

Key takeaways:
- The `Provider` trait defines a common interface for all LLM backends
- The `ProviderManager` centralizes provider management and request routing
- The `DiscoveryService` automatically detects and configures installed providers
- The architecture supports easy extension with new provider types
- Comprehensive metrics collection helps monitor and optimize performance

As local LLMs continue to evolve rapidly, this architecture provides a solid foundation for incorporating new technologies and capabilities while maintaining backward compatibility and a consistent user experience.

## References

- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [LocalAI Documentation](https://github.com/go-skynet/LocalAI/blob/master/docs/openai-compatibility.md)
- [llama.cpp Server API](https://github.com/ggerganov/llama.cpp/blob/master/examples/server/README.md)
- [Tauri Command API](https://tauri.app/v1/guides/features/command/)
- [React Hooks Documentation](https://reactjs.org/docs/hooks-intro.html)
