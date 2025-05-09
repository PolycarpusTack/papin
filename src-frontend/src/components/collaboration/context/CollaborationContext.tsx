// CollaborationContext.tsx
//
// This file provides a React context for collaboration features including:
// - Session management
// - User presence
// - Real-time synchronization
// - Audio/video calls

import React, { createContext, useReducer, useContext, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

// Types from the backend
export interface User {
  id: string;
  name: string;
  role: UserRole;
  avatar?: string;
  color: string;
  online: boolean;
  last_active: string;
  device_id: string;
  metadata: Record<string, string>;
}

export enum UserRole {
  Owner = 'Owner',
  CoOwner = 'CoOwner',
  Editor = 'Editor',
  Commentator = 'Commentator',
  Viewer = 'Viewer',
}

export interface Session {
  id: string;
  name: string;
  active: boolean;
  created_at: string;
  updated_at: string;
  conversation_id: string;
  users: Record<string, User>;
  metadata: Record<string, string>;
}

export enum ConnectionStatus {
  Disconnected = 'Disconnected',
  Connecting = 'Connecting',
  Connected = 'Connected',
  Limited = 'Limited',
  Error = 'Error',
}

export interface CursorPosition {
  user_id: string;
  device_id: string;
  x: number;
  y: number;
  element_id?: string;
  timestamp: string;
}

export interface Selection {
  user_id: string;
  device_id: string;
  start_id: string;
  end_id: string;
  start_offset: number;
  end_offset: number;
  timestamp: string;
}

export interface MediaDevice {
  id: string;
  name: string;
  kind: string;
}

export interface Participant {
  user_id: string;
  name: string;
  device_id: string;
  audio_enabled: boolean;
  video_enabled: boolean;
  is_speaking: boolean;
  audio_level: number;
  network_quality: number;
  joined_at: string;
}

export interface Call {
  id: string;
  session_id: string;
  has_audio: boolean;
  has_video: boolean;
  start_time: string;
  participants: Record<string, Participant>;
}

export interface CollaborationConfig {
  enabled: boolean;
  max_users_per_session: number;
  auto_discover: boolean;
  show_presence: boolean;
  enable_av: boolean;
  sync_interval_ms: number;
  p2p_enabled: boolean;
  server_urls: string[];
  username?: string;
  user_avatar?: string;
}

export interface CollaborationStatistics {
  session_count: number;
  total_users: number;
  active_sessions: number;
  cursor_updates: number;
  selection_updates: number;
  messages_sent: number;
  messages_received: number;
  sync_operations: number;
  conflicts_resolved: number;
  calls_initiated: number;
  call_duration_seconds: number;
  current_session_id?: string;
  connection_status: ConnectionStatus;
}

// State and actions
interface CollaborationState {
  initialized: boolean;
  config: CollaborationConfig;
  connectionStatus: ConnectionStatus;
  currentUser?: User;
  currentSession?: Session;
  sessions: Session[];
  users: User[];
  cursors: Record<string, CursorPosition>;
  selections: Record<string, Selection>;
  mediaDevices: MediaDevice[];
  activeCall?: Call;
  statistics: CollaborationStatistics;
  error?: string;
}

type ActionType =
  | { type: 'INITIALIZE_SUCCESS'; payload: { config: CollaborationConfig } }
  | { type: 'UPDATE_CONFIG'; payload: { config: CollaborationConfig } }
  | { type: 'UPDATE_CONNECTION_STATUS'; payload: { status: ConnectionStatus } }
  | { type: 'SET_CURRENT_USER'; payload: { user: User } }
  | { type: 'CREATE_SESSION_SUCCESS'; payload: { session: Session } }
  | { type: 'JOIN_SESSION_SUCCESS'; payload: { session: Session } }
  | { type: 'LEAVE_SESSION_SUCCESS' }
  | { type: 'UPDATE_USERS'; payload: { users: User[] } }
  | { type: 'UPDATE_CURSORS'; payload: { cursors: Record<string, CursorPosition> } }
  | { type: 'UPDATE_SELECTIONS'; payload: { selections: Record<string, Selection> } }
  | { type: 'UPDATE_CALL'; payload: { call?: Call } }
  | { type: 'UPDATE_MEDIA_DEVICES'; payload: { devices: MediaDevice[] } }
  | { type: 'UPDATE_STATISTICS'; payload: { statistics: CollaborationStatistics } }
  | { type: 'SET_ERROR'; payload: { error: string } }
  | { type: 'CLEAR_ERROR' };

// Default state
const initialState: CollaborationState = {
  initialized: false,
  config: {
    enabled: true,
    max_users_per_session: 10,
    auto_discover: true,
    show_presence: true,
    enable_av: false,
    sync_interval_ms: 1000,
    p2p_enabled: true,
    server_urls: [
      'https://signaling.mcp-client.com',
      'stun:stun.mcp-client.com:19302',
      'turn:turn.mcp-client.com:3478',
    ],
  },
  connectionStatus: ConnectionStatus.Disconnected,
  sessions: [],
  users: [],
  cursors: {},
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
    connection_status: ConnectionStatus.Disconnected,
  },
};

