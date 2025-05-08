import React, { createContext, useContext, useState, ReactNode } from 'react';
import './help.css';

// Help topic structure
export interface HelpTopic {
  id: string;
  title: string;
  content: string;
  category: string;
  keywords: string[];
  related?: string[];
}

// Explanation structure for inline help
export interface Explanation {
  id: string;
  title: string;
  content: string;
  targetSelector?: string;
}

// Context interface
interface HelpContextType {
  // Topics management
  topics: HelpTopic[];
  explanations: Explanation[];
  registerTopic: (topic: HelpTopic) => void;
  registerExplanation: (explanation: Explanation) => void;
  
  // Help panel controls
  isHelpPanelOpen: boolean;
  openHelpPanel: () => void;
  closeHelpPanel: () => void;
  openTopicInPanel: (topicId: string) => void;
  
  // Active help panel state
  activeTopicId: string | null;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  searchResults: HelpTopic[];
  
  // Inline help tooltips
  enableInlineHelp: boolean;
  toggleInlineHelp: () => void;
  activeExplanation: Explanation | null;
  showExplanation: (id: string) => void;
  hideExplanation: () => void;
}

const HelpContext = createContext<HelpContextType>({
  topics: [],
  explanations: [],
  registerTopic: () => {},
  registerExplanation: () => {},
  
  isHelpPanelOpen: false,
  openHelpPanel: () => {},
  closeHelpPanel: () => {},
  openTopicInPanel: () => {},
  
  activeTopicId: null,
  searchQuery: '',
  setSearchQuery: () => {},
  searchResults: [],
  
  enableInlineHelp: false,
  toggleInlineHelp: () => {},
  activeExplanation: null,
  showExplanation: () => {},
  hideExplanation: () => {},
});

export const useHelp = () => useContext(HelpContext);

interface HelpProviderProps {
  children: ReactNode;
}

export const HelpProvider: React.FC<HelpProviderProps> = ({ children }) => {
  // Help content
  const [topics, setTopics] = useState<HelpTopic[]>([]);
  const [explanations, setExplanations] = useState<Explanation[]>([]);
  
  // Help panel state
  const [isHelpPanelOpen, setIsHelpPanelOpen] = useState(false);
  const [activeTopicId, setActiveTopicId] = useState<string | null>(null);
  
  // Search
  const [searchQuery, setSearchQuery] = useState('');
  
  // Inline help state
  const [enableInlineHelp, setEnableInlineHelp] = useState(() => {
    const savedPref = localStorage.getItem('mcp-enable-inline-help');
    return savedPref !== null ? savedPref === 'true' : false;
  });
  const [activeExplanation, setActiveExplanation] = useState<Explanation | null>(null);
  
  // Register a new help topic
  const registerTopic = (topic: HelpTopic) => {
    setTopics(prev => {
      // Check if topic already exists
      const exists = prev.some(t => t.id === topic.id);
      if (exists) {
        return prev.map(t => t.id === topic.id ? topic : t);
      }
      return [...prev, topic];
    });
  };
  
  // Register an explanation for inline help
  const registerExplanation = (explanation: Explanation) => {
    setExplanations(prev => {
      // Check if explanation already exists
      const exists = prev.some(e => e.id === explanation.id);
      if (exists) {
        return prev.map(e => e.id === explanation.id ? explanation : e);
      }
      return [...prev, explanation];
    });
  };
  
  // Help panel controls
  const openHelpPanel = () => {
    setIsHelpPanelOpen(true);
  };
  
  const closeHelpPanel = () => {
    setIsHelpPanelOpen(false);
  };
  
  const openTopicInPanel = (topicId: string) => {
    setActiveTopicId(topicId);
    openHelpPanel();
  };
  
  // Toggle inline help
  const toggleInlineHelp = () => {
    setEnableInlineHelp(prev => {
      const newValue = !prev;
      localStorage.setItem('mcp-enable-inline-help', String(newValue));
      return newValue;
    });
  };
  
  // Show/hide explanations
  const showExplanation = (id: string) => {
    const explanation = explanations.find(e => e.id === id);
    if (explanation) {
      setActiveExplanation(explanation);
    }
  };
  
  const hideExplanation = () => {
    setActiveExplanation(null);
  };
  
  // Filter topics based on search query
  const searchResults = searchQuery.trim() === '' 
    ? [] 
    : topics.filter(topic => {
        const query = searchQuery.toLowerCase();
        return (
          topic.title.toLowerCase().includes(query) ||
          topic.content.toLowerCase().includes(query) ||
          topic.keywords.some(keyword => keyword.toLowerCase().includes(query))
        );
      });
  
  // Get active topic
  const activeTopic = activeTopicId
    ? topics.find(topic => topic.id === activeTopicId)
    : null;
  
  return (
    <HelpContext.Provider
      value={{
        topics,
        explanations,
        registerTopic,
        registerExplanation,
        
        isHelpPanelOpen,
        openHelpPanel,
        closeHelpPanel,
        openTopicInPanel,
        
        activeTopicId,
        searchQuery,
        setSearchQuery,
        searchResults,
        
        enableInlineHelp,
        toggleInlineHelp,
        activeExplanation,
        showExplanation,
        hideExplanation,
      }}
    >
      {children}
      {isHelpPanelOpen && (
        <HelpPanel activeTopic={activeTopic} onClose={closeHelpPanel} />
      )}
      {activeExplanation && <ExplanationTooltip />}
      {enableInlineHelp && <InlineHelpOverlay />}
    </HelpContext.Provider>
  );
};

// Help panel component
interface HelpPanelProps {
  activeTopic: HelpTopic | null;
  onClose: () => void;
}

