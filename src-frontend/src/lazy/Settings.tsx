import React, { useState } from 'react';
import { Input } from '../components/ui/Input';
import { Button } from '../components/ui/Button';
import './Settings.css';

const Settings: React.FC = () => {
  const [apiKey, setApiKey] = useState('');
  const [selectedModel, setSelectedModel] = useState('claude-3-opus-20240229');
  
  const handleSaveSettings = () => {
    // In a real app, we would send this to the backend
    console.log('Saving settings:', { apiKey, selectedModel });
    // Show a success message or notification
    alert('Settings saved successfully!');
  };
  
  return (
    <div className="settings-container">
      <div className="settings-content">
        <div className="settings-section">
          <h2 className="settings-heading">General Settings</h2>
          
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">API Key</label>
              <Input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Enter your Anthropic API key"
                fullWidth
              />
              <p className="form-hint">
                Your API key is stored securely and never shared.
              </p>
            </div>
            
            <div className="form-group">
              <label className="form-label">Default Model</label>
              <select 
                className="select-input"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
              >
                <option value="claude-3-opus-20240229">Claude 3 Opus</option>
                <option value="claude-3-sonnet-20240229">Claude 3 Sonnet</option>
                <option value="claude-3-haiku-20240307">Claude 3 Haiku</option>
              </select>
              <p className="form-hint">
                The model that will be used by default for new conversations.
              </p>
            </div>
            
            <div className="form-group">
              <label className="form-label">Language</label>
              <select className="select-input">
                <option value="en">English</option>
                <option value="fr">French</option>
                <option value="de">German</option>
                <option value="es">Spanish</option>
              </select>
            </div>
            
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input type="checkbox" defaultChecked />
                <span>Enable message history</span>
              </label>
              <p className="form-hint">
                Store conversation history locally for future reference.
              </p>
            </div>
            
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input type="checkbox" defaultChecked />
                <span>Enable desktop notifications</span>
              </label>
            </div>
          </div>
        </div>
        
        <div className="settings-section">
          <h2 className="settings-heading">Advanced Settings</h2>
          
          <div className="settings-form">
            <div className="form-group">
              <label className="form-label">Temperature</label>
              <div className="range-control">
                <input 
                  type="range" 
                  min="0" 
                  max="1" 
                  step="0.1" 
                  defaultValue="0.7" 
                  className="range-input"
                />
                <span className="range-value">0.7</span>
              </div>
              <p className="form-hint">
                Controls randomness in responses. Lower values are more deterministic.
              </p>
            </div>
            
            <div className="form-group">
              <label className="form-label">Max Tokens</label>
              <Input
                type="number"
                defaultValue="4000"
                fullWidth
              />
              <p className="form-hint">
                Maximum number of tokens for model responses.
              </p>
            </div>
            
            <div className="form-group checkbox-group">
              <label className="checkbox-label">
                <input type="checkbox" defaultChecked />
                <span>Use streaming mode</span>
              </label>
              <p className="form-hint">
                Receive responses as they are generated instead of waiting for complete responses.
              </p>
            </div>
          </div>
        </div>
        
        <div className="settings-actions">
          <Button 
            variant="outline"
            onClick={() => {
              // Reset form or prompt for confirmation
              if (confirm('Reset all settings to default values?')) {
                setApiKey('');
                setSelectedModel('claude-3-opus-20240229');
              }
            }}
          >
            Reset to Default
          </Button>
          
          <Button 
            variant="primary"
            onClick={handleSaveSettings}
          >
            Save Settings
          </Button>
        </div>
      </div>
    </div>
  );
};

export default Settings;
