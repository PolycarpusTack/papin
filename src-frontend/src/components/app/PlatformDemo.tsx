// src-frontend/src/components/app/PlatformDemo.tsx
//
// Demo component showing platform-specific features

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { usePlatform } from '../../hooks/usePlatform';
import { useTheme } from '../theme/ThemeProvider';
import { 
  getPlatformInfo, 
  showNotification, 
  registerShortcuts, 
  minimizeWindow, 
  maximizeWindow, 
  closeWindow,
  usePlatformUi,
  type PlatformInfo,
  initializePlatform
} from '../../platform/integration';
import { WindowControls } from '../common/WindowControls';
import { KeyboardShortcut } from '../common/KeyboardShortcut';

/**
 * Demo component showing platform-specific features
 */
export function PlatformDemo(): JSX.Element {
  const { platform, isLoading: platformLoading } = usePlatform();
  const { theme, mode, setMode } = useTheme();
  const platformUi = usePlatformUi();
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Effect to load platform info
  useEffect(() => {
    const loadPlatformInfo = async () => {
      try {
        setIsLoading(true);
        
        // Initialize platform
        await initializePlatform();
        
        // Get platform info
        const info = await getPlatformInfo();
        setPlatformInfo(info);
        
        // Register shortcuts
        await registerShortcuts({
          'toggle-offline': () => {
            console.log('Toggle offline mode shortcut triggered');
            showNotification('Offline Mode', 'Toggling offline mode...', { category: 'offline' });
          },
          'force-sync': () => {
            console.log('Force sync shortcut triggered');
            showNotification('Sync', 'Forcing sync...', { category: 'sync' });
          },
          'toggle-theme': () => {
            // Toggle theme
            setMode(mode === 'light' ? 'dark' : mode === 'dark' ? 'system' : 'light');
            showNotification('Theme', `Switched to ${mode === 'light' ? 'dark' : mode === 'dark' ? 'system' : 'light'} theme`);
          },
        });
        
        setError(null);
      } catch (e) {
        console.error('Failed to load platform info:', e);
        setError(`Failed to load platform info: ${e}`);
      } finally {
        setIsLoading(false);
      }
    };
    
    loadPlatformInfo();
  }, []);
  
  // Function to send a test notification
  const sendTestNotification = async () => {
    await showNotification(
      'Platform Test',
      `This is a test notification on ${platform}`,
      { urgency: 'normal' }
    );
  };
  
  // Render loading state
  if (isLoading || platformLoading) {
    return (
      <div className="platform-demo loading">
        <div className="loading-spinner"></div>
        <div className="loading-text">Loading platform information...</div>
      </div>
    );
  }
  
  // Render error state
  if (error) {
    return (
      <div className="platform-demo error">
        <div className="error-icon">❌</div>
        <div className="error-message">{error}</div>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }
  
  return (
    <div className={`platform-demo ${platformUi.cssClasses.join(' ')}`}>
      <div className="demo-titlebar">
        <div className="title">Platform Demo - {platform}</div>
        <WindowControls 
          position={platformUi.windowControlsPosition}
          showMinimize
          showMaximize
          showClose
        />
      </div>
      
      <div className="demo-content">
        <div className="demo-section">
          <h2>Platform Information</h2>
          {platformInfo && (
            <div className="platform-info">
              <div className="info-row">
                <div className="info-label">Platform:</div>
                <div className="info-value">{platformInfo.platform}</div>
              </div>
              <div className="info-row">
                <div className="info-label">Version:</div>
                <div className="info-value">{platformInfo.version}</div>
              </div>
              <div className="info-row">
                <div className="info-label">Architecture:</div>
                <div className="info-value">{platformInfo.architecture}</div>
              </div>
              <div className="info-details">
                <h3>Additional Details</h3>
                <ul>
                  {Object.entries(platformInfo.details).map(([key, value]) => (
                    <li key={key}>
                      <span className="detail-key">{key}:</span> {value}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          )}
        </div>
        
        <div className="demo-section">
          <h2>Platform-Specific UI</h2>
          <div className="ui-elements">
            <div className="ui-element">
              <div className="element-label">Button Style:</div>
              <button className="platform-button">Platform Button</button>
            </div>
            <div className="ui-element">
              <div className="element-label">Input Style:</div>
              <input type="text" className="platform-input" placeholder="Platform Input" />
            </div>
            <div className="ui-element">
              <div className="element-label">Dropdown Style:</div>
              <select className="platform-select">
                <option>Option 1</option>
                <option>Option 2</option>
                <option>Option 3</option>
              </select>
            </div>
            <div className="ui-element">
              <div className="element-label">Checkbox Style:</div>
              <div className="checkbox-group">
                <input type="checkbox" id="checkbox1" className="platform-checkbox" />
                <label htmlFor="checkbox1">Platform Checkbox</label>
              </div>
            </div>
          </div>
        </div>
        
        <div className="demo-section">
          <h2>Keyboard Shortcuts</h2>
          <div className="shortcuts-list">
            <div className="shortcut-item">
              <div className="shortcut-label">Toggle Offline Mode:</div>
              <KeyboardShortcut shortcut={platformUi.shortcuts['toggle-offline']} />
            </div>
            <div className="shortcut-item">
              <div className="shortcut-label">Force Sync:</div>
              <KeyboardShortcut shortcut={platformUi.shortcuts['force-sync']} />
            </div>
            <div className="shortcut-item">
              <div className="shortcut-label">Create Checkpoint:</div>
              <KeyboardShortcut shortcut={platformUi.shortcuts['create-checkpoint']} />
            </div>
            <div className="shortcut-item">
              <div className="shortcut-label">Open Settings:</div>
              <KeyboardShortcut shortcut={platformUi.shortcuts['open-settings']} />
            </div>
            <div className="shortcut-item">
              <div className="shortcut-label">Toggle Theme:</div>
              <KeyboardShortcut shortcut={platformUi.shortcuts['toggle-theme']} />
            </div>
            <div className="shortcut-item">
              <div className="shortcut-label">Quit Application:</div>
              <KeyboardShortcut shortcut={platformUi.shortcuts['quit-app']} />
            </div>
          </div>
        </div>
        
        <div className="demo-section">
          <h2>Notifications</h2>
          <div className="notification-demo">
            <p>
              Test platform-specific notifications with different urgency levels:
            </p>
            <div className="notification-buttons">
              <button 
                className="notification-button low"
                onClick={() => showNotification(
                  'Low Urgency',
                  'This is a low urgency notification',
                  { urgency: 'low' }
                )}
              >
                Low Urgency
              </button>
              <button 
                className="notification-button normal"
                onClick={() => showNotification(
                  'Normal Urgency',
                  'This is a normal urgency notification',
                  { urgency: 'normal' }
                )}
              >
                Normal Urgency
              </button>
              <button 
                className="notification-button critical"
                onClick={() => showNotification(
                  'Critical Urgency',
                  'This is a critical urgency notification',
                  { urgency: 'critical' }
                )}
              >
                Critical Urgency
              </button>
            </div>
            <div className="notification-categories">
              <p>Test notifications in different categories:</p>
              <div className="notification-buttons">
                <button 
                  className="notification-button"
                  onClick={() => showNotification(
                    'System Notification',
                    'This is a system notification',
                    { category: 'system' }
                  )}
                >
                  System
                </button>
                <button 
                  className="notification-button"
                  onClick={() => showNotification(
                    'Network Notification',
                    'This is a network notification',
                    { category: 'network' }
                  )}
                >
                  Network
                </button>
                <button 
                  className="notification-button"
                  onClick={() => showNotification(
                    'Offline Notification',
                    'This is an offline notification',
                    { category: 'offline' }
                  )}
                >
                  Offline
                </button>
                <button 
                  className="notification-button"
                  onClick={() => showNotification(
                    'Error Notification',
                    'This is an error notification',
                    { category: 'error', urgency: 'critical' }
                  )}
                >
                  Error
                </button>
              </div>
            </div>
          </div>
        </div>
        
        <div className="demo-section">
          <h2>Window Controls</h2>
          <div className="window-control-demo">
            <p>Test window control operations:</p>
            <div className="window-control-buttons">
              <button onClick={() => minimizeWindow()}>Minimize</button>
              <button onClick={() => maximizeWindow()}>Maximize/Restore</button>
              <button onClick={() => closeWindow()}>Close</button>
            </div>
          </div>
        </div>
        
        <div className="demo-section">
          <h2>Theme Management</h2>
          <div className="theme-demo">
            <p>Current theme mode: <strong>{mode}</strong></p>
            <div className="theme-buttons">
              <button
                className={`theme-button ${mode === 'light' ? 'active' : ''}`}
                onClick={() => setMode('light')}
              >
                Light
              </button>
              <button
                className={`theme-button ${mode === 'dark' ? 'active' : ''}`}
                onClick={() => setMode('dark')}
              >
                Dark
              </button>
              <button
                className={`theme-button ${mode === 'system' ? 'active' : ''}`}
                onClick={() => setMode('system')}
              >
                System
              </button>
            </div>
            <div className="theme-preview">
              <div className="theme-sample light">
                <div className="sample-header">Light Theme Sample</div>
                <div className="sample-content">
                  <div className="sample-text">Sample text in light theme</div>
                  <button className="sample-button">Button</button>
                </div>
              </div>
              <div className="theme-sample dark">
                <div className="sample-header">Dark Theme Sample</div>
                <div className="sample-content">
                  <div className="sample-text">Sample text in dark theme</div>
                  <button className="sample-button">Button</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * CSS for the PlatformDemo component
 */
export const PlatformDemoStyles = `
.platform-demo {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background-color: var(--background-color);
  color: var(--text-color);
}

.demo-titlebar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--primary-color);
  color: white;
  padding: 8px 16px;
  -webkit-app-region: drag;
  user-select: none;
}

.title {
  font-weight: bold;
  font-size: 16px;
}

.demo-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.demo-section {
  background-color: var(--surface-color);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow-small);
  overflow: hidden;
}

.demo-section h2 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 18px;
  color: var(--primary-color);
  border-bottom: 1px solid var(--border-color);
  padding-bottom: 10px;
}

/* Platform information */
.platform-info {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-row {
  display: flex;
  margin-bottom: 8px;
}

.info-label {
  flex: 0 0 120px;
  font-weight: bold;
}

.info-value {
  flex: 1;
}

.info-details h3 {
  font-size: 14px;
  margin-top: 15px;
  margin-bottom: 10px;
}

.info-details ul {
  margin: 0;
  padding-left: 20px;
}

.detail-key {
  font-weight: bold;
}

/* UI Elements */
.ui-elements {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 20px;
}

.ui-element {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.element-label {
  font-weight: bold;
}

.platform-button {
  padding: 8px 16px;
  background-color: var(--primary-color);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: background-color 0.2s;
}

.platform-button:hover {
  background-color: var(--secondary-color);
}

.platform-input {
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  width: 100%;
}

.platform-select {
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background-color: var(--surface-color);
  color: var(--text-color);
  width: 100%;
}

.checkbox-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.platform-checkbox {
  width: 18px;
  height: 18px;
}

/* Keyboard shortcuts */
.shortcuts-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 10px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background-color: rgba(0, 0, 0, 0.03);
  border-radius: var(--border-radius);
}

.shortcut-label {
  font-weight: bold;
}

/* Notifications */
.notification-demo {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.notification-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.notification-button {
  padding: 8px 16px;
  border: none;
  border-radius: var(--border-radius);
  cursor: pointer;
  background-color: var(--primary-color);
  color: white;
}

.notification-button.low {
  background-color: #3498db;
}

.notification-button.normal {
  background-color: #2ecc71;
}

.notification-button.critical {
  background-color: #e74c3c;
}

.notification-categories {
  margin-top: 10px;
}

/* Window controls */
.window-control-demo {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.window-control-buttons {
  display: flex;
  gap: 10px;
}

.window-control-buttons button {
  padding: 8px 16px;
  border: none;
  border-radius: var(--border-radius);
  cursor: pointer;
  background-color: var(--primary-color);
  color: white;
}

/* Theme management */
.theme-demo {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.theme-buttons {
  display: flex;
  gap: 10px;
}

.theme-button {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  cursor: pointer;
  background-color: var(--surface-color);
  color: var(--text-color);
}

.theme-button.active {
  background-color: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.theme-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  margin-top: 15px;
}

.theme-sample {
  flex: 1;
  min-width: 250px;
  border-radius: var(--border-radius);
  overflow: hidden;
}

.theme-sample.light {
  background-color: #f8f9fa;
  color: #212529;
  border: 1px solid #dee2e6;
}

.theme-sample.dark {
  background-color: #212529;
  color: #f8f9fa;
  border: 1px solid #495057;
}

.sample-header {
  background-color: #007bff;
  color: white;
  padding: 10px;
  font-weight: bold;
}

.theme-sample.dark .sample-header {
  background-color: #0069d9;
}

.sample-content {
  padding: 15px;
}

.sample-text {
  margin-bottom: 10px;
}

.sample-button {
  padding: 5px 10px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.theme-sample.light .sample-button {
  background-color: #007bff;
  color: white;
}

.theme-sample.dark .sample-button {
  background-color: #0069d9;
  color: white;
}

/* Loading state */
.platform-demo.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(0, 0, 0, 0.1);
  border-left-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  font-size: 16px;
  color: var(--text-color);
}

/* Error state */
.platform-demo.error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  padding: 20px;
  text-align: center;
}

.error-icon {
  font-size: 48px;
  margin-bottom: 20px;
  color: #e74c3c;
}

.error-message {
  font-size: 16px;
  color: var(--text-color);
  margin-bottom: 20px;
  max-width: 500px;
}

.platform-demo.error button {
  padding: 8px 16px;
  background-color: var(--primary-color);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  cursor: pointer;
}

/* Platform-specific styles */

/* Windows */
.platform-windows .platform-button {
  border-radius: 2px;
}

.platform-windows .platform-input,
.platform-windows .platform-select {
  border-radius: 2px;
}

.platform-windows .demo-section {
  border-radius: 2px;
  border: 1px solid var(--border-color);
}

/* macOS */
.platform-macos .platform-button {
  border-radius: 6px;
}

.platform-macos .platform-input,
.platform-macos .platform-select {
  border-radius: 6px;
}

.platform-macos .demo-section {
  border-radius: 10px;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.05), 0 1px 3px rgba(0, 0, 0, 0.1);
}

.platform-macos .demo-titlebar {
  background: linear-gradient(to bottom, #f0f0f0, #e0e0e0);
  color: black;
}

/* Linux */
.platform-linux .platform-button,
.platform-linux .platform-input,
.platform-linux .platform-select,
.platform-linux .demo-section,
.platform-linux .notification-button,
.platform-linux .window-control-buttons button,
.platform-linux .theme-button,
.platform-linux .shortcut-item {
  border-radius: 0;
}

.platform-linux .demo-titlebar {
  background-color: #574b90;
}
`;
