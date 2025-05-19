import React, { useState, useEffect, useRef } from 'react';
import modelManager, { Model, Provider, StorageInfo, ModelFilter as ModelFilterType } from '../../api/EnhancedModelManager';
import ModelList from './ModelList';
import ProviderList from './ProviderList';
import StorageUsage from './StorageUsage';
import ModelFilterComponent from './ModelFilterComponent';
import './ModelManagement.css';

/**
 * ModelManagement Component
 * 
 * Comprehensive UI for managing LLM models:
 * - Provider selection
 * - Model filtering and searching
 * - Model downloading and deletion
 * - Storage management
 */
const ModelManagement: React.FC = () => {
  // Mounted ref to track component lifecycle
  const isMountedRef = useRef<boolean>(false);
  
  // Providers state
  const [providers, setProviders] = useState<Provider[]>([]);
  const [activeProviderId, setActiveProviderId] = useState<string>('');
  
  // Models state
  const [allModels, setAllModels] = useState<Model[]>([]);
  const [filteredModels, setFilteredModels] = useState<Model[]>([]);
  const [selectedModelId, setSelectedModelId] = useState<string | null>(null);
  
  // Filter state
  const [filter, setFilter] = useState<ModelFilterType>({});
  
  // UI state
  const [activeTab, setActiveTab] = useState<'available' | 'installed'>('installed');
  const [downloading, setDownloading] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
  const [importDialogOpen, setImportDialogOpen] = useState<boolean>(false);
  const [importPath, setImportPath] = useState<string>('');
  
  // Unsubscribe functions
  const [unsubscribeFns, setUnsubscribeFns] = useState<(() => void)[]>([]);

  // Load initial data
  useEffect(() => {
    async function loadData() {
      try {
        setLoading(true);
        setError(null);

        // Get providers
        const providerList = await modelManager.getProviders();
        setProviders(providerList);
        
        // Get active provider
        const activeProvider = await modelManager.getActiveProvider();
        if (activeProvider) {
          setActiveProviderId(activeProvider.id);
        }
        
        // Get all models
        const models = await modelManager.getAllModels();
        setAllModels(models);
        setFilteredModels(models);
        
        // Get storage info
        const storage = await modelManager.getStorageInfo();
        if (storage) {
          setStorageInfo(storage);
        }
        
        // Register for model events
        const unsubscribeModelEvents = await modelManager.subscribeToModelEvents(
          (eventType, modelId, data) => {
            if (eventType === 'added' || eventType === 'removed' || eventType === 'updated') {
              // Refresh model list
              refreshModels();
            }
          }
        );
        
        // Register for download events
        const unsubscribeDownloadEvents = await modelManager.subscribeToDownloadEvents(
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
            refreshStorageInfo();
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
        
        // Store unsubscribe functions
        setUnsubscribeFns([unsubscribeModelEvents, unsubscribeDownloadEvents]);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    }
    
    loadData();
    
    // Set mounted ref
    isMountedRef.current = true;
    
    // Cleanup on unmount
    return () => {
      isMountedRef.current = false;
      unsubscribeFns.forEach(fn => fn());
    };
  }, [unsubscribeFns]);
  
  // Apply filters to models when filter changes
  useEffect(() => {
    applyFilters();
  }, [filter, allModels, activeTab]);
  
  // Filter models based on current filter
  const applyFilters = () => {
    let filtered = [...allModels];
    
    // Filter by installed status based on active tab
    filtered = filtered.filter(model => 
      activeTab === 'installed' ? model.isInstalled : true
    );
    
    // Apply search query
    if (filter.searchQuery) {
      const query = filter.searchQuery.toLowerCase();
      filtered = filtered.filter(model => 
        model.name.toLowerCase().includes(query) || 
        model.description.toLowerCase().includes(query)
      );
    }
    
    // Filter by provider
    if (filter.providerId) {
      filtered = filtered.filter(model => model.provider === filter.providerId);
    }
    
    // Filter by installed state
    if (filter.isInstalled !== undefined) {
      filtered = filtered.filter(model => model.isInstalled === filter.isInstalled);
    }
    
    // Filter by architecture
    if (filter.architecture && filter.architecture.length > 0) {
      filtered = filtered.filter(model => 
        filter.architecture?.includes(model.architecture)
      );
    }
    
    // Filter by format
    if (filter.format && filter.format.length > 0) {
      filtered = filtered.filter(model => 
        filter.format?.includes(model.format)
      );
    }
    
    // Filter by quantization
    if (filter.quantization && filter.quantization.length > 0) {
      filtered = filtered.filter(model => 
        filter.quantization?.includes(model.quantization)
      );
    }
    
    // Filter by capabilities
    if (filter.capabilities) {
      if (filter.capabilities.textGeneration) {
        filtered = filtered.filter(model => model.capabilities.textGeneration);
      }
      if (filter.capabilities.chat) {
        filtered = filtered.filter(model => model.capabilities.chat);
      }
      if (filter.capabilities.embeddings) {
        filtered = filtered.filter(model => model.capabilities.embeddings);
      }
      if (filter.capabilities.imageGeneration) {
        filtered = filtered.filter(model => model.capabilities.imageGeneration);
      }
    }
    
    // Filter by context length
    if (filter.minContextLength) {
      filtered = filtered.filter(model => model.contextLength >= (filter.minContextLength || 0));
    }
    
    // Filter by size
    if (filter.maxSizeMb) {
      filtered = filtered.filter(model => model.sizeMb <= (filter.maxSizeMb || Infinity));
    }
    
    setFilteredModels(filtered);
  };
  
  // Refresh models
  const refreshModels = async () => {
    try {
      const models = await modelManager.getAllModels();
      setAllModels(models);
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Refresh storage info
  const refreshStorageInfo = async () => {
    try {
      const storage = await modelManager.getStorageInfo();
      if (storage) {
        setStorageInfo(storage);
      }
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Handle provider selection
  const handleProviderSelect = async (providerId: string) => {
    try {
      await modelManager.setActiveProvider(providerId);
      setActiveProviderId(providerId);
      
      // Update filter to include the selected provider
      setFilter(prev => ({
        ...prev,
        providerId
      }));
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Handle model download
  const handleDownloadModel = async (modelId: string) => {
    try {
      const success = await modelManager.downloadModel(modelId);
      if (success) {
        setDownloading(prev => ({
          ...prev,
          [modelId]: 0
        }));
      }
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Handle cancel download
  const handleCancelDownload = async (modelId: string) => {
    try {
      await modelManager.cancelDownload(modelId);
      setDownloading(prev => {
        const newState = { ...prev };
        delete newState[modelId];
        return newState;
      });
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Handle delete model
  const handleDeleteModel = async (modelId: string) => {
    try {
      await modelManager.deleteModel(modelId);
      refreshModels();
      refreshStorageInfo();
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Handle model import
  const handleImportModel = async () => {
    if (!importPath) return;
    
    try {
      const importId = `imported-${Date.now()}`;
      const model = await modelManager.importModel(importPath, importId);
      if (model) {
        setImportDialogOpen(false);
        setImportPath('');
        refreshModels();
        refreshStorageInfo();
      }
    } catch (err) {
      setError(`Import failed: ${String(err)}`);
    }
  };
  
  // Handle storage cleanup
  const handleCleanupStorage = async () => {
    try {
      const bytesFreed = await modelManager.cleanupUnusedModels(30); // Clean models not used in 30 days
      refreshStorageInfo();
      if (bytesFreed > 0) {
        const freedMb = Math.round(bytesFreed / 1024 / 1024);
        setSuccessMessage(`Successfully freed ${freedMb} MB of storage space.`);
      } else {
        setSuccessMessage('No unused models found to clean up.');
      }
    } catch (err) {
      setError(`Cleanup failed: ${String(err)}`);
    }
  };
  
  // Handle filter changes
  const handleFilterChange = (newFilter: ModelFilterType) => {
    setFilter(newFilter);
  };
  
  // Handle tab change
  const handleTabChange = (tab: 'available' | 'installed') => {
    setActiveTab(tab);
  };
  
  // Render alerts
  const renderAlerts = () => {
    return (
      <>
        {error && (
          <div className="error-alert">
            <p>{error}</p>
            <button 
              onClick={() => setError(null)}
              className="error-dismiss"
              aria-label="Dismiss error"
            >
              Dismiss
            </button>
          </div>
        )}
        
        {successMessage && (
          <div className="success-alert">
            <p>{successMessage}</p>
            <button 
              onClick={() => setSuccessMessage(null)}
              className="success-dismiss"
              aria-label="Dismiss message"
            >
              Dismiss
            </button>
          </div>
        )}
      </>
    );
  };
  
  // Handle keydown for import dialog
  const handleImportDialogKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setImportDialogOpen(false);
    } else if (e.key === 'Enter' && importPath) {
      handleImportModel();
    }
  };
  
  // Render import dialog
  const renderImportDialog = () => {
    if (!importDialogOpen) return null;
    
    return (
      <div 
        className="import-dialog-overlay"
        onClick={(e) => {
          // Close when clicking the overlay
          if (e.target === e.currentTarget) {
            setImportDialogOpen(false);
          }
        }}
      >
        <div 
          className="import-dialog"
          role="dialog"
          aria-labelledby="import-dialog-title"
          aria-describedby="import-dialog-description"
          onKeyDown={handleImportDialogKeyDown}
          tabIndex={-1}
        >
          <h3 id="import-dialog-title">Import Model</h3>
          <p id="import-dialog-description">Enter the path to the model file you want to import:</p>
          <input 
            type="text" 
            value={importPath}
            onChange={(e) => setImportPath(e.target.value)}
            placeholder="/path/to/model.gguf"
            className="import-input"
            autoFocus
          />
          <div className="import-actions">
            <button
              onClick={() => setImportDialogOpen(false)}
              className="cancel-button"
              type="button"
            >
              Cancel
            </button>
            <button
              onClick={handleImportModel}
              className="import-button"
              disabled={!importPath}
              aria-disabled={!importPath ? 'true' : 'false'}
              type="button"
            >
              Import
            </button>
          </div>
        </div>
      </div>
    );
  };
  
  return (
    <div className="model-management">
      {renderAlerts()}
      {renderImportDialog()}
      
      <div className="model-management-header">
        <h1>Model Management</h1>
        <div className="header-actions">
          <button
            onClick={() => setImportDialogOpen(true)}
            className="import-model-button"
          >
            Import Model
          </button>
        </div>
      </div>
      
      <div className="model-management-layout">
        {/* Sidebar */}
        <div className="model-management-sidebar">
          <h2 className="sidebar-title">Providers</h2>
          <ProviderList
            providers={providers}
            activeProviderId={activeProviderId}
            onSelectProvider={handleProviderSelect}
            isLoading={loading}
          />
          
          {storageInfo && (
            <StorageUsage
              storageInfo={storageInfo}
              isLoading={loading}
              onCleanupStorage={handleCleanupStorage}
            />
          )}
        </div>
        
        {/* Main content */}
        <div className="model-management-content">
          {/* Tab navigation */}
          <div className="model-tabs" role="tablist" aria-label="Model categories">
            <button
              role="tab"
              id="tab-installed"
              aria-controls="panel-installed"
              aria-selected={activeTab === 'installed'}
              className={`tab-button ${activeTab === 'installed' ? 'active' : ''}`}
              onClick={() => handleTabChange('installed')}
            >
              Installed Models
            </button>
            <button
              role="tab"
              id="tab-available"
              aria-controls="panel-available"
              aria-selected={activeTab === 'available'}
              className={`tab-button ${activeTab === 'available' ? 'active' : ''}`}
              onClick={() => handleTabChange('available')}
            >
              Available Models
            </button>
          </div>
          
          {/* Filter section */}
          <ModelFilterComponent
            onFilterChange={handleFilterChange}
            initialFilter={filter}
            showProviderFilter={true}
            providers={providers.map(p => ({ id: p.id, name: p.name }))}
          />
          
          {/* Model list */}
          <div 
            className="model-list-container"
            role="tabpanel"
            id={`panel-${activeTab}`}
            aria-labelledby={`tab-${activeTab}`}
          >
            <ModelList
              models={filteredModels}
              isLoading={loading}
              onDownload={handleDownloadModel}
              onDelete={handleDeleteModel}
              onCancelDownload={handleCancelDownload}
              downloadProgress={downloading}
              selectedModelId={selectedModelId || undefined}
              onSelectModel={(id) => setSelectedModelId(id)}
              showActions={true}
              emptyMessage={
                activeTab === 'installed' 
                  ? "No installed models found. Go to 'Available Models' to download some."
                  : "No available models found with current filters."
              }
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default ModelManagement;