# Model Management Test Guide

This document explains how to use the model management test script to verify the functionality of the LLM model management system in Papin.

## Overview

The model management system allows users to:

1. Browse available models from different providers
2. Download models for offline use
3. Manage downloaded models (update, delete, etc.)
4. Monitor disk usage and set limits
5. Receive real-time updates on model operations

The test script (`examples/model_management_test.js`) demonstrates these capabilities by simulating a complete workflow of discovering, downloading, and managing models.

## Running the Test

### Option 1: In Tauri Dev Mode

1. Start the Tauri development server:
   ```bash
   npm run tauri dev
   ```

2. Open the browser developer console to see the test output

3. Add this to your main page to run the test:
   ```html
   <script src="examples/model_management_test.js"></script>
   ```

### Option 2: Standalone Mode

The test script includes mock implementations of Tauri's functions, so it can also be run standalone:

1. Open the script in a browser:
   ```bash
   cd /mnt/c/Projects/Papin
   npx http-server
   ```

2. Navigate to http://localhost:8080/examples/model_management_test.js in your browser

3. Open the developer console to see the test output

## Expected Output

When run successfully, the test script will produce output similar to:

```
Starting Model Management Test
==============================

[TEST] Getting all LLM providers
Found 3 providers:
- Ollama: Local model runner for LLama and other models
- LocalAI: Self-hosted OpenAI API compatible server
- llama.cpp: Embedded llama.cpp integration for efficient local inference

[TEST] Getting all models
Found 2 models:
- Llama 2 7B (Llama, 7B, Not Installed)
- Mistral 7B (Mistral, 7B, Installed)

[TEST] Getting installed models
Found 1 installed models:
- Mistral 7B (Mistral, 7B)

[TEST] Getting disk usage
Disk usage: 4.66 GB of 18.63 GB (25.0%)

[TEST] Downloading model: Llama 2 7B
Download started for Llama 2 7B
Initial progress: 0.0%

[TEST] Checking download status
Download status for Llama 2 7B:
- Progress: 35.0%
- Downloaded: 1335.14 MB of 3814.70 MB
- Speed: 4.96 MB/s
- ETA: 8 minutes 20 seconds

[TEST] Registering for model registry events
Event listener registered
Waiting for events (will continue in 3 seconds)...
Received event: downloadProgress for model llama-2-7b
Progress update: 42.0% at 4.86 MB/s
Unsubscribed from events

[TEST] Model Management Test completed successfully
```

## Understanding the Test

The test script follows these steps:

1. **List Providers**: Gets all available LLM providers (Ollama, LocalAI, llama.cpp)
   
2. **List Models**: Gets all models available across all providers
   
3. **Check Installed Models**: Gets only the models that are already installed
   
4. **Check Disk Usage**: Gets information about model storage usage
   
5. **Download a Model**: Starts downloading a model that isn't installed yet
   
6. **Check Download Status**: Polls the download status to see progress
   
7. **Register for Events**: Sets up an event listener to receive real-time updates
   
8. **Process Events**: Receives and processes events as they occur
   
9. **Unsubscribe**: Cleans up the event listener when done

## TypeScript API Example

The test script also includes a commented section showing how to use the TypeScript API in a production environment. This demonstrates how the raw invoke calls can be replaced with typed API functions for better developer experience.

## Troubleshooting

If the test fails, check for these common issues:

- **Tauri Not Running**: If testing in Tauri mode, ensure the Tauri development server is running
  
- **Missing Backend Implementation**: Verify that all the required Tauri commands are properly registered
  
- **Network Issues**: If downloading actual models, ensure network connectivity
  
- **Permission Issues**: Make sure the application has permission to write to the model directory

## Extending the Test

To test additional functionality:

1. Add new mock responses to the `mockInvoke` function for any new commands
   
2. Add new test steps to the `runTest` function
   
3. Update the TypeScript example to demonstrate any new API functions