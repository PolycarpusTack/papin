# Local LLM Integration

This document provides comprehensive information about the local LLM (Large Language Model) integration in the MCP Client, including user guides, developer documentation, and troubleshooting information.

## Overview

The MCP Client supports local LLM providers to enable offline capabilities and reduce dependency on remote API services. Local LLMs can be used for:

- Text generation when offline
- Chat conversations without internet connectivity
- Reduced latency for certain operations
- Privacy-sensitive workloads that should not leave the device
- Fallback when cloud services are unavailable

The implementation follows a provider-based approach, allowing different LLM backends to be integrated through a common interface. This architecture enables users to choose the best solution for their specific hardware, needs, and preferences.

## User Guide

### Setting Up Local LLM Providers

#### Enabling Offline Mode

1. Open the MCP Client application
2. Navigate to Settings > Offline
3. Toggle "Enable Offline Mode" to enable local LLM capabilities
4. Optionally, enable "Auto-switch based on connectivity" to automatically switch between local and cloud models based on your network status

#### Understanding Provider Selection

The MCP Client supports various LLM providers, each with different characteristics:

- **Ollama**: User-friendly, wide model support, good performance
- **LocalAI**: OpenAI-compatible API, extensive customization
- **llama.cpp**: Optimal performance, specialized for local inference
- **Custom**: For advanced users with custom LLM implementations

Choose the provider that best suits your needs based on:
- Ease of use
- Performance requirements
- Model availability
- Hardware capabilities

#### Configuring LLM Providers

Each provider requires its own setup:

##### Ollama

