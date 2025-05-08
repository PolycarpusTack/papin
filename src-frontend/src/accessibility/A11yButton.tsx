import React from 'react';
import { useAccessibility } from './AccessibilityProvider';

const A11yButton: React.FC = () => {
  const { toggleA11yPanel } = useAccessibility();

  return (
    <button 
      className="a11y-quick-access" 
      onClick={toggleA11yPanel}
      aria-label="Accessibility settings"
    >
      <span className="a11y-quick-access-tooltip">Accessibility Settings</span>
      <svg 
        xmlns="http://www.w3.org/2000/svg" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        strokeWidth="2" 
        strokeLinecap="round" 
        strokeLinejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <path d="M12 8v4M12 16h.01" />
      </svg>
    </button>
  );
};

export default A11yButton;
