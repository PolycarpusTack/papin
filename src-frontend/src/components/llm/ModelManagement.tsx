import React, { useState, useEffect } from 'react';
import { ModelManager, ModelInfo, Provider } from '../../api/ModelManager';

/**
 * ModelManagement Component
 * 
 * Demonstrates the usage of the ModelManager API to interact with LLM models
 * Includes features for listing models, downloading, viewing status, and more
 */
const ModelManagement: React.FC = () => {
  // State for data
  const [providers, setProviders] = useState<Provider[]>([]);
  const [activeProvider, setActiveProvider] = useState<string>('');
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [downloadedModels, setDownloadedModels] = useState<ModelInfo[]>([]);
  const [downloading, setDownloading] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Load providers and models on component mount
  useEffect(() => {
    async function loadData() {
      try {
        setLoading(true);
        setError(null);

        // Get providers
        const providerList = await ModelManager.getProviders();
        setProviders(providerList);
        
        // Get active provider
        const active = await ModelManager.getActiveProvider();
        setActiveProvider(active);
        
        // Get available models
        const availableModels = await ModelManager.listAvailableModels();
        setModels(availableModels);
        
        // Get downloaded models
        const installedModels = await ModelManager.listDownloadedModels();
        setDownloadedModels(installedModels);
        
        // Register for model events
        const unlisteners = await ModelManager.registerModelEvents(
          // Progress handler
          (modelId, progress) => {
            setDownloading(prev => ({
              ...prev,
              [modelId]: progress
            }));
          },
          // Complete handler
          (modelId) => {
            setDownloading(prev => {
              const newState = { ...prev };
              delete newState[modelId];
              return newState;
            });
            
            // Refresh model lists to show downloaded status
            refreshModels();
          },
          // Error handler
          (modelId, errorMsg) => {
            setError(`Download error for model ${modelId}: ${errorMsg}`);
            setDownloading(prev => {
              const newState = { ...prev };
              delete newState[modelId];
              return newState;
            });
          }
        );
        
        // Clean up event listeners when component unmounts
        return () => {
          unlisteners.forEach(fn => fn());
        };
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    }
    
    loadData();
  }, []);

  // Refresh model lists
  const refreshModels = async () => {
    try {
      const availableModels = await ModelManager.listAvailableModels();
      setModels(availableModels);
      
      const installedModels = await ModelManager.listDownloadedModels();
      setDownloadedModels(installedModels);
    } catch (err) {
      setError(String(err));
    }
  };

  // Set active provider
  const handleSetActiveProvider = async (providerType: string) => {
    try {
      await ModelManager.setActiveProvider(providerType);
      setActiveProvider(providerType);
      refreshModels();
    } catch (err) {
      setError(String(err));
    }
  };

  // Download a model
  const handleDownloadModel = async (modelId: string) => {
    try {
      await ModelManager.downloadModel(modelId);
      setDownloading(prev => ({
        ...prev,
        [modelId]: 0
      }));
    } catch (err) {
      setError(String(err));
    }
  };

  // Cancel a download
  const handleCancelDownload = async (modelId: string) => {
    try {
      await ModelManager.cancelDownload(modelId);
      setDownloading(prev => {
        const newState = { ...prev };
        delete newState[modelId];
        return newState;
      });
    } catch (err) {
      setError(String(err));
    }
  };

  // Delete a model
  const handleDeleteModel = async (modelId: string) => {
    try {
      await ModelManager.deleteModel(modelId);
      refreshModels();
    } catch (err) {
      setError(String(err));
    }
  };

  // Format bytes to human-readable size
  const formatBytes = (bytes: number, decimals = 2): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
  };

  if (loading) {
    return <div className="p-4">Loading model information...</div>;
  }

  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold mb-4">Model Management</h1>
      
      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
          {error}
        </div>
      )}
      
      {/* Provider selection */}
      <div className="mb-6">
        <h2 className="text-lg font-semibold mb-2">LLM Providers</h2>
        <div className="flex flex-wrap gap-2">
          {providers.map(provider => (
            <button
              key={provider.provider_type}
              className={`px-4 py-2 rounded ${activeProvider === provider.provider_type 
                ? 'bg-blue-500 text-white' 
                : 'bg-gray-200 hover:bg-gray-300'}`}
              onClick={() => handleSetActiveProvider(provider.provider_type)}
            >
              {provider.name}
            </button>
          ))}
        </div>
      </div>
      
      {/* Downloaded models section */}
      <div className="mb-6">
        <h2 className="text-lg font-semibold mb-2">Downloaded Models</h2>
        {downloadedModels.length === 0 ? (
          <p className="text-gray-500">No models downloaded yet.</p>
        ) : (
          <div className="space-y-3">
            {downloadedModels.map(model => (
              <div key={model.id} className="border rounded p-3 flex justify-between items-center">
                <div>
                  <div className="font-medium">{model.name}</div>
                  <div className="text-sm text-gray-500">
                    {formatBytes(model.size_bytes)} • {model.provider}
                  </div>
                </div>
                <button
                  className="px-3 py-1 bg-red-100 text-red-800 rounded hover:bg-red-200"
                  onClick={() => handleDeleteModel(model.id)}
                >
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
      
      {/* Available models section */}
      <div>
        <h2 className="text-lg font-semibold mb-2">Available Models</h2>
        <div className="space-y-3">
          {models
            .filter(model => !model.is_downloaded)
            .map(model => (
              <div key={model.id} className="border rounded p-3">
                <div className="flex justify-between items-center">
                  <div>
                    <div className="font-medium">{model.name}</div>
                    <div className="text-sm text-gray-500">
                      {formatBytes(model.size_bytes)} • {model.provider}
                    </div>
                  </div>
                  
                  {downloading[model.id] !== undefined ? (
                    <div className="flex items-center gap-2">
                      <div className="w-32 bg-gray-200 rounded-full h-2.5">
                        <div 
                          className="bg-blue-600 h-2.5 rounded-full" 
                          style={{ width: `${downloading[model.id] * 100}%` }}
                        ></div>
                      </div>
                      <span className="text-sm">{Math.round(downloading[model.id] * 100)}%</span>
                      <button
                        className="px-3 py-1 bg-red-100 text-red-800 rounded hover:bg-red-200"
                        onClick={() => handleCancelDownload(model.id)}
                      >
                        Cancel
                      </button>
                    </div>
                  ) : (
                    <button
                      className="px-3 py-1 bg-blue-100 text-blue-800 rounded hover:bg-blue-200"
                      onClick={() => handleDownloadModel(model.id)}
                    >
                      Download
                    </button>
                  )}
                </div>
                
                {model.description && (
                  <div className="mt-2 text-sm">{model.description}</div>
                )}
                
                {model.tags.length > 0 && (
                  <div className="mt-2 flex flex-wrap gap-1">
                    {model.tags.map(tag => (
                      <span key={tag} className="px-2 py-0.5 bg-gray-100 text-xs rounded">
                        {tag}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            ))}
        </div>
      </div>
    </div>
  );
};

export default ModelManagement;