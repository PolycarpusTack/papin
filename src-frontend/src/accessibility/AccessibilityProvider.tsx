import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import './accessibility.css';

// Accessibility settings interface
interface AccessibilitySettings {
  highContrast: boolean;
  largeText: boolean;
  reducedMotion: boolean;
  screenReader: boolean;
  focusIndicators: boolean;
  dyslexicFont: boolean;
  textSpacing: boolean;
}

// Context interface
interface AccessibilityContextType {
  settings: AccessibilitySettings;
  updateSettings: (settings: Partial<AccessibilitySettings>) => void;
  resetSettings: () => void;
  showA11yPanel: boolean;
  toggleA11yPanel: () => void;
}

// Default settings
const defaultSettings: AccessibilitySettings = {
  highContrast: false,
  largeText: false,
  reducedMotion: false,
  screenReader: false,
  focusIndicators: true,
  dyslexicFont: false,
  textSpacing: false,
};

// Create context
const AccessibilityContext = createContext<AccessibilityContextType>({
  settings: defaultSettings,
  updateSettings: () => {},
  resetSettings: () => {},
  showA11yPanel: false,
  toggleA11yPanel: () => {},
});

export const useAccessibility = () => useContext(AccessibilityContext);

interface AccessibilityProviderProps {
  children: ReactNode;
}

