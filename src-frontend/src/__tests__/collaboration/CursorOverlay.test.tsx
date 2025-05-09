import React, { useRef } from 'react';
import { render, fireEvent, screen } from '@testing-library/react';
import { CursorOverlay } from '../../components/collaboration';
import { CollaborationContext, ConnectionStatus, UserRole } from '../../components/collaboration/context/CollaborationContext';

// Mock the useCollaboration hook
jest.mock('../../../hooks/useCollaboration', () => ({
  useCollaboration: jest.fn(),
}));

// Mock the collaboration context with test data
const mockContextValue = {
  state: {
    initialized: true,
    config: {
      enabled: true,
      show_presence: true,
    },
    connectionStatus: ConnectionStatus.Connected,
    users: [
      {
        id: 'user-1',
        name: 'Test User',
        role: UserRole.Editor,
        color: '#ff0000',
        online: true,
        last_active: new Date().toISOString(),
        device_id: 'device-1',
        metadata: {},
      },
    ],
    cursors: {
      'user-1': {
        user_id: 'user-1',
        device_id: 'device-1',
        x: 0.5,
        y: 0.5,
        timestamp: new Date().toISOString(),
      },
    },
    currentUser: {
      id: 'current-user',
      name: 'Current User',
      role: UserRole.Owner,
      color: '#00ff00',
      online: true,
      last_active: new Date().toISOString(),
      device_id: 'device-current',
      metadata: {},
    },
    sessions: [],
    selections: {},
    mediaDevices: [],
    statistics: {
      session_count: 0,
      total_users: 0,
      active_sessions: 0,
      cursor_updates: 0,
      selection_updates: 0,
      messages_sent: 0,
      messages_received: 0,
      sync_operations: 0,
      conflicts_resolved: 0,
      calls_initiated: 0,
      call_duration_seconds: 0,
      connection_status: ConnectionStatus.Connected,
    },
  },
  updateCursorPosition: jest.fn(),
};

// Test component that wraps CursorOverlay with a ref
const TestComponent = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  
  return (
    <div ref={containerRef} style={{ width: '500px', height: '500px' }}>
      <CursorOverlay containerRef={containerRef} />
    </div>
  );
};

describe('CursorOverlay', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });
  
  it('renders cursors for other users', () => {
    // Provide the mock context value
    render(
      <CollaborationContext.Provider value={mockContextValue as any}>
        <TestComponent />
      </CollaborationContext.Provider>
    );
    
    // Check if the cursor element is rendered
    const cursorElement = document.querySelector('.cursor-wrapper');
    expect(cursorElement).toBeInTheDocument();
  });
  
  it('does not render cursors when presence is disabled', () => {
    const disabledContext = {
      ...mockContextValue,
      state: {
        ...mockContextValue.state,
        config: {
          ...mockContextValue.state.config,
          show_presence: false,
        },
      },
    };
    
    render(
      <CollaborationContext.Provider value={disabledContext as any}>
        <TestComponent />
      </CollaborationContext.Provider>
    );
    
    // Check that no cursor elements are rendered
    const cursorElement = document.querySelector('.cursor-wrapper');
    expect(cursorElement).not.toBeInTheDocument();
  });
  
  it('calls updateCursorPosition on mouse move', () => {
    render(
      <CollaborationContext.Provider value={mockContextValue as any}>
        <TestComponent />
      </CollaborationContext.Provider>
    );
    
    // Simulate a mouse move event
    const container = document.querySelector('div[style]');
    
    // Mock getBoundingClientRect for the container
    container.getBoundingClientRect = jest.fn().mockReturnValue({
      width: 500,
      height: 500,
      left: 0,
      top: 0,
    });
    
    fireEvent.mouseMove(container, { clientX: 250, clientY: 250 });
    
    // Check if updateCursorPosition was called with normalized coordinates
    expect(mockContextValue.updateCursorPosition).toHaveBeenCalledWith(0.5, 0.5, undefined);
  });
  
  it('does not render stale cursors', () => {
    // Create a stale cursor (older than 10 seconds)
    const staleTimestamp = new Date();
    staleTimestamp.setSeconds(staleTimestamp.getSeconds() - 15);
    
    const staleContext = {
      ...mockContextValue,
      state: {
        ...mockContextValue.state,
        cursors: {
          'user-1': {
            ...mockContextValue.state.cursors['user-1'],
            timestamp: staleTimestamp.toISOString(),
          },
        },
      },
    };
    
    render(
      <CollaborationContext.Provider value={staleContext as any}>
        <TestComponent />
      </CollaborationContext.Provider>
    );
    
    // Check that no cursor elements are rendered
    const cursorElement = document.querySelector('.cursor-wrapper');
    expect(cursorElement).not.toBeInTheDocument();
  });
});
