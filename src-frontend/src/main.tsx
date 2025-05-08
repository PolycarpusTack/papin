import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// Performance measurement for bootstrap time
const startTime = performance.now();

// Log startup metrics
document.addEventListener('DOMContentLoaded', () => {
  const loadTime = performance.now() - startTime;
  console.log(`DOM loaded in ${loadTime.toFixed(2)}ms`);
});

// Initial render must be as fast as possible (<500ms goal)
ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// Log total render time
window.addEventListener('load', () => {
  const totalTime = performance.now() - startTime;
  console.log(`Total load time: ${totalTime.toFixed(2)}ms`);
  
  // Send metrics to backend (would be implemented in real app)
  // invoke('log_performance', { metric: 'initial_load', value: totalTime });
});