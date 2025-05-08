import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './Settings.css';

interface AppInfo {
  name: string;
  version: string;
  platform: string;
}

// Settings component - lazy loaded when needed
const Settings: React.FC = () => {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [enabledFeatures, setEnabledFeatures] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const info = await invoke<AppInfo>('get_app_info');
        const features = await invoke<string[]>('get_enabled_features');
        
        setAppInfo(info);
        setEnabledFeatures(features);
        setLoading(false);
      } catch (err) {
        console.error('Failed to load settings:', err);
        setLoading(false);
      }
    };
    
    loadSettings();
  }, []);
  
  if (loading) {
    return (
      <div className="settings-loading">
        <div className="loading-spinner"></div>
        <p>Loading settings...</p>
      </div>
    );
  }
  
  return (
    <div className="settings-container fade-in">
      <h2>Settings</h2>
      
      <div className="settings-section">
        <h3>Application Info</h3>
        <div className="settings-info">
          <div className="info-item">
            <span className="info-label">Name:</span>
            <span className="info-value">{appInfo?.name}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Version:</span>
            <span className="info-value">{appInfo?.version}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Platform:</span>
            <span className="info-value">{appInfo?.platform}</span>
          </div>
        </div>
      </div>
      
      <div className="settings-section">
        <h3>Enabled Features</h3>
        <div className="feature-list">
          {enabledFeatures.map((feature) => (
            <div key={feature} className="feature-item">
              <div className="feature-checkbox checked"></div>
              <span className="feature-name">{feature.replace('_', ' ')}</span>
            </div>
          ))}
        </div>
      </div>
      
      <div className="settings-section">
        <h3>Theme</h3>
        <div className="theme-selector">
          <button className="theme-option active">
            <span className="theme-icon">🌙</span>
            <span className="theme-name">System Default</span>
          </button>
          <button className="theme-option">
            <span className="theme-icon">🌙</span>
            <span className="theme-name">Dark</span>
          </button>
          <button className="theme-option">
            <span className="theme-icon">☀️</span>
            <span className="theme-name">Light</span>
          </button>
        </div>
      </div>
      
      <div className="settings-section">
        <h3>API Settings</h3>
        <div className="form-group">
          <label>API Key</label>
          <input type="password" placeholder="Enter API Key" value="••••••••••••••••••••" />
        </div>
        
        <div className="form-group">
          <label>Model</label>
          <select>
            <option>claude-3-5-sonnet</option>
            <option>claude-3-opus</option>
            <option>claude-3-5-haiku</option>
          </select>
        </div>
      </div>
      
      <div className="settings-footer">
        <button className="settings-button primary">Save Changes</button>
        <button className="settings-button secondary">Reset to Default</button>
      </div>
    </div>
  );
};

export default Settings;