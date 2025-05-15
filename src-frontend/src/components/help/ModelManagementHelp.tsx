import React, { useState, useEffect, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import { useSearchParams } from 'react-router-dom';
import remarkGfm from 'remark-gfm';
import './Help.css';

// Import CSS for interactive components
import './HelpInteractive.css';

// Mock function for now - would be connected to Tauri in production
const fetchHelpContent = async (): Promise<string> => {
  // In production, this would use Tauri's fs API to read the markdown
  try {
    const response = await fetch('/docs/MODEL_MANAGEMENT.md');
    return await response.text();
  } catch (error) {
    console.error("Failed to load help content:", error);
    return "# Error Loading Help Content\nPlease try again later.";
  }
};

interface Section {
  id: string;
  title: string;
  level: number;
  element: HTMLElement;
}

const ModelManagementHelp: React.FC = () => {
  const [content, setContent] = useState<string>('');
  const [sections, setSections] = useState<Section[]>([]);
  const [activeSection, setActiveSection] = useState<string>('overview');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const contentRef = useRef<HTMLDivElement>(null);
  const [searchParams, setSearchParams] = useSearchParams();

  useEffect(() => {
    // Load content
    fetchHelpContent().then(content => {
      setContent(content);
    });

    // Check for section in URL
    const sectionParam = searchParams.get('section');
    if (sectionParam) {
      setActiveSection(sectionParam);
    }
  }, [searchParams]);

  useEffect(() => {
    // Extract sections after content is loaded and rendered
    if (content && contentRef.current) {
      // Allow time for React Markdown to render
      setTimeout(() => {
        const headings = contentRef.current?.querySelectorAll('h2, h3, h4') || [];
        const extractedSections: Section[] = [];
        
        headings.forEach((heading) => {
          const id = heading.getAttribute('id') || '';
          const title = heading.textContent || '';
          const level = parseInt(heading.tagName.replace('H', ''), 10);
          
          if (id && title) {
            extractedSections.push({
              id,
              title,
              level,
              element: heading as HTMLElement
            });
          }
        });
        
        setSections(extractedSections);
      }, 100);
    }
  }, [content]);

  useEffect(() => {
    // Scroll to the active section
    if (activeSection && contentRef.current) {
      const element = document.getElementById(activeSection);
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    }
  }, [activeSection, sections]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim() || !contentRef.current) return;

    // Simple search implementation - in production this would be more sophisticated
    const results: any[] = [];
    const allElements = contentRef.current.querySelectorAll('p, li, h2, h3, h4, th, td');
    
    allElements.forEach(element => {
      const text = element.textContent?.toLowerCase() || '';
      if (text.includes(searchQuery.toLowerCase())) {
        // Find the closest section heading
        let current = element;
        let heading = null;
        
        while (current && !heading) {
          current = current.previousElementSibling as HTMLElement;
          if (current && ['H2', 'H3', 'H4'].includes(current.tagName)) {
            heading = current;
          }
        }
        
        if (!heading) {
          // If no heading found by traversing siblings, find closest parent section
          let parent = element.parentElement;
          while (parent && !heading) {
            const parentHeading = parent.querySelector('h2, h3, h4');
            if (parentHeading) {
              heading = parentHeading;
            }
            parent = parent.parentElement;
          }
        }
        
        const sectionId = heading?.getAttribute('id') || '';
        const sectionTitle = heading?.textContent || 'Unknown Section';
        
        results.push({
          sectionId,
          sectionTitle,
          text: element.textContent,
          element
        });
      }
    });
    
    setSearchResults(results);
  };

  const clearSearch = () => {
    setSearchQuery('');
    setSearchResults([]);
  };

  const navigateToResult = (result: any) => {
    if (result.sectionId) {
      setActiveSection(result.sectionId);
      setSearchParams({ section: result.sectionId });
    }
    if (result.element) {
      result.element.scrollIntoView({ behavior: 'smooth' });
      // Highlight the element temporarily
      result.element.classList.add('search-highlight');
      setTimeout(() => {
        result.element.classList.remove('search-highlight');
      }, 3000);
    }
    clearSearch();
  };

  const renderSidebar = () => {
    // Group sections by level
    const mainSections = sections.filter(s => s.level === 2);
    
    return (
      <div className="help-sidebar">
        <div className="help-search">
          <form onSubmit={handleSearch}>
            <input
              type="text"
              placeholder="Search help..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            <button type="submit">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18">
                <path fill="none" d="M0 0h24v24H0z"/>
                <path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
              </svg>
            </button>
          </form>
        </div>
        
        {searchResults.length > 0 ? (
          <div className="search-results">
            <div className="search-header">
              <h3>Search Results ({searchResults.length})</h3>
              <button onClick={clearSearch} className="clear-search">×</button>
            </div>
            <ul>
              {searchResults.map((result, index) => (
                <li key={index} onClick={() => navigateToResult(result)}>
                  <strong>{result.sectionTitle}</strong>
                  <p>{result.text}</p>
                </li>
              ))}
            </ul>
          </div>
        ) : (
          <ul className="section-nav">
            {mainSections.map((section) => (
              <li 
                key={section.id}
                className={activeSection === section.id ? 'active' : ''}
              >
                <a 
                  href={`#${section.id}`}
                  onClick={(e) => {
                    e.preventDefault();
                    setActiveSection(section.id);
                    setSearchParams({ section: section.id });
                  }}
                >
                  {section.title}
                </a>
                
                {/* Render subsections if this is the active section */}
                {activeSection === section.id && (
                  <ul className="subsection-nav">
                    {sections
                      .filter(s => s.level === 3 && s.element.offsetTop > section.element.offsetTop && 
                             (sections.find(next => next.level === 2 && next.element.offsetTop > section.element.offsetTop)?.element.offsetTop || Infinity) > s.element.offsetTop)
                      .map((subsection) => (
                        <li 
                          key={subsection.id}
                          className={activeSection === subsection.id ? 'active' : ''}
                        >
                          <a 
                            href={`#${subsection.id}`}
                            onClick={(e) => {
                              e.preventDefault();
                              setActiveSection(subsection.id);
                              setSearchParams({ section: subsection.id });
                            }}
                          >
                            {subsection.title}
                          </a>
                        </li>
                      ))}
                  </ul>
                )}
              </li>
            ))}
          </ul>
        )}
        
        <div className="help-actions">
          <button className="action-button">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18">
              <path fill="none" d="M0 0h24v24H0z"/>
              <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
            </svg>
            Download PDF
          </button>
          <button className="action-button">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18">
              <path fill="none" d="M0 0h24v24H0z"/>
              <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
            </svg>
            Back to Top
          </button>
        </div>
      </div>
    );
  };

  const renderInteractiveElements = () => {
    if (!contentRef.current) return;

    // Initialize tabbed sections
    const tabbedSections = contentRef.current.querySelectorAll('.tabbed-section');
    tabbedSections.forEach(section => {
      const tabs = section.querySelectorAll('.tab');
      const contents = section.querySelectorAll('.tab-content');
      
      tabs.forEach((tab, index) => {
        tab.addEventListener('click', () => {
          // Remove active class from all tabs and contents
          tabs.forEach(t => t.classList.remove('active'));
          contents.forEach(c => c.classList.remove('active'));
          
          // Add active class to selected tab and content
          tab.classList.add('active');
          const target = tab.getAttribute('data-target');
          const targetContent = section.querySelector(`#${target}`);
          if (targetContent) targetContent.classList.add('active');
        });
      });
    });

    // Initialize expandable sections
    const expandableSections = contentRef.current.querySelectorAll('.expandable-section');
    expandableSections.forEach(section => {
      const header = section.querySelector('.section-header');
      const content = section.querySelector('.section-content');
      
      if (header && content) {
        header.addEventListener('click', () => {
          content.classList.toggle('expanded');
          header.classList.toggle('expanded');
        });
      }
    });

    // Initialize wizard forms
    const wizardButtons = contentRef.current.querySelectorAll('.model-wizard button, .best-practices-wizard button');
    wizardButtons.forEach(button => {
      button.addEventListener('click', () => {
        alert('Wizard would launch here in the full implementation');
      });
    });

    // Initialize troubleshooting selector
    const troubleshootingSelector = contentRef.current.querySelector('#issue-selector');
    const troubleshootingButton = contentRef.current.querySelector('.problem-selector button');
    
    if (troubleshootingSelector && troubleshootingButton) {
      troubleshootingButton.addEventListener('click', () => {
        const selectedIssue = (troubleshootingSelector as HTMLSelectElement).value;
        const issueCards = contentRef.current!.querySelectorAll('.issue-card');
        
        issueCards.forEach(card => {
          card.classList.remove('active');
        });
        
        // Find and activate selected issue card
        switch(selectedIssue) {
          case 'download-fails':
            issueCards[0].classList.add('active');
            issueCards[0].scrollIntoView({ behavior: 'smooth' });
            break;
          case 'download-interrupted':
            issueCards[1].classList.add('active');
            issueCards[1].scrollIntoView({ behavior: 'smooth' });
            break;
          case 'model-wont-run':
            issueCards[2].classList.add('active');
            issueCards[2].scrollIntoView({ behavior: 'smooth' });
            break;
          case 'performance-issues':
            issueCards[3].classList.add('active');
            issueCards[3].scrollIntoView({ behavior: 'smooth' });
            break;
        }
      });
    }

    // Initialize diagnostic tool
    const diagnosticButton = contentRef.current.querySelector('.diagnostic-tool button');
    if (diagnosticButton) {
      diagnosticButton.addEventListener('click', () => {
        alert('Diagnostic tool would run here in the full implementation');
      });
    }

    // Initialize solution buttons
    const solutionButtons = contentRef.current.querySelectorAll('.solution-btn');
    solutionButtons.forEach(button => {
      button.addEventListener('click', () => {
        alert(`Solution action: ${button.textContent}`);
      });
    });
  };

  useEffect(() => {
    // Initialize interactive elements after content is rendered
    if (content) {
      setTimeout(() => {
        renderInteractiveElements();
      }, 200);
    }
  }, [content]);

  // Custom components for ReactMarkdown
  const components = {
    // Add ID attributes to headings for linking
    h2: ({node, ...props}) => <h2 id={props.children?.toString().toLowerCase().replace(/\s+/g, '-')} {...props} />,
    h3: ({node, ...props}) => <h3 id={props.children?.toString().toLowerCase().replace(/\s+/g, '-')} {...props} />,
    h4: ({node, ...props}) => <h4 id={props.children?.toString().toLowerCase().replace(/\s+/g, '-')} {...props} />,
    
    // Handle images with captions
    img: ({node, ...props}) => (
      <div className="image-container">
        <img src={props.src} alt={props.alt || ''} />
        {props.alt && <div className="image-caption">{props.alt}</div>}
      </div>
    ),
    
    // Custom code blocks with syntax highlighting
    code: ({node, inline, className, children, ...props}) => {
      if (inline) {
        return <code className={className} {...props}>{children}</code>;
      }
      
      return (
        <div className="code-block">
          <div className="code-header">
            <span>{className?.replace(/language-/, '') || 'code'}</span>
            <button className="copy-button" onClick={() => {
              navigator.clipboard.writeText(children.toString());
              alert('Code copied to clipboard');
            }}>Copy</button>
          </div>
          <pre className={className}>
            <code {...props}>{children}</code>
          </pre>
        </div>
      );
    }
  };

  return (
    <div className="help-container">
      {renderSidebar()}
      
      <div className="help-content" ref={contentRef}>
        {content ? (
          <ReactMarkdown 
            components={components}
            remarkPlugins={[remarkGfm]}
          >
            {content}
          </ReactMarkdown>
        ) : (
          <div className="loading">Loading help content...</div>
        )}
      </div>
      
      {/* Contextual Help Button (fixed position) */}
      <div className="contextual-help-button">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
          <path fill="none" d="M0 0h24v24H0z"/>
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm2.07-7.75l-.9.92C13.45 12.9 13 13.5 13 15h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H8c0-2.21 1.79-4 4-4s4 1.79 4 4c0 .88-.36 1.68-.93 2.25z"/>
        </svg>
      </div>
      
      {/* Video Tutorials Drawer (collapsed by default) */}
      <div className="video-tutorials-drawer">
        <div className="drawer-tab" onClick={() => {
          const drawer = document.querySelector('.video-tutorials-drawer');
          drawer?.classList.toggle('expanded');
        }}>
          <span>Video Tutorials</span>
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18">
            <path fill="none" d="M0 0h24v24H0z"/>
            <path d="M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6-6 6z"/>
          </svg>
        </div>
        <div className="drawer-content">
          <h3>Quick Video Tutorials</h3>
          <div className="video-list">
            <div className="video-item">
              <div className="video-thumbnail">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                  <path fill="none" d="M0 0h24v24H0z"/>
                  <path d="M10 16.5l6-4.5-6-4.5v9zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/>
                </svg>
              </div>
              <div className="video-info">
                <h4>Getting Started with Models</h4>
                <p>2:45 • Basic introduction</p>
              </div>
            </div>
            <div className="video-item">
              <div className="video-thumbnail">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                  <path fill="none" d="M0 0h24v24H0z"/>
                  <path d="M10 16.5l6-4.5-6-4.5v9zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/>
                </svg>
              </div>
              <div className="video-info">
                <h4>Advanced Model Comparison</h4>
                <p>5:12 • Compare models efficiently</p>
              </div>
            </div>
            <div className="video-item">
              <div className="video-thumbnail">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                  <path fill="none" d="M0 0h24v24H0z"/>
                  <path d="M10 16.5l6-4.5-6-4.5v9zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/>
                </svg>
              </div>
              <div className="video-info">
                <h4>Troubleshooting Downloads</h4>
                <p>3:28 • Fix common issues</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ModelManagementHelp;