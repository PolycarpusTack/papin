// Try to import Tauri API, but provide a fallback for dev mode
let invoke: (cmd: string, args?: any) => Promise<any>;
let listen: (event: string, callback: (event: any) => void) => Promise<() => void>;

try {
  const tauriApi = require('@tauri-apps/api/tauri');
  const eventApi = require('@tauri-apps/api/event');
  invoke = tauriApi.invoke;
  listen = eventApi.listen;
} catch (e) {
  // Mock for development mode without Tauri
  console.log('Running in development mode without Tauri');
  
  // Mock data for development
  const mockModels: Model[] = [
    {
      id: 'mistral-7b-instruct',
      name: 'Mistral 7B Instruct',
      description: 'A powerful and efficient 7B parameter instruction-tuned language model',
      architecture: 'Mistral',
      format: 'GGUF',
      parameterCount: 7000000000,
      contextLength: 8192,
      quantization: 'Q4_K_M',
      sizeMb: 4200,
      provider: 'ollama',
      isInstalled: true,
      isLoaded: true,
      license: 'Apache 2.0',
      tags: ['instruction-tuned', 'chat', 'efficient'],
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: false,
        imageGeneration: false
      },
      metadata: {
        modelFamily: 'Mistral AI',
        contextWindow: '8k',
        performance: 'high'
      }
    },
    {
      id: 'llama2-7b',
      name: 'Llama 2 7B',
      description: 'Meta\'s Llama 2 7B parameter base model',
      architecture: 'Llama',
      format: 'GGUF',
      parameterCount: 7000000000,
      contextLength: 4096,
      quantization: 'Q4_0',
      sizeMb: 3900,
      provider: 'ollama',
      isInstalled: false,
      isLoaded: false,
      license: 'Meta Llama 2 License',
      tags: ['base-model', 'general-purpose'],
      capabilities: {
        textGeneration: true,
        chat: false,
        embeddings: false,
        imageGeneration: false
      },
      metadata: {
        modelFamily: 'Meta AI',
        contextWindow: '4k',
        performance: 'medium'
      }
    },
    {
      id: 'phi-2',
      name: 'Phi-2',
      description: 'Microsoft\'s compact and efficient 2.7B parameter model',
      architecture: 'Phi',
      format: 'GGUF',
      parameterCount: 2700000000,
      contextLength: 2048,
      quantization: 'Q5_K_M',
      sizeMb: 1800,
      provider: 'ollama',
      isInstalled: true,
      isLoaded: false,
      license: 'MIT',
      tags: ['small', 'efficient', 'instruction-tuned'],
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: false,
        imageGeneration: false
      },
      metadata: {
        modelFamily: 'Microsoft Research',
        contextWindow: '2k',
        performance: 'fast'
      }
    }
  ];
  
  const mockProviders: Provider[] = [
    {
      id: 'ollama',
      name: 'Ollama',
      description: 'Run open-source LLMs locally',
      version: '0.1.0',
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: true,
        imageGeneration: false
      },
      status: 'active',
      requiresApiKey: false,
      supportsLocalModels: true
    },
    {
      id: 'localai',
      name: 'LocalAI',
      description: 'Self-hosted OpenAI-compatible API',
      version: '0.2.0',
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: true,
        imageGeneration: true
      },
      status: 'available',
      requiresApiKey: false,
      supportsLocalModels: true
    }
  ];
  
  const mockStorageInfo: StorageInfo = {
    totalBytes: 100000000000,
    usedBytes: 35000000000,
    availableBytes: 65000000000,
    maxAllowedBytes: 100000000000,
    percentUsed: 35,
    modelCount: 2
  };
  
  // Mock event listeners and callbacks
  const mockEventListeners: Record<string, Function[]> = {};
  
  // Mock invoke function
  invoke = async (cmd: string, args?: any) => {
    console.log(`Mock invoke: ${cmd}`, args);
    
    switch (cmd) {
      case 'get_model_providers':
        return mockProviders;
      case 'get_active_provider':
        return mockProviders[0];
      case 'get_all_models':
        return mockModels;
      case 'get_installed_models':
        return mockModels.filter(m => m.isInstalled);
      case 'get_model_storage_info':
        return mockStorageInfo;
      case 'get_provider_stats':
        return {
          modelCount: 5,
          installedModelCount: 2,
          totalSizeMb: 6000,
          supportsQuantization: true,
          recommendedModels: ['mistral-7b-instruct', 'phi-2']
        };
      case 'set_active_provider':
        return true;
      case 'download_model':
        // Trigger download events after a short delay
        setTimeout(() => {
          // Progress updates
          for (let i = 0; i <= 10; i++) {
            setTimeout(() => {
              if (i < 10) {
                const listeners = mockEventListeners['model-download-progress'] || [];
                listeners.forEach(cb => cb({
                  payload: { modelId: args.modelId, progress: i / 10 }
                }));
              } else {
                // Complete
                const listeners = mockEventListeners['model-download-complete'] || [];
                listeners.forEach(cb => cb({
                  payload: { modelId: args.modelId }
                }));
                
                // Update mock data
                const modelIndex = mockModels.findIndex(m => m.id === args.modelId);
                if (modelIndex >= 0) {
                  mockModels[modelIndex].isInstalled = true;
                }
              }
            }, i * 500);
          }
        }, 100);
        return true;
      case 'cancel_download':
        setTimeout(() => {
          const listeners = mockEventListeners['model-download-complete'] || [];
          listeners.forEach(cb => cb({
            payload: { modelId: args.modelId }
          }));
        }, 100);
        return true;
      case 'delete_model':
        // Update mock data
        const modelIndex = mockModels.findIndex(m => m.id === args.modelId);
        if (modelIndex >= 0) {
          mockModels[modelIndex].isInstalled = false;
        }
        return true;
      case 'cleanup_unused_models':
        return 1500000000; // Return bytes freed
      default:
        return null;
    }
  };
  
  // Mock listen function
  listen = async (event: string, callback: (event: any) => void) => {
    console.log(`Mock listen: ${event}`);
    if (!mockEventListeners[event]) {
      mockEventListeners[event] = [];
    }
    mockEventListeners[event].push(callback);
    
    // Return unlistener function
    return () => {
      mockEventListeners[event] = mockEventListeners[event].filter(cb => cb !== callback);
    };
  };
}

