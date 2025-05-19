import { invoke } from '@tauri-apps/api/tauri';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  status?: 'sending' | 'streaming' | 'complete' | 'failed' | 'cancelled';
}

export interface Conversation {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  model_id: string;
  message_count: number;
}

/**
 * API for chat-related operations
 */
class ChatApi {
  /**
   * Get available AI models
   */
  async getAvailableModels() {
    try {
      return await invoke<any[]>('get_available_models');
    } catch (error) {
      console.error('Failed to get available models:', error);
      throw error;
    }
  }
  
  /**
   * Create a new conversation
   */
  async createConversation(title: string, modelId: string) {
    try {
      return await invoke<Conversation>('create_conversation', { 
        title, 
        modelId 
      });
    } catch (error) {
      console.error('Failed to create conversation:', error);
      throw error;
    }
  }
  
  /**
   * Get a conversation by ID
   */
  async getConversation(id: string) {
    try {
      return await invoke<Conversation>('get_conversation', { id });
    } catch (error) {
      console.error(`Failed to get conversation ${id}:`, error);
      throw error;
    }
  }
  
  /**
   * Get all conversations
   */
  async getConversations() {
    try {
      return await invoke<Conversation[]>('get_conversations');
    } catch (error) {
      console.error('Failed to get conversations:', error);
      throw error;
    }
  }
  
  /**
   * Delete a conversation
   */
  async deleteConversation(id: string) {
    try {
      return await invoke<void>('delete_conversation', { id });
    } catch (error) {
      console.error(`Failed to delete conversation ${id}:`, error);
      throw error;
    }
  }
  
  /**
   * Get messages for a conversation
   */
  async getMessages(conversationId: string) {
    try {
      const messages = await invoke<any[]>('get_messages', { conversationId });
      
      // Transform the messages to the expected format
      return messages.map(msg => {
        const message = msg.message;
        return {
          id: message.id,
          role: message.role,
          content: message.content,
          timestamp: message.timestamp || Date.now(),
          status: msg.status
        } as Message;
      });
    } catch (error) {
      console.error(`Failed to get messages for conversation ${conversationId}:`, error);
      throw error;
    }
  }
  
  /**
   * Send a message in a conversation
   */
  async sendMessage(conversationId: string, content: string) {
    try {
      const response = await invoke<any>('send_message', {
        conversationId,
        content
      });
      
      const message = response.message;
      return {
        id: message.id,
        role: message.role,
        content: message.content,
        timestamp: message.timestamp || Date.now(),
        status: response.status
      } as Message;
    } catch (error) {
      console.error(`Failed to send message in conversation ${conversationId}:`, error);
      throw error;
    }
  }
  
  /**
   * Register for streaming message updates
   */
  async registerForStreamingUpdates(
    conversationId: string,
    onPartialContent: (messageId: string, content: string) => void,
    onComplete: (messageId: string) => void
  ) {
    // In a real implementation, we would use events to receive streaming updates
    // For now, we'll mock this functionality
    
    // Return an unsubscribe function
    return () => {
      // Unsubscribe from streaming updates
    };
  }
}

export default new ChatApi();