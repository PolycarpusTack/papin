import React from 'react';
import { useHelp } from './HelpProvider';

interface HelpTriggerProps {
  topicId: string;
  className?: string;
}

const HelpTrigger: React.FC<HelpTriggerProps> = ({ topicId, className = '' }) => {
  const { openTopicInPanel, topics } = useHelp();
  
  const topic = topics.find(t => t.id === topicId);
  if (!topic) return null;
  
  return (
    <button
      className={`help-trigger ${className}`}
      onClick={() => openTopicInPanel(topicId)}
      aria-label={`Help: ${topic.title}`}
      title={topic.title}
    >
      ?
    </button>
  );
};

export default HelpTrigger;
