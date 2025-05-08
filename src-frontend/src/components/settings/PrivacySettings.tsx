import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useFeatureFlags } from '../../contexts/FeatureFlagContext';

interface TelemetryConfig {
  enabled: boolean;
  client_id: string;
  collection_categories: Record<string, boolean>;
  batch_size: number;
  batch_interval_seconds: number;
  server_url: string;
}

const categoryDescriptions: Record<string, string> = {
  app_lifecycle: 'Application startup and shutdown events',
  feature_usage: 'Which features you use and how often',
  errors: 'Error reports to help improve stability',
  performance: 'Performance metrics to optimize the application',
  user_actions: 'User interface interactions and workflow patterns',
  system_info: 'Information about your system configuration',
  logs: 'Application logs for troubleshooting',
};

const PrivacySettings: React.FC = () => {
  const { isFeatureEnabled } = useFeatureFlags();
  const [config, setConfig] = useState<TelemetryConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [hasChanges, setHasChanges] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  
  // Fetch current configuration
  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const currentConfig = await invoke<TelemetryConfig>('get_telemetry_config');
        setConfig(currentConfig);
        setIsLoading(false);
      } catch (error) {
        console.error('Failed to fetch telemetry config:', error);
        setIsLoading(false);
      }
    };
    
    fetchConfig();
  }, []);
  
  // Toggle master switch
  const handleToggleTelemetry = () => {
    if (!config) return;
    
    const newConfig = {
      ...config,
      enabled: !config.enabled,
    };
    
    setConfig(newConfig);
    setHasChanges(true);
  };
  
  // Toggle category
  const handleToggleCategory = (category: string) => {
    if (!config) return;
    
    const newCategories = {
      ...config.collection_categories,
      [category]: !config.collection_categories[category],
    };
    
    const newConfig = {
      ...config,
      collection_categories: newCategories,
    };
    
    setConfig(newConfig);
    setHasChanges(true);
  };
  
  // Reset client ID
  const handleResetClientId = () => {
    if (!config) return;
    
    // Generate new UUID for client ID
    const newClientId = crypto.randomUUID();
    
    const newConfig = {
      ...config,
      client_id: newClientId,
    };
    
    setConfig(newConfig);
    setHasChanges(true);
  };
  
  // Save changes
  const handleSaveChanges = async () => {
    if (!config) return;
    
    try {
      await invoke('update_telemetry_config', { config });
      setHasChanges(false);
      
      // Show saved notification
      alert('Privacy settings saved successfully!');
    } catch (error) {
      console.error('Failed to update telemetry config:', error);
      alert('Failed to save privacy settings. Please try again.');
    }
  };
  
  // Reset changes
  const handleResetChanges = async () => {
    try {
      const currentConfig = await invoke<TelemetryConfig>('get_telemetry_config');
      setConfig(currentConfig);
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch telemetry config:', error);
    }
  };
  
  // Delete telemetry data
  const handleDeleteData = async () => {
    try {
      await invoke('delete_telemetry_data');
      setDeleteConfirm(false);
      
      // Refresh config to get new client ID
      const currentConfig = await invoke<TelemetryConfig>('get_telemetry_config');
      setConfig(currentConfig);
      
      // Show success notification
      alert('Telemetry data deleted successfully. You have been assigned a new anonymous ID.');
    } catch (error) {
      console.error('Failed to delete telemetry data:', error);
      alert('Failed to delete telemetry data. Please try again.');
    }
  };
  
  if (isLoading) {
    return <div className="loading">Loading privacy settings...</div>;
  }
  
  if (!config) {
    return <div className="error">Failed to load privacy settings.</div>;
  }
  
  // Check if the advanced telemetry feature is enabled
  const telemetryEnabled = isFeatureEnabled('advanced_telemetry');
  
  if (!telemetryEnabled) {
    return (
      <div className="privacy-settings">
        <h2>Privacy & Telemetry Settings</h2>
        <div className="feature-disabled">
          <p>Advanced telemetry features are currently disabled. To access these settings, please enable the "Advanced Telemetry" feature.</p>
          <p>You can enable this feature in the Feature Flags section or by joining the Alpha canary group.</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="privacy-settings">
      <h2>Privacy & Telemetry Settings</h2>
      
      <div className="telemetry-main-card">
        <div className="telemetry-main-toggle">
          <h3>Enable Telemetry & Usage Data</h3>
          <label className="switch">
            <input
              type="checkbox"
              checked={config.enabled}
              onChange={handleToggleTelemetry}
            />
            <span className="slider round"></span>
          </label>
          <span>{config.enabled ? 'Enabled' : 'Disabled'}</span>
        </div>
        
        <p className="description">
          Telemetry data helps us improve the MCP Client by understanding how it's used and identifying issues. All data is anonymized and never contains personal information or message content.
        </p>
        
        <div className="client-id-section">
          <div>
            <h4>Your anonymous client ID:</h4>
            <code>{config.client_id}</code>
          </div>
          <button 
            className="secondary-button" 
            onClick={handleResetClientId}
          >
            Generate New ID
          </button>
        </div>
      </div>
      
      <h3>Data Collection Categories</h3>
      <p className="subcategory-desc">
        Configure which types of data you're comfortable sharing. Adjustments only apply if telemetry is enabled.
      </p>
      
      <div className="category-toggles">
        {Object.entries(config.collection_categories).map(([category, enabled]) => (
          <div key={category} className="category-card">
            <div className="category-header">
              <h4>{formatCategoryName(category)}</h4>
              <label className="switch">
                <input
                  type="checkbox"
                  checked={enabled}
                  onChange={() => handleToggleCategory(category)}
                  disabled={!config.enabled}
                />
                <span className="slider round"></span>
              </label>
            </div>
            
            <p className="category-description">
              {categoryDescriptions[category] || 'No description available'}
            </p>
            
            {category === 'performance' && enabled && (
              <div className="subcategory-list">
                <h5>Includes:</h5>
                <ul>
                  <li>CPU & memory usage</li>
                  <li>API latency & throughput</li>
                  <li>UI responsiveness metrics</li>
                  <li>Startup & operation times</li>
                </ul>
              </div>
            )}
          </div>
        ))}
      </div>
      
      {hasChanges && (
        <div className="settings-actions">
          <button className="secondary-button" onClick={handleResetChanges}>
            Cancel
          </button>
          <button className="primary-button" onClick={handleSaveChanges}>
            Save Changes
          </button>
        </div>
      )}
      
      <div className="data-control-card">
        <h3>Your Data Control</h3>
        <p>
          You can delete all collected telemetry data associated with your client ID at any time.
          This will also generate a new anonymous ID for your device.
        </p>
        
        {!deleteConfirm ? (
          <button 
            className="danger-button" 
            onClick={() => setDeleteConfirm(true)}
          >
            Delete My Data
          </button>
        ) : (
          <div className="confirm-delete">
            <p className="warning">Are you sure you want to delete all telemetry data? This action cannot be undone.</p>
            <div className="confirm-buttons">
              <button 
                className="secondary-button" 
                onClick={() => setDeleteConfirm(false)}
              >
                Cancel
              </button>
              <button 
                className="danger-button" 
                onClick={handleDeleteData}
              >
                Yes, Delete My Data
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

// Helper function to format category names
function formatCategoryName(category: string): string {
  return category
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export default PrivacySettings;