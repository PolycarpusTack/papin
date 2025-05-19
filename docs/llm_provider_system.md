# LLM Provider System

The LLM Provider System in Papin offers a flexible and extensible architecture for integrating various local Large Language Model (LLM) providers. This document explains the design, components, and usage of this system.

## Architecture Overview

The system is designed around a provider-based architecture with several key components:

```
┌─────────────────────────┐
│    User Interface       │
└───────────┬─────────────┘
            │
┌───────────▼─────────────┐
│      LLM Manager        │
└───────────┬─────────────┘
            │
┌───────────▼─────────────┐
│    Provider Factory     │
└───────────┬─────────────┘
            │
┌───────────▼─────────────┐
│    Provider Interface   │
└───────────┬─────────────┘
            │
     ┌──────┴─────────────────┐
     │                        │
┌────▼───┐  ┌─────────┐  ┌────▼───┐  ┌─────────┐
│ Ollama │  │ LocalAI │  │llama.cpp│  │ Custom  │
└────────┘  └─────────┘  └─────────┘  └─────────┘
```

### Core Components

1. **Provider Interface** (`src/offline/llm/provider.rs`): Defines the trait and data structures that all providers must implement.

2. **Provider Implementations** (`src/offline/llm/providers/`): Contains concrete implementations for different LLM backends.
   - `ollama.rs`: Implementation for the Ollama provider
   - `localai.rs`: Implementation for the LocalAI provider
   - Additional providers can be added in this directory

3. **Provider Factory** (`src/offline/llm/factory.rs`): Manages provider creation, registration, and discovery.

4. **LLM Manager** (`src/offline/llm/mod.rs`): High-level interface that applications use to interact with the LLM system.

5. **Discovery System** (`src/offline/llm/discovery.rs`): Detects installed LLM backends on the user's system.

6. **Migration System** (`src/offline/llm/migration.rs`): Handles migration from older systems to the new provider-based architecture.

7. **Platform Optimizations** (`src/offline/llm/platform.rs`): Provides platform-specific optimizations for different hardware.

## Provider Interface

The Provider interface (`Provider` trait) defines the contract that all LLM providers must implement. Key methods include:

- **Model Management**: List, download, and delete models
- **Text Generation**: Generate text from prompts, with both synchronous and streaming modes
- **Model Loading**: Load and unload models from memory
- **Status Checking**: Check if a provider and models are available

Each provider must implement this interface, adapting the API specifics of their respective LLM backend to conform to this common interface.

## Providers

### Ollama Provider

