import React, { useEffect, useState } from 'react';
import './HelpTopic.css';
import { helpTopics } from './helpContent';
import { UserLevel } from './HelpCenter';

interface HelpTopicProps {
  topicId: string;
  userLevel: UserLevel;
}

const HelpTopic: React.FC<HelpTopicProps> = ({ topicId, userLevel }) => {
  const [topic, setTopic] = useState<any | null>(null);
  
  useEffect(() => {
    const selectedTopic = helpTopics.find(t => t.id === topicId);
    if (selectedTopic) {
      setTopic(selectedTopic);
    }
  }, [topicId]);

  if (!topic) {
    return <div className="help-topic-loading">Loading topic...</div>;
  }

  // Determine which content to show based on user level
  const getContentForLevel = (content: any) => {
    if (userLevel === 'beginner') {
      return content.beginner || content.content;
    } else if (userLevel === 'intermediate') {
      return content.intermediate || content.content;
    } else {
      return content.advanced || content.content;
    }
  };

  return (
    <div className="help-topic">
      <h2>{topic.title}</h2>
      
      {topic.intro && (
        <div className="topic-intro">
          <p>{getContentForLevel(topic.intro)}</p>
        </div>
      )}
      
      {topic.content.map((section: any, index: number) => (
        <div key={index} className="topic-section">
          <h3>{section.title}</h3>
          <div className="section-content">
            {typeof getContentForLevel(section) === 'string' ? (
              <p>{getContentForLevel(section)}</p>
            ) : (
              Array.isArray(getContentForLevel(section)) ? 
                getContentForLevel(section).map((item: any, i: number) => (
                  <div key={i} className="content-item">
                    {item.title && <h4>{item.title}</h4>}
                    <p>{item.content}</p>
                    {item.code && (
                      <pre>
                        <code>{item.code}</code>
                      </pre>
                    )}
                    {item.image && (
                      <div className="image-container">
                        <img src={item.image} alt={item.title || 'Help illustration'} />
                      </div>
                    )}
                    {item.note && (
                      <div className="note-box">
                        <strong>Note:</strong> {item.note}
                      </div>
                    )}
                    {item.warning && (
                      <div className="warning-box">
                        <strong>Warning:</strong> {item.warning}
                      </div>
                    )}
                    {item.tip && (
                      <div className="tip-box">
                        <strong>Tip:</strong> {item.tip}
                      </div>
                    )}
                  </div>
                ))
              : <p>{section.content}</p>
            )}
          </div>
        </div>
      ))}
      
      {topic.examples && (
        <div className="topic-examples">
          <h3>Examples</h3>
          {topic.examples.map((example: any, index: number) => (
            <div key={index} className="example">
              <h4>{example.title}</h4>
              <p>{example.description}</p>
              {example.code && (
                <pre>
                  <code>{example.code}</code>
                </pre>
              )}
              {example.steps && (
                <ol className="steps">
                  {example.steps.map((step: string, i: number) => (
                    <li key={i}>{step}</li>
                  ))}
                </ol>
              )}
            </div>
          ))}
        </div>
      )}
      
      {topic.faq && (
        <div className="topic-faq">
          <h3>Frequently Asked Questions</h3>
          {topic.faq.map((item: any, index: number) => (
            <div key={index} className="faq-item">
              <h4>{item.question}</h4>
              <p>{getContentForLevel(item)}</p>
            </div>
          ))}
        </div>
      )}
      
      {topic.relatedTopics && topic.relatedTopics.length > 0 && (
        <div className="related-topics">
          <h3>Related Topics</h3>
          <ul>
            {topic.relatedTopics.map((relatedId: string) => {
              const relatedTopic = helpTopics.find(t => t.id === relatedId);
              return relatedTopic ? (
                <li key={relatedId}>
                  <a href={`#${relatedId}`} onClick={(e) => {
                    e.preventDefault();
                    // You would typically use a navigation function here
                    // For now we'll just log
                    console.log(`Navigate to topic: ${relatedId}`);
                  }}>
                    {relatedTopic.title}
                  </a>
                </li>
              ) : null;
            })}
          </ul>
        </div>
      )}
    </div>
  );
};

export default HelpTopic;
