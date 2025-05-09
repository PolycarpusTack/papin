// UserBadge.tsx
//
// This component renders a badge with user information.
// It's used to display user names next to their cursors and selections.

import React, { useState } from 'react';
import { User, UserRole } from '../context/CollaborationContext';

interface UserBadgeProps {
  user: User;
  showRole?: boolean;
  alwaysShow?: boolean;
}

// Helper to get role display name
const getRoleDisplayName = (role: UserRole): string => {
  switch (role) {
    case UserRole.Owner:
      return 'Owner';
    case UserRole.CoOwner:
      return 'Co-owner';
    case UserRole.Editor:
      return 'Editor';
    case UserRole.Commentator:
      return 'Commentator';
    case UserRole.Viewer:
      return 'Viewer';
    default:
      return 'User';
  }
};

// Helper to get role badge color
const getRoleBadgeColor = (role: UserRole): string => {
  switch (role) {
    case UserRole.Owner:
      return '#FF5722';
    case UserRole.CoOwner:
      return '#FF9800';
    case UserRole.Editor:
      return '#2196F3';
    case UserRole.Commentator:
      return '#4CAF50';
    case UserRole.Viewer:
      return '#9E9E9E';
    default:
      return '#9E9E9E';
  }
};

const UserBadge: React.FC<UserBadgeProps> = ({ 
  user, 
  showRole = true,
  alwaysShow = false
}) => {
  const [isHovered, setIsHovered] = useState(false);
  
  // Only show badge on hover unless alwaysShow is true
  const isVisible = alwaysShow || isHovered;
  
  // Create user initials for avatar
  const getInitials = (name: string): string => {
    const parts = name.split(' ');
    if (parts.length === 1) {
      return name.substring(0, 2).toUpperCase();
    }
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  };
  
  return (
    <div
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      style={{
        position: 'absolute',
        left: '20px',
        top: '-10px',
        display: 'flex',
        alignItems: 'center',
        transform: isVisible ? 'translateY(-100%)' : 'translateY(-100%) scale(0.8)',
        opacity: isVisible ? 1 : 0,
        transition: 'transform 0.2s ease, opacity 0.2s ease',
        pointerEvents: 'none',
        zIndex: 5,
      }}
      className="cursor-badge"
    >
      {/* User badge with name and optional role */}
      <div
        style={{
          backgroundColor: user.color,
          color: '#fff',
          padding: '4px 8px',
          borderRadius: '4px',
          fontWeight: 500,
          fontSize: '12px',
          whiteSpace: 'nowrap',
          display: 'flex',
          alignItems: 'center',
          boxShadow: '0 2px 5px rgba(0, 0, 0, 0.2)',
          maxWidth: '200px',
        }}
      >
        {/* Avatar/Initials */}
        <div
          style={{
            width: '18px',
            height: '18px',
            borderRadius: '50%',
            backgroundColor: '#ffffff44',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            marginRight: '6px',
            fontSize: '10px',
            fontWeight: 600,
            overflow: 'hidden',
          }}
        >
          {user.avatar ? (
            <img
              src={user.avatar}
              alt={user.name}
              style={{ width: '100%', height: '100%', objectFit: 'cover' }}
            />
          ) : (
            getInitials(user.name)
          )}
        </div>

        {/* Name */}
        <span
          style={{
            overflow: 'hidden',
            textOverflow: 'ellipsis',
          }}
        >
          {user.name}
        </span>
        
        {/* Role badge (if enabled) */}
        {showRole && (
          <div
            style={{
              backgroundColor: getRoleBadgeColor(user.role),
              fontSize: '9px',
              padding: '2px 4px',
              borderRadius: '3px',
              marginLeft: '5px',
              lineHeight: 1,
            }}
          >
            {getRoleDisplayName(user.role)}
          </div>
        )}
      </div>
      
      {/* Pointer */}
      <div
        style={{
          width: 0,
          height: 0,
          borderLeft: '5px solid transparent',
          borderRight: '5px solid transparent',
          borderTop: `5px solid ${user.color}`,
          position: 'absolute',
          bottom: '-5px',
          left: '20px',
        }}
      />
    </div>
  );
};

export default UserBadge;
