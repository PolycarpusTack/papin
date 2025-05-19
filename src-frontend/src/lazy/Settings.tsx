import React, { useState } from 'react';
import { Button } from '../components/ui/Button';
import { SettingsProvider } from '../contexts/SettingsContext';
import GeneralSettings from '../components/settings/GeneralSettings';
import AppearanceSettings from '../components/settings/AppearanceSettings';
import ModelSettings from '../components/settings/ModelSettings';
import OfflineSettings from '../components/settings/OfflineSettings';
import './Settings.css';

/**
 * Settings Panel Component
 */
const Settings: React.FC = () => {
  // Track active section
  const [activeSection, setActiveSection] = useState<string>('general');
  
  // Handle section change
  const changeSection = (section: string) => {
    setActiveSection(section);
    // Scroll to top when changing section
    window.scrollTo(0, 0);
  };
  
  return (
    <SettingsProvider>
      <div className="settings-container">
        <div className="settings-sidebar">
          <h1 className="settings-title">Settings</h1>
          
          <nav className="settings-navigation">
            <ul>
              <li>
                <Button
                  className={`settings-nav-button ${activeSection === 'general' ? 'active' : ''}`}
                  variant="text"
                  onClick={() => changeSection('general')}
                >
                  General
                </Button>
              </li>
              <li>
                <Button
                  className={`settings-nav-button ${activeSection === 'appearance' ? 'active' : ''}`}
                  variant="text"
                  onClick={() => changeSection('appearance')}
                >
                  Appearance
                </Button>
              </li>
              <li>
                <Button
                  className={`settings-nav-button ${activeSection === 'model' ? 'active' : ''}`}
                  variant="text"
                  onClick={() => changeSection('model')}
                >
                  Model Parameters
                </Button>
              </li>
              <li>
                <Button
                  className={`settings-nav-button ${activeSection === 'offline' ? 'active' : ''}`}
                  variant="text"
                  onClick={() => changeSection('offline')}
                >
                  Offline Mode
                </Button>
              </li>
            </ul>
          </nav>
          
          <div className="settings-sidebar-footer">
            <Button
              variant="text"
              onClick={() => {
                if (window.confirm('Are you sure you want to reset all settings to default values?')) {
                  // Reset settings will be handled by the SettingsProvider
                  alert('Settings reset to defaults');
                }
              }}
            >
              Reset to Defaults
            </Button>
          </div>
        </div>
        
        <div className="settings-content">
          <div className="settings-panel">
            {activeSection === 'general' && <GeneralSettings />}
            {activeSection === 'appearance' && <AppearanceSettings />}
            {activeSection === 'model' && <ModelSettings />}
            {activeSection === 'offline' && <OfflineSettings />}
          </div>
          
          <div className="settings-actions">
            <Button
              variant="primary"
              className="save-settings-button"
              onClick={() => {
                // Save settings will be handled by the SettingsProvider
                alert('Settings saved!');
              }}
            >
              Save All Settings
            </Button>
          </div>
        </div>
      </div>
    </SettingsProvider>
  );
};

export default Settings;