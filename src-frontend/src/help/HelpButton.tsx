import React from 'react';
import { useHelp } from './HelpProvider';

interface HelpButtonProps {
  className?: string;
}

const HelpButton: React.FC<HelpButtonProps> = ({ className = '' }) => {
  const { openHelpPanel } = useHelp();
  
  return (
    <button
      className={`help-button ${className}`}
      onClick={openHelpPanel}
      aria-label="Open help center"
    >
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
        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
        <line x1="12" y1="17" x2="12.01" y2="17" />
      </svg>
      Help Center
    </button>
  );
};

export default HelpButton;