// Provider information
export interface Provider {
  id: string;
  name: string;
  description: string;
  version?: string;
  capabilities: {
    textGeneration: boolean;
    chat: boolean;
    embeddings: boolean;
    imageGeneration: boolean;
  };
  status: 'active' | 'available' | 'unavailable';
  requiresApiKey: boolean;
  supportsLocalModels: boolean;
  logoUrl?: string;
}

// Model format types
export type ModelFormat = 'GGUF' | 'GGML' | 'ONNX' | 'PyTorch' | 'TensorFlow' | 'Safetensors' | 'Other';

// Model architecture types
export type ModelArchitecture = 
  | 'Llama' 
  | 'Mistral' 
  | 'Falcon' 
  | 'GPT-J' 
  | 'MPT' 
  | 'Phi' 
  | 'Pythia' 
  | 'Cerebras'
  | 'Claude'
  | 'GPT'
  | 'PaLM'
  | 'Gemma'
  | 'BERT'
  | 'Other';

// Quantization types
export type QuantizationType = 
  | 'None' 
  | 'Q4_K_M' 
  | 'Q4_0' 
  | 'Q4_1'
  | 'Q5_K_M' 
  | 'Q5_0' 
  | 'Q5_1'
  | 'Q8_0'
  | 'Q8_1' 
  | 'Int8' 
  | 'Int4' 
  | 'Other';

// Model information
export interface Model {
  id: string;
  name: string;
  description: string;
  family?: string;
  architecture: ModelArchitecture;
  format: ModelFormat;
  parameterCount: number;
  contextLength: number;
  quantization: QuantizationType;
  sizeMb: number;
  provider: string;
  isInstalled: boolean;
  isLoaded: boolean;
  downloadUrl?: string;
  license?: string;
  createdAt?: string;
  lastUsed?: string;
  tags: string[];
  metrics?: {
    inferenceSpeedTokensPerSecond?: number;
    memoryUsageMb?: number;
    loadTimeMs?: number;
  };
  capabilities: {
    textGeneration: boolean;
    chat: boolean;
    embeddings: boolean;
    imageGeneration: boolean;
  };
  metadata: Record<string, string>;
}

