// src-frontend/src/components/theme/ThemeProvider.tsx
//
// Platform-aware theme provider

import React, { createContext, useContext, useEffect, useState } from 'react';
import { usePlatform } from '../../hooks/usePlatform';
import { PlatformTheme, ThemeMode, getPlatformTheme, applyTheme } from '../../styles/theme';

// Theme context
interface ThemeContextType {
  theme: PlatformTheme;
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// Props for ThemeProvider
interface ThemeProviderProps {
  children: React.ReactNode;
  initialMode?: ThemeMode;
}

/**
 * Platform-aware theme provider
 */
export function ThemeProvider({ 
  children, 
  initialMode = 'system' 
}: ThemeProviderProps): JSX.Element {
  const { platform } = usePlatform();
  const [mode, setMode] = useState<ThemeMode>(() => {
    // Try to load from localStorage first
    const savedMode = localStorage.getItem('themeMode') as ThemeMode;
    return savedMode || initialMode;
  });
  
  // Generate the theme based on platform and mode
  const [theme, setTheme] = useState<PlatformTheme>(() => 
    getPlatformTheme(platform, mode)
  );
  
  // Update theme when platform or mode changes
  useEffect(() => {
    if (platform !== 'unknown') {
      const newTheme = getPlatformTheme(platform, mode);
      setTheme(newTheme);
      applyTheme(newTheme);
    }
  }, [platform, mode]);
  
  // Listen for system theme changes if using system mode
  useEffect(() => {
    if (mode === 'system') {
      const darkModeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      
      const updateTheme = () => {
        const newTheme = getPlatformTheme(platform, mode);
        setTheme(newTheme);
        applyTheme(newTheme);
      };
      
      darkModeMediaQuery.addEventListener('change', updateTheme);
      
      return () => {
        darkModeMediaQuery.removeEventListener('change', updateTheme);
      };
    }
  }, [mode, platform]);
  
  // Handle theme mode changes
  const handleSetMode = (newMode: ThemeMode) => {
    setMode(newMode);
    localStorage.setItem('themeMode', newMode);
  };
  
  return (
    <ThemeContext.Provider value={{ theme, mode, setMode: handleSetMode }}>
      {children}
    </ThemeContext.Provider>
  );
}

/**
 * Hook to use the theme context
 */
export function useTheme(): ThemeContextType {
  const context = useContext(ThemeContext);
  
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  
  return context;
}

/**
 * ThemeSwitcher component
 */
export function ThemeSwitcher(): JSX.Element {
  const { mode, setMode } = useTheme();
  
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setMode(e.target.value as ThemeMode);
  };
  
  return (
    <select value={mode} onChange={handleChange} className="theme-switcher">
      <option value="light">Light</option>
      <option value="dark">Dark</option>
      <option value="system">System</option>
    </select>
  );
}
