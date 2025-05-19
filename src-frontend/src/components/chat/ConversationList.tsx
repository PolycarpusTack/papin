import React from 'react';
import { Conversation } from '../../api/ChatApi';
import './ConversationList.css';

interface ConversationListProps {
  conversations: Conversation[];
  activeConversationId?: string;
  onSelect: (conversationId: string) => void;
  onDelete: (conversationId: string) => void;
}

/**
 * Sidebar conversation list component
 */
const ConversationList: React.FC<ConversationListProps> = ({
  conversations,
  activeConversationId,
  onSelect,
  onDelete
}) => {
  // Format date for display
  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp);
    const now = new Date();
    
    // If today, show time
    if (date.toDateString() === now.toDateString()) {
      return date.toLocaleTimeString(undefined, {
        hour: '2-digit',
        minute: '2-digit',
      });
    }
    
    // If this year, show month and day
    if (date.getFullYear() === now.getFullYear()) {
      return date.toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
      });
    }
    
    // Otherwise show date with year
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };
  
  // Handle delete with confirmation
  const handleDelete = (e: React.MouseEvent, conversationId: string) => {
    e.stopPropagation();
    
    // Confirm deletion
    if (window.confirm('Are you sure you want to delete this conversation?')) {
      onDelete(conversationId);
    }
  };
  
  // If no conversations, show empty state
  if (conversations.length === 0) {
    return (
      <div className="conversation-list empty">
        <p>No conversations yet</p>
        <p className="empty-help">Start a new conversation using the button above.</p>
      </div>
    );
  }
  
  return (
    <div className="conversation-list">
      {conversations.map(conversation => (
        <div
          key={conversation.id}
          className={`conversation-item ${activeConversationId === conversation.id ? 'active' : ''}`}
          onClick={() => onSelect(conversation.id)}
        >
          <div className="conversation-info">
            <div className="conversation-title">{conversation.title}</div>
            <div className="conversation-meta">
              <span className="conversation-date">
                {formatDate(conversation.updated_at)}
              </span>
              <span className="conversation-count">
                {conversation.message_count} {conversation.message_count === 1 ? 'message' : 'messages'}
              </span>
            </div>
          </div>
          
          <button
            className="conversation-delete"
            onClick={(e) => handleDelete(e, conversation.id)}
            aria-label="Delete conversation"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
              <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"/>
              <path fillRule="evenodd" d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1zM4.118 4 4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/>
            </svg>
          </button>
        </div>
      ))}
    </div>
  );
};

export default ConversationList;