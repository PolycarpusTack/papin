// examples/model_management_test.js
// A test script to demonstrate the model management functionality

// This script assumes it's running in the context of the Tauri app
// with access to the invoke function and event listener

// Mock functions for testing outside of Tauri
function mockInvoke(command, args) {
  console.log(`MOCK INVOKE: ${command}`, args || {});
  
  // Return mock data based on the command
  switch (command) {
    case 'get_all_providers':
      return {
        success: true,
        data: [
          { 
            provider_type: "Ollama", 
            name: "Ollama", 
            description: "Local model runner for LLama and other models",
            version: "1.0.0",
            default_endpoint: "http://localhost:11434",
            supports_text_generation: true,
            supports_chat: true,
            supports_embeddings: true,
            requires_api_key: false
          },
          { 
            provider_type: "LocalAI", 
            name: "LocalAI", 
            description: "Self-hosted OpenAI API compatible server",
            version: "1.0.0",
            default_endpoint: "http://localhost:8080",
            supports_text_generation: true,
            supports_chat: true,
            supports_embeddings: true,
            requires_api_key: false
          },
          { 
            provider_type: "LlamaCpp", 
            name: "llama.cpp", 
            description: "Embedded llama.cpp integration for efficient local inference",
            version: "1.0.0",
            default_endpoint: "local://models",
            supports_text_generation: true,
            supports_chat: true,
            supports_embeddings: false,
            requires_api_key: false
          }
        ]
      };
    case 'get_all_models':
      return {
        success: true,
        data: [
          {
            id: "llama-2-7b",
            name: "Llama 2 7B",
            description: "Open-source LLM with 7B parameters",
            architecture: "Llama",
            format: "GGUF",
            parameter_count: 7.0,
            quantization: "Q4_K_M",
            context_length: 4096,
            size_mb: 4000,
            download_url: "https://huggingface.co/TheBloke/Llama-2-7B-GGUF/resolve/main/llama-2-7b.Q4_K_M.gguf",
            source: "Meta",
            license: "Meta License",
            installed: false,
            loaded: false,
            suggested_provider: "LlamaCpp",
            capabilities: {
              text_generation: true,
              embeddings: false,
              vision: false,
              audio: false,
              chat: true,
              function_calling: false,
              streaming: true,
              code_optimized: false,
              multilingual: false
            },
            metadata: {
              family: "Llama"
            }
          },
          {
            id: "mistral-7b",
            name: "Mistral 7B",
            description: "Mistral 7B model with excellent performance",
            architecture: "Mistral",
            format: "GGUF",
            parameter_count: 7.0,
            quantization: "Q4_K_M",
            context_length: 8192,
            size_mb: 5000,
            download_url: "https://huggingface.co/TheBloke/Mistral-7B-v0.1-GGUF/resolve/main/mistral-7b-v0.1.Q4_K_M.gguf",
            source: "Mistral AI",
            license: "Apache 2.0",
            installed: true,
            loaded: false,
            suggested_provider: "LlamaCpp",
            capabilities: {
              text_generation: true,
              embeddings: false,
              vision: false,
              audio: false,
              chat: true,
              function_calling: false,
              streaming: true,
              code_optimized: false,
              multilingual: false
            },
            metadata: {
              family: "Mistral"
            }
          }
        ]
      };
    case 'get_installed_models':
      return {
        success: true,
        data: [
          {
            id: "mistral-7b",
            name: "Mistral 7B",
            description: "Mistral 7B model with excellent performance",
            architecture: "Mistral",
            format: "GGUF",
            parameter_count: 7.0,
            quantization: "Q4_K_M",
            context_length: 8192,
            size_mb: 5000,
            download_url: "https://huggingface.co/TheBloke/Mistral-7B-v0.1-GGUF/resolve/main/mistral-7b-v0.1.Q4_K_M.gguf",
            source: "Mistral AI",
            license: "Apache 2.0",
            installed: true,
            loaded: false,
            suggested_provider: "LlamaCpp",
            capabilities: {
              text_generation: true,
              embeddings: false,
              vision: false,
              audio: false,
              chat: true,
              function_calling: false,
              streaming: true,
              code_optimized: false,
              multilingual: false
            },
            metadata: {
              family: "Mistral"
            }
          }
        ]
      };
    case 'download_model':
      return {
        success: true,
        data: {
          model_id: args.model_id,
          progress_percent: 0,
          bytes_downloaded: 0,
          total_bytes: 4000000000,
          speed_bps: 5000000,
          eta_seconds: 800,
          completed: false,
          start_time: new Date().toISOString(),
          last_update: new Date().toISOString()
        }
      };
    case 'get_download_status':
      return {
        success: true,
        data: {
          model_id: args.model_id,
          progress_percent: 35,
          bytes_downloaded: 1400000000,
          total_bytes: 4000000000,
          speed_bps: 5200000,
          eta_seconds: 500,
          completed: false,
          start_time: new Date(Date.now() - 300000).toISOString(),
          last_update: new Date().toISOString()
        }
      };
    case 'get_disk_usage':
      return {
        success: true,
        data: {
          used_bytes: 5000000000,
          limit_bytes: 20000000000,
          available_bytes: 15000000000,
          usage_percent: 25
        }
      };
    case 'register_model_registry_events':
      // Simulate registering for events
      console.log("Registering for model registry events");
      // Start a timer to simulate events
      setTimeout(() => {
        console.log("EVENT: Model download progress update");
        if (typeof mockEventCallback === 'function') {
          mockEventCallback({
            type: 'downloadProgress',
            modelId: 'llama-2-7b',
            progress: {
              model_id: 'llama-2-7b',
              progress_percent: 42,
              bytes_downloaded: 1680000000,
              total_bytes: 4000000000,
              speed_bps: 5100000,
              eta_seconds: 450,
              completed: false,
              start_time: new Date(Date.now() - 330000).toISOString(),
              last_update: new Date().toISOString()
            }
          });
        }
      }, 2000);
      
      return { success: true, data: true };
    default:
      return { success: false, error: `Command '${command}' not implemented in mock` };
  }
}

