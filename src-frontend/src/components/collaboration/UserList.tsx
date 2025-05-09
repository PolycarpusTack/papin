// UserList.tsx
//
// This component displays a list of users in the current collaboration session,
// allowing management of users and their roles.

import React, { useState } from 'react';
import { useCollaboration } from '../../hooks/useCollaboration';
import { User, UserRole } from './context/CollaborationContext';

interface UserListProps {
  // Optional props can be added here
}

const UserList: React.FC<UserListProps> = () => {
  const { state, inviteUser, removeUser, changeUserRole } = useCollaboration();
  const { users, currentUser } = state;
  
  const [email, setEmail] = useState<string>('');
  const [selectedRole, setSelectedRole] = useState<UserRole>(UserRole.Editor);
  const [showInviteForm, setShowInviteForm] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  
  // Handle inviting a new user
  const handleInviteUser = async () => {
    if (!email.trim()) {
      setError('Please enter an email address');
      return;
    }
    
    setIsLoading(true);
    setError(null);
    
    try {
      await inviteUser(email, selectedRole);
      setEmail('');
      setShowInviteForm(false);
    } catch (err) {
      setError(`Failed to invite user: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };
  
  // Handle removing a user
  const handleRemoveUser = async (userId: string) => {
    if (!confirm('Are you sure you want to remove this user?')) {
      return;
    }
    
    try {
      await removeUser(userId);
    } catch (err) {
      console.error('Failed to remove user:', err);
      alert(`Failed to remove user: ${err}`);
    }
  };
  
  // Handle changing a user's role
  const handleChangeRole = async (userId: string, newRole: UserRole) => {
    try {
      await changeUserRole(userId, newRole);
    } catch (err) {
      console.error('Failed to change user role:', err);
      alert(`Failed to change user role: ${err}`);
    }
  };
  
  // Check if current user has permission to manage users
  const canManageUsers = () => {
    if (!currentUser) return false;
    return (
      currentUser.role === UserRole.Owner || 
      currentUser.role === UserRole.CoOwner
    );
  };
  
  // Get time since user was last active
  const getLastActiveTime = (user: User) => {
    const lastActive = new Date(user.last_active);
    const now = new Date();
    const diffMs = now.getTime() - lastActive.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    
    if (diffSec < 60) {
      return 'Just now';
    } else if (diffSec < 3600) {
      const mins = Math.floor(diffSec / 60);
      return `${mins} ${mins === 1 ? 'minute' : 'minutes'} ago`;
    } else if (diffSec < 86400) {
      const hours = Math.floor(diffSec / 3600);
      return `${hours} ${hours === 1 ? 'hour' : 'hours'} ago`;
    } else {
      const days = Math.floor(diffSec / 86400);
      return `${days} ${days === 1 ? 'day' : 'days'} ago`;
    }
  };
  
  // Render invite form
  const renderInviteForm = () => {
    if (!showInviteForm) {
      return (
        <button
          onClick={() => setShowInviteForm(true)}
          className="collaboration-button primary"
          style={{
            marginBottom: '15px',
            width: '100%',
          }}
        >
          Invite User
        </button>
      );
    }
    
    return (
      <div style={{
        padding: '15px',
        backgroundColor: '#F5F5F5',
        borderRadius: '6px',
        marginBottom: '15px',
      }}>
        <h4 style={{ margin: '0 0 10px 0' }}>Invite a User</h4>
        
        {error && (
          <div className="collaboration-error" style={{ fontSize: '14px' }}>
            {error}
          </div>
        )}
        
        <div className="collaboration-form-group" style={{ marginBottom: '10px' }}>
          <label 
            htmlFor="email" 
            className="collaboration-label"
            style={{ fontSize: '14px' }}
          >
            Email Address
          </label>
          <input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="Enter email address"
            className="collaboration-input"
          />
        </div>
        
        <div className="collaboration-form-group" style={{ marginBottom: '15px' }}>
          <label 
            htmlFor="role" 
            className="collaboration-label"
            style={{ fontSize: '14px' }}
          >
            Role
          </label>
          <select
            id="role"
            value={selectedRole}
            onChange={(e) => setSelectedRole(e.target.value as UserRole)}
            className="collaboration-select"
          >
            <option value={UserRole.Editor}>Editor</option>
            <option value={UserRole.CoOwner}>Co-Owner</option>
            <option value={UserRole.Commentator}>Commentator</option>
            <option value={UserRole.Viewer}>Viewer</option>
          </select>
        </div>
        
        <div className="collaboration-button-group" style={{ marginTop: '0' }}>
          <button
            onClick={() => setShowInviteForm(false)}
            className="collaboration-button secondary"
            style={{ fontSize: '14px' }}
          >
            Cancel
          </button>
          <button
            onClick={handleInviteUser}
            disabled={isLoading || !email.trim()}
            className="collaboration-button primary"
            style={{
              fontSize: '14px',
              opacity: isLoading || !email.trim() ? 0.7 : 1,
            }}
          >
            {isLoading ? 'Inviting...' : 'Send Invite'}
          </button>
        </div>
      </div>
    );
  };
  
  // If no users in session
  if (users.length === 0) {
    return (
      <div style={{ padding: '15px', textAlign: 'center' }}>
        <p>No users in this session yet.</p>
        
        {canManageUsers() && renderInviteForm()}
      </div>
    );
  }
  
  return (
    <div className="user-list">
      {canManageUsers() && renderInviteForm()}
      
      {users.map(user => (
        <div
          key={user.id}
          className="user-item"
          style={{
            backgroundColor: user.id === currentUser?.id ? '#E3F2FD' : 'white',
          }}
        >
          <div className="user-item-info">
            {/* User color indicator */}
            <div style={{
              width: '12px',
              height: '12px',
              borderRadius: '50%',
              backgroundColor: user.color,
              marginRight: '10px',
            }}/>
            
            {/* User avatar/name */}
            <div className="user-details">
              <div className="user-name">
                {user.name}
                {user.id === currentUser?.id && (
                  <span style={{ fontSize: '12px', marginLeft: '5px', color: '#757575' }}>
                    (You)
                  </span>
                )}
              </div>
              <div style={{ display: 'flex', alignItems: 'center', fontSize: '12px', color: '#757575' }}>
                <div style={{
                  width: '8px',
                  height: '8px',
                  borderRadius: '50%',
                  backgroundColor: user.online ? '#4CAF50' : '#9E9E9E',
                  marginRight: '5px',
                }}/>
                {user.online ? 'Online' : `Last active ${getLastActiveTime(user)}`}
              </div>
            </div>
          </div>
          
          {/* User role and actions */}
          <div className="user-actions">
            {/* Role indicator */}
            <div style={{
              padding: '4px 8px',
              borderRadius: '4px',
              backgroundColor: '#EEEEEE',
              fontSize: '12px',
              marginRight: '10px',
            }}>
              {user.role}
            </div>
            
            {/* Actions for owners/co-owners */}
            {canManageUsers() && user.id !== currentUser?.id && (
              <div style={{ display: 'flex', gap: '5px' }}>
                {/* Role change dropdown */}
                <select
                  value={user.role}
                  onChange={(e) => handleChangeRole(user.id, e.target.value as UserRole)}
                  style={{
                    padding: '4px',
                    fontSize: '12px',
                    borderRadius: '4px',
                    border: '1px solid #BDBDBD',
                  }}
                >
                  <option value={UserRole.CoOwner}>Co-Owner</option>
                  <option value={UserRole.Editor}>Editor</option>
                  <option value={UserRole.Commentator}>Commentator</option>
                  <option value={UserRole.Viewer}>Viewer</option>
                </select>
                
                {/* Remove button */}
                <button
                  onClick={() => handleRemoveUser(user.id)}
                  className="collaboration-button danger"
                  style={{
                    padding: '4px 8px',
                    fontSize: '12px',
                  }}
                >
                  Remove
                </button>
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};

export default UserList;
