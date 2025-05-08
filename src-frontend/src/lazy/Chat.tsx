import React, { useState, useEffect } from 'react';
import './Chat.css';

// Chat component - lazy loaded after shell is ready
const Chat: React.FC = () => {
  const [loaded, setLoaded] = useState(false);
  
  useEffect(() => {
    // Simulate loading chat history or other resources
    setTimeout(() => {
      setLoaded(true);
    }, 300);
  }, []);
  
  if (!loaded) {
    return (
      <div className="chat-container">
        <div className="chat-loading">
          <div className="loading-spinner"></div>
          <p>Loading conversation...</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="chat-container fade-in">
      <div className="chat-header">
        <h2>Claude MCP</h2>
        <div className="chat-status online">Online</div>
      </div>
      
      <div className="chat-messages">
        <div className="message system">
          <div className="message-content">
            Welcome to Claude MCP! This is a placeholder for the chat interface.
          </div>
          <div className="message-time">Just now</div>
        </div>
      </div>
      
      <div className="chat-input-container">
        <textarea 
          className="chat-input" 
          placeholder="Type a message..."
          rows={3}
        />
        <button className="send-button">Send</button>
      </div>
    </div>
  );
};

export default Chat;