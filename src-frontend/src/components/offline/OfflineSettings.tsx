import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { 
  Card, 
  CardContent, 
  CardDescription, 
  CardFooter, 
  CardHeader, 
  CardTitle 
} from '../../ui/card';
import { 
  Tabs, 
  TabsContent, 
  TabsList, 
  TabsTrigger 
} from '../../ui/tabs';
import { 
  Form, 
  FormControl, 
  FormDescription, 
  FormField, 
  FormItem, 
  FormLabel, 
  FormMessage 
} from '../../ui/form';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '../../ui/select';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogFooter, 
  DialogHeader, 
  DialogTitle, 
  DialogTrigger 
} from '../../ui/dialog';
import { Alert, AlertDescription, AlertTitle } from '../../ui/alert';
import { Badge } from '../../ui/badge';
import { Button } from '../../ui/button';
import { Input } from '../../ui/input';
import { Label } from '../../ui/label';
import { Slider } from '../../ui/slider';
import { Switch } from '../../ui/switch';
import { Separator } from '../../ui/separator';
import { ToggleGroup, ToggleGroupItem } from '../../ui/toggle-group';
import { Progress } from '../../ui/progress';
import { ScrollArea } from '../../ui/scroll-area';
import { Skeleton } from '../../ui/skeleton';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../../ui/tooltip';
import { usePlatform } from '../../hooks/usePlatform';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { 
  WifiIcon, 
  WifiOffIcon, 
  CloudIcon, 
  CloudOffIcon, 
  RefreshCwIcon, 
  ChevronRightIcon, 
  PlusIcon, 
  TrashIcon, 
  DownloadIcon, 
  CheckIcon, 
  AlertTriangleIcon, 
  InfoIcon, 
  CpuIcon, 
  HardDriveIcon, 
  ClockIcon, 
  SettingsIcon,
  Laptop2Icon,
  ServerIcon,
  GlobeIcon
} from '../../ui/icons';

// Type definitions
interface LLMModel {
  id: string;
  name: string;
  size: number;
  installed: boolean;
  supported: boolean;
  downloading?: boolean;
  downloadProgress?: number;
  default?: boolean;
}

interface OfflineSettings {
  enabled: boolean;
  autoDetectConnectivity: boolean;
  syncFrequency: 'manual' | 'onConnection' | 'hourly' | 'daily';
  syncMetadata: boolean;
  syncContent: boolean;
  syncOnMobileData: boolean;
  checkpointFrequency: 'message' | 'conversation' | 'manual' | 'time';
  checkpointTimeInterval: number;
  checkpointMaxCount: number;
  checkpointMaxSize: number;
  llmEnabled: boolean;
  defaultModelId: string;
  autoEnableOfflineOnPoorConnection: boolean;
  offlineIndicator: boolean;
  removeOldCheckpoints: boolean;
  removeOldCheckpointsAfterDays: number;
}

// Validation schema
const offlineSettingsSchema = z.object({
  enabled: z.boolean(),
  autoDetectConnectivity: z.boolean(),
  syncFrequency: z.enum(['manual', 'onConnection', 'hourly', 'daily']),
  syncMetadata: z.boolean(),
  syncContent: z.boolean(),
  syncOnMobileData: z.boolean(),
  checkpointFrequency: z.enum(['message', 'conversation', 'manual', 'time']),
  checkpointTimeInterval: z.number().min(1).max(60),
  checkpointMaxCount: z.number().min(1).max(1000),
  checkpointMaxSize: z.number().min(1).max(10000),
  llmEnabled: z.boolean(),
  defaultModelId: z.string(),
  autoEnableOfflineOnPoorConnection: z.boolean(),
  offlineIndicator: z.boolean(),
  removeOldCheckpoints: z.boolean(),
  removeOldCheckpointsAfterDays: z.number().min(1).max(365),
});

/**
 * OfflineSettings Component
 * 
 * Provides controls for configuring offline mode behavior, managing local LLM models,
 * synchronization settings, and checkpointing behavior. Includes platform-specific optimizations.
 */
