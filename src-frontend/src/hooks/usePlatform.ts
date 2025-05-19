import { useEffect, useState } from 'react';
// Try to import Tauri API, but provide a fallback for dev mode
let invoke: (cmd: string, args?: any) => Promise<any>;
try {
  const tauriApi = require('@tauri-apps/api/tauri');
  invoke = tauriApi.invoke;
} catch (e) {
  // Mock for development mode without Tauri
  console.log('Running in development mode without Tauri');
  invoke = async (cmd: string, args?: any) => {
    console.log(`Mock invoke: ${cmd}`, args);
    if (cmd === 'get_hardware_capabilities') {
      return {
        cpu_cores: 4,
        logical_cores: 8,
        total_memory: 16000000000,
        gpu_info: {
          name: 'Mock GPU',
          vendor: 'Mock Vendor',
          memory: 4000000000,
        },
        platform: 'windows',
        supports_metal: false,
        supports_directml: true,
        supports_opencl: true,
        supports_cuda: false,
      };
    }
    return null;
  };
}

export type Platform = 'windows' | 'macos' | 'linux' | 'unknown';

export interface HardwareCapabilities {
  cpuCores: number;
  logicalCores: number;
  totalMemory: number;
  gpuInfo?: {
    name: string;
    vendor: string;
    memory?: number;
  };
  platform: Platform;
  supportsMetal: boolean;
  supportsDirectml: boolean;
  supportsOpencl: boolean;
  supportsCuda: boolean;
}

export const usePlatform = () => {
  const [platform, setPlatform] = useState<Platform>('unknown');
  const [capabilities, setCapabilities] = useState<HardwareCapabilities | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const detectPlatform = async () => {
      try {
        // Get hardware capabilities from Tauri command
        const hwCapabilities = await invoke<any>('get_hardware_capabilities');
        
        // Convert from snake_case to camelCase
        const formattedCapabilities: HardwareCapabilities = {
          cpuCores: hwCapabilities.cpu_cores,
          logicalCores: hwCapabilities.logical_cores,
          totalMemory: hwCapabilities.total_memory,
          gpuInfo: hwCapabilities.gpu_info ? {
            name: hwCapabilities.gpu_info.name,
            vendor: hwCapabilities.gpu_info.vendor,
            memory: hwCapabilities.gpu_info.memory,
          } : undefined,
          platform: hwCapabilities.platform as Platform,
          supportsMetal: hwCapabilities.supports_metal,
          supportsDirectml: hwCapabilities.supports_directml,
          supportsOpencl: hwCapabilities.supports_opencl,
          supportsCuda: hwCapabilities.supports_cuda,
        };
        
        setCapabilities(formattedCapabilities);
        setPlatform(formattedCapabilities.platform);
      } catch (err) {
        console.error('Failed to detect platform:', err);
        setError(err instanceof Error ? err.message : String(err));
        
        // Fallback platform detection using user agent
        const userAgent = navigator.userAgent.toLowerCase();
        
        if (userAgent.indexOf('win') !== -1) {
          setPlatform('windows');
        } else if (userAgent.indexOf('mac') !== -1) {
          setPlatform('macos');
        } else if (userAgent.indexOf('linux') !== -1) {
          setPlatform('linux');
        } else {
          setPlatform('unknown');
        }
      } finally {
        setIsLoading(false);
      }
    };

    detectPlatform();
  }, []);

  return {
    platform,
    capabilities,
    isLoading,
    error,
    isWindows: platform === 'windows',
    isMac: platform === 'macos',
    isLinux: platform === 'linux',
  };
};

export default usePlatform;