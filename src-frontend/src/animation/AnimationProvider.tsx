import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import './animations.css';

interface AnimationContextType {
  animationsEnabled: boolean;
  animationSpeed: 'normal' | 'slow' | 'fast';
  toggleAnimations: () => void;
  setAnimationSpeed: (speed: 'normal' | 'slow' | 'fast') => void;
}

const AnimationContext = createContext<AnimationContextType>({
  animationsEnabled: true,
  animationSpeed: 'normal',
  toggleAnimations: () => {},
  setAnimationSpeed: () => {},
});

export const useAnimation = () => useContext(AnimationContext);

interface AnimationProviderProps {
  children: ReactNode;
}

export const AnimationProvider: React.FC<AnimationProviderProps> = ({ children }) => {
  // Get saved preferences from localStorage or use defaults
  const getSavedAnimationPreference = (): boolean => {
    const savedPref = localStorage.getItem('mcp-animations-enabled');
    return savedPref !== null ? savedPref === 'true' : true;
  };

  const getSavedAnimationSpeed = (): 'normal' | 'slow' | 'fast' => {
    const savedSpeed = localStorage.getItem('mcp-animation-speed');
    return (savedSpeed as 'normal' | 'slow' | 'fast') || 'normal';
  };

  const [animationsEnabled, setAnimationsEnabled] = useState<boolean>(getSavedAnimationPreference());
  const [animationSpeed, setAnimationSpeed] = useState<'normal' | 'slow' | 'fast'>(getSavedAnimationSpeed());

  // Toggle animations on/off
  const toggleAnimations = () => {
    const newValue = !animationsEnabled;
    setAnimationsEnabled(newValue);
    localStorage.setItem('mcp-animations-enabled', String(newValue));
  };

  // Set animation speed and save to localStorage
  const handleSetAnimationSpeed = (speed: 'normal' | 'slow' | 'fast') => {
    setAnimationSpeed(speed);
    localStorage.setItem('mcp-animation-speed', speed);
  };

  // Apply animation settings to document
  useEffect(() => {
    const root = document.documentElement;
    
    // Handle animation enabling/disabling
    if (animationsEnabled) {
      document.body.classList.remove('animate-none');
    } else {
      document.body.classList.add('animate-none');
    }
    
    // Handle animation speed
    let multiplier: number;
    switch (animationSpeed) {
      case 'slow':
        multiplier = 1.5;
        break;
      case 'fast':
        multiplier = 0.6;
        break;
      default:
        multiplier = 1;
    }
    
    root.style.setProperty('--user-animation-multiplier', String(multiplier));
    
    // Also check prefers-reduced-motion
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    const handleReducedMotionChange = (e: MediaQueryListEvent) => {
      if (e.matches && animationsEnabled) {
        // Don't disable completely, but reduce speed
        root.style.setProperty('--user-animation-multiplier', '0.5');
      } else {
        // Restore the multiplier based on user preference
        root.style.setProperty('--user-animation-multiplier', String(multiplier));
      }
    };
    
    mediaQuery.addEventListener('change', handleReducedMotionChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleReducedMotionChange);
    };
  }, [animationsEnabled, animationSpeed]);

  return (
    <AnimationContext.Provider 
      value={{ 
        animationsEnabled, 
        animationSpeed, 
        toggleAnimations, 
        setAnimationSpeed: handleSetAnimationSpeed 
      }}
    >
      {children}
    </AnimationContext.Provider>
  );
};

export default AnimationProvider;
