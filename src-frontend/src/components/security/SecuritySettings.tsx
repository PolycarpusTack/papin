import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import PermissionsManager from './PermissionsManager';
import DataFlowVisualization from './DataFlowVisualization';
import './SecuritySettings.css';

// Types
interface SecurityConfig {
  e2ee_enabled: boolean;
  use_secure_enclave: boolean;
  data_flow_tracking_enabled: boolean;
  default_permission_level: string;
  interactive_permissions: boolean;
  anonymize_telemetry: boolean;
  encrypt_local_storage: boolean;
  credential_cache_duration: number;
  clipboard_security_enabled: boolean;
}

const SecuritySettings: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'general' | 'permissions' | 'data-flow' | 'credentials'>('general');
  const [config, setConfig] = useState<SecurityConfig | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [savingConfig, setSavingConfig] = useState<boolean>(false);
  const [showCredential, setShowCredential] = useState<boolean>(false);
  const [credentialKey, setCredentialKey] = useState<string>('');
  const [credentialValue, setCredentialValue] = useState<string>('');
  const [credentials, setCredentials] = useState<string[]>([]);
  const [credentialError, setCredentialError] = useState<string | null>(null);
  const [savedMessage, setSavedMessage] = useState<string | null>(null);
  
  // Load security config on mount
  useEffect(() => {
    loadData();
  }, []);
  
  // Function to load data from backend
  const loadData = async () => {
    try {
      setLoading(true);
      
      // Load security config
      const configData = await invoke<SecurityConfig>('get_security_config');
      setConfig(configData);
      
      // Load credential keys
      await loadCredentials();
      
      setError(null);
    } catch (err) {
      console.error('Error loading security settings:', err);
      setError(`Failed to load security settings: ${err}`);
    } finally {
      setLoading(false);
    }
  };
  
  // Load credentials list
  const loadCredentials = async () => {
    try {
      const credentialsList = await invoke<string[]>('list_secure_credentials');
      setCredentials(credentialsList);
    } catch (err) {
      console.error('Error loading credentials:', err);
      setCredentialError(`Failed to load credentials: ${err}`);
    }
  };
  
  // Save security config
  const saveConfig = async () => {
    if (!config) return;
    
    try {
      setSavingConfig(true);
      
      await invoke('update_security_config', { config });
      
      // Show saved message
      setSavedMessage('Settings saved successfully');
      setTimeout(() => setSavedMessage(null), 3000);
    } catch (err) {
      console.error('Error saving security settings:', err);
      setError(`Failed to save security settings: ${err}`);
    } finally {
      setSavingConfig(false);
    }
  };
  
  // Handle input change
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    if (!config) return;
    
    const { name, value, type } = e.target;
    
    if (type === 'checkbox') {
      const checked = (e.target as HTMLInputElement).checked;
      setConfig({ ...config, [name]: checked });
    } else if (type === 'number') {
      setConfig({ ...config, [name]: parseInt(value, 10) });
    } else {
      setConfig({ ...config, [name]: value });
    }
  };
  
  // Store a credential
  const storeCredential = async () => {
    if (!credentialKey || !credentialValue) {
      setCredentialError('Both key and value are required');
      return;
    }
    
    try {
      await invoke('store_secure_credential', { key: credentialKey, value: credentialValue });
      
      // Reset form
      setCredentialKey('');
      setCredentialValue('');
      setShowCredential(false);
      setCredentialError(null);
      
      // Reload credentials
      await loadCredentials();
      
      // Show message
      setSavedMessage('Credential stored successfully');
      setTimeout(() => setSavedMessage(null), 3000);
    } catch (err) {
      console.error('Error storing credential:', err);
      setCredentialError(`Failed to store credential: ${err}`);
    }
  };
  
  // Delete a credential
  const deleteCredential = async (key: string) => {
    try {
      await invoke('delete_secure_credential', { key });
      
      // Reload credentials
      await loadCredentials();
      
      // Show message
      setSavedMessage('Credential deleted successfully');
      setTimeout(() => setSavedMessage(null), 3000);
    } catch (err) {
      console.error('Error deleting credential:', err);
      setCredentialError(`Failed to delete credential: ${err}`);
    }
  };
  
  // Get a credential
  const getCredential = async (key: string) => {
    try {
      const value = await invoke<string>('get_secure_credential', { key });
      
      // Set value in form
      setCredentialKey(key);
      setCredentialValue(value);
      setShowCredential(true);
      setCredentialError(null);
    } catch (err) {
      console.error('Error getting credential:', err);
      setCredentialError(`Failed to get credential: ${err}`);
    }
  };
  
  // Rotate encryption keys
  const rotateEncryptionKeys = async () => {
    try {
      await invoke('rotate_encryption_keys');
      
      // Show message
      setSavedMessage('Encryption keys rotated successfully');
      setTimeout(() => setSavedMessage(null), 3000);
    } catch (err) {
      console.error('Error rotating encryption keys:', err);
      setError(`Failed to rotate encryption keys: ${err}`);
    }
  };
  
  // Render General tab
  const renderGeneralTab = () => {
    if (!config) return null;
    
    return (
      <div className="settings-tab">
        <h3>General Security Settings</h3>
        
        <div className="settings-section">
          <div className="setting-group">
            <label htmlFor="e2ee_enabled" className="toggle-label">
              <input
                type="checkbox"
                id="e2ee_enabled"
                name="e2ee_enabled"
                checked={config.e2ee_enabled}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Enable End-to-End Encryption</span>
            </label>
            <p className="setting-description">
              Encrypts your data before sending it to the cloud, ensuring that only you can read it.
            </p>
          </div>
          
          <div className="setting-group">
            <label htmlFor="use_secure_enclave" className="toggle-label">
              <input
                type="checkbox"
                id="use_secure_enclave"
                name="use_secure_enclave"
                checked={config.use_secure_enclave}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Use Secure Enclave for Credentials</span>
            </label>
            <p className="setting-description">
              Stores credentials in your device's secure hardware storage for maximum protection.
            </p>
          </div>
          
          <div className="setting-group">
            <label htmlFor="encrypt_local_storage" className="toggle-label">
              <input
                type="checkbox"
                id="encrypt_local_storage"
                name="encrypt_local_storage"
                checked={config.encrypt_local_storage}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Encrypt Local Storage</span>
            </label>
            <p className="setting-description">
              Encrypts all locally stored data to protect it if your device is lost or stolen.
            </p>
          </div>
          
          <div className="setting-group">
            <label htmlFor="clipboard_security_enabled" className="toggle-label">
              <input
                type="checkbox"
                id="clipboard_security_enabled"
                name="clipboard_security_enabled"
                checked={config.clipboard_security_enabled}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Enable Clipboard Security</span>
            </label>
            <p className="setting-description">
              Automatically clears the clipboard after copying sensitive information.
            </p>
          </div>
        </div>
        
        <div className="settings-section">
          <h4>Privacy Settings</h4>
          
          <div className="setting-group">
            <label htmlFor="anonymize_telemetry" className="toggle-label">
              <input
                type="checkbox"
                id="anonymize_telemetry"
                name="anonymize_telemetry"
                checked={config.anonymize_telemetry}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Anonymize Telemetry Data</span>
            </label>
            <p className="setting-description">
              Removes personally identifiable information from usage data before sending it.
            </p>
          </div>
          
          <div className="setting-group">
            <label htmlFor="data_flow_tracking_enabled" className="toggle-label">
              <input
                type="checkbox"
                id="data_flow_tracking_enabled"
                name="data_flow_tracking_enabled"
                checked={config.data_flow_tracking_enabled}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Enable Data Flow Tracking</span>
            </label>
            <p className="setting-description">
              Tracks how your data moves through the application and shows you where it goes.
            </p>
          </div>
        </div>
        
        <div className="settings-section">
          <h4>Permission Settings</h4>
          
          <div className="setting-group">
            <label htmlFor="default_permission_level">
              Default Permission Level
              <select
                id="default_permission_level"
                name="default_permission_level"
                value={config.default_permission_level}
                onChange={handleInputChange}
                className="select-input"
              >
                <option value="AlwaysAllow">Always Allow</option>
                <option value="AskFirstTime">Ask First Time</option>
                <option value="AskEveryTime">Ask Every Time</option>
                <option value="NeverAllow">Never Allow</option>
              </select>
            </label>
            <p className="setting-description">
              The default level for new permissions that aren't explicitly set.
            </p>
          </div>
          
          <div className="setting-group">
            <label htmlFor="interactive_permissions" className="toggle-label">
              <input
                type="checkbox"
                id="interactive_permissions"
                name="interactive_permissions"
                checked={config.interactive_permissions}
                onChange={handleInputChange}
              />
              <span className="toggle-text">Enable Interactive Permissions</span>
            </label>
            <p className="setting-description">
              Prompts you for permissions when they're needed rather than using defaults.
            </p>
          </div>
          
          <div className="setting-group">
            <label htmlFor="credential_cache_duration">
              Credential Cache Duration (seconds)
              <input
                type="number"
                id="credential_cache_duration"
                name="credential_cache_duration"
                value={config.credential_cache_duration}
                onChange={handleInputChange}
                min="0"
                max="86400"
                className="number-input"
              />
            </label>
            <p className="setting-description">
              How long to keep credentials in memory before requiring re-authentication.
            </p>
          </div>
        </div>
        
        <div className="settings-section">
          <h4>Advanced Security</h4>
          
          <div className="setting-group">
            <button 
              className="action-button" 
              onClick={rotateEncryptionKeys}
            >
              Rotate Encryption Keys
            </button>
            <p className="setting-description">
              Generates new encryption keys and re-encrypts your data for increased security.
            </p>
          </div>
        </div>
        
        <div className="settings-actions">
          <button
            className="save-button"
            onClick={saveConfig}
            disabled={savingConfig}
          >
            {savingConfig ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>
    );
  };
  
  // Render Credentials tab
  const renderCredentialsTab = () => {
    return (
      <div className="settings-tab">
        <h3>Secure Credentials Manager</h3>
        
        <div className="credentials-header">
          <p className="credentials-description">
            Securely store and manage sensitive information like API keys and passwords.
            {config?.use_secure_enclave 
              ? ' Credentials are stored in your device\'s secure enclave.' 
              : ' Credentials are stored encrypted on your device.'}
          </p>
          
          <button
            className="add-button"
            onClick={() => {
              setCredentialKey('');
              setCredentialValue('');
              setShowCredential(true);
              setCredentialError(null);
            }}
          >
            Add Credential
          </button>
        </div>
        
        {credentialError && (
          <div className="credential-error">
            {credentialError}
          </div>
        )}
        
        {showCredential && (
          <div className="credential-form">
            <div className="form-header">
              <h4>{credentialKey ? 'Edit Credential' : 'Add Credential'}</h4>
              <button
                className="close-button"
                onClick={() => setShowCredential(false)}
              >
                ×
              </button>
            </div>
            
            <div className="form-group">
              <label htmlFor="credential-key">
                Credential Key
                <input
                  type="text"
                  id="credential-key"
                  value={credentialKey}
                  onChange={(e) => setCredentialKey(e.target.value)}
                  placeholder="e.g., api_key, database_password"
                  readOnly={!!credentialKey && credentials.includes(credentialKey)}
                />
              </label>
            </div>
            
            <div className="form-group">
              <label htmlFor="credential-value">
                Credential Value
                <input
                  type="password"
                  id="credential-value"
                  value={credentialValue}
                  onChange={(e) => setCredentialValue(e.target.value)}
                  placeholder="Secure value to store"
                />
              </label>
            </div>
            
            <div className="form-actions">
              <button
                className="cancel-button"
                onClick={() => setShowCredential(false)}
              >
                Cancel
              </button>
              <button
                className="store-button"
                onClick={storeCredential}
                disabled={!credentialKey || !credentialValue}
              >
                Store Credential
              </button>
            </div>
          </div>
        )}
        
        <div className="credentials-list">
          <h4>Stored Credentials</h4>
          
          {credentials.length === 0 ? (
            <div className="no-credentials">
              No credentials stored yet. Click "Add Credential" to add one.
            </div>
          ) : (
            <div className="credentials-table">
              <div className="credentials-table-header">
                <div className="credential-key">Credential Key</div>
                <div className="credential-actions">Actions</div>
              </div>
              
              {credentials.map(key => (
                <div key={key} className="credential-row">
                  <div className="credential-key">{key}</div>
                  <div className="credential-actions">
                    <button
                      className="view-button"
                      onClick={() => getCredential(key)}
                      title="View/Edit"
                    >
                      View
                    </button>
                    <button
                      className="delete-button"
                      onClick={() => {
                        if (confirm(`Are you sure you want to delete the credential "${key}"?`)) {
                          deleteCredential(key);
                        }
                      }}
                      title="Delete"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    );
  };
  
  return (
    <div className="security-settings">
      <div className="settings-header">
        <h2>Security & Privacy</h2>
        
        {savedMessage && (
          <div className="saved-message">
            {savedMessage}
          </div>
        )}
      </div>
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      <div className="tabs-container">
        <div className="tabs-navigation">
          <button 
            className={activeTab === 'general' ? 'active' : ''}
            onClick={() => setActiveTab('general')}
          >
            General
          </button>
          <button 
            className={activeTab === 'permissions' ? 'active' : ''}
            onClick={() => setActiveTab('permissions')}
          >
            Permissions
          </button>
          <button 
            className={activeTab === 'data-flow' ? 'active' : ''}
            onClick={() => setActiveTab('data-flow')}
          >
            Data Flow
          </button>
          <button 
            className={activeTab === 'credentials' ? 'active' : ''}
            onClick={() => setActiveTab('credentials')}
          >
            Credentials
          </button>
        </div>
        
        <div className="tab-content">
          {loading ? (
            <div className="loading-message">
              <div className="loading-spinner"></div>
              <div>Loading security settings...</div>
            </div>
          ) : (
            <>
              {activeTab === 'general' && renderGeneralTab()}
              {activeTab === 'permissions' && <PermissionsManager />}
              {activeTab === 'data-flow' && <DataFlowVisualization />}
              {activeTab === 'credentials' && renderCredentialsTab()}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default SecuritySettings;