export const AccessibilityProvider: React.FC<AccessibilityProviderProps> = ({ children }) => {
  // Load settings from localStorage or use defaults
  const getInitialSettings = (): AccessibilitySettings => {
    const savedSettings = localStorage.getItem('mcp-accessibility-settings');
    if (savedSettings) {
      try {
        return { ...defaultSettings, ...JSON.parse(savedSettings) };
      } catch (e) {
        console.error('Failed to parse accessibility settings:', e);
      }
    }
    return { ...defaultSettings };
  };

  const [settings, setSettings] = useState<AccessibilitySettings>(getInitialSettings);
  const [showA11yPanel, setShowA11yPanel] = useState<boolean>(false);

  // Update settings
  const updateSettings = (newSettings: Partial<AccessibilitySettings>) => {
    const updatedSettings = { ...settings, ...newSettings };
    setSettings(updatedSettings);
    localStorage.setItem('mcp-accessibility-settings', JSON.stringify(updatedSettings));
  };

  // Reset to defaults
  const resetSettings = () => {
    setSettings({ ...defaultSettings });
    localStorage.setItem('mcp-accessibility-settings', JSON.stringify(defaultSettings));
  };

  // Toggle accessibility panel
  const toggleA11yPanel = () => {
    setShowA11yPanel(prev => !prev);
  };

  // Apply settings to document
  useEffect(() => {
    const root = document.documentElement;
    
    // High contrast
    if (settings.highContrast) {
      root.classList.add('a11y-high-contrast');
    } else {
      root.classList.remove('a11y-high-contrast');
    }
    
    // Large text
    if (settings.largeText) {
      root.classList.add('a11y-large-text');
    } else {
      root.classList.remove('a11y-large-text');
    }
    
    // Reduced motion (this works alongside the AnimationProvider)
    if (settings.reducedMotion) {
      root.classList.add('a11y-reduced-motion');
    } else {
      root.classList.remove('a11y-reduced-motion');
    }
    
    // Screen reader - add ARIA role descriptions and enhanced keyboard support
    if (settings.screenReader) {
      root.classList.add('a11y-screen-reader');
    } else {
      root.classList.remove('a11y-screen-reader');
    }
    
    // Focus indicators - enhanced visibility for keyboard focus
    if (settings.focusIndicators) {
      root.classList.add('a11y-focus-indicators');
    } else {
      root.classList.remove('a11y-focus-indicators');
    }
    
    // Dyslexic-friendly font
    if (settings.dyslexicFont) {
      root.classList.add('a11y-dyslexic-font');
    } else {
      root.classList.remove('a11y-dyslexic-font');
    }
    
    // Text spacing
    if (settings.textSpacing) {
      root.classList.add('a11y-text-spacing');
    } else {
      root.classList.remove('a11y-text-spacing');
    }
  }, [settings]);

  // Check for OS/browser settings on initial load
  useEffect(() => {
    const checkOSSettings = () => {
      // Check prefers-reduced-motion
      const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      
      // Check prefers-contrast
      const prefersContrast = window.matchMedia('(prefers-contrast: more)').matches;
      
      // Apply OS settings if they're not already overridden by user preferences
      const savedSettings = localStorage.getItem('mcp-accessibility-settings');
      
      if (!savedSettings) {
        updateSettings({
          reducedMotion: prefersReducedMotion,
          highContrast: prefersContrast,
        });
      }
    };
    
    checkOSSettings();
    
    // Create keyboard shortcut to toggle accessibility panel (Alt+A)
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key === 'a') {
        e.preventDefault();
        toggleA11yPanel();
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  return (
    <AccessibilityContext.Provider
      value={{
        settings,
        updateSettings,
        resetSettings,
        showA11yPanel,
        toggleA11yPanel,
      }}
    >
      {children}
      {showA11yPanel && <AccessibilityPanel />}
    </AccessibilityContext.Provider>
  );
};

// Accessibility settings panel
const AccessibilityPanel: React.FC = () => {
  const { settings, updateSettings, resetSettings, toggleA11yPanel } = useAccessibility();

  return (
    <div className="a11y-panel-overlay" onClick={toggleA11yPanel}>
      <div className="a11y-panel" onClick={e => e.stopPropagation()}>
        <div className="a11y-panel-header">
          <h2>Accessibility Settings</h2>
          <button className="a11y-panel-close" onClick={toggleA11yPanel} aria-label="Close accessibility panel">
            &times;
          </button>
        </div>
        
        <div className="a11y-panel-content">
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.highContrast}
                onChange={e => updateSettings({ highContrast: e.target.checked })}
              />
              <span>High Contrast</span>
            </label>
            <p className="a11y-setting-description">Increases contrast for better readability</p>
          </div>
          
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.largeText}
                onChange={e => updateSettings({ largeText: e.target.checked })}
              />
              <span>Large Text</span>
            </label>
            <p className="a11y-setting-description">Increases font size throughout the application</p>
          </div>
          
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.reducedMotion}
                onChange={e => updateSettings({ reducedMotion: e.target.checked })}
              />
              <span>Reduced Motion</span>
            </label>
            <p className="a11y-setting-description">Minimizes animations and transitions</p>
          </div>
          
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.screenReader}
                onChange={e => updateSettings({ screenReader: e.target.checked })}
              />
              <span>Screen Reader Support</span>
            </label>
            <p className="a11y-setting-description">Enhances compatibility with screen readers</p>
          </div>
          
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.focusIndicators}
                onChange={e => updateSettings({ focusIndicators: e.target.checked })}
              />
              <span>Focus Indicators</span>
            </label>
            <p className="a11y-setting-description">Shows clear visual indicators when navigating with keyboard</p>
          </div>
          
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.dyslexicFont}
                onChange={e => updateSettings({ dyslexicFont: e.target.checked })}
              />
              <span>Dyslexic-Friendly Font</span>
            </label>
            <p className="a11y-setting-description">Uses a font designed to be easier to read with dyslexia</p>
          </div>
          
          <div className="a11y-setting">
            <label className="a11y-setting-label">
              <input
                type="checkbox"
                checked={settings.textSpacing}
                onChange={e => updateSettings({ textSpacing: e.target.checked })}
              />
              <span>Increased Text Spacing</span>
            </label>
            <p className="a11y-setting-description">Adds more space between letters, words, and lines</p>
          </div>
        </div>
        
        <div className="a11y-panel-footer">
          <button className="a11y-panel-reset" onClick={resetSettings}>
            Reset to Defaults
          </button>
          <p className="a11y-panel-shortcut">
            Press <kbd>Alt</kbd> + <kbd>A</kbd> to toggle this panel
          </p>
        </div>
      </div>
    </div>
  );
};

export default AccessibilityProvider;
