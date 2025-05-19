import React from 'react';
import { usePlatform } from '../hooks/usePlatform';
import { 
  Monitor, 
  AppleIcon, 
  GithubIcon,
  WindowsIcon, 
  Computer, 
  Info
} from 'lucide-react';

interface PlatformIndicatorProps {
  showDetails?: boolean;
  className?: string;
}

/**
 * Shows the current platform as an icon with optional details
 */
const PlatformIndicator: React.FC<PlatformIndicatorProps> = ({ 
  showDetails = false,
  className = ''
}) => {
  const { platform, isTouch } = usePlatform();
  
  // Helper to get platform icon
  const getPlatformIcon = () => {
    switch (platform) {
      case 'windows':
        return <WindowsIcon size={18} className="text-blue-500" />;
      case 'macos':
        return <AppleIcon size={18} className="text-gray-800 dark:text-gray-200" />;
      case 'linux':
        return <GithubIcon size={18} className="text-orange-500" />;
      default:
        return <Computer size={18} className="text-gray-500" />;
    }
  };
  
  // Helper to get platform name
  const getPlatformName = () => {
    switch (platform) {
      case 'windows':
        return 'Windows';
      case 'macos':
        return 'macOS';
      case 'linux':
        return 'Linux';
      default:
        return 'Unknown';
    }
  };
  
  if (!showDetails) {
    return (
      <div className={`flex items-center ${className}`} title={`Running on ${getPlatformName()}`}>
        {getPlatformIcon()}
      </div>
    );
  }
  
  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {getPlatformIcon()}
      <span className="text-sm font-medium">{getPlatformName()}</span>
      {isTouch && (
        <span className="ml-1 text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 px-1.5 py-0.5 rounded">
          Touch
        </span>
      )}
    </div>
  );
};

export default PlatformIndicator;
