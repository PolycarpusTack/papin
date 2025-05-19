import React from 'react';
import MessageContent from './MessageContent';
import './Message.css';

interface MessageProps {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  status?: 'sending' | 'streaming' | 'complete' | 'failed';
  isLastMessage?: boolean;
}

/**
 * Message component for displaying chat messages
 */
const Message: React.FC<MessageProps> = ({ 
  id, 
  role, 
  content, 
  timestamp, 
  status = 'complete',
  isLastMessage = false
}) => {
  // Generate the avatar text based on role
  const getAvatarText = () => {
    switch(role) {
      case 'user':
        return 'U';
      case 'assistant':
        return 'C';
      case 'system':
        return 'S';
      default:
        return '?';
    }
  };
  
  // Generate status indicator if needed
  const renderStatusIndicator = () => {
    if (role !== 'user' || status === 'complete') return null;
    
    let statusText = '';
    let statusClass = '';
    
    switch(status) {
      case 'sending':
        statusText = 'Sending...';
        statusClass = 'status-sending';
        break;
      case 'streaming':
        statusText = 'Receiving...';
        statusClass = 'status-streaming';
        break;
      case 'failed':
        statusText = 'Failed to send';
        statusClass = 'status-failed';
        break;
    }
    
    return (
      <div className={`message-status ${statusClass}`}>
        {statusText}
      </div>
    );
  };
  
  // Typing indicator for streaming assistant messages
  const renderTypingIndicator = () => {
    if (role !== 'assistant' || status !== 'streaming' || !isLastMessage) return null;
    
    return (
      <div className="typing-indicator">
        <span></span>
        <span></span>
        <span></span>
      </div>
    );
  };
  
  return (
    <div className={`message ${role} ${status !== 'complete' ? 'in-progress' : ''}`}>
      <div className="message-avatar">
        {getAvatarText()}
      </div>
      <div className="message-content-wrapper">
        <div className="message-bubble">
          <MessageContent content={content} role={role} />
          {renderTypingIndicator()}
        </div>
        <div className="message-footer">
          <div className="message-time">
            {new Date(timestamp).toLocaleTimeString(undefined, {
              hour: '2-digit',
              minute: '2-digit',
            })}
          </div>
          {renderStatusIndicator()}
        </div>
      </div>
    </div>
  );
};

export default Message;