import React, { useState, useRef, useEffect } from 'react';
import { Button } from '../ui/Button';
import './ChatInput.css';

interface ChatInputProps {
  onSendMessage: (message: string) => void;
  isDisabled?: boolean;
  placeholder?: string;
}

/**
 * Expandable text input for the chat interface
 */
const ChatInput: React.FC<ChatInputProps> = ({ 
  onSendMessage,
  isDisabled = false,
  placeholder = "Type a message..."
}) => {
  const [inputValue, setInputValue] = useState('');
  const inputRef = useRef<HTMLTextAreaElement>(null);
  
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
  
  // Auto-resize when inputValue changes
  useEffect(() => {
    autoResizeTextarea();
  }, [inputValue]);
  
  // Focus the textarea when the component mounts
  useEffect(() => {
    if (inputRef.current && !isDisabled) {
      inputRef.current.focus();
    }
  }, [isDisabled]);
  
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
  
  // Handle sending a message
  const handleSendMessage = () => {
    if (!inputValue.trim() || isDisabled) return;
    
    onSendMessage(inputValue);
    setInputValue('');
    
    // Reset the textarea height
    if (inputRef.current) {
      inputRef.current.style.height = '56px';
    }
  };
  
  return (
    <div className="chat-input-container">
      <textarea 
        ref={inputRef}
        className="chat-input" 
        placeholder={placeholder}
        value={inputValue}
        onChange={handleInputChange}
        onKeyDown={handleKeyDown}
        rows={1}
        disabled={isDisabled}
        data-testid="chat-input"
      />
      <Button 
        onClick={handleSendMessage}
        disabled={!inputValue.trim() || isDisabled}
        className="send-button"
        data-testid="send-button"
      >
        <span className="send-button-text">Send</span>
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
          <path d="M15.964.686a.5.5 0 0 0-.65-.65L.767 5.855H.766l-.452.18a.5.5 0 0 0-.082.887l.41.26.001.002 4.995 3.178 3.178 4.995.002.002.26.41a.5.5 0 0 0 .886-.083l6-15Zm-1.833 1.89L6.637 10.07l-.215-.338a.5.5 0 0 0-.154-.154l-.338-.215 7.494-7.494 1.178-.471-.47 1.178Z"/>
        </svg>
      </Button>
    </div>
  );
};

export default ChatInput;