import React from 'react';
import { render, screen, act, waitFor } from '@testing-library/react';
import { CollaborationProvider, useCollaboration } from '../../components/collaboration';
import { invoke } from '@tauri-apps/api/tauri';

// Mock Tauri invoke function
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn(),
}));

// Helper component that uses the collaboration context
const TestComponent = () => {
  const { state, createSession, joinSession } = useCollaboration();
  
  return (
    <div>
      <div data-testid="connection-status">{state.connectionStatus}</div>
      <button 
        data-testid="create-session-btn" 
        onClick={() => createSession('Test Session', 'conversation-123')}
      >
        Create Session
      </button>
      <button 
        data-testid="join-session-btn" 
        onClick={() => joinSession('session-123')}
      >
        Join Session
      </button>
    </div>
  );
};

describe('CollaborationContext', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });
  
  it('initializes with default state', () => {
    render(
      <CollaborationProvider>
        <TestComponent />
      </CollaborationProvider>
    );
    
    expect(screen.getByTestId('connection-status')).toHaveTextContent('Disconnected');
  });
  
  it('creates a session successfully', async () => {
    // Mock the invoke function to return a successful response
    (invoke as jest.Mock).mockResolvedValueOnce({
      id: 'session-123',
      name: 'Test Session',
      active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      conversation_id: 'conversation-123',
      users: {},
      metadata: {},
    });
    
    // Mock get_session_users to return empty array
    (invoke as jest.Mock).mockResolvedValueOnce([]);
    
    render(
      <CollaborationProvider>
        <TestComponent />
      </CollaborationProvider>
    );
    
    // Click the create session button
    act(() => {
      screen.getByTestId('create-session-btn').click();
    });
    
    // Wait for the invoke call to be made
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_session', {
        name: 'Test Session',
        conversationId: 'conversation-123',
      });
    });
  });
  
  it('joins a session successfully', async () => {
    // Mock the invoke function to return a successful response
    (invoke as jest.Mock).mockResolvedValueOnce({
      id: 'session-123',
      name: 'Test Session',
      active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      conversation_id: 'conversation-123',
      users: {},
      metadata: {},
    });
    
    // Mock get_session_users to return empty array
    (invoke as jest.Mock).mockResolvedValueOnce([]);
    
    render(
      <CollaborationProvider>
        <TestComponent />
      </CollaborationProvider>
    );
    
    // Click the join session button
    act(() => {
      screen.getByTestId('join-session-btn').click();
    });
    
    // Wait for the invoke call to be made
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('join_session', {
        sessionId: 'session-123',
      });
    });
  });
  
  it('handles errors when creating a session', async () => {
    // Mock the invoke function to return an error
    (invoke as jest.Mock).mockRejectedValueOnce(new Error('Failed to create session'));
    
    render(
      <CollaborationProvider>
        <TestComponent />
      </CollaborationProvider>
    );
    
    // Click the create session button
    act(() => {
      screen.getByTestId('create-session-btn').click();
    });
    
    // Wait for the invoke call to be made
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_session', {
        name: 'Test Session',
        conversationId: 'conversation-123',
      });
    });
    
    // Error should be caught and state should remain the same
    expect(screen.getByTestId('connection-status')).toHaveTextContent('Disconnected');
  });
});
