import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import './KeyboardNavigation.css';

// Key action types
export type KeyAction = {
  key: string;
  altKey?: boolean;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  metaKey?: boolean;
  description: string;
  handler: () => void;
  disabled?: boolean;
  scope?: string;
};

// Interface for the keyboard context
interface KeyboardContextType {
  registerAction: (action: KeyAction) => () => void;
  unregisterAction: (key: string) => void;
  activeScope: string;
  setActiveScope: (scope: string) => void;
  showShortcutsDialog: () => void;
  hideShortcutsDialog: () => void;
  isShortcutsDialogOpen: boolean;
  getFormattedShortcut: (action: KeyAction) => string;
  actions: KeyAction[];
}

const KeyboardContext = createContext<KeyboardContextType>({
  registerAction: () => () => {},
  unregisterAction: () => {},
  activeScope: 'global',
  setActiveScope: () => {},
  showShortcutsDialog: () => {},
  hideShortcutsDialog: () => {},
  isShortcutsDialogOpen: false,
  getFormattedShortcut: () => '',
  actions: [],
});

export const useKeyboard = () => useContext(KeyboardContext);

// Helper function to format keyboard shortcuts for display
const formatShortcut = (action: KeyAction): string => {
  const parts: string[] = [];
  
  if (action.ctrlKey) parts.push('Ctrl');
  if (action.altKey) parts.push('Alt');
  if (action.shiftKey) parts.push('Shift');
  if (action.metaKey) parts.push('⌘');
  
  parts.push(action.key.toUpperCase());
  
  return parts.join(' + ');
};

// Match a keyboard event against a registered action
const isMatchingKeyEvent = (event: KeyboardEvent, action: KeyAction): boolean => {
  return (
    event.key.toLowerCase() === action.key.toLowerCase() &&
    Boolean(event.altKey) === Boolean(action.altKey) &&
    Boolean(event.ctrlKey) === Boolean(action.ctrlKey) &&
    Boolean(event.shiftKey) === Boolean(action.shiftKey) &&
    Boolean(event.metaKey) === Boolean(action.metaKey)
  );
};

interface KeyboardProviderProps {
  children: ReactNode;
}

export const KeyboardProvider: React.FC<KeyboardProviderProps> = ({ children }) => {
  const [actions, setActions] = useState<KeyAction[]>([]);
  const [activeScope, setActiveScope] = useState<string>('global');
  const [isShortcutsDialogOpen, setIsShortcutsDialogOpen] = useState<boolean>(false);
  
  // Register a keyboard action
  const registerAction = (action: KeyAction) => {
    const scopedAction = {
      ...action,
      scope: action.scope || 'global',
    };
    
    setActions(prevActions => {
      // Filter out any existing actions with the same key combination and scope
      const filteredActions = prevActions.filter(
        act => !(
          act.key === scopedAction.key &&
          act.altKey === scopedAction.altKey &&
          act.ctrlKey === scopedAction.ctrlKey &&
          act.shiftKey === scopedAction.shiftKey &&
          act.metaKey === scopedAction.metaKey &&
          act.scope === scopedAction.scope
        )
      );
      
      return [...filteredActions, scopedAction];
    });
    
    // Return a function to unregister this action
    return () => {
      setActions(prevActions => 
        prevActions.filter(a => a !== scopedAction)
      );
    };
  };
  
  // Unregister a keyboard action by key
  const unregisterAction = (key: string) => {
    setActions(prevActions => 
      prevActions.filter(action => action.key !== key)
    );
  };
  
  // Handle keyboard events
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Skip if we're in an input element
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        event.target instanceof HTMLSelectElement ||
        (event.target as HTMLElement).isContentEditable
      ) {
        return;
      }
      
      // Global action to show keyboard shortcuts dialog
      if (event.key === '?' && event.shiftKey) {
        event.preventDefault();
        setIsShortcutsDialogOpen(true);
        return;
      }
      
      // Find matching action in current scope or global scope
      const matchingAction = actions.find(
        action => 
          !action.disabled && 
          isMatchingKeyEvent(event, action) && 
          (action.scope === activeScope || action.scope === 'global')
      );
      
      if (matchingAction) {
        event.preventDefault();
        matchingAction.handler();
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [actions, activeScope]);
  
  // Show and hide keyboard shortcuts dialog
  const showShortcutsDialog = () => setIsShortcutsDialogOpen(true);
  const hideShortcutsDialog = () => setIsShortcutsDialogOpen(false);
  
  return (
    <KeyboardContext.Provider
      value={{
        registerAction,
        unregisterAction,
        activeScope,
        setActiveScope,
        showShortcutsDialog,
        hideShortcutsDialog,
        isShortcutsDialogOpen,
        getFormattedShortcut: formatShortcut,
        actions,
      }}
    >
      {children}
      {isShortcutsDialogOpen && (
        <KeyboardShortcutsDialog onClose={hideShortcutsDialog} />
      )}
    </KeyboardContext.Provider>
  );
};

// Component to display keyboard shortcuts
const KeyboardShortcutsDialog: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const { actions, activeScope, getFormattedShortcut } = useKeyboard();
  
  // Group actions by scope
  const groupedActions = actions.reduce<Record<string, KeyAction[]>>(
    (groups, action) => {
      const scope = action.scope || 'global';
      if (!groups[scope]) {
        groups[scope] = [];
      }
      groups[scope].push(action);
      return groups;
    }, 
    {}
  );
  
  // Format scope name for display
  const formatScopeName = (scope: string): string => {
    return scope
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, str => str.toUpperCase());
  };
  
  return (
    <div className="keyboard-shortcuts-overlay" onClick={onClose}>
      <div className="keyboard-shortcuts-dialog" onClick={e => e.stopPropagation()}>
        <div className="keyboard-shortcuts-header">
          <h2>Keyboard Shortcuts</h2>
          <button className="keyboard-shortcuts-close" onClick={onClose}>
            &times;
          </button>
        </div>
        
        <div className="keyboard-shortcuts-content">
          {Object.entries(groupedActions).map(([scope, scopeActions]) => (
            <div key={scope} className="keyboard-shortcuts-section">
              <h3 className="keyboard-shortcuts-scope">
                {formatScopeName(scope)}
                {scope === activeScope && scope !== 'global' && (
                  <span className="keyboard-shortcuts-active-tag">Active</span>
                )}
              </h3>
              
              <ul className="keyboard-shortcuts-list">
                {scopeActions.map((action, index) => (
                  <li key={index} className="keyboard-shortcuts-item">
                    <span className="keyboard-shortcuts-description">
                      {action.description}
                    </span>
                    <span className="keyboard-shortcuts-keys">
                      <kbd>{getFormattedShortcut(action)}</kbd>
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
        
        <div className="keyboard-shortcuts-footer">
          <p>Press <kbd>?</kbd> to toggle this dialog at any time</p>
        </div>
      </div>
    </div>
  );
};

export default KeyboardProvider;
