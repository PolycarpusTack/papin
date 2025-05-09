// CollaborationSettings.tsx
//
// This component provides a UI for configuring collaboration settings.

import React, { useState } from 'react';
import { useCollaboration } from '../../../hooks/useCollaboration';
import { CollaborationConfig } from '../context/CollaborationContext';

interface CollaborationSettingsProps {
  // Optional props
}

const CollaborationSettings: React.FC<CollaborationSettingsProps> = () => {
  const { state, updateConfig } = useCollaboration();
  const { config } = state;
  
  const [localConfig, setLocalConfig] = useState<CollaborationConfig>(config);
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState<boolean>(false);
  
  // Handle input changes
  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target;
    
    setLocalConfig(prev => {
      // Handle different input types
      if (type === 'checkbox') {
        const checked = (e.target as HTMLInputElement).checked;
        return { ...prev, [name]: checked };
      } else if (type === 'number') {
        return { ...prev, [name]: parseInt(value, 10) };
      } else {
        return { ...prev, [name]: value };
      }
    });
    
    // Clear success message on changes
    setSaveSuccess(false);
  };
  
  // Handle saving the configuration
  const handleSave = async () => {
    setIsSaving(true);
    setError(null);
    setSaveSuccess(false);
    
    try {
      await updateConfig(localConfig);
      setSaveSuccess(true);
    } catch (err) {
      setError(`Failed to save settings: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };
  
  return (
    <div className="collaboration-settings">
      <h3 style={{ marginTop: 0 }}>Collaboration Settings</h3>
      
      {error && (
        <div className="collaboration-error">
          {error}
        </div>
      )}
      
      {saveSuccess && (
        <div style={{
          padding: '10px',
          backgroundColor: '#E8F5E9',
          color: '#2E7D32',
          borderRadius: '4px',
          marginBottom: '15px',
        }}>
          Settings saved successfully!
        </div>
      )}
      
      <div className="collaboration-form-group">
        <label className="collaboration-label">
          <input
            type="checkbox"
            name="enabled"
            checked={localConfig.enabled}
            onChange={handleChange}
            style={{ marginRight: '8px' }}
          />
          Enable collaboration features
        </label>
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label">
          <input
            type="checkbox"
            name="show_presence"
            checked={localConfig.show_presence}
            onChange={handleChange}
            style={{ marginRight: '8px' }}
          />
          Show user presence (cursors, selections)
        </label>
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label">
          <input
            type="checkbox"
            name="enable_av"
            checked={localConfig.enable_av}
            onChange={handleChange}
            style={{ marginRight: '8px' }}
          />
          Enable audio/video calls
        </label>
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label">
          <input
            type="checkbox"
            name="auto_discover"
            checked={localConfig.auto_discover}
            onChange={handleChange}
            style={{ marginRight: '8px' }}
          />
          Auto-discover other devices
        </label>
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label">
          <input
            type="checkbox"
            name="p2p_enabled"
            checked={localConfig.p2p_enabled}
            onChange={handleChange}
            style={{ marginRight: '8px' }}
          />
          Enable peer-to-peer mode
        </label>
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label" htmlFor="max_users">
          Maximum users per session
        </label>
        <input
          id="max_users"
          type="number"
          name="max_users_per_session"
          value={localConfig.max_users_per_session}
          onChange={handleChange}
          min={1}
          max={50}
          className="collaboration-input"
        />
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label" htmlFor="sync_interval">
          Sync interval (milliseconds)
        </label>
        <input
          id="sync_interval"
          type="number"
          name="sync_interval_ms"
          value={localConfig.sync_interval_ms}
          onChange={handleChange}
          min={100}
          max={10000}
          step={100}
          className="collaboration-input"
        />
      </div>
      
      <div className="collaboration-form-group">
        <label className="collaboration-label" htmlFor="username">
          Display name (optional)
        </label>
        <input
          id="username"
          type="text"
          name="username"
          value={localConfig.username || ''}
          onChange={handleChange}
          placeholder="Your name in collaborative sessions"
          className="collaboration-input"
        />
      </div>
      
      <div className="collaboration-button-group" style={{ marginTop: '20px' }}>
        <button
          onClick={handleSave}
          disabled={isSaving}
          className="collaboration-button primary"
        >
          {isSaving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
};

export default CollaborationSettings;
