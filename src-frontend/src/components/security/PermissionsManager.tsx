import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './PermissionsManager.css';

// Types
interface Permission {
  id: string;
  name: string;
  description: string;
  level: PermissionLevel;
  category: string;
  last_modified: string;
  usage_count: number;
  required: boolean;
}

interface PermissionStatistics {
  total_permissions: number;
  count_by_level: Record<string, number>;
  count_by_category: Record<string, number>;
  total_requests: number;
  granted_count: number;
  denied_count: number;
  most_used_permissions: [string, number][];
}

enum PermissionLevel {
  AlwaysAllow = "AlwaysAllow",
  AskFirstTime = "AskFirstTime",
  AskEveryTime = "AskEveryTime",
  NeverAllow = "NeverAllow",
}

const PermissionsManager: React.FC = () => {
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [statistics, setStatistics] = useState<PermissionStatistics | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'all' | 'privacy' | 'network' | 'system' | 'data' | 'security'>('all');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [showResetConfirm, setShowResetConfirm] = useState<boolean>(false);
  
  // Load permissions on mount
  useEffect(() => {
    loadData();
  }, []);
  
  // Function to load data from backend
  const loadData = async () => {
    try {
      setLoading(true);
      
      // Load permissions
      const permissionsData = await invoke<any[]>('get_all_permissions');
      
      // Convert from JSON values to typed objects
      const typedPermissions: Permission[] = permissionsData.map(item => ({
        id: item.id,
        name: item.name,
        description: item.description,
        level: item.level as PermissionLevel,
        category: item.category,
        last_modified: item.last_modified,
        usage_count: item.usage_count,
        required: item.required,
      }));
      
      setPermissions(typedPermissions);
      
      // Load statistics
      const statsData = await invoke<PermissionStatistics>('get_permission_statistics');
      setStatistics(statsData);
      
      setError(null);
    } catch (err) {
      console.error('Error loading permissions data:', err);
      setError(`Failed to load permissions: ${err}`);
    } finally {
      setLoading(false);
    }
  };
  
  // Change permission level
  const changePermissionLevel = async (id: string, level: PermissionLevel) => {
    try {
      await invoke('set_permission_level', { id, level });
      
      // Update local state
      setPermissions(permissions.map(permission => 
        permission.id === id 
          ? { ...permission, level, last_modified: new Date().toISOString() } 
          : permission
      ));
      
      // Reload statistics
      const statsData = await invoke<PermissionStatistics>('get_permission_statistics');
      setStatistics(statsData);
    } catch (err) {
      console.error('Error changing permission level:', err);
      setError(`Failed to change permission level: ${err}`);
    }
  };
  
  // Reset permission to default
  const resetPermission = async (id: string) => {
    try {
      await invoke('reset_permission', { id });
      
      // Reload data to get updated state
      loadData();
    } catch (err) {
      console.error('Error resetting permission:', err);
      setError(`Failed to reset permission: ${err}`);
    }
  };
  
  // Reset all permissions to default
  const resetAllPermissions = async () => {
    try {
      await invoke('reset_all_permissions');
      
      // Reload data to get updated state
      loadData();
      
      // Close confirm dialog
      setShowResetConfirm(false);
    } catch (err) {
      console.error('Error resetting all permissions:', err);
      setError(`Failed to reset all permissions: ${err}`);
    }
  };
  
  // Format timestamp
  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };
  
  // Format level name for display
  const formatLevelName = (level: string): string => {
    switch (level) {
      case PermissionLevel.AlwaysAllow:
        return "Always Allow";
      case PermissionLevel.AskFirstTime:
        return "Ask First Time";
      case PermissionLevel.AskEveryTime:
        return "Ask Every Time";
      case PermissionLevel.NeverAllow:
        return "Never Allow";
      default:
        return level;
    }
  };
  
  // Filter permissions by tab and search query
  const filteredPermissions = (() => {
    let filtered = [...permissions];
    
    // Filter by category
    if (activeTab !== 'all') {
      filtered = filtered.filter(p => 
        p.category.toLowerCase() === activeTab.toLowerCase()
      );
    }
    
    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(p =>
        p.name.toLowerCase().includes(query) ||
        p.description.toLowerCase().includes(query) ||
        p.id.toLowerCase().includes(query)
      );
    }
    
    return filtered;
  })();
  
  // Render permission level selector with appropriate styling
  const renderLevelSelector = (permission: Permission) => {
    const isDisabled = permission.required && permission.level === PermissionLevel.AlwaysAllow;
    
    return (
      <select
        value={permission.level}
        onChange={(e) => changePermissionLevel(permission.id, e.target.value as PermissionLevel)}
        className={`permission-level-select level-${permission.level.toLowerCase()}`}
        disabled={isDisabled}
        title={isDisabled ? "This permission is required for core functionality" : undefined}
      >
        <option value={PermissionLevel.AlwaysAllow}>Always Allow</option>
        <option value={PermissionLevel.AskFirstTime}>Ask First Time</option>
        <option value={PermissionLevel.AskEveryTime}>Ask Every Time</option>
        <option value={PermissionLevel.NeverAllow}>Never Allow</option>
      </select>
    );
  };
  
  return (
    <div className="permissions-manager">
      <div className="permissions-header">
        <h2>Permissions Manager</h2>
        
        <div className="header-controls">
          <div className="search-box">
            <input
              type="text"
              placeholder="Search permissions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            {searchQuery && (
              <button 
                className="clear-search"
                onClick={() => setSearchQuery('')}
              >
                ×
              </button>
            )}
          </div>
          
          <button 
            className="reset-all-button"
            onClick={() => setShowResetConfirm(true)}
          >
            Reset All Permissions
          </button>
        </div>
      </div>
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      <div className="permissions-content">
        <div className="permissions-sidebar">
          <nav className="categories-nav">
            <button
              className={activeTab === 'all' ? 'active' : ''}
              onClick={() => setActiveTab('all')}
            >
              All Categories
              <span className="count">{permissions.length}</span>
            </button>
            
            <button
              className={activeTab === 'privacy' ? 'active' : ''}
              onClick={() => setActiveTab('privacy')}
            >
              Privacy
              <span className="count">
                {permissions.filter(p => p.category === 'Privacy').length}
              </span>
            </button>
            
            <button
              className={activeTab === 'network' ? 'active' : ''}
              onClick={() => setActiveTab('network')}
            >
              Network
              <span className="count">
                {permissions.filter(p => p.category === 'Network').length}
              </span>
            </button>
            
            <button
              className={activeTab === 'system' ? 'active' : ''}
              onClick={() => setActiveTab('system')}
            >
              System
              <span className="count">
                {permissions.filter(p => p.category === 'System').length}
              </span>
            </button>
            
            <button
              className={activeTab === 'data' ? 'active' : ''}
              onClick={() => setActiveTab('data')}
            >
              Data
              <span className="count">
                {permissions.filter(p => p.category === 'Data').length}
              </span>
            </button>
            
            <button
              className={activeTab === 'security' ? 'active' : ''}
              onClick={() => setActiveTab('security')}
            >
              Security
              <span className="count">
                {permissions.filter(p => p.category === 'Security').length}
              </span>
            </button>
          </nav>
          
          {statistics && (
            <div className="statistics-panel">
              <h3>Statistics</h3>
              
              <div className="stat-item">
                <div className="stat-label">Total Permissions</div>
                <div className="stat-value">{statistics.total_permissions}</div>
              </div>
              
              <div className="stat-item">
                <div className="stat-label">Permission Requests</div>
                <div className="stat-value">{statistics.total_requests}</div>
              </div>
              
              <div className="stat-item">
                <div className="stat-label">Granted Requests</div>
                <div className="stat-value granted">{statistics.granted_count}</div>
              </div>
              
              <div className="stat-item">
                <div className="stat-label">Denied Requests</div>
                <div className="stat-value denied">{statistics.denied_count}</div>
              </div>
              
              <h4>Permission Levels</h4>
              <div className="levels-chart">
                {Object.entries(statistics.count_by_level).map(([level, count]) => (
                  <div key={level} className="level-bar-container">
                    <div className="level-label">{formatLevelName(level)}</div>
                    <div className="level-bar">
                      <div 
                        className={`level-bar-fill level-${level.toLowerCase()}`}
                        style={{ width: `${(count / statistics.total_permissions) * 100}%` }}
                      ></div>
                    </div>
                    <div className="level-count">{count}</div>
                  </div>
                ))}
              </div>
              
              <h4>Most Used Permissions</h4>
              {statistics.most_used_permissions.length === 0 ? (
                <div className="no-data">No usage data yet</div>
              ) : (
                <div className="most-used-list">
                  {statistics.most_used_permissions.map(([id, count]) => {
                    const permission = permissions.find(p => p.id === id);
                    return permission ? (
                      <div key={id} className="most-used-item">
                        <div className="most-used-name">{permission.name}</div>
                        <div className="most-used-count">{count} uses</div>
                      </div>
                    ) : null;
                  })}
                </div>
              )}
            </div>
          )}
        </div>
        
        <div className="permissions-list">
          {loading ? (
            <div className="loading-message">
              <div className="loading-spinner"></div>
              <div>Loading permissions...</div>
            </div>
          ) : filteredPermissions.length === 0 ? (
            <div className="no-permissions">
              {searchQuery 
                ? `No permissions matching "${searchQuery}"` 
                : activeTab !== 'all' 
                  ? `No permissions in the ${activeTab} category` 
                  : 'No permissions found'
              }
            </div>
          ) : (
            <>
              <div className="permissions-count">
                {filteredPermissions.length} permission{filteredPermissions.length !== 1 ? 's' : ''}
                {activeTab !== 'all' && ` in ${activeTab}`}
                {searchQuery && ` matching "${searchQuery}"`}
              </div>
              
              {filteredPermissions.map(permission => (
                <div key={permission.id} className="permission-card">
                  <div className="permission-header">
                    <div>
                      <h3 className="permission-name">{permission.name}</h3>
                      <div className="permission-meta">
                        <span className="permission-id">{permission.id}</span>
                        <span className="permission-category">{permission.category}</span>
                        {permission.required && (
                          <span className="permission-required">Required</span>
                        )}
                      </div>
                    </div>
                    
                    <div className="permission-actions">
                      {renderLevelSelector(permission)}
                      <button 
                        className="reset-button" 
                        onClick={() => resetPermission(permission.id)}
                        disabled={permission.required && permission.level === PermissionLevel.AlwaysAllow}
                        title="Reset to default"
                      >
                        Reset
                      </button>
                    </div>
                  </div>
                  
                  <div className="permission-description">
                    {permission.description}
                  </div>
                  
                  <div className="permission-footer">
                    <div className="permission-info">
                      <span className="permission-usage">
                        Used {permission.usage_count} time{permission.usage_count !== 1 ? 's' : ''}
                      </span>
                      <span className="permission-last-modified">
                        Last changed: {formatTimestamp(permission.last_modified)}
                      </span>
                    </div>
                    
                    <div className={`permission-status level-${permission.level.toLowerCase()}`}>
                      {formatLevelName(permission.level)}
                    </div>
                  </div>
                </div>
              ))}
            </>
          )}
        </div>
      </div>
      
      {showResetConfirm && (
        <div className="modal-overlay">
          <div className="confirm-modal">
            <h3>Reset All Permissions?</h3>
            <p>
              This will reset all permissions to their default values. 
              This action cannot be undone.
            </p>
            <div className="modal-actions">
              <button 
                className="cancel-button"
                onClick={() => setShowResetConfirm(false)}
              >
                Cancel
              </button>
              <button 
                className="confirm-button"
                onClick={resetAllPermissions}
              >
                Reset All
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default PermissionsManager;
