import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar } from 'recharts';
import './ResourceDashboard.css';

interface MetricTimeSeries {
  timestamp: string;
  value: number;
}

interface HistogramStats {
  min: number;
  max: number;
  avg: number;
  count: number;
  p50: number;
  p90: number;
  p99: number;
}

interface TimerStats {
  min_ms: number;
  max_ms: number;
  avg_ms: number;
  count: number;
  p50_ms: number;
  p90_ms: number;
  p99_ms: number;
}

interface SystemResources {
  cpu_usage: number;
  memory_usage: number;
  memory_total: number;
  disk_usage: number;
  disk_total: number;
  network_rx: number;
  network_tx: number;
}

interface TelemetryStats {
  events_collected: number;
  events_sent: number;
  metrics_collected: number;
  metrics_sent: number;
  logs_collected: number;
  logs_sent: number;
  batches_sent: number;
  batches_failed: number;
  last_batch_sent?: string;
}

interface MetricsData {
  counters: Record<string, number>;
  gauges: Record<string, number>;
  histograms: Record<string, HistogramStats>;
  timers: Record<string, TimerStats>;
  
  cpu_history: MetricTimeSeries[];
  memory_history: MetricTimeSeries[];
  api_latency_history: MetricTimeSeries[];
  
  system: SystemResources;
  telemetry: TelemetryStats;
}

const ResourceDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'overview' | 'cpu' | 'memory' | 'network' | 'api' | 'telemetry'>('overview');
  const [metrics, setMetrics] = useState<MetricsData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  const [refreshInterval, setRefreshInterval] = useState<number>(5000);

  // Fetch metrics data
  const fetchMetrics = async () => {
    try {
      setLoading(true);
      
      // Invoke Tauri commands to get metrics
      const [
        counters,
        gauges,
        histograms,
        timers,
        cpuHistory,
        memoryHistory,
        apiLatencyHistory,
        systemResources,
        telemetryStats
      ] = await Promise.all([
        invoke('get_counters_report'),
        invoke('get_gauges_report'),
        invoke('get_histograms_report'),
        invoke('get_timers_report'),
        invoke('get_metric_history', { metricName: 'system.cpu_usage', metricType: 'Gauge' }),
        invoke('get_metric_history', { metricName: 'system.memory_usage', metricType: 'Gauge' }),
        invoke('get_metric_history', { metricName: 'api.latency', metricType: 'Timer' }),
        invoke('get_system_resources'),
        invoke('get_telemetry_stats')
      ]);
      
      setMetrics({
        counters: counters as Record<string, number>,
        gauges: gauges as Record<string, number>,
        histograms: histograms as Record<string, HistogramStats>,
        timers: timers as Record<string, TimerStats>,
        cpu_history: cpuHistory as MetricTimeSeries[],
        memory_history: memoryHistory as MetricTimeSeries[],
        api_latency_history: apiLatencyHistory as MetricTimeSeries[],
        system: systemResources as SystemResources,
        telemetry: telemetryStats as TelemetryStats
      });
      
      setError(null);
    } catch (err) {
      console.error('Error fetching metrics:', err);
      setError(`Failed to fetch metrics: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // Set up auto-refresh
  useEffect(() => {
    fetchMetrics();
    
    let intervalId: number;
    
    if (autoRefresh) {
      intervalId = window.setInterval(fetchMetrics, refreshInterval);
    }
    
    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [autoRefresh, refreshInterval]);

  // Format bytes to a human-readable format
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  // Format milliseconds for display
  const formatTime = (ms: number): string => {
    if (ms < 1) {
      return `${(ms * 1000).toFixed(2)} μs`;
    } else if (ms < 1000) {
      return `${ms.toFixed(2)} ms`;
    } else {
      return `${(ms / 1000).toFixed(2)} s`;
    }
  };

  // Render overview tab
  const renderOverview = () => {
    if (!metrics) return null;
    
    return (
      <div className="overview-grid">
        <div className="metric-card">
          <h3>CPU Usage</h3>
          <div className="metric-value">{metrics.system.cpu_usage.toFixed(1)}%</div>
          <ResponsiveContainer width="100%" height={100}>
            <LineChart data={metrics.cpu_history}>
              <Line type="monotone" dataKey="value" stroke="#8884d8" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        
        <div className="metric-card">
          <h3>Memory Usage</h3>
          <div className="metric-value">
            {formatBytes(metrics.system.memory_usage)} / {formatBytes(metrics.system.memory_total)}
          </div>
          <div className="meter">
            <div 
              className="meter-fill" 
              style={{ width: `${(metrics.system.memory_usage / metrics.system.memory_total) * 100}%` }}
            />
          </div>
        </div>
        
        <div className="metric-card">
          <h3>Disk Usage</h3>
          <div className="metric-value">
            {formatBytes(metrics.system.disk_usage)} / {formatBytes(metrics.system.disk_total)}
          </div>
          <div className="meter">
            <div 
              className="meter-fill" 
              style={{ width: `${(metrics.system.disk_usage / metrics.system.disk_total) * 100}%` }}
            />
          </div>
        </div>
        
        <div className="metric-card">
          <h3>Network Traffic</h3>
          <div className="metric-value">
            ↓ {formatBytes(metrics.system.network_rx)}/s &nbsp; ↑ {formatBytes(metrics.system.network_tx)}/s
          </div>
        </div>
        
        <div className="metric-card">
          <h3>API Latency</h3>
          <div className="metric-value">
            {metrics.timers['api.latency'] ? formatTime(metrics.timers['api.latency'].avg_ms) : 'N/A'}
          </div>
          <ResponsiveContainer width="100%" height={100}>
            <LineChart data={metrics.api_latency_history}>
              <Line type="monotone" dataKey="value" stroke="#82ca9d" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        
        <div className="metric-card">
          <h3>Messages</h3>
          <div className="metric-value">
            {metrics.counters['message.sent'] || 0} sent / {metrics.counters['message.received'] || 0} received
          </div>
        </div>
      </div>
    );
  };

  // Render CPU tab
  const renderCpuTab = () => {
    if (!metrics) return null;
    
    return (
      <div className="detailed-metrics">
        <div className="metric-detail-card">
          <h3>CPU Usage Over Time</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={metrics.cpu_history}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
              />
              <YAxis domain={[0, 100]} />
              <Tooltip
                formatter={(value: number) => `${value.toFixed(1)}%`}
                labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()}
              />
              <Legend />
              <Line name="CPU Usage %" type="monotone" dataKey="value" stroke="#8884d8" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        
        <div className="metric-detail-card">
          <h3>Process CPU Usage</h3>
          <div className="stats-grid">
            {Object.entries(metrics.gauges)
              .filter(([key]) => key.includes('cpu'))
              .map(([key, value]) => (
                <div key={key} className="stat-item">
                  <div className="stat-label">{key.replace('system.', '')}</div>
                  <div className="stat-value">{value.toFixed(1)}%</div>
                </div>
              ))}
          </div>
        </div>
      </div>
    );
  };

  // Render Memory tab
  const renderMemoryTab = () => {
    if (!metrics) return null;
    
    // Calculate memory usage breakdown
    const usageBreakdown = [
      { name: 'Application', value: metrics.gauges['memory.app'] || 0 },
      { name: 'System', value: metrics.gauges['memory.system'] || 0 },
      { name: 'Other', value: metrics.system.memory_usage - (metrics.gauges['memory.app'] || 0) - (metrics.gauges['memory.system'] || 0) }
    ].filter(item => item.value > 0);
    
    return (
      <div className="detailed-metrics">
        <div className="metric-detail-card">
          <h3>Memory Usage Over Time</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={metrics.memory_history}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()}
              />
              <YAxis 
                domain={[0, metrics.system.memory_total]} 
                tickFormatter={(value) => formatBytes(value)}
              />
              <Tooltip
                formatter={(value: number) => formatBytes(value)}
                labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()}
              />
              <Legend />
              <Line name="Memory Usage" type="monotone" dataKey="value" stroke="#82ca9d" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        
        <div className="metric-detail-card">
          <h3>Memory Usage Breakdown</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={usageBreakdown}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis tickFormatter={(value) => formatBytes(value)} />
              <Tooltip formatter={(value: number) => formatBytes(value)} />
              <Legend />
              <Bar name="Memory Usage" dataKey="value" fill="#8884d8" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    );
  };

  // Render Network tab
  const renderNetworkTab = () => {
    if (!metrics) return null;
    
    return (
      <div className="detailed-metrics">
        <div className="metric-detail-card">
          <h3>Network Throughput</h3>
          <div className="stats-grid">
            <div className="stat-item">
              <div className="stat-label">Download Rate</div>
              <div className="stat-value">{formatBytes(metrics.system.network_rx)}/s</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Upload Rate</div>
              <div className="stat-value">{formatBytes(metrics.system.network_tx)}/s</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Total Downloaded</div>
              <div className="stat-value">{formatBytes(metrics.counters['network.rx.total'] || 0)}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Total Uploaded</div>
              <div className="stat-value">{formatBytes(metrics.counters['network.tx.total'] || 0)}</div>
            </div>
          </div>
        </div>
        
        <div className="metric-detail-card">
          <h3>API Requests</h3>
          <div className="stats-grid">
            <div className="stat-item">
              <div className="stat-label">Total Requests</div>
              <div className="stat-value">{metrics.counters['api.requests.total'] || 0}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Successful Requests</div>
              <div className="stat-value">{metrics.counters['api.requests.success'] || 0}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Failed Requests</div>
              <div className="stat-value">{metrics.counters['api.requests.failure'] || 0}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Error Rate</div>
              <div className="stat-value">
                {metrics.counters['api.requests.total'] 
                  ? ((metrics.counters['api.requests.failure'] || 0) / metrics.counters['api.requests.total'] * 100).toFixed(2)
                  : 0}%
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  };

  // Render API tab
  const renderApiTab = () => {
    if (!metrics) return null;
    
    return (
      <div className="detailed-metrics">
        <div className="metric-detail-card">
          <h3>API Latency Over Time</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={metrics.api_latency_history}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()}
              />
              <YAxis tickFormatter={(value) => formatTime(value)} />
              <Tooltip
                formatter={(value: number) => formatTime(value)}
                labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()}
              />
              <Legend />
              <Line name="API Latency" type="monotone" dataKey="value" stroke="#8884d8" dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
        
        <div className="metric-detail-card">
          <h3>API Endpoints</h3>
          <table className="metrics-table">
            <thead>
              <tr>
                <th>Endpoint</th>
                <th>Requests</th>
                <th>Avg. Latency</th>
                <th>Success Rate</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(metrics.timers)
                .filter(([key]) => key.startsWith('api.endpoint.'))
                .map(([key, value]) => {
                  const endpoint = key.replace('api.endpoint.', '');
                  const requests = metrics.counters[`api.endpoint.${endpoint}.count`] || 0;
                  const successes = metrics.counters[`api.endpoint.${endpoint}.success`] || 0;
                  const successRate = requests > 0 ? (successes / requests * 100).toFixed(1) : '100.0';
                  
                  return (
                    <tr key={key}>
                      <td>{endpoint}</td>
                      <td>{requests}</td>
                      <td>{formatTime(value.avg_ms)}</td>
                      <td className={Number(successRate) < 99 ? 'stat-warning' : ''}>{successRate}%</td>
                    </tr>
                  );
                })}
            </tbody>
          </table>
        </div>
      </div>
    );
  };

  // Render Telemetry tab
  const renderTelemetryTab = () => {
    if (!metrics) return null;
    
    return (
      <div className="detailed-metrics">
        <div className="metric-detail-card">
          <h3>Telemetry Overview</h3>
          <div className="stats-grid">
            <div className="stat-item">
              <div className="stat-label">Events Collected</div>
              <div className="stat-value">{metrics.telemetry.events_collected}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Events Sent</div>
              <div className="stat-value">{metrics.telemetry.events_sent}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Metrics Collected</div>
              <div className="stat-value">{metrics.telemetry.metrics_collected}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Metrics Sent</div>
              <div className="stat-value">{metrics.telemetry.metrics_sent}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Logs Collected</div>
              <div className="stat-value">{metrics.telemetry.logs_collected}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Logs Sent</div>
              <div className="stat-value">{metrics.telemetry.logs_sent}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Batches Sent</div>
              <div className="stat-value">{metrics.telemetry.batches_sent}</div>
            </div>
            <div className="stat-item">
              <div className="stat-label">Batches Failed</div>
              <div className="stat-value">{metrics.telemetry.batches_failed}</div>
            </div>
          </div>
        </div>
        
        <div className="metric-detail-card">
          <h3>Telemetry Status</h3>
          <div className="stats-detail">
            <div className="stat-row">
              <div className="stat-label">Last Batch Sent</div>
              <div className="stat-value">
                {metrics.telemetry.last_batch_sent 
                  ? new Date(metrics.telemetry.last_batch_sent).toLocaleString()
                  : 'Never'}
              </div>
            </div>
            <div className="stat-row">
              <div className="stat-label">Sending Rate</div>
              <div className="stat-value">
                {metrics.counters['telemetry.sending_rate'] 
                  ? `${metrics.counters['telemetry.sending_rate'].toFixed(2)} batches/minute`
                  : 'N/A'}
              </div>
            </div>
            <div className="stat-row">
              <div className="stat-label">Success Rate</div>
              <div className="stat-value">
                {metrics.telemetry.batches_sent + metrics.telemetry.batches_failed > 0
                  ? `${(metrics.telemetry.batches_sent / (metrics.telemetry.batches_sent + metrics.telemetry.batches_failed) * 100).toFixed(1)}%`
                  : 'N/A'}
              </div>
            </div>
            <div className="stat-row">
              <div className="stat-label">Connection Status</div>
              <div className="stat-value">
                <span className={metrics.gauges['telemetry.connected'] > 0 ? 'status-indicator connected' : 'status-indicator disconnected'}></span>
                {metrics.gauges['telemetry.connected'] > 0 ? 'Connected' : 'Disconnected'}
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="resource-dashboard">
      <div className="dashboard-header">
        <h2>System Resources & Performance</h2>
        <div className="dashboard-controls">
          <label className="refresh-control">
            <input 
              type="checkbox" 
              checked={autoRefresh} 
              onChange={(e) => setAutoRefresh(e.target.checked)} 
            />
            Auto-refresh
          </label>
          {autoRefresh && (
            <select 
              value={refreshInterval} 
              onChange={(e) => setRefreshInterval(Number(e.target.value))}
              className="interval-select"
            >
              <option value={1000}>Every 1s</option>
              <option value={5000}>Every 5s</option>
              <option value={10000}>Every 10s</option>
              <option value={30000}>Every 30s</option>
              <option value={60000}>Every minute</option>
            </select>
          )}
          <button 
            onClick={fetchMetrics} 
            className="refresh-button"
            disabled={loading}
          >
            {loading ? 'Refreshing...' : 'Refresh Now'}
          </button>
        </div>
      </div>
      
      <div className="dashboard-tabs">
        <button 
          className={activeTab === 'overview' ? 'active' : ''} 
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button 
          className={activeTab === 'cpu' ? 'active' : ''} 
          onClick={() => setActiveTab('cpu')}
        >
          CPU
        </button>
        <button 
          className={activeTab === 'memory' ? 'active' : ''} 
          onClick={() => setActiveTab('memory')}
        >
          Memory
        </button>
        <button 
          className={activeTab === 'network' ? 'active' : ''} 
          onClick={() => setActiveTab('network')}
        >
          Network
        </button>
        <button 
          className={activeTab === 'api' ? 'active' : ''} 
          onClick={() => setActiveTab('api')}
        >
          API
        </button>
        <button 
          className={activeTab === 'telemetry' ? 'active' : ''} 
          onClick={() => setActiveTab('telemetry')}
        >
          Telemetry
        </button>
      </div>
      
      <div className="dashboard-content">
        {error && (
          <div className="error-message">
            {error}
          </div>
        )}
        
        {loading && !metrics ? (
          <div className="loading-indicator">
            <div className="spinner"></div>
            <p>Loading metrics data...</p>
          </div>
        ) : (
          <>
            {activeTab === 'overview' && renderOverview()}
            {activeTab === 'cpu' && renderCpuTab()}
            {activeTab === 'memory' && renderMemoryTab()}
            {activeTab === 'network' && renderNetworkTab()}
            {activeTab === 'api' && renderApiTab()}
            {activeTab === 'telemetry' && renderTelemetryTab()}
          </>
        )}
      </div>
    </div>
  );
};

export default ResourceDashboard;
