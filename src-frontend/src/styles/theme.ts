// src-frontend/src/styles/theme.ts
// Theme and style constants for the application

export const appTheme = {
  // Color palette
  colors: {
    // Primary brand colors
    primary: {
      main: '#e50914', // Netflix red
      light: '#f40612',
      dark: '#b20710',
      contrast: '#ffffff',
    },
    
    // Secondary brand colors
    secondary: {
      main: '#0071eb', // Netflix blue
      light: '#2196f3',
      dark: '#0a67d1',
      contrast: '#ffffff',
    },
    
    // Dark theme colors
    dark: {
      background: {
        primary: '#141414', // Netflix main background
        secondary: '#181818', // Card background
        tertiary: '#232323', // Dropdown background
      },
      text: {
        primary: '#ffffff',
        secondary: '#b3b3b3',
        disabled: '#757575',
      },
      divider: 'rgba(255, 255, 255, 0.1)',
      button: {
        primary: '#e50914',
        secondary: 'rgba(255, 255, 255, 0.1)',
        disabled: 'rgba(255, 255, 255, 0.05)',
      },
      input: {
        background: 'rgba(255, 255, 255, 0.1)',
        placeholder: '#8c8c8c',
        text: '#ffffff',
        border: 'transparent',
        focusBorder: '#00b7ff',
      },
      card: {
        background: '#181818',
        hover: '#232323',
        border: 'rgba(255, 255, 255, 0.1)',
      },
      popover: {
        background: '#181818',
        border: 'rgba(255, 255, 255, 0.1)',
      },
      backdrop: 'rgba(0, 0, 0, 0.7)',
    },
    
    // Light theme colors
    light: {
      background: {
        primary: '#f3f3f3',
        secondary: '#ffffff',
        tertiary: '#f8f8f8',
      },
      text: {
        primary: '#111111',
        secondary: '#757575',
        disabled: '#9e9e9e',
      },
      divider: 'rgba(0, 0, 0, 0.1)',
      button: {
        primary: '#e50914',
        secondary: 'rgba(0, 0, 0, 0.1)',
        disabled: 'rgba(0, 0, 0, 0.05)',
      },
      input: {
        background: '#ffffff',
        placeholder: '#8c8c8c',
        text: '#111111',
        border: '#dbdbdb',
        focusBorder: '#0071eb',
      },
      card: {
        background: '#ffffff',
        hover: '#f8f8f8',
        border: '#dbdbdb',
      },
      popover: {
        background: '#ffffff',
        border: '#dbdbdb',
      },
      backdrop: 'rgba(0, 0, 0, 0.5)',
    },
    
    // Status colors
    status: {
      success: {
        main: '#00C853',
        light: 'rgba(0, 200, 83, 0.1)',
      },
      warning: {
        main: '#FFA000',
        light: 'rgba(255, 160, 0, 0.1)',
      },
      error: {
        main: '#e50914',
        light: 'rgba(229, 9, 20, 0.1)',
      },
      info: {
        main: '#0071eb',
        light: 'rgba(0, 113, 235, 0.1)',
      },
    },
  },
  
  // Typography
  typography: {
    fontFamily: '"Netflix Sans", "Helvetica Neue", Helvetica, Arial, sans-serif',
    fontWeights: {
      light: 300,
      regular: 400,
      medium: 500,
      bold: 700,
    },
    sizes: {
      xs: '0.75rem',     // 12px
      sm: '0.875rem',    // 14px
      md: '1rem',        // 16px
      lg: '1.125rem',    // 18px
      xl: '1.25rem',     // 20px
      '2xl': '1.5rem',   // 24px
      '3xl': '1.875rem', // 30px
      '4xl': '2.25rem',  // 36px
      '5xl': '3rem',     // 48px
    },
    lineHeights: {
      tight: 1.2,
      base: 1.5,
      loose: 1.8,
    },
  },
  
  // Spacing
  spacing: {
    xs: '0.25rem',    // 4px
    sm: '0.5rem',     // 8px
    md: '1rem',       // 16px
    lg: '1.5rem',     // 24px
    xl: '2rem',       // 32px
    '2xl': '2.5rem',  // 40px
    '3xl': '3rem',    // 48px
  },
  
  // Breakpoints
  breakpoints: {
    xs: '0px',
    sm: '600px',
    md: '960px',
    lg: '1280px',
    xl: '1920px',
  },
  
  // Animation
  animation: {
    durations: {
      fast: '150ms',
      normal: '300ms',
      slow: '500ms',
    },
    easings: {
      standard: 'cubic-bezier(0.4, 0.0, 0.2, 1)',
      accelerate: 'cubic-bezier(0.4, 0.0, 1, 1)',
      decelerate: 'cubic-bezier(0.0, 0.0, 0.2, 1)',
    },
  },
  
  // Shadows
  shadows: {
    sm: '0 1px 3px rgba(0, 0, 0, 0.1)',
    md: '0 4px 6px rgba(0, 0, 0, 0.1)',
    lg: '0 10px 15px rgba(0, 0, 0, 0.1)',
    xl: '0 20px 25px rgba(0, 0, 0, 0.1)',
    '2xl': '0 25px 50px rgba(0, 0, 0, 0.25)',
    inner: 'inset 0 2px 4px rgba(0, 0, 0, 0.05)',
  },
  
  // Border radius
  borderRadius: {
    sm: '0.125rem',   // 2px
    md: '0.25rem',    // 4px
    lg: '0.5rem',     // 8px
    xl: '0.75rem',    // 12px
    '2xl': '1rem',    // 16px
    full: '9999px',
  },
  
  // Z-index
  zIndex: {
    navbar: 1000,
    dropdown: 1010,
    modal: 1100,
    tooltip: 1200,
    toast: 1300,
  },
};

