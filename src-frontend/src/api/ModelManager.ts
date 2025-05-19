// src-frontend/src/api/ModelManager.ts
// TypeScript API wrapper for the model management system

import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

// Type definitions
export interface Provider {
  provider_type: string;
  name: string;
  description: string;
  version: string;
  default_endpoint: string;
  supports_text_generation: boolean;
  supports_chat: boolean;
  supports_embeddings: boolean;
  requires_api_key: boolean;
}

export interface AvailabilityResult {
  available: boolean;
  version?: string;
  error?: string;
  response_time_ms?: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  size_bytes: number;
  is_downloaded: boolean;
  provider_metadata: Record<string, any>;
  provider: string;
  supports_text_generation: boolean;
  supports_completion: boolean;
  supports_chat: boolean;
  supports_embeddings: boolean;
  supports_image_generation: boolean;
  quantization?: string;
  parameter_count_b?: number;
  context_length?: number;
  model_family?: string;
  created_at?: string;
  tags: string[];
  license?: string;
}

export interface InProgressStatus {
  percent: number;
  bytes_downloaded?: number;
  total_bytes?: number;
  eta_seconds?: number;
  bytes_per_second?: number;
}

export interface CompletedStatus {
  completed_at?: string;
  duration_seconds?: number;
}

export interface FailedStatus {
  reason: string;
  error_code?: string;
  failed_at?: string;
}

export interface CancelledStatus {
  cancelled_at?: string;
}

export interface DownloadStatus {
  status: string;
  NotStarted?: Record<string, any>;
  InProgress?: InProgressStatus;
  Completed?: CompletedStatus;
  Failed?: FailedStatus;
  Cancelled?: CancelledStatus;
}

export interface CommandResponse<T> {
  success: boolean;
  error?: string;
  data?: T;
}

export interface ProgressEvent {
  modelId: string;
  progress: number;
}

export interface CompleteEvent {
  modelId: string;
}

export interface ErrorEvent {
  modelId: string;
  error: string;
}

// ModelManager API
export const ModelManager = {
  /**
   * Get all available LLM providers
   * @returns {Promise<Provider[]>} List of available providers
   */
  async getProviders(): Promise<Provider[]> {
    const result = await invoke<CommandResponse<Provider[]>>('get_all_providers');
    return result.data || [];
  },
  
  /**
   * Check if providers are available
   * @returns {Promise<Record<string, AvailabilityResult>>} Availability status for each provider
   */
  async checkProviderAvailability(): Promise<Record<string, AvailabilityResult>> {
    const result = await invoke<CommandResponse<Record<string, AvailabilityResult>>>('get_all_provider_availability');
    return result.data || {};
  },
  
  /**
   * Get current active provider
   * @returns {Promise<string>} The active provider type
   */
  async getActiveProvider(): Promise<string> {
    const result = await invoke<CommandResponse<string>>('get_active_provider');
    return result.data || '';
  },
  
  /**
   * Set active provider
   * @param {string} providerType The provider type to set as active
   * @returns {Promise<boolean>} Success result
   */
  async setActiveProvider(providerType: string): Promise<boolean> {
    const result = await invoke<CommandResponse<boolean>>('set_active_provider', { providerType });
    return result.success;
  },
  
  /**
   * List all available models
   * @param {string} [providerType] Optional provider type, uses active if not specified
   * @returns {Promise<ModelInfo[]>} List of available models
   */
  async listAvailableModels(providerType?: string): Promise<ModelInfo[]> {
    const result = await invoke<CommandResponse<ModelInfo[]>>('list_available_models', { providerType });
    return result.data || [];
  },
  
  /**
   * List all downloaded models
   * @param {string} [providerType] Optional provider type, uses active if not specified
   * @returns {Promise<ModelInfo[]>} List of downloaded models
   */
  async listDownloadedModels(providerType?: string): Promise<ModelInfo[]> {
    const result = await invoke<CommandResponse<ModelInfo[]>>('list_downloaded_models', { providerType });
    return result.data || [];
  },
  
  /**
   * Download a model
   * @param {string} modelId The ID of the model to download
   * @param {string} [providerType] Optional provider type, uses active if not specified
   * @returns {Promise<DownloadStatus>} Download status
   */
  async downloadModel(modelId: string, providerType?: string): Promise<DownloadStatus | null> {
    const result = await invoke<CommandResponse<DownloadStatus>>('download_model', { modelId, providerType });
    return result.data || null;
  },
  
  /**
   * Get download status for a model
   * @param {string} modelId The ID of the model
   * @returns {Promise<DownloadStatus>} Download status
   */
  async getDownloadStatus(modelId: string): Promise<DownloadStatus | null> {
    const result = await invoke<CommandResponse<DownloadStatus>>('get_download_status', { modelId });
    return result.data || null;
  },
  
  /**
   * Cancel a model download
   * @param {string} modelId The ID of the model
   * @param {string} [providerType] Optional provider type, uses active if not specified
   * @returns {Promise<boolean>} Success result
   */
  async cancelDownload(modelId: string, providerType?: string): Promise<boolean> {
    const result = await invoke<CommandResponse<boolean>>('cancel_download', { modelId, providerType });
    return result.success;
  },
  
  /**
   * Delete a model
   * @param {string} modelId The ID of the model
   * @param {string} [providerType] Optional provider type, uses active if not specified
   * @returns {Promise<boolean>} Success result
   */
  async deleteModel(modelId: string, providerType?: string): Promise<boolean> {
    const result = await invoke<CommandResponse<boolean>>('delete_model', { modelId, providerType });
    return result.success;
  },
  
  /**
   * Register for model download events
   * @param {Function} onProgress Progress event handler
   * @param {Function} onComplete Completion event handler
   * @param {Function} onError Error event handler
   * @returns {Promise<UnlistenFn[]>} Cleanup functions to unregister events
   */
  async registerModelEvents(
    onProgress: (modelId: string, progress: number) => void,
    onComplete: (modelId: string) => void,
    onError: (modelId: string, error: string) => void
  ): Promise<UnlistenFn[]> {
    const unlistenProgress = await listen<ProgressEvent>('model-download-progress', (event) => {
      onProgress(event.payload.modelId, event.payload.progress);
    });
    
    const unlistenComplete = await listen<CompleteEvent>('model-download-complete', (event) => {
      onComplete(event.payload.modelId);
    });
    
    const unlistenError = await listen<ErrorEvent>('model-download-error', (event) => {
      onError(event.payload.modelId, event.payload.error);
    });
    
    return [unlistenProgress, unlistenComplete, unlistenError];
  },
  
  /**
   * Generate text using a model
   * @param {string} prompt The prompt for text generation
   * @param {number} [maxTokens] Maximum tokens to generate
   * @param {string} [modelId] Optional model ID, uses default if not specified
   * @param {string} [providerType] Optional provider type, uses active if not specified
   * @returns {Promise<string>} Generated text
   */
  async generateText(
    prompt: string,
    maxTokens?: number,
    modelId?: string,
    providerType?: string
  ): Promise<string | null> {
    const result = await invoke<CommandResponse<string>>('generate_text', { 
      prompt,
      maxTokens,
      modelId,
      providerType
    });
    return result.data || null;
  }
};

// Helper hook for React components
export function useModelManager() {
  return ModelManager;
}

export default ModelManager;