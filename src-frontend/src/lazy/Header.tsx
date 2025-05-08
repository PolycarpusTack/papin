import React from 'react';
import './Header.css';

interface HeaderProps {
  currentView: string;
  onViewChange: (view: string) => void;
}

const Header: React.FC<HeaderProps> = ({ currentView, onViewChange }) => {
  return (
    <header className="app-header fade-in">
      <div className="header-logo">
        <div className="header-icon"></div>
        <h1>Claude MCP</h1>
      </div>
      
      <nav className="header-nav">
        <button 
          className={`nav-item ${currentView === 'chat' ? 'active' : ''}`}
          onClick={() => onViewChange('chat')}
        >
          Chat
        </button>
        <button 
          className={`nav-item ${currentView === 'settings' ? 'active' : ''}`}
          onClick={() => onViewChange('settings')}
        >
          Settings
        </button>
      </nav>
      
      <div className="header-actions">
        <button className="action-button" title="Help">
          ?
        </button>
        <div className="user-menu">
          <div className="user-avatar"></div>
        </div>
      </div>
    </header>
  );
};

export default Header;