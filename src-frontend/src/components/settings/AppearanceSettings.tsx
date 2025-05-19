import React from 'react';
import { useSettings } from '../../contexts/SettingsContext';
import { ThemeMode } from '../../api/SettingsApi';
import './SettingsSections.css';

/**
 * Appearance settings component
 */
const AppearanceSettings: React.FC = () => {
  const { 
    settings, 
    updateUiSettings,
    setTheme,
    theme
  } = useSettings();
  
  // Handle theme change
  const handleThemeChange = (newTheme: ThemeMode) => {
    setTheme(newTheme);
  };
  
  // Handle font size change
  const handleFontSizeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const fontSize = parseInt(e.target.value, 10);
    updateUiSettings({ fontSize: fontSize as 12 | 14 | 16 | 18 | 20 });
  };
  
  // Handle animation toggle
  const handleAnimationsToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateUiSettings({ animations: e.target.checked });
  };
  
  // Handle high contrast toggle
  const handleHighContrastToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateUiSettings({ highContrast: e.target.checked });
  };
  
  // Handle reduced motion toggle
  const handleReducedMotionToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateUiSettings({ reducedMotion: e.target.checked });
  };
  
  return (
    <div className="settings-section">
      <h2>Appearance</h2>
      
      <div className="settings-form">
        {/* Theme Selection */}
        <div className="form-group">
          <label className="form-label">Theme</label>
          <div className="theme-options">
            <div
              className={`theme-option ${theme === 'light' ? 'active' : ''}`}
              onClick={() => handleThemeChange('light')}
            >
              <div className="theme-preview light-theme">
                <div className="theme-preview-header"></div>
                <div className="theme-preview-content"></div>
              </div>
              <span>Light</span>
            </div>
            
            <div
              className={`theme-option ${theme === 'dark' ? 'active' : ''}`}
              onClick={() => handleThemeChange('dark')}
            >
              <div className="theme-preview dark-theme">
                <div className="theme-preview-header"></div>
                <div className="theme-preview-content"></div>
              </div>
              <span>Dark</span>
            </div>
            
            <div
              className={`theme-option ${theme === 'system' ? 'active' : ''}`}
              onClick={() => handleThemeChange('system')}
            >
              <div className="theme-preview system-theme">
                <div className="theme-preview-header"></div>
                <div className="theme-preview-content"></div>
                <div className="system-theme-icon">OS</div>
              </div>
              <span>System</span>
            </div>
          </div>
        </div>
        
        {/* Font Size */}
        <div className="form-group">
          <label className="form-label">Font Size</label>
          <select
            className="select-input"
            value={settings.ui.fontSize}
            onChange={handleFontSizeChange}
          >
            <option value="12">Small (12px)</option>
            <option value="14">Normal (14px)</option>
            <option value="16">Medium (16px)</option>
            <option value="18">Large (18px)</option>
            <option value="20">Extra Large (20px)</option>
          </select>
          <p className="form-hint">
            Controls text size throughout the application.
          </p>
        </div>
        
        {/* Animations */}
        <div className="form-group checkbox-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.ui.animations}
              onChange={handleAnimationsToggle}
            />
            <span>Enable Animations</span>
          </label>
          <p className="form-hint">
            Turn on/off all animations throughout the application.
          </p>
        </div>
        
        {/* High Contrast */}
        <div className="form-group checkbox-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.ui.highContrast}
              onChange={handleHighContrastToggle}
            />
            <span>High Contrast</span>
          </label>
          <p className="form-hint">
            Increases contrast for better readability.
          </p>
        </div>
        
        {/* Reduced Motion */}
        <div className="form-group checkbox-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.ui.reducedMotion}
              onChange={handleReducedMotionToggle}
            />
            <span>Reduced Motion</span>
          </label>
          <p className="form-hint">
            Minimizes animations and transitions for accessibility.
          </p>
        </div>
      </div>
    </div>
  );
};

export default AppearanceSettings;