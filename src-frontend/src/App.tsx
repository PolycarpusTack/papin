import React, { Suspense, useState, useEffect } from 'react';
import Shell, { LoadState } from './components/Shell';
import './App.css';

// Lazy-loaded components
const LazyChat = React.lazy(() => import('./lazy/Chat'));
const LazySettings = React.lazy(() => import('./lazy/Settings'));
const LazySidebar = React.lazy(() => import('./lazy/Sidebar'));
const LazyHeader = React.lazy(() => import('./lazy/Header'));

// Helper for simulating loading delays
const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Main application component
function App() {
  const [loadState, setLoadState] = useState<LoadState>(LoadState.ShellLoading);
  const [currentView, setCurrentView] = useState<string>('chat');
  
  useEffect(() => {
    // Simulate load state progression from the backend
    // In a real app, this would come from backend events
    const progressLoadState = async () => {
      // Wait for shell to be ready
      await sleep(100);
      setLoadState(LoadState.ShellReady);
      
      // Core services loading
      await sleep(200);
      setLoadState(LoadState.CoreServicesLoading);
      
      // Core services ready
      await sleep(150);
      setLoadState(LoadState.CoreServicesReady);
      
      // Secondary loading
      await sleep(200);
      setLoadState(LoadState.SecondaryLoading);
      
      // Everything loaded
      await sleep(100);
      setLoadState(LoadState.FullyLoaded);
    };
    
    progressLoadState();
  }, []);
  
  // Render loading shell
  if (loadState !== LoadState.FullyLoaded) {
    return <Shell />;
  }
  
  // Main application UI, loaded lazily
  return (
    <div className="app">
      <Suspense fallback={<div className="loading-component">Loading header...</div>}>
        <LazyHeader 
          onViewChange={setCurrentView} 
          currentView={currentView} 
        />
      </Suspense>
      
      <div className="app-content">
        <Suspense fallback={<div className="loading-component">Loading sidebar...</div>}>
          <LazySidebar currentView={currentView} />
        </Suspense>
        
        <main className="main-content">
          {currentView === 'chat' && (
            <Suspense fallback={<div className="loading-component">Loading chat...</div>}>
              <LazyChat />
            </Suspense>
          )}
          
          {currentView === 'settings' && (
            <Suspense fallback={<div className="loading-component">Loading settings...</div>}>
              <LazySettings />
            </Suspense>
          )}
        </main>
      </div>
    </div>
  );
}

export default App;