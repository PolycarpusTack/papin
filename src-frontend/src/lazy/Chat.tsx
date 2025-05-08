import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Button } from '../components/ui/Button';
import './Chat.css';

// Define message types
interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

// Chat component - lazy loaded after shell is ready
const Chat: React.FC = () => {
  const [loaded, setLoaded] = useState(false);
  const [loading, setLoading] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  
  // Generate a unique ID for messages
  const generateId = () => {
    return `msg_${Date.now()}_${Math.floor(Math.random() * 1000)}`;
  };
  
  // Scroll to the latest message
  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };
  
  // Auto-resize the textarea based on content
  const autoResizeTextarea = () => {
    if (inputRef.current) {
      // Reset height to auto to correctly calculate the new height
      inputRef.current.style.height = 'auto';
      // Set the new height based on the scroll height
      const newHeight = Math.min(
        Math.max(56, inputRef.current.scrollHeight), // Min height 56px
        200 // Max height 200px
      );
      inputRef.current.style.height = `${newHeight}px`;
    }
  };
  
  // Load initial chat data
  useEffect(() => {
    const loadChat = async () => {
      try {
        // In a real app, we would fetch messages from backend
        // const initialMessages = await invoke<Message[]>('get_chat_messages');
        
        // For now, just add a system welcome message
        const welcomeMessage: Message = {
          id: generateId(),
          role: 'system',
          content: 'Welcome to Claude MCP client. How can I help you today?',
          timestamp: Date.now(),
        };
        
        setMessages([welcomeMessage]);
        setLoaded(true);
      } catch (error) {
        console.error('Failed to load chat:', error);
      }
    };
    
    loadChat();
  }, []);
  
  // Auto-resize textarea and scroll to bottom when messages change
  useEffect(() => {
    autoResizeTextarea();
    scrollToBottom();
  }, [messages, inputValue]);
  
  // Focus the textarea when the component loads
  useEffect(() => {
    if (loaded && inputRef.current) {
      inputRef.current.focus();
    }
  }, [loaded]);
  
  // Handle sending a message
  const handleSendMessage = async () => {
    if (!inputValue.trim()) return;
    
    const userMessage: Message = {
      id: generateId(),
      role: 'user',
      content: inputValue,
      timestamp: Date.now(),
    };
    
    setMessages(prev => [...prev, userMessage]);
    setInputValue('');
    setLoading(true);
    
    try {
      // In a real app, we would send the message to the backend
      // const response = await invoke<Message>('send_message', { content: inputValue });
      
      // For now, just simulate a response after a delay
      setTimeout(() => {
        const assistantMessage: Message = {
          id: generateId(),
          role: 'assistant',
          content: `I received your message: "${inputValue}". This is a placeholder response as the MCP backend is not yet connected.`,
          timestamp: Date.now(),
        };
        
        setMessages(prev => [...prev, assistantMessage]);
        setLoading(false);
      }, 1000);
    } catch (error) {
      console.error('Failed to send message:', error);
      setLoading(false);
      
      // Add an error message
      const errorMessage: Message = {
        id: generateId(),
        role: 'system',
        content: 'Failed to send message. Please try again.',
        timestamp: Date.now(),
      };
      
      setMessages(prev => [...prev, errorMessage]);
    }
  };
  
  // Handle textarea input changes
  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInputValue(e.target.value);
  };
  
  // Handle keyboard shortcuts
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Send message on Enter without Shift
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };
  
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
        <div className="chat-status online">Connected</div>
      </div>
      
      <div className="chat-messages">
        {messages.map((message) => (
          <div 
            key={message.id} 
            className={`message ${message.role}`}
          >
            <div className="message-avatar">
              {message.role === 'user' ? 'U' : message.role === 'assistant' ? 'C' : 'S'}
            </div>
            <div className="message-content-wrapper">
              <div className="message-content">
                {message.content}
              </div>
              <div className="message-time">
                {new Date(message.timestamp).toLocaleTimeString(undefined, {
                  hour: '2-digit',
                  minute: '2-digit',
                })}
              </div>
            </div>
          </div>
        ))}
        {loading && (
          <div className="message assistant loading">
            <div className="message-avatar">C</div>
            <div className="message-content-wrapper">
              <div className="message-content">
                <div className="typing-indicator">
                  <span></span>
                  <span></span>
                  <span></span>
                </div>
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      
      <div className="chat-input-container">
        <textarea 
          ref={inputRef}
          className="chat-input" 
          placeholder="Type a message..."
          value={inputValue}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          rows={1}
        />
        <Button 
          onClick={handleSendMessage}
          disabled={!inputValue.trim() || loading}
          className="send-button"
        >
          Send
        </Button>
      </div>
    </div>
  );
};

export default Chat;
