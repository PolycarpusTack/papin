// src-frontend/src/Routes.tsx
// Application routing configuration

import React, { lazy, Suspense } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import styled from 'styled-components';

// Layout components
import AppShell from './components/AppShell';

// Lazy-loaded components
const ModelManagement = lazy(() => import('./components/llm/ModelManagement'));
const Dashboard = lazy(() => import('./components/dashboard/ResourceDashboard'));
const Settings = lazy(() => import('./lazy/Settings'));
const OfflineSettings = lazy(() => import('./components/offline/OfflineSettings'));

// Loading component
const LoadingFallback = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  width: 100%;
  background-color: #141414;
  color: white;
  font-family: 'Netflix Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  
  &::after {
    content: '';
    width: 40px;
    height: 40px;
    border: 4px solid rgba(255, 255, 255, 0.1);
    border-top: 4px solid #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
`;

/**
 * Main routing component for the application
 */
const AppRoutes: React.FC = () => {
  return (
    <Suspense fallback={<LoadingFallback />}>
      <Routes>
        <Route path="/" element={<AppShell />}>
          {/* Default redirect */}
          <Route index element={<Navigate to="/dashboard" replace />} />
          
          {/* Main routes */}
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="models" element={<ModelManagement />} />
          <Route path="settings" element={<Settings />} />
          <Route path="settings/offline" element={<OfflineSettings />} />
          
          {/* Fallback for unknown routes */}
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Route>
      </Routes>
    </Suspense>
  );
};

export default AppRoutes;