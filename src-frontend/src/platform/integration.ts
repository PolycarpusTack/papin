// src-frontend/src/platform/integration.ts
//
// Platform-specific integration for frontend

import { invoke } from '@tauri-apps/api/tauri';
import { appWindow, LogicalSize, PhysicalSize, WebviewWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/api/dialog';
import { arch, platform, type, version } from '@tauri-apps/api/os';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { notify, isPermissionGranted, requestPermission } from '@tauri-apps/api/notification';
import { usePlatform, type Platform } from '../hooks/usePlatform';

/**
 * Platform specific information
 */
export interface PlatformInfo {
  platform: Platform;
  version: string;
  architecture: string;
  details: Record<string, string>;
}

/**
 * Notification urgency
 */
export type NotificationUrgency = 'low' | 'normal' | 'critical';

/**
 * Notification category
 */
export type NotificationCategory = 'general' | 'system' | 'network' | 'offline' | 'sync' | 'error';

/**
 * Keyboard shortcut identifiers
 */
export type ShortcutId =
  | 'toggle-offline'
  | 'force-sync'
  | 'create-checkpoint'
  | 'open-settings'
  | 'toggle-theme'
  | 'reload-app'
  | 'quit-app';

/**
 * Get platform information
 */
export async function getPlatformInfo(): Promise<PlatformInfo> {
  try {
    // Try to get platform info from our Rust command
    return await invoke<PlatformInfo>('get_platform_info');
  } catch (e) {
    console.error('Failed to get platform info from Rust command:', e);
    
    // Fallback to Tauri OS API
    const platformName = await platform();
    const versionStr = await version();
    const archStr = await arch();
    const typeStr = await type();
    
    // Map platform to our Platform type
    let mappedPlatform: Platform = 'unknown';
    
    if (platformName === 'win32') {
      mappedPlatform = 'windows';
    } else if (platformName === 'darwin') {
      mappedPlatform = 'macos';
    } else if (platformName === 'linux') {
      mappedPlatform = 'linux';
    }
    
    return {
      platform: mappedPlatform,
      version: versionStr,
      architecture: archStr,
      details: {
        type: typeStr,
      },
    };
  }
}

/**
 * Show a platform notification
 */
export async function showNotification(
  title: string,
  body: string,
  options?: {
    category?: NotificationCategory;
    urgency?: NotificationUrgency;
  }
): Promise<void> {
  const category = options?.category || 'general';
  const urgency = options?.urgency || 'normal';
  
  try {
    // Check if we have permission
    let permissionGranted = await isPermissionGranted();
    
    if (!permissionGranted) {
      // Request permission
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }
    
    if (permissionGranted) {
      // Try to use our custom Rust command first
      try {
        await invoke('send_notification', {
          title,
          body,
          category,
          urgency,
        });
      } catch (e) {
        console.warn('Failed to send notification via Rust command, falling back to Tauri API:', e);
        
        // Fall back to Tauri API
        await notify({
          title,
          body,
        });
      }
    } else {
      console.warn('Notification permission not granted');
    }
  } catch (e) {
    console.error('Failed to show notification:', e);
  }
}

/**
 * Register global keyboard shortcuts
 */
export async function registerShortcuts(
  handlers: Partial<Record<ShortcutId, () => void>>
): Promise<UnlistenFn> {
  // Get the platform
  const platformInfo = await getPlatformInfo();
  const isMacOS = platformInfo.platform === 'macos';
  
  // Map shortcut IDs to key combinations for each platform
  const shortcutMap: Record<ShortcutId, string> = {
    'toggle-offline': isMacOS ? 'Command+Shift+O' : 'Ctrl+Shift+O',
    'force-sync': isMacOS ? 'Command+Shift+S' : 'Ctrl+Shift+S',
    'create-checkpoint': isMacOS ? 'Command+Shift+C' : 'Ctrl+Shift+C',
    'open-settings': isMacOS ? 'Command+,' : 'Ctrl+,',
    'toggle-theme': isMacOS ? 'Command+Shift+T' : 'Ctrl+Shift+T',
    'reload-app': isMacOS ? 'Command+R' : 'Ctrl+R',
    'quit-app': isMacOS ? 'Command+Q' : 'Ctrl+Q',
  };
  
  // Register shortcuts with Tauri
  return await listen('tauri://shortcut', (event) => {
    // Check which shortcut was triggered
    for (const [id, shortcut] of Object.entries(shortcutMap)) {
      if (event.payload === shortcut && handlers[id as ShortcutId]) {
        handlers[id as ShortcutId]?.();
        break;
      }
    }
  });
}

/**
 * Minimize the application window
 */
export async function minimizeWindow(): Promise<void> {
  await appWindow.minimize();
}

/**
 * Maximize the application window
 */
export async function maximizeWindow(): Promise<void> {
  if (await appWindow.isMaximized()) {
    await appWindow.unmaximize();
  } else {
    await appWindow.maximize();
  }
}

/**
 * Close the application window
 */
export async function closeWindow(): Promise<void> {
  await appWindow.close();
}

/**
 * Open a file selection dialog
 */
export async function openFileDialog(options?: {
  multiple?: boolean;
  filters?: Record<string, string[]>;
  directory?: boolean;
  title?: string;
}): Promise<string | string[] | null> {
  return await open({
    multiple: options?.multiple ?? false,
    filters: options?.filters,
    directory: options?.directory ?? false,
    title: options?.title,
  });
}

/**
 * Open a file save dialog
 */
export async function saveFileDialog(options?: {
  filters?: Record<string, string[]>;
  defaultPath?: string;
  title?: string;
}): Promise<string | null> {
  return await save({
    filters: options?.filters,
    defaultPath: options?.defaultPath,
    title: options?.title,
  });
}

/**
 * Platform-specific UI component properties
 */
export interface PlatformUiProps {
  /**
   * Platform information
   */
  platformInfo: PlatformInfo;
  
  /**
   * Window controls position (left or right)
   */
  windowControlsPosition: 'left' | 'right';
  
  /**
   * Platform-specific CSS classes
   */
  cssClasses: string[];
  
  /**
   * Platform-specific keyboard shortcuts
   */
  shortcuts: Record<ShortcutId, string>;
  
  /**
   * Platform-specific media queries
   */
  mediaQueries: {
    darkMode: string;
    highContrast: string;
    reducedMotion: string;
  };
}

/**
 * Get platform-specific UI properties
 */
export function getPlatformUiProps(platform: Platform): PlatformUiProps {
  // Platform-specific properties based on platform
  switch (platform) {
    case 'windows':
      return {
        platformInfo: {
          platform: 'windows',
          version: '',
          architecture: '',
          details: {},
        },
        windowControlsPosition: 'right',
        cssClasses: ['platform-windows'],
        shortcuts: {
          'toggle-offline': 'Ctrl+Shift+O',
          'force-sync': 'Ctrl+Shift+S',
          'create-checkpoint': 'Ctrl+Shift+C',
          'open-settings': 'Ctrl+,',
          'toggle-theme': 'Ctrl+Shift+T',
          'reload-app': 'Ctrl+R',
          'quit-app': 'Ctrl+Q',
        },
        mediaQueries: {
          darkMode: '(prefers-color-scheme: dark)',
          highContrast: '(-ms-high-contrast: active)',
          reducedMotion: '(prefers-reduced-motion: reduce)',
        },
      };
    
    case 'macos':
      return {
        platformInfo: {
          platform: 'macos',
          version: '',
          architecture: '',
          details: {},
        },
        windowControlsPosition: 'left',
        cssClasses: ['platform-macos'],
        shortcuts: {
          'toggle-offline': '⌘+Shift+O',
          'force-sync': '⌘+Shift+S',
          'create-checkpoint': '⌘+Shift+C',
          'open-settings': '⌘+,',
          'toggle-theme': '⌘+Shift+T',
          'reload-app': '⌘+R',
          'quit-app': '⌘+Q',
        },
        mediaQueries: {
          darkMode: '(prefers-color-scheme: dark)',
          highContrast: '(prefers-contrast: more)',
          reducedMotion: '(prefers-reduced-motion: reduce)',
        },
      };
    
    case 'linux':
      return {
        platformInfo: {
          platform: 'linux',
          version: '',
          architecture: '',
          details: {},
        },
        windowControlsPosition: 'right',
        cssClasses: ['platform-linux'],
        shortcuts: {
          'toggle-offline': 'Ctrl+Shift+O',
          'force-sync': 'Ctrl+Shift+S',
          'create-checkpoint': 'Ctrl+Shift+C',
          'open-settings': 'Ctrl+,',
          'toggle-theme': 'Ctrl+Shift+T',
          'reload-app': 'Ctrl+R',
          'quit-app': 'Ctrl+Q',
        },
        mediaQueries: {
          darkMode: '(prefers-color-scheme: dark)',
          highContrast: '(prefers-contrast: high)',
          reducedMotion: '(prefers-reduced-motion: reduce)',
        },
      };
    
    default:
      // Default/fallback properties
      return {
        platformInfo: {
          platform: 'unknown',
          version: '',
          architecture: '',
          details: {},
        },
        windowControlsPosition: 'right',
        cssClasses: [],
        shortcuts: {
          'toggle-offline': 'Ctrl+Shift+O',
          'force-sync': 'Ctrl+Shift+S',
          'create-checkpoint': 'Ctrl+Shift+C',
          'open-settings': 'Ctrl+,',
          'toggle-theme': 'Ctrl+Shift+T',
          'reload-app': 'Ctrl+R',
          'quit-app': 'Ctrl+Q',
        },
        mediaQueries: {
          darkMode: '(prefers-color-scheme: dark)',
          highContrast: '(prefers-contrast: more)',
          reducedMotion: '(prefers-reduced-motion: reduce)',
        },
      };
  }
}

/**
 * React hook for platform-specific UI properties
 */
export function usePlatformUi() {
  const { platform } = usePlatform();
  return getPlatformUiProps(platform);
}

/**
 * Platform-specific initialization
 */
export async function initializePlatform(): Promise<void> {
  // Get platform information
  const platformInfo = await getPlatformInfo();
  
  // Apply platform-specific CSS classes to document
  document.body.classList.add(`platform-${platformInfo.platform}`);
  
  // Listen for theme changes
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaQuery.addEventListener('change', (e) => {
    if (e.matches) {
      document.body.classList.add('dark-theme');
      document.body.classList.remove('light-theme');
    } else {
      document.body.classList.add('light-theme');
      document.body.classList.remove('dark-theme');
    }
  });
  
  // Initial theme class
  if (mediaQuery.matches) {
    document.body.classList.add('dark-theme');
  } else {
    document.body.classList.add('light-theme');
  }
  
  // Listen for high contrast mode changes
  const highContrastQuery = window.matchMedia(
    platformInfo.platform === 'windows'
      ? '(-ms-high-contrast: active)'
      : '(prefers-contrast: more)'
  );
  
  highContrastQuery.addEventListener('change', (e) => {
    if (e.matches) {
      document.body.classList.add('high-contrast');
    } else {
      document.body.classList.remove('high-contrast');
    }
  });
  
  // Initial high contrast class
  if (highContrastQuery.matches) {
    document.body.classList.add('high-contrast');
  }
  
  // Listen for reduced motion preference changes
  const reducedMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  reducedMotionQuery.addEventListener('change', (e) => {
    if (e.matches) {
      document.body.classList.add('reduced-motion');
    } else {
      document.body.classList.remove('reduced-motion');
    }
  });
  
  // Initial reduced motion class
  if (reducedMotionQuery.matches) {
    document.body.classList.add('reduced-motion');
  }
  
  // Apply platform-specific window behaviors
  switch (platformInfo.platform) {
    case 'windows':
      // Use system decorations for Windows
      await appWindow.setDecorations(true);
      break;
    
    case 'macos':
      // Set traffic light position for macOS
      await appWindow.setDecorations(true);
      break;
    
    case 'linux':
      // Linux-specific window handling
      await appWindow.setDecorations(true);
      break;
  }
}