// Netflix-inspired UI component variants
export const componentVariants = {
  // Button variants
  button: {
    primary: {
      backgroundColor: appTheme.colors.primary.main,
      color: '#ffffff',
      hoverBg: appTheme.colors.primary.light,
      activeBg: appTheme.colors.primary.dark,
    },
    secondary: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      color: '#ffffff',
      hoverBg: 'rgba(255, 255, 255, 0.2)',
      activeBg: 'rgba(255, 255, 255, 0.15)',
    },
    text: {
      backgroundColor: 'transparent',
      color: '#ffffff',
      hoverBg: 'rgba(255, 255, 255, 0.1)',
      activeBg: 'rgba(255, 255, 255, 0.05)',
    },
  },
  
  // Card variants
  card: {
    default: {
      backgroundColor: '#181818',
      hoverTransform: 'scale(1.05)',
      transitionDuration: '300ms',
      shadow: '0 10px 30px rgba(0, 0, 0, 0.3)',
    },
    featured: {
      backgroundColor: '#181818',
      hoverTransform: 'scale(1.08)',
      transitionDuration: '300ms',
      shadow: '0 15px 40px rgba(0, 0, 0, 0.4)',
    },
    flat: {
      backgroundColor: '#181818',
      hoverTransform: 'none',
      transitionDuration: '300ms',
      shadow: 'none',
    },
  },
  
  // Input variants
  input: {
    default: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      textColor: '#ffffff',
      placeholderColor: '#8c8c8c',
      focusBorderColor: '#0071eb',
    },
    filled: {
      backgroundColor: '#333333',
      textColor: '#ffffff',
      placeholderColor: '#8c8c8c',
      focusBorderColor: '#0071eb',
    },
  },
  
  // Badge variants
  badge: {
    success: {
      backgroundColor: 'rgba(0, 200, 83, 0.2)',
      textColor: '#00C853',
    },
    warning: {
      backgroundColor: 'rgba(255, 160, 0, 0.2)',
      textColor: '#FFA000',
    },
    error: {
      backgroundColor: 'rgba(229, 9, 20, 0.2)',
      textColor: '#E50914',
    },
    info: {
      backgroundColor: 'rgba(0, 113, 235, 0.2)',
      textColor: '#0071EB',
    },
    default: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      textColor: '#FFFFFF',
    },
  },
  
  // Progress bar variants
  progressBar: {
    default: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      fillColor: '#0071EB',
      height: '4px',
    },
    success: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      fillColor: '#00C853',
      height: '4px',
    },
    warning: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      fillColor: '#FFA000',
      height: '4px',
    },
    error: {
      backgroundColor: 'rgba(255, 255, 255, 0.1)',
      fillColor: '#E50914',
      height: '4px',
    },
  },
};

