import React from 'react';
import { FeatureLevel } from './ProgressiveDisclosure';

interface LockedFeatureMessageProps {
  level: FeatureLevel;
  className?: string;
}

const LockedFeatureMessage: React.FC<LockedFeatureMessageProps> = ({ level, className = '' }) => {
  return (
    <div className={`locked-feature-message ${className}`}>
      This feature is available at the <strong>{level}</strong> level.
      <br />
      Continue using the app to unlock it!
    </div>
  );
};

export default LockedFeatureMessage;
