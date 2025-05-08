import React, { Suspense } from 'react';
import { LoadState } from './Shell';
import CommandPalette from './CommandPalette';
import useCommandPalette from '../hooks/useCommandPalette';
import './AppShell.css';

const LazyHeader = React.lazy(() => import('../lazy/Header'));
const LazySidebar = React.lazy(() => import('../lazy/Sidebar'));
const LazyChat = React.lazy(() => import('../lazy/Chat'));
const LazySettings = React.lazy(() => import('../lazy/Settings'));

interface AppShellProps {
  loadState: LoadState;
}

const AppShell: React.FC<AppShellProps> = ({ loadState }) => {
  const [currentView, setCurrentView] = React.useState<string>('chat');
  const { isOpen, commands, close } = useCommandPalette([
    {
      id: 'new-chat',
      name: 'New Chat',
      description: 'Start a new conversation',
      shortcut: 'Ctrl+N',
      action: () => console.log('New chat'),
    },
    {
      id: 'open-settings',
      name: 'Open Settings',
      description: 'Configure application settings',
      action: () => setCurrentView('settings'),
    },
    {
      id: 'switch-to-chat',
      name: 'Switch to Chat',
      description: 'Return to the chat interface',
      action: () => setCurrentView('chat'),
    },
    {
      id: 'toggle-theme',
      name: 'Toggle Dark Mode',
      description: 'Switch between light and dark mode',
      shortcut: 'Ctrl+T',
      action: () => {
        const currentTheme = document.documentElement.getAttribute('data-theme');
        document.documentElement.setAttribute(
          'data-theme', 
          currentTheme === 'dark' ? 'light' : 'dark'
        );
      },
    },
  ]);

  if (loadState !== LoadState.FullyLoaded) {
    return null; // Shell component will handle display during loading
  }

  return (
    <div className="app-shell">
      <Suspense fallback={<div className="loading-component">Loading header...</div>}>
        <LazyHeader 
          onViewChange={setCurrentView} 
          currentView={currentView} 
        />
      </Suspense>
      
      <div className="app-shell-content">
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
      
      <CommandPalette 
        isOpen={isOpen} 
        onClose={close} 
        commands={commands} 
      />
    </div>
  );
};

export default AppShell;
