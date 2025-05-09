# User Guide for Configuring LLM Providers

This guide provides step-by-step instructions for configuring different LLM providers in the MCP Client. It covers installation, configuration, model management, and usage details for each supported provider.

## Getting Started with Offline Mode

### Enabling Offline Capabilities

1. Open the MCP Client application
2. Navigate to Settings > Offline
3. You'll see the Offline Mode section at the top:
   - Toggle "Enable Offline Mode" to ON to activate local LLM capabilities
   - Toggle "Auto-switch based on connectivity" to ON if you want the application to automatically switch between online and offline modes based on your network status

![Offline Mode Settings](../assets/images/offline_mode_settings.png)

### Selecting a Provider

The MCP Client can automatically detect installed providers on your system. To select a provider:

1. In the Settings > Offline screen, under "Local LLM Provider", select your preferred provider from the dropdown menu
2. The application will verify if the provider is available
3. If available, you'll see a green checkmark indicating it's ready to use
4. If unavailable, you'll see a warning message with information on how to install or properly configure it

## Provider-Specific Setup

### Ollama

[Ollama](https://ollama.ai/) is one of the easiest providers to set up and offers a wide range of models with good performance.

#### Installation

1. **Windows**:
   - Download the installer from [ollama.ai/download](https://ollama.ai/download)
   - Run the installer and follow the on-screen instructions
   - Ollama will start automatically after installation

2. **macOS**:
   - Download the application from [ollama.ai/download](https://ollama.ai/download)
   - Drag the Ollama app to your Applications folder
   - Open Ollama from your Applications folder

3. **Linux**:
   - Run the installation script:
     ```bash
     curl -fsSL https://ollama.ai/install.sh | sh
     ```
   - Start the Ollama service:
     ```bash
     ollama serve
     ```

#### Configuration in MCP Client

1. In the Offline Settings, select "Ollama" from the provider dropdown
2. The default endpoint is `http://localhost:11434` (this should work without changes in most cases)
3. No API key is required for Ollama

#### Working with Models

Ollama makes model management simple:

1. In the "Models" tab, you'll see all available models for Ollama
2. Click the download icon next to any model to start downloading it
3. The progress bar will show the download status
4. Once downloaded, the model will appear in the "Downloaded Models" tab
5. To set a model as the default for offline use, select it in the "Default Model" dropdown

#### Available Models

Ollama provides a wide range of models, including:

- **Llama 2**: Meta's powerful open-source models (7B, 13B, 70B parameters)
- **Mistral**: High-performing 7B parameter model
- **Vicuna**: Fine-tuned LLaMA models
- **Phi-2**: Microsoft's compact but powerful model
- **Orca 2**: Microsoft Research models with strong reasoning capabilities
- **Many more**: Check the Models tab for the complete list

### LocalAI

[LocalAI](https://github.com/go-skynet/LocalAI) provides an OpenAI-compatible API for various open-source models.

#### Installation

1. **Using Docker** (recommended):
   ```bash
   docker run -p 8080:8080 localai/localai:latest
   ```

2. **Manual Installation**:
   - Follow the instructions on the [LocalAI GitHub repository](https://github.com/go-skynet/LocalAI)
   - Basic steps involve:
     ```bash
     git clone https://github.com/go-skynet/LocalAI
     cd LocalAI
     make build
     ./localai serve
     ```

#### Configuration in MCP Client

1. In the Offline Settings, select "LocalAI" from the provider dropdown
2. Set the endpoint URL to where your LocalAI server is running:
   - Default for local installations: `http://localhost:8080`
   - If using Docker with a different port mapping, adjust accordingly
3. No API key is required unless you've configured authentication for your LocalAI server

#### Working with Models

LocalAI requires models to be placed in its models directory:

1. Download models from Hugging Face or other sources
2. Place them in the LocalAI models directory (default: `./models/`)
3. In the MCP Client, refresh the Models list to see the available models
4. Models should automatically appear in the list if they're in the correct directory
5. Select a model as default in the "Default Model" dropdown

#### Available Models

LocalAI supports various model formats:

- GGUF models (formerly GGML)
- ONNX models
- TensorFlow models
- PyTorch models
- And more

### llama.cpp

[llama.cpp](https://github.com/ggerganov/llama.cpp) is a highly optimized C/C++ implementation for running LLaMA models on CPUs.

#### Installation

1. **Clone and build the repository**:
   ```bash
   git clone https://github.com/ggerganov/llama.cpp
   cd llama.cpp
   make
   ```

2. **Start the server with a model**:
   ```bash
   # Download a model (example)
   wget https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.1-GGUF/resolve/main/mistral-7b-instruct-v0.1.Q4_K_M.gguf

   # Start server
   ./server -m mistral-7b-instruct-v0.1.Q4_K_M.gguf -c 2048
   ```

#### Configuration in MCP Client

1. In the Offline Settings, select "LlamaCpp" from the provider dropdown
2. Set the endpoint URL to where your llama.cpp server is running:
   - Default: `http://localhost:8000`
   - If you've specified a different port, adjust accordingly
3. No API key is required

#### Working with Models

llama.cpp requires you to manage models manually:

1. Download GGUF models from sources like Hugging Face
   - Popular model repositories: [TheBloke](https://huggingface.co/TheBloke)
2. Start the llama.cpp server with your chosen model
3. In MCP Client, the model will be automatically detected from the running server
4. You can switch models by stopping the server and restarting it with a different model

#### Model Selection

When choosing models for llama.cpp, consider:

1. **Quantization level**: 
   - Q4_K_M: Good balance of quality and memory usage
   - Q5_K_M: Better quality but higher memory
   - Q2_K: Lower quality but minimal memory
   
2. **Model size**:
   - 7B models: Work on most systems, even with limited RAM
   - 13B models: Require at least 8GB of RAM
   - 70B models: Require 16GB+ RAM or GPU acceleration

3. **Context length**:
   - Models support different context sizes (2k, 4k, 8k, etc.)
   - Larger contexts use more memory but can "remember" more

### Custom Provider

The Custom Provider option allows you to integrate a specialized or self-made LLM service.

#### Configuration

1. In the Offline Settings, select "Custom" from the provider dropdown
2. Enter the endpoint URL for your custom LLM service
3. Configure the API key if your service requires authentication
4. In the Advanced Configuration section, you can set provider-specific parameters

#### Requirements for Custom Providers

Your custom provider should implement a REST API with these minimum endpoints:

- `GET /models`: List available models
- `POST /generate`: Generate text from a prompt
- `GET /info`: Provide provider information

Refer to the Developer Documentation for more details on implementing a custom provider.

## Managing Default Model

To set a model as default for offline use:

1. Go to Settings > Offline
2. In the Local LLM Provider section, locate the "Default Model" dropdown
3. Select the model you want to use as default from your downloaded models
4. Click "Save Configuration" to apply the changes

The default model will be used automatically when:
- You're in offline mode
- You don't specify a particular model in your requests

## Advanced Features

### Automatic Provider Discovery

The MCP Client can automatically detect and configure providers installed on your system:

1. Go to Settings > Offline
2. Click "Refresh Providers" to scan for available providers
3. Newly discovered providers will appear in the provider dropdown
4. The system will attempt to auto-configure discovered providers

### Performance Metrics

You can monitor the performance of your local LLMs:

1. Go to Dashboard > LLM to view performance metrics
2. View metrics such as:
   - Tokens per second
   - Latency measurements
   - Memory usage
   - Success rate

### Offline Metrics Collection

The application can collect anonymous performance metrics to help improve the system:

1. In Settings > Offline, find the "LLM Performance Metrics" section
2. Review the privacy notice and what data is collected
3. Toggle the metrics collection option based on your preference

## Best Practices

### Choosing the Right Provider

Consider these factors when selecting a provider:

- **Ease of use**: Ollama is the simplest option for beginners
- **Performance**: llama.cpp offers the best optimization for limited hardware
- **Integration**: LocalAI provides the best OpenAI API compatibility
- **Model support**: Ollama and LocalAI support the widest range of models

### Optimizing for Your Hardware

To get the best performance:

1. **For systems with limited RAM** (8GB or less):
   - Use llama.cpp with Q4_K_M or Q2_K quantized models
   - Stick to 7B parameter models
   - Limit context size to 2048 tokens

2. **For mid-range systems** (16GB RAM):
   - Any provider works well
   - Can use 13B parameter models
   - Context sizes up to 4096 tokens

3. **For high-end systems** (32GB+ RAM or GPU):
   - Any provider with GPU acceleration
   - Can use 70B parameter models
   - Larger context sizes (8192+ tokens)

### Managing Model Downloads

1. **Disk space**: Models can range from 2GB to 40GB each
2. **Download order**: Start with smaller models first
3. **Organization**: Remove unused models to save space

## Conclusion

By following this guide, you should be able to configure and use any of the supported LLM providers in the MCP Client. Each provider has its strengths and is suited for different use cases and hardware configurations.

For troubleshooting common issues, refer to the Troubleshooting Guide in the documentation.