// CSS transitions for different elements
export const transitions = {
  button: 'background-color 0.2s ease, transform 0.1s ease',
  card: 'transform 0.3s ease, box-shadow 0.3s ease',
  modal: 'opacity 0.3s ease, transform 0.3s ease',
  dropdown: 'opacity 0.2s ease, transform 0.2s ease',
  fadeIn: 'opacity 0.3s ease-in',
  fadeOut: 'opacity 0.2s ease-out',
  slideIn: 'transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1)',
  slideOut: 'transform 0.2s cubic-bezier(0.4, 0.0, 1, 1)',
};

// Netflix-inspired CSS animations
export const animations = {
  fadeIn: `
    @keyframes fadeIn {
      from { opacity: 0; }
      to { opacity: 1; }
    }
  `,
  fadeOut: `
    @keyframes fadeOut {
      from { opacity: 1; }
      to { opacity: 0; }
    }
  `,
  slideUp: `
    @keyframes slideUp {
      from { transform: translateY(20px); opacity: 0; }
      to { transform: translateY(0); opacity: 1; }
    }
  `,
  slideDown: `
    @keyframes slideDown {
      from { transform: translateY(-20px); opacity: 0; }
      to { transform: translateY(0); opacity: 1; }
    }
  `,
  pulse: `
    @keyframes pulse {
      0% { transform: scale(1); }
      50% { transform: scale(1.05); }
      100% { transform: scale(1); }
    }
  `,
  shimmer: `
    @keyframes shimmer {
      0% { background-position: -1000px 0; }
      100% { background-position: 1000px 0; }
    }
  `,
};

// Global styles for use across the application
export const globalStyles = `
  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }
  
  html, body {
    font-family: ${appTheme.typography.fontFamily};
    font-size: 16px;
    font-weight: ${appTheme.typography.fontWeights.regular};
    line-height: ${appTheme.typography.lineHeights.base};
    background-color: ${appTheme.colors.dark.background.primary};
    color: ${appTheme.colors.dark.text.primary};
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }
  
  h1, h2, h3, h4, h5, h6 {
    font-weight: ${appTheme.typography.fontWeights.bold};
    line-height: ${appTheme.typography.lineHeights.tight};
    margin-bottom: ${appTheme.spacing.md};
  }
  
  p {
    margin-bottom: ${appTheme.spacing.md};
  }
  
  a {
    color: ${appTheme.colors.secondary.main};
    text-decoration: none;
    transition: color 0.2s ease;
  }
  
  a:hover {
    color: ${appTheme.colors.secondary.light};
  }
  
  button {
    cursor: pointer;
    font-family: ${appTheme.typography.fontFamily};
  }
  
  /* Custom scrollbar */
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  
  ::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }
  
  ::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }
  
  ::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
  
  /* Netflix-inspired scrolling row */
  .horizontal-scroll {
    display: flex;
    overflow-x: auto;
    scroll-behavior: smooth;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    margin: 0 -4%;
    padding: 0 4%;
  }
  
  .horizontal-scroll::-webkit-scrollbar {
    display: none;
  }
`;

export default appTheme;