// src-frontend/src/App.tsx
//
// Main application entry point

import React, { useState, useEffect } from 'react';
// No need for BrowserRouter here as it's in main.tsx
import { ThemeProvider as StyledThemeProvider } from 'styled-components';
import { ThemeProvider as LegacyThemeProvider } from './components/theme/ThemeProvider';
import { ThemeProvider as PlatformThemeProvider } from './components/ThemeProvider';
import ErrorBoundary from './components/ErrorBoundary';
import AppRoutes from './Routes';
import { appTheme, globalStyles } from './styles/theme';

/**
 * Main application component
 */
export function App() {
  // State for dark mode preference
  const [isDarkMode, setIsDarkMode] = useState<boolean>(true);
  
  // Detect system theme preference on mount
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    setIsDarkMode(mediaQuery.matches);
    
    // Listen for changes in system theme
    const handler = (e: MediaQueryListEvent) => setIsDarkMode(e.matches);
    mediaQuery.addEventListener('change', handler);
    
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return (
    // Use our new platform-aware ThemeProvider
    <PlatformThemeProvider isDark={isDarkMode}>
      {/* Use the legacy ThemeProvider for backward compatibility */}
      <LegacyThemeProvider initialMode={isDarkMode ? "dark" : "light"}>
        {/* Use styled-components ThemeProvider */}
        <StyledThemeProvider theme={appTheme}>
          <div className="app">
            <style>
              {globalStyles}
            </style>
            <ErrorBoundary>
              <AppRoutes />
            </ErrorBoundary>
          </div>
        </StyledThemeProvider>
      </LegacyThemeProvider>
    </PlatformThemeProvider>
  );
}

export default App;