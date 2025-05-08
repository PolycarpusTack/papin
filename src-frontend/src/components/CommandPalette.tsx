import React, { useState, useEffect, useRef } from 'react';
import './CommandPalette.css';

export interface CommandItem {
  id: string;
  name: string;
  description?: string;
  shortcut?: string;
  category?: string;
  icon?: React.ReactNode;
  action: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: CommandItem[];
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  commands,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const commandListRef = useRef<HTMLUListElement>(null);
  
  // Filter commands based on search query
  const filteredCommands = commands.filter((command) => {
    const query = searchQuery.toLowerCase();
    return (
      command.name.toLowerCase().includes(query) ||
      (command.description?.toLowerCase().includes(query)) ||
      (command.category?.toLowerCase().includes(query))
    );
  });
  
  // Focus the search input when the palette opens
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 10);
    } else {
      // Reset search and selection when closing
      setSearchQuery('');
      setSelectedIndex(0);
    }
  }, [isOpen]);
  
  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prevIndex) => 
          prevIndex < filteredCommands.length - 1 ? prevIndex + 1 : prevIndex
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prevIndex) => 
          prevIndex > 0 ? prevIndex - 1 : prevIndex
        );
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          executeCommand(filteredCommands[selectedIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  };
  
  // Execute a command and close the palette
  const executeCommand = (command: CommandItem) => {
    command.action();
    onClose();
  };
  
  // Scroll selected item into view
  useEffect(() => {
    if (commandListRef.current && filteredCommands.length > 0) {
      const selectedElement = commandListRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({
          block: 'nearest',
        });
      }
    }
  }, [selectedIndex, filteredCommands.length]);
  
  if (!isOpen) return null;
  
  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={(e) => e.stopPropagation()}>
        <div className="command-palette-header">
          <div className="command-palette-search">
            <svg 
              className="command-palette-search-icon" 
              xmlns="http://www.w3.org/2000/svg" 
              viewBox="0 0 24 24" 
              fill="none" 
              stroke="currentColor" 
              strokeWidth="2" 
              strokeLinecap="round" 
              strokeLinejoin="round"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
              ref={searchInputRef}
              type="text"
              className="command-palette-search-input"
              placeholder="Type a command or search..."
              value={searchQuery}
              onChange={(e) => {
                setSearchQuery(e.target.value);
                setSelectedIndex(0); // Reset selection when search changes
              }}
              onKeyDown={handleKeyDown}
            />
          </div>
        </div>
        
        <div className="command-palette-content">
          {filteredCommands.length > 0 ? (
            <ul className="command-palette-list" ref={commandListRef}>
              {filteredCommands.map((command, index) => (
                <li
                  key={command.id}
                  className={`command-palette-item ${index === selectedIndex ? 'selected' : ''}`}
                  onClick={() => executeCommand(command)}
                  onMouseEnter={() => setSelectedIndex(index)}
                >
                  <div className="command-palette-item-content">
                    {command.icon && (
                      <div className="command-palette-item-icon">
                        {command.icon}
                      </div>
                    )}
                    <div className="command-palette-item-text">
                      <div className="command-palette-item-name">
                        {command.name}
                      </div>
                      {command.description && (
                        <div className="command-palette-item-description">
                          {command.description}
                        </div>
                      )}
                    </div>
                  </div>
                  {command.shortcut && (
                    <div className="command-palette-item-shortcut">
                      {command.shortcut.split('+').map((key, i) => (
                        <kbd key={i} className="command-palette-kbd">
                          {key.trim()}
                        </kbd>
                      ))}
                    </div>
                  )}
                </li>
              ))}
            </ul>
          ) : (
            <div className="command-palette-empty">
              No commands found
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default CommandPalette;
