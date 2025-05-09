import React, { useState, useEffect } from 'react';
import './HelpCenter.css';
import HelpTopic from './HelpTopic';
import HelpSearch from './HelpSearch';
import HelpGuide from './HelpGuide';
import { helpTopics } from './helpContent';

export type UserLevel = 'beginner' | 'intermediate' | 'advanced';

const HelpCenter: React.FC = () => {
  const [selectedTopic, setSelectedTopic] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [userLevel, setUserLevel] = useState<UserLevel>('beginner');
  const [filteredTopics, setFilteredTopics] = useState(helpTopics);

  useEffect(() => {
    if (searchQuery) {
      const filtered = helpTopics.filter(topic => 
        topic.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        topic.content.some(item => 
          item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          item.content.toLowerCase().includes(searchQuery.toLowerCase())
        )
      );
      setFilteredTopics(filtered);
    } else {
      setFilteredTopics(helpTopics);
    }
  }, [searchQuery]);

  const handleTopicSelect = (topicId: string) => {
    setSelectedTopic(topicId);
  };

  const handleSearch = (query: string) => {
    setSearchQuery(query);
    setSelectedTopic(null);
  };

  const handleUserLevelChange = (level: UserLevel) => {
    setUserLevel(level);
  };

  return (
    <div className="help-center">
      <div className="help-header">
        <h1>Papin Help Center</h1>
        <div className="user-level-selector">
          <span>Expertise Level:</span>
          <div className="level-buttons">
            <button 
              className={userLevel === 'beginner' ? 'active' : ''} 
              onClick={() => handleUserLevelChange('beginner')}
            >
              Beginner
            </button>
            <button 
              className={userLevel === 'intermediate' ? 'active' : ''} 
              onClick={() => handleUserLevelChange('intermediate')}
            >
              Intermediate
            </button>
            <button 
              className={userLevel === 'advanced' ? 'active' : ''} 
              onClick={() => handleUserLevelChange('advanced')}
            >
              Advanced
            </button>
          </div>
        </div>
      </div>
      
      <div className="help-content">
        <div className="help-sidebar">
          <HelpSearch onSearch={handleSearch} />
          <div className="topic-list">
            <h2>Topics</h2>
            {filteredTopics.map(topic => (
              <div 
                key={topic.id}
                className={`topic-item ${selectedTopic === topic.id ? 'active' : ''}`}
                onClick={() => handleTopicSelect(topic.id)}
              >
                {topic.title}
              </div>
            ))}
          </div>
        </div>
        
        <div className="help-main">
          {selectedTopic ? (
            <HelpTopic 
              topicId={selectedTopic} 
              userLevel={userLevel} 
            />
          ) : searchQuery ? (
            <div className="search-results">
              <h2>Search Results for "{searchQuery}"</h2>
              {filteredTopics.length === 0 ? (
                <p>No results found. Try a different search term.</p>
              ) : (
                filteredTopics.map(topic => (
                  <div key={topic.id} className="search-result-item">
                    <h3 onClick={() => handleTopicSelect(topic.id)}>{topic.title}</h3>
                    <p>{topic.summary}</p>
                  </div>
                ))
              )}
            </div>
          ) : (
            <HelpGuide userLevel={userLevel} onTopicSelect={handleTopicSelect} />
          )}
        </div>
      </div>
    </div>
  );
};

export default HelpCenter;
