import React from 'react';
import './HelpGuide.css';
import { helpTopics } from './helpContent';
import { UserLevel } from './HelpCenter';

interface HelpGuideProps {
  userLevel: UserLevel;
  onTopicSelect: (topicId: string) => void;
}

const HelpGuide: React.FC<HelpGuideProps> = ({ userLevel, onTopicSelect }) => {
  // Group topics by category
  const groupedTopics: Record<string, any[]> = {};
  
  helpTopics.forEach(topic => {
    if (!groupedTopics[topic.category]) {
      groupedTopics[topic.category] = [];
    }
    groupedTopics[topic.category].push(topic);
  });

  // Content tailored to user level
  const getWelcomeContent = () => {
    switch (userLevel) {
      case 'beginner':
        return {
          title: "Welcome to Papin Help Center",
          description: "We're here to help you get started with Papin, an MCP Client. Explore our guides and tutorials to learn the basics.",
          guidance: "If you're new to Papin, we recommend starting with 'Getting Started' and 'Basic Features'."
        };
      case 'intermediate':
        return {
          title: "Welcome to Papin Help Center",
          description: "Find detailed explanations and guides for Papin's features and capabilities.",
          guidance: "Explore our advanced topics to get the most out of Papin's powerful capabilities."
        };
      case 'advanced':
        return {
          title: "Welcome to Papin Technical Documentation",
          description: "Access comprehensive technical documentation for Papin's architecture, APIs, and advanced features.",
          guidance: "Our developer resources include detailed architecture overviews, configuration references, and performance optimization guides."
        };
      default:
        return {
          title: "Welcome to Papin Help Center",
          description: "Find answers, guides, and resources for Papin, an MCP Client.",
          guidance: "Select a topic from the sidebar or explore the featured topics below."
        };
    }
  };

  const welcomeContent = getWelcomeContent();

  return (
    <div className="help-guide">
      <div className="guide-welcome">
        <h2>{welcomeContent.title}</h2>
        <p>{welcomeContent.description}</p>
        <p className="guide-tip">{welcomeContent.guidance}</p>
      </div>

      {userLevel === 'beginner' && (
        <div className="quick-start-guide">
          <h3>Quick Start Guide</h3>
          <div className="steps">
            <div className="step">
              <div className="step-number">1</div>
              <div className="step-content">
                <h4>Installation</h4>
                <p>Download and install Papin on your computer.</p>
                <button className="step-button" onClick={() => onTopicSelect('installation')}>
                  Installation Guide
                </button>
              </div>
            </div>
            <div className="step">
              <div className="step-number">2</div>
              <div className="step-content">
                <h4>Account Setup</h4>
                <p>Create your account and configure basic settings.</p>
                <button className="step-button" onClick={() => onTopicSelect('account-setup')}>
                  Account Guide
                </button>
              </div>
            </div>
            <div className="step">
              <div className="step-number">3</div>
              <div className="step-content">
                <h4>First Conversation</h4>
                <p>Learn how to start your first AI conversation.</p>
                <button className="step-button" onClick={() => onTopicSelect('conversations')}>
                  Conversation Guide
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="featured-topics">
        <h3>Featured Topics</h3>
        <div className="topic-cards">
          {['offline-capabilities', 'performance-monitoring', 'local-llm'].map(topicId => {
            const topic = helpTopics.find(t => t.id === topicId);
            if (!topic) return null;
            
            return (
              <div 
                key={topic.id} 
                className="topic-card"
                onClick={() => onTopicSelect(topic.id)}
              >
                <h4>{topic.title}</h4>
                <p>{topic.summary}</p>
                <span className="learn-more">Learn more →</span>
              </div>
            );
          })}
        </div>
      </div>

      <div className="help-categories">
        {Object.entries(groupedTopics).map(([category, topics]) => (
          <div key={category} className="help-category">
            <h3>{category}</h3>
            <div className="category-topics">
              {topics.map(topic => (
                <div 
                  key={topic.id} 
                  className="category-topic"
                  onClick={() => onTopicSelect(topic.id)}
                >
                  <h4>{topic.title}</h4>
                  <p>{topic.summary}</p>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default HelpGuide;
