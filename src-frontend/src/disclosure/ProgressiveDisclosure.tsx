import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import './disclosure.css';

// Feature level types
export type FeatureLevel = 'basic' | 'intermediate' | 'advanced' | 'expert';

// Interface for the context
interface DisclosureContextType {
  userLevel: FeatureLevel;
  setUserLevel: (level: FeatureLevel) => void;
  isFeatureVisible: (requiredLevel: FeatureLevel) => boolean;
  userPoints: number;
  addPoints: (points: number) => void;
  resetPoints: () => void;
  getNextLevelThreshold: () => number;
  getNextLevelProgress: () => number;
  showAdvancedFeatures: boolean;
  toggleAdvancedFeatures: () => void;
}

// Feature level thresholds (points needed to unlock)
const LEVEL_THRESHOLDS = {
  basic: 0,
  intermediate: 100,
  advanced: 500,
  expert: 1500,
};

// Context
const DisclosureContext = createContext<DisclosureContextType>({
  userLevel: 'basic',
  setUserLevel: () => {},
  isFeatureVisible: () => false,
  userPoints: 0,
  addPoints: () => {},
  resetPoints: () => {},
  getNextLevelThreshold: () => 0,
  getNextLevelProgress: () => 0,
  showAdvancedFeatures: false,
  toggleAdvancedFeatures: () => {},
});

export const useDisclosure = () => useContext(DisclosureContext);

interface ProgressiveDisclosureProviderProps {
  children: ReactNode;
}

export const ProgressiveDisclosureProvider: React.FC<ProgressiveDisclosureProviderProps> = ({ children }) => {
  // Load initial user level and points from localStorage
  const [userLevel, setUserLevel] = useState<FeatureLevel>(() => {
    const savedLevel = localStorage.getItem('mcp-user-level');
    return (savedLevel as FeatureLevel) || 'basic';
  });
  
  const [userPoints, setUserPoints] = useState(() => {
    const savedPoints = localStorage.getItem('mcp-user-points');
    return savedPoints ? parseInt(savedPoints, 10) : 0;
  });
  
  // Advanced feature override
  const [showAdvancedFeatures, setShowAdvancedFeatures] = useState(() => {
    const savedPref = localStorage.getItem('mcp-show-advanced');
    return savedPref === 'true';
  });
  
  // Save state to localStorage when it changes
  useEffect(() => {
    localStorage.setItem('mcp-user-level', userLevel);
  }, [userLevel]);
  
  useEffect(() => {
    localStorage.setItem('mcp-user-points', String(userPoints));
  }, [userPoints]);
  
  useEffect(() => {
    localStorage.setItem('mcp-show-advanced', String(showAdvancedFeatures));
  }, [showAdvancedFeatures]);
  
  // Check if a feature is visible based on user level
  const isFeatureVisible = (requiredLevel: FeatureLevel): boolean => {
    // If the advanced features toggle is on, show everything
    if (showAdvancedFeatures) return true;
    
    // Otherwise, check the user's level
    const levels: FeatureLevel[] = ['basic', 'intermediate', 'advanced', 'expert'];
    const userLevelIndex = levels.indexOf(userLevel);
    const requiredLevelIndex = levels.indexOf(requiredLevel);
    
    return userLevelIndex >= requiredLevelIndex;
  };
  
  // Add points to the user's score and potentially level up
  const addPoints = (points: number) => {
    setUserPoints(prev => {
      const newPoints = prev + points;
      
      // Check for level up
      if (
        userLevel === 'basic' && newPoints >= LEVEL_THRESHOLDS.intermediate ||
        userLevel === 'intermediate' && newPoints >= LEVEL_THRESHOLDS.advanced ||
        userLevel === 'advanced' && newPoints >= LEVEL_THRESHOLDS.expert
      ) {
        // Determine new level
        let newLevel: FeatureLevel = 'basic';
        if (newPoints >= LEVEL_THRESHOLDS.expert) {
          newLevel = 'expert';
        } else if (newPoints >= LEVEL_THRESHOLDS.advanced) {
          newLevel = 'advanced';
        } else if (newPoints >= LEVEL_THRESHOLDS.intermediate) {
          newLevel = 'intermediate';
        }
        
        // If level changed, show level up notification
        if (newLevel !== userLevel) {
          setUserLevel(newLevel);
          showLevelUpNotification(newLevel);
        }
      }
      
      return newPoints;
    });
  };
  
  // Reset user points
  const resetPoints = () => {
    setUserPoints(0);
    setUserLevel('basic');
  };
  
  // Get threshold for next level
  const getNextLevelThreshold = (): number => {
    switch (userLevel) {
      case 'basic':
        return LEVEL_THRESHOLDS.intermediate;
      case 'intermediate':
        return LEVEL_THRESHOLDS.advanced;
      case 'advanced':
        return LEVEL_THRESHOLDS.expert;
      case 'expert':
        return LEVEL_THRESHOLDS.expert; // Already at max level
    }
  };
  
  // Get progress to next level (0-100)
  const getNextLevelProgress = (): number => {
    if (userLevel === 'expert') return 100;
    
    const currentThreshold = LEVEL_THRESHOLDS[userLevel];
    const nextThreshold = getNextLevelThreshold();
    
    const pointsInLevel = userPoints - currentThreshold;
    const pointsNeededForNextLevel = nextThreshold - currentThreshold;
    
    return Math.min(100, Math.round((pointsInLevel / pointsNeededForNextLevel) * 100));
  };
  
  // Toggle showing advanced features
  const toggleAdvancedFeatures = () => {
    setShowAdvancedFeatures(prev => !prev);
  };
  
  // Show level up notification
  const showLevelUpNotification = (newLevel: FeatureLevel) => {
    // Create a level up notification element
    const notification = document.createElement('div');
    notification.className = 'level-up-notification';
    
    const title = document.createElement('h3');
    title.textContent = `Level Up!`;
    
    const message = document.createElement('p');
    message.textContent = `You've reached the ${newLevel} level. New features are now available!`;
    
    const closeButton = document.createElement('button');
    closeButton.textContent = '×';
    closeButton.className = 'level-up-notification-close';
    closeButton.addEventListener('click', () => {
      document.body.removeChild(notification);
    });
    
    notification.appendChild(closeButton);
    notification.appendChild(title);
    notification.appendChild(message);
    
    // Add animation class
    notification.classList.add('notification-enter');
    
    // Add to document
    document.body.appendChild(notification);
    
    // Auto-remove after 5 seconds
    setTimeout(() => {
      notification.classList.add('notification-exit');
      setTimeout(() => {
        if (document.body.contains(notification)) {
          document.body.removeChild(notification);
        }
      }, 300);
    }, 5000);
  };
  
  return (
    <DisclosureContext.Provider
      value={{
        userLevel,
        setUserLevel,
        isFeatureVisible,
        userPoints,
        addPoints,
        resetPoints,
        getNextLevelThreshold,
        getNextLevelProgress,
        showAdvancedFeatures,
        toggleAdvancedFeatures,
      }}
    >
      {children}
    </DisclosureContext.Provider>
  );
};