// Reducer function
function collaborationReducer(state: CollaborationState, action: ActionType): CollaborationState {
  switch (action.type) {
    case 'INITIALIZE_SUCCESS':
      return {
        ...state,
        initialized: true,
        config: action.payload.config,
      };
    case 'UPDATE_CONFIG':
      return {
        ...state,
        config: action.payload.config,
      };
    case 'UPDATE_CONNECTION_STATUS':
      return {
        ...state,
        connectionStatus: action.payload.status,
      };
    case 'SET_CURRENT_USER':
      return {
        ...state,
        currentUser: action.payload.user,
      };
    case 'CREATE_SESSION_SUCCESS':
      return {
        ...state,
        currentSession: action.payload.session,
        sessions: [...state.sessions, action.payload.session],
      };
    case 'JOIN_SESSION_SUCCESS':
      return {
        ...state,
        currentSession: action.payload.session,
        sessions: [...state.sessions.filter(s => s.id !== action.payload.session.id), action.payload.session],
      };
    case 'LEAVE_SESSION_SUCCESS':
      return {
        ...state,
        currentSession: undefined,
        users: [],
        cursors: {},
        selections: {},
        activeCall: undefined,
      };
    case 'UPDATE_USERS':
      return {
        ...state,
        users: action.payload.users,
      };
    case 'UPDATE_CURSORS':
      return {
        ...state,
        cursors: action.payload.cursors,
      };
    case 'UPDATE_SELECTIONS':
      return {
        ...state,
        selections: action.payload.selections,
      };
    case 'UPDATE_CALL':
      return {
        ...state,
        activeCall: action.payload.call,
      };
    case 'UPDATE_MEDIA_DEVICES':
      return {
        ...state,
        mediaDevices: action.payload.devices,
      };
    case 'UPDATE_STATISTICS':
      return {
        ...state,
        statistics: action.payload.statistics,
      };
    case 'SET_ERROR':
      return {
        ...state,
        error: action.payload.error,
      };
    case 'CLEAR_ERROR':
      return {
        ...state,
        error: undefined,
      };
    default:
      return state;
  }
}

// Context interface
interface CollaborationContextType {
  state: CollaborationState;
  initializeCollaboration: (config?: CollaborationConfig) => Promise<void>;
  updateConfig: (config: CollaborationConfig) => Promise<void>;
  createSession: (name: string, conversationId: string) => Promise<Session>;
  joinSession: (sessionId: string) => Promise<Session>;
  leaveSession: () => Promise<void>;
  inviteUser: (email: string, role: UserRole) => Promise<void>;
  removeUser: (userId: string) => Promise<void>;
  changeUserRole: (userId: string, role: UserRole) => Promise<void>;
  updateCursorPosition: (x: number, y: number, elementId?: string) => Promise<void>;
  updateSelection: (startId: string, endId: string, startOffset: number, endOffset: number) => Promise<void>;
  startAudioCall: () => Promise<void>;
  startVideoCall: () => Promise<void>;
  endCall: () => Promise<void>;
  toggleMute: () => Promise<boolean>;
  toggleVideo: () => Promise<boolean>;
  updateUsername: (name: string) => Promise<void>;
  updateAvatar: (avatar?: string) => Promise<void>;
  refreshUsers: () => Promise<void>;
  refreshCursors: () => Promise<void>;
  refreshSelections: () => Promise<void>;
  refreshCall: () => Promise<void>;
  refreshMediaDevices: () => Promise<void>;
  refreshStatistics: () => Promise<void>;
  refreshAll: () => Promise<void>;
}

// Create the context
export const CollaborationContext = createContext<CollaborationContextType | undefined>(undefined);

