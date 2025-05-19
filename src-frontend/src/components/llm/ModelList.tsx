import React, { useState } from 'react';
import { Model } from '../../api/EnhancedModelManager';
import './ModelList.css';

interface ModelListProps {
  models: Model[];
  isLoading?: boolean;
  onDownload?: (modelId: string) => void;
  onDelete?: (modelId: string) => void;
  onCancelDownload?: (modelId: string) => void;
  downloadProgress?: Record<string, number>;
  selectedModelId?: string;
  onSelectModel?: (modelId: string) => void;
  showActions?: boolean;
  emptyMessage?: string;
}

/**
 * ModelList component displays a list of models with actions
 */
const ModelList: React.FC<ModelListProps> = ({
  models,
  isLoading = false,
  onDownload,
  onDelete,
  onCancelDownload,
  downloadProgress = {},
  selectedModelId,
  onSelectModel,
  showActions = true,
  emptyMessage = "No models found"
}) => {
  const [expandedModelId, setExpandedModelId] = useState<string | null>(null);

  // Format bytes to human-readable size
  const formatBytes = (bytes: number): string => {
    if (bytes < 1000000) {
      return `${(bytes / 1000000).toFixed(2)} MB`;
    }
    return `${(bytes / 1000000000).toFixed(2)} GB`;
  };

  // Format parameters count
  const formatParams = (params: number): string => {
    if (params < 1000000000) {
      return `${(params / 1000000).toFixed(0)}M`;
    }
    return `${(params / 1000000000).toFixed(1)}B`;
  };

  // Toggle model details
  const toggleDetails = (modelId: string) => {
    setExpandedModelId(expandedModelId === modelId ? null : modelId);
  };

  // Render empty state
  if (models.length === 0 && !isLoading) {
    return (
      <div className="model-list-empty">
        <div className="model-list-empty-icon">📚</div>
        <p>{emptyMessage}</p>
      </div>
    );
  }

  // Render loading state
  if (isLoading) {
    return (
      <div className="model-list-loading">
        <div className="model-list-spinner"></div>
        <p>Loading models...</p>
      </div>
    );
  }

  return (
    <div className="model-list">
      {models.map(model => (
        <div 
          key={model.id}
          className={`model-item ${selectedModelId === model.id ? 'selected' : ''}`}
          onClick={() => onSelectModel && onSelectModel(model.id)}
        >
          <div className="model-item-header">
            <div className="model-item-info">
              <div className="model-item-name">
                {model.name}
                {model.isInstalled && (
                  <span className="model-installed-badge">Installed</span>
                )}
                {model.isLoaded && (
                  <span className="model-loaded-badge">Loaded</span>
                )}
              </div>
              <div className="model-item-meta">
                {formatParams(model.parameterCount)} parameters • {formatBytes(model.sizeMb * 1000000)} • {model.architecture}
              </div>
            </div>
            
            {showActions && (
              <div className="model-item-actions">
                {downloadProgress[model.id] !== undefined ? (
                  <>
                    <div className="download-progress">
                      <div className="download-bar">
                        <div 
                          className="download-fill" 
                          style={{ width: `${downloadProgress[model.id] * 100}%` }}
                        ></div>
                      </div>
                      <div className="download-percentage">
                        {Math.round(downloadProgress[model.id] * 100)}%
                      </div>
                    </div>
                    {onCancelDownload && (
                      <button 
                        className="model-action-button cancel"
                        onClick={(e) => {
                          e.stopPropagation();
                          onCancelDownload(model.id);
                        }}
                      >
                        Cancel
                      </button>
                    )}
                  </>
                ) : model.isInstalled ? (
                  <>
                    <button 
                      className="model-action-button delete"
                      onClick={(e) => {
                        e.stopPropagation();
                        if (onDelete) {
                          // Confirm deletion
                          if (window.confirm(`Are you sure you want to delete ${model.name}?`)) {
                            onDelete(model.id);
                          }
                        }
                      }}
                    >
                      Delete
                    </button>
                    <button 
                      className="model-action-button details"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleDetails(model.id);
                      }}
                    >
                      {expandedModelId === model.id ? 'Hide' : 'Details'}
                    </button>
                  </>
                ) : (
                  <>
                    {onDownload && (
                      <button 
                        className="model-action-button download"
                        onClick={(e) => {
                          e.stopPropagation();
                          onDownload(model.id);
                        }}
                      >
                        Download
                      </button>
                    )}
                    <button 
                      className="model-action-button details"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleDetails(model.id);
                      }}
                    >
                      {expandedModelId === model.id ? 'Hide' : 'Details'}
                    </button>
                  </>
                )}
              </div>
            )}
          </div>
          
          {/* Expanded details section */}
          {expandedModelId === model.id && (
            <div className="model-item-details">
              {model.description && (
                <div className="model-detail-section">
                  <h4>Description</h4>
                  <p>{model.description}</p>
                </div>
              )}
              
              <div className="model-detail-grid">
                <div className="model-detail-section">
                  <h4>Specifications</h4>
                  <div className="model-detail-table">
                    <div className="model-detail-row">
                      <span className="model-detail-label">Architecture</span>
                      <span className="model-detail-value">{model.architecture}</span>
                    </div>
                    <div className="model-detail-row">
                      <span className="model-detail-label">Format</span>
                      <span className="model-detail-value">{model.format}</span>
                    </div>
                    <div className="model-detail-row">
                      <span className="model-detail-label">Parameters</span>
                      <span className="model-detail-value">{formatParams(model.parameterCount)}</span>
                    </div>
                    <div className="model-detail-row">
                      <span className="model-detail-label">Context Length</span>
                      <span className="model-detail-value">{model.contextLength.toLocaleString()} tokens</span>
                    </div>
                    <div className="model-detail-row">
                      <span className="model-detail-label">Quantization</span>
                      <span className="model-detail-value">{model.quantization}</span>
                    </div>
                    <div className="model-detail-row">
                      <span className="model-detail-label">Size</span>
                      <span className="model-detail-value">{formatBytes(model.sizeMb * 1000000)}</span>
                    </div>
                    {model.license && (
                      <div className="model-detail-row">
                        <span className="model-detail-label">License</span>
                        <span className="model-detail-value">{model.license}</span>
                      </div>
                    )}
                    {model.family && (
                      <div className="model-detail-row">
                        <span className="model-detail-label">Family</span>
                        <span className="model-detail-value">{model.family}</span>
                      </div>
                    )}
                  </div>
                </div>
                
                <div className="model-detail-section">
                  <h4>Capabilities</h4>
                  <div className="model-capabilities">
                    <div className={`model-capability ${model.capabilities.textGeneration ? 'supported' : 'unsupported'}`}>
                      Text Generation
                    </div>
                    <div className={`model-capability ${model.capabilities.chat ? 'supported' : 'unsupported'}`}>
                      Chat
                    </div>
                    <div className={`model-capability ${model.capabilities.embeddings ? 'supported' : 'unsupported'}`}>
                      Embeddings
                    </div>
                    <div className={`model-capability ${model.capabilities.imageGeneration ? 'supported' : 'unsupported'}`}>
                      Image Generation
                    </div>
                  </div>
                  
                  {model.tags.length > 0 && (
                    <>
                      <h4>Tags</h4>
                      <div className="model-tags">
                        {model.tags.map(tag => (
                          <span key={tag} className="model-tag">{tag}</span>
                        ))}
                      </div>
                    </>
                  )}
                </div>
              </div>
              
              {model.isInstalled && model.metrics && (
                <div className="model-detail-section">
                  <h4>Performance Metrics</h4>
                  <div className="model-metrics">
                    {model.metrics.inferenceSpeedTokensPerSecond && (
                      <div className="model-metric">
                        <span className="model-metric-value">{model.metrics.inferenceSpeedTokensPerSecond.toFixed(1)} tok/s</span>
                        <span className="model-metric-label">Inference Speed</span>
                      </div>
                    )}
                    {model.metrics.memoryUsageMb && (
                      <div className="model-metric">
                        <span className="model-metric-value">{model.metrics.memoryUsageMb.toFixed(0)} MB</span>
                        <span className="model-metric-label">Memory Usage</span>
                      </div>
                    )}
                    {model.metrics.loadTimeMs && (
                      <div className="model-metric">
                        <span className="model-metric-value">{(model.metrics.loadTimeMs / 1000).toFixed(1)}s</span>
                        <span className="model-metric-label">Load Time</span>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default ModelList;