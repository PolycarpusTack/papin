import React, { useState, useCallback, useRef, CSSProperties } from 'react';

interface RippleStyle extends CSSProperties {
  top: number;
  left: number;
  width: number;
  height: number;
}

interface Ripple {
  id: number;
  style: RippleStyle;
}

interface RippleEffectOptions {
  color?: string;
  duration?: number;
}

/**
 * A hook for adding ripple effects (similar to Material Design) to elements
 * 
 * @param options - Options for customizing the ripple effect
 * @returns Props to spread on the target element and a component to render inside
 */
export const useRippleEffect = (options: RippleEffectOptions = {}) => {
  const {
    color = 'rgba(255, 255, 255, 0.3)',
    duration = 500,
  } = options;
  
  const [ripples, setRipples] = useState<Ripple[]>([]);
  const nextId = useRef(0);
  
  const addRipple = useCallback((e: React.MouseEvent | React.TouchEvent) => {
    const target = e.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    
    let pageX, pageY;
    
    if ('touches' in e) {
      pageX = e.touches[0].pageX;
      pageY = e.touches[0].pageY;
    } else {
      pageX = e.pageX;
      pageY = e.pageY;
    }
    
    const left = pageX - (rect.left + window.scrollX);
    const top = pageY - (rect.top + window.scrollY);
    
    const size = Math.max(rect.width, rect.height) * 2;
    
    const newRipple: Ripple = {
      id: nextId.current++,
      style: {
        top: top - size / 2,
        left: left - size / 2,
        width: size,
        height: size,
      },
    };
    
    setRipples(prev => [...prev, newRipple]);
    
    // Remove the ripple after the animation completes
    setTimeout(() => {
      setRipples(prev => prev.filter(r => r.id !== newRipple.id));
    }, duration + 100); // Add 100ms buffer
  }, [duration]);
  
  // Clear all ripples
  const reset = useCallback(() => {
    setRipples([]);
  }, []);
  
  // Props to spread on the target element
  const props = {
    onClick: addRipple,
    style: { position: 'relative', overflow: 'hidden' } as CSSProperties,
  };
  
  // Component to render inside the target
  const RippleEffect: React.FC = () => (
    <>
      {ripples.map(ripple => (
        <span
          key={ripple.id}
          style={{
            position: 'absolute',
            borderRadius: '50%',
            backgroundColor: color,
            opacity: 0.6,
            transform: 'scale(0)',
            animation: `ripple ${duration}ms ease-out`,
            ...ripple.style,
          }}
        />
      ))}
      <style>
        {`
          @keyframes ripple {
            to {
              opacity: 0;
              transform: scale(2);
            }
          }
        `}
      </style>
    </>
  );
  
  return {
    ripples,
    props,
    reset,
    RippleEffect,
  };
};

export default useRippleEffect;