// Provider component
export const CollaborationProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(collaborationReducer, initialState);
  const [refreshInterval, setRefreshInterval] = useState<number | null>(null);

  // Initialize collaboration system
  const initializeCollaboration = async (config?: CollaborationConfig) => {
    try {
      await invoke('init_collaboration_system', { config });
      const loadedConfig = await invoke<CollaborationConfig>('get_collaboration_config');
      dispatch({ type: 'INITIALIZE_SUCCESS', payload: { config: loadedConfig } });
      
      // Start refresh interval if enabled
      if (loadedConfig.enabled) {
        setRefreshInterval(window.setInterval(() => {
          refreshAll();
        }, 2000));
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to initialize collaboration: ${error}` } });
    }
  };

  // Clean up interval on unmount
  useEffect(() => {
    return () => {
      if (refreshInterval) {
        clearInterval(refreshInterval);
      }
    };
  }, [refreshInterval]);

  // Update configuration
  const updateConfig = async (config: CollaborationConfig) => {
    try {
      await invoke('update_collaboration_config', { config });
      dispatch({ type: 'UPDATE_CONFIG', payload: { config } });
      
      // Update refresh interval if needed
      if (config.enabled && !refreshInterval) {
        setRefreshInterval(window.setInterval(() => {
          refreshAll();
        }, 2000));
      } else if (!config.enabled && refreshInterval) {
        clearInterval(refreshInterval);
        setRefreshInterval(null);
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to update config: ${error}` } });
    }
  };

  // Create a new session
  const createSession = async (name: string, conversationId: string) => {
    try {
      const session = await invoke<Session>('create_session', { name, conversationId });
      dispatch({ type: 'CREATE_SESSION_SUCCESS', payload: { session } });
      await refreshUsers();
      return session;
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to create session: ${error}` } });
      throw error;
    }
  };

  // Join an existing session
  const joinSession = async (sessionId: string) => {
    try {
      const session = await invoke<Session>('join_session', { sessionId });
      dispatch({ type: 'JOIN_SESSION_SUCCESS', payload: { session } });
      await refreshUsers();
      return session;
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to join session: ${error}` } });
      throw error;
    }
  };

  // Leave the current session
  const leaveSession = async () => {
    try {
      await invoke('leave_session');
      dispatch({ type: 'LEAVE_SESSION_SUCCESS' });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to leave session: ${error}` } });
      throw error;
    }
  };

  // Invite a user to the current session
  const inviteUser = async (email: string, role: UserRole) => {
    try {
      await invoke('invite_user', { email, role });
      await refreshUsers();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to invite user: ${error}` } });
      throw error;
    }
  };

  // Remove a user from the current session
  const removeUser = async (userId: string) => {
    try {
      await invoke('remove_user', { userId });
      await refreshUsers();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to remove user: ${error}` } });
      throw error;
    }
  };

  // Change a user's role
  const changeUserRole = async (userId: string, role: UserRole) => {
    try {
      await invoke('change_user_role', { userId, role });
      await refreshUsers();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to change user role: ${error}` } });
      throw error;
    }
  };

  // Update cursor position
  const updateCursorPosition = async (x: number, y: number, elementId?: string) => {
    try {
      await invoke('update_cursor_position', { x, y, elementId });
    } catch (error) {
      console.error(`Failed to update cursor position: ${error}`);
      // Don't show an error dialog for cursor updates
    }
  };

  // Update text selection
  const updateSelection = async (
    startId: string,
    endId: string,
    startOffset: number,
    endOffset: number
  ) => {
    try {
      await invoke('update_selection', { startId, endId, startOffset, endOffset });
    } catch (error) {
      console.error(`Failed to update selection: ${error}`);
      // Don't show an error dialog for selection updates
    }
  };

  // Start an audio call
  const startAudioCall = async () => {
    try {
      await invoke('start_audio_call');
      await refreshCall();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to start audio call: ${error}` } });
      throw error;
    }
  };

  // Start a video call
  const startVideoCall = async () => {
    try {
      await invoke('start_video_call');
      await refreshCall();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to start video call: ${error}` } });
      throw error;
    }
  };

  // End the current call
  const endCall = async () => {
    try {
      await invoke('end_call');
      dispatch({ type: 'UPDATE_CALL', payload: { call: undefined } });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to end call: ${error}` } });
      throw error;
    }
  };

  // Toggle mute status
  const toggleMute = async () => {
    try {
      const audioEnabled = await invoke<boolean>('toggle_mute');
      await refreshCall();
      return audioEnabled;
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to toggle mute: ${error}` } });
      throw error;
    }
  };

  // Toggle video status
  const toggleVideo = async () => {
    try {
      const videoEnabled = await invoke<boolean>('toggle_video');
      await refreshCall();
      return videoEnabled;
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to toggle video: ${error}` } });
      throw error;
    }
  };

  // Update username
  const updateUsername = async (name: string) => {
    try {
      await invoke('update_username', { name });
      await refreshUsers();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to update username: ${error}` } });
      throw error;
    }
  };

  // Update avatar
  const updateAvatar = async (avatar?: string) => {
    try {
      await invoke('update_avatar', { avatar });
      await refreshUsers();
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: { error: `Failed to update avatar: ${error}` } });
      throw error;
    }
  };

  // Refresh users
  const refreshUsers = async () => {
    try {
      const users = await invoke<User[]>('get_session_users');
      dispatch({ type: 'UPDATE_USERS', payload: { users } });
    } catch (error) {
      console.error(`Failed to refresh users: ${error}`);
    }
  };

  // Refresh cursors
  const refreshCursors = async () => {
    try {
      const cursors = await invoke<Record<string, CursorPosition>>('get_cursors');
      dispatch({ type: 'UPDATE_CURSORS', payload: { cursors } });
    } catch (error) {
      console.error(`Failed to refresh cursors: ${error}`);
    }
  };

  // Refresh selections
  const refreshSelections = async () => {
    try {
      const selections = await invoke<Record<string, Selection>>('get_selections');
      dispatch({ type: 'UPDATE_SELECTIONS', payload: { selections } });
    } catch (error) {
      console.error(`Failed to refresh selections: ${error}`);
    }
  };

  // Refresh call status
  const refreshCall = async () => {
    try {
      const call = await invoke<Call | null>('get_active_call');
      dispatch({ type: 'UPDATE_CALL', payload: { call: call || undefined } });
    } catch (error) {
      console.error(`Failed to refresh call: ${error}`);
    }
  };

  // Refresh media devices
  const refreshMediaDevices = async () => {
    try {
      const devices = await invoke<MediaDevice[]>('get_media_devices');
      dispatch({ type: 'UPDATE_MEDIA_DEVICES', payload: { devices } });
    } catch (error) {
      console.error(`Failed to refresh media devices: ${error}`);
    }
  };

  // Refresh statistics
  const refreshStatistics = async () => {
    try {
      const statistics = await invoke<CollaborationStatistics>('get_collaboration_statistics');
      dispatch({ type: 'UPDATE_STATISTICS', payload: { statistics } });
      
      // Also update connection status
      const status = await invoke<ConnectionStatus>('get_connection_status');
      dispatch({ type: 'UPDATE_CONNECTION_STATUS', payload: { status } });
    } catch (error) {
      console.error(`Failed to refresh statistics: ${error}`);
    }
  };

  // Refresh all data
  const refreshAll = async () => {
    if (!state.initialized || !state.config.enabled) return;
    
    await Promise.all([
      refreshStatistics(),
      state.currentSession ? refreshUsers() : Promise.resolve(),
      state.config.show_presence ? refreshCursors() : Promise.resolve(),
      state.config.show_presence ? refreshSelections() : Promise.resolve(),
      state.config.enable_av ? refreshCall() : Promise.resolve(),
      state.config.enable_av ? refreshMediaDevices() : Promise.resolve(),
    ]);
  };

  const contextValue: CollaborationContextType = {
    state,
    initializeCollaboration,
    updateConfig,
    createSession,
    joinSession,
    leaveSession,
    inviteUser,
    removeUser,
    changeUserRole,
    updateCursorPosition,
    updateSelection,
    startAudioCall,
    startVideoCall,
    endCall,
    toggleMute,
    toggleVideo,
    updateUsername,
    updateAvatar,
    refreshUsers,
    refreshCursors,
    refreshSelections,
    refreshCall,
    refreshMediaDevices,
    refreshStatistics,
    refreshAll,
  };

  return (
    <CollaborationContext.Provider value={contextValue}>
      {children}
    </CollaborationContext.Provider>
  );
};

// Custom hook for using the collaboration context
export const useCollaboration = () => {
  const context = useContext(CollaborationContext);
  if (context === undefined) {
    throw new Error('useCollaboration must be used within a CollaborationProvider');
  }
  return context;
};
