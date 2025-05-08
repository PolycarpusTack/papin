import React from 'react';
import { FeatureLevel } from './ProgressiveDisclosure';

interface FeatureBadgeProps {
  level: FeatureLevel;
  className?: string;
}

const FeatureBadge: React.FC<FeatureBadgeProps> = ({ level, className = '' }) => {
  return (
    <span className={`feature-level-badge ${level} ${className}`}>
      {level.charAt(0).toUpperCase() + level.slice(1)}
    </span>
  );
};

export default FeatureBadge;
