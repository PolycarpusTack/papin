import React from 'react';
import HelpButton from '../help/HelpButton';

interface HelpMenuProps {
  showHelpCenter: () => void;
}

const HelpMenu: React.FC<HelpMenuProps> = ({ showHelpCenter }) => {
  return (
    <div className="menu-dropdown">
      <ul className="menu-list">
        <li className="menu-item" onClick={showHelpCenter}>
          <div className="menu-item-icon">
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm2.07-7.75l-.9.92C13.45 12.9 13 13.5 13 15h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H8c0-2.21 1.79-4 4-4s4 1.79 4 4c0 .88-.36 1.68-.93 2.25z" />
            </svg>
          </div>
          <span>Help Center</span>
          <div className="menu-item-shortcut">F1</div>
        </li>
        <li className="menu-item">
          <div className="menu-item-icon">
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path d="M11 18h2v-2h-2v2zm1-16C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm0-14c-2.21 0-4 1.79-4 4h2c0-1.1.9-2 2-2s2 .9 2 2c0 2-3 1.75-3 5h2c0-2.25 3-2.5 3-5 0-2.21-1.79-4-4-4z" />
            </svg>
          </div>
          <span>View Documentation</span>
        </li>
        <li className="menu-divider"></li>
        <li className="menu-item">
          <div className="menu-item-icon">
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
            </svg>
          </div>
          <span>Check for Updates</span>
        </li>
        <li className="menu-item">
          <div className="menu-item-icon">
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path d="M11 9h2V7h-2v2zm0 8h2v-6h-2v6zm1-15C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z" />
            </svg>
          </div>
          <span>About Papin</span>
        </li>
      </ul>
    </div>
  );
};

export default HelpMenu;
