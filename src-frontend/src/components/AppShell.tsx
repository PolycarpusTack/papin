// AppShell.tsx
//
// Main layout component for the application with a Netflix-inspired dark theme

import React, { useState, useEffect } from 'react';
import { Outlet, NavLink, useLocation } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import styled, { css } from 'styled-components';
import { FeatureFlags } from '../contexts/FeatureFlagContext';
import HelpButton from './help/HelpButton';

// Import collaboration components if needed
import { 
  CollaborationProvider, 
  CollaborationPanel,
  CursorOverlay,
  SelectionOverlay,
  ConnectionStatus
} from './collaboration';

// Import styles
import './AppShell.css';

interface AppShellProps {
  featureFlags?: FeatureFlags;
  // Other props would go here in a real implementation
}

// Icon components
const DashboardIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z" />
  </svg>
);

const ModelsIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z" />
  </svg>
);

const SettingsIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z" />
  </svg>
);

const UserIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" />
  </svg>
);

const MenuIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z" />
  </svg>
);

const AppShell: React.FC<AppShellProps> = ({ featureFlags = {} }) => {
  const location = useLocation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState<boolean>(false);
  const [helpVisible, setHelpVisible] = useState<boolean>(false);
  const [collaborationEnabled, setCollaborationEnabled] = useState<boolean>(false);
  const [showCollaborationPanel, setShowCollaborationPanel] = useState<boolean>(false);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>(ConnectionStatus.Disconnected);
  
  // Initialize feature flags
  useEffect(() => {
    // Check if collaboration feature is enabled
    if (featureFlags && 'contains' in featureFlags && featureFlags.contains('COLLABORATION')) {
      setCollaborationEnabled(true);
    }
  }, [featureFlags]);
  
  // Initialize collaboration system if needed
  useEffect(() => {
    if (collaborationEnabled) {
      const initCollaboration = async () => {
        try {
          // Initialize the collaboration system with default config
          await invoke('init_collaboration_system', { config: null });
          
          // Get the initial connection status
          const status = await invoke<ConnectionStatus>('get_connection_status');
          setConnectionStatus(status);
        } catch (error) {
          console.error('Failed to initialize collaboration system:', error);
        }
      };
      
      initCollaboration();
    }
  }, [collaborationEnabled]);
  
  // Set up keyboard shortcut for help
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // F1 key for help
      if (e.key === 'F1') {
        e.preventDefault();
        setHelpVisible(true);
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);
  
  // Poll for connection status updates
  useEffect(() => {
    if (!collaborationEnabled) return;
    
    const interval = setInterval(async () => {
      try {
        const status = await invoke<ConnectionStatus>('get_connection_status');
        setConnectionStatus(status);
      } catch (error) {
        console.error('Failed to get connection status:', error);
      }
    }, 5000);
    
    return () => clearInterval(interval);
  }, [collaborationEnabled]);
  
  // Toggle mobile menu
  const toggleMobileMenu = () => {
    setMobileMenuOpen(!mobileMenuOpen);
  };

  // Close mobile menu when location changes
  useEffect(() => {
    setMobileMenuOpen(false);
  }, [location]);
  
  // Toggle collaboration panel if needed
  const toggleCollaborationPanel = () => {
    setShowCollaborationPanel(!showCollaborationPanel);
  };
  
  // Render collaboration status indicator if needed
  const renderCollaborationStatus = () => {
    if (!collaborationEnabled) return null;
    
    let color = '#9E9E9E';
    let title = 'Collaboration: Disconnected';
    
    switch (connectionStatus) {
      case ConnectionStatus.Connected:
        color = '#4CAF50';
        title = 'Collaboration: Connected';
        break;
      case ConnectionStatus.Connecting:
        color = '#FFC107';
        title = 'Collaboration: Connecting';
        break;
      case ConnectionStatus.Limited:
        color = '#FF9800';
        title = 'Collaboration: Limited Connectivity';
        break;
      case ConnectionStatus.Error:
        color = '#F44336';
        title = 'Collaboration: Error';
        break;
      default:
        break;
    }
    
    return (
      <CollabButton
        onClick={toggleCollaborationPanel}
        title={title}
      >
        <svg 
          width="20" 
          height="20" 
          viewBox="0 0 24 24" 
          fill="none" 
          stroke="currentColor" 
          strokeWidth="2" 
          strokeLinecap="round" 
          strokeLinejoin="round"
        >
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
          <circle cx="9" cy="7" r="4"></circle>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
        
        {/* Status indicator dot */}
        <StatusIndicator style={{ backgroundColor: color }} />
      </CollabButton>
    );
  };
  
  return (
    <ShellWrapper>
      {/* Mobile menu toggle */}
      <MobileMenuButton onClick={toggleMobileMenu} aria-label="Toggle menu">
        <MenuIcon />
      </MobileMenuButton>

      {/* Header */}
      <Header>
        <LogoContainer>
          <AppLogo>Papin</AppLogo>
          <AppSubtitle>MCP Client</AppSubtitle>
        </LogoContainer>

        {/* Desktop Navigation */}
        <DesktopNav>
          <NavItem to="/dashboard" $isActive={location.pathname === '/dashboard' || location.pathname === '/'}>
            <DashboardIcon />
            <NavText>Dashboard</NavText>
          </NavItem>
          <NavItem to="/models" $isActive={location.pathname.startsWith('/models')}>
            <ModelsIcon />
            <NavText>Models</NavText>
          </NavItem>
          <NavItem to="/settings" $isActive={location.pathname.startsWith('/settings')}>
            <SettingsIcon />
            <NavText>Settings</NavText>
          </NavItem>
        </DesktopNav>

        {/* Actions toolbar */}
        <ActionsToolbar>
          {collaborationEnabled && renderCollaborationStatus()}
          <ActionButton>
            <UserIcon />
          </ActionButton>
          <HelpButton 
            isOpen={helpVisible} 
            onOpenChange={(open) => setHelpVisible(open)} 
          />
        </ActionsToolbar>
      </Header>

      {/* Mobile Navigation (shown when toggled) */}
      <MobileNav $isOpen={mobileMenuOpen}>
        <NavItem to="/dashboard" $isActive={location.pathname === '/dashboard' || location.pathname === '/'}>
          <DashboardIcon />
          <NavText>Dashboard</NavText>
        </NavItem>
        <NavItem to="/models" $isActive={location.pathname.startsWith('/models')}>
          <ModelsIcon />
          <NavText>Models</NavText>
        </NavItem>
        <NavItem to="/settings" $isActive={location.pathname.startsWith('/settings')}>
          <SettingsIcon />
          <NavText>Settings</NavText>
        </NavItem>
      </MobileNav>
      
      {/* Main Content Area */}
      <Content>
        <Outlet />
        
        {/* Collaboration features if enabled */}
        {collaborationEnabled && showCollaborationPanel && (
          <CollaborationPanelWrapper>
            <CollaborationPanel conversationId="current-conversation" />
          </CollaborationPanelWrapper>
        )}
      </Content>
    </ShellWrapper>
  );
};

