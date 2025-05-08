import { useState, useCallback } from 'react';

/**
 * A hook for adding press/click feedback effect to interactive elements
 * 
 * @returns Props to spread on the target element and a reset function
 */
export const usePressEffect = () => {
  const [isPressed, setIsPressed] = useState(false);
  
  const handleMouseDown = useCallback(() => {
    setIsPressed(true);
  }, []);
  
  const handleMouseUp = useCallback(() => {
    setIsPressed(false);
  }, []);
  
  const handleMouseLeave = useCallback(() => {
    setIsPressed(false);
  }, []);
  
  const reset = useCallback(() => {
    setIsPressed(false);
  }, []);
  
  // Props to spread on the target element
  const props = {
    className: isPressed ? 'button-press' : '',
    onMouseDown: handleMouseDown,
    onMouseUp: handleMouseUp,
    onMouseLeave: handleMouseLeave,
    onTouchStart: handleMouseDown,
    onTouchEnd: handleMouseUp,
    onTouchCancel: handleMouseUp,
  };
  
  return {
    isPressed,
    props,
    reset,
  };
};

export default usePressEffect;
