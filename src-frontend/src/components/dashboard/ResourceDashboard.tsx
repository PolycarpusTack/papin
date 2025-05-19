import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Card, CardHeader, CardContent, CardFooter } from '../../ui/card';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../../ui/tabs';
import { Skeleton } from '../../ui/skeleton';
import { Alert, AlertDescription } from '../../ui/alert';
import { Button } from '../../ui/button';
import { Progress } from '../../ui/progress';
import { Badge } from '../../ui/badge';
import { Separator } from '../../ui/separator';
import { usePlatform } from '../../hooks/usePlatform';
import { 
  CpuIcon, 
  MemoryIcon, 
  HardDriveIcon, 
  NetworkIcon, 
  RefreshCwIcon,
  AlertTriangleIcon,
  GlobeIcon,
  ClockIcon,
  BarChart2Icon
} from '../../ui/icons';

// Types for the metrics data
interface MetricPoint {
  timestamp: number;
  value: number;
}

interface SystemMetrics {
  cpu: {
    usage: number;
    temperature: number;
    history: MetricPoint[];
  };
  memory: {
    total: number;
    used: number;
    available: number;
    history: MetricPoint[];
  };
  disk: {
    total: number;
    used: number;
    available: number;
    readRate: number;
    writeRate: number;
    history: MetricPoint[];
  };
  network: {
    status: 'online' | 'offline' | 'limited';
    downloadRate: number;
    uploadRate: number;
    latency: number;
    history: MetricPoint[];
  }
}

// Initial empty state
const initialMetrics: SystemMetrics = {
  cpu: { usage: 0, temperature: 0, history: [] },
  memory: { total: 0, used: 0, available: 0, history: [] },
  disk: { total: 0, used: 0, available: 0, readRate: 0, writeRate: 0, history: [] },
  network: { status: 'offline', downloadRate: 0, uploadRate: 0, latency: 0, history: [] }
};

/**
 * ResourceDashboard Component
 * 
 * Displays system resource metrics with real-time updates and historical data charts.
 * Features platform-specific optimizations and responsive design.
 */
const ResourceDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<SystemMetrics>(initialMetrics);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<string>('overview');
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  const { platform, isMacOS, isWindows, isLinux } = usePlatform();

  // Format bytes to human-readable format
  const formatBytes = (bytes: number, decimals = 2): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
  };

  // Fetch metrics from backend
  const fetchMetrics = async () => {
    try {
      const systemMetrics = await invoke<SystemMetrics>('get_system_metrics');
      setMetrics(systemMetrics);
      setError(null);
    } catch (err) {
      console.error('Failed to fetch metrics:', err);
      setError(`Failed to fetch system metrics: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // Setup initial data fetch and event listeners
  useEffect(() => {
    fetchMetrics();
    
    // Subscribe to metric updates from backend
    const unsubscribe = listen<SystemMetrics>('metrics-update', (event) => {
      setMetrics(event.payload);
    });

    // Setup auto-refresh interval
    let interval: NodeJS.Timeout | null = null;
    if (autoRefresh) {
      interval = setInterval(fetchMetrics, 5000); // Refresh every 5 seconds
    }

    // Cleanup
    return () => {
      unsubscribe.then(unsub => unsub());
      if (interval) clearInterval(interval);
    };
  }, [autoRefresh]);

  // Apply platform-specific optimizations
  const getRefreshInterval = () => {
    if (isMacOS) return 5000; // macOS uses default 5s
    if (isWindows) return 3000; // Windows refreshes faster
    if (isLinux) return 7000; // Linux uses more conservative refresh
    return 5000; // Default fallback
  };

  // Platform-specific rendering optimizations
  const renderPlatformOptimized = () => {
    if (isMacOS) {
      // macOS-specific styling or components
      return (
        <Badge variant="outline" className="ml-2">
          macOS Optimized
        </Badge>
      );
    } else if (isWindows) {
      // Windows-specific styling or components
      return (
        <Badge variant="outline" className="ml-2">
          Windows Optimized
        </Badge>
      );
    } else if (isLinux) {
      // Linux-specific styling or components
      return (
        <Badge variant="outline" className="ml-2">
          Linux Optimized
        </Badge>
      );
    }
    return null;
  };

  // Different chart colors based on platform
  const getChartColors = () => {
    if (isMacOS) {
      return {
        cpu: '#34c759',
        memory: '#5ac8fa',
        disk: '#ff9500',
        network: '#af52de'
      };
    } else if (isWindows) {
      return {
        cpu: '#0078d7',
        memory: '#7986cb',
        disk: '#ff8f00',
        network: '#00b0ff'
      };
    } else {
      return {
        cpu: '#26a69a',
        memory: '#42a5f5',
        disk: '#ffb300',
        network: '#5c6bc0'
      };
    }
  };

  const chartColors = getChartColors();

  // Render loading state
  if (loading) {
    return (
      <div className="p-4">
        <h2 className="text-2xl font-bold mb-4">System Resources</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[...Array(4)].map((_, index) => (
            <Card key={index}>
              <CardHeader>
                <Skeleton className="h-8 w-3/4" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-24 w-full" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  // Render error state
  if (error) {
    return (
      <div className="p-4">
        <Alert variant="destructive">
          <AlertTriangleIcon className="h-4 w-4" />
          <AlertDescription>
            {error}
            <Button 
              variant="outline" 
              size="sm" 
              className="ml-2" 
              onClick={fetchMetrics}
            >
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="p-4">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-bold">System Resources {renderPlatformOptimized()}</h2>
        <div className="flex items-center space-x-2">
          <Button 
            variant="outline" 
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            {autoRefresh ? 'Auto-refreshing' : 'Auto-refresh paused'}
            <ClockIcon className="ml-2 h-4 w-4" />
          </Button>
          <Button 
            variant="outline" 
            size="sm" 
            onClick={fetchMetrics}
          >
            Refresh
            <RefreshCwIcon className="ml-2 h-4 w-4" />
          </Button>
        </div>
      </div>

      <Tabs 
        defaultValue="overview" 
        value={activeTab} 
        onValueChange={setActiveTab}
        className="space-y-4"
      >
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="cpu">CPU</TabsTrigger>
          <TabsTrigger value="memory">Memory</TabsTrigger>
          <TabsTrigger value="disk">Storage</TabsTrigger>
          <TabsTrigger value="network">Network</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* CPU Overview */}
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="flex items-center">
                  <CpuIcon className="h-4 w-4 mr-2" />
                  <h3 className="font-medium">CPU Usage</h3>
                </div>
                <span className={`text-sm ${metrics.cpu.usage > 80 ? 'text-red-500' : ''}`}>
                  {metrics.cpu.usage.toFixed(1)}%
                </span>
              </CardHeader>
              <CardContent>
                <Progress value={metrics.cpu.usage} className="h-2" />
                <div className="mt-4 h-24">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={metrics.cpu.history}>
                      <Line 
                        type="monotone" 
                        dataKey="value" 
                        stroke={chartColors.cpu} 
                        strokeWidth={2} 
                        dot={false} 
                      />
                      <XAxis dataKey="timestamp" hide />
                      <YAxis hide domain={[0, 100]} />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
              <CardFooter>
                <div className="text-xs text-muted-foreground">
                  Temperature: {metrics.cpu.temperature}°C
                </div>
              </CardFooter>
            </Card>

            {/* Memory Overview */}
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="flex items-center">
                  <MemoryIcon className="h-4 w-4 mr-2" />
                  <h3 className="font-medium">Memory Usage</h3>
                </div>
                <span className="text-sm">
                  {formatBytes(metrics.memory.used)} / {formatBytes(metrics.memory.total)}
                </span>
              </CardHeader>
              <CardContent>
                <Progress 
                  value={(metrics.memory.used / metrics.memory.total) * 100} 
                  className="h-2" 
                />
                <div className="mt-4 h-24">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={metrics.memory.history}>
                      <Line 
                        type="monotone" 
                        dataKey="value" 
                        stroke={chartColors.memory} 
                        strokeWidth={2} 
                        dot={false} 
                      />
                      <XAxis dataKey="timestamp" hide />
                      <YAxis hide domain={[0, 100]} />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
              <CardFooter>
                <div className="text-xs text-muted-foreground">
                  Available: {formatBytes(metrics.memory.available)}
                </div>
              </CardFooter>
            </Card>

            {/* Disk Overview */}
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="flex items-center">
                  <HardDriveIcon className="h-4 w-4 mr-2" />
                  <h3 className="font-medium">Disk Usage</h3>
                </div>
                <span className="text-sm">
                  {formatBytes(metrics.disk.used)} / {formatBytes(metrics.disk.total)}
                </span>
              </CardHeader>
              <CardContent>
                <Progress 
                  value={(metrics.disk.used / metrics.disk.total) * 100} 
                  className="h-2" 
                />
                <div className="grid grid-cols-2 gap-2 mt-4">
                  <div className="text-xs">
                    <div className="text-muted-foreground">Read</div>
                    <div>{formatBytes(metrics.disk.readRate)}/s</div>
                  </div>
                  <div className="text-xs">
                    <div className="text-muted-foreground">Write</div>
                    <div>{formatBytes(metrics.disk.writeRate)}/s</div>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <div className="text-xs text-muted-foreground">
                  Available: {formatBytes(metrics.disk.available)}
                </div>
              </CardFooter>
            </Card>

            {/* Network Overview */}
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="flex items-center">
                  <NetworkIcon className="h-4 w-4 mr-2" />
                  <h3 className="font-medium">Network</h3>
                </div>
                <div>
                  {metrics.network.status === 'online' && (
                    <Badge variant="outline" className="bg-green-500/10 text-green-500">
                      Online
                    </Badge>
                  )}
                  {metrics.network.status === 'limited' && (
                    <Badge variant="outline" className="bg-yellow-500/10 text-yellow-500">
                      Limited
                    </Badge>
                  )}
                  {metrics.network.status === 'offline' && (
                    <Badge variant="outline" className="bg-red-500/10 text-red-500">
                      Offline
                    </Badge>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 gap-4">
                  <div className="text-xs">
                    <div className="text-muted-foreground">Download</div>
                    <div>{formatBytes(metrics.network.downloadRate)}/s</div>
                  </div>
                  <div className="text-xs">
                    <div className="text-muted-foreground">Upload</div>
                    <div>{formatBytes(metrics.network.uploadRate)}/s</div>
                  </div>
                </div>
                <div className="mt-2 text-xs">
                  <div className="text-muted-foreground">Latency</div>
                  <div>{metrics.network.latency.toFixed(1)} ms</div>
                </div>
              </CardContent>
              <CardFooter>
                <div className="text-xs">
                  <Button variant="ghost" size="sm" className="h-6 px-2">
                    <GlobeIcon className="h-3 w-3 mr-1" />
                    Details
                  </Button>
                </div>
              </CardFooter>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="cpu" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium">CPU Performance</h3>
                <div className="text-sm text-muted-foreground">
                  Current Usage: {metrics.cpu.usage.toFixed(1)}%
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={metrics.cpu.history}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis 
                      dataKey="timestamp" 
                      tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                    />
                    <YAxis domain={[0, 100]} />
                    <Tooltip 
                      formatter={(value) => [`${value}%`, 'Usage']}
                      labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                    />
                    <Line 
                      type="monotone" 
                      dataKey="value" 
                      name="CPU Usage" 
                      stroke={chartColors.cpu} 
                      strokeWidth={2} 
                      dot={false} 
                      activeDot={{ r: 6 }} 
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
            <CardFooter>
              <div className="grid grid-cols-3 gap-4 w-full">
                <div>
                  <div className="text-sm font-medium">Temperature</div>
                  <div className="text-2xl">{metrics.cpu.temperature}°C</div>
                </div>
                <div>
                  <div className="text-sm font-medium">Peak Usage</div>
                  <div className="text-2xl">
                    {Math.max(...metrics.cpu.history.map(point => point.value), 0).toFixed(1)}%
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium">Average</div>
                  <div className="text-2xl">
                    {(metrics.cpu.history.reduce((sum, point) => sum + point.value, 0) / 
                      (metrics.cpu.history.length || 1)).toFixed(1)}%
                  </div>
                </div>
              </div>
            </CardFooter>
          </Card>
        </TabsContent>

        <TabsContent value="memory" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium">Memory Usage</h3>
                <div className="text-sm text-muted-foreground">
                  {formatBytes(metrics.memory.used)} / {formatBytes(metrics.memory.total)}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={metrics.memory.history}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis 
                      dataKey="timestamp" 
                      tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                    />
                    <YAxis domain={[0, 100]} />
                    <Tooltip 
                      formatter={(value) => [`${value}%`, 'Usage']}
                      labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                    />
                    <Line 
                      type="monotone" 
                      dataKey="value" 
                      name="Memory Usage" 
                      stroke={chartColors.memory} 
                      strokeWidth={2} 
                      dot={false} 
                      activeDot={{ r: 6 }} 
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>

              <Separator className="my-4" />

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <div className="text-sm font-medium">Total Memory</div>
                  <div className="text-2xl">{formatBytes(metrics.memory.total)}</div>
                </div>
                <div>
                  <div className="text-sm font-medium">Used Memory</div>
                  <div className="text-2xl">{formatBytes(metrics.memory.used)}</div>
                </div>
                <div>
                  <div className="text-sm font-medium">Available Memory</div>
                  <div className="text-2xl">{formatBytes(metrics.memory.available)}</div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="disk" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium">Storage Performance</h3>
                <div className="text-sm text-muted-foreground">
                  {formatBytes(metrics.disk.used)} / {formatBytes(metrics.disk.total)}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <Progress 
                value={(metrics.disk.used / metrics.disk.total) * 100} 
                className="h-4 mb-4" 
              />

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div>
                  <h4 className="text-sm font-medium mb-2">Read Rate</h4>
                  <div className="h-40">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={metrics.disk.history}>
                        <XAxis 
                          dataKey="timestamp" 
                          tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                        />
                        <YAxis />
                        <Tooltip 
                          formatter={(value) => [formatBytes(value), 'Read Rate']}
                          labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                        />
                        <Line 
                          type="monotone" 
                          dataKey="readRate" 
                          name="Read Rate" 
                          stroke={chartColors.disk} 
                          strokeWidth={2} 
                          dot={false} 
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                  <div className="text-center mt-2">
                    Current: {formatBytes(metrics.disk.readRate)}/s
                  </div>
                </div>

                <div>
                  <h4 className="text-sm font-medium mb-2">Write Rate</h4>
                  <div className="h-40">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={metrics.disk.history}>
                        <XAxis 
                          dataKey="timestamp" 
                          tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                        />
                        <YAxis />
                        <Tooltip 
                          formatter={(value) => [formatBytes(value), 'Write Rate']}
                          labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                        />
                        <Line 
                          type="monotone" 
                          dataKey="writeRate" 
                          name="Write Rate" 
                          stroke="#ff9500" 
                          strokeWidth={2} 
                          dot={false} 
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                  <div className="text-center mt-2">
                    Current: {formatBytes(metrics.disk.writeRate)}/s
                  </div>
                </div>
              </div>
            </CardContent>
            <CardFooter>
              <div className="grid grid-cols-3 gap-4 w-full">
                <div>
                  <div className="text-sm font-medium">Total Space</div>
                  <div className="text-xl">{formatBytes(metrics.disk.total)}</div>
                </div>
                <div>
                  <div className="text-sm font-medium">Used Space</div>
                  <div className="text-xl">{formatBytes(metrics.disk.used)}</div>
                </div>
                <div>
                  <div className="text-sm font-medium">Available</div>
                  <div className="text-xl">{formatBytes(metrics.disk.available)}</div>
                </div>
              </div>
            </CardFooter>
          </Card>
        </TabsContent>

        <TabsContent value="network" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium">Network Status</h3>
                <div>
                  {metrics.network.status === 'online' && (
                    <Badge className="bg-green-500 text-white">
                      Online
                    </Badge>
                  )}
                  {metrics.network.status === 'limited' && (
                    <Badge className="bg-yellow-500 text-white">
                      Limited Connectivity
                    </Badge>
                  )}
                  {metrics.network.status === 'offline' && (
                    <Badge className="bg-red-500 text-white">
                      Offline
                    </Badge>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div>
                  <h4 className="text-sm font-medium mb-2">Download Rate</h4>
                  <div className="h-40">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={metrics.network.history}>
                        <XAxis 
                          dataKey="timestamp" 
                          tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                        />
                        <YAxis />
                        <Tooltip 
                          formatter={(value) => [formatBytes(value), 'Download']}
                          labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                        />
                        <Line 
                          type="monotone" 
                          dataKey="downloadRate" 
                          name="Download" 
                          stroke={chartColors.network} 
                          strokeWidth={2} 
                          dot={false} 
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                  <div className="text-center mt-2">
                    Current: {formatBytes(metrics.network.downloadRate)}/s
                  </div>
                </div>

                <div>
                  <h4 className="text-sm font-medium mb-2">Upload Rate</h4>
                  <div className="h-40">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={metrics.network.history}>
                        <XAxis 
                          dataKey="timestamp" 
                          tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                        />
                        <YAxis />
                        <Tooltip 
                          formatter={(value) => [formatBytes(value), 'Upload']}
                          labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                        />
                        <Line 
                          type="monotone" 
                          dataKey="uploadRate" 
                          name="Upload" 
                          stroke="#ff3b30" 
                          strokeWidth={2} 
                          dot={false} 
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                  <div className="text-center mt-2">
                    Current: {formatBytes(metrics.network.uploadRate)}/s
                  </div>
                </div>
              </div>

              <Separator className="my-4" />
              
              <div>
                <h4 className="text-sm font-medium mb-2">Latency</h4>
                <div className="h-40">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={metrics.network.history}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis 
                        dataKey="timestamp" 
                        tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()} 
                      />
                      <YAxis />
                      <Tooltip 
                        formatter={(value) => [`${value} ms`, 'Latency']}
                        labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()} 
                      />
                      <Line 
                        type="monotone" 
                        dataKey="latency" 
                        name="Latency" 
                        stroke="#8884d8" 
                        strokeWidth={2} 
                        dot={false} 
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
                <div className="text-center mt-2">
                  Current: {metrics.network.latency.toFixed(1)} ms
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default ResourceDashboard;