[Ollama](https://ollama.ai/) is a user-friendly tool for running various LLMs locally.

1. **Installation**:
   - Download and install Ollama from [https://ollama.ai/download](https://ollama.ai/download)
   - Follow the installation instructions for your platform
   - Ensure Ollama is running (it typically starts automatically)

2. **Configuration in MCP Client**:
   - In Settings > Offline, select "Ollama" as your Local LLM Provider
   - The default endpoint is `http://localhost:11434` (usually no change needed)
   - No API key is required for Ollama

3. **Downloading Models**:
   - In the "Models" tab, browse available models
   - Click the download icon next to a model to download it
   - Wait for the download to complete (this may take time depending on the model size)
   - Optionally, set a default model to use when in offline mode

##### LocalAI

[LocalAI](https://github.com/go-skynet/LocalAI) is an API compatible with OpenAI's API but running locally.

1. **Installation**:
   - Follow the installation instructions at [LocalAI GitHub repository](https://github.com/go-skynet/LocalAI)
   - Start the LocalAI server using the provided scripts or Docker commands

2. **Configuration in MCP Client**:
   - In Settings > Offline, select "LocalAI" as your Local LLM Provider
   - Set the endpoint URL (default is `http://localhost:8080`)
   - No API key is required by default, but you can configure one if you've set up authentication

3. **Downloading Models**:
   - LocalAI models are managed through the LocalAI interface
   - In the "Models" tab of MCP Client, you'll see available models detected from LocalAI
   - Download models using the LocalAI interface, and they will appear in MCP Client

##### llama.cpp

[llama.cpp](https://github.com/ggerganov/llama.cpp) is a popular implementation of LLaMA models optimized for CPU usage.

1. **Installation**:
   - Clone and build the repository from [llama.cpp GitHub](https://github.com/ggerganov/llama.cpp)
   - Start the server with `./server -m /path/to/model.gguf`

2. **Configuration in MCP Client**:
   - In Settings > Offline, select "LlamaCpp" as your Local LLM Provider
   - Set the endpoint URL (default is `http://localhost:8000`)
   - No API key is required

3. **Downloading Models**:
   - Download GGUF models from [Hugging Face](https://huggingface.co/)
   - Point llama.cpp server to your model file
   - MCP Client will detect the model from the server

##### Custom Provider

For advanced users who have custom LLM servers:

1. **Setup**:
   - Ensure your custom provider has a REST API endpoint
   - Configure authentication if required

2. **Configuration in MCP Client**:
   - In Settings > Offline, select "Custom" as your Local LLM Provider
   - Enter the endpoint URL for your custom provider
   - Configure API key if required
   - Use Advanced Configuration to fine-tune provider-specific settings

### Managing Models

#### Downloading Models

1. In Settings > Offline, browse to the Models tab
2. View available models for your selected provider
3. Click the download icon to start downloading a model
4. Monitor the download progress in the UI
5. Downloaded models will appear in the "Downloaded Models" tab

#### Setting a Default Model

1. Go to Settings > Offline
2. In the Local LLM Provider section, find the "Default Model" dropdown
3. Select a model from your downloaded models
4. This model will be used by default for offline operations

#### Managing Downloads

- **Cancel Download**: Click the pause icon during an in-progress download
- **Delete Model**: Click the trash icon next to a downloaded model to remove it
- **Model Info**: Click the info icon to view detailed information about the model

### Using Local LLMs

Once configured, local LLMs are used automatically when:

1. You are in offline mode (either manually enabled or auto-switched due to connectivity)
2. The client needs to generate text or respond to chat messages

You can see which model is being used in the conversation interface, which will indicate "Using local model: [model name]" when in offline mode.

### Performance Metrics

MCP Client includes a dashboard for monitoring LLM performance:

1. Go to Dashboard > LLM to view the performance metrics
2. The dashboard shows:
   - Generation counts and success rates
   - Throughput (tokens per second)
   - Latency metrics (average, p90, p99)
   - Model-specific performance statistics
   - Resource usage during inference

Note: Metrics collection is privacy-respecting and opt-in. You can configure it in the Settings > Offline section.

## Provider Comparison

| Feature | Ollama | LocalAI | llama.cpp | Custom |
|---------|--------|---------|-----------|--------|
| **Ease of Setup** | ★★★★★ | ★★★☆☆ | ★★☆☆☆ | ★☆☆☆☆ |
| **Performance** | ★★★★☆ | ★★★★☆ | ★★★★★ | Varies |
| **Model Variety** | ★★★★☆ | ★★★★★ | ★★★☆☆ | Varies |
| **Memory Usage** | ★★★☆☆ | ★★★★☆ | ★★★★★ | Varies |
| **API Compatibility** | Ollama API | OpenAI API | Simple API | Custom |
| **Quantization Options** | Limited | Extensive | Extensive | Varies |
| **GPU Support** | Yes | Yes | Yes | Varies |
| **Default Port** | 11434 | 8080 | 8000 | Custom |
| **UI for Management** | Yes | No | No | Varies |
| **Auto-discovery** | Yes | Yes | Limited | No |
| **Embedding Support** | Yes | Yes | Limited | Varies |

### Detailed Comparison

#### Ollama
- **Strengths**: Easy to set up and use, good selection of models, automatic model management
- **Weaknesses**: Less control over model loading parameters, limited quantization options
- **Best for**: Beginners, users who want a simple experience with good performance
- **Hardware Requirements**: 8GB RAM minimum, 16GB recommended, GPU optional but beneficial
- **Models**: Llama 2, Mistral, Vicuna, and many others with easy download commands

#### LocalAI
- **Strengths**: OpenAI API compatibility, extensive customization options, multi-modal support
- **Weaknesses**: More complex setup, requires more technical knowledge
- **Best for**: Users who need OpenAI compatibility or want extensive customization
- **Hardware Requirements**: 8GB RAM minimum, 16GB recommended, GPU optional
- **Models**: Compatible with GGUF, GGML, and other common formats

#### llama.cpp
- **Strengths**: Best performance, highly optimized, extensive control over parameters
- **Weaknesses**: More technical to set up, requires command-line knowledge
- **Best for**: Performance-focused users, limited hardware, technical users
- **Hardware Requirements**: 4GB RAM minimum (for small models), scales with model size
- **Models**: Primarily GGUF format models

#### Custom Providers
- **Strengths**: Complete flexibility, can integrate any LLM backend
- **Weaknesses**: Requires implementation and maintenance by user
- **Best for**: Advanced users, specialized use cases, research
- **Hardware Requirements**: Depends on implementation
- **Models**: Depends on implementation

### Recommendations

- **For beginners**: Start with Ollama, which offers the simplest setup experience
- **For best performance**: llama.cpp provides the most optimized inference
- **For OpenAI compatibility**: LocalAI offers the most compatible API
- **For advanced/custom needs**: Use Custom provider with your specific implementation

## Troubleshooting

### Common Issues

#### Provider Not Available

**Symptoms**:
- "Provider not available" warning in the UI
- Unable to see or download models

**Solutions**:
1. Ensure the provider software is installed and running
2. Check that the endpoint URL is correct
3. Verify network connectivity to the local service (try opening the URL in a browser)
4. Check for any firewalls blocking the connection
5. Restart the provider service and MCP Client

#### Model Download Failures

**Symptoms**:
- Download starts but fails before completion
- Error message in the download status

**Solutions**:
1. Check available disk space
2. Ensure you have stable internet connection during download
3. Try downloading a smaller model first
4. Check provider logs for specific errors:
   - Ollama: `~/.ollama/logs` or system logs
   - LocalAI: Check the terminal where LocalAI is running
   - llama.cpp: Check the terminal output
5. Restart the provider service and try again

#### High Memory Usage

**Symptoms**:
- System becomes slow when using local models
- Application crashes during text generation
- Out of memory errors

**Solutions**:
1. Use smaller models or models with higher quantization (e.g., q4_K_M instead of f16)
2. Close other memory-intensive applications
3. Increase system swap/page file size
4. For advanced users, adjust context window size in model parameters
5. Try a different provider that may have better memory management

#### Slow Generation Speed

**Symptoms**:
- Very slow responses when using local models
- Generation takes significantly longer than expected

**Solutions**:
1. Check CPU/GPU usage during generation
2. Try a smaller or more optimized model
3. Ensure you're using GPU acceleration if available:
   - Ollama automatically uses GPU if available
   - LocalAI may need specific configuration
   - llama.cpp needs to be compiled with appropriate flags
4. Reduce the model's context window size if configurable
5. Check for other processes using significant CPU/GPU resources

#### Offline Mode Not Working

**Symptoms**:
- Still using cloud APIs despite being offline
- Error messages when trying to generate text offline

**Solutions**:
1. Ensure offline mode is enabled in Settings
2. Verify a default model is selected
3. Check that the selected model is downloaded and available
4. Restart the provider service
5. Restart MCP Client
6. Check logs for specific errors

### Provider-Specific Troubleshooting

#### Ollama

**Common issues**:
- Service stops unexpectedly
- Model downloads incomplete

**Troubleshooting**:
1. Update Ollama to the latest version
2. Check logs: `~/.ollama/logs`
3. Run `ollama serve` manually to see console output
4. Remove and re-add problematic models: `ollama rm <model>`
5. Check GitHub issues for known problems

#### LocalAI

**Common issues**:
- Model compatibility
- API errors
- Configuration complexity

**Troubleshooting**:
1. Run LocalAI with verbose logging: `LOCAL_AI_DEBUG=1`
2. Check model formats are compatible
3. Verify configuration file has correct model paths
4. Update to the latest LocalAI version
5. Test API directly using curl before using through MCP Client

#### llama.cpp

**Common issues**:
- Compilation problems
- Model format compatibility
- Server configuration

**Troubleshooting**:
1. Ensure you're using GGUF model format (newer versions)
2. Check compilation flags match your hardware
3. Run server with verbose logging
4. Test direct API calls to isolate issues
5. Try different server parameters (threads, context size)

### Diagnostic Information

When reporting issues, please include:

1. MCP Client version
2. Provider type and version
3. Model name and size
4. System specifications (OS, CPU, RAM, GPU)
5. Relevant log entries
6. Steps to reproduce the issue

## Developer Documentation

### Architecture

The offline LLM capabilities are implemented with a provider-based architecture:

```
┌─────────────────────────┐
│    UI (Offline Settings)│
└───────────┬─────────────┘
            │
┌───────────▼─────────────┐       ┌───────────────────┐
│    Provider Manager     │◄──────►   Discovery Service│
└───────────┬─────────────┘       └───────────────────┘
            │
┌───────────▼─────────────┐
│     Provider Interface  │
└───────────┬─────────────┘
            │
     ┌──────┴─────────────────┐
     │                        │
┌────▼───┐  ┌─────────┐  ┌────▼───┐  ┌─────────┐
│ Ollama │  │ LocalAI │  │llama.cpp│  │ Custom  │
└────────┘  └─────────┘  └─────────┘  └─────────┘
```

Key components:

1. **Provider Interface**: Common interface for all LLM providers
2. **Provider Manager**: Manages provider configurations and selection
3. **Discovery Service**: Auto-detection of installed providers
4. **Provider Implementations**: Specific implementations for each supported provider
5. **Metrics Collection**: Performance monitoring for local models
6. **UI Components**: User interface for configuration and monitoring

### Key Files

#### Backend (Rust)

- **Provider Interface and Manager**:  
  `src/commands/offline/llm.rs` - Provider types and manager implementation

- **Provider Discovery**:  
  `src/offline/llm/discovery.rs` - Provider discovery logic  
  `src/offline/llm/migration.rs` - Migration from legacy systems

- **Local LLM Implementation**:  
  `src/offline/llm/mod.rs` - Core LLM functionality

- **Metrics Collection**:  
  `src/observability/metrics/llm.rs` - LLM-specific metrics

#### Frontend (TypeScript/React)

- **Settings UI**:  
  `src-frontend/src/components/offline/OfflineSettings.tsx` - Offline settings UI  
  `src-frontend/src/components/offline/LLMMetricsPrivacyNotice.tsx` - Privacy notice for metrics

- **Dashboard**:  
  `src-frontend/src/components/dashboard/llm/LLMPerformanceDashboard.tsx` - LLM metrics dashboard

### Provider Interface

The provider interface defines common functionality that all LLM providers must implement:

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

### Adding a New Provider

To add a new LLM provider:

1. **Create a new provider implementation**:
   ```rust
   pub struct MyProvider {
       // Provider-specific fields
   }
   
   impl Provider for MyProvider {
       // Implement required methods
   }
   ```

2. **Register the provider in the provider manager**:
   ```rust
   // In provider_manager.rs
   pub fn register_provider(&mut self, provider: Box<dyn Provider>) {
       let provider_type = provider.get_type().to_string();
       self.providers.insert(provider_type.clone(), provider);
   }
   ```

3. **Add provider-specific UI configuration**:
   - Update `OfflineSettings.tsx` to handle your provider configuration
   - Add any provider-specific settings

4. **Add metrics collection**:
   - Update metrics collection for provider-specific metrics

5. **Update documentation**:
   - Add provider details to this documentation
   - Include setup instructions and troubleshooting

### Performance Metrics Collection

The LLM metrics system collects the following information:

1. **Generation Metrics**:
   - Latency (time to complete generation)
   - Throughput (tokens per second)
   - Time to first token
   - Success/failure rates

2. **Provider Metrics**:
   - Available models
   - Success/failure of operations
   - Resource usage (CPU, memory)

3. **Model Metrics**:
   - Model-specific performance
   - Usage counts
   - Error rates

Metrics collection is privacy-focused and opt-in. The collected data is used to:

1. Provide dashboard visualizations for users
2. Optimize the application performance
3. Identify issues with specific models or providers

### Best Practices for Implementation

1. **Error Handling**:
   - Gracefully handle provider unavailability
   - Provide clear error messages to users
   - Fallback mechanisms when local inference fails

2. **Resource Management**:
   - Monitor memory usage during inference
   - Implement timeouts for operations
   - Release resources when not in use

3. **Configuration**:
   - Validate user-provided endpoints and settings
   - Store provider configurations securely
   - Implement sensible defaults

4. **Performance**:
   - Benchmark your provider implementation
   - Optimize for both speed and resource usage
   - Use async operations where appropriate

## Examples

### Using the API

#### Example 1: Basic Chat Generation with Ollama

```typescript
// In a React component
const handleLocalGeneration = async (prompt: string) => {
  try {
    const response = await invoke('generate_text', {
      provider_type: 'Ollama',
      model_id: 'llama2',
      prompt: prompt,
      max_tokens: 1000,
      temperature: 0.7,
      options: {}
    });
    
    setResponse(response.data);
    
  } catch (error) {
    console.error('Generation failed:', error);
    setError('Failed to generate text locally');
  }
};
```

#### Example 2: Streaming Generation with LocalAI

```typescript
// In a React component
const streamLocalGeneration = async (prompt: string) => {
  try {
    // Start streaming
    await invoke('stream_text', {
      provider_type: 'LocalAI',
      model_id: 'gpt-3.5-turbo',
      prompt: prompt,
      max_tokens: 1000,
      temperature: 0.7,
      options: {}
    });
    
    // Set up a listener for streaming events
    const unlisten = await listen('generation_chunk', (event) => {
      const chunk = event.payload as string;
      
      if (chunk === '[DONE]') {
        // Stream complete
        unlisten();
      } else {
        // Append chunk to the UI
        setResponse(prev => prev + chunk);
      }
    });
    
  } catch (error) {
    console.error('Streaming failed:', error);
    setError('Failed to stream text locally');
  }
};
```

#### Example 3: Advanced Configuration with llama.cpp

```typescript
// In a React component
const generateWithAdvancedConfig = async (prompt: string) => {
  try {
    const response = await invoke('generate_text', {
      provider_type: 'LlamaCpp',
      model_id: 'wizardlm-13b',
      prompt: prompt,
      max_tokens: 2000,
      temperature: 0.5,
      options: {
        // Provider-specific parameters
        context_size: 4096,
        repeat_penalty: 1.1,
        top_k: 40,
        top_p: 0.9,
        threads: 4,
        seed: 42
      }
    });
    
    setResponse(response.data);
    
  } catch (error) {
    console.error('Generation failed:', error);
    setError('Failed to generate text locally');
  }
};
```

### UI Integration Examples

#### Example 1: Showing Provider Status

```tsx
// In a React component
const ProviderStatus = ({ providerType }: { providerType: string }) => {
  const [isAvailable, setIsAvailable] = useState<boolean>(false);
  const [version, setVersion] = useState<string>('');
  
  useEffect(() => {
    const checkAvailability = async () => {
      try {
        const response = await invoke('check_provider_availability', { 
          provider_type: providerType 
        });
        
        if (response.success && response.data) {
          setIsAvailable(response.data.available);
          setVersion(response.data.version || 'Unknown');
        }
      } catch (error) {
        console.error('Failed to check provider:', error);
        setIsAvailable(false);
      }
    };
    
    checkAvailability();
  }, [providerType]);
  
  return (
    <div className="provider-status">
      <div className={`status-indicator ${isAvailable ? 'available' : 'unavailable'}`} />
      <div className="provider-info">
        <span className="provider-name">{providerType}</span>
        {isAvailable && <span className="provider-version">v{version}</span>}
      </div>
      <div className="status-text">
        {isAvailable ? 'Available' : 'Unavailable'}
      </div>
    </div>
  );
};
```

#### Example 2: Model Selection Dropdown

```tsx
// In a React component
const ModelSelector = ({ 
  providerType, 
  onModelSelect 
}: { 
  providerType: string;
  onModelSelect: (modelId: string) => void;
}) => {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  
  useEffect(() => {
    const fetchModels = async () => {
      try {
        setLoading(true);
        const response = await invoke('list_downloaded_models', {
          provider_type: providerType
        });
        
        if (response.success && response.data) {
          setModels(response.data);
        }
      } catch (error) {
        console.error('Failed to fetch models:', error);
      } finally {
        setLoading(false);
      }
    };
    
    fetchModels();
  }, [providerType]);
  
  return (
    <div className="model-selector">
      <label>Select Model:</label>
      {loading ? (
        <div className="loading">Loading models...</div>
      ) : (
        <select 
          onChange={(e) => onModelSelect(e.target.value)}
          disabled={models.length === 0}
        >
          <option value="">Select a model</option>
          {models.map((model) => (
            <option key={model.id} value={model.id}>
              {model.name}
            </option>
          ))}
        </select>
      )}
      {models.length === 0 && !loading && (
        <div className="no-models">
          No models available. Please download models in Settings.
        </div>
      )}
    </div>
  );
};
```

## Conclusion

The local LLM integration in MCP Client provides a powerful way to use language models without relying on cloud services. The provider-based approach allows flexibility in choosing the right solution for your needs while maintaining a consistent interface.

For users, this means more control over their data and the ability to work offline. For developers, it offers a well-structured framework for extending functionality with new providers and capabilities.

As the field of local LLMs continues to evolve rapidly, this architecture allows the MCP Client to adapt and incorporate new advancements while maintaining backward compatibility and a consistent user experience.

## Resources

- [Ollama Documentation](https://github.com/ollama/ollama/blob/main/README.md)
- [LocalAI Documentation](https://github.com/go-skynet/LocalAI/blob/master/README.md)
- [llama.cpp Documentation](https://github.com/ggerganov/llama.cpp/blob/master/README.md)
- [GGUF Format Specification](https://github.com/ggerganov/ggml/blob/master/docs/gguf.md)
- [MCP Client Offline Mode Documentation](./offline_capabilities.md)