const HelpPanel: React.FC<HelpPanelProps> = ({ activeTopic, onClose }) => {
  const { topics, searchQuery, setSearchQuery, searchResults, openTopicInPanel } = useHelp();
  
  // Group topics by category for sidebar
  const categories = topics.reduce<Record<string, HelpTopic[]>>((acc, topic) => {
    if (!acc[topic.category]) {
      acc[topic.category] = [];
    }
    acc[topic.category].push(topic);
    return acc;
  }, {});
  
  return (
    <div className="help-panel-overlay" onClick={onClose}>
      <div className="help-panel" onClick={e => e.stopPropagation()}>
        <div className="help-panel-sidebar">
          <div className="help-panel-search">
            <input
              type="text"
              placeholder="Search help topics..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              className="help-panel-search-input"
            />
          </div>
          
          <div className="help-panel-nav">
            {searchQuery.trim() !== '' ? (
              <div className="help-panel-search-results">
                <h3 className="help-panel-category-title">Search Results</h3>
                
                {searchResults.length > 0 ? (
                  <ul className="help-panel-topic-list">
                    {searchResults.map(topic => (
                      <li key={topic.id} className="help-panel-topic-item">
                        <button
                          className={`help-panel-topic-button ${
                            activeTopic?.id === topic.id ? 'active' : ''
                          }`}
                          onClick={() => openTopicInPanel(topic.id)}
                        >
                          {topic.title}
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="help-panel-no-results">No results found</p>
                )}
              </div>
            ) : (
              <>
                {Object.entries(categories).map(([category, categoryTopics]) => (
                  <div key={category} className="help-panel-category">
                    <h3 className="help-panel-category-title">{category}</h3>
                    <ul className="help-panel-topic-list">
                      {categoryTopics.map(topic => (
                        <li key={topic.id} className="help-panel-topic-item">
                          <button
                            className={`help-panel-topic-button ${
                              activeTopic?.id === topic.id ? 'active' : ''
                            }`}
                            onClick={() => openTopicInPanel(topic.id)}
                          >
                            {topic.title}
                          </button>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </>
            )}
          </div>
        </div>
        
        <div className="help-panel-content">
          <div className="help-panel-header">
            <h2 className="help-panel-title">
              {activeTopic ? activeTopic.title : 'Help Center'}
            </h2>
            <button className="help-panel-close" onClick={onClose}>
              &times;
            </button>
          </div>
          
          <div className="help-panel-body">
            {activeTopic ? (
              <>
                <div 
                  className="help-topic-content"
                  dangerouslySetInnerHTML={{ __html: activeTopic.content }}
                />
                
                {activeTopic.related && activeTopic.related.length > 0 && (
                  <div className="help-topic-related">
                    <h3>Related Topics</h3>
                    <ul className="help-topic-related-list">
                      {activeTopic.related.map(relatedId => {
                        const relatedTopic = topics.find(t => t.id === relatedId);
                        if (!relatedTopic) return null;
                        
                        return (
                          <li key={relatedId} className="help-topic-related-item">
                            <button
                              className="help-topic-related-button"
                              onClick={() => openTopicInPanel(relatedId)}
                            >
                              {relatedTopic.title}
                            </button>
                          </li>
                        );
                      })}
                    </ul>
                  </div>
                )}
              </>
            ) : (
              <div className="help-panel-welcome">
                <h3>Welcome to the Help Center</h3>
                <p>
                  Select a topic from the sidebar or search for help on a specific topic.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

// Explanation tooltip component
const ExplanationTooltip: React.FC = () => {
  const { activeExplanation, hideExplanation } = useHelp();
  
  if (!activeExplanation) return null;
  
  // Position the tooltip
  let tooltipPosition = { top: 0, left: 0 };
  
  // If there's a target selector, position near the target element
  if (activeExplanation.targetSelector) {
    const targetElement = document.querySelector(activeExplanation.targetSelector);
    if (targetElement) {
      const rect = targetElement.getBoundingClientRect();
      tooltipPosition = {
        top: rect.bottom + 10,
        left: rect.left + rect.width / 2,
      };
    }
  } else {
    // Otherwise, center in viewport
    tooltipPosition = {
      top: window.innerHeight / 2,
      left: window.innerWidth / 2,
    };
  }
  
  return (
    <div
      className="explanation-tooltip"
      style={{
        top: `${tooltipPosition.top}px`,
        left: `${tooltipPosition.left}px`,
      }}
    >
      <div className="explanation-tooltip-header">
        <h3 className="explanation-tooltip-title">{activeExplanation.title}</h3>
        <button
          className="explanation-tooltip-close"
          onClick={hideExplanation}
          aria-label="Close explanation"
        >
          &times;
        </button>
      </div>
      <div className="explanation-tooltip-content">
        {activeExplanation.content}
      </div>
    </div>
  );
};

// Inline help overlay - shows help icons near elements with explanations
const InlineHelpOverlay: React.FC = () => {
  const { explanations, showExplanation } = useHelp();
  
  return (
    <div className="inline-help-overlay">
      {explanations
        .filter(explanation => explanation.targetSelector)
        .map(explanation => {
          const targetElement = document.querySelector(explanation.targetSelector!);
          if (!targetElement) return null;
          
          const rect = targetElement.getBoundingClientRect();
          
          return (
            <button
              key={explanation.id}
              className="inline-help-icon"
              onClick={() => showExplanation(explanation.id)}
              aria-label={`Help for ${explanation.title}`}
              style={{
                top: `${rect.top}px`,
                left: `${rect.right + 5}px`,
              }}
            >
              ?
            </button>
          );
        })}
    </div>
  );
};

export default HelpProvider;
