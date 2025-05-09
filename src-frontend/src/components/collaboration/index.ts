// index.ts for the collaboration module
//
// This file exports all components, hooks, and types for the collaboration system.

// Context and hooks
export { 
  CollaborationContext, 
  CollaborationProvider,
  useCollaboration,
  ConnectionStatus,
  UserRole,
  type User,
  type Session,
  type CursorPosition,
  type Selection,
  type MediaDevice,
  type Participant,
  type Call,
  type CollaborationConfig,
  type CollaborationStatistics
} from './context/CollaborationContext';

// Main components
export { default as CollaborationPanel } from './CollaborationPanel';
export { default as UserList } from './UserList';

// Presence components
export { default as CursorOverlay } from './presence/CursorOverlay';
export { default as SelectionOverlay } from './presence/SelectionOverlay';
export { default as UserBadge } from './presence/UserBadge';

// Settings components
export { default as CollaborationSettings } from './settings/CollaborationSettings';

// Call components
export { default as CallControls } from './call/CallControls';

// Whiteboard components
export { default as CollaborativeWhiteboard } from './whiteboard/CollaborativeWhiteboard';