The Ollama provider integrates with [Ollama](https://ollama.ai/), a user-friendly tool for running various LLMs locally.

**Features**:
- Model listing and management
- Text generation with streaming support
- Automatic model downloading
- GPU acceleration support
- Platform-specific optimizations

**Configuration Options**:
- API endpoint URL
- Timeout settings
- SSL verification options
- Custom headers

### LocalAI Provider

The LocalAI provider integrates with [LocalAI](https://github.com/go-skynet/LocalAI), an API compatible with OpenAI's API but running locally.

**Features**:
- OpenAI-compatible API
- Support for various model formats (GGUF, GGML)
- Text generation with both completion and chat APIs
- Streaming support
- Authentication options

**Configuration Options**:
- API endpoint URL
- API key for authentication
- Timeout settings
- SSL verification options
- Custom headers

## Provider Factory

The Provider Factory is responsible for:

1. **Provider Registration**: Register known providers with the system
2. **Provider Creation**: Create providers with custom configurations
3. **Provider Discovery**: Detect and automatically select available providers
4. **Default Provider Management**: Set and retrieve the default provider

The factory is implemented as a singleton, accessible via the `get_provider_factory()` function, ensuring a consistent provider registry across the application.

## LLM Manager

The LLM Manager provides a high-level interface for applications to interact with LLMs. It handles:

1. **Provider Selection**: Choose which provider to use
2. **Model Management**: List, download, and select models
3. **Text Generation**: Generate text from prompts
4. **Platform Optimization**: Apply hardware-specific optimizations

Applications should use the LLM Manager rather than interacting directly with providers, as it provides additional features like:

- Automatic provider discovery
- Default model selection
- Platform-specific optimizations
- Error handling and recovery

## Integration with Papin

The LLM Provider System integrates with the Papin application through:

1. **Offline Capabilities**: Enables use of local LLMs when internet connectivity is limited
2. **Tauri Commands**: Bridges between the Rust backend and TypeScript frontend
3. **User Interface**: Settings panel for provider and model management
4. **Performance Optimizations**: Automatic tuning for the user's hardware

## Usage Examples

### Basic Usage

```rust
use crate::offline::llm::{create_llm_manager, GenerationOptions};

async fn generate_text() -> Result<String, Box<dyn std::error::Error>> {
    // Create and initialize the LLM manager
    let manager = create_llm_manager();
    manager.initialize().await?;
    
    // Generate text with default options
    let response = manager.generate_text(
        "llama2:7b", // Model ID
        "Tell me a story about a robot.", // Prompt
        None, // Default options
    ).await?;
    
    Ok(response)
}
```

### Streaming Generation

```rust
use crate::offline::llm::{create_llm_manager, GenerationOptions};

async fn generate_with_streaming() -> Result<(), Box<dyn std::error::Error>> {
    // Create and initialize the LLM manager
    let manager = create_llm_manager();
    manager.initialize().await?;
    
    // Generate text with streaming
    manager.generate_text_streaming(
        "llama2:7b", // Model ID
        "Tell me a story about a robot.", // Prompt
        None, // Default options
        |chunk| {
            println!("Received chunk: {}", chunk);
            true // Continue generation
        }
    ).await?;
    
    Ok(())
}
```

### Custom Provider Configuration

```rust
use crate::offline::llm::{
    LLMConfig, LLMManager, provider::ProviderType,
    providers::ollama::OllamaConfig,
};
use serde_json::json;

fn create_custom_manager() -> LLMManager {
    // Create custom configuration
    let ollama_config = OllamaConfig {
        endpoint: "http://localhost:11434".to_string(),
        timeout_seconds: 60,
        verify_ssl: true,
        headers: Default::default(),
    };
    
    // Create LLM config with custom provider config
    let config = LLMConfig {
        provider_type: ProviderType::Ollama,
        provider_config: json!(ollama_config),
        default_model: Some("llama2:7b".to_string()),
        auto_discover: true,
        monitor_for_models: true,
        enable_optimizations: true,
    };
    
    // Create manager with custom config
    LLMManager::with_config(config)
}
```

## Extending the System

### Adding a New Provider

To add a new LLM provider:

1. **Create Provider Implementation**: Implement the `Provider` trait for your new provider.

2. **Update Provider Factory**: Register your provider in the factory.

3. **Add Provider-Specific Configuration**: Define configuration options for your provider.

4. **Update Discovery System**: Enable auto-detection of your provider.

5. **Update UI**: Add UI components for configuring your provider.

Example of a new provider registration:

```rust
// In factory.rs, add to register_default_providers()
let my_provider_result = MyProvider::new();
if let Ok(provider) = my_provider_result {
    self.register_provider(Arc::new(provider));
}
```

## Performance Considerations

The LLM Provider System incorporates several performance optimizations:

1. **Hardware Detection**: Automatically detects CPU and GPU capabilities.

2. **Adaptive Parameters**: Adjusts batch size, threads, and other parameters based on hardware.

3. **Provider Selection**: Chooses the optimal provider based on available hardware.

4. **Memory Management**: Adjusts context size and other parameters based on available memory.

## Error Handling

The system uses a structured error handling approach:

1. **Provider Errors**: Uses the `ProviderError` enum for provider-specific errors.

2. **Result Type**: All operations return a `Result<T, ProviderError>` for consistent error handling.

3. **Error Recovery**: Implements auto-discovery and fallbacks when primary providers fail.

4. **Error Reporting**: Provides clear, context-specific error messages.

## Future Enhancements

Planned enhancements to the LLM Provider System include:

1. **Additional Providers**: Support for more LLM backends.

2. **Advanced Metrics**: Detailed performance monitoring and diagnostics.

3. **Model Fine-tuning**: Support for fine-tuning local models.

4. **Provider Chaining**: Ability to chain multiple providers together.

5. **Distributed Inference**: Support for distributed inference across multiple devices.

## Conclusion

The LLM Provider System provides a flexible, extensible architecture for integrating local LLMs into the Papin application. By abstracting the provider-specific details behind a common interface, it enables easy switching between different LLM backends while providing a consistent experience for users and developers.