import { useState, useEffect, useCallback } from 'react';
import { CommandItem } from '../components/CommandPalette';

export const useCommandPalette = (initialCommands: CommandItem[] = []) => {
  const [isOpen, setIsOpen] = useState(false);
  const [commands, setCommands] = useState<CommandItem[]>(initialCommands);
  
  const registerCommand = useCallback((command: CommandItem) => {
    setCommands(prev => {
      // Don't add duplicate commands (based on ID)
      if (prev.some(cmd => cmd.id === command.id)) {
        return prev;
      }
      return [...prev, command];
    });
    
    // Return unregister function
    return () => {
      setCommands(prev => prev.filter(cmd => cmd.id !== command.id));
    };
  }, []);
  
  const registerCommands = useCallback((newCommands: CommandItem[]) => {
    setCommands(prev => {
      const uniqueCommands = newCommands.filter(
        newCmd => !prev.some(cmd => cmd.id === newCmd.id)
      );
      return [...prev, ...uniqueCommands];
    });
    
    // Return unregister function
    return () => {
      const ids = newCommands.map(cmd => cmd.id);
      setCommands(prev => prev.filter(cmd => !ids.includes(cmd.id)));
    };
  }, []);
  
  const open = useCallback(() => {
    setIsOpen(true);
  }, []);
  
  const close = useCallback(() => {
    setIsOpen(false);
  }, []);
  
  // Add global keyboard shortcut to open the command palette (Cmd+K or Ctrl+K)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsOpen(prev => !prev);
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    
    return () => {
      window.addEventListener('keydown', handleKeyDown);
    };
  }, []);
  
  return {
    isOpen,
    commands,
    registerCommand,
    registerCommands,
    open,
    close,
  };
};

export default useCommandPalette;
