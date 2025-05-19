import React from 'react';
import { useSettings } from '../../contexts/SettingsContext';
import './SettingsSections.css';

/**
 * Model settings component
 */
const ModelSettings: React.FC = () => {
  const { 
    settings, 
    updateModelSettings 
  } = useSettings();
  
  // Handle temperature change
  const handleTemperatureChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    updateModelSettings({ temperature: value });
  };
  
  // Handle max tokens change
  const handleMaxTokensChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value, 10);
    updateModelSettings({ maxTokens: value });
  };
  
  // Handle system prompt change
  const handleSystemPromptChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    updateModelSettings({ systemPrompt: e.target.value || undefined });
  };
  
  // Handle streaming toggle
  const handleStreamingToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateModelSettings({ streaming: e.target.checked });
  };
  
  return (
    <div className="settings-section">
      <h2>Model Parameters</h2>
      
      <div className="settings-form">
        {/* Temperature */}
        <div className="form-group">
          <label className="form-label">Temperature: {settings.model.temperature.toFixed(1)}</label>
          <div className="range-control">
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={settings.model.temperature}
              onChange={handleTemperatureChange}
              className="range-input"
            />
          </div>
          <div className="range-labels">
            <span>More Deterministic</span>
            <span>More Random</span>
          </div>
          <p className="form-hint">
            Controls randomness in responses. Lower values are more deterministic.
          </p>
        </div>
        
        {/* Max Tokens */}
        <div className="form-group">
          <label className="form-label">Maximum Tokens: {settings.model.maxTokens}</label>
          <div className="range-control">
            <input
              type="range"
              min="1000"
              max="100000"
              step="1000"
              value={settings.model.maxTokens}
              onChange={handleMaxTokensChange}
              className="range-input"
            />
          </div>
          <div className="range-labels">
            <span>Shorter (1K)</span>
            <span>Longer (100K)</span>
          </div>
          <p className="form-hint">
            Maximum number of tokens for model responses.
          </p>
        </div>
        
        {/* System Prompt */}
        <div className="form-group">
          <label className="form-label">Default System Prompt</label>
          <textarea
            className="textarea-input"
            rows={4}
            value={settings.model.systemPrompt || ''}
            onChange={handleSystemPromptChange}
            placeholder="Enter a system prompt that will be used for all conversations"
          />
          <p className="form-hint">
            Custom system prompt to control model behavior. Leave empty for default.
          </p>
        </div>
        
        {/* Streaming */}
        <div className="form-group checkbox-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.model.streaming}
              onChange={handleStreamingToggle}
            />
            <span>Enable Streaming</span>
          </label>
          <p className="form-hint">
            When enabled, responses will appear as they are generated rather than all at once.
          </p>
        </div>
      </div>
    </div>
  );
};

export default ModelSettings;