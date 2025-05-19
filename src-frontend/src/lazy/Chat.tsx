import React, { useState, useEffect } from 'react';
import ConversationPanel from '../components/chat/ConversationPanel';
import ConversationList from '../components/chat/ConversationList';
import './Chat.css';
import { invoke } from '@tauri-apps/api/tauri';
import ChatApi, { Conversation } from '../api/ChatApi';

// Chat component - main chat interface with conversations list and active conversation
const Chat: React.FC = () => {
  const [loaded, setLoaded] = useState(false);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<string | undefined>(undefined);
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Load conversations on component mount
  useEffect(() => {
    const loadConversations = async () => {
      try {
        setLoading(true);
        const conversationsData = await ChatApi.getConversations();
        setConversations(conversationsData);
        
        // Set the most recent conversation as active if available
        if (conversationsData.length > 0) {
          const sortedConversations = [...conversationsData].sort(
            (a, b) => b.updated_at - a.updated_at
          );
          setActiveConversationId(sortedConversations[0].id);
        }
        
        setLoaded(true);
      } catch (err) {
        console.error('Failed to load conversations:', err);
        setError('Failed to load conversations');
      } finally {
        setLoading(false);
      }
    };
    
    loadConversations();
  }, []);
  
  // Toggle sidebar visibility
  const toggleSidebar = () => {
    setSidebarVisible(!sidebarVisible);
  };
  
  // Create a new conversation
  const createNewConversation = async () => {
    try {
      const defaultModel = "claude-3-opus-20240229"; // This would come from user settings
      const newConversation = await ChatApi.createConversation("New Conversation", defaultModel);
      
      // Add to conversations list
      setConversations(prev => [newConversation, ...prev]);
      
      // Set as active conversation
      setActiveConversationId(newConversation.id);
    } catch (err) {
      console.error('Failed to create conversation:', err);
      setError('Failed to create new conversation');
    }
  };
  
  // Handle selecting a conversation
  const handleConversationSelect = (conversationId: string) => {
    setActiveConversationId(conversationId);
    
    // On mobile, hide sidebar after selection
    if (window.innerWidth < 768) {
      setSidebarVisible(false);
    }
  };
  
  // Delete a conversation
  const handleDeleteConversation = async (conversationId: string) => {
    try {
      await ChatApi.deleteConversation(conversationId);
      
      // Remove from list
      setConversations(prev => prev.filter(c => c.id !== conversationId));
      
      // If active conversation was deleted, select another one
      if (activeConversationId === conversationId) {
        const remaining = conversations.filter(c => c.id !== conversationId);
        if (remaining.length > 0) {
          setActiveConversationId(remaining[0].id);
        } else {
          setActiveConversationId(undefined);
        }
      }
    } catch (err) {
      console.error('Failed to delete conversation:', err);
      setError('Failed to delete conversation');
    }
  };
  
  if (!loaded) {
    return (
      <div className="chat-container">
        <div className="chat-loading">
          <div className="loading-spinner"></div>
          <p>Loading conversations...</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="chat-container">
      {error && (
        <div className="chat-error">
          <p>{error}</p>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}
      
      <div className={`chat-layout ${sidebarVisible ? 'sidebar-visible' : 'sidebar-hidden'}`}>
        <div className="chat-sidebar">
          <div className="sidebar-header">
            <h2>Conversations</h2>
            <button 
              className="new-conversation-button"
              onClick={createNewConversation}
              aria-label="New conversation"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                <path d="M8 0a1 1 0 0 1 1 1v6h6a1 1 0 1 1 0 2H9v6a1 1 0 1 1-2 0V9H1a1 1 0 0 1 0-2h6V1a1 1 0 0 1 1-1z"/>
              </svg>
              New
            </button>
          </div>
          
          <ConversationList 
            conversations={conversations}
            activeConversationId={activeConversationId}
            onSelect={handleConversationSelect}
            onDelete={handleDeleteConversation}
          />
        </div>
        
        <div className="main-content">
          <button 
            className="toggle-sidebar-button"
            onClick={toggleSidebar}
            aria-label={sidebarVisible ? "Hide sidebar" : "Show sidebar"}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 16 16">
              {sidebarVisible ? (
                <path fillRule="evenodd" d="M6 12.5a.5.5 0 0 0 .5.5h8a.5.5 0 0 0 .5-.5v-9a.5.5 0 0 0-.5-.5h-8a.5.5 0 0 0-.5.5v9zm-5-8a.5.5 0 0 0-.5.5v7a.5.5 0 0 0 .5.5h3a.5.5 0 0 0 .5-.5v-7a.5.5 0 0 0-.5-.5h-3z"/>
              ) : (
                <path fillRule="evenodd" d="M2 12.5a.5.5 0 0 1-.5.5h-1a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h1a.5.5 0 0 1 .5.5v9zm3-8.5a.5.5 0 0 0-.5.5v7a.5.5 0 0 0 .5.5h1a.5.5 0 0 0 .5-.5v-7a.5.5 0 0 0-.5-.5h-1zm3.5.5a.5.5 0 0 1 .5-.5h7a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-.5.5h-7a.5.5 0 0 1-.5-.5v-7z"/>
              )}
            </svg>
          </button>
          
          <ConversationPanel 
            conversationId={activeConversationId}
            onNewConversation={createNewConversation}
          />
        </div>
      </div>
    </div>
  );
};

export default Chat;