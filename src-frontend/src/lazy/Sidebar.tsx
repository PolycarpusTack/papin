import React, { useState } from 'react';
import { Button } from '../components/ui/Button';
import './Sidebar.css';

interface SidebarProps {
  currentView: string;
}

// Define a conversation type for chat history
interface Conversation {
  id: string;
  title: string;
  timestamp: number;
  preview: string;
}

const Sidebar: React.FC<SidebarProps> = ({ currentView }) => {
  const [conversations, setConversations] = useState<Conversation[]>([
    {
      id: 'conv_1',
      title: 'Getting Started with MCP',
      timestamp: Date.now() - 1000 * 60 * 15, // 15 minutes ago
      preview: 'Learn about the Model Context Protocol...',
    },
    {
      id: 'conv_2',
      title: 'API Integration',
      timestamp: Date.now() - 1000 * 60 * 60 * 2, // 2 hours ago
      preview: 'How to integrate with Claude API...',
    },
    {
      id: 'conv_3',
      title: 'Streaming vs Non-Streaming',
      timestamp: Date.now() - 1000 * 60 * 60 * 24, // 1 day ago
      preview: 'Comparing streaming and non-streaming modes...',
    },
  ]);
  
  // Show appropriate sidebar content based on current view
  if (currentView === 'settings') {
    return (
      <aside className="sidebar">
        <div className="sidebar-header">
          <h3>Settings</h3>
        </div>
        <nav className="sidebar-nav">
          <button className="sidebar-nav-item active">General</button>
          <button className="sidebar-nav-item">API Keys</button>
          <button className="sidebar-nav-item">Models</button>
          <button className="sidebar-nav-item">Appearance</button>
          <button className="sidebar-nav-item">Advanced</button>
        </nav>
      </aside>
    );
  }
  
  // Default chat sidebar
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h3>Conversations</h3>
        <Button 
          variant="primary"
          size="sm"
          className="new-chat-button"
        >
          New Chat
        </Button>
      </div>
      
      <div className="conversation-list">
        {conversations.map((conversation) => (
          <div key={conversation.id} className="conversation-item">
            <div className="conversation-title">{conversation.title}</div>
            <div className="conversation-preview">{conversation.preview}</div>
            <div className="conversation-time">
              {formatTime(conversation.timestamp)}
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
};

// Helper function to format timestamps
const formatTime = (timestamp: number): string => {
  const now = Date.now();
  const diff = now - timestamp;
  
  // Less than a minute
  if (diff < 1000 * 60) {
    return 'Just now';
  }
  
  // Less than an hour
  if (diff < 1000 * 60 * 60) {
    const minutes = Math.floor(diff / (1000 * 60));
    return `${minutes}m ago`;
  }
  
  // Less than a day
  if (diff < 1000 * 60 * 60 * 24) {
    const hours = Math.floor(diff / (1000 * 60 * 60));
    return `${hours}h ago`;
  }
  
  // Otherwise show the date
  return new Date(timestamp).toLocaleDateString();
};

export default Sidebar;
