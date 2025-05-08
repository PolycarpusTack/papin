import React, { createContext, useContext, useState, useEffect } from 'react';

export type ThemeType = 'light' | 'dark' | 'system';

interface ThemeContextType {
  theme: ThemeType;
  setTheme: (theme: ThemeType) => void;
  actualTheme: 'light' | 'dark'; // The actual applied theme after system preference is applied
}

const ThemeContext = createContext<ThemeContextType>({
  theme: 'system',
  setTheme: () => {},
  actualTheme: 'light',
});

export const useTheme = () => useContext(ThemeContext);

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  // Get saved theme from localStorage or default to system
  const getSavedTheme = (): ThemeType => {
    const savedTheme = localStorage.getItem('mcp-theme');
    return (savedTheme as ThemeType) || 'system';
  };

  const [theme, setTheme] = useState<ThemeType>(getSavedTheme());
  const [actualTheme, setActualTheme] = useState<'light' | 'dark'>('light');

  // Function to determine if system prefers dark mode
  const systemPrefersDark = () => {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
  };

  // Update the theme
  const updateTheme = (newTheme: ThemeType) => {
    setTheme(newTheme);
    localStorage.setItem('mcp-theme', newTheme);
    
    // Determine the actual theme to apply
    if (newTheme === 'system') {
      setActualTheme(systemPrefersDark() ? 'dark' : 'light');
    } else {
      setActualTheme(newTheme);
    }
  };

  // Set up the initial theme
  useEffect(() => {
    updateTheme(theme);
    
    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleSystemThemeChange = (e: MediaQueryListEvent) => {
      if (theme === 'system') {
        setActualTheme(e.matches ? 'dark' : 'light');
      }
    };
    
    mediaQuery.addEventListener('change', handleSystemThemeChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleSystemThemeChange);
    };
  }, [theme]);

  // Apply the theme to the document
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', actualTheme);
    
    // Also add a class for easier styling
    if (actualTheme === 'dark') {
      document.documentElement.classList.add('dark-theme');
      document.documentElement.classList.remove('light-theme');
    } else {
      document.documentElement.classList.add('light-theme');
      document.documentElement.classList.remove('dark-theme');
    }
  }, [actualTheme]);

  return (
    <ThemeContext.Provider value={{ theme, setTheme: updateTheme, actualTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};
