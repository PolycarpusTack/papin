import React, { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import LLMMetricsPrivacyNotice from './LLMMetricsPrivacyNotice';
import { 
  Box, Button, FormControl, FormLabel, Select, TextField, 
  Typography, Paper, Divider, CircularProgress, 
  List, ListItem, ListItemText, ListItemSecondaryAction,
  IconButton, Alert, Snackbar, Tabs, Tab, Switch, FormControlLabel,
  MenuItem, InputAdornment, Grid, Card, CardContent, Chip, Accordion,
  AccordionSummary, AccordionDetails, Dialog, DialogTitle, DialogContent,
  DialogActions, LinearProgress
} from '@mui/material';
import RefreshIcon from '@mui/icons-material/Refresh';
import DownloadIcon from '@mui/icons-material/Download';
import DeleteIcon from '@mui/icons-material/Delete';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ErrorIcon from '@mui/icons-material/Error';
import PauseCircleIcon from '@mui/icons-material/PauseCircle';
import WifiIcon from '@mui/icons-material/Wifi';
import WifiOffIcon from '@mui/icons-material/WifiOff';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import SettingsIcon from '@mui/icons-material/Settings';
import KeyIcon from '@mui/icons-material/Key';
import HttpIcon from '@mui/icons-material/Http';
import MemoryIcon from '@mui/icons-material/Memory';
import StorageIcon from '@mui/icons-material/Storage';
import ModelTrainingIcon from '@mui/icons-material/ModelTraining';
import ChatIcon from '@mui/icons-material/Chat';
import ImageIcon from '@mui/icons-material/Image';
import CodeIcon from '@mui/icons-material/Code';
import LanguageIcon from '@mui/icons-material/Language';
import InfoIcon from '@mui/icons-material/Info';

// Types
interface ModelInfo {
  id: string;
  name: string;
  description: string;
  size_bytes: number;
  is_downloaded: boolean;
  provider_metadata: Record<string, any>;
  provider: string;
  supports_text_generation: boolean;
  supports_completion: boolean;
  supports_chat: boolean;
  supports_embeddings: boolean;
  supports_image_generation: boolean;
  quantization?: string;
  parameter_count_b?: number;
  context_length?: number;
  model_family?: string;
  created_at?: string;
  tags: string[];
  license?: string;
}

interface DownloadStatus {
  status: string;
  NotStarted?: {};
  InProgress?: { 
    percent: number,
    bytes_downloaded?: number,
    total_bytes?: number,
    eta_seconds?: number,
    bytes_per_second?: number
  };
  Completed?: {
    completed_at?: string,
    duration_seconds?: number
  };
  Failed?: { 
    reason: string,
    error_code?: string,
    failed_at?: string
  };
  Cancelled?: {
    cancelled_at?: string
  };
}

interface CommandResponse<T> {
  success: boolean;
  error?: string;
  data?: T;
}

interface ProviderInfo {
  provider_type: string;
  name: string;
  description: string;
  version: string;
  default_endpoint: string;
  supports_text_generation: boolean;
  supports_chat: boolean;
  supports_embeddings: boolean;
  requires_api_key: boolean;
}

interface AvailabilityResult {
  available: boolean;
  version?: string;
  error?: string;
  response_time_ms?: number;
}

interface ProviderConfig {
  provider_type: string;
  endpoint_url: string;
  api_key?: string;
  default_model?: string;
  enable_advanced_config: boolean;
  advanced_config: Record<string, any>;
}

interface OfflineConfig {
  enabled: boolean;
  auto_switch: boolean;
  llm_config: ProviderConfig;
  max_history_size: number;
  enable_debug: boolean;
}

// Helper functions
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const getDownloadStatusText = (status: DownloadStatus): string => {
  switch (status.status) {
    case 'NotStarted':
      return 'Not Downloaded';
    case 'InProgress':
      if (status.InProgress?.percent !== undefined) {
        return `Downloading ${status.InProgress.percent.toFixed(2)}%`;
      }
      return 'Downloading...';
    case 'Completed':
      return 'Downloaded';
    case 'Failed':
      return `Failed: ${status.Failed?.reason}`;
    case 'Cancelled':
      return 'Cancelled';
    default:
      return 'Unknown';
  }
};

const getDownloadStatusIcon = (status: DownloadStatus) => {
  switch (status.status) {
    case 'NotStarted':
      return null;
    case 'InProgress':
      return <CircularProgress size={20} variant="determinate" value={status.InProgress?.percent || 0} />;
    case 'Completed':
      return <CheckCircleIcon color="success" />;
    case 'Failed':
      return <ErrorIcon color="error" />;
    case 'Cancelled':
      return <PauseCircleIcon color="warning" />;
    default:
      return null;
  }
};

const formatEta = (seconds?: number): string => {
  if (seconds === undefined) return 'Unknown';
  
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  } else if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${mins}m ${secs}s`;
  } else {
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  }
};

// Convert provider_type string to display name
const getProviderDisplayName = (providerType: string): string => {
  switch (providerType) {
    case 'Ollama':
      return 'Ollama';
    case 'LocalAI':
      return 'LocalAI';
    case 'LlamaCpp':
      return 'llama.cpp';
    case 'Custom':
      return 'Custom Provider';
    default:
      if (providerType.startsWith('Custom(')) {
        return providerType.substring(7, providerType.length - 1);
      }
      return providerType;
  }
};

// Main component
const OfflineSettings: React.FC = () => {
  // State for offline mode
  const [isOfflineMode, setIsOfflineMode] = useState<boolean>(false);
  const [autoSwitchMode, setAutoSwitchMode] = useState<boolean>(true);
  const [isConnected, setIsConnected] = useState<boolean>(true);
  
  // State for providers
  const [availableProviders, setAvailableProviders] = useState<ProviderInfo[]>([]);
  const [providerAvailability, setProviderAvailability] = useState<Record<string, AvailabilityResult>>({});
  const [selectedProviderType, setSelectedProviderType] = useState<string>('Ollama');
  const [providerEndpoint, setProviderEndpoint] = useState<string>('http://localhost:11434');
  const [apiKey, setApiKey] = useState<string>('');
  const [defaultModel, setDefaultModel] = useState<string>('');
  const [enableAdvancedConfig, setEnableAdvancedConfig] = useState<boolean>(false);
  const [advancedConfig, setAdvancedConfig] = useState<Record<string, any>>({});
  
  // State for models
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [downloadedModels, setDownloadedModels] = useState<ModelInfo[]>([]);
  const [downloadStatus, setDownloadStatus] = useState<Record<string, DownloadStatus>>({});
  
  // UI state
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [tabIndex, setTabIndex] = useState<number>(0);
  const [configChanged, setConfigChanged] = useState<boolean>(false);
  const [openModelInfoDialog, setOpenModelInfoDialog] = useState<boolean>(false);
  const [selectedModelInfo, setSelectedModelInfo] = useState<ModelInfo | null>(null);
  
  // Load initial data
  useEffect(() => {
    fetchOfflineConfig();
    checkNetworkStatus();
    fetchProviders();
    
    // Set up interval to refresh download status and network status
    const intervalId = setInterval(() => {
      refreshDownloadStatus();
      checkNetworkStatus();
    }, 5000);
    
    return () => clearInterval(intervalId);
  }, []);
  
  // Effect to update configChanged flag
  useEffect(() => {
    setConfigChanged(true);
  }, [selectedProviderType, providerEndpoint, apiKey, defaultModel, enableAdvancedConfig, advancedConfig, isOfflineMode, autoSwitchMode]);
  
  // Effect to fetch models when provider changes
  useEffect(() => {
    fetchModels();
  }, [selectedProviderType, providerEndpoint]);
  
  // Check network status
  const checkNetworkStatus = async () => {
    try {
      const response: CommandResponse<boolean> = await invoke('check_network');
      if (response.success && response.data !== undefined) {
        setIsConnected(response.data);
      }
    } catch (err) {
      console.error('Failed to check network status:', err);
      setIsConnected(false);
    }
  };
  
  // Fetch provider information
  const fetchProviders = async () => {
    try {
      // Get all providers
      const allProvidersResponse: CommandResponse<ProviderInfo[]> = await invoke('get_all_providers');
      if (allProvidersResponse.success && allProvidersResponse.data) {
        setAvailableProviders(allProvidersResponse.data);
      }
      
      // Get provider availability
      const availabilityResponse: CommandResponse<Record<string, AvailabilityResult>> = await invoke('get_all_provider_availability');
      if (availabilityResponse.success && availabilityResponse.data) {
        setProviderAvailability(availabilityResponse.data);
      }
    } catch (err) {
      console.error('Failed to fetch providers:', err);
      setError(`Failed to fetch providers: ${err}`);
    }
  };
  
  // Fetch offline configuration
  const fetchOfflineConfig = async () => {
    setLoading(true);
    try {
      const configResponse: CommandResponse<OfflineConfig> = await invoke('get_offline_config');
      
      if (configResponse.success && configResponse.data) {
        const config = configResponse.data;
        
        // Update state
        setIsOfflineMode(config.enabled);
        setAutoSwitchMode(config.auto_switch);
        
        const llmConfig = config.llm_config;
        setSelectedProviderType(llmConfig.provider_type);
        setProviderEndpoint(llmConfig.endpoint_url);
        setApiKey(llmConfig.api_key || '');
        setDefaultModel(llmConfig.default_model || '');
        setEnableAdvancedConfig(llmConfig.enable_advanced_config);
        setAdvancedConfig(llmConfig.advanced_config);
        
        // Reset changed flag as we just loaded the config
        setConfigChanged(false);
        
        // Fetch models for the current provider
        fetchModels();
      }
    } catch (err) {
      console.error('Failed to fetch offline config:', err);
      setError(`Failed to fetch offline configuration: ${err}`);
    } finally {
      setLoading(false);
    }
  };
  
  // Save offline configuration
  const saveOfflineConfig = async () => {
    setLoading(true);
    try {
      const config: OfflineConfig = {
        enabled: isOfflineMode,
        auto_switch: autoSwitchMode,
        llm_config: {
          provider_type: selectedProviderType,
          endpoint_url: providerEndpoint,
          api_key: apiKey || undefined,
          default_model: defaultModel || undefined,
          enable_advanced_config: enableAdvancedConfig,
          advanced_config: advancedConfig,
        },
        max_history_size: 100, // Default value
        enable_debug: false,   // Default value
      };
      
      const response: CommandResponse<boolean> = await invoke('update_offline_config', { config });
      
      if (response.success) {
        setSuccess('Configuration saved successfully');
        setConfigChanged(false);
        fetchModels(); // Refresh models with the new configuration
      } else if (response.error) {
        setError(`Failed to save configuration: ${response.error}`);
      }
    } catch (err) {
      console.error('Failed to save configuration:', err);
      setError(`Failed to save configuration: ${err}`);
    } finally {
      setLoading(false);
    }
  };
  
  // Toggle offline mode
  const toggleOfflineMode = async () => {
    setIsOfflineMode(!isOfflineMode);
  };
  
  // Toggle auto switch mode
  const toggleAutoSwitchMode = async () => {
    setAutoSwitchMode(!autoSwitchMode);
  };
  
  // Check if the selected provider is available
  const isSelectedProviderAvailable = useMemo(() => {
    const availability = providerAvailability[selectedProviderType];
    return availability?.available || false;
  }, [selectedProviderType, providerAvailability]);
  
  // Fetch models from the backend
  const fetchModels = async () => {
    if (!isSelectedProviderAvailable) {
      // Don't try to fetch models if the provider is not available
      setAvailableModels([]);
      setDownloadedModels([]);
      return;
    }
    
    setLoading(true);
    try {
      const availableResponse: CommandResponse<ModelInfo[]> = await invoke('list_available_models');
      const downloadedResponse: CommandResponse<ModelInfo[]> = await invoke('list_downloaded_models');
      
      if (availableResponse.success && availableResponse.data) {
        setAvailableModels(availableResponse.data);
      } else if (availableResponse.error) {
        console.error('Error fetching available models:', availableResponse.error);
      }
      
      if (downloadedResponse.success && downloadedResponse.data) {
        setDownloadedModels(downloadedResponse.data);
      } else if (downloadedResponse.error) {
        console.error('Error fetching downloaded models:', downloadedResponse.error);
      }
      
      refreshDownloadStatus();
    } catch (err) {
      console.error('Failed to fetch models:', err);
      setError(`Failed to fetch models: ${err}`);
    } finally {
      setLoading(false);
    }
  };
  
  // Refresh download status for all models
  const refreshDownloadStatus = async () => {
    // Skip if no models or provider not available
    if (availableModels.length === 0 || !isSelectedProviderAvailable) {
      return;
    }
    
    const newStatus: Record<string, DownloadStatus> = {};
    
    for (const model of availableModels) {
      try {
        const response: CommandResponse<DownloadStatus> = await invoke('get_download_status', { modelId: model.id });
        if (response.success && response.data) {
          newStatus[model.id] = response.data;
        }
      } catch (err) {
        console.error(`Failed to get download status for ${model.id}:`, err);
      }
    }
    
    setDownloadStatus(newStatus);
  };
  
  // Download a model
  const downloadModel = async (modelId: string) => {
    try {
      const response: CommandResponse<boolean> = await invoke('download_model', { modelId });
      
      if (response.success) {
        // Update the status immediately to show download started
        setDownloadStatus(prev => ({
          ...prev,
          [modelId]: { 
            status: 'InProgress',
            InProgress: { percent: 0 } 
          },
        }));
        
        setSuccess(`Started downloading model ${modelId}`);
        
        // Start polling the download status
        const intervalId = setInterval(async () => {
          try {
            const statusResponse: CommandResponse<DownloadStatus> = await invoke('get_download_status', { modelId });
            if (statusResponse.success && statusResponse.data) {
              setDownloadStatus(prev => ({
                ...prev,
                [modelId]: statusResponse.data,
              }));
              
              // Check if download is complete or failed
              if (statusResponse.data.status === 'Completed' || 
                  statusResponse.data.status === 'Failed' || 
                  statusResponse.data.status === 'Cancelled') {
                clearInterval(intervalId);
                if (statusResponse.data.status === 'Completed') {
                  setSuccess(`Model ${modelId} downloaded successfully`);
                  fetchModels(); // Refresh model lists
                } else if (statusResponse.data.status === 'Failed') {
                  setError(`Failed to download model ${modelId}: ${statusResponse.data.Failed?.reason}`);
                }
              }
            }
          } catch (err) {
            console.error(`Failed to get download status for ${modelId}:`, err);
          }
        }, 2000);
        
        // Cleanup interval after 10 minutes (failsafe)
        setTimeout(() => clearInterval(intervalId), 10 * 60 * 1000);
      } else if (response.error) {
        setError(`Failed to download model: ${response.error}`);
      }
    } catch (err) {
      setError(`Failed to download model: ${err}`);
    }
  };
  
  // Cancel a model download
  const cancelDownload = async (modelId: string) => {
    try {
      const response: CommandResponse<boolean> = await invoke('cancel_download', { modelId });
      
      if (response.success) {
        setSuccess(`Cancelled download of model ${modelId}`);
        
        // Update status immediately
        setDownloadStatus(prev => ({
          ...prev,
          [modelId]: { 
            status: 'Cancelled',
            Cancelled: { cancelled_at: new Date().toISOString() } 
          },
        }));
      } else if (response.error) {
        setError(`Failed to cancel download: ${response.error}`);
      }
    } catch (err) {
      setError(`Failed to cancel download: ${err}`);
    }
  };
  
  // Delete a model
  const deleteModel = async (modelId: string) => {
    try {
      const response: CommandResponse<boolean> = await invoke('delete_model', { modelId });
      
      if (response.success) {
        setSuccess(`Deleted model ${modelId}`);
        fetchModels(); // Refresh model lists
      } else if (response.error) {
        setError(`Failed to delete model: ${response.error}`);
      }
    } catch (err) {
      setError(`Failed to delete model: ${err}`);
    }
  };
  
  // Open model info dialog
  const openModelInfo = (model: ModelInfo) => {
    setSelectedModelInfo(model);
    setOpenModelInfoDialog(true);
  };
  
  // Handle tab change
  const handleTabChange = (_: React.SyntheticEvent, newValue: number) => {
    setTabIndex(newValue);
  };
  
  // Filter models by provider
  const filteredAvailableModels = useMemo(() => {
    return availableModels.filter(model => 
      model.provider === selectedProviderType || 
      // Special case for legacy models without provider field
      (model.provider === undefined && selectedProviderType === 'Ollama')
    );
  }, [availableModels, selectedProviderType]);
  
  const filteredDownloadedModels = useMemo(() => {
    return downloadedModels.filter(model => 
      model.provider === selectedProviderType ||
      // Special case for legacy models without provider field
      (model.provider === undefined && selectedProviderType === 'Ollama')
    );
  }, [downloadedModels, selectedProviderType]);

  return (
    <Box sx={{ padding: 3, maxWidth: 1000, margin: '0 auto' }}>
      <Typography variant="h4" gutterBottom>
        Offline Settings
      </Typography>
      
      {/* Network Status */}
      <Paper sx={{ p: 3, mb: 4 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
          <Typography variant="h6">
            Network Status
          </Typography>
          
          <Box sx={{ display: 'flex', alignItems: 'center' }}>
            {isConnected ? (
              <>
                <WifiIcon color="success" sx={{ mr: 1 }} />
                <Typography color="success.main">Connected</Typography>
              </>
            ) : (
              <>
                <WifiOffIcon color="error" sx={{ mr: 1 }} />
                <Typography color="error.main">Disconnected</Typography>
              </>
            )}
            <IconButton 
              size="small" 
              sx={{ ml: 2 }}
              onClick={checkNetworkStatus}
            >
              <RefreshIcon />
            </IconButton>
          </Box>
        </Box>
        
        <Grid container spacing={3}>
          <Grid item xs={12} md={6}>
            <FormControlLabel
              control={
                <Switch
                  checked={isOfflineMode}
                  onChange={toggleOfflineMode}
                  color="primary"
                />
              }
              label="Enable Offline Mode"
            />
            
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              {isOfflineMode 
                ? "The application will use local LLMs for text generation." 
                : "The application will use cloud services for text generation."}
            </Typography>
          </Grid>
          
          <Grid item xs={12} md={6}>
            <FormControlLabel
              control={
                <Switch
                  checked={autoSwitchMode}
                  onChange={toggleAutoSwitchMode}
                  color="primary"
                />
              }
              label="Auto-switch based on connectivity"
            />
            
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              {autoSwitchMode 
                ? "The application will automatically switch between online and offline modes based on network connectivity." 
                : "You will need to manually switch between online and offline modes."}
            </Typography>
          </Grid>
        </Grid>
      </Paper>
      
      {/* Provider Selection */}
      <Paper sx={{ p: 3, mb: 4 }}>
        <Typography variant="h6" gutterBottom>
          Local LLM Provider
        </Typography>
        
        <Grid container spacing={3}>
          <Grid item xs={12} md={6}>
            <FormControl fullWidth sx={{ mb: 2 }}>
              <FormLabel>Provider Type</FormLabel>
              <Select
                value={selectedProviderType}
                onChange={(e) => setSelectedProviderType(e.target.value)}
              >
                {availableProviders.map((provider) => (
                  <MenuItem 
                    key={provider.provider_type} 
                    value={provider.provider_type}
                    disabled={!providerAvailability[provider.provider_type]?.available}
                  >
                    {getProviderDisplayName(provider.provider_type)}
                    {providerAvailability[provider.provider_type]?.available && 
                      <CheckCircleIcon color="success" fontSize="small" sx={{ ml: 1 }} />
                    }
                  </MenuItem>
                ))}
              </Select>
              {!isSelectedProviderAvailable && (
                <Alert severity="warning" sx={{ mt: 1 }}>
                  This provider is not available. Make sure it's installed and running at the specified endpoint.
                </Alert>
              )}
            </FormControl>
            
            {selectedProviderType && availableProviders.find(p => p.provider_type === selectedProviderType) && (
              <Card variant="outlined" sx={{ mb: 2, mt: 2 }}>
                <CardContent>
                  <Typography variant="subtitle1">
                    {getProviderDisplayName(selectedProviderType)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    {availableProviders.find(p => p.provider_type === selectedProviderType)?.description}
                  </Typography>
                  
                  <Box sx={{ mt: 2, display: 'flex', flexWrap: 'wrap', gap: 1 }}>
                    {availableProviders.find(p => p.provider_type === selectedProviderType)?.supports_text_generation && (
                      <Chip icon={<LanguageIcon />} label="Text Generation" size="small" color="primary" variant="outlined" />
                    )}
                    {availableProviders.find(p => p.provider_type === selectedProviderType)?.supports_chat && (
                      <Chip icon={<ChatIcon />} label="Chat" size="small" color="primary" variant="outlined" />
                    )}
                    {availableProviders.find(p => p.provider_type === selectedProviderType)?.supports_embeddings && (
                      <Chip icon={<MemoryIcon />} label="Embeddings" size="small" color="primary" variant="outlined" />
                    )}
                  </Box>
                </CardContent>
              </Card>
            )}
          </Grid>
          
          <Grid item xs={12} md={6}>
            <FormControl fullWidth sx={{ mb: 2 }}>
              <FormLabel>Endpoint URL</FormLabel>
              <TextField
                value={providerEndpoint}
                onChange={(e) => setProviderEndpoint(e.target.value)}
                placeholder={
                  selectedProviderType === 'Ollama' 
                    ? "http://localhost:11434" 
                    : selectedProviderType === 'LocalAI'
                      ? "http://localhost:8080"
                      : "http://localhost:8000"
                }
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <HttpIcon />
                    </InputAdornment>
                  ),
                }}
              />
            </FormControl>
            
            {availableProviders.find(p => p.provider_type === selectedProviderType)?.requires_api_key && (
              <FormControl fullWidth sx={{ mb: 2 }}>
                <FormLabel>API Key</FormLabel>
                <TextField
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="Enter API key"
                  InputProps={{
                    startAdornment: (
                      <InputAdornment position="start">
                        <KeyIcon />
                      </InputAdornment>
                    ),
                  }}
                />
              </FormControl>
            )}
            
            <FormControl fullWidth sx={{ mb: 3 }}>
              <FormLabel>Default Model</FormLabel>
              <Select
                value={defaultModel}
                onChange={(e) => setDefaultModel(e.target.value)}
                displayEmpty
              >
                <MenuItem value="">
                  <em>None</em>
                </MenuItem>
                {filteredDownloadedModels.map((model) => (
                  <MenuItem key={model.id} value={model.id}>
                    {model.name}
                  </MenuItem>
                ))}
              </Select>
              <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                The default model will be used when no specific model is specified.
              </Typography>
            </FormControl>
            
            <Accordion 
              expanded={enableAdvancedConfig}
              onChange={() => setEnableAdvancedConfig(!enableAdvancedConfig)}
              sx={{ mb: 2 }}
            >
              <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <SettingsIcon sx={{ mr: 1 }} />
                  <Typography>Advanced Configuration</Typography>
                </Box>
              </AccordionSummary>
              <AccordionDetails>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                  These settings are for advanced users only. Incorrect settings may cause issues.
                </Typography>
                
                {/* This could be expanded with provider-specific advanced settings */}
                <Alert severity="info">
                  Advanced configuration options are not currently implemented.
                </Alert>
              </AccordionDetails>
            </Accordion>
          </Grid>
        </Grid>
        
        <Box sx={{ mt: 3, display: 'flex', justifyContent: 'space-between' }}>
          <Button
            variant="outlined"
            onClick={fetchProviders}
            startIcon={<RefreshIcon />}
            disabled={loading}
          >
            Refresh Providers
          </Button>
          
          <Button
            variant="contained"
            onClick={saveOfflineConfig}
            disabled={loading || !configChanged}
          >
            {loading ? <CircularProgress size={24} /> : 'Save Configuration'}
          </Button>
        </Box>
      </Paper>
      
      {/* LLM Performance Metrics */}
      <Paper sx={{ p: 3, mb: 4 }}>
        <Typography variant="h6" gutterBottom>
          LLM Performance Metrics
        </Typography>
        
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Help improve the performance of local LLM providers by allowing anonymous collection of metrics data.
          This data helps us optimize the application and understand which models and providers work best.
        </Typography>
        
        <LLMMetricsPrivacyNotice />
      </Paper>
      
      {/* Models */}
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="h6">
            Models
          </Typography>
          
          <Button 
            startIcon={<RefreshIcon />}
            onClick={fetchModels}
            disabled={loading || !isSelectedProviderAvailable}
          >
            Refresh
          </Button>
        </Box>
        
        {!isSelectedProviderAvailable ? (
          <Alert severity="warning">
            Provider is not available. Make sure it's installed and running at {providerEndpoint}.
          </Alert>
        ) : (
          <>
            <Tabs value={tabIndex} onChange={handleTabChange} sx={{ mb: 2 }}>
              <Tab label="Available Models" />
              <Tab label="Downloaded Models" />
            </Tabs>
            
            <Divider sx={{ mb: 2 }} />
            
            {loading && <LinearProgress sx={{ mb: 2 }} />}
            
            {tabIndex === 0 && !loading && (
              <>
                {filteredAvailableModels.length === 0 ? (
                  <Alert severity="info">No models available. Make sure your provider is running.</Alert>
                ) : (
                  <List>
                    {filteredAvailableModels.map((model) => {
                      const status = downloadStatus[model.id] || { status: 'NotStarted' };
                      const isDownloaded = status.status === 'Completed';
                      const isDownloading = status.status === 'InProgress';
                      
                      return (
                        <ListItem 
                          key={model.id} 
                          divider
                          secondaryAction={
                            <Box sx={{ display: 'flex', alignItems: 'center' }}>
                              {getDownloadStatusIcon(status)}
                              
                              <Box sx={{ ml: 1 }}>
                                {!isDownloaded && !isDownloading && (
                                  <IconButton edge="end" onClick={() => downloadModel(model.id)}>
                                    <DownloadIcon />
                                  </IconButton>
                                )}
                                
                                {isDownloading && (
                                  <IconButton edge="end" onClick={() => cancelDownload(model.id)}>
                                    <PauseCircleIcon />
                                  </IconButton>
                                )}
                                
                                {isDownloaded && (
                                  <IconButton edge="end" onClick={() => deleteModel(model.id)}>
                                    <DeleteIcon />
                                  </IconButton>
                                )}
                              </Box>
                              
                              <IconButton edge="end" onClick={() => openModelInfo(model)}>
                                <InfoIcon />
                              </IconButton>
                            </Box>
                          }
                        >
                          <ListItemText
                            primary={
                              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                                <Typography>{model.name}</Typography>
                                {model.parameter_count_b && (
                                  <Chip 
                                    label={`${model.parameter_count_b}B`} 
                                    size="small" 
                                    color="primary" 
                                    sx={{ ml: 1 }}
                                  />
                                )}
                                {model.quantization && (
                                  <Chip 
                                    label={model.quantization} 
                                    size="small" 
                                    color="secondary" 
                                    sx={{ ml: 1 }}
                                  />
                                )}
                              </Box>
                            }
                            secondary={
                              <>
                                <Typography component="span" variant="body2" color="text.primary">
                                  {formatBytes(model.size_bytes)}
                                </Typography>
                                {" • "}
                                {getDownloadStatusText(status)}
                                
                                {status.status === 'InProgress' && status.InProgress?.eta_seconds !== undefined && (
                                  <>
                                    {" • "}
                                    ETA: {formatEta(status.InProgress.eta_seconds)}
                                  </>
                                )}
                                
                                {status.status === 'InProgress' && status.InProgress?.bytes_per_second !== undefined && (
                                  <>
                                    {" • "}
                                    Speed: {formatBytes(status.InProgress.bytes_per_second ?? 0)}/s
                                  </>
                                )}
                                
                                <Box sx={{ mt: 1 }}>
                                  {model.description}
                                </Box>
                                
                                {model.tags.length > 0 && (
                                  <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                                    {model.tags.map((tag, index) => (
                                      <Chip 
                                        key={index} 
                                        label={tag} 
                                        size="small" 
                                        variant="outlined"
                                        sx={{ 
                                          fontSize: '0.7rem', 
                                          height: '20px',
                                          '& .MuiChip-label': { px: 1 } 
                                        }}
                                      />
                                    ))}
                                  </Box>
                                )}
                              </>
                            }
                          />
                        </ListItem>
                      );
                    })}
                  </List>
                )}
              </>
            )}
            
            {tabIndex === 1 && !loading && (
              <>
                {filteredDownloadedModels.length === 0 ? (
                  <Alert severity="info">No models downloaded yet.</Alert>
                ) : (
                  <List>
                    {filteredDownloadedModels.map((model) => (
                      <ListItem 
                        key={model.id} 
                        divider
                        secondaryAction={
                          <Box sx={{ display: 'flex', alignItems: 'center' }}>
                            <IconButton edge="end" onClick={() => deleteModel(model.id)}>
                              <DeleteIcon />
                            </IconButton>
                            <IconButton edge="end" onClick={() => openModelInfo(model)}>
                              <InfoIcon />
                            </IconButton>
                          </Box>
                        }
                      >
                        <ListItemText
                          primary={
                            <Box sx={{ display: 'flex', alignItems: 'center' }}>
                              <Typography>{model.name}</Typography>
                              {model.parameter_count_b && (
                                <Chip 
                                  label={`${model.parameter_count_b}B`} 
                                  size="small" 
                                  color="primary" 
                                  sx={{ ml: 1 }}
                                />
                              )}
                              {model.quantization && (
                                <Chip 
                                  label={model.quantization} 
                                  size="small" 
                                  color="secondary" 
                                  sx={{ ml: 1 }}
                                />
                              )}
                              {model.id === defaultModel && (
                                <Chip 
                                  label="Default" 
                                  size="small" 
                                  color="success" 
                                  sx={{ ml: 1 }}
                                />
                              )}
                            </Box>
                          }
                          secondary={
                            <>
                              <Typography component="span" variant="body2" color="text.primary">
                                {formatBytes(model.size_bytes)}
                              </Typography>
                              
                              <Box sx={{ mt: 1 }}>
                                {model.description}
                              </Box>
                              
                              {model.tags.length > 0 && (
                                <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                                  {model.tags.map((tag, index) => (
                                    <Chip 
                                      key={index} 
                                      label={tag} 
                                      size="small" 
                                      variant="outlined"
                                      sx={{ 
                                        fontSize: '0.7rem', 
                                        height: '20px',
                                        '& .MuiChip-label': { px: 1 } 
                                      }}
                                    />
                                  ))}
                                </Box>
                              )}
                              
                              <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                                {model.supports_text_generation && (
                                  <Chip icon={<LanguageIcon fontSize="small" />} label="Text" size="small" variant="outlined" />
                                )}
                                {model.supports_chat && (
                                  <Chip icon={<ChatIcon fontSize="small" />} label="Chat" size="small" variant="outlined" />
                                )}
                                {model.supports_embeddings && (
                                  <Chip icon={<MemoryIcon fontSize="small" />} label="Embeddings" size="small" variant="outlined" />
                                )}
                                {model.supports_image_generation && (
                                  <Chip icon={<ImageIcon fontSize="small" />} label="Images" size="small" variant="outlined" />
                                )}
                              </Box>
                            </>
                          }
                        />
                      </ListItem>
                    ))}
                  </List>
                )}
              </>
            )}
          </>
        )}
      </Paper>
      
      {/* Model Info Dialog */}
      <Dialog 
        open={openModelInfoDialog} 
        onClose={() => setOpenModelInfoDialog(false)}
        fullWidth
        maxWidth="md"
      >
        <DialogTitle>
          Model Information: {selectedModelInfo?.name}
        </DialogTitle>
        <DialogContent dividers>
          {selectedModelInfo && (
            <Grid container spacing={2}>
              <Grid item xs={12} md={6}>
                <Typography variant="subtitle1">General Information</Typography>
                <List dense>
                  <ListItem>
                    <ListItemText primary="ID" secondary={selectedModelInfo.id} />
                  </ListItem>
                  <ListItem>
                    <ListItemText primary="Name" secondary={selectedModelInfo.name} />
                  </ListItem>
                  <ListItem>
                    <ListItemText primary="Provider" secondary={getProviderDisplayName(selectedModelInfo.provider)} />
                  </ListItem>
                  <ListItem>
                    <ListItemText primary="Description" secondary={selectedModelInfo.description} />
                  </ListItem>
                  <ListItem>
                    <ListItemText primary="Size" secondary={formatBytes(selectedModelInfo.size_bytes)} />
                  </ListItem>
                  {selectedModelInfo.created_at && (
                    <ListItem>
                      <ListItemText 
                        primary="Created At" 
                        secondary={new Date(selectedModelInfo.created_at).toLocaleString()} 
                      />
                    </ListItem>
                  )}
                  {selectedModelInfo.license && (
                    <ListItem>
                      <ListItemText primary="License" secondary={selectedModelInfo.license} />
                    </ListItem>
                  )}
                </List>
              </Grid>
              
              <Grid item xs={12} md={6}>
                <Typography variant="subtitle1">Model Specifications</Typography>
                <List dense>
                  {selectedModelInfo.parameter_count_b && (
                    <ListItem>
                      <ListItemText 
                        primary="Parameter Count" 
                        secondary={`${selectedModelInfo.parameter_count_b} billion`} 
                      />
                    </ListItem>
                  )}
                  {selectedModelInfo.quantization && (
                    <ListItem>
                      <ListItemText primary="Quantization" secondary={selectedModelInfo.quantization} />
                    </ListItem>
                  )}
                  {selectedModelInfo.context_length && (
                    <ListItem>
                      <ListItemText 
                        primary="Context Length" 
                        secondary={`${selectedModelInfo.context_length} tokens`} 
                      />
                    </ListItem>
                  )}
                  {selectedModelInfo.model_family && (
                    <ListItem>
                      <ListItemText primary="Model Family" secondary={selectedModelInfo.model_family} />
                    </ListItem>
                  )}
                </List>
                
                <Typography variant="subtitle1" sx={{ mt: 2 }}>Capabilities</Typography>
                <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 1 }}>
                  {selectedModelInfo.supports_text_generation && (
                    <Chip icon={<LanguageIcon />} label="Text Generation" color="primary" variant="outlined" />
                  )}
                  {selectedModelInfo.supports_chat && (
                    <Chip icon={<ChatIcon />} label="Chat" color="primary" variant="outlined" />
                  )}
                  {selectedModelInfo.supports_completion && (
                    <Chip icon={<CodeIcon />} label="Completion" color="primary" variant="outlined" />
                  )}
                  {selectedModelInfo.supports_embeddings && (
                    <Chip icon={<MemoryIcon />} label="Embeddings" color="primary" variant="outlined" />
                  )}
                  {selectedModelInfo.supports_image_generation && (
                    <Chip icon={<ImageIcon />} label="Image Generation" color="primary" variant="outlined" />
                  )}
                </Box>
                
                {selectedModelInfo.tags.length > 0 && (
                  <>
                    <Typography variant="subtitle1" sx={{ mt: 2 }}>Tags</Typography>
                    <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                      {selectedModelInfo.tags.map((tag, index) => (
                        <Chip key={index} label={tag} size="small" />
                      ))}
                    </Box>
                  </>
                )}
              </Grid>
              
              {Object.keys(selectedModelInfo.provider_metadata).length > 0 && (
                <Grid item xs={12}>
                  <Typography variant="subtitle1">Provider-Specific Metadata</Typography>
                  <Box sx={{ mt: 1, maxHeight: '200px', overflow: 'auto', bgcolor: '#f5f5f5', p: 2, borderRadius: 1 }}>
                    <pre>{JSON.stringify(selectedModelInfo.provider_metadata, null, 2)}</pre>
                  </Box>
                </Grid>
              )}
            </Grid>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpenModelInfoDialog(false)}>
            Close
          </Button>
        </DialogActions>
      </Dialog>
      
      {/* Notifications */}
      <Snackbar
        open={!!error}
        autoHideDuration={6000}
        onClose={() => setError(null)}
      >
        <Alert onClose={() => setError(null)} severity="error">
          {error}
        </Alert>
      </Snackbar>
      
      <Snackbar
        open={!!success}
        autoHideDuration={6000}
        onClose={() => setSuccess(null)}
      >
        <Alert onClose={() => setSuccess(null)} severity="success">
          {success}
        </Alert>
      </Snackbar>
    </Box>
  );
};

export default OfflineSettings;