const OfflineSettings: React.FC = () => {
  const [networkStatus, setNetworkStatus] = useState<'online' | 'offline' | 'limited'>('offline');
  const [models, setModels] = useState<LLMModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('general');
  const [syncStatus, setSyncStatus] = useState<'idle' | 'syncing' | 'error'>('idle');
  const [lastSyncTime, setLastSyncTime] = useState<Date | null>(null);
  const [diskSpace, setDiskSpace] = useState({ total: 0, used: 0, available: 0 });
  const [selectedModelForDetails, setSelectedModelForDetails] = useState<LLMModel | null>(null);
  const { platform, isMacOS, isWindows, isLinux } = usePlatform();

  // Initialize form with react-hook-form
  const form = useForm<OfflineSettings>({
    resolver: zodResolver(offlineSettingsSchema),
    defaultValues: {
      enabled: false,
      autoDetectConnectivity: true,
      syncFrequency: 'onConnection',
      syncMetadata: true,
      syncContent: true,
      syncOnMobileData: false,
      checkpointFrequency: 'conversation',
      checkpointTimeInterval: 5,
      checkpointMaxCount: 100,
      checkpointMaxSize: 1000,
      llmEnabled: false,
      defaultModelId: '',
      autoEnableOfflineOnPoorConnection: true,
      offlineIndicator: true,
      removeOldCheckpoints: true,
      removeOldCheckpointsAfterDays: 30,
    },
  });

  // Format bytes to human-readable size
  const formatBytes = (bytes: number, decimals = 2): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
  };

  // Load settings from backend
  const loadSettings = async () => {
    try {
      setLoading(true);
      const settings = await invoke<OfflineSettings>('get_offline_settings');
      form.reset(settings);
      
      // Fetch available models
      const availableModels = await invoke<LLMModel[]>('get_available_llm_models');
      setModels(availableModels);
      
      // Get network status
      const status = await invoke<'online' | 'offline' | 'limited'>('get_network_status');
      setNetworkStatus(status);
      
      // Get disk space info
      const space = await invoke<{ total: number, used: number, available: number }>('get_storage_info');
      setDiskSpace(space);
      
      // Get last sync time
      const lastSync = await invoke<number | null>('get_last_sync_time');
      if (lastSync) {
        setLastSyncTime(new Date(lastSync));
      }
    } catch (error) {
      console.error('Failed to load offline settings:', error);
    } finally {
      setLoading(false);
    }
  };

  // Save settings to backend
  const saveSettings = async (data: OfflineSettings) => {
    try {
      setSaving(true);
      setSaveError(null);
      await invoke('save_offline_settings', { settings: data });
      
      // If offline mode was just enabled, set up appropriate listeners
      if (data.enabled && !form.getValues('enabled')) {
        setupOfflineListeners();
      }
    } catch (error) {
      console.error('Failed to save offline settings:', error);
      setSaveError(`Failed to save settings: ${error}`);
    } finally {
      setSaving(false);
    }
  };

  // Set up listeners for network and sync events
  const setupOfflineListeners = async () => {
    try {
      const unlistenNetwork = await listen<'online' | 'offline' | 'limited'>(
        'network-status-changed', 
        (event) => {
          setNetworkStatus(event.payload);
        }
      );
      
      const unlistenSync = await listen<'idle' | 'syncing' | 'error'>(
        'sync-status-changed', 
        (event) => {
          setSyncStatus(event.payload);
        }
      );
      
      const unlistenSyncTime = await listen<number>(
        'last-sync-updated', 
        (event) => {
          setLastSyncTime(new Date(event.payload));
        }
      );
      
      return () => {
        unlistenNetwork.then(unsub => unsub());
        unlistenSync.then(unsub => unsub());
        unlistenSyncTime.then(unsub => unsub());
      };
    } catch (error) {
      console.error('Failed to set up offline listeners:', error);
    }
  };

  // Download model
  const downloadModel = async (modelId: string) => {
    try {
      await invoke('download_llm_model', { modelId });
      // The model list will be updated via an event listener
    } catch (error) {
      console.error(`Failed to download model ${modelId}:`, error);
    }
  };

  // Delete model
  const deleteModel = async (modelId: string) => {
    try {
      await invoke('delete_llm_model', { modelId });
      setModels(models.map(model => 
        model.id === modelId 
          ? { ...model, installed: false, downloading: false, downloadProgress: 0 } 
          : model
      ));
    } catch (error) {
      console.error(`Failed to delete model ${modelId}:`, error);
    }
  };

  // Manually trigger sync
  const triggerSync = async () => {
    try {
      setSyncStatus('syncing');
      await invoke('sync_offline_data');
      // The sync status will be updated via an event listener
    } catch (error) {
      console.error('Failed to trigger sync:', error);
      setSyncStatus('error');
    }
  };

  // Set default model
  const setDefaultModel = async (modelId: string) => {
    try {
      await invoke('set_default_llm_model', { modelId });
      form.setValue('defaultModelId', modelId);
      setModels(models.map(model => ({
        ...model,
        default: model.id === modelId
      })));
    } catch (error) {
      console.error(`Failed to set default model ${modelId}:`, error);
    }
  };

  // Platform-specific UI adjustments
  const getPlatformSpecificElements = () => {
    if (isMacOS) {
      return {
        fontFamily: 'SF Pro Text, -apple-system, BlinkMacSystemFont, sans-serif',
        accentColor: '#007AFF',
        modelLocation: '~/Library/Application Support/Papin/models',
        iconSize: 16,
        borderRadius: 'rounded-md'
      };
    } else if (isWindows) {
      return {
        fontFamily: 'Segoe UI, system-ui, sans-serif',
        accentColor: '#0078D4',
        modelLocation: '%APPDATA%\\Papin\\models',
        iconSize: 18,
        borderRadius: 'rounded-sm'
      };
    } else {
      // Linux or other
      return {
        fontFamily: 'Ubuntu, system-ui, sans-serif',
        accentColor: '#E95420',
        modelLocation: '~/.local/share/papin/models',
        iconSize: 16,
        borderRadius: 'rounded'
      };
    }
  };

  const platformUI = getPlatformSpecificElements();

  // Run only once on component mount
  useEffect(() => {
    loadSettings();
    
    // Set up listeners for model download progress
    const setupModelListeners = async () => {
      const unlistenModelProgress = await listen<{ modelId: string, progress: number }>(
        'model-download-progress', 
        (event) => {
          const { modelId, progress } = event.payload;
          setModels(prevModels => prevModels.map(model => 
            model.id === modelId 
              ? { ...model, downloading: true, downloadProgress: progress } 
              : model
          ));
        }
      );
      
      const unlistenModelComplete = await listen<{ modelId: string }>(
        'model-download-complete', 
        (event) => {
          const { modelId } = event.payload;
          setModels(prevModels => prevModels.map(model => 
            model.id === modelId 
              ? { ...model, installed: true, downloading: false, downloadProgress: 100 } 
              : model
          ));
        }
      );
      
      const unlistenModelError = await listen<{ modelId: string, error: string }>(
        'model-download-error', 
        (event) => {
          const { modelId } = event.payload;
          setModels(prevModels => prevModels.map(model => 
            model.id === modelId 
              ? { ...model, downloading: false, downloadProgress: 0 } 
              : model
          ));
        }
      );
      
      return () => {
        unlistenModelProgress.then(unsub => unsub());
        unlistenModelComplete.then(unsub => unsub());
        unlistenModelError.then(unsub => unsub());
      };
    };
    
    const cleanup = setupModelListeners();
    
    // If offline mode is enabled, set up network and sync listeners
    if (form.getValues('enabled')) {
      const offlineCleanup = setupOfflineListeners();
      return () => {
        cleanup.then(fn => fn && fn());
        offlineCleanup.then(fn => fn && fn());
      };
    }
    
    return () => {
      cleanup.then(fn => fn && fn());
    };
  }, []);

  // Get platform-specific network status indicator
  const getNetworkStatusIndicator = () => {
    let icon, label, color;
    
    switch (networkStatus) {
      case 'online':
        icon = <WifiIcon className="h-4 w-4" />;
        label = 'Online';
        color = 'bg-green-500/10 text-green-500';
        break;
      case 'limited':
        icon = <WifiIcon className="h-4 w-4" />;
        label = 'Limited Connection';
        color = 'bg-yellow-500/10 text-yellow-500';
        break;
      case 'offline':
        icon = <WifiOffIcon className="h-4 w-4" />;
        label = 'Offline';
        color = 'bg-red-500/10 text-red-500';
        break;
    }
    
    return (
      <Badge variant="outline" className={`${color} flex items-center gap-1`}>
        {icon}
        <span>{label}</span>
      </Badge>
    );
  };

  // Get sync status indicator
  const getSyncStatusIndicator = () => {
    let icon, label, color;
    
    switch (syncStatus) {
      case 'idle':
        icon = <CloudIcon className="h-4 w-4" />;
        label = 'Sync Ready';
        color = 'text-muted-foreground';
        break;
      case 'syncing':
        icon = <RefreshCwIcon className="h-4 w-4 animate-spin" />;
        label = 'Syncing...';
        color = 'text-blue-500';
        break;
      case 'error':
        icon = <AlertTriangleIcon className="h-4 w-4" />;
        label = 'Sync Error';
        color = 'text-red-500';
        break;
    }
    
    return (
      <div className={`flex items-center gap-1 ${color}`}>
        {icon}
        <span className="text-sm">{label}</span>
      </div>
    );
  };

  if (loading) {
    return (
      <div className="p-4 space-y-6">
        <div className="flex items-center justify-between">
          <Skeleton className="h-8 w-60" />
          <Skeleton className="h-6 w-24" />
        </div>
        <Skeleton className="h-12 w-full" />
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {[...Array(4)].map((_, i) => (
            <Card key={i}>
              <CardHeader>
                <Skeleton className="h-6 w-3/4" />
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

  return (
    <div className="p-4 space-y-4" style={{ fontFamily: platformUI.fontFamily }}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h1 className="text-2xl font-bold">Offline Settings</h1>
          {form.getValues('enabled') && getNetworkStatusIndicator()}
        </div>
        
        {form.getValues('enabled') && (
          <div className="flex items-center gap-4">
            {getSyncStatusIndicator()}
            
            <Button 
              variant="outline" 
              size="sm" 
              onClick={triggerSync}
              disabled={syncStatus === 'syncing' || networkStatus === 'offline'}
            >
              <RefreshCwIcon className="h-4 w-4 mr-2" />
              Sync Now
            </Button>
            
            {lastSyncTime && (
              <span className="text-xs text-muted-foreground">
                Last synced: {lastSyncTime.toLocaleString()}
              </span>
            )}
          </div>
        )}
      </div>
      
      <Form {...form}>
        <form onSubmit={form.handleSubmit(saveSettings)}>
          <Tabs defaultValue={activeTab} onValueChange={setActiveTab} className="space-y-4">
            <TabsList className={platformUI.borderRadius}>
              <TabsTrigger value="general">General</TabsTrigger>
              <TabsTrigger value="sync">Synchronization</TabsTrigger>
              <TabsTrigger value="models">Local Models</TabsTrigger>
              <TabsTrigger value="checkpoints">Checkpoints</TabsTrigger>
              <TabsTrigger value="advanced">Advanced</TabsTrigger>
            </TabsList>
            
            {/* General Tab */}
            <TabsContent value="general" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Offline Mode</CardTitle>
                  <CardDescription>
                    Configure how the application behaves when internet connectivity is limited or unavailable
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                  <FormField
                    control={form.control}
                    name="enabled"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">Enable Offline Mode</FormLabel>
                          <FormDescription>
                            Allow the application to function without an internet connection
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                  
                  {form.watch('enabled') && (
                    <>
                      <FormField
                        control={form.control}
                        name="autoDetectConnectivity"
                        render={({ field }) => (
                          <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                            <div className="space-y-0.5">
                              <FormLabel className="text-base">Auto-detect Network Status</FormLabel>
                              <FormDescription>
                                Automatically detect and respond to changes in network connectivity
                              </FormDescription>
                            </div>
                            <FormControl>
                              <Switch
                                checked={field.value}
                                onCheckedChange={field.onChange}
                              />
                            </FormControl>
                          </FormItem>
                        )}
                      />
                      
                      <FormField
                        control={form.control}
                        name="autoEnableOfflineOnPoorConnection"
                        render={({ field }) => (
                          <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                            <div className="space-y-0.5">
                              <FormLabel className="text-base">Auto-enable Offline Mode</FormLabel>
                              <FormDescription>
                                Automatically enable offline mode when connection is poor or limited
                              </FormDescription>
                            </div>
                            <FormControl>
                              <Switch
                                checked={field.value}
                                onCheckedChange={field.onChange}
                                disabled={!form.watch('autoDetectConnectivity')}
                              />
                            </FormControl>
                          </FormItem>
                        )}
                      />
                      
                      <FormField
                        control={form.control}
                        name="offlineIndicator"
                        render={({ field }) => (
                          <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                            <div className="space-y-0.5">
                              <FormLabel className="text-base">Show Offline Indicator</FormLabel>
                              <FormDescription>
                                Display an indicator when the application is operating in offline mode
                              </FormDescription>
                            </div>
                            <FormControl>
                              <Switch
                                checked={field.value}
                                onCheckedChange={field.onChange}
                              />
                            </FormControl>
                          </FormItem>
                        )}
                      />
                    </>
                  )}
                </CardContent>
                
                {form.watch('enabled') && networkStatus === 'offline' && (
                  <CardFooter>
                    <Alert variant="warning">
                      <AlertTriangleIcon className="h-4 w-4" />
                      <AlertTitle>Currently Offline</AlertTitle>
                      <AlertDescription>
                        You are currently offline. Some features may be limited, and changes will sync when a connection is available.
                      </AlertDescription>
                    </Alert>
                  </CardFooter>
                )}
              </Card>
              
              {form.watch('enabled') && (
                <Card>
                  <CardHeader>
                    <CardTitle>Local Model Support</CardTitle>
                    <CardDescription>
                      Configure the local LLM for use when offline
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    <FormField
                      control={form.control}
                      name="llmEnabled"
                      render={({ field }) => (
                        <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                          <div className="space-y-0.5">
                            <FormLabel className="text-base">Enable Local LLM</FormLabel>
                            <FormDescription>
                              Enable local language model for offline use
                            </FormDescription>
                          </div>
                          <FormControl>
                            <Switch
                              checked={field.value}
                              onCheckedChange={field.onChange}
                            />
                          </FormControl>
                        </FormItem>
                      )}
                    />
                    
                    {form.watch('llmEnabled') && (
                      <div className="space-y-4">
                        <FormField
                          control={form.control}
                          name="defaultModelId"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>Default Model</FormLabel>
                              <Select
                                onValueChange={field.onChange}
                                defaultValue={field.value}
                              >
                                <FormControl>
                                  <SelectTrigger className={platformUI.borderRadius}>
                                    <SelectValue placeholder="Select a model" />
                                  </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                  {models
                                    .filter(model => model.installed)
                                    .map((model) => (
                                      <SelectItem key={model.id} value={model.id}>
                                        {model.name} ({formatBytes(model.size)})
                                      </SelectItem>
                                    ))}
                                </SelectContent>
                              </Select>
                              <FormDescription>
                                Select the default model to use in offline mode
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        
                        <div className="mt-4">
                          <Button 
                            variant="outline" 
                            type="button" 
                            onClick={() => setActiveTab('models')}
                          >
                            Manage Models
                            <ChevronRightIcon className="ml-2 h-4 w-4" />
                          </Button>
                        </div>
                      </div>
                    )}
                  </CardContent>
                </Card>
              )}
            </TabsContent>
            
            {/* Synchronization Tab */}
            <TabsContent value="sync" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Synchronization Settings</CardTitle>
                  <CardDescription>
                    Configure how your data synchronizes between online and offline modes
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                  <FormField
                    control={form.control}
                    name="syncFrequency"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Sync Frequency</FormLabel>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                          disabled={!form.watch('enabled')}
                        >
                          <FormControl>
                            <SelectTrigger className={platformUI.borderRadius}>
                              <SelectValue placeholder="Select frequency" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="manual">Manual Only</SelectItem>
                            <SelectItem value="onConnection">When Connection Available</SelectItem>
                            <SelectItem value="hourly">Hourly</SelectItem>
                            <SelectItem value="daily">Daily</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormDescription>
                          How often to sync data when a connection is available
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <FormField
                      control={form.control}
                      name="syncMetadata"
                      render={({ field }) => (
                        <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                          <div className="space-y-0.5">
                            <FormLabel className="text-base">Sync Metadata</FormLabel>
                            <FormDescription>
                              Sync conversation metadata and settings
                            </FormDescription>
                          </div>
                          <FormControl>
                            <Switch
                              checked={field.value}
                              onCheckedChange={field.onChange}
                              disabled={!form.watch('enabled')}
                            />
                          </FormControl>
                        </FormItem>
                      )}
                    />
                    
                    <FormField
                      control={form.control}
                      name="syncContent"
                      render={({ field }) => (
                        <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                          <div className="space-y-0.5">
                            <FormLabel className="text-base">Sync Content</FormLabel>
                            <FormDescription>
                              Sync full conversation content
                            </FormDescription>
                          </div>
                          <FormControl>
                            <Switch
                              checked={field.value}
                              onCheckedChange={field.onChange}
                              disabled={!form.watch('enabled')}
                            />
                          </FormControl>
                        </FormItem>
                      )}
                    />
                  </div>
                  
                  <FormField
                    control={form.control}
                    name="syncOnMobileData"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">Sync on Mobile Data</FormLabel>
                          <FormDescription>
                            Allow synchronization when connected via mobile data
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                            disabled={!form.watch('enabled')}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                </CardContent>
                
                {form.watch('enabled') && (
                  <CardFooter className="flex justify-between">
                    {getSyncStatusIndicator()}
                    
                    <Button 
                      variant="outline" 
                      type="button" 
                      onClick={triggerSync}
                      disabled={syncStatus === 'syncing' || networkStatus === 'offline'}
                    >
                      <RefreshCwIcon className={`mr-2 h-4 w-4 ${syncStatus === 'syncing' ? 'animate-spin' : ''}`} />
                      Sync Now
                    </Button>
                  </CardFooter>
                )}
              </Card>
              
              {form.watch('enabled') && (
                <Card>
                  <CardHeader>
                    <CardTitle>Sync Status</CardTitle>
                    <CardDescription>
                      Current synchronization status and history
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div className="rounded-lg border p-3">
                          <div className="text-xs text-muted-foreground mb-1">Network Status</div>
                          <div className="font-medium">{getNetworkStatusIndicator()}</div>
                        </div>
                        
                        <div className="rounded-lg border p-3">
                          <div className="text-xs text-muted-foreground mb-1">Last Successful Sync</div>
                          <div className="font-medium">
                            {lastSyncTime ? lastSyncTime.toLocaleString() : 'Never'}
                          </div>
                        </div>
                        
                        <div className="rounded-lg border p-3">
                          <div className="text-xs text-muted-foreground mb-1">Sync Mode</div>
                          <div className="font-medium">
                            {form.watch('syncFrequency') === 'manual' && 'Manual Only'}
                            {form.watch('syncFrequency') === 'onConnection' && 'When Connected'}
                            {form.watch('syncFrequency') === 'hourly' && 'Hourly'}
                            {form.watch('syncFrequency') === 'daily' && 'Daily'}
                          </div>
                        </div>
                      </div>
                      
                      {syncStatus === 'error' && (
                        <Alert variant="destructive" className="mt-4">
                          <AlertTriangleIcon className="h-4 w-4" />
                          <AlertTitle>Sync Error</AlertTitle>
                          <AlertDescription>
                            Last synchronization attempt failed. Please try again or check network connection.
                          </AlertDescription>
                        </Alert>
                      )}
                    </div>
                  </CardContent>
                </Card>
              )}
            </TabsContent>
            
            {/* Models Tab */}
            <TabsContent value="models" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Local LLM Models</CardTitle>
                  <CardDescription>
                    Manage language models for offline use
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {form.watch('enabled') ? (
                    <>
                      <FormField
                        control={form.control}
                        name="llmEnabled"
                        render={({ field }) => (
                          <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                            <div className="space-y-0.5">
                              <FormLabel className="text-base">Enable Local LLM</FormLabel>
                              <FormDescription>
                                Use local language models when offline
                              </FormDescription>
                            </div>
                            <FormControl>
                              <Switch
                                checked={field.value}
                                onCheckedChange={field.onChange}
                              />
                            </FormControl>
                          </FormItem>
                        )}
                      />
                      
                      {form.watch('llmEnabled') && (
                        <div className="space-y-4">
                          <div className="flex items-center justify-between">
                            <h3 className="text-sm font-medium">Available Models</h3>
                            <div className="text-xs text-muted-foreground">
                              Storage: {formatBytes(diskSpace.used)} / {formatBytes(diskSpace.total)}
                            </div>
                          </div>
                          
                          <div className="rounded-lg border">
                            <ScrollArea className="h-[320px]">
                              <div className="p-0">
                                {models.length === 0 ? (
                                  <div className="flex flex-col items-center justify-center h-40 text-muted-foreground">
                                    <ServerIcon className="h-10 w-10 mb-2" />
                                    <p>No models available</p>
                                  </div>
                                ) : (
                                  <div className="divide-y">
                                    {models.map((model) => (
                                      <div 
                                        key={model.id}
                                        className="p-3 hover:bg-muted/50 flex items-center justify-between"
                                      >
                                        <div className="flex items-center gap-3">
                                          <div>
                                            {model.installed ? (
                                              <Badge variant="outline" className="bg-green-500/10 text-green-500">
                                                Installed
                                              </Badge>
                                            ) : model.downloading ? (
                                              <Badge variant="outline" className="bg-blue-500/10 text-blue-500">
                                                Downloading
                                              </Badge>
                                            ) : (
                                              <Badge variant="outline" className="bg-muted text-muted-foreground">
                                                Available
                                              </Badge>
                                            )}
                                          </div>
                                          
                                          <div>
                                            <div className="font-medium flex items-center">
                                              {model.name}
                                              {model.id === form.watch('defaultModelId') && (
                                                <Badge 
                                                  variant="outline" 
                                                  className="bg-primary/10 text-primary ml-2"
                                                >
                                                  Default
                                                </Badge>
                                              )}
                                            </div>
                                            <div className="text-xs text-muted-foreground">
                                              {formatBytes(model.size)} {!model.supported && " • Not optimized for this platform"}
                                            </div>
                                            
                                            {model.downloading && (
                                              <div className="mt-2 w-full max-w-xs">
                                                <Progress value={model.downloadProgress} className="h-1" />
                                                <div className="text-xs text-muted-foreground mt-1">
                                                  {Math.round(model.downloadProgress || 0)}% • Downloading...
                                                </div>
                                              </div>
                                            )}
                                          </div>
                                        </div>
                                        
                                        <div className="flex items-center gap-2">
                                          {model.installed ? (
                                            <>
                                              <TooltipProvider>
                                                <Tooltip>
                                                  <TooltipTrigger asChild>
                                                    <Button 
                                                      variant="ghost" 
                                                      size="sm"
                                                      onClick={() => setSelectedModelForDetails(model)}
                                                    >
                                                      <InfoIcon className="h-4 w-4" />
                                                    </Button>
                                                  </TooltipTrigger>
                                                  <TooltipContent>
                                                    <p>View model details</p>
                                                  </TooltipContent>
                                                </Tooltip>
                                              </TooltipProvider>
                                              
                                              <TooltipProvider>
                                                <Tooltip>
                                                  <TooltipTrigger asChild>
                                                    <Button 
                                                      variant="ghost" 
                                                      size="sm"
                                                      onClick={() => setDefaultModel(model.id)}
                                                      disabled={model.id === form.watch('defaultModelId')}
                                                    >
                                                      <CheckIcon className="h-4 w-4" />
                                                    </Button>
                                                  </TooltipTrigger>
                                                  <TooltipContent>
                                                    <p>Set as default model</p>
                                                  </TooltipContent>
                                                </Tooltip>
                                              </TooltipProvider>
                                              
                                              <TooltipProvider>
                                                <Tooltip>
                                                  <TooltipTrigger asChild>
                                                    <Button 
                                                      variant="ghost" 
                                                      size="sm"
                                                      onClick={() => deleteModel(model.id)}
                                                      disabled={model.id === form.watch('defaultModelId')}
                                                    >
                                                      <TrashIcon className="h-4 w-4" />
                                                    </Button>
                                                  </TooltipTrigger>
                                                  <TooltipContent>
                                                    <p>Delete model</p>
                                                  </TooltipContent>
                                                </Tooltip>
                                              </TooltipProvider>
                                            </>
                                          ) : model.downloading ? (
                                            <Button 
                                              variant="ghost" 
                                              size="sm" 
                                              disabled
                                            >
                                              <RefreshCwIcon className="h-4 w-4 animate-spin" />
                                            </Button>
                                          ) : (
                                            <Button 
                                              variant="outline" 
                                              size="sm"
                                              onClick={() => downloadModel(model.id)}
                                              disabled={diskSpace.available < model.size}
                                            >
                                              <DownloadIcon className="h-4 w-4 mr-1" />
                                              Download
                                            </Button>
                                          )}
                                        </div>
                                      </div>
                                    ))}
                                  </div>
                                )}
                              </div>
                            </ScrollArea>
                          </div>
                          
                          <div className="text-xs text-muted-foreground">
                            <p>
                              <CpuIcon className="h-3 w-3 inline mr-1" />
                              Models are stored in: <code>{platformUI.modelLocation}</code>
                            </p>
                            <p className="mt-1">
                              <HardDriveIcon className="h-3 w-3 inline mr-1" />
                              Available space: {formatBytes(diskSpace.available)}
                            </p>
                          </div>
                          
                          {diskSpace.available < 500 * 1024 * 1024 && (
                            <Alert variant="warning">
                              <AlertTriangleIcon className="h-4 w-4" />
                              <AlertTitle>Low Storage</AlertTitle>
                              <AlertDescription>
                                You have less than 500MB of available storage. Some models may not download successfully.
                              </AlertDescription>
                            </Alert>
                          )}
                        </div>
                      )}
                    </>
                  ) : (
                    <div className="flex flex-col items-center justify-center py-10 text-center">
                      <CloudOffIcon className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-medium">Offline Mode is Disabled</h3>
                      <p className="text-muted-foreground max-w-md mt-2">
                        Enable offline mode in the General tab to manage local LLM models.
                      </p>
                      <Button 
                        variant="outline" 
                        className="mt-4"
                        onClick={() => {
                          form.setValue('enabled', true);
                          setActiveTab('general');
                        }}
                      >
                        Enable Offline Mode
                      </Button>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
            
            {/* Checkpoints Tab */}
            <TabsContent value="checkpoints" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Checkpointing</CardTitle>
                  <CardDescription>
                    Configure how conversations are saved for offline use
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                  {form.watch('enabled') ? (
                    <>
                      <FormField
                        control={form.control}
                        name="checkpointFrequency"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>Checkpoint Frequency</FormLabel>
                            <Select
                              onValueChange={field.onChange}
                              defaultValue={field.value}
                            >
                              <FormControl>
                                <SelectTrigger className={platformUI.borderRadius}>
                                  <SelectValue placeholder="Select frequency" />
                                </SelectTrigger>
                              </FormControl>
                              <SelectContent>
                                <SelectItem value="message">After Every Message</SelectItem>
                                <SelectItem value="conversation">After Each Conversation</SelectItem>
                                <SelectItem value="manual">Manual Only</SelectItem>
                                <SelectItem value="time">Timed Interval</SelectItem>
                              </SelectContent>
                            </Select>
                            <FormDescription>
                              When to save checkpoints of your conversations
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      
                      {form.watch('checkpointFrequency') === 'time' && (
                        <FormField
                          control={form.control}
                          name="checkpointTimeInterval"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>Checkpoint Interval (minutes)</FormLabel>
                              <div className="flex items-center gap-4">
                                <FormControl>
                                  <Slider
                                    min={1}
                                    max={60}
                                    step={1}
                                    value={[field.value]}
                                    onValueChange={(value) => field.onChange(value[0])}
                                  />
                                </FormControl>
                                <span className="w-12 text-center">{field.value}</span>
                              </div>
                              <FormDescription>
                                Create checkpoints every {field.value} minute{field.value !== 1 ? 's' : ''}
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      )}
                      
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <FormField
                          control={form.control}
                          name="checkpointMaxCount"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>Maximum Checkpoints</FormLabel>
                              <FormControl>
                                <Input
                                  type="number"
                                  min={1}
                                  max={1000}
                                  {...field}
                                  onChange={(e) => field.onChange(parseInt(e.target.value) || 1)}
                                  className={platformUI.borderRadius}
                                />
                              </FormControl>
                              <FormDescription>
                                Maximum number of checkpoints to keep
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        
                        <FormField
                          control={form.control}
                          name="checkpointMaxSize"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>Max Checkpoint Size (MB)</FormLabel>
                              <FormControl>
                                <Input
                                  type="number"
                                  min={1}
                                  max={10000}
                                  {...field}
                                  onChange={(e) => field.onChange(parseInt(e.target.value) || 1)}
                                  className={platformUI.borderRadius}
                                />
                              </FormControl>
                              <FormDescription>
                                Maximum total size for all checkpoints (MB)
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </div>
                      
                      <FormField
                        control={form.control}
                        name="removeOldCheckpoints"
                        render={({ field }) => (
                          <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                            <div className="space-y-0.5">
                              <FormLabel className="text-base">Auto-remove Old Checkpoints</FormLabel>
                              <FormDescription>
                                Automatically remove checkpoints older than a specified time
                              </FormDescription>
                            </div>
                            <FormControl>
                              <Switch
                                checked={field.value}
                                onCheckedChange={field.onChange}
                              />
                            </FormControl>
                          </FormItem>
                        )}
                      />
                      
                      {form.watch('removeOldCheckpoints') && (
                        <FormField
                          control={form.control}
                          name="removeOldCheckpointsAfterDays"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>Remove After (days)</FormLabel>
                              <div className="flex items-center gap-4">
                                <FormControl>
                                  <Slider
                                    min={1}
                                    max={365}
                                    step={1}
                                    value={[field.value]}
                                    onValueChange={(value) => field.onChange(value[0])}
                                  />
                                </FormControl>
                                <span className="w-12 text-center">{field.value}</span>
                              </div>
                              <FormDescription>
                                Remove checkpoints older than {field.value} day{field.value !== 1 ? 's' : ''}
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      )}
                    </>
                  ) : (
                    <div className="flex flex-col items-center justify-center py-10 text-center">
                      <CloudOffIcon className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-medium">Offline Mode is Disabled</h3>
                      <p className="text-muted-foreground max-w-md mt-2">
                        Enable offline mode in the General tab to configure checkpointing behavior.
                      </p>
                      <Button 
                        variant="outline" 
                        className="mt-4"
                        onClick={() => {
                          form.setValue('enabled', true);
                          setActiveTab('general');
                        }}
                      >
                        Enable Offline Mode
                      </Button>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
            
            {/* Advanced Tab */}
            <TabsContent value="advanced" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Advanced Settings</CardTitle>
                  <CardDescription>
                    Additional configuration options for advanced users
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                  {form.watch('enabled') ? (
                    <>
                      <div className="rounded-lg border p-4">
                        <h3 className="text-sm font-medium mb-2">Platform Information</h3>
                        <div className="grid grid-cols-2 gap-2 text-sm">
                          <div className="text-muted-foreground">Platform:</div>
                          <div className="font-medium">{platform}</div>
                          
                          <div className="text-muted-foreground">Storage Location:</div>
                          <div className="font-medium"><code>{platformUI.modelLocation}</code></div>
                          
                          <div className="text-muted-foreground">Available Space:</div>
                          <div className="font-medium">{formatBytes(diskSpace.available)}</div>
                        </div>
                      </div>
                      
                      <div className="space-y-4">
                        <FormField
                          control={form.control}
                          name="syncFrequency"
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>Resource Management</FormLabel>
                              <ToggleGroup 
                                type="single" 
                                value={field.value}
                                onValueChange={(value) => {
                                  if (value) field.onChange(value);
                                }}
                                className="justify-start"
                              >
                                <ToggleGroupItem value="manual" className={platformUI.borderRadius}>
                                  Conservative
                                </ToggleGroupItem>
                                <ToggleGroupItem value="onConnection" className={platformUI.borderRadius}>
                                  Balanced
                                </ToggleGroupItem>
                                <ToggleGroupItem value="hourly" className={platformUI.borderRadius}>
                                  Aggressive
                                </ToggleGroupItem>
                              </ToggleGroup>
                              <FormDescription>
                                Control how aggressively the application uses system resources for offline functions
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </div>
                      
                      <div className="space-y-4">
                        <h3 className="text-sm font-medium">Diagnostics</h3>
                        <Button 
                          variant="outline" 
                          type="button"
                          className="mr-2"
                          onClick={async () => {
                            await invoke('test_network_connectivity');
                          }}
                        >
                          <GlobeIcon className="mr-2 h-4 w-4" />
                          Test Network Connectivity
                        </Button>
                        <Button 
                          variant="outline" 
                          type="button"
                          onClick={async () => {
                            await invoke('test_file_system_access');
                          }}
                        >
                          <HardDriveIcon className="mr-2 h-4 w-4" />
                          Test File System Access
                        </Button>
                      </div>
                      
                      <div className="space-y-2">
                        <h3 className="text-sm font-medium">Reset Options</h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <Button 
                            variant="outline" 
                            type="button"
                            onClick={async () => {
                              if (confirm('Are you sure you want to clear all checkpoints?')) {
                                await invoke('clear_all_checkpoints');
                              }
                            }}
                          >
                            Clear All Checkpoints
                          </Button>
                          <Button 
                            variant="outline" 
                            type="button"
                            onClick={async () => {
                              if (confirm('Are you sure you want to reset all offline settings?')) {
                                await invoke('reset_offline_settings');
                                await loadSettings();
                              }
                            }}
                          >
                            Reset All Settings
                          </Button>
                        </div>
                      </div>
                    </>
                  ) : (
                    <div className="flex flex-col items-center justify-center py-10 text-center">
                      <SettingsIcon className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-medium">Offline Mode is Disabled</h3>
                      <p className="text-muted-foreground max-w-md mt-2">
                        Enable offline mode in the General tab to access advanced settings.
                      </p>
                      <Button 
                        variant="outline" 
                        className="mt-4"
                        onClick={() => {
                          form.setValue('enabled', true);
                          setActiveTab('general');
                        }}
                      >
                        Enable Offline Mode
                      </Button>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
          
          {saveError && (
            <Alert variant="destructive" className="mt-4">
              <AlertTriangleIcon className="h-4 w-4" />
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{saveError}</AlertDescription>
            </Alert>
          )}
          
          <div className="flex justify-end space-x-4 mt-6">
            <Button 
              variant="outline" 
              type="button" 
              onClick={() => form.reset()}
              disabled={saving}
            >
              Reset
            </Button>
            <Button 
              type="submit" 
              disabled={saving || !form.formState.isDirty}
              style={{ backgroundColor: platformUI.accentColor }}
            >
              {saving ? (
                <>
                  <RefreshCwIcon className="mr-2 h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : 'Save Changes'}
            </Button>
          </div>
        </form>
      </Form>
      
      {/* Model Details Dialog */}
      {selectedModelForDetails && (
        <Dialog 
          open={!!selectedModelForDetails} 
          onOpenChange={(open) => !open && setSelectedModelForDetails(null)}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{selectedModelForDetails.name}</DialogTitle>
              <DialogDescription>
                Model details and information
              </DialogDescription>
            </DialogHeader>
            
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div className="text-muted-foreground">Size:</div>
                <div className="font-medium">{formatBytes(selectedModelForDetails.size)}</div>
                
                <div className="text-muted-foreground">Status:</div>
                <div className="font-medium">
                  {selectedModelForDetails.installed ? (
                    <Badge variant="outline" className="bg-green-500/10 text-green-500">
                      Installed
                    </Badge>
                  ) : (
                    <Badge variant="outline" className="bg-muted text-muted-foreground">
                      Not Installed
                    </Badge>
                  )}
                </div>
                
                <div className="text-muted-foreground">Default Model:</div>
                <div className="font-medium">
                  {selectedModelForDetails.id === form.watch('defaultModelId') ? 'Yes' : 'No'}
                </div>
                
                <div className="text-muted-foreground">Platform Optimized:</div>
                <div className="font-medium">
                  {selectedModelForDetails.supported ? 'Yes' : 'No'}
                </div>
                
                <div className="text-muted-foreground">Location:</div>
                <div className="font-medium">
                  <code>{platformUI.modelLocation}</code>
                </div>
              </div>
              
              <Separator />
              
              <div className="space-y-2">
                <h4 className="text-sm font-medium">Performance</h4>
                <p className="text-sm text-muted-foreground">
                  This model {selectedModelForDetails.supported ? 'is' : 'is not'} optimized for your current platform ({platform}).
                  {!selectedModelForDetails.supported && " Performance may be reduced compared to an optimized model."}
                </p>
                
                {selectedModelForDetails.id === form.watch('defaultModelId') ? (
                  <Badge variant="outline" className="bg-primary/10 text-primary">
                    Current Default Model
                  </Badge>
                ) : (
                  <Button 
                    variant="outline" 
                    size="sm"
                    onClick={() => {
                      setDefaultModel(selectedModelForDetails.id);
                      setSelectedModelForDetails(null);
                    }}
                  >
                    Set as Default
                  </Button>
                )}
              </div>
            </div>
            
            <DialogFooter>
              {selectedModelForDetails.installed && selectedModelForDetails.id !== form.watch('defaultModelId') && (
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => {
                    deleteModel(selectedModelForDetails.id);
                    setSelectedModelForDetails(null);
                  }}
                >
                  <TrashIcon className="h-4 w-4 mr-2" />
                  Delete Model
                </Button>
              )}
              <Button 
                onClick={() => setSelectedModelForDetails(null)}
              >
                Close
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
};

export default OfflineSettings;
