// AppShell.tsx
//
// This is a sample implementation of the AppShell component with collaboration features.
// It demonstrates how to integrate collaboration into the main UI.

import React, { useRef, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { FeatureFlags } from '../../src/feature_flags';

// Import collaboration components
import { 
  CollaborationProvider, 
  CollaborationPanel,
  CursorOverlay,
  SelectionOverlay,
  ConnectionStatus
} from './collaboration';

// Import styles
import './AppShell.css';
import '../styles/collaboration.css';

interface AppShellProps {
  featureFlags: FeatureFlags;
  // Other props would go here in a real implementation
}

const AppShell: React.FC<AppShellProps> = ({ featureFlags }) => {
  const mainContentRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<HTMLDivElement>(null);
  const [showCollaborationPanel, setShowCollaborationPanel] = useState<boolean>(false);
  const [currentConversationId, setCurrentConversationId] = useState<string>('conversation-123'); // Example ID
  const [collaborationEnabled, setCollaborationEnabled] = useState<boolean>(false);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>(ConnectionStatus.Disconnected);
  
  // Initialize feature flags
  useEffect(() => {
    // Check if collaboration feature is enabled
    if (featureFlags.contains(FeatureFlags.COLLABORATION)) {
      setCollaborationEnabled(true);
    }
  }, [featureFlags]);
  
  // Initialize collaboration system
  useEffect(() => {
    if (collaborationEnabled) {
      const initCollaboration = async () => {
        try {
          // Initialize the collaboration system with default config
          await invoke('init_collaboration_system', { config: null });
          
          // Get the initial connection status
          const status = await invoke<ConnectionStatus>('get_connection_status');
          setConnectionStatus(status);
        } catch (error) {
          console.error('Failed to initialize collaboration system:', error);
        }
      };
      
      initCollaboration();
    }
  }, [collaborationEnabled]);
  
  // Poll for connection status updates
  useEffect(() => {
    if (!collaborationEnabled) return;
    
    const interval = setInterval(async () => {
      try {
        const status = await invoke<ConnectionStatus>('get_connection_status');
        setConnectionStatus(status);
      } catch (error) {
        console.error('Failed to get connection status:', error);
      }
    }, 5000);
    
    return () => clearInterval(interval);
  }, [collaborationEnabled]);
  
  // Update current conversation ID when conversation changes
  const handleConversationChange = (conversationId: string) => {
    setCurrentConversationId(conversationId);
  };
  
  // Toggle collaboration panel
  const toggleCollaborationPanel = () => {
    setShowCollaborationPanel(!showCollaborationPanel);
  };
  
  // Render collaboration status indicator
  const renderCollaborationStatus = () => {
    if (!collaborationEnabled) return null;
    
    let color = '#9E9E9E';
    let title = 'Collaboration: Disconnected';
    
    switch (connectionStatus) {
      case ConnectionStatus.Connected:
        color = '#4CAF50';
        title = 'Collaboration: Connected';
        break;
      case ConnectionStatus.Connecting:
        color = '#FFC107';
        title = 'Collaboration: Connecting';
        break;
      case ConnectionStatus.Limited:
        color = '#FF9800';
        title = 'Collaboration: Limited Connectivity';
        break;
      case ConnectionStatus.Error:
        color = '#F44336';
        title = 'Collaboration: Error';
        break;
      default:
        break;
    }
    
    return (
      <button
        onClick={toggleCollaborationPanel}
        title={title}
        className="collaboration-status-button"
      >
        <svg 
          width="20" 
          height="20" 
          viewBox="0 0 24 24" 
          fill="none" 
          stroke="currentColor" 
          strokeWidth="2" 
          strokeLinecap="round" 
          strokeLinejoin="round"
        >
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
          <circle cx="9" cy="7" r="4"></circle>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
        
        {/* Status indicator dot */}
        <div 
          className="collaboration-status-indicator"
          style={{ backgroundColor: color }}
        />
      </button>
    );
  };
  
  return (
    <CollaborationProvider>
      <div className="app-shell">
        {/* App toolbar with collaboration button */}
        <header className="app-header">
          <div className="app-logo">MCP Client</div>
          <div className="app-toolbar">
            {/* Other toolbar buttons would go here */}
            {collaborationEnabled && renderCollaborationStatus()}
          </div>
        </header>
        
        <div className="app-content">
          {/* Main sidebar */}
          <div className="app-sidebar">
            {/* Sidebar content would go here */}
          </div>
          
          {/* Main content area with conversation */}
          <div className="app-main" ref={mainContentRef}>
            <div className="conversation-container" ref={editorRef}>
              {/* Conversation content would go here */}
              <div id="message-1" className="message">
                This is a sample message that can be edited collaboratively.
              </div>
              <div id="message-2" className="message">
                Multiple users can see each other's cursors and selections when collaboration is enabled.
              </div>
            </div>
            
            {/* Collaboration cursor overlay */}
            {collaborationEnabled && (
              <CursorOverlay containerRef={mainContentRef} />
            )}
            
            {/* Collaboration selection overlay */}
            {collaborationEnabled && (
              <SelectionOverlay editorRef={editorRef} />
            )}
          </div>
          
          {/* Collaboration panel (shown when toggled) */}
          {collaborationEnabled && showCollaborationPanel && (
            <div className="collaboration-panel-container">
              <CollaborationPanel conversationId={currentConversationId} />
            </div>
          )}
        </div>
      </div>
    </CollaborationProvider>
  );
};

export default AppShell;