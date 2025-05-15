// src-frontend/src/components/common/KeyboardShortcut.tsx
//
// Platform-aware keyboard shortcut component

import React from 'react';
import { usePlatform, formatShortcut } from '../../hooks/usePlatform';

interface KeyboardShortcutProps {
  shortcut: string;
  className?: string;
  compact?: boolean;
}

/**
 * Displays a keyboard shortcut with platform-appropriate key symbols
 */
export function KeyboardShortcut({
  shortcut,
  className = '',
  compact = false,
}: KeyboardShortcutProps): JSX.Element {
  const { platform } = usePlatform();
  const formattedShortcut = formatShortcut(shortcut, platform);
  
  if (compact) {
    return (
      <kbd className={`keyboard-shortcut compact ${className}`}>
        {formattedShortcut}
      </kbd>
    );
  }
  
  // Split the shortcut into individual keys
  const keys = formattedShortcut.split(' + ');
  
  return (
    <span className={`keyboard-shortcut ${className}`}>
      {keys.map((key, index) => (
        <React.Fragment key={index}>
          <kbd className="keyboard-shortcut-key">{key}</kbd>
          {index < keys.length - 1 && (
            <span className="keyboard-shortcut-separator">+</span>
          )}
        </React.Fragment>
      ))}
    </span>
  );
}

/**
 * CSS for the KeyboardShortcut component
 */
export const KeyboardShortcutStyles = `
.keyboard-shortcut {
  display: inline-flex;
  align-items: center;
  font-family: var(--monospace-font-family);
  white-space: nowrap;
}

.keyboard-shortcut.compact {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  background-color: rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(0, 0, 0, 0.1);
  font-size: 0.85em;
}

.keyboard-shortcut-key {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 1px;
  border-radius: 4px;
  background-color: rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(0, 0, 0, 0.1);
  box-shadow: 0 1px 0 rgba(0, 0, 0, 0.1);
  font-size: 0.85em;
  font-family: var(--monospace-font-family);
}

.keyboard-shortcut-separator {
  margin: 0 2px;
  opacity: 0.7;
}

/* Platform-specific styles */
.platform-macos .keyboard-shortcut-key {
  border-radius: 6px;
  box-shadow: 0 1px 0 rgba(0, 0, 0, 0.05);
}

.platform-windows .keyboard-shortcut-key {
  border-radius: 2px;
  box-shadow: 0 2px 0 rgba(0, 0, 0, 0.15);
}
`;
