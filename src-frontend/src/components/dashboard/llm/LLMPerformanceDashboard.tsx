import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, 
  Tooltip, Legend, ResponsiveContainer, PieChart, Pie, Cell
} from 'recharts';
import { 
  Box, Typography, Paper, Divider, CircularProgress, 
  Alert, Tabs, Tab, FormControlLabel, Switch, Select, 
  MenuItem, InputLabel, FormControl, Grid, Card, CardContent,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow
} from '@mui/material';

// Types for our metrics data
interface ProviderPerformanceMetrics {
  provider_type: string;
  generation_count: number;
  successful_generations: number;
  failed_generations: number;
  avg_tokens_per_second: number;
  avg_latency_ms: number;
  p90_latency_ms: number;
  p99_latency_ms: number;
  avg_cpu_usage: number;
  avg_memory_usage: number;
  last_updated: string;
}

interface ModelPerformanceMetrics {
  model_id: string;
  provider_type: string;
  generation_count: number;
  tokens_generated: number;
  successful_generations: number;
  failed_generations: number;
  avg_tokens_per_second: number;
  avg_latency_ms: number;
  p90_latency_ms: number;
  p99_latency_ms: number;
  avg_time_to_first_token_ms: number;
  avg_tokens_per_request: number;
  last_updated: string;
}

interface LLMMetricsData {
  provider_metrics: Record<string, ProviderPerformanceMetrics>;
  model_metrics: Record<string, ModelPerformanceMetrics>;
  active_provider?: string;
  default_model?: string;
  enabled: boolean;
}

// Component to format time in ms to a human-readable format
const formatTime = (ms: number): string => {
  if (ms < 1) {
    return `${(ms * 1000).toFixed(2)} μs`;
  } else if (ms < 1000) {
    return `${ms.toFixed(2)} ms`;
  } else {
    return `${(ms / 1000).toFixed(2)} s`;
  }
};

