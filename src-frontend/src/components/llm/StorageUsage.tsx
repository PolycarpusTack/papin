import React from 'react';
import { StorageInfo } from '../../api/EnhancedModelManager';
import './StorageUsage.css';

interface StorageUsageProps {
  storageInfo: StorageInfo;
  isLoading?: boolean;
  onCleanupStorage?: () => void;
}

/**
 * StorageUsage component displays storage usage information for models
 */
const StorageUsage: React.FC<StorageUsageProps> = ({
  storageInfo,
  isLoading = false,
  onCleanupStorage
}) => {
  // Format bytes to human-readable size
  const formatBytes = (bytes: number, decimals = 1): string => {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
  };
  
  // Get color for usage bar based on percentage
  const getUsageColor = (percentage: number): string => {
    if (percentage > 90) return 'var(--color-error)';
    if (percentage > 70) return 'var(--color-warning)';
    return 'var(--color-primary)';
  };
  
  // If loading, show skeleton
  if (isLoading) {
    return (
      <div className="storage-usage skeleton">
        <div className="storage-header skeleton-text"></div>
        <div className="storage-bar skeleton-bar"></div>
        <div className="storage-details">
          <div className="skeleton-text"></div>
          <div className="skeleton-text"></div>
        </div>
      </div>
    );
  }
  
  const usedPercentage = storageInfo.percentUsed;
  const barColor = getUsageColor(usedPercentage);
  
  return (
    <div className="storage-usage">
      <div className="storage-header">
        <h3>Storage Usage</h3>
        {onCleanupStorage && (
          <button 
            className="cleanup-button" 
            onClick={onCleanupStorage}
            title="Clean up unused models to free space"
          >
            Clean Up
          </button>
        )}
      </div>
      
      <div className="storage-bar-container">
        <div className="storage-bar">
          <div 
            className="storage-bar-fill" 
            style={{ 
              width: `${Math.min(100, usedPercentage)}%`,
              backgroundColor: barColor
            }}
          ></div>
        </div>
        <div className="storage-percentage">{Math.round(usedPercentage)}%</div>
      </div>
      
      <div className="storage-details">
        <div className="storage-detail">
          <span className="storage-label">Used</span>
          <span className="storage-value">{formatBytes(storageInfo.usedBytes)}</span>
        </div>
        <div className="storage-detail">
          <span className="storage-label">Available</span>
          <span className="storage-value">{formatBytes(storageInfo.availableBytes)}</span>
        </div>
        <div className="storage-detail">
          <span className="storage-label">Total</span>
          <span className="storage-value">{formatBytes(storageInfo.totalBytes)}</span>
        </div>
      </div>
      
      <div className="storage-model-count">
        <span className="model-count-value">{storageInfo.modelCount}</span>
        <span className="model-count-label">
          {storageInfo.modelCount === 1 ? 'Model' : 'Models'} Installed
        </span>
      </div>
    </div>
  );
};

export default StorageUsage;