import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useFeatureFlags } from '../../contexts/FeatureFlagContext';

// Resource data interface
interface ResourceData {
  timestamp: number;
  cpu: number;
  memory: number;
  fps: number | null;
  messageCount: number;
  apiLatency: number;
  apiCalls: number;
}

interface SystemInfo {
  totalMemory: number;
  usedMemory: number;
  totalSwap: number;
  usedSwap: number;
  cpuCount: number;
  systemName: string;
  kernelVersion: string;
  osVersion: string;
  hostName: string;
}

// Dashboard tabs
enum DashboardTab {
  Overview = 'overview',
  CPU = 'cpu',
  Memory = 'memory',
  Network = 'network',
  Messages = 'messages',
}

const formatBytes = (bytes: number, decimals = 2) => {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

const ResourceDashboard: React.FC = () => {
  const { isFeatureEnabled } = useFeatureFlags();
  const [activeTab, setActiveTab] = useState<DashboardTab>(DashboardTab.Overview);
  const [resourceData, setResourceData] = useState<ResourceData[]>([]);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [uptime, setUptime] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshInterval, setRefreshInterval] = useState<number>(5000);

  // Fetch resource data
  const fetchResourceData = async () => {
    try {
      const data = await invoke<ResourceData[]>('get_resource_metrics', {
        timeRange: 300000, // 5 minutes in milliseconds
      });
      
      setResourceData(data);
      
      // Fetch system info
      const sysInfo = await invoke<SystemInfo>('get_system_info');
      setSystemInfo(sysInfo);
      
      // Fetch uptime
      const uptimeSeconds = await invoke<number>('get_uptime');
      setUptime(uptimeSeconds);
      
      setIsLoading(false);
    } catch (err: any) {
      setError(err.toString());
      setIsLoading(false);
    }
  };

  // Set up polling
  useEffect(() => {
    // Only enable if the feature is enabled
    if (!isFeatureEnabled('resource_monitoring')) {
      setError('Resource monitoring is disabled. Enable it in feature flags to use this dashboard.');
      setIsLoading(false);
      return;
    }
    
    fetchResourceData();
    
    const interval = setInterval(() => {
      fetchResourceData();
    }, refreshInterval);
    
    return () => clearInterval(interval);
  }, [refreshInterval, isFeatureEnabled]);

  // Handle tab change
  const handleTabChange = (tab: DashboardTab) => {
    setActiveTab(tab);
  };

  // Format uptime
  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    
    return `${days}d ${hours}h ${minutes}m ${remainingSeconds}s`;
  };

  // Render loading state
  if (isLoading) {
    return (
      <div className="resource-dashboard-loading">
        <div className="spinner"></div>
        <p>Loading resource metrics...</p>
      </div>
    );
  }

  // Render error state
  if (error) {
    return (
      <div className="resource-dashboard-error">
        <h3>Error loading resource metrics</h3>
        <p>{error}</p>
        <button onClick={fetchResourceData}>Retry</button>
      </div>
    );
  }

  return (
    <div className="resource-dashboard">
      <h2>System Performance</h2>
      
      <div className="dashboard-header">
        <div className="system-info">
          {systemInfo && (
            <>
              <div className="info-item">
                <strong>System:</strong> {systemInfo.systemName}
              </div>
              <div className="info-item">
                <strong>OS:</strong> {systemInfo.osVersion}
              </div>
              <div className="info-item">
                <strong>CPU Cores:</strong> {systemInfo.cpuCount}
              </div>
              <div className="info-item">
                <strong>Total Memory:</strong> {formatBytes(systemInfo.totalMemory)}
              </div>
              <div className="info-item">
                <strong>Uptime:</strong> {formatUptime(uptime)}
              </div>
            </>
          )}
        </div>
      </div>
      
      <div className="dashboard-tabs">
        <button 
          className={activeTab === DashboardTab.Overview ? 'active' : ''} 
          onClick={() => handleTabChange(DashboardTab.Overview)}
        >
          Overview
        </button>
        <button 
          className={activeTab === DashboardTab.CPU ? 'active' : ''} 
          onClick={() => handleTabChange(DashboardTab.CPU)}
        >
          CPU
        </button>
        <button 
          className={activeTab === DashboardTab.Memory ? 'active' : ''} 
          onClick={() => handleTabChange(DashboardTab.Memory)}
        >
          Memory
        </button>
        <button 
          className={activeTab === DashboardTab.Network ? 'active' : ''} 
          onClick={() => handleTabChange(DashboardTab.Network)}
        >
          Network
        </button>
        <button 
          className={activeTab === DashboardTab.Messages ? 'active' : ''} 
          onClick={() => handleTabChange(DashboardTab.Messages)}
        >
          Messages
        </button>
      </div>
      
      <div className="dashboard-content">
        {activeTab === DashboardTab.Overview && (
          <div className="overview-charts">
            <div className="chart-container">
              <h3>CPU Usage</h3>
              <ResponsiveContainer width="100%" height={200}>
                <LineChart data={resourceData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis unit="%" domain={[0, 100]} />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [`${value}%`, 'CPU']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="cpu" stroke="#8884d8" name="CPU Usage" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="chart-container">
              <h3>Memory Usage</h3>
              <ResponsiveContainer width="100%" height={200}>
                <LineChart data={resourceData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis 
                    tickFormatter={(value) => formatBytes(value, 0)} 
                  />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [formatBytes(Number(value)), 'Memory']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="memory" stroke="#82ca9d" name="Memory Usage" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="chart-container">
              <h3>API Latency</h3>
              <ResponsiveContainer width="100%" height={200}>
                <LineChart 
                  data={resourceData.filter(d => d.apiLatency > 0)}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis unit="ms" />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [`${value} ms`, 'Latency']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="apiLatency" stroke="#ffc658" name="API Latency" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="chart-container">
              <h3>FPS</h3>
              <ResponsiveContainer width="100%" height={200}>
                <LineChart 
                  data={resourceData.filter(d => d.fps !== null)}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis domain={[0, 60]} />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [value, 'FPS']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="fps" stroke="#ff8042" name="Frames Per Second" />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>
        )}
        
        {activeTab === DashboardTab.CPU && (
          <div className="cpu-details">
            <div className="chart-container full-width">
              <h3>CPU Usage Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={resourceData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis unit="%" domain={[0, 100]} />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [`${value}%`, 'CPU']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="cpu" stroke="#8884d8" name="CPU Usage" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="stats-grid">
              <div className="stat-card">
                <h4>CPU Cores</h4>
                <div className="stat-value">{systemInfo?.cpuCount || 'N/A'}</div>
              </div>
              
              <div className="stat-card">
                <h4>Current CPU</h4>
                <div className="stat-value">
                  {resourceData.length > 0 ? `${resourceData[resourceData.length - 1].cpu.toFixed(1)}%` : 'N/A'}
                </div>
              </div>
              
              <div className="stat-card">
                <h4>Avg CPU (5min)</h4>
                <div className="stat-value">
                  {resourceData.length > 0 
                    ? `${(resourceData.reduce((sum, item) => sum + item.cpu, 0) / resourceData.length).toFixed(1)}%` 
                    : 'N/A'}
                </div>
              </div>
              
              <div className="stat-card">
                <h4>Max CPU (5min)</h4>
                <div className="stat-value">
                  {resourceData.length > 0 
                    ? `${Math.max(...resourceData.map(item => item.cpu)).toFixed(1)}%` 
                    : 'N/A'}
                </div>
              </div>
            </div>
          </div>
        )}
        
        {activeTab === DashboardTab.Memory && (
          <div className="memory-details">
            <div className="chart-container full-width">
              <h3>Memory Usage Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={resourceData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis 
                    tickFormatter={(value) => formatBytes(value, 0)} 
                  />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [formatBytes(Number(value)), 'Memory']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="memory" stroke="#82ca9d" name="Memory Usage" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="stats-grid">
              <div className="stat-card">
                <h4>Total Memory</h4>
                <div className="stat-value">{systemInfo ? formatBytes(systemInfo.totalMemory) : 'N/A'}</div>
              </div>
              
              <div className="stat-card">
                <h4>Current Memory</h4>
                <div className="stat-value">
                  {resourceData.length > 0 
                    ? formatBytes(resourceData[resourceData.length - 1].memory) 
                    : 'N/A'}
                </div>
              </div>
              
              <div className="stat-card">
                <h4>Memory Usage</h4>
                <div className="stat-value">
                  {resourceData.length > 0 && systemInfo
                    ? `${((resourceData[resourceData.length - 1].memory / systemInfo.totalMemory) * 100).toFixed(1)}%` 
                    : 'N/A'}
                </div>
                <div className="progress-bar">
                  <div 
                    className="progress-fill" 
                    style={{ 
                      width: `${resourceData.length > 0 && systemInfo
                        ? ((resourceData[resourceData.length - 1].memory / systemInfo.totalMemory) * 100)
                        : 0}%` 
                    }}
                  />
                </div>
              </div>
              
              <div className="stat-card">
                <h4>Peak Memory</h4>
                <div className="stat-value">
                  {resourceData.length > 0 
                    ? formatBytes(Math.max(...resourceData.map(item => item.memory))) 
                    : 'N/A'}
                </div>
              </div>
            </div>
          </div>
        )}
        
        {activeTab === DashboardTab.Network && (
          <div className="network-details">
            <div className="chart-container full-width">
              <h3>API Latency Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart 
                  data={resourceData.filter(d => d.apiLatency > 0)}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis unit="ms" />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [`${value} ms`, 'Latency']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="apiLatency" stroke="#ffc658" name="API Latency" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="chart-container full-width">
              <h3>API Calls Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart 
                  data={resourceData.filter(d => d.apiCalls > 0)}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [value, 'API Calls']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="apiCalls" stroke="#ff8042" name="API Calls" />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>
        )}
        
        {activeTab === DashboardTab.Messages && (
          <div className="messages-details">
            <div className="chart-container full-width">
              <h3>Message Count Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart 
                  data={resourceData.filter(d => d.messageCount > 0)}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="timestamp" 
                    tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                  />
                  <YAxis />
                  <Tooltip 
                    labelFormatter={(timestamp) => new Date(Number(timestamp)).toLocaleString()}
                    formatter={(value) => [value, 'Messages']} 
                  />
                  <Legend />
                  <Line type="monotone" dataKey="messageCount" stroke="#8884d8" name="Message Count" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            
            <div className="stats-grid">
              <div className="stat-card">
                <h4>Total Messages</h4>
                <div className="stat-value">
                  {resourceData.length > 0 
                    ? resourceData.reduce((sum, item) => sum + item.messageCount, 0)
                    : 'N/A'}
                </div>
              </div>
              
              <div className="stat-card">
                <h4>Messages/Min</h4>
                <div className="stat-value">
                  {resourceData.length > 0 
                    ? (resourceData.reduce((sum, item) => sum + item.messageCount, 0) / 5).toFixed(1)
                    : 'N/A'}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
      
      <div className="dashboard-controls">
        <label>
          Refresh Interval: 
          <select 
            value={refreshInterval} 
            onChange={(e) => setRefreshInterval(Number(e.target.value))}
          >
            <option value={1000}>1 second</option>
            <option value={2000}>2 seconds</option>
            <option value={5000}>5 seconds</option>
            <option value={10000}>10 seconds</option>
            <option value={30000}>30 seconds</option>
          </select>
        </label>
        
        <button onClick={fetchResourceData}>
          Refresh Now
        </button>
      </div>
    </div>
  );
};

export default ResourceDashboard;