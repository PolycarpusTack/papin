import React, { createContext, useContext, ReactNode, useMemo } from 'react';
import usePlatform from '../hooks/usePlatform';

// Base theme values
interface ThemeColors {
  primary: string;
  secondary: string;
  background: string;
  card: string;
  text: string;
  border: string;
  error: string;
  success: string;
  warning: string;
  info: string;
}

interface ThemeSpacing {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
}

interface ThemeRadii {
  sm: string;
  md: string;
  lg: string;
  full: string;
}

interface ThemeFonts {
  regular: string;
  medium: string;
  bold: string;
  mono: string;
}

export interface Theme {
  colors: ThemeColors;
  spacing: ThemeSpacing;
  radii: ThemeRadii;
  fonts: ThemeFonts;
  isDark: boolean;
}

// Create the context
const ThemeContext = createContext<Theme | undefined>(undefined);

// Platform-specific theme values
const getThemeForPlatform = (platform: string, isDark: boolean): Theme => {
  // Base theme
  const baseTheme: Theme = {
    colors: {
      primary: isDark ? '#4f87fb' : '#0066cc',
      secondary: isDark ? '#6c757d' : '#5a6268',
      background: isDark ? '#121212' : '#f8f9fa',
      card: isDark ? '#1e1e1e' : '#ffffff',
      text: isDark ? '#e0e0e0' : '#212529',
      border: isDark ? '#2d2d2d' : '#dee2e6',
      error: isDark ? '#ff5252' : '#dc3545',
      success: isDark ? '#00e676' : '#28a745',
      warning: isDark ? '#ffab40' : '#ffc107',
      info: isDark ? '#40c4ff' : '#17a2b8',
    },
    spacing: {
      xs: '0.25rem',
      sm: '0.5rem',
      md: '1rem',
      lg: '1.5rem',
      xl: '2rem',
    },
    radii: {
      sm: '0.25rem',
      md: '0.5rem',
      lg: '1rem',
      full: '9999px',
    },
    fonts: {
      regular: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      medium: '"Inter Medium", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      bold: '"Inter Bold", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      mono: '"JetBrains Mono", "SF Mono", "Cascadia Code", Menlo, Monaco, Consolas, monospace',
    },
    isDark,
  };

  // Platform-specific theme adjustments
  switch (platform) {
    case 'windows':
      return {
        ...baseTheme,
        radii: {
          ...baseTheme.radii,
          sm: '0.125rem',
          md: '0.25rem',
          lg: '0.5rem',
        },
        fonts: {
          ...baseTheme.fonts,
          regular: '"Segoe UI", sans-serif',
          medium: '"Segoe UI Semibold", sans-serif',
          bold: '"Segoe UI Bold", sans-serif',
          mono: '"Cascadia Code", Consolas, monospace',
        },
      };
    case 'macos':
      return {
        ...baseTheme,
        radii: {
          ...baseTheme.radii,
          sm: '0.375rem',
          md: '0.75rem',
          lg: '1.25rem',
        },
        fonts: {
          ...baseTheme.fonts,
          regular: '-apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif',
          medium: '-apple-system, BlinkMacSystemFont, "SF Pro Text Semibold", sans-serif',
          bold: '-apple-system, BlinkMacSystemFont, "SF Pro Text Bold", sans-serif',
          mono: '"SF Mono", Menlo, monospace',
        },
      };
    case 'linux':
      return {
        ...baseTheme,
        fonts: {
          ...baseTheme.fonts,
          regular: '"Noto Sans", "Ubuntu", sans-serif',
          medium: '"Noto Sans Medium", "Ubuntu Medium", sans-serif',
          bold: '"Noto Sans Bold", "Ubuntu Bold", sans-serif',
          mono: '"DejaVu Sans Mono", "Ubuntu Mono", monospace',
        },
      };
    default:
      return baseTheme;
  }
};

interface ThemeProviderProps {
  children: ReactNode;
  isDark?: boolean;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({
  children,
  isDark = false,
}) => {
  const { platform, isLoading } = usePlatform();
  
  const theme = useMemo(() => {
    return getThemeForPlatform(platform, isDark);
  }, [platform, isDark]);

  // If platform is loading, use a fallback theme
  if (isLoading) {
    return <>{children}</>;
  }

  return (
    <ThemeContext.Provider value={theme}>
      {children}
    </ThemeContext.Provider>
  );
};

// Custom hook to use the theme
export const useTheme = (): Theme => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

export default ThemeProvider;