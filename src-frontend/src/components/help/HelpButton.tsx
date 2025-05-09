import React, { useState, useEffect } from 'react';
import './HelpButton.css';
import HelpCenter from './HelpCenter';
import { createPortal } from 'react-dom';

interface HelpButtonProps {
  className?: string;
  isOpen?: boolean;
  onOpenChange?: (isOpen: boolean) => void;
}

const HelpButton: React.FC<HelpButtonProps> = ({ 
  className, 
  isOpen: propIsOpen, 
  onOpenChange 
}) => {
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    if (propIsOpen !== undefined) {
      setIsOpen(propIsOpen);
    }
  }, [propIsOpen]);

  const toggleHelp = () => {
    const newState = !isOpen;
    setIsOpen(newState);
    if (onOpenChange) {
      onOpenChange(newState);
    }
  };

  const closeHelp = () => {
    setIsOpen(false);
    if (onOpenChange) {
      onOpenChange(false);
    }
  };

  return (
    <>
      <button 
        className={`help-button ${className || ''}`} 
        onClick={toggleHelp}
        aria-label="Help"
        title="Open Help Center"
      >
        <svg className="help-icon" viewBox="0 0 24 24" width="20" height="20">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm2.07-7.75l-.9.92C13.45 12.9 13 13.5 13 15h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H8c0-2.21 1.79-4 4-4s4 1.79 4 4c0 .88-.36 1.68-.93 2.25z" />
        </svg>
      </button>
      
      {isOpen && createPortal(
        <div className="help-modal-overlay" onClick={closeHelp}>
          <div className="help-modal" onClick={e => e.stopPropagation()}>
            <button className="help-close-button" onClick={closeHelp} aria-label="Close Help">
              <svg viewBox="0 0 24 24" width="24" height="24">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
            <HelpCenter />
          </div>
        </div>,
        document.body
      )}
    </>
  );
};

export default HelpButton;
