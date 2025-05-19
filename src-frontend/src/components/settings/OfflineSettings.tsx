import React, { useState, useEffect } from 'react';
import { Button } from '../ui/Button';
import { useSettings } from '../../contexts/SettingsContext';
import './SettingsSections.css';

/**
 * Offline settings component
 */
const OfflineSettings: React.FC = () => {
  const { settings, updateOfflineSettings } = useSettings();
  const [diskUsage, setDiskUsage] = useState<number | null>(null);
  const [isGatheringInfo, setIsGatheringInfo] = useState(false);
  
  // Fetch disk usage info
  useEffect(() => {
    const fetchDiskUsage = async () => {
      if (settings.offline.enabled) {
        try {
          setIsGatheringInfo(true);
          // In a real implementation, we would call a Tauri command to get actual disk usage
          // For now, we'll just simulate it
          await new Promise(resolve => setTimeout(resolve, 1000));
          
          // Calculate a random usage between 0 and the max disk space
          const usage = Math.floor(Math.random() * settings.offline.maxDiskSpace * 0.7);
          setDiskUsage(usage);
        } catch (error) {
          console.error('Failed to get disk usage:', error);
        } finally {
          setIsGatheringInfo(false);
        }
      } else {
        setDiskUsage(null);
      }
    };
    
    fetchDiskUsage();
  }, [settings.offline.enabled, settings.offline.maxDiskSpace]);
  
  // Handle offline mode toggle
  const handleOfflineModeToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateOfflineSettings({ enabled: e.target.checked });
  };
  
  // Handle auto sync toggle
  const handleAutoSyncToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateOfflineSettings({ autoSync: e.target.checked });
  };
  
  // Handle max disk space change
  const handleMaxDiskSpaceChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = parseInt(e.target.value, 10);
    updateOfflineSettings({ maxDiskSpace: value });
  };
  
  // Format bytes to human-readable size
  const formatSize = (sizeInMB: number): string => {
    if (sizeInMB < 1000) {
      return `${sizeInMB} MB`;
    } else {
      return `${(sizeInMB / 1000).toFixed(1)} GB`;
    }
  };
  
  return (
    <div className="settings-section">
      <h2>Offline Mode</h2>
      
      <div className="settings-form">
        {/* Offline Mode */}
        <div className="form-group checkbox-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.offline.enabled}
              onChange={handleOfflineModeToggle}
            />
            <span>Enable Offline Mode</span>
          </label>
          <p className="form-hint">
            When enabled, you can use the application without an internet connection using local models.
          </p>
        </div>
        
        {settings.offline.enabled && (
          <>
            {/* Auto Sync */}
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.offline.autoSync}
                  onChange={handleAutoSyncToggle}
                />
                <span>Automatic Synchronization</span>
              </label>
              <p className="form-hint">
                When enabled, conversations will automatically sync when you reconnect to the internet.
              </p>
            </div>
            
            {/* Max Disk Space */}
            <div className="form-group">
              <label className="form-label">Maximum Disk Space</label>
              <select
                className="select-input"
                value={settings.offline.maxDiskSpace}
                onChange={handleMaxDiskSpaceChange}
              >
                <option value="1000">1 GB</option>
                <option value="5000">5 GB</option>
                <option value="10000">10 GB</option>
                <option value="20000">20 GB</option>
                <option value="50000">50 GB</option>
              </select>
              <p className="form-hint">
                Maximum amount of disk space to use for storing offline models and data.
              </p>
            </div>
            
            {/* Disk Usage */}
            <div className="form-group">
              <label className="form-label">Current Disk Usage</label>
              <div className="disk-usage">
                {isGatheringInfo ? (
                  <div className="disk-usage-loading">
                    Gathering information...
                  </div>
                ) : diskUsage !== null ? (
                  <>
                    <div className="disk-usage-bar">
                      <div 
                        className="disk-usage-fill" 
                        style={{ width: `${(diskUsage / settings.offline.maxDiskSpace) * 100}%` }}
                      ></div>
                    </div>
                    <div className="disk-usage-text">
                      {formatSize(diskUsage)} of {formatSize(settings.offline.maxDiskSpace)} used
                    </div>
                  </>
                ) : (
                  <div className="disk-usage-empty">
                    No data available
                  </div>
                )}
              </div>
            </div>
            
            {/* Clear Cache Button */}
            <div className="form-group">
              <Button
                variant="secondary"
                onClick={() => {
                  if (window.confirm('Are you sure you want to clear all offline data? This action cannot be undone.')) {
                    alert('Offline data cleared successfully!');
                    // In a real implementation, we would call a Tauri command to clear the cache
                    setDiskUsage(0);
                  }
                }}
              >
                Clear Offline Cache
              </Button>
              <p className="form-hint">
                Removes all downloaded models and offline data.
              </p>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default OfflineSettings;