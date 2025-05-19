import React, { useEffect, useRef } from 'react';
import { parseMarkdown, initializeCodeCopyButtons, linkify } from '../../utils/markdown';
import './MessageContent.css';

interface MessageContentProps {
  content: string;
  role: 'user' | 'assistant' | 'system';
}

/**
 * Component to render message content with proper formatting based on the role
 * - Assistant messages use markdown formatting
 * - User messages have basic text with link detection
 * - System messages are displayed as plain text with link detection
 */
const MessageContent: React.FC<MessageContentProps> = ({ content, role }) => {
  const contentRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    // Only initialize copy buttons after component has rendered
    if (role === 'assistant' && contentRef.current) {
      initializeCodeCopyButtons();
    }
  }, [content, role]);
  
  // For assistant messages, parse markdown with code highlighting
  if (role === 'assistant') {
    return (
      <div 
        ref={contentRef}
        className="message-content-markdown"
        dangerouslySetInnerHTML={{ __html: parseMarkdown(content) }}
      />
    );
  }
  
  // For user messages, simple formatting with linkification
  if (role === 'user') {
    return (
      <div
        className="message-content-text"
        dangerouslySetInnerHTML={{ __html: linkify(content) }}
      />
    );
  }
  
  // For system messages, italicized with linkification
  return (
    <div
      className="message-content-system"
      dangerouslySetInnerHTML={{ __html: linkify(content) }}
    />
  );
};

export default MessageContent;