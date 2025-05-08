import React, { useState, useEffect } from 'react';

interface FeatureDiscoveryProps {
  message: string;
  showCount?: number;
  localStorageKey?: string;
  children: React.ReactNode;
}

const FeatureDiscovery: React.FC<FeatureDiscoveryProps> = ({
  message,
  showCount = 3,
  localStorageKey,
  children,
}) => {
  const [showTooltip, setShowTooltip] = useState(false);
  
  useEffect(() => {
    // If a localStorage key is provided, use it to check whether to show the tooltip
    if (localStorageKey) {
      const viewCount = parseInt(localStorage.getItem(localStorageKey) || '0', 10);
      
      if (viewCount < showCount) {
        setShowTooltip(true);
        localStorage.setItem(localStorageKey, String(viewCount + 1));
      }
    } else {
      // If no key is provided, always show the tooltip
      setShowTooltip(true);
    }
    
    // Hide the tooltip after 5 seconds
    const timer = setTimeout(() => {
      setShowTooltip(false);
    }, 5000);
    
    return () => {
      clearTimeout(timer);
    };
  }, [localStorageKey, showCount]);
  
  return (
    <div className="feature-discovery">
      {children}
      
      {showTooltip && (
        <div className="feature-discovery-tooltip">
          {message}
        </div>
      )}
    </div>
  );
};

export default FeatureDiscovery;