// Component to conditionally render based on feature level
interface ProgressiveFeatureProps {
  level: FeatureLevel;
  children: ReactNode;
  fallback?: ReactNode;
}

export const ProgressiveFeature: React.FC<ProgressiveFeatureProps> = ({
  level,
  children,
  fallback,
}) => {
  const { isFeatureVisible } = useDisclosure();
  
  const isVisible = isFeatureVisible(level);
  
  if (isVisible) {
    return <>{children}</>;
  }
  
  return fallback ? <>{fallback}</> : null;
};

// Progress indicator for profile
export const LevelProgressIndicator: React.FC = () => {
  const { userLevel, userPoints, getNextLevelProgress, getNextLevelThreshold } = useDisclosure();
  
  const progress = getNextLevelProgress();
  const nextThreshold = getNextLevelThreshold();
  const isMaxLevel = userLevel === 'expert';
  
  return (
    <div className="level-progress">
      <div className="level-info">
        <span className="user-level">
          Level: <strong>{userLevel}</strong>
        </span>
        <span className="user-points">
          {userPoints} points
        </span>
      </div>
      
      <div className="progress-bar-container">
        <div 
          className="progress-bar-fill"
          style={{ width: `${progress}%` }}
        />
        <span className="progress-label">
          {isMaxLevel 
            ? 'Max Level Reached!' 
            : `${progress}% to ${nextThreshold} points`}
        </span>
      </div>
    </div>
  );
};

// Advanced features toggle
export const AdvancedFeaturesToggle: React.FC = () => {
  const { showAdvancedFeatures, toggleAdvancedFeatures } = useDisclosure();
  
  return (
    <label className="advanced-features-toggle">
      <input
        type="checkbox"
        checked={showAdvancedFeatures}
        onChange={toggleAdvancedFeatures}
      />
      <span className="toggle-label">Show all advanced features</span>
    </label>
  );
};

export default ProgressiveDisclosureProvider;
