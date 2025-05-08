import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './Shell.css';

// Types for loading state from the backend
export enum LoadState {
  NotStarted = 'NotStarted',
  ShellLoading = 'ShellLoading',
  ShellReady = 'ShellReady',
  CoreServicesLoading = 'CoreServicesLoading',
  CoreServicesReady = 'CoreServicesReady',
  SecondaryLoading = 'SecondaryLoading',
  FullyLoaded = 'FullyLoaded',
  Error = 'Error',
}

// Type for app info
interface AppInfo {
  name: string;
  version: string;
  platform: string;
}

// Shell component - designed to load in <500ms
const Shell: React.FC = () => {
  const [loadState, setLoadState] = useState<LoadState>(LoadState.ShellLoading);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [enabledFeatures, setEnabledFeatures] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loadProgress, setLoadProgress] = useState<number>(0);

  // Set up loading state listener
  useEffect(() => {
    const loadApp = async () => {
      try {
        // Load basic app info asap
        const info = await invoke<AppInfo>('get_app_info');
        setAppInfo(info);

        // Get enabled features
        const features = await invoke<string[]>('get_enabled_features');
        setEnabledFeatures(features);

        // Update progress based on loading state
        // In a real app, we would subscribe to backend state updates
        setLoadState(LoadState.ShellReady);
        setLoadProgress(20);

        setTimeout(() => {
          setLoadState(LoadState.CoreServicesLoading);
          setLoadProgress(40);

          setTimeout(() => {
            setLoadState(LoadState.CoreServicesReady);
            setLoadProgress(60);

            setTimeout(() => {
              setLoadState(LoadState.SecondaryLoading);
              setLoadProgress(80);

              setTimeout(() => {
                setLoadState(LoadState.FullyLoaded);
                setLoadProgress(100);
              }, 300);
            }, 200);
          }, 150);
        }, 100);
      } catch (err) {
        console.error('Failed to initialize app:', err);
        setError(String(err));
        setLoadState(LoadState.Error);
      }
    };

    loadApp();
  }, []);

  // Show error state
  if (loadState === LoadState.Error) {
    return (
      <div className="shell shell-error">
        <div className="error-container">
          <h2>Error Starting Application</h2>
          <p>{error}</p>
          <button onClick={() => window.location.reload()}>
            Restart Application
          </button>
        </div>
      </div>
    );
  }

  // Show loading state
  if (loadState !== LoadState.FullyLoaded) {
    return (
      <div className="shell shell-loading">
        <div className="logo-container">
          <div className="app-icon">
            {/* Simple placeholder icon */}
            <div className="icon-inner"></div>
          </div>
          <h1>{appInfo?.name || 'Claude MCP'}</h1>
        </div>
        <div className="loading-indicator">
          <div className="progress-bar">
            <div 
              className="progress-fill"
              style={{ width: `${loadProgress}%` }}
            ></div>
          </div>
          <div className="loading-text">
            {loadStateToMessage(loadState)}
          </div>
        </div>
        <div className="version-info">
          {appInfo?.version && `v${appInfo.version}`}
        </div>
      </div>
    );
  }

  // When fully loaded, the main app will be loaded dynamically in App.tsx
  // This component serves as a lightweight shell only
  return null;
};

// Convert load state to user-friendly message
function loadStateToMessage(state: LoadState): string {
  switch (state) {
    case LoadState.ShellLoading:
      return 'Starting up...';
    case LoadState.ShellReady:
      return 'Initializing...';
    case LoadState.CoreServicesLoading:
      return 'Loading core services...';
    case LoadState.CoreServicesReady:
      return 'Preparing workspace...';
    case LoadState.SecondaryLoading:
      return 'Almost ready...';
    case LoadState.FullyLoaded:
      return 'Ready!';
    case LoadState.Error:
      return 'Error starting application';
    default:
      return 'Loading...';
  }
}

export default Shell;