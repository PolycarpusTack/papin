import React, { useState } from 'react';
import { Input } from '../ui/Input';
import { Button } from '../ui/Button';
import { useSettings } from '../../contexts/SettingsContext';
import './SettingsSections.css';

interface GeneralSettingsProps {
  showSaveButton?: boolean;
}

/**
 * General settings component
 */
const GeneralSettings: React.FC<GeneralSettingsProps> = ({ 
  showSaveButton = false 
}) => {
  const { 
    settings, 
    updateApiSettings,
    setApiKey,
    hasApiKey,
    saveAllSettings,
    error
  } = useSettings();
  
  const [apiKeyValue, setApiKeyValue] = useState('');
  const [apiKeySet, setApiKeySet] = useState<boolean | null>(null);
  const [saveMessage, setSaveMessage] = useState('');
  
  // Check if API key is set
  React.useEffect(() => {
    const checkApiKey = async () => {
      const hasKey = await hasApiKey();
      setApiKeySet(hasKey);
    };
    
    checkApiKey();
  }, [hasApiKey]);
  
  // Handle saving API key
  const handleSaveApiKey = async () => {
    if (!apiKeyValue.trim()) {
      setSaveMessage('API key cannot be empty');
      return;
    }
    
    try {
      await setApiKey(apiKeyValue);
      setApiKeyValue('');
      setApiKeySet(true);
      setSaveMessage('API key saved successfully');
      
      // Clear message after a delay
      setTimeout(() => {
        setSaveMessage('');
      }, 3000);
    } catch (err) {
      setSaveMessage('Failed to save API key');
      console.error('Failed to save API key:', err);
    }
  };
  
  // Handle save all settings
  const handleSaveSettings = async () => {
    try {
      await saveAllSettings();
      setSaveMessage('Settings saved successfully');
      
      // Clear message after a delay
      setTimeout(() => {
        setSaveMessage('');
      }, 3000);
    } catch (err) {
      setSaveMessage('Failed to save settings');
      console.error('Failed to save settings:', err);
    }
  };
  
  // Update API URL
  const handleApiUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateApiSettings({ url: e.target.value });
  };
  
  // Update default model
  const handleModelChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    updateApiSettings({ model: e.target.value });
  };
  
  return (
    <div className="settings-section">
      <h2>General Settings</h2>
      
      {error && (
        <div className="settings-error">
          {error}
        </div>
      )}
      
      {saveMessage && (
        <div className="settings-message">
          {saveMessage}
        </div>
      )}
      
      <div className="settings-form">
        {/* API Key */}
        <div className="form-group">
          <label className="form-label">
            API Key
            {apiKeySet !== null && (
              <span className={`api-key-status ${apiKeySet ? 'set' : 'not-set'}`}>
                {apiKeySet ? '(Set)' : '(Not Set)'}
              </span>
            )}
          </label>
          <div className="input-with-button">
            <Input
              type="password"
              value={apiKeyValue}
              onChange={(e) => setApiKeyValue(e.target.value)}
              placeholder="Enter your API key"
              fullWidth
            />
            <Button 
              onClick={handleSaveApiKey} 
              disabled={!apiKeyValue.trim()}
            >
              Save Key
            </Button>
          </div>
          <p className="form-hint">
            Your API key is stored securely and never shared.
          </p>
        </div>
        
        {/* API URL */}
        <div className="form-group">
          <label className="form-label">API Endpoint</label>
          <Input
            type="text"
            value={settings.api.url}
            onChange={handleApiUrlChange}
            placeholder="Enter API endpoint URL"
            fullWidth
          />
          <p className="form-hint">
            WebSocket endpoint for the Model Context Protocol.
          </p>
        </div>
        
        {/* Default Model */}
        <div className="form-group">
          <label className="form-label">Default Model</label>
          <select 
            className="select-input"
            value={settings.api.model}
            onChange={handleModelChange}
          >
            <option value="claude-3-opus-20240229">Claude 3 Opus</option>
            <option value="claude-3-sonnet-20240229">Claude 3 Sonnet</option>
            <option value="claude-3-haiku-20240307">Claude 3 Haiku</option>
            <option value="claude-2.1">Claude 2.1</option>
            <option value="claude-2.0">Claude 2.0</option>
          </select>
          <p className="form-hint">
            The model that will be used by default for new conversations.
          </p>
        </div>
      </div>
      
      {showSaveButton && (
        <div className="settings-actions">
          <Button
            variant="primary"
            onClick={handleSaveSettings}
          >
            Save Settings
          </Button>
        </div>
      )}
    </div>
  );
};

export default GeneralSettings;