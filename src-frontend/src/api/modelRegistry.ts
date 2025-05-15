// src-frontend/src/api/modelRegistry.ts

import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Model information type
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  architecture: string;
  format: string;
  parameter_count: number;
  quantization: string;
  context_length: number;
  size_mb: number;
  download_url?: string;
  source: string;
  license: string;
  installed: boolean;
  loaded: boolean;
  last_used?: string;
  installed_date?: string;
  suggested_provider?: string;
  capabilities: Record<string, boolean>;
  metadata: Record<string, string>;
}

// Download status type
export interface DownloadStatus {
  model_id: string;
  progress_percent: number;
  bytes_downloaded: number;
  total_bytes: number;
  speed_bps: number;
  eta_seconds: number;
  completed: boolean;
  error?: string;
  start_time: string;
  last_update: string;
}

// Disk usage information
export interface DiskUsageInfo {
  used_bytes: number;
  limit_bytes: number;
  available_bytes: number;
  usage_percent: number;
}

// Model Registry Event types
export type ModelRegistryEventType = 
  | 'added'
  | 'updated'
  | 'removed'
  | 'downloadStarted'
  | 'downloadProgress'
  | 'downloadCompleted'
  | 'downloadFailed'
  | 'loaded'
  | 'unloaded';

export interface ModelRegistryEvent {
  type: ModelRegistryEventType;
  modelId: string;
  // Additional data based on event type
  progress?: DownloadStatus;
  error?: string;
}

// Response type for commands
export interface CommandResponse<T> {
  success: boolean;
  error?: string;
  data?: T;
}

/**
 * Get all models in the registry
 */
export async function getAllModels(): Promise<ModelInfo[]> {
  const response = await invoke<CommandResponse<ModelInfo[]>>('get_all_models');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get models');
  }
  return response.data;
}

/**
 * Get a specific model by ID
 */
export async function getModel(modelId: string): Promise<ModelInfo> {
  const response = await invoke<CommandResponse<ModelInfo>>('get_model', { model_id: modelId });
  if (!response.success || !response.data) {
    throw new Error(response.error || `Failed to get model: ${modelId}`);
  }
  return response.data;
}

/**
 * Get installed models
 */
export async function getInstalledModels(): Promise<ModelInfo[]> {
  const response = await invoke<CommandResponse<ModelInfo[]>>('get_installed_models');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get installed models');
  }
  return response.data;
}

/**
 * Get models compatible with a provider
 */
export async function getCompatibleModels(provider: string): Promise<ModelInfo[]> {
  const response = await invoke<CommandResponse<ModelInfo[]>>('get_compatible_models', { provider });
  if (!response.success || !response.data) {
    throw new Error(response.error || `Failed to get compatible models for provider: ${provider}`);
  }
  return response.data;
}

/**
 * Update model metadata
 */
export async function updateModelMetadata(
  modelId: string, 
  updates: Record<string, string>
): Promise<ModelInfo> {
  const response = await invoke<CommandResponse<ModelInfo>>(
    'update_model_metadata', 
    { model_id: modelId, updates }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || `Failed to update model metadata: ${modelId}`);
  }
  return response.data;
}

/**
 * Download a model
 */
export async function downloadModel(
  modelId: string, 
  url: string, 
  provider: string
): Promise<DownloadStatus> {
  const response = await invoke<CommandResponse<DownloadStatus>>(
    'download_model', 
    { model_id: modelId, url, provider }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || `Failed to start download for model: ${modelId}`);
  }
  return response.data;
}

/**
 * Get download status
 */
export async function getDownloadStatus(modelId: string): Promise<DownloadStatus> {
  const response = await invoke<CommandResponse<DownloadStatus>>(
    'get_download_status', 
    { model_id: modelId }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || `Failed to get download status for model: ${modelId}`);
  }
  return response.data;
}

/**
 * Cancel a download
 */
export async function cancelDownload(modelId: string): Promise<boolean> {
  const response = await invoke<CommandResponse<boolean>>(
    'cancel_download', 
    { model_id: modelId }
  );
  if (!response.success) {
    throw new Error(response.error || `Failed to cancel download for model: ${modelId}`);
  }
  return true;
}

/**
 * Delete a model
 */
export async function deleteModel(modelId: string): Promise<boolean> {
  const response = await invoke<CommandResponse<boolean>>(
    'delete_model', 
    { model_id: modelId }
  );
  if (!response.success) {
    throw new Error(response.error || `Failed to delete model: ${modelId}`);
  }
  return true;
}

/**
 * Import a model from an external path
 */
export async function importModel(sourcePath: string, modelId: string): Promise<ModelInfo> {
  const response = await invoke<CommandResponse<ModelInfo>>(
    'import_model', 
    { source_path: sourcePath, model_id: modelId }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || `Failed to import model from: ${sourcePath}`);
  }
  return response.data;
}

/**
 * Export a model to an external path
 */
export async function exportModel(modelId: string, destinationPath: string): Promise<boolean> {
  const response = await invoke<CommandResponse<boolean>>(
    'export_model', 
    { model_id: modelId, destination_path: destinationPath }
  );
  if (!response.success) {
    throw new Error(response.error || `Failed to export model: ${modelId}`);
  }
  return true;
}

/**
 * Get disk usage information
 */
export async function getDiskUsage(): Promise<DiskUsageInfo> {
  const response = await invoke<CommandResponse<DiskUsageInfo>>('get_disk_usage');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get disk usage information');
  }
  return response.data;
}

/**
 * Set disk space limit
 */
export async function setDiskSpaceLimit(limitBytes: number): Promise<DiskUsageInfo> {
  const response = await invoke<CommandResponse<DiskUsageInfo>>(
    'set_disk_space_limit', 
    { limit_bytes: limitBytes }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to set disk space limit');
  }
  return response.data;
}

/**
 * Register for model registry events
 * Returns an unsubscribe function
 */
export async function registerModelRegistryEvents(
  callback: (event: ModelRegistryEvent) => void
): Promise<() => void> {
  // First, tell the backend to start sending events
  const response = await invoke<CommandResponse<boolean>>('register_model_registry_events');
  if (!response.success) {
    throw new Error(response.error || 'Failed to register for model registry events');
  }
  
  // Now listen for the events
  const unsubscribe = await listen<ModelRegistryEvent>(
    'model-registry-event', 
    (event) => callback(event.payload)
  );
  
  return unsubscribe;
}

/**
 * A React hook to subscribe to model registry events
 */
export function useModelRegistryEvents(
  callback: (event: ModelRegistryEvent) => void,
  dependencies: React.DependencyList = []
): void {
  React.useEffect(() => {
    let unsubscribe: (() => void) | null = null;
    
    // Set up subscription
    const setup = async () => {
      unsubscribe = await registerModelRegistryEvents(callback);
    };
    
    setup().catch(console.error);
    
    // Clean up subscription
    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, dependencies);
}