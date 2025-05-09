# Troubleshooting Guide for Local LLM Integration

This guide provides solutions to common issues you might encounter when using local LLM providers with the MCP Client.

## Provider Connection Issues

### Provider Not Available

**Symptoms**:
- "Provider not available" warning in the UI
- Unable to see or download models
- Red warning icon next to provider name

**Potential Causes**:
1. Provider software is not installed
2. Provider service is not running
3. Incorrect endpoint URL
4. Firewall blocking the connection
5. Provider crashed or is unresponsive

**Solutions**:

#### For Ollama:
1. **Verify Installation**:
   ```bash
   # Check if Ollama is in PATH
   which ollama  # on macOS/Linux
   where ollama  # on Windows
   ```

2. **Check if Ollama is Running**:
   ```bash
   # Check process
   ps aux | grep ollama  # on macOS/Linux
   tasklist | findstr ollama  # on Windows
   
   # Check if the API is responding
   curl http://localhost:11434/api/version
   ```

3. **Start Ollama Server**:
   ```bash
   ollama serve
   ```

4. **Check Logs**:
   - macOS/Linux: `~/.ollama/logs/` or `/var/log/ollama.log`
   - Windows: `%USERPROFILE%\.ollama\logs\`

#### For LocalAI:
1. **Check Docker Container** (if using Docker):
   ```bash
   docker ps | grep localai
   
   # If not running, start it
   docker run -p 8080:8080 localai/localai:latest
   ```

2. **Test API Endpoint**:
   ```bash
   curl http://localhost:8080/version
   ```

3. **Check Logs**:
   ```bash
   # If running as Docker container
   docker logs $(docker ps | grep localai | awk '{print $1}')
   
   # If running locally
   cat localai.log  # or check terminal where LocalAI is running
   ```

#### For llama.cpp:
1. **Verify Server is Running**:
   ```bash
   # Check process
   ps aux | grep llama-server  # on macOS/Linux
   tasklist | findstr llama-server  # on Windows
   
   # Test endpoint
   curl http://localhost:8000/info
   ```

2. **Start Server**:
   ```bash
   ./server -m /path/to/model.gguf -c 2048
   ```

3. **Check Terminal Output** for any errors

### Firewall Issues

If you suspect a firewall issue:

1. **Check Firewall Settings**:
   - Windows: Check Windows Defender Firewall settings
   - macOS: Check System Settings > Network > Firewall
   - Linux: Check `ufw` or `iptables` rules

2. **Add Exceptions** for:
   - Ollama: Port 11434
   - LocalAI: Port 8080
   - llama.cpp: Port 8000 (or your custom port)

3. **Test Local Connection**:
   ```bash
   # Simple connection test
   telnet localhost 11434  # For Ollama
   telnet localhost 8080   # For LocalAI
   telnet localhost 8000   # For llama.cpp
   ```

## Model Issues

### Model Download Failures

**Symptoms**:
- Download starts but fails before completion
- Error message in the download status
- Download stuck at a certain percentage

**Potential Causes**:
1. Insufficient disk space
2. Network interruption
3. Provider service issues
4. Permission problems
5. Corrupted download cache

**Solutions**:

1. **Check Disk Space**:
   ```bash
   # On macOS/Linux
   df -h
   
   # On Windows
   dir /a /-c
   ```

2. **Clear Download Cache**:
   - Ollama: 
     ```bash
     rm -rf ~/.ollama/cache/*  # macOS/Linux
     rd /s /q %USERPROFILE%\.ollama\cache  # Windows
     ```
   - LocalAI: Check the LocalAI models directory

3. **Check Provider Logs** for specific error messages

4. **Try a Smaller Model**: Start with a smaller model to test if the issue is related to file size

5. **Manual Download**:
   - Ollama: Use the CLI `ollama pull modelname`
   - LocalAI: Download model files directly to the models directory
   - llama.cpp: Download GGUF files manually from Hugging Face

### Model Not Found After Download

**Symptoms**:
- Download completed successfully, but model doesn't appear in the downloaded list
- "Model not found" error when trying to use a downloaded model

**Solutions**:

1. **Refresh Model List**:
   - Click the "Refresh" button in the Models tab
   - Restart the MCP Client
   - Restart the provider service

2. **Check Model Location**:
   - Ollama: `~/.ollama/models/` (macOS/Linux) or `%USERPROFILE%\.ollama\models\` (Windows)
   - LocalAI: Check your configured models directory
   - llama.cpp: Check the path used when starting the server

3. **Verify Model Format**:
   - Ensure the model is in a compatible format
   - For llama.cpp, make sure you're using GGUF format (not older GGML)

4. **Check File Permissions**:
   - Ensure the user running MCP Client has read access to the model files

## Performance Issues

### High Memory Usage

**Symptoms**:
- System becomes slow when using local models
- Application crashes during text generation
- Out of memory errors
- System swap usage increases significantly

**Solutions**:

1. **Use Lower Precision Models**:
   - Switch from f16 to Q4_K_M or Q2_K quantization
   - These models use significantly less memory with minimal quality loss

2. **Reduce Context Length**:
   - Set a smaller context window (e.g., 2048 instead of 4096)
   - For llama.cpp server, restart with `-c 2048` option

3. **Choose Smaller Models**:
   - Switch from 13B or 70B parameter models to 7B models
   - Smaller models require much less RAM

4. **Adjust System Configuration**:
   - Increase swap space/page file size
   - Close other memory-intensive applications before using local LLMs
   - For llama.cpp, use `--threads` option to limit the number of threads

5. **Check for Memory Leaks**:
   - Restart the provider service after extended use
   - Monitor memory usage with tools like Task Manager (Windows), Activity Monitor (macOS), or `top`/`htop` (Linux)

### Slow Generation Speed

**Symptoms**:
- Very slow responses when using local models
- Generation takes significantly longer than expected

**Solutions**:

1. **Enable GPU Acceleration** if available:
   - Ollama: Uses GPU automatically if available
   - LocalAI: Check GPU configuration in settings
   - llama.cpp: Compile with GPU support (CUDA, Metal, or ROCm)

2. **Optimize Thread Count**:
   - For llama.cpp, set `--threads` to match your CPU core count
   - Example: `./server -m model.gguf -c 2048 --threads 8`

3. **Monitor System Resource Usage**:
   - Check CPU/GPU usage during generation
   - Ensure other processes aren't consuming resources

4. **Use Batch Processing** for multiple generations:
   - Where supported, batch requests together
   - This can be more efficient than separate requests

5. **Check for Network Bottlenecks**:
   - Ensure you're using localhost connections without any proxies
   - Check if antivirus software is scanning network traffic

### GPU-Specific Issues

**Symptoms**:
- Model runs on CPU instead of GPU
- CUDA/Metal/ROCm errors
- Crashes when trying to use GPU acceleration

**Solutions**:

#### NVIDIA GPUs:
1. **Verify CUDA Installation**:
   ```bash
   nvcc --version
   nvidia-smi
   ```

2. **Check Provider GPU Support**:
   - Ollama: Check log for GPU detection
   - llama.cpp: Compile with `-DLLAMA_CUBLAS=ON`
   - LocalAI: Verify GPU configuration

3. **Memory Issues**:
   - Monitor VRAM usage with `nvidia-smi`
   - Try models that fit in available VRAM
   - Use mixed precision (INT8/INT4) for larger models

#### Apple Silicon:
1. **Verify Metal Support**:
   - llama.cpp: Compile with `-DLLAMA_METAL=ON`
   - Ollama: Uses Metal automatically on macOS

2. **Check Activity Monitor** for GPU usage

3. **Verify Model Compatibility** with Metal

## Offline Mode Issues

### Offline Mode Not Working

**Symptoms**:
- Still using cloud APIs despite being offline
- Error messages when trying to generate text offline

**Solutions**:

1. **Check Offline Mode Settings**:
   - Verify "Enable Offline Mode" is toggled ON
   - Check if "Auto-switch based on connectivity" is enabled

2. **Verify Default Model is Selected**:
   - Go to Settings > Offline and check "Default Model" dropdown
   - Make sure a downloaded model is selected

3. **Test Provider Connection**:
   - Ensure the provider service is running
   - Try connecting to provider API manually

4. **Check Network Status Detection**:
   - The application might be incorrectly detecting network status
   - Try toggling Offline Mode manually instead of relying on auto-switch

5. **Restart Components**:
   - Restart the provider service
   - Restart MCP Client
   - In extreme cases, restart your computer

### Automatic Switching Issues

**Symptoms**:
- Doesn't switch to offline mode when network is disconnected
- Stays in offline mode despite network being available

**Solutions**:

1. **Check Internet Connection Detection**:
   - Some network configurations can cause incorrect detection
   - Try using a manual switch instead of auto-switch

2. **Verify Provider Availability**:
   - Ensure the local provider is available and configured correctly
   - Check if the default model is selected

3. **Check Application Logs**:
   - Look for network status events and mode switching
   - Identify any errors in the switching logic

## Provider-Specific Troubleshooting

### Ollama Issues

**Common Problem: Model Fails to Pull**

**Symptoms**:
- `ollama pull` command fails
- Download starts but doesn't complete
- "Error pulling model" message

**Solutions**:
1. **Check Network Connection**:
   - Ensure stable internet during download
   - Try using a different network if available

2. **Check Disk Space**:
   - Models can be large (2-40GB)
   - Ensure sufficient free space

3. **Remove Partially Downloaded Model**:
   ```bash
   ollama rm modelname
   ```

4. **Update Ollama**:
   - Download and install the latest version

5. **Check Ollama Logs**:
   ```bash
   cat ~/.ollama/logs/ollama.log  # macOS/Linux
   type %USERPROFILE%\.ollama\logs\ollama.log  # Windows
   ```

**Common Problem: Ollama Service Crashes**

**Solutions**:
1. **Restart Ollama**:
   ```bash
   # macOS/Linux
   killall ollama
   ollama serve
   
   # Windows
   taskkill /F /IM ollama.exe
   # Then restart Ollama
   ```

2. **Check System Resources**:
   - Ensure sufficient RAM for models
   - Monitor CPU usage

3. **Check for Corrupt Models**:
   - Try removing and re-downloading problematic models

### LocalAI Issues

**Common Problem: Model Loading Errors**

**Symptoms**:
- Error message when trying to use a model
- Model appears in list but fails when used

**Solutions**:
1. **Check Model Format**:
   - Ensure the model is in a compatible format
   - LocalAI supports GGUF, GGML, ONNX, and other formats

2. **Verify Model Configuration**:
   - Check for proper model configuration in LocalAI
   - Ensure model is in the correct directory

3. **Run with Verbose Logging**:
   ```bash
   LOCAL_AI_DEBUG=1 localai serve
   ```

4. **Check Model Compatibility**:
   - Some models require specific LocalAI versions
   - Check the LocalAI documentation for compatibility

**Common Problem: API Compatibility Issues**

**Symptoms**:
- Error messages about API format
- Requests fail with HTTP errors

**Solutions**:
1. **Check API Format**:
   - LocalAI implements OpenAI-compatible API
   - Ensure requests follow OpenAI API specifications

2. **Verify Endpoint URLs**:
   - Use correct endpoints for different operations:
     - `/v1/chat/completions` for chat
     - `/v1/completions` for completion
     - `/v1/embeddings` for embeddings

3. **Check Headers and Authentication**:
   - If you've configured authentication, ensure proper headers

### llama.cpp Issues

**Common Problem: Compilation Issues**

**Symptoms**:
- Errors when building llama.cpp
- Missing dependencies messages

**Solutions**:
1. **Install Required Dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential cmake
   
   # macOS
   brew install cmake
   
   # Windows
   # Install Visual Studio with C++ workload or MinGW
   ```

2. **Build with Specific Options**:
   ```bash
   # Basic build
   make
   
   # With CUDA support
   make LLAMA_CUBLAS=1
   
   # With Metal support (macOS)
   make LLAMA_METAL=1
   ```

3. **Check for Compatible Compiler**:
   - Ensure you have a modern C++ compiler (supporting C++11 or later)

**Common Problem: Model Format Compatibility**

**Symptoms**:
- "Model format not recognized" error
- Server crashes when loading model

**Solutions**:
1. **Use GGUF Format**:
   - Newer versions require GGUF, not GGML
   - Convert GGML to GGUF if needed with `convert-llama-ggml-to-gguf`

2. **Check llama.cpp Version**:
   - Ensure it's compatible with your model format
   - Update to the latest version for best compatibility

3. **Command Line Options**:
   - Use correct parameters for your model:
     ```bash
     ./server -m model.gguf -c 2048 --host 0.0.0.0 --port 8000
     ```

## UI and Configuration Issues

### Settings Not Saving

**Symptoms**:
- Configuration changes don't persist after restarting the application
- Settings revert to previous values

**Solutions**:
1. **Check Permissions**:
   - Ensure the application has write permissions to its config directory
   - Run as administrator/sudo if necessary

2. **Clear Application Cache**:
   - Close the application
   - Find and rename/delete the application's cache directory
   - Restart the application

3. **Check for Configuration Conflicts**:
   - Another process might be overwriting settings
   - Check for multiple instances of the application

### UI Responsiveness Issues

**Symptoms**:
- UI becomes slow or unresponsive during model operations
- Freezing during model downloads or text generation

**Solutions**:
1. **Enable Streaming Responses**:
   - Where supported, use streaming mode for generation
   - This improves UI responsiveness

2. **Reduce UI Updates**:
   - Lower the frequency of progress updates for long operations
   - This reduces rendering overhead

3. **Check System Resources**:
   - Monitor CPU, RAM, and disk I/O
   - Close other resource-intensive applications

## Collecting Diagnostic Information

When reporting issues, include the following information:

### System Information
```
- OS: [Windows/macOS/Linux + version]
- CPU: [make/model]
- RAM: [amount]
- GPU: [if applicable, make/model]
- Disk Space: [free space available]
```

### MCP Client Information
```
- Version: [x.y.z]
- Offline Mode Enabled: [Yes/No]
- Auto-switch Enabled: [Yes/No]
- Selected Provider: [Ollama/LocalAI/llama.cpp/Custom]
- Provider Version: [version number]
```

### Provider Information
```
- Provider Type: [Ollama/LocalAI/llama.cpp/Custom]
- Provider Version: [version number]
- Endpoint URL: [URL]
- Selected Model: [model name]
- Model Size: [parameters/file size]
- Quantization: [if applicable]
```

### Log Files

**MCP Client Logs**:
- Location: [application-specific]
- Include relevant error messages or warnings

**Provider Logs**:
- Ollama: `~/.ollama/logs/ollama.log`
- LocalAI: Check terminal or Docker logs
- llama.cpp: Check terminal output

### Steps to Reproduce
```
1. Detailed step-by-step instructions to reproduce the issue
2. Include any specific text prompts or operations
3. Mention any relevant system conditions (e.g., low disk space, high CPU load)
```

## Advanced Troubleshooting

### Network Packet Capture

For API communication issues, capture network traffic:

1. **Using Wireshark**:
   - Filter by port: `tcp.port == 11434` (for Ollama)
   - Examine HTTP requests and responses

2. **Using tcpdump**:
   ```bash
   sudo tcpdump -i lo0 -A -s 0 port 11434 > ollama_traffic.txt
   ```

3. **Using Fiddler** (Windows):
   - Configure to capture localhost traffic
   - Examine requests to provider endpoints

### Debugging Provider API

Test provider APIs directly:

#### Ollama API
```bash
# List models
curl http://localhost:11434/api/tags

# Generate text
curl -X POST http://localhost:11434/api/generate -d '{
  "model": "llama2",
  "prompt": "Hello, world!",
  "stream": false
}'
```

#### LocalAI API
```bash
# List models
curl http://localhost:8080/models

# Generate text
curl -X POST http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "prompt": "Hello, world!",
    "max_tokens": 100
  }'
```

#### llama.cpp API
```bash
# Get model info
curl http://localhost:8000/info

# Generate text
curl -X POST http://localhost:8000/completion \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Hello, world!",
    "n_predict": 100
  }'
```

## Further Assistance

If you're still experiencing issues after trying these troubleshooting steps:

1. **Check GitHub Repositories** for known issues:
   - MCP Client: [GitHub Repository]
   - Ollama: [https://github.com/ollama/ollama/issues](https://github.com/ollama/ollama/issues)
   - LocalAI: [https://github.com/go-skynet/LocalAI/issues](https://github.com/go-skynet/LocalAI/issues)
   - llama.cpp: [https://github.com/ggerganov/llama.cpp/issues](https://github.com/ggerganov/llama.cpp/issues)

2. **Join Community Forums**:
   - Ollama Discord
   - LocalAI Discord
   - llama.cpp discussions

3. **Submit Detailed Bug Reports**:
   - Include all diagnostic information listed above
   - Be specific about the issue and reproduction steps
   - Mention any workarounds you've tried

By following this troubleshooting guide, you should be able to resolve most common issues with local LLM integration in the MCP Client. If you discover new issues or solutions, please contribute them to help improve the application.