// Styled Components
const ShellWrapper = styled.div`
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background-color: #141414; /* Netflix dark background */
  color: #e5e5e5;
  font-family: 'Helvetica Neue', Arial, sans-serif;
`;

const Header = styled.header`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 68px;
  background-color: rgba(20, 20, 20, 0.9);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
`;

const LogoContainer = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
`;

const AppLogo = styled.div`
  font-size: 24px;
  font-weight: 700;
  color: #e50914; /* Netflix red */
`;

const AppSubtitle = styled.div`
  font-size: 14px;
  font-weight: 400;
  opacity: 0.7;
  
  @media (max-width: 768px) {
    display: none;
  }
`;

const DesktopNav = styled.nav`
  display: flex;
  gap: 24px;
  
  @media (max-width: 768px) {
    display: none;
  }
`;

const MobileNav = styled.nav<{ $isOpen: boolean }>`
  display: none;
  flex-direction: column;
  width: 100%;
  background-color: #1a1a1a;
  overflow: hidden;
  max-height: ${props => props.$isOpen ? '300px' : '0px'};
  transition: max-height 0.3s ease;
  
  @media (max-width: 768px) {
    display: flex;
  }
`;

const MobileMenuButton = styled.button`
  display: none;
  background: none;
  border: none;
  color: #e5e5e5;
  padding: 8px;
  cursor: pointer;
  position: absolute;
  left: 16px;
  top: 20px;
  z-index: 101;
  
  @media (max-width: 768px) {
    display: block;
  }
`;

const NavItem = styled(NavLink)<{ $isActive: boolean }>`
  display: flex;
  align-items: center;
  gap: 8px;
  text-decoration: none;
  color: ${props => props.$isActive ? '#ffffff' : '#b3b3b3'};
  font-weight: ${props => props.$isActive ? '600' : '400'};
  padding: 8px 12px;
  border-radius: 4px;
  transition: color 0.2s, background-color 0.2s;
  
  &:hover {
    color: #ffffff;
    background-color: rgba(255, 255, 255, 0.1);
  }
  
  @media (max-width: 768px) {
    padding: 16px;
    border-radius: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
`;

const NavText = styled.span`
  font-size: 16px;
`;

const ActionsToolbar = styled.div`
  display: flex;
  align-items: center;
  gap: 16px;
`;

const ActionButton = styled.button`
  background: none;
  border: none;
  color: #e5e5e5;
  padding: 8px;
  cursor: pointer;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s;
  
  &:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }
`;

const CollabButton = styled(ActionButton)`
  position: relative;
`;

const StatusIndicator = styled.div`
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
`;

const Content = styled.main`
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  position: relative;
  
  /* Netflix-inspired scrollbar */
  scrollbar-width: thin;
  scrollbar-color: #4d4d4d #141414;
  
  &::-webkit-scrollbar {
    width: 8px;
  }
  
  &::-webkit-scrollbar-track {
    background: #141414;
  }
  
  &::-webkit-scrollbar-thumb {
    background-color: #4d4d4d;
    border-radius: 4px;
  }
`;

const CollaborationPanelWrapper = styled.div`
  position: absolute;
  top: 0;
  right: 0;
  width: 320px;
  height: 100%;
  background-color: rgba(26, 26, 26, 0.9);
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  z-index: 10;
`;

export default AppShell;