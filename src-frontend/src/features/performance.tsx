import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Route } from 'react-router-dom';

// Import the components
import ResourceDashboard from '../components/dashboard/ResourceDashboard';
import OfflineSettings from '../components/offline/OfflineSettings';

// Routes configuration
export const performanceRoutes = [
  {
    path: '/dashboard/resources',
    name: 'Resource Dashboard',
    icon: 'activity',
    component: ResourceDashboard,
  },
  {
    path: '/settings/offline',
    name: 'Offline Settings',
    icon: 'hard-drive',
    component: OfflineSettings,
  },
];

// Add the routes to the application
export const PerformanceRoutes: React.FC = () => {
  return (
    <>
      {performanceRoutes.map((route) => (
        <Route key={route.path} path={route.path} element={<route.component />} />
      ))}
    </>
  );
};

// Generate sidebar menu items
export const performanceMenuItems = performanceRoutes.map((route) => ({
  to: route.path,
  name: route.name,
  icon: route.icon,
}));

// Initialize performance monitoring
export const initializePerformance = async () => {
  try {
    // Check if performance monitoring is supported
    const isSupported = await invoke('is_feature_supported', {
      feature: 'performance_monitoring',
    });

    if (isSupported) {
      console.log('Performance monitoring initialized');
      
      // Get hardware capabilities for logging
      const capabilities = await invoke('get_hardware_capabilities');
      console.log('Hardware capabilities:', capabilities);
      
      // Start resource monitoring
      await invoke('initialize_resource_monitoring');
    } else {
      console.warn('Performance monitoring not fully supported on this platform');
    }
  } catch (err) {
    console.error('Failed to initialize performance monitoring:', err);
  }
};

// Check for platform-specific features
export const checkPlatformFeatures = async () => {
  try {
    const platformFeatures = {
      offline_llm: await invoke('is_feature_supported', { feature: 'offline_llm' }),
      realtime_transcription: await invoke('is_feature_supported', { feature: 'realtime_transcription' }),
      advanced_ui: await invoke('is_feature_supported', { feature: 'advanced_ui' }),
    };
    
    console.log('Platform features:', platformFeatures);
    return platformFeatures;
  } catch (err) {
    console.error('Failed to check platform features:', err);
    return {
      offline_llm: false,
      realtime_transcription: false,
      advanced_ui: true,
    };
  }
};

// Export all
export default {
  routes: PerformanceRoutes,
  menuItems: performanceMenuItems,
  initialize: initializePerformance,
  checkFeatures: checkPlatformFeatures,
};
