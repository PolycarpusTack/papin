import { invoke } from '@tauri-apps/api/tauri';

/**
 * UI theme options
 */
export type ThemeMode = 'light' | 'dark' | 'system';

/**
 * Font size options (in pixels)
 */
export type FontSize = 12 | 14 | 16 | 18 | 20;

/**
 * API settings interface
 */
export interface ApiSettings {
  url: string;
  model: string;
  version: string;
}

/**
 * UI settings interface
 */
export interface UiSettings {
  darkMode: boolean;
  fontSize: FontSize;
  animations: boolean;
  systemTheme: boolean;
  highContrast: boolean;
  reducedMotion: boolean;
}

/**
 * Model settings interface
 */
export interface ModelSettings {
  temperature: number;
  maxTokens: number;
  systemPrompt?: string;
  streaming: boolean;
}

/**
 * Offline settings interface
 */
export interface OfflineSettings {
  enabled: boolean;
  autoSync: boolean;
  downloadedModels: string[];
  maxDiskSpace: number; // in MB
}

/**
 * Complete settings interface
 */
export interface Settings {
  api: ApiSettings;
  ui: UiSettings;
  model: ModelSettings;
  offline: OfflineSettings;
}

/**
 * API for settings-related operations
 */
class SettingsApi {
  /**
   * Get current settings
   */
  async getSettings(): Promise<Settings> {
    try {
      const settings = await invoke<any>('get_settings');
      
      // Transform backend settings to frontend format
      return {
        api: {
          url: settings.api.url,
          model: settings.api.model,
          version: settings.api.version,
        },
        ui: {
          darkMode: settings.ui.dark_mode,
          fontSize: settings.ui.font_size,
          animations: settings.ui.animations,
          systemTheme: settings.ui.system_theme,
          highContrast: settings.ui.high_contrast || false,
          reducedMotion: settings.ui.reduced_motion || false,
        },
        model: {
          temperature: settings.model.temperature,
          maxTokens: settings.model.max_tokens,
          systemPrompt: settings.model.system_prompt,
          streaming: settings.model.streaming,
        },
        offline: {
          enabled: settings.offline?.enabled || false,
          autoSync: settings.offline?.auto_sync || true,
          downloadedModels: settings.offline?.downloaded_models || [],
          maxDiskSpace: settings.offline?.max_disk_space || 5000, // 5GB default
        }
      };
    } catch (error) {
      console.error('Failed to get settings:', error);
      throw error;
    }
  }
  
  /**
   * Save settings
   */
  async saveSettings(settings: Settings): Promise<void> {
    try {
      // Transform frontend settings to backend format
      const backendSettings = {
        api: {
          url: settings.api.url,
          model: settings.api.model,
          version: settings.api.version,
        },
        ui: {
          dark_mode: settings.ui.darkMode,
          font_size: settings.ui.fontSize,
          animations: settings.ui.animations,
          system_theme: settings.ui.systemTheme,
          high_contrast: settings.ui.highContrast,
          reduced_motion: settings.ui.reducedMotion,
        },
        model: {
          temperature: settings.model.temperature,
          max_tokens: settings.model.maxTokens,
          system_prompt: settings.model.systemPrompt,
          streaming: settings.model.streaming,
        },
        offline: {
          enabled: settings.offline.enabled,
          auto_sync: settings.offline.autoSync,
          downloaded_models: settings.offline.downloadedModels,
          max_disk_space: settings.offline.maxDiskSpace,
        }
      };
      
      await invoke('save_settings', { settings: backendSettings });
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    }
  }
  
  /**
   * Set API key (this will be stored encrypted)
   */
  async setApiKey(apiKey: string): Promise<void> {
    try {
      await invoke('set_api_key', { apiKey });
    } catch (error) {
      console.error('Failed to set API key:', error);
      throw error;
    }
  }
  
  /**
   * Check if an API key is set
   */
  async hasApiKey(): Promise<boolean> {
    try {
      return await invoke<boolean>('has_api_key');
    } catch (error) {
      console.error('Failed to check API key:', error);
      throw error;
    }
  }
  
  /**
   * Reset settings to defaults
   */
  async resetSettings(): Promise<Settings> {
    try {
      await invoke('reset_settings');
      return this.getSettings();
    } catch (error) {
      console.error('Failed to reset settings:', error);
      throw error;
    }
  }
  
  /**
   * Get available models
   */
  async getAvailableModels(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_available_models');
    } catch (error) {
      console.error('Failed to get available models:', error);
      throw error;
    }
  }
}

export default new SettingsApi();