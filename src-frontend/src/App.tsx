import React, { useState, useEffect } from 'react';
import Shell, { LoadState } from './components/Shell';
import AppShell from './components/AppShell';
import { ThemeProvider } from './theme/ThemeContext';
import { AnimationProvider } from './animation';
import { KeyboardProvider } from './keyboard';
import { AccessibilityProvider } from './accessibility';
import { TourProvider } from './tours';
import { HelpProvider } from './help';
import { ProgressiveDisclosureProvider } from './disclosure';
import { A11yButton } from './accessibility';
import './App.css';

// Import base styles and animation utilities
import './theme/variables.css';

// Main application component
function App() {
  const [loadState, setLoadState] = useState<LoadState>(LoadState.ShellLoading);
  
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
  
  return (
    <ThemeProvider>
      <AnimationProvider>
        <KeyboardProvider>
          <AccessibilityProvider>
            <TourProvider>
              <HelpProvider>
                <ProgressiveDisclosureProvider>
                  {loadState !== LoadState.FullyLoaded ? (
                    <Shell />
                  ) : (
                    <>
                      <AppShell loadState={loadState} />
                      <A11yButton />
                    </>
                  )}
                </ProgressiveDisclosureProvider>
              </HelpProvider>
            </TourProvider>
          </AccessibilityProvider>
        </KeyboardProvider>
      </AnimationProvider>
    </ThemeProvider>
  );
}

// Helper for simulating loading delays
const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export default App;