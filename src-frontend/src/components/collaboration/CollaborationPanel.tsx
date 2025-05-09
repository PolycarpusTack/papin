// CollaborationPanel.tsx
//
// Main panel for collaboration features including:
// - Session management
// - User list
// - Call controls
// - Whiteboard
// - Collaboration settings

import React, { useState, useEffect } from 'react';
import { useCollaboration } from '../../hooks/useCollaboration';
import UserList from './UserList';
import { ConnectionStatus, UserRole } from './context/CollaborationContext';
import CallControls from './call/CallControls';
import CollaborationSettings from './settings/CollaborationSettings';
import CollaborativeWhiteboard from './whiteboard/CollaborativeWhiteboard';

// Import whiteboard styles
import '../../styles/whiteboard.css';

// Tab options
enum Tab {
  Users,
  Call,
  Whiteboard,
  Settings,
}

interface CollaborationPanelProps {
  conversationId: string;
}

const CollaborationPanel: React.FC<CollaborationPanelProps> = ({ conversationId }) => {
  const { state, createSession, joinSession, leaveSession } = useCollaboration();
  const { currentSession, connectionStatus, config } = state;
  
  const [activeTab, setActiveTab] = useState<Tab>(Tab.Users);
  const [sessionName, setSessionName] = useState<string>('');
  const [sessionIdToJoin, setSessionIdToJoin] = useState<string>('');
  const [showCreateDialog, setShowCreateDialog] = useState<boolean>(false);
  const [showJoinDialog, setShowJoinDialog] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    // Reset the form when dialog visibility changes
    if (!showCreateDialog) {
      setSessionName('');
    }
    if (!showJoinDialog) {
      setSessionIdToJoin('');
    }
  }, [showCreateDialog, showJoinDialog]);
  
  // Handle creating a new session
  const handleCreateSession = async () => {
    if (!sessionName.trim()) {
      setError('Please enter a session name');
      return;
    }
    
    setIsLoading(true);
    setError(null);
    
    try {
      await createSession(sessionName, conversationId);
      setShowCreateDialog(false);
    } catch (err) {
      setError(`Failed to create session: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };
  
  // Handle joining an existing session
  const handleJoinSession = async () => {
    if (!sessionIdToJoin.trim()) {
      setError('Please enter a session ID');
      return;
    }
    
    setIsLoading(true);
    setError(null);
    
    try {
      await joinSession(sessionIdToJoin);
      setShowJoinDialog(false);
    } catch (err) {
      setError(`Failed to join session: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };
  
  // Handle leaving the current session
  const handleLeaveSession = async () => {
    setIsLoading(true);
    
    try {
      await leaveSession();
    } catch (err) {
      console.error('Failed to leave session:', err);
    } finally {
      setIsLoading(false);
    }
  };
  
  // Render connection status indicator
  const renderConnectionStatus = () => {
    let color = '#9E9E9E';
    let label = 'Disconnected';
    
    switch (connectionStatus) {
      case ConnectionStatus.Connected:
        color = '#4CAF50';
        label = 'Connected';
        break;
      case ConnectionStatus.Connecting:
        color = '#FFC107';
        label = 'Connecting';
        break;
      case ConnectionStatus.Limited:
        color = '#FF9800';
        label = 'Limited';
        break;
      case ConnectionStatus.Error:
        color = '#F44336';
        label = 'Error';
        break;
      default:
        break;
    }
    
    return (
      <div style={{ display: 'flex', alignItems: 'center' }}>
        <div
          style={{
            width: '10px',
            height: '10px',
            borderRadius: '50%',
            backgroundColor: color,
            marginRight: '6px',
          }}
        />
        <span>{label}</span>
      </div>
    );
  };
  
  // Render session details
  const renderSessionDetails = () => {
    if (!currentSession) {
      return (
        <div style={{ padding: '20px', textAlign: 'center' }}>
          <p>You are not in a collaborative session.</p>
          <div style={{ display: 'flex', justifyContent: 'center', gap: '10px', marginTop: '15px' }}>
            <button
              onClick={() => setShowCreateDialog(true)}
              disabled={isLoading}
              className="collaboration-button primary"
            >
              Create Session
            </button>
            <button
              onClick={() => setShowJoinDialog(true)}
              disabled={isLoading}
              className="collaboration-button primary"
            >
              Join Session
            </button>
          </div>
        </div>
      );
    }
    
    return (
      <div style={{ padding: '10px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '15px' }}>
          <div>
            <h3 style={{ margin: '0 0 5px 0' }}>{currentSession.name}</h3>
            <div style={{ fontSize: '12px', color: '#757575' }}>
              Session ID: {currentSession.id}
            </div>
          </div>
          <button
            onClick={handleLeaveSession}
            disabled={isLoading}
            className="collaboration-button danger"
          >
            Leave
          </button>
        </div>
        
        {/* Tabs navigation */}
        <div className="collaboration-tabs">
          <button
            onClick={() => setActiveTab(Tab.Users)}
            className={`collaboration-tab ${activeTab === Tab.Users ? 'active' : ''}`}
          >
            Users
          </button>
          
          {config.enable_av && (
            <button
              onClick={() => setActiveTab(Tab.Call)}
              className={`collaboration-tab ${activeTab === Tab.Call ? 'active' : ''}`}
            >
              Call
            </button>
          )}
          
          <button
            onClick={() => setActiveTab(Tab.Whiteboard)}
            className={`collaboration-tab ${activeTab === Tab.Whiteboard ? 'active' : ''}`}
          >
            Whiteboard
          </button>
          
          <button
            onClick={() => setActiveTab(Tab.Settings)}
            className={`collaboration-tab ${activeTab === Tab.Settings ? 'active' : ''}`}
          >
            Settings
          </button>
        </div>
        
        {/* Tab content */}
        <div style={{ padding: '10px' }}>
          {activeTab === Tab.Users && <UserList />}
          {activeTab === Tab.Call && <CallControls />}
          {activeTab === Tab.Whiteboard && (
            <CollaborativeWhiteboard 
              sessionId={currentSession.id}
              width={280}
              height={400}
            />
          )}
          {activeTab === Tab.Settings && <CollaborationSettings />}
        </div>
      </div>
    );
  };
  
  // Create session dialog
  const renderCreateSessionDialog = () => {
    if (!showCreateDialog) return null;
    
    return (
      <div className="collaboration-dialog-overlay">
        <div className="collaboration-dialog">
          <h3 className="collaboration-dialog-title">Create New Collaboration Session</h3>
          
          {error && (
            <div className="collaboration-error">
              {error}
            </div>
          )}
          
          <div className="collaboration-form-group">
            <label 
              htmlFor="sessionName" 
              className="collaboration-label"
            >
              Session Name
            </label>
            <input
              id="sessionName"
              type="text"
              value={sessionName}
              onChange={(e) => setSessionName(e.target.value)}
              placeholder="Enter a name for your session"
              className="collaboration-input"
            />
          </div>
          
          <div className="collaboration-button-group">
            <button
              onClick={() => setShowCreateDialog(false)}
              disabled={isLoading}
              className="collaboration-button secondary"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateSession}
              disabled={isLoading || !sessionName.trim()}
              className="collaboration-button primary"
              style={{
                opacity: isLoading || !sessionName.trim() ? 0.7 : 1,
              }}
            >
              {isLoading ? 'Creating...' : 'Create Session'}
            </button>
          </div>
        </div>
      </div>
    );
  };
  
  // Join session dialog
  const renderJoinSessionDialog = () => {
    if (!showJoinDialog) return null;
    
    return (
      <div className="collaboration-dialog-overlay">
        <div className="collaboration-dialog">
          <h3 className="collaboration-dialog-title">Join Collaboration Session</h3>
          
          {error && (
            <div className="collaboration-error">
              {error}
            </div>
          )}
          
          <div className="collaboration-form-group">
            <label 
              htmlFor="sessionId" 
              className="collaboration-label"
            >
              Session ID
            </label>
            <input
              id="sessionId"
              type="text"
              value={sessionIdToJoin}
              onChange={(e) => setSessionIdToJoin(e.target.value)}
              placeholder="Enter the session ID to join"
              className="collaboration-input"
            />
          </div>
          
          <div className="collaboration-button-group">
            <button
              onClick={() => setShowJoinDialog(false)}
              disabled={isLoading}
              className="collaboration-button secondary"
            >
              Cancel
            </button>
            <button
              onClick={handleJoinSession}
              disabled={isLoading || !sessionIdToJoin.trim()}
              className="collaboration-button primary"
              style={{
                opacity: isLoading || !sessionIdToJoin.trim() ? 0.7 : 1,
              }}
            >
              {isLoading ? 'Joining...' : 'Join Session'}
            </button>
          </div>
        </div>
      </div>
    );
  };
  
  return (
    <div className="collaboration-panel">
      {/* Header with status */}
      <div className="collaboration-panel-header">
        <h3 className="collaboration-panel-title">Collaboration</h3>
        {renderConnectionStatus()}
      </div>
      
      {/* Main content */}
      <div className="collaboration-panel-content">
        {renderSessionDetails()}
      </div>
      
      {/* Dialogs */}
      {renderCreateSessionDialog()}
      {renderJoinSessionDialog()}
    </div>
  );
};

export default CollaborationPanel;
