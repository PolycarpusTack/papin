// CursorOverlay.tsx
//
// This component renders remote user cursors on top of the application.
// It displays a cursor icon and name badge for each connected user.

import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useCollaboration } from '../../../hooks/useCollaboration';
import { CursorPosition, User } from '../context/CollaborationContext';
import UserBadge from './UserBadge';
import { throttle } from '../../../utils/throttle';

interface CursorOverlayProps {
  containerRef: React.RefObject<HTMLElement>;
  zIndex?: number;
  // Set to customize the throttle time for cursor updates (in ms)
  throttleTime?: number;
}

const CursorOverlay: React.FC<CursorOverlayProps> = ({ 
  containerRef, 
  zIndex = 1000,
  throttleTime = 50 // 50ms default throttle time
}) => {
  const { state, updateCursorPosition } = useCollaboration();
  const { cursors, users, config } = state;
  const overlayRef = useRef<HTMLDivElement>(null);
  const [userMap, setUserMap] = useState<Record<string, User>>({});
  
  // Create a map of users by ID for easy lookup
  useEffect(() => {
    const map: Record<string, User> = {};
    users.forEach(user => {
      map[user.id] = user;
    });
    setUserMap(map);
  }, [users]);

  // Create a throttled version of the updateCursorPosition function
  const throttledUpdateCursor = useCallback(
    throttle((x: number, y: number, elementId?: string) => {
      updateCursorPosition(x, y, elementId);
    }, throttleTime),
    [updateCursorPosition, throttleTime]
  );

  // Track mouse movement in the container to update the user's cursor position
  useEffect(() => {
    if (!containerRef.current || !config.show_presence) return;

    const container = containerRef.current;
    
    const handleMouseMove = (e: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      // Normalize position as 0-1 relative to container
      const x = (e.clientX - rect.left) / rect.width;
      const y = (e.clientY - rect.top) / rect.height;
      
      // Get element under cursor if available
      const element = document.elementFromPoint(e.clientX, e.clientY);
      const elementId = element?.id || undefined;
      
      // Send cursor position update (throttled)
      throttledUpdateCursor(x, y, elementId);
    };

    container.addEventListener('mousemove', handleMouseMove);
    
    return () => {
      container.removeEventListener('mousemove', handleMouseMove);
    };
  }, [containerRef, config.show_presence, throttledUpdateCursor]);

  // Don't render if presence is disabled
  if (!config.show_presence) {
    return null;
  }

  // Calculate absolute position from normalized coordinates
  const getAbsolutePosition = (cursor: CursorPosition) => {
    if (!containerRef.current) {
      return { x: 0, y: 0 };
    }

    const rect = containerRef.current.getBoundingClientRect();
    return {
      x: rect.left + cursor.x * rect.width,
      y: rect.top + cursor.y * rect.height,
    };
  };

  return (
    <div 
      ref={overlayRef}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        pointerEvents: 'none',
        zIndex,
      }}
      className="cursor-overlay"
    >
      {Object.entries(cursors).map(([userId, cursor]) => {
        const user = userMap[userId];
        if (!user) return null;

        const { x, y } = getAbsolutePosition(cursor);
        
        // Cursor was updated in the last 10 seconds (not stale)
        const isFresh = new Date(cursor.timestamp).getTime() > (Date.now() - 10000);
        
        // Don't render stale cursors
        if (!isFresh) return null;

        return (
          <div
            key={`cursor-${userId}`}
            style={{
              position: 'absolute',
              left: `${x}px`,
              top: `${y}px`,
              transform: 'translate(-50%, -50%)',
              pointerEvents: 'none',
              transition: 'left 0.1s ease, top 0.1s ease',
            }}
            className="cursor-wrapper"
            data-user-id={userId}
          >
            {/* Cursor icon */}
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke={user.color}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="cursor-icon"
            >
              <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z" />
            </svg>
            
            {/* User badge */}
            <UserBadge user={user} />
          </div>
        );
      })}
    </div>
  );
};

export default CursorOverlay;
