import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface FeatureFlag {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  rollout_strategy: any;
  dependencies: string[];
  created_at: number;
  updated_at: number;
  metadata: Record<string, string>;
}

interface FeatureFlagContextType {
  flags: FeatureFlag[];
  enabledFlags: FeatureFlag[];
  isFeatureEnabled: (flagId: string) => boolean;
  getFlag: (flagId: string) => FeatureFlag | undefined;
  refreshFlags: () => Promise<void>;
  toggleFlag: (flagId: string, enabled: boolean) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

const FeatureFlagContext = createContext<FeatureFlagContextType>({
  flags: [],
  enabledFlags: [],
  isFeatureEnabled: () => false,
  getFlag: () => undefined,
  refreshFlags: async () => {},
  toggleFlag: async () => {},
  isLoading: true,
  error: null,
});

export const useFeatureFlags = () => useContext(FeatureFlagContext);

// Feature flag IDs
export const FLAG_ADVANCED_TELEMETRY = 'advanced_telemetry';
export const FLAG_PERFORMANCE_DASHBOARD = 'performance_dashboard';
export const FLAG_DEBUG_LOGGING = 'debug_logging';
export const FLAG_RESOURCE_MONITORING = 'resource_monitoring';
export const FLAG_CRASH_REPORTING = 'crash_reporting';

export const FeatureFlagProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [flags, setFlags] = useState<FeatureFlag[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const fetchFlags = async () => {
    try {
      setIsLoading(true);
      const featureFlags = await invoke<FeatureFlag[]>('get_feature_flags');
      setFlags(featureFlags);
      setError(null);
    } catch (error: any) {
      console.error('Failed to fetch feature flags:', error);
      setError('Failed to fetch feature flags: ' + error.toString());
    } finally {
      setIsLoading(false);
    }
  };
  
  // Fetch flags on initial render
  useEffect(() => {
    fetchFlags();
    
    // Set up polling to refresh flags every 30 seconds
    const interval = setInterval(() => {
      fetchFlags();
    }, 30000);
    
    return () => clearInterval(interval);
  }, []);
  
  // Get enabled flags
  const enabledFlags = flags.filter(flag => flag.enabled);
  
  // Check if a feature is enabled
  const isFeatureEnabled = (flagId: string): boolean => {
    const flag = flags.find(f => f.id === flagId);
    return flag ? flag.enabled : false;
  };
  
  // Get flag by ID
  const getFlag = (flagId: string): FeatureFlag | undefined => {
    return flags.find(f => f.id === flagId);
  };
  
  // Toggle a feature flag
  const toggleFlag = async (flagId: string, enabled: boolean): Promise<void> => {
    try {
      await invoke('toggle_feature_flag', { flagId, enabled });
      
      // Update local state
      setFlags(flags.map(flag => 
        flag.id === flagId ? { ...flag, enabled } : flag
      ));
    } catch (error: any) {
      console.error('Failed to toggle feature flag:', error);
      throw new Error('Failed to toggle feature flag: ' + error.toString());
    }
  };
  
  return (
    <FeatureFlagContext.Provider
      value={{
        flags,
        enabledFlags,
        isFeatureEnabled,
        getFlag,
        refreshFlags: fetchFlags,
        toggleFlag,
        isLoading,
        error,
      }}
    >
      {children}
    </FeatureFlagContext.Provider>
  );
};

export default FeatureFlagContext;