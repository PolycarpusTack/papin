import React from 'react';
import './Sidebar.css';

// Props for Sidebar component
interface SidebarProps {
  currentView: string;
}

// Sidebar component - lazy loaded after shell is ready
const Sidebar: React.FC<SidebarProps> = ({ currentView }) => {
  return (
    <div className="sidebar fade-in">
      <div className="sidebar-header">
        <h3>Conversations</h3>
        <button className="new-chat-button">+</button>
      </div>
      
      <div className="sidebar-content">
        <div className="conversation-list">
          <div className="conversation active">
            <div className="conversation-title">Welcome</div>
            <div className="conversation-preview">Welcome to Claude MCP!</div>
            <div className="conversation-time">Just now</div>
          </div>
          
          <div className="conversation">
            <div className="conversation-title">Example Conversation</div>
            <div className="conversation-preview">This is a placeholder conversation...</div>
            <div className="conversation-time">Yesterday</div>
          </div>
          
          <div className="conversation">
            <div className="conversation-title">Another Topic</div>
            <div className="conversation-preview">More placeholder content here...</div>
            <div className="conversation-time">2 days ago</div>
          </div>
        </div>
      </div>
      
      <div className="sidebar-footer">
        <button className="sidebar-button">
          <span className="button-icon">⚙️</span>
          <span className="button-text">Settings</span>
        </button>
      </div>
    </div>
  );
};

export default Sidebar;