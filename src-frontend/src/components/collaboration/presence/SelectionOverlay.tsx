// SelectionOverlay.tsx
//
// This component renders remote user selections in the text editor.
// It displays a colored highlight over text that other users have selected.

import React, { useEffect, useState } from 'react';
import { useCollaboration } from '../../../hooks/useCollaboration';
import { Selection, User } from '../context/CollaborationContext';

interface SelectionOverlayProps {
  editorRef: React.RefObject<HTMLElement>;
}

// Helper to get DOM range from selection data
const getSelectionRange = (
  selection: Selection, 
  container: HTMLElement
): Range | null => {
  try {
    // Find start and end elements by ID
    const startElement = container.querySelector(`#${selection.start_id}`);
    const endElement = container.querySelector(`#${selection.end_id}`);
    
    if (!startElement || !endElement) {
      return null;
    }
    
    const range = document.createRange();
    
    // Set start position
    if (startElement.childNodes.length > 0) {
      range.setStart(startElement.childNodes[0], selection.start_offset);
    } else {
      range.setStart(startElement, selection.start_offset);
    }
    
    // Set end position
    if (endElement.childNodes.length > 0) {
      range.setEnd(endElement.childNodes[0], selection.end_offset);
    } else {
      range.setEnd(endElement, selection.end_offset);
    }
    
    return range;
  } catch (error) {
    console.error('Error creating selection range:', error);
    return null;
  }
};

const SelectionOverlay: React.FC<SelectionOverlayProps> = ({ editorRef }) => {
  const { state, updateSelection } = useCollaboration();
  const { selections, users, config } = state;
  const [userMap, setUserMap] = useState<Record<string, User>>({});
  const [selectionElements, setSelectionElements] = useState<HTMLElement[]>([]);
  
  // Create a map of users by ID for easy lookup
  useEffect(() => {
    const map: Record<string, User> = {};
    users.forEach(user => {
      map[user.id] = user;
    });
    setUserMap(map);
  }, [users]);
  
  // Track local text selection to send updates
  useEffect(() => {
    if (!editorRef.current || !config.show_presence) return;
    
    const container = editorRef.current;
    
    const handleSelectionChange = () => {
      const selection = document.getSelection();
      if (!selection || selection.isCollapsed || !selection.rangeCount) return;
      
      const range = selection.getRangeAt(0);
      const startContainer = range.startContainer.parentElement;
      const endContainer = range.endContainer.parentElement;
      
      if (!startContainer || !endContainer) return;
      
      // Get element IDs for the selection endpoints
      const startId = startContainer.id || startContainer.parentElement?.id;
      const endId = endContainer.id || endContainer.parentElement?.id;
      
      if (!startId || !endId) return;
      
      // Send selection update
      updateSelection(
        startId,
        endId,
        range.startOffset,
        range.endOffset
      );
    };
    
    document.addEventListener('selectionchange', handleSelectionChange);
    
    return () => {
      document.removeEventListener('selectionchange', handleSelectionChange);
    };
  }, [editorRef, config.show_presence, updateSelection]);
  
  // Render remote selections when they change
  useEffect(() => {
    if (!editorRef.current || !config.show_presence) {
      // Clean up any existing highlights
      selectionElements.forEach(el => el.remove());
      setSelectionElements([]);
      return;
    }
    
    const container = editorRef.current;
    
    // Clean up existing highlights
    selectionElements.forEach(el => el.remove());
    
    // Create new highlights for each selection
    const newElements: HTMLElement[] = [];
    
    Object.entries(selections).forEach(([userId, selection]) => {
      const user = userMap[userId];
      if (!user) return;
      
      // Selection was updated in the last 30 seconds (not stale)
      const isFresh = new Date(selection.timestamp).getTime() > (Date.now() - 30000);
      if (!isFresh) return;
      
      const range = getSelectionRange(selection, container);
      if (!range) return;
      
      // Create highlight elements
      const rects = range.getClientRects();
      for (let i = 0; i < rects.length; i++) {
        const rect = rects[i];
        const highlight = document.createElement('div');
        
        // Position the highlight
        highlight.style.position = 'absolute';
        highlight.style.left = `${rect.left}px`;
        highlight.style.top = `${rect.top}px`;
        highlight.style.width = `${rect.width}px`;
        highlight.style.height = `${rect.height}px`;
        highlight.style.backgroundColor = `${user.color}33`; // 20% opacity
        highlight.style.pointerEvents = 'none';
        highlight.style.zIndex = '999';
        highlight.dataset.userId = userId;
        highlight.className = 'selection-highlight';
        
        document.body.appendChild(highlight);
        newElements.push(highlight);
      }
    });
    
    setSelectionElements(newElements);
    
    // Clean up on unmount
    return () => {
      newElements.forEach(el => el.remove());
    };
  }, [editorRef, selections, userMap, config.show_presence, selectionElements]);
  
  // This is a DOM manipulation component that doesn't render anything directly
  return null;
};

export default SelectionOverlay;
