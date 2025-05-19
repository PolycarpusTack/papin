import React, { useState, useEffect, useRef } from 'react';
import Message from './Message';
import ChatInput from './ChatInput';
import ChatApi, { Message as MessageType, Conversation } from '../../api/ChatApi';
import './ConversationPanel.css';

interface ConversationPanelProps {
  conversationId?: string;
  onNewConversation?: () => void;
}

/**
 * Main conversation panel component
 */
const ConversationPanel: React.FC<ConversationPanelProps> = ({ 
  conversationId,
  onNewConversation 
}) => {
  const [messages, setMessages] = useState<MessageType[]>([]);
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [loading, setLoading] = useState(false);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  // Scroll to the latest message
  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };
  
  // Load conversation and messages
  useEffect(() => {
    const loadConversation = async () => {
      // Reset state when conversation changes
      setMessages([]);
      setConversation(null);
      setError(null);
      
      if (!conversationId) {
        // No conversation ID, show welcome message
        const welcomeMessage: MessageType = {
          id: `welcome_${Date.now()}`,
          role: 'system',
          content: 'Start a new conversation by typing a message below.',
          timestamp: Date.now(),
        };
        setMessages([welcomeMessage]);
        return;
      }
      
      setLoading(true);
      
      try {
        // Load conversation details
        const conversationData = await ChatApi.getConversation(conversationId);
        setConversation(conversationData);
        
        // Load messages
        const messagesData = await ChatApi.getMessages(conversationId);
        setMessages(messagesData);
      } catch (err) {
        console.error('Failed to load conversation:', err);
        setError('Failed to load conversation. Please try again.');
      } finally {
        setLoading(false);
      }
    };
    
    loadConversation();
  }, [conversationId]);
  
  // Scroll to bottom when messages change
  useEffect(() => {
    scrollToBottom();
  }, [messages]);
  
  // Handle sending a message
  const handleSendMessage = async (content: string) => {
    if (!content.trim()) return;
    
    const isNewConversation = !conversationId;
    
    // Create temporary user message
    const tempUserMessage: MessageType = {
      id: `temp_${Date.now()}`,
      role: 'user',
      content,
      timestamp: Date.now(),
      status: 'sending',
    };
    
    // Add temporary message to state
    setMessages(prev => [...prev, tempUserMessage]);
    setSending(true);
    
    try {
      if (isNewConversation) {
        // Create a new conversation if needed
        if (onNewConversation) {
          onNewConversation();
        }
        
        // Since we don't have a real conversation yet, simulate a response
        setTimeout(() => {
          // Update user message status
          setMessages(prev => 
            prev.map(m => 
              m.id === tempUserMessage.id 
                ? { ...m, status: 'complete' } 
                : m
            )
          );
          
          // Add assistant response
          const assistantMessage: MessageType = {
            id: `temp_response_${Date.now()}`,
            role: 'assistant',
            content: 'I\'ve created a new conversation. How can I help you today?',
            timestamp: Date.now(),
          };
          
          setMessages(prev => [...prev, assistantMessage]);
          setSending(false);
        }, 1000);
        
        return;
      }
      
      // Send message to backend
      const userMessage = await ChatApi.sendMessage(conversationId, content);
      
      // Replace temporary message with real one
      setMessages(prev => 
        prev.map(m => 
          m.id === tempUserMessage.id ? userMessage : m
        )
      );
      
      // Add temporary assistant response (will be replaced by streaming)
      const tempAssistantMessage: MessageType = {
        id: `temp_assistant_${Date.now()}`,
        role: 'assistant',
        content: '',
        timestamp: Date.now(),
        status: 'streaming',
      };
      
      setMessages(prev => [...prev, tempAssistantMessage]);
      
      // In a real implementation, we would register for streaming updates here
      // For now, simulate a streaming response
      let responseContent = '';
      const fullResponse = "Thank you for your message. I'm simulating a streaming response for demonstration purposes. In the real implementation, this would come from the MCP backend with actual streaming.";
      
      // Split response into chunks to simulate streaming
      const responseChunks = fullResponse.split(' ');
      
      for (let i = 0; i < responseChunks.length; i++) {
        await new Promise(resolve => setTimeout(resolve, 100));
        responseContent += (i > 0 ? ' ' : '') + responseChunks[i];
        
        // Update the streaming message
        setMessages(prev => 
          prev.map(m => 
            m.id === tempAssistantMessage.id 
              ? { ...m, content: responseContent } 
              : m
          )
        );
      }
      
      // Mark as complete after streaming
      setTimeout(() => {
        setMessages(prev => 
          prev.map(m => 
            m.id === tempAssistantMessage.id 
              ? { ...m, status: 'complete' } 
              : m
          )
        );
        setSending(false);
      }, 500);
      
    } catch (err) {
      console.error('Failed to send message:', err);
      setError('Failed to send message. Please try again.');
      
      // Update temporary message to show error
      setMessages(prev => 
        prev.map(m => 
          m.id === tempUserMessage.id 
            ? { ...m, status: 'failed' } 
            : m
        )
      );
      setSending(false);
    }
  };
  
  // Show loading state
  if (loading) {
    return (
      <div className="conversation-panel">
        <div className="loading-indicator">
          <div className="loading-spinner"></div>
          <p>Loading conversation...</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="conversation-panel">
      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button 
            className="error-dismiss"
            onClick={() => setError(null)}
            aria-label="Dismiss error"
          >
            ×
          </button>
        </div>
      )}
      
      <div className="conversation-header">
        <h2>{conversation?.title || 'New Conversation'}</h2>
        {conversation && (
          <div className="conversation-model">
            Model: {conversation.model_id}
          </div>
        )}
      </div>
      
      <div className="messages-container">
        {messages.map((message, index) => (
          <Message
            key={message.id}
            id={message.id}
            role={message.role}
            content={message.content}
            timestamp={message.timestamp}
            status={message.status}
            isLastMessage={index === messages.length - 1}
          />
        ))}
        <div ref={messagesEndRef} />
      </div>
      
      <ChatInput 
        onSendMessage={handleSendMessage}
        isDisabled={sending}
        placeholder={sending ? "Waiting for response..." : "Type a message..."}
      />
    </div>
  );
};

export default ConversationPanel;