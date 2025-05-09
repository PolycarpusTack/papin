import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './OfflineSettings.css';

// Define interfaces for our data
interface OfflineConfig {
  enabled: boolean;
  auto_switch: boolean;
  max_history_size: number;
  auto_sync: boolean;
  models_directory: string;
  default_model: string;
  checkpoint_directory: string;
  checkpoint_interval_seconds: number;
  compress_checkpoints: boolean;
  network_check_interval_seconds: number;
}

interface OfflineStats {
  connectivity_status: 'Online' | 'Limited' | 'Offline';
  offline_active: boolean;
  pending_sync_count: number;
  current_model: string;
  last_checkpoint_time?: string;
  checkpoint_count: number;
}

const OfflineSettings: React.FC = () => {
  // State for the offline configuration
  const [config, setConfig] = useState<OfflineConfig>({
    enabled: true,
    auto_switch: true,
    max_history_size: 100,
    auto_sync: true,
    models_directory: 'models',
    default_model: 'ggml-model-q4_0.bin',
    checkpoint_directory: 'checkpoints',
    checkpoint_interval_seconds: 60,
    compress_checkpoints: true,
    network_check_interval_seconds: 30,
  });
  
  // State for offline stats
  const [stats, setStats] = useState<OfflineStats | null>(null);
  
  // State for available models
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  
  // State for loading indicators
  const [loading, setLoading] = useState<boolean>(true);
  const [saving, setSaving] = useState<boolean>(false);
  const [syncing, setSyncing] = useState<boolean>(false);
  
  // State for notifications
  const [notification, setNotification] = useState<{ type: 'success' | 'error', message: string } | null>(null);
  
  // Load configuration and stats on component mount
  useEffect(() => {
    loadConfigAndStats();
    loadAvailableModels();
  }, []);
  
  // Load configuration and stats from the backend
  const loadConfigAndStats = async () => {
    setLoading(true);
    
    try {
      // Load configuration
      const [configResult, statsResult] = await Promise.all([
        invoke<OfflineConfig>('get_offline_config'),
        invoke<OfflineStats>('get_offline_stats'),
      ]);
      
      setConfig(configResult);
      setStats(statsResult);
    } catch (err) {
      console.error('Error loading offline settings:', err);
      setNotification({
        type: 'error',
        message: `Error loading settings: ${err}`,
      });
    } finally {
      setLoading(false);
    }
  };
  
  // Load available local models
  const loadAvailableModels = async () => {
    try {
      const models = await invoke<string[]>('get_available_local_models');
      setAvailableModels(models);
    } catch (err) {
      console.error('Error loading available models:', err);
    }
  };
  
  // Save configuration
  const saveConfig = async () => {
    setSaving(true);
    
    try {
      const result = await invoke<{ success: boolean, message: string }>('update_offline_config', { config });
      
      if (result.success) {
        setNotification({
          type: 'success',
          message: 'Settings saved successfully',
        });
        
        // Reload stats to reflect changes
        const statsResult = await invoke<OfflineStats>('get_offline_stats');
        setStats(statsResult);
      } else {
        setNotification({
          type: 'error',
          message: result.message,
        });
      }
    } catch (err) {
      console.error('Error saving settings:', err);
      setNotification({
        type: 'error',
        message: `Error saving settings: ${err}`,
      });
    } finally {
      setSaving(false);
    }
  };
  
  // Handle form submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    saveConfig();
  };
  
  // Handle input changes
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target;
    
    // Handle different input types
    if (type === 'checkbox') {
      const checked = (e.target as HTMLInputElement).checked;
      setConfig({ ...config, [name]: checked });
    } else if (type === 'number') {
      setConfig({ ...config, [name]: parseInt(value, 10) });
    } else {
      setConfig({ ...config, [name]: value });
    }
  };
  
  // Manually enable offline mode
  const enableOfflineMode = async () => {
    try {
      const result = await invoke<{ success: boolean, message: string }>('enable_offline_mode');
      
      if (result.success) {
        setNotification({
          type: 'success',
          message: 'Offline mode enabled',
        });
        
        // Reload stats
        const statsResult = await invoke<OfflineStats>('get_offline_stats');
        setStats(statsResult);
      } else {
        setNotification({
          type: 'error',
          message: result.message,
        });
      }
    } catch (err) {
      console.error('Error enabling offline mode:', err);
      setNotification({
        type: 'error',
        message: `Error enabling offline mode: ${err}`,
      });
    }
  };
  
  // Manually disable offline mode
  const disableOfflineMode = async () => {
    try {
      const result = await invoke<{ success: boolean, message: string }>('disable_offline_mode');
      
      if (result.success) {
        setNotification({
          type: 'success',
          message: 'Offline mode disabled',
        });
        
        // Reload stats
        const statsResult = await invoke<OfflineStats>('get_offline_stats');
        setStats(statsResult);
      } else {
        setNotification({
          type: 'error',
          message: result.message,
        });
      }
    } catch (err) {
      console.error('Error disabling offline mode:', err);
      setNotification({
        type: 'error',
        message: `Error disabling offline mode: ${err}`,
      });
    }
  };
  
  // Manually sync changes
  const syncChanges = async () => {
    setSyncing(true);
    
    try {
      const result = await invoke<{ success: boolean, message: string }>('sync_offline_changes');
      
      if (result.success) {
        setNotification({
          type: 'success',
          message: 'Changes synced successfully',
        });
        
        // Reload stats
        const statsResult = await invoke<OfflineStats>('get_offline_stats');
        setStats(statsResult);
      } else {
        setNotification({
          type: 'error',
          message: result.message,
        });
      }
    } catch (err) {
      console.error('Error syncing changes:', err);
      setNotification({
        type: 'error',
        message: `Error syncing changes: ${err}`,
      });
    } finally {
      setSyncing(false);
    }
  };
  
  // Clear notification after 5 seconds
  useEffect(() => {
    if (notification) {
      const timer = setTimeout(() => {
        setNotification(null);
      }, 5000);
      
      return () => clearTimeout(timer);
    }
  }, [notification]);
  
  // Format date for display
  const formatDate = (dateString?: string) => {
    if (!dateString) return 'Never';
    
    return new Date(dateString).toLocaleString();
  };
  
  return (
    <div className="offline-settings">
      <h2>Offline Mode Settings</h2>
      
      {notification && (
        <div className={`notification ${notification.type}`}>
          {notification.message}
        </div>
      )}
      
      <div className="settings-grid">
        <div className="settings-section">
          <h3>Configuration</h3>
          
          {loading ? (
            <div className="loading">Loading settings...</div>
          ) : (
            <form onSubmit={handleSubmit}>
              <div className="form-group">
                <label htmlFor="enabled">
                  <input
                    type="checkbox"
                    id="enabled"
                    name="enabled"
                    checked={config.enabled}
                    onChange={handleInputChange}
                  />
                  Enable offline mode
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="auto_switch">
                  <input
                    type="checkbox"
                    id="auto_switch"
                    name="auto_switch"
                    checked={config.auto_switch}
                    onChange={handleInputChange}
                  />
                  Automatically switch to offline mode when connectivity is lost
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="auto_sync">
                  <input
                    type="checkbox"
                    id="auto_sync"
                    name="auto_sync"
                    checked={config.auto_sync}
                    onChange={handleInputChange}
                  />
                  Automatically sync when connectivity is restored
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="max_history_size">
                  Maximum conversation history size
                  <input
                    type="number"
                    id="max_history_size"
                    name="max_history_size"
                    value={config.max_history_size}
                    onChange={handleInputChange}
                    min="10"
                    max="1000"
                  />
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="models_directory">
                  Models directory
                  <input
                    type="text"
                    id="models_directory"
                    name="models_directory"
                    value={config.models_directory}
                    onChange={handleInputChange}
                  />
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="default_model">
                  Default local model
                  <select
                    id="default_model"
                    name="default_model"
                    value={config.default_model}
                    onChange={handleInputChange}
                  >
                    {availableModels.length === 0 ? (
                      <option value="">No models available</option>
                    ) : (
                      availableModels.map(model => (
                        <option key={model} value={model}>{model}</option>
                      ))
                    )}
                  </select>
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="checkpoint_directory">
                  Checkpoint directory
                  <input
                    type="text"
                    id="checkpoint_directory"
                    name="checkpoint_directory"
                    value={config.checkpoint_directory}
                    onChange={handleInputChange}
                  />
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="checkpoint_interval_seconds">
                  Checkpoint interval (seconds)
                  <input
                    type="number"
                    id="checkpoint_interval_seconds"
                    name="checkpoint_interval_seconds"
                    value={config.checkpoint_interval_seconds}
                    onChange={handleInputChange}
                    min="10"
                    max="3600"
                  />
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="compress_checkpoints">
                  <input
                    type="checkbox"
                    id="compress_checkpoints"
                    name="compress_checkpoints"
                    checked={config.compress_checkpoints}
                    onChange={handleInputChange}
                  />
                  Compress checkpoints to save space
                </label>
              </div>
              
              <div className="form-group">
                <label htmlFor="network_check_interval_seconds">
                  Network check interval (seconds)
                  <input
                    type="number"
                    id="network_check_interval_seconds"
                    name="network_check_interval_seconds"
                    value={config.network_check_interval_seconds}
                    onChange={handleInputChange}
                    min="5"
                    max="300"
                  />
                </label>
              </div>
              
              <div className="form-actions">
                <button 
                  type="submit" 
                  className="primary-button"
                  disabled={saving}
                >
                  {saving ? 'Saving...' : 'Save Settings'}
                </button>
              </div>
            </form>
          )}
        </div>
        
        <div className="settings-section">
          <h3>Status</h3>
          
          {loading ? (
            <div className="loading">Loading status...</div>
          ) : stats ? (
            <div className="status-info">
              <div className="status-item">
                <span className="status-label">Connectivity:</span>
                <span className={`status-value status-${stats.connectivity_status.toLowerCase()}`}>
                  {stats.connectivity_status}
                </span>
              </div>
              
              <div className="status-item">
                <span className="status-label">Offline Mode:</span>
                <span className={`status-value ${stats.offline_active ? 'status-active' : 'status-inactive'}`}>
                  {stats.offline_active ? 'Active' : 'Inactive'}
                </span>
              </div>
              
              <div className="status-item">
                <span className="status-label">Current Model:</span>
                <span className="status-value">{stats.current_model}</span>
              </div>
              
              <div className="status-item">
                <span className="status-label">Pending Sync Items:</span>
                <span className="status-value">{stats.pending_sync_count}</span>
              </div>
              
              <div className="status-item">
                <span className="status-label">Last Checkpoint:</span>
                <span className="status-value">{formatDate(stats.last_checkpoint_time)}</span>
              </div>
              
              <div className="status-item">
                <span className="status-label">Checkpoint Count:</span>
                <span className="status-value">{stats.checkpoint_count}</span>
              </div>
              
              <div className="status-actions">
                {stats.connectivity_status === 'Online' && stats.offline_active && (
                  <button 
                    className="action-button" 
                    onClick={disableOfflineMode}
                  >
                    Disable Offline Mode
                  </button>
                )}
                
                {stats.connectivity_status === 'Online' && !stats.offline_active && (
                  <button 
                    className="action-button" 
                    onClick={enableOfflineMode}
                  >
                    Enable Offline Mode
                  </button>
                )}
                
                {(stats.connectivity_status === 'Limited' || stats.connectivity_status === 'Offline') && !stats.offline_active && (
                  <button 
                    className="action-button" 
                    onClick={enableOfflineMode}
                  >
                    Enable Offline Mode
                  </button>
                )}
                
                {stats.connectivity_status === 'Online' && stats.pending_sync_count > 0 && (
                  <button 
                    className="action-button sync-button" 
                    onClick={syncChanges}
                    disabled={syncing}
                  >
                    {syncing ? 'Syncing...' : `Sync Changes (${stats.pending_sync_count})`}
                  </button>
                )}
              </div>
            </div>
          ) : (
            <div className="error-message">Failed to load status information</div>
          )}
        </div>
      </div>
      
      <div className="offline-help">
        <h3>About Offline Mode</h3>
        <p>
          Offline mode allows you to use Claude even when you don't have internet connectivity.
          It uses a local language model that runs on your device instead of connecting to the cloud.
        </p>
        <p>
          <strong>Note:</strong> The local model has limitations compared to the cloud version of Claude.
          Responses may be shorter, less accurate, and take longer to generate.
        </p>
        <p>
          When connectivity is restored, your offline conversations will be automatically synchronized
          with the cloud, allowing you to continue your work seamlessly.
        </p>
      </div>
    </div>
  );
};

export default OfflineSettings;