// Storage information
export interface StorageInfo {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  maxAllowedBytes: number;
  percentUsed: number;
  modelCount: number;
}

// Download status
export interface DownloadStatus {
  modelId: string;
  progress: number;
  bytesDownloaded: number;
  totalBytes: number;
  speedBps: number;
  etaSeconds: number;
  isComplete: boolean;
  isError: boolean;
  errorMessage?: string;
  startTime: string;
  lastUpdateTime: string;
}

// Provider stats
export interface ProviderStats {
  modelCount: number;
  installedModelCount: number;
  totalSizeMb: number;
  supportsQuantization: boolean;
  recommendedModels: string[];
}

// Model search/filter criteria
export interface ModelFilter {
  providerId?: string;
  architecture?: ModelArchitecture[];
  format?: ModelFormat[];
  quantization?: QuantizationType[];
  minContextLength?: number;
  maxSizeMb?: number;
  isInstalled?: boolean;
  tags?: string[];
  capabilities?: {
    textGeneration?: boolean;
    chat?: boolean;
    embeddings?: boolean;
    imageGeneration?: boolean;
  };
  searchQuery?: string;
}

// Model sort options
export type ModelSortOption = 
  | 'name' 
  | 'size' 
  | 'parameterCount' 
  | 'contextLength'
  | 'lastUsed'
  | 'inferenceSpeed';

// Batch operation result
export interface BatchResult {
  successCount: number;
  failureCount: number;
  errors: Record<string, string>;
}

// Enhanced Model Manager API
export class EnhancedModelManager {
  private static instance: EnhancedModelManager;
  private eventListeners: Record<string, UnlistenFn[]> = {};

  private constructor() {
    // Private constructor to enforce singleton
  }

  // Get the singleton instance
  public static getInstance(): EnhancedModelManager {
    if (!EnhancedModelManager.instance) {
      EnhancedModelManager.instance = new EnhancedModelManager();
    }
    return EnhancedModelManager.instance;
  }

  // ==================== Provider Methods ====================

  /**
   * Get all available providers
   */
  async getProviders(): Promise<Provider[]> {
    try {
      const result = await invoke<Provider[]>('get_model_providers');
      return result || [];
    } catch (error) {
      console.error('Failed to get providers:', error);
      return [];
    }
  }

  /**
   * Get active provider
   */
  async getActiveProvider(): Promise<Provider | null> {
    try {
      const result = await invoke<Provider>('get_active_provider');
      return result;
    } catch (error) {
      console.error('Failed to get active provider:', error);
      return null;
    }
  }

  /**
   * Set active provider
   */
  async setActiveProvider(providerId: string): Promise<boolean> {
    try {
      await invoke<void>('set_active_provider', { providerId });
      return true;
    } catch (error) {
      console.error('Failed to set active provider:', error);
      return false;
    }
  }

  /**
   * Get provider statistics
   */
  async getProviderStats(providerId: string): Promise<ProviderStats | null> {
    try {
      const result = await invoke<ProviderStats>('get_provider_stats', { providerId });
      return result;
    } catch (error) {
      console.error(`Failed to get provider stats for ${providerId}:`, error);
      return null;
    }
  }

