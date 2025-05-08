import React from 'react';
import ThemeToggle from '../theme/ThemeToggle';
import useCommandPalette from '../hooks/useCommandPalette';
import './Header.css';

interface HeaderProps {
  currentView: string;
  onViewChange: (view: string) => void;
}

const Header: React.FC<HeaderProps> = ({ currentView, onViewChange }) => {
  const { open: openCommandPalette } = useCommandPalette();
  
  return (
    <header className="app-header">
      <div className="header-left">
        <div className="app-logo">
          <div className="logo-icon"></div>
          <span className="logo-text">Claude MCP</span>
        </div>
        
        <nav className="main-nav">
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
      </div>
      
      <div className="header-right">
        <button className="command-palette-button" onClick={openCommandPalette}>
          <span className="command-icon">⌘</span>
          <span>Command Palette</span>
          <kbd className="keyboard-shortcut">Ctrl+K</kbd>
        </button>
        
        <ThemeToggle />
      </div>
    </header>
  );
};

export default Header;