// Mock event callback
let mockEventCallback = null;

// Mock event listener
function mockListen(event, callback) {
  console.log(`MOCK LISTEN: Registered for ${event} events`);
  mockEventCallback = callback;
  return () => {
    console.log(`MOCK LISTEN: Unregistered from ${event} events`);
    mockEventCallback = null;
  };
}

// Use actual Tauri functions if available, otherwise use mocks
const invoke = window?.tauri?.invoke || mockInvoke;
const listen = window?.tauri?.event?.listen || mockListen;

// ----------------------
// Test Runner
// ----------------------

async function runTest() {
  console.log("Starting Model Management Test");
  console.log("==============================");
  
  try {
    // Step 1: Get all providers
    console.log("\n[TEST] Getting all LLM providers");
    const providersResponse = await invoke('get_all_providers');
    if (!providersResponse.success) {
      throw new Error(`Failed to get providers: ${providersResponse.error}`);
    }
    
    const providers = providersResponse.data;
    console.log(`Found ${providers.length} providers:`);
    providers.forEach(p => console.log(`- ${p.name}: ${p.description}`));
    
    // Step 2: Get all models
    console.log("\n[TEST] Getting all models");
    const modelsResponse = await invoke('get_all_models');
    if (!modelsResponse.success) {
      throw new Error(`Failed to get models: ${modelsResponse.error}`);
    }
    
    const models = modelsResponse.data;
    console.log(`Found ${models.length} models:`);
    models.forEach(m => {
      console.log(`- ${m.name} (${m.architecture}, ${m.parameter_count}B, ${m.installed ? 'Installed' : 'Not Installed'})`);
    });
    
    // Step 3: Get installed models
    console.log("\n[TEST] Getting installed models");
    const installedResponse = await invoke('get_installed_models');
    if (!installedResponse.success) {
      throw new Error(`Failed to get installed models: ${installedResponse.error}`);
    }
    
    const installedModels = installedResponse.data;
    console.log(`Found ${installedModels.length} installed models:`);
    installedModels.forEach(m => {
      console.log(`- ${m.name} (${m.architecture}, ${m.parameter_count}B)`);
    });
    
    // Step 4: Get disk usage
    console.log("\n[TEST] Getting disk usage");
    const diskUsageResponse = await invoke('get_disk_usage');
    if (!diskUsageResponse.success) {
      throw new Error(`Failed to get disk usage: ${diskUsageResponse.error}`);
    }
    
    const diskUsage = diskUsageResponse.data;
    console.log(`Disk usage: ${(diskUsage.used_bytes / 1024 / 1024 / 1024).toFixed(2)} GB of ${(diskUsage.limit_bytes / 1024 / 1024 / 1024).toFixed(2)} GB (${diskUsage.usage_percent.toFixed(1)}%)`);
    
    // Step 5: Download a model
    const modelToDownload = models.find(m => !m.installed);
    if (modelToDownload) {
      console.log(`\n[TEST] Downloading model: ${modelToDownload.name}`);
      const downloadResponse = await invoke('download_model', {
        model_id: modelToDownload.id,
        url: modelToDownload.download_url,
        provider: modelToDownload.suggested_provider || "LlamaCpp"
      });
      
      if (!downloadResponse.success) {
        throw new Error(`Failed to start download: ${downloadResponse.error}`);
      }
      
      console.log(`Download started for ${modelToDownload.name}`);
      console.log(`Initial progress: ${downloadResponse.data.progress_percent.toFixed(1)}%`);
      
      // Step 6: Check download status
      console.log("\n[TEST] Checking download status");
      const statusResponse = await invoke('get_download_status', {
        model_id: modelToDownload.id
      });
      
      if (!statusResponse.success) {
        throw new Error(`Failed to get download status: ${statusResponse.error}`);
      }
      
      const status = statusResponse.data;
      console.log(`Download status for ${modelToDownload.name}:`);
      console.log(`- Progress: ${status.progress_percent.toFixed(1)}%`);
      console.log(`- Downloaded: ${(status.bytes_downloaded / 1024 / 1024).toFixed(2)} MB of ${(status.total_bytes / 1024 / 1024).toFixed(2)} MB`);
      console.log(`- Speed: ${(status.speed_bps / 1024 / 1024).toFixed(2)} MB/s`);
      console.log(`- ETA: ${Math.floor(status.eta_seconds / 60)} minutes ${status.eta_seconds % 60} seconds`);
      
      // Step 7: Listen for events
      console.log("\n[TEST] Registering for model registry events");
      const eventsResponse = await invoke('register_model_registry_events');
      if (!eventsResponse.success) {
        throw new Error(`Failed to register for events: ${eventsResponse.error}`);
      }
      
      console.log("Event listener registered");
      
      // Set up event listener
      const unsubscribe = await listen('model-registry-event', (event) => {
        const payload = event.payload;
        console.log(`Received event: ${payload.type} for model ${payload.modelId}`);
        
        if (payload.type === 'downloadProgress' && payload.progress) {
          const progress = payload.progress;
          console.log(`Progress update: ${progress.progress_percent.toFixed(1)}% at ${(progress.speed_bps / 1024 / 1024).toFixed(2)} MB/s`);
        } else if (payload.type === 'downloadCompleted') {
          console.log(`Download completed for model ${payload.modelId}`);
        } else if (payload.type === 'downloadFailed') {
          console.log(`Download failed for model ${payload.modelId}: ${payload.error}`);
        }
      });
      
      console.log("Waiting for events (will continue in 3 seconds)...");
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      // Unsubscribe from events
      unsubscribe();
      console.log("Unsubscribed from events");
    } else {
      console.log("\n[TEST] No models available for download test");
    }
    
    // Test complete
    console.log("\n[TEST] Model Management Test completed successfully");
    
  } catch (error) {
    console.error("Test failed:", error);
  }
}

