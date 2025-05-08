import { useState, useCallback } from 'react';

interface HoverEffectOptions {
  delayEnter?: number;
  delayLeave?: number;
}

/**
 * A hook for adding hover effects with customizable delays
 * 
 * @param options - Options for customizing the hover effect
 * @returns isHovered state and props to spread on the target element
 */
export const useHoverEffect = (options: HoverEffectOptions = {}) => {
  const {
    delayEnter = 0,
    delayLeave = 0,
  } = options;
  
  const [isHovered, setIsHovered] = useState(false);
  const [hoverTimer, setHoverTimer] = useState<NodeJS.Timeout | null>(null);
  
  const handleMouseEnter = useCallback(() => {
    // Clear any existing timer
    if (hoverTimer) {
      clearTimeout(hoverTimer);
    }
    
    if (delayEnter > 0) {
      // Set a timer to activate the hover state
      const timer = setTimeout(() => {
        setIsHovered(true);
        setHoverTimer(null);
      }, delayEnter);
      
      setHoverTimer(timer);
    } else {
      // No delay, activate immediately
      setIsHovered(true);
    }
  }, [delayEnter, hoverTimer]);
  
  const handleMouseLeave = useCallback(() => {
    // Clear any existing timer
    if (hoverTimer) {
      clearTimeout(hoverTimer);
    }
    
    if (delayLeave > 0) {
      // Set a timer to deactivate the hover state
      const timer = setTimeout(() => {
        setIsHovered(false);
        setHoverTimer(null);
      }, delayLeave);
      
      setHoverTimer(timer);
    } else {
      // No delay, deactivate immediately
      setIsHovered(false);
    }
  }, [delayLeave, hoverTimer]);
  
  // Clean up timers if component unmounts
  const reset = useCallback(() => {
    if (hoverTimer) {
      clearTimeout(hoverTimer);
    }
    setIsHovered(false);
    setHoverTimer(null);
  }, [hoverTimer]);
  
  // Props to spread on the target element
  const props = {
    onMouseEnter: handleMouseEnter,
    onMouseLeave: handleMouseLeave,
  };
  
  return {
    isHovered,
    props,
    reset,
  };
};

export default useHoverEffect;