  /**
   * Check if a provider is available
   */
  async checkProviderAvailability(providerId: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>('check_provider_availability', { providerId });
      return result;
    } catch (error) {
      console.error(`Failed to check availability for ${providerId}:`, error);
      return false;
    }
  }

  // ==================== Model Methods ====================

  /**
   * Get all models
   */
  async getAllModels(): Promise<Model[]> {
    try {
      const result = await invoke<Model[]>('get_all_models');
      return result || [];
    } catch (error) {
      console.error('Failed to get all models:', error);
      return [];
    }
  }

  /**
   * Get installed models
   */
  async getInstalledModels(): Promise<Model[]> {
    try {
      const result = await invoke<Model[]>('get_installed_models');
      return result || [];
    } catch (error) {
      console.error('Failed to get installed models:', error);
      return [];
    }
  }

  /**
   * Get models from a specific provider
   */
  async getModelsByProvider(providerId: string): Promise<Model[]> {
    try {
      const result = await invoke<Model[]>('get_models_by_provider', { providerId });
      return result || [];
    } catch (error) {
      console.error(`Failed to get models for provider ${providerId}:`, error);
      return [];
    }
  }

  /**
   * Search/filter models
   */
  async searchModels(filter: ModelFilter, sortBy?: ModelSortOption, limit?: number): Promise<Model[]> {
    try {
      const result = await invoke<Model[]>('search_models', { filter, sortBy, limit });
      return result || [];
    } catch (error) {
      console.error('Failed to search models:', error);
      return [];
    }
  }

  /**
   * Get model by ID
   */
  async getModel(modelId: string): Promise<Model | null> {
    try {
      const result = await invoke<Model>('get_model', { modelId });
      return result;
    } catch (error) {
      console.error(`Failed to get model ${modelId}:`, error);
      return null;
    }
  }

  /**
   * Download a model
   */
  async downloadModel(modelId: string, providerId?: string): Promise<boolean> {
    try {
      await invoke<void>('download_model', { modelId, providerId });
      return true;
    } catch (error) {
      console.error(`Failed to download model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Cancel a download
   */
  async cancelDownload(modelId: string): Promise<boolean> {
    try {
      await invoke<void>('cancel_download', { modelId });
      return true;
    } catch (error) {
      console.error(`Failed to cancel download for model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Get download status
   */
  async getDownloadStatus(modelId: string): Promise<DownloadStatus | null> {
    try {
      const result = await invoke<DownloadStatus>('get_download_status', { modelId });
      return result;
    } catch (error) {
      console.error(`Failed to get download status for model ${modelId}:`, error);
      return null;
    }
  }

  /**
   * Get all active downloads
   */
  async getAllDownloads(): Promise<Record<string, DownloadStatus>> {
    try {
      const result = await invoke<Record<string, DownloadStatus>>('get_all_downloads');
      return result || {};
    } catch (error) {
      console.error('Failed to get all downloads:', error);
      return {};
    }
  }

  /**
   * Delete a model
   */
  async deleteModel(modelId: string): Promise<boolean> {
    try {
      await invoke<void>('delete_model', { modelId });
      return true;
    } catch (error) {
      console.error(`Failed to delete model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Load a model into memory
   */
  async loadModel(modelId: string): Promise<boolean> {
    try {
      await invoke<void>('load_model', { modelId });
      return true;
    } catch (error) {
      console.error(`Failed to load model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Unload a model from memory
   */
  async unloadModel(modelId: string): Promise<boolean> {
    try {
      await invoke<void>('unload_model', { modelId });
      return true;
    } catch (error) {
      console.error(`Failed to unload model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Update model metadata
   */
  async updateModelMetadata(modelId: string, metadata: Record<string, string>): Promise<boolean> {
    try {
      await invoke<void>('update_model_metadata', { modelId, metadata });
      return true;
    } catch (error) {
      console.error(`Failed to update metadata for model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Import model from path
   */
  async importModel(
    filePath: string, 
    modelId: string, 
    metadata?: Record<string, string>
  ): Promise<Model | null> {
    try {
      const result = await invoke<Model>('import_model', { filePath, modelId, metadata });
      return result;
    } catch (error) {
      console.error(`Failed to import model from ${filePath}:`, error);
      return null;
    }
  }

  /**
   * Export model to path
   */
  async exportModel(modelId: string, destinationPath: string): Promise<boolean> {
    try {
      await invoke<void>('export_model', { modelId, destinationPath });
      return true;
    } catch (error) {
      console.error(`Failed to export model ${modelId}:`, error);
      return false;
    }
  }

  /**
   * Batch delete models
   */
  async batchDeleteModels(modelIds: string[]): Promise<BatchResult> {
    try {
      const result = await invoke<BatchResult>('batch_delete_models', { modelIds });
      return result;
    } catch (error) {
      console.error('Failed to batch delete models:', error);
      return {
        successCount: 0,
        failureCount: modelIds.length,
        errors: modelIds.reduce((acc, id) => ({ ...acc, [id]: String(error) }), {})
      };
    }
  }

  // ==================== Storage Methods ====================

  /**
   * Get storage information
   */
  async getStorageInfo(): Promise<StorageInfo | null> {
    try {
      const result = await invoke<StorageInfo>('get_model_storage_info');
      return result;
    } catch (error) {
      console.error('Failed to get storage info:', error);
      return null;
    }
  }

  /**
   * Set disk space limit for models
   */
  async setDiskSpaceLimit(limitBytes: number): Promise<boolean> {
    try {
      await invoke<void>('set_disk_space_limit', { limitBytes });
      return true;
    } catch (error) {
      console.error('Failed to set disk space limit:', error);
      return false;
    }
  }

  /**
   * Clean up unused model files to free space
   */
  async cleanupUnusedModels(olderThanDays?: number): Promise<number> {
    try {
      const bytesFreed = await invoke<number>('cleanup_unused_models', { olderThanDays });
      return bytesFreed;
    } catch (error) {
      console.error('Failed to clean up unused models:', error);
      return 0;
    }
  }

  // ==================== Event Listeners ====================

  /**
   * Register for model download progress events
   */
  async subscribeToDownloadEvents(
    onProgress: (modelId: string, progress: number) => void,
    onComplete: (modelId: string) => void,
    onError: (modelId: string, error: string) => void
  ): Promise<() => void> {
    try {
      // Register with the backend to receive events
      await invoke<void>('register_for_download_events');
      
      // Listen for the events
      const unlistenProgress = await listen<{ modelId: string; progress: number }>(
        'model-download-progress', 
        (event) => onProgress(event.payload.modelId, event.payload.progress)
      );
      
      const unlistenComplete = await listen<{ modelId: string }>(
        'model-download-complete', 
        (event) => onComplete(event.payload.modelId)
      );
      
      const unlistenError = await listen<{ modelId: string; error: string }>(
        'model-download-error', 
        (event) => onError(event.payload.modelId, event.payload.error)
      );
      
      // Store the unsubscribe functions
      this.eventListeners['download'] = [unlistenProgress, unlistenComplete, unlistenError];
      
      // Return an unsubscribe function
      return () => {
        unlistenProgress();
        unlistenComplete();
        unlistenError();
        delete this.eventListeners['download'];
        // Tell the backend we're no longer listening
        invoke<void>('unregister_from_download_events').catch(console.error);
      };
    } catch (error) {
      console.error('Failed to subscribe to download events:', error);
      return () => {}; // Empty function as fallback
    }
  }

  /**
   * Register for model registry events (added, removed, updated, etc.)
   */
  async subscribeToModelEvents(
    callback: (eventType: string, modelId: string, data?: any) => void
  ): Promise<() => void> {
    try {
      // Register with the backend
      await invoke<void>('register_for_model_events');
      
      // Listen for events
      const unlisten = await listen<{ type: string; modelId: string; data?: any }>(
        'model-registry-event',
        (event) => callback(event.payload.type, event.payload.modelId, event.payload.data)
      );
      
      // Store the unsubscribe function
      this.eventListeners['model'] = [unlisten];
      
      // Return an unsubscribe function
      return () => {
        unlisten();
        delete this.eventListeners['model'];
        // Tell the backend we're no longer listening
        invoke<void>('unregister_from_model_events').catch(console.error);
      };
    } catch (error) {
      console.error('Failed to subscribe to model events:', error);
      return () => {}; // Empty function as fallback
    }
  }

  /**
   * Clean up all event listeners
   */
  cleanup(): void {
    Object.values(this.eventListeners).forEach(listeners => {
      listeners.forEach(unlistenFn => unlistenFn());
    });
    this.eventListeners = {};
  }
}

// Convenience function to get the instance
export const modelManager = EnhancedModelManager.getInstance();

export default modelManager;