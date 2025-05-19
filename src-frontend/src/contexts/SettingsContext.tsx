import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import SettingsApi, { 
  Settings, 
  ApiSettings, 
  UiSettings, 
  ModelSettings, 
  OfflineSettings,
  ThemeMode 
} from '../api/SettingsApi';

interface SettingsContextType {
  // Full settings object
  settings: Settings;
  
  // Loading state
  loading: boolean;
  
  // Error state
  error: string | null;
  
  // Setters for individual sections
  updateApiSettings: (settings: Partial<ApiSettings>) => Promise<void>;
  updateUiSettings: (settings: Partial<UiSettings>) => Promise<void>;
  updateModelSettings: (settings: Partial<ModelSettings>) => Promise<void>;
  updateOfflineSettings: (settings: Partial<OfflineSettings>) => Promise<void>;
  
  // API key methods
  setApiKey: (apiKey: string) => Promise<void>;
  hasApiKey: () => Promise<boolean>;
  
  // Theme helpers
  setTheme: (theme: ThemeMode) => Promise<void>;
  theme: ThemeMode;
  
  // Reset settings
  resetSettings: () => Promise<void>;
  
  // Save all settings
  saveAllSettings: () => Promise<void>;
  
  // Settings changed flag
  hasUnsavedChanges: boolean;
}

// Default settings
const defaultSettings: Settings = {
  api: {
    url: 'wss://api.anthropic.com/v1/messages',
    model: 'claude-3-sonnet-20240229',
    version: 'v1',
  },
  ui: {
    darkMode: false,
    fontSize: 14,
    animations: true,
    systemTheme: true,
    highContrast: false,
    reducedMotion: false,
  },
  model: {
    temperature: 0.7,
    maxTokens: 4096,
    streaming: true,
  },
  offline: {
    enabled: false,
    autoSync: true,
    downloadedModels: [],
    maxDiskSpace: 5000, // 5GB default
  }
};

// Create context with default values
const SettingsContext = createContext<SettingsContextType>({
  settings: defaultSettings,
  loading: true,
  error: null,
  updateApiSettings: async () => {},
  updateUiSettings: async () => {},
  updateModelSettings: async () => {},
  updateOfflineSettings: async () => {},
  setApiKey: async () => {},
  hasApiKey: async () => false,
  setTheme: async () => {},
  theme: 'system',
  resetSettings: async () => {},
  saveAllSettings: async () => {},
  hasUnsavedChanges: false,
});

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [originalSettings, setOriginalSettings] = useState<Settings>(defaultSettings);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  
  // Calculate the current theme from settings
  const theme = settings.ui.systemTheme ? 'system' : (settings.ui.darkMode ? 'dark' : 'light');
  
  // Load settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        setLoading(true);
        const loadedSettings = await SettingsApi.getSettings();
        setSettings(loadedSettings);
        setOriginalSettings(JSON.parse(JSON.stringify(loadedSettings))); // Deep copy
        setError(null);
      } catch (err) {
        console.error('Failed to load settings:', err);
        setError('Failed to load settings. Using defaults.');
        // Use defaults if loading fails
        setSettings(defaultSettings);
        setOriginalSettings(JSON.parse(JSON.stringify(defaultSettings)));
      } finally {
        setLoading(false);
      }
    };
    
    loadSettings();
  }, []);
  
  // Check for unsaved changes whenever settings change
  useEffect(() => {
    if (!loading) {
      const hasChanges = JSON.stringify(settings) !== JSON.stringify(originalSettings);
      setHasUnsavedChanges(hasChanges);
    }
  }, [settings, originalSettings, loading]);
  
  // Update API settings
  const updateApiSettings = async (newSettings: Partial<ApiSettings>) => {
    setSettings(prev => ({
      ...prev,
      api: {
        ...prev.api,
        ...newSettings,
      }
    }));
  };
  
  // Update UI settings
  const updateUiSettings = async (newSettings: Partial<UiSettings>) => {
    setSettings(prev => ({
      ...prev,
      ui: {
        ...prev.ui,
        ...newSettings,
      }
    }));
  };
  
  // Update model settings
  const updateModelSettings = async (newSettings: Partial<ModelSettings>) => {
    setSettings(prev => ({
      ...prev,
      model: {
        ...prev.model,
        ...newSettings,
      }
    }));
  };
  
  // Update offline settings
  const updateOfflineSettings = async (newSettings: Partial<OfflineSettings>) => {
    setSettings(prev => ({
      ...prev,
      offline: {
        ...prev.offline,
        ...newSettings,
      }
    }));
  };
  
  // Set API key
  const setApiKey = async (apiKey: string) => {
    try {
      await SettingsApi.setApiKey(apiKey);
    } catch (err) {
      console.error('Failed to set API key:', err);
      setError('Failed to set API key');
      throw err;
    }
  };
  
  // Check if API key is set
  const hasApiKey = async (): Promise<boolean> => {
    try {
      return await SettingsApi.hasApiKey();
    } catch (err) {
      console.error('Failed to check API key:', err);
      setError('Failed to check API key');
      return false;
    }
  };
  
  // Set theme (convenience method that updates UI settings)
  const setTheme = async (theme: ThemeMode) => {
    if (theme === 'system') {
      // Set to system theme
      updateUiSettings({ systemTheme: true });
    } else {
      // Set to specific theme
      updateUiSettings({ 
        systemTheme: false,
        darkMode: theme === 'dark'
      });
    }
  };
  
  // Reset settings to defaults
  const resetSettings = async () => {
    try {
      setLoading(true);
      const resetSettings = await SettingsApi.resetSettings();
      setSettings(resetSettings);
      setOriginalSettings(JSON.parse(JSON.stringify(resetSettings)));
      setError(null);
    } catch (err) {
      console.error('Failed to reset settings:', err);
      setError('Failed to reset settings');
    } finally {
      setLoading(false);
    }
  };
  
  // Save all settings
  const saveAllSettings = async () => {
    try {
      setLoading(true);
      await SettingsApi.saveSettings(settings);
      setOriginalSettings(JSON.parse(JSON.stringify(settings))); // Update original settings after save
      setError(null);
    } catch (err) {
      console.error('Failed to save settings:', err);
      setError('Failed to save settings');
      throw err;
    } finally {
      setLoading(false);
    }
  };
  
  const value = {
    settings,
    loading,
    error,
    updateApiSettings,
    updateUiSettings,
    updateModelSettings,
    updateOfflineSettings,
    setApiKey,
    hasApiKey,
    setTheme,
    theme,
    resetSettings,
    saveAllSettings,
    hasUnsavedChanges,
  };
  
  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
}

// Hook for using settings
export function useSettings() {
  const context = useContext(SettingsContext);
  if (context === undefined) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
}

export default SettingsContext;