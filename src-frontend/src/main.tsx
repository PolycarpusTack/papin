// src-frontend/src/main.tsx
//
// React entry point

import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from './App';
import './styles/global.css';

// Determine the root element
const rootElement = document.getElementById('root') as HTMLElement;

// Create the root and render the app
ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);