// ----------------------
// Typescript API Example
// ----------------------

// This section shows how the TypeScript API would be used
// in a production environment instead of raw invoke calls

/*
import { 
  getAllModels, 
  getInstalledModels, 
  downloadModel,
  getDownloadStatus,
  getDiskUsage,
  registerModelRegistryEvents
} from '../src-frontend/src/api/modelRegistry';

async function typescriptExample() {
  try {
    // Get all models
    const models = await getAllModels();
    console.log(`Found ${models.length} models`);
    
    // Get installed models
    const installedModels = await getInstalledModels();
    console.log(`${installedModels.length} models are installed`);
    
    // Check disk usage
    const diskUsage = await getDiskUsage();
    console.log(`Using ${(diskUsage.used_bytes / 1073741824).toFixed(2)} GB of ${(diskUsage.limit_bytes / 1073741824).toFixed(2)} GB`);
    
    // Find a model to download
    const modelToDownload = models.find(m => !m.installed);
    if (modelToDownload && modelToDownload.download_url) {
      // Start download
      await downloadModel(
        modelToDownload.id,
        modelToDownload.download_url,
        modelToDownload.suggested_provider || "LlamaCpp"
      );
      
      // Check status
      const status = await getDownloadStatus(modelToDownload.id);
      console.log(`Download progress: ${status.progress_percent.toFixed(1)}%`);
      
      // Register for events
      const unsubscribe = await registerModelRegistryEvents((event) => {
        if (event.type === 'downloadProgress') {
          console.log(`Download progress: ${event.progress?.progress_percent.toFixed(1)}%`);
        } else if (event.type === 'downloadCompleted') {
          console.log('Download completed!');
        }
      });
      
      // Later, unsubscribe when done
      setTimeout(() => {
        unsubscribe();
      }, 10000);
    }
  } catch (error) {
    console.error("Error:", error);
  }
}
*/

// Run the test when this script is executed
runTest();