// Color palette for charts
const COLORS = ['#8884d8', '#82ca9d', '#ffc658', '#ff8042', '#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

// Main dashboard component
const LLMPerformanceDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState<number>(0);
  const [metrics, setMetrics] = useState<LLMMetricsData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  const [refreshInterval, setRefreshInterval] = useState<number>(5000);
  const [selectedProvider, setSelectedProvider] = useState<string>('all');
  const [selectedModel, setSelectedModel] = useState<string>('all');

  // Fetch metrics data
  const fetchMetrics = async () => {
    try {
      setLoading(true);
      
      // Invoke Tauri commands to get metrics
      const [
        providerMetrics,
        modelMetrics,
        activeProvider,
        defaultModel,
        metricsEnabled
      ] = await Promise.all([
        invoke('get_llm_provider_metrics'),
        invoke('get_llm_model_metrics'),
        invoke('get_active_llm_provider'),
        invoke('get_default_llm_model'),
        invoke('get_llm_metrics_enabled')
      ]);
      
      setMetrics({
        provider_metrics: providerMetrics as Record<string, ProviderPerformanceMetrics>,
        model_metrics: modelMetrics as Record<string, ModelPerformanceMetrics>,
        active_provider: activeProvider as string,
        default_model: defaultModel as string,
        enabled: metricsEnabled as boolean
      });
      
      setError(null);
    } catch (err) {
      console.error('Error fetching LLM metrics:', err);
      setError(`Failed to fetch LLM metrics: ${err}`);
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

  // Handle tab change
  const handleTabChange = (_: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  // Filter provider metrics based on selection
  const filteredProviderMetrics = React.useMemo(() => {
    if (!metrics) return {};
    
    if (selectedProvider === 'all') {
      return metrics.provider_metrics;
    }
    
    return Object.entries(metrics.provider_metrics)
      .filter(([key]) => key === selectedProvider)
      .reduce((obj, [key, value]) => {
        obj[key] = value;
        return obj;
      }, {} as Record<string, ProviderPerformanceMetrics>);
  }, [metrics, selectedProvider]);

  // Filter model metrics based on selection
  const filteredModelMetrics = React.useMemo(() => {
    if (!metrics) return {};
    
    let filtered = metrics.model_metrics;
    
    // Filter by provider if needed
    if (selectedProvider !== 'all') {
      filtered = Object.entries(filtered)
        .filter(([_, value]) => value.provider_type === selectedProvider)
        .reduce((obj, [key, value]) => {
          obj[key] = value;
          return obj;
        }, {} as Record<string, ModelPerformanceMetrics>);
    }
    
    // Filter by model if needed
    if (selectedModel !== 'all') {
      filtered = Object.entries(filtered)
        .filter(([_, value]) => value.model_id === selectedModel)
        .reduce((obj, [key, value]) => {
          obj[key] = value;
          return obj;
        }, {} as Record<string, ModelPerformanceMetrics>);
    }
    
    return filtered;
  }, [metrics, selectedProvider, selectedModel]);

  // Get all available providers for filter dropdown
  const availableProviders = React.useMemo(() => {
    if (!metrics) return [];
    return [...new Set(Object.values(metrics.provider_metrics).map(m => m.provider_type))];
  }, [metrics]);

  // Get all available models for filter dropdown
  const availableModels = React.useMemo(() => {
    if (!metrics) return [];
    return [...new Set(Object.values(metrics.model_metrics).map(m => m.model_id))];
  }, [metrics]);

  // Prepare data for provider comparison chart
  const providerComparisonData = React.useMemo(() => {
    if (!metrics) return [];
    
    return Object.values(filteredProviderMetrics).map(provider => ({
      name: provider.provider_type,
      'Avg Tokens/sec': provider.avg_tokens_per_second,
      'Avg Latency (ms)': provider.avg_latency_ms,
      'Success Rate': provider.generation_count > 0 
        ? (provider.successful_generations / provider.generation_count) * 100 
        : 0
    }));
  }, [filteredProviderMetrics]);

  // Prepare data for model comparison chart
  const modelComparisonData = React.useMemo(() => {
    if (!metrics) return [];
    
    return Object.values(filteredModelMetrics).map(model => ({
      name: model.model_id,
      'Avg Tokens/sec': model.avg_tokens_per_second,
      'Avg Latency (ms)': model.avg_latency_ms,
      'Avg Time to First Token (ms)': model.avg_time_to_first_token_ms,
      'Success Rate': model.generation_count > 0 
        ? (model.successful_generations / model.generation_count) * 100 
        : 0
    }));
  }, [filteredModelMetrics]);

  // Prepare data for success/failure pie chart
  const successFailureData = React.useMemo(() => {
    if (!metrics) return [];
    
    const totalSuccess = Object.values(filteredModelMetrics).reduce((sum, model) => 
      sum + model.successful_generations, 0);
    
    const totalFailed = Object.values(filteredModelMetrics).reduce((sum, model) => 
      sum + model.failed_generations, 0);
    
    return [
      { name: 'Successful', value: totalSuccess },
      { name: 'Failed', value: totalFailed }
    ];
  }, [filteredModelMetrics]);

  // Render overview tab
  const renderOverview = () => {
    if (!metrics) return null;
    
    // Get total metrics across all providers and models
    const totalGenerations = Object.values(metrics.provider_metrics).reduce((sum, p) => 
      sum + p.generation_count, 0);
    
    const totalSuccess = Object.values(metrics.provider_metrics).reduce((sum, p) => 
      sum + p.successful_generations, 0);
    
    const totalFailed = Object.values(metrics.provider_metrics).reduce((sum, p) => 
      sum + p.failed_generations, 0);
    
    const totalTokens = Object.values(metrics.model_metrics).reduce((sum, m) => 
      sum + m.tokens_generated, 0);
    
    // Calculate averages
    const avgTokensPerSecond = Object.values(metrics.model_metrics).reduce((sum, m) => 
      sum + m.avg_tokens_per_second, 0) / Math.max(1, Object.values(metrics.model_metrics).length);
    
    const avgLatency = Object.values(metrics.model_metrics).reduce((sum, m) => 
      sum + m.avg_latency_ms, 0) / Math.max(1, Object.values(metrics.model_metrics).length);

    return (
      <Box sx={{ mt: 3 }}>
        <Grid container spacing={3}>
          {/* Summary Cards */}
          <Grid item xs={12} md={6} lg={3}>
            <Card sx={{ height: '100%' }}>
              <CardContent>
                <Typography variant="h6" color="text.secondary" gutterBottom>
                  Total Generations
                </Typography>
                <Typography variant="h3">
                  {totalGenerations.toLocaleString()}
                </Typography>
                <Box sx={{ display: 'flex', mt: 2, justifyContent: 'space-between' }}>
                  <Typography variant="body2" color="success.main">
                    Success: {totalSuccess.toLocaleString()}
                  </Typography>
                  <Typography variant="body2" color="error.main">
                    Failed: {totalFailed.toLocaleString()}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={6} lg={3}>
            <Card sx={{ height: '100%' }}>
              <CardContent>
                <Typography variant="h6" color="text.secondary" gutterBottom>
                  Tokens Generated
                </Typography>
                <Typography variant="h3">
                  {totalTokens.toLocaleString()}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                  Across all models and providers
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={6} lg={3}>
            <Card sx={{ height: '100%' }}>
              <CardContent>
                <Typography variant="h6" color="text.secondary" gutterBottom>
                  Avg. Throughput
                </Typography>
                <Typography variant="h3">
                  {avgTokensPerSecond.toFixed(2)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                  Tokens per second
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={6} lg={3}>
            <Card sx={{ height: '100%' }}>
              <CardContent>
                <Typography variant="h6" color="text.secondary" gutterBottom>
                  Avg. Latency
                </Typography>
                <Typography variant="h3">
                  {formatTime(avgLatency)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                  Response time
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          {/* Success/Failure Chart */}
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Generation Success Rate
                </Typography>
                {successFailureData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <PieChart>
                      <Pie
                        data={successFailureData}
                        cx="50%"
                        cy="50%"
                        labelLine={false}
                        outerRadius={80}
                        fill="#8884d8"
                        dataKey="value"
                        label={({ name, percent }) => `${name}: ${(percent * 100).toFixed(0)}%`}
                      >
                        {successFailureData.map((_, index) => (
                          <Cell key={`cell-${index}`} fill={index === 0 ? '#4caf50' : '#f44336'} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value: number) => [value.toLocaleString(), 'Generations']} />
                      <Legend />
                    </PieChart>
                  </ResponsiveContainer>
                ) : (
                  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 300 }}>
                    <Typography variant="body1" color="text.secondary">
                      No generation data available
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
          
          {/* Provider Comparison */}
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Provider Performance Comparison
                </Typography>
                {providerComparisonData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={providerComparisonData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis yAxisId="left" orientation="left" stroke="#8884d8" />
                      <YAxis yAxisId="right" orientation="right" stroke="#82ca9d" />
                      <Tooltip />
                      <Legend />
                      <Bar yAxisId="left" dataKey="Avg Tokens/sec" fill="#8884d8" />
                      <Bar yAxisId="right" dataKey="Avg Latency (ms)" fill="#82ca9d" />
                    </BarChart>
                  </ResponsiveContainer>
                ) : (
                  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 300 }}>
                    <Typography variant="body1" color="text.secondary">
                      No provider comparison data available
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
          
          {/* Active Provider Info */}
          <Grid item xs={12}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Current Configuration
                </Typography>
                <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 4, mt: 2 }}>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Active Provider
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {metrics.active_provider || 'None'}
                    </Typography>
                  </Box>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Default Model
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {metrics.default_model || 'None'}
                    </Typography>
                  </Box>
                  <Box>
                    <Typography variant="body2" color="text.secondary">
                      Metrics Enabled
                    </Typography>
                    <Typography variant="body1" fontWeight="bold" color={metrics.enabled ? 'success.main' : 'error.main'}>
                      {metrics.enabled ? 'Yes' : 'No'}
                    </Typography>
                  </Box>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Box>
    );
  };

  // Render providers tab
  const renderProvidersTab = () => {
    if (!metrics) return null;
    
    return (
      <Box sx={{ mt: 3 }}>
        <Grid container spacing={3}>
          {/* Provider Filter */}
          <Grid item xs={12}>
            <FormControl fullWidth variant="outlined" size="small">
              <InputLabel id="provider-filter-label">Provider</InputLabel>
              <Select
                labelId="provider-filter-label"
                value={selectedProvider}
                onChange={(e) => setSelectedProvider(e.target.value)}
                label="Provider"
              >
                <MenuItem value="all">All Providers</MenuItem>
                {availableProviders.map((provider) => (
                  <MenuItem key={provider} value={provider}>{provider}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
          
          {/* Provider Metrics Table */}
          <Grid item xs={12}>
            <TableContainer component={Paper}>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Provider</TableCell>
                    <TableCell align="right">Generations</TableCell>
                    <TableCell align="right">Success Rate</TableCell>
                    <TableCell align="right">Avg. Tokens/sec</TableCell>
                    <TableCell align="right">Avg. Latency</TableCell>
                    <TableCell align="right">P90 Latency</TableCell>
                    <TableCell align="right">P99 Latency</TableCell>
                    <TableCell align="right">CPU Usage</TableCell>
                    <TableCell align="right">Memory Usage</TableCell>
                    <TableCell align="right">Last Updated</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {Object.values(filteredProviderMetrics).length > 0 ? (
                    Object.values(filteredProviderMetrics).map((provider) => (
                      <TableRow key={provider.provider_type}>
                        <TableCell component="th" scope="row">
                          {provider.provider_type}
                          {metrics.active_provider === provider.provider_type && (
                            <Typography component="span" color="primary" sx={{ ml: 1, fontSize: '0.75rem' }}>
                              (Active)
                            </Typography>
                          )}
                        </TableCell>
                        <TableCell align="right">{provider.generation_count.toLocaleString()}</TableCell>
                        <TableCell align="right">
                          {provider.generation_count > 0 
                            ? `${((provider.successful_generations / provider.generation_count) * 100).toFixed(1)}%` 
                            : 'N/A'}
                        </TableCell>
                        <TableCell align="right">{provider.avg_tokens_per_second.toFixed(2)}</TableCell>
                        <TableCell align="right">{formatTime(provider.avg_latency_ms)}</TableCell>
                        <TableCell align="right">{formatTime(provider.p90_latency_ms)}</TableCell>
                        <TableCell align="right">{formatTime(provider.p99_latency_ms)}</TableCell>
                        <TableCell align="right">{provider.avg_cpu_usage > 0 ? `${provider.avg_cpu_usage.toFixed(1)}%` : 'N/A'}</TableCell>
                        <TableCell align="right">{provider.avg_memory_usage > 0 ? `${(provider.avg_memory_usage / (1024 * 1024)).toFixed(1)} MB` : 'N/A'}</TableCell>
                        <TableCell align="right">
                          {provider.last_updated ? new Date(provider.last_updated).toLocaleTimeString() : 'N/A'}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell colSpan={10} align="center">
                        No provider metrics available
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </TableContainer>
          </Grid>
          
          {/* Provider Performance Charts */}
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Provider Throughput
                </Typography>
                {providerComparisonData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={providerComparisonData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis />
                      <Tooltip formatter={(value: number) => [`${value.toFixed(2)} tokens/sec`, 'Throughput']} />
                      <Legend />
                      <Bar dataKey="Avg Tokens/sec" fill="#8884d8" />
                    </BarChart>
                  </ResponsiveContainer>
                ) : (
                  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 300 }}>
                    <Typography variant="body1" color="text.secondary">
                      No throughput data available
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Provider Latency
                </Typography>
                {providerComparisonData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={providerComparisonData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis />
                      <Tooltip formatter={(value: number) => [formatTime(value), 'Latency']} />
                      <Legend />
                      <Bar dataKey="Avg Latency (ms)" fill="#82ca9d" />
                    </BarChart>
                  </ResponsiveContainer>
                ) : (
                  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 300 }}>
                    <Typography variant="body1" color="text.secondary">
                      No latency data available
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Box>
    );
  };

  // Render models tab
  const renderModelsTab = () => {
    if (!metrics) return null;
    
    return (
      <Box sx={{ mt: 3 }}>
        <Grid container spacing={3}>
          {/* Filters */}
          <Grid item xs={12} md={6}>
            <FormControl fullWidth variant="outlined" size="small">
              <InputLabel id="provider-filter-label">Provider</InputLabel>
              <Select
                labelId="provider-filter-label"
                value={selectedProvider}
                onChange={(e) => setSelectedProvider(e.target.value)}
                label="Provider"
              >
                <MenuItem value="all">All Providers</MenuItem>
                {availableProviders.map((provider) => (
                  <MenuItem key={provider} value={provider}>{provider}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
          <Grid item xs={12} md={6}>
            <FormControl fullWidth variant="outlined" size="small">
              <InputLabel id="model-filter-label">Model</InputLabel>
              <Select
                labelId="model-filter-label"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                label="Model"
              >
                <MenuItem value="all">All Models</MenuItem>
                {availableModels.map((model) => (
                  <MenuItem key={model} value={model}>{model}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
          
          {/* Model Metrics Table */}
          <Grid item xs={12}>
            <TableContainer component={Paper}>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Model</TableCell>
                    <TableCell>Provider</TableCell>
                    <TableCell align="right">Generations</TableCell>
                    <TableCell align="right">Tokens</TableCell>
                    <TableCell align="right">Success Rate</TableCell>
                    <TableCell align="right">Avg. Tokens/sec</TableCell>
                    <TableCell align="right">Avg. Latency</TableCell>
                    <TableCell align="right">Time to First Token</TableCell>
                    <TableCell align="right">Tokens/Request</TableCell>
                    <TableCell align="right">Last Updated</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {Object.values(filteredModelMetrics).length > 0 ? (
                    Object.values(filteredModelMetrics).map((model) => (
                      <TableRow key={`${model.provider_type}:${model.model_id}`}>
                        <TableCell component="th" scope="row">
                          {model.model_id}
                          {metrics.default_model === model.model_id && (
                            <Typography component="span" color="primary" sx={{ ml: 1, fontSize: '0.75rem' }}>
                              (Default)
                            </Typography>
                          )}
                        </TableCell>
                        <TableCell>{model.provider_type}</TableCell>
                        <TableCell align="right">{model.generation_count.toLocaleString()}</TableCell>
                        <TableCell align="right">{model.tokens_generated.toLocaleString()}</TableCell>
                        <TableCell align="right">
                          {model.generation_count > 0 
                            ? `${((model.successful_generations / model.generation_count) * 100).toFixed(1)}%` 
                            : 'N/A'}
                        </TableCell>
                        <TableCell align="right">{model.avg_tokens_per_second.toFixed(2)}</TableCell>
                        <TableCell align="right">{formatTime(model.avg_latency_ms)}</TableCell>
                        <TableCell align="right">{formatTime(model.avg_time_to_first_token_ms)}</TableCell>
                        <TableCell align="right">{model.avg_tokens_per_request.toFixed(1)}</TableCell>
                        <TableCell align="right">
                          {model.last_updated ? new Date(model.last_updated).toLocaleTimeString() : 'N/A'}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell colSpan={10} align="center">
                        No model metrics available
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </TableContainer>
          </Grid>
          
          {/* Model Performance Charts */}
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Model Throughput
                </Typography>
                {modelComparisonData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={modelComparisonData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis />
                      <Tooltip formatter={(value: number) => [`${value.toFixed(2)} tokens/sec`, 'Throughput']} />
                      <Legend />
                      <Bar dataKey="Avg Tokens/sec" fill="#8884d8" />
                    </BarChart>
                  </ResponsiveContainer>
                ) : (
                  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 300 }}>
                    <Typography variant="body1" color="text.secondary">
                      No throughput data available
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Model Response Time
                </Typography>
                {modelComparisonData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <BarChart data={modelComparisonData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis />
                      <Tooltip formatter={(value: number) => [formatTime(value), 'Time']} />
                      <Legend />
                      <Bar dataKey="Avg Latency (ms)" fill="#82ca9d" />
                      <Bar dataKey="Avg Time to First Token (ms)" fill="#ffc658" />
                    </BarChart>
                  </ResponsiveContainer>
                ) : (
                  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 300 }}>
                    <Typography variant="body1" color="text.secondary">
                      No response time data available
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Box>
    );
  };

  return (
    <Box sx={{ mt: 2 }}>
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="h5">
            Local LLM Performance
          </Typography>
          
          <Box sx={{ display: 'flex', alignItems: 'center' }}>
            <FormControlLabel
              control={
                <Switch
                  checked={autoRefresh}
                  onChange={(e) => setAutoRefresh(e.target.checked)}
                  color="primary"
                />
              }
              label="Auto-refresh"
              sx={{ mr: 2 }}
            />
            
            {autoRefresh && (
              <FormControl variant="outlined" size="small" sx={{ minWidth: 120, mr: 2 }}>
                <Select
                  value={refreshInterval}
                  onChange={(e) => setRefreshInterval(Number(e.target.value))}
                >
                  <MenuItem value={1000}>Every 1s</MenuItem>
                  <MenuItem value={5000}>Every 5s</MenuItem>
                  <MenuItem value={10000}>Every 10s</MenuItem>
                  <MenuItem value={30000}>Every 30s</MenuItem>
                  <MenuItem value={60000}>Every 1m</MenuItem>
                </Select>
              </FormControl>
            )}
            
            <Button 
              variant="outlined" 
              color="primary" 
              onClick={fetchMetrics}
              disabled={loading}
            >
              Refresh
            </Button>
          </Box>
        </Box>
        
        {!metrics?.enabled && (
          <Alert severity="info" sx={{ mb: 3 }}>
            LLM performance metrics collection is currently disabled. Enable it in the offline settings to see performance data.
          </Alert>
        )}
        
        {error && (
          <Alert severity="error" sx={{ mb: 3 }}>
            {error}
          </Alert>
        )}
        
        {loading && !metrics && (
          <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
            <CircularProgress />
          </Box>
        )}
        
        {metrics && (
          <>
            <Tabs 
              value={activeTab} 
              onChange={handleTabChange} 
              sx={{ borderBottom: 1, borderColor: 'divider' }}
            >
              <Tab label="Overview" />
              <Tab label="Providers" />
              <Tab label="Models" />
            </Tabs>
            
            {activeTab === 0 && renderOverview()}
            {activeTab === 1 && renderProvidersTab()}
            {activeTab === 2 && renderModelsTab()}
          </>
        )}
      </Paper>
    </Box>
  );
};

export default LLMPerformanceDashboard;