import { useState, useCallback, useRef, useEffect } from 'react';

type MicroInteractionType = 
  | 'press' 
  | 'pulse' 
  | 'shake' 
  | 'bounce' 
  | 'wiggle' 
  | 'tada' 
  | 'jello' 
  | 'heartbeat';

interface MicroInteractionOptions {
  duration?: number;
  delay?: number;
  count?: number;
}

/**
 * A hook for adding micro-interactions to UI elements
 * 
 * @param type - The type of micro-interaction
 * @param options - Options for customizing the interaction
 * @returns An object with the current animation class, a trigger function, and a reset function
 */
export const useMicroInteraction = (
  type: MicroInteractionType,
  options: MicroInteractionOptions = {}
) => {
  const {
    duration = 500,
    delay = 0,
    count = 1,
  } = options;
  
  const [isAnimating, setIsAnimating] = useState(false);
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const counterRef = useRef(0);
  
  // Clear any existing timers when component unmounts
  useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);
  
  // Reset the animation state
  const reset = useCallback(() => {
    setIsAnimating(false);
    counterRef.current = 0;
    
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);
  
  // Trigger the animation
  const trigger = useCallback(() => {
    if (isAnimating && count === 1) {
      // If already animating and we only want to play once, just return
      return;
    }
    
    // Reset any existing animation
    reset();
    
    // Start the animation after the specified delay
    timerRef.current = setTimeout(() => {
      setIsAnimating(true);
      counterRef.current = 1;
      
      // Set a timeout to end the animation
      if (count !== Infinity) {
        timerRef.current = setTimeout(() => {
          setIsAnimating(false);
          timerRef.current = null;
        }, duration);
      }
      
      // For repeating animations
      if (count > 1) {
        const intervalId = setInterval(() => {
          counterRef.current += 1;
          
          // Force a re-render to restart the animation
          setIsAnimating(false);
          setTimeout(() => setIsAnimating(true), 10);
          
          if (counterRef.current >= count) {
            clearInterval(intervalId);
            
            // End the animation after the last iteration
            setTimeout(() => {
              setIsAnimating(false);
            }, duration);
          }
        }, duration);
        
        // Store the interval ID to clear it when unmounting
        timerRef.current = intervalId as unknown as NodeJS.Timeout;
      }
    }, delay);
  }, [isAnimating, count, duration, delay, reset]);
  
  // Determine the current animation class
  const animationClass = isAnimating ? type : '';
  
  // Additional properties for the element
  const props = {
    className: animationClass,
    style: isAnimating ? { animationDuration: `${duration}ms` } : undefined,
  };
  
  return {
    isAnimating,
    animationClass,
    props,
    trigger,
    reset,
  };
};

export default useMicroInteraction;
