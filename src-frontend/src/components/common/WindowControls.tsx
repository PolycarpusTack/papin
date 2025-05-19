// src-frontend/src/components/common/WindowControls.tsx
//
// Platform-specific window controls

import React from 'react';
import { usePlatform } from '../../hooks/usePlatform';
import { appWindow } from '@tauri-apps/api/window';

interface WindowControlsProps {
  className?: string;
  showMaximize?: boolean;
  showMinimize?: boolean;
  showClose?: boolean;
  position?: 'left' | 'right';
}

/**
 * Platform-specific window controls (close, minimize, maximize)
 */
export function WindowControls({
  className = '',
  showMaximize = true,
  showMinimize = true,
  showClose = true,
  position,
}: WindowControlsProps): JSX.Element {
  const { platform } = usePlatform();
  
  // Determine position based on platform if not explicitly specified
  const controlPosition = position || (platform === 'macos' ? 'left' : 'right');
  
  const handleClose = async () => {
    await appWindow.close();
  };
  
  const handleMinimize = async () => {
    await appWindow.minimize();
  };
  
  const handleMaximize = async () => {
    if (await appWindow.isMaximized()) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  };
  
  // Button components based on platform
  const renderButtons = () => {
    if (platform === 'macos') {
      return (
        <>
          {showClose && (
            <button
              className="window-control window-control-close macos"
              onClick={handleClose}
              aria-label="Close"
            />
          )}
          {showMinimize && (
            <button
              className="window-control window-control-minimize macos"
              onClick={handleMinimize}
              aria-label="Minimize"
            />
          )}
          {showMaximize && (
            <button
              className="window-control window-control-maximize macos"
              onClick={handleMaximize}
              aria-label="Maximize"
            />
          )}
        </>
      );
    }
    
    // Windows and Linux (similar layout)
    return (
      <>
        {showMinimize && (
          <button
            className={`window-control window-control-minimize ${platform}`}
            onClick={handleMinimize}
            aria-label="Minimize"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
            </svg>
          </button>
        )}
        {showMaximize && (
          <button
            className={`window-control window-control-maximize ${platform}`}
            onClick={handleMaximize}
            aria-label="Maximize"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="2.5" y="2.5" width="7" height="7" stroke="currentColor" fill="none" strokeWidth="1" />
            </svg>
          </button>
        )}
        {showClose && (
          <button
            className={`window-control window-control-close ${platform}`}
            onClick={handleClose}
            aria-label="Close"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path
                d="M3,3 L9,9 M9,3 L3,9"
                stroke="currentColor"
                strokeWidth="1"
                fill="none"
              />
            </svg>
          </button>
        )}
      </>
    );
  };
  
  return (
    <div className={`window-controls ${controlPosition} ${className}`}>
      {renderButtons()}
    </div>
  );
}

/**
 * CSS for the WindowControls component
 */
export const WindowControlsStyles = `
.window-controls {
  display: flex;
  align-items: center;
  -webkit-app-region: no-drag;
  user-select: none;
}

.window-controls.left {
  margin-left: 12px;
}

.window-controls.right {
  margin-right: 12px;
}

.window-control {
  width: 12px;
  height: 12px;
  margin: 0 6px;
  border-radius: 50%;
  border: none;
  outline: none;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

/* macOS style */
.window-control.macos {
  transition: opacity 0.15s ease;
}

.window-control-close.macos {
  background-color: #ff5f57;
}

.window-control-minimize.macos {
  background-color: #ffbd2e;
}

.window-control-maximize.macos {
  background-color: #28c940;
}

/* Windows style */
.window-control.windows {
  width: 45px;
  height: 32px;
  border-radius: 0;
  margin: 0;
  color: rgba(0, 0, 0, 0.7);
  background-color: transparent;
  transition: background-color 0.15s ease;
}

.window-control.windows:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

.window-control-close.windows {
  color: #000;
}

.window-control-close.windows:hover {
  background-color: #e81123;
  color: #fff;
}

/* Linux style */
.window-control.linux {
  width: 45px;
  height: 32px;
  border-radius: 0;
  margin: 0;
  color: rgba(0, 0, 0, 0.7);
  background-color: transparent;
  transition: background-color 0.15s ease;
}

.window-control.linux:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

.window-control-close.linux {
  color: #000;
}

.window-control-close.linux:hover {
  background-color: #e81123;
  color: #fff;
}

/* Dark theme adjustments */
:root[data-theme="dark"] .window-control.windows,
:root[data-theme="dark"] .window-control.linux {
  color: rgba(255, 255, 255, 0.7);
}

:root[data-theme="dark"] .window-control.windows:hover,
:root[data-theme="dark"] .window-control.linux:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

:root[data-theme="dark"] .window-control-close.windows,
:root[data-theme="dark"] .window-control-close.linux {
  color: #fff;
}
`;
