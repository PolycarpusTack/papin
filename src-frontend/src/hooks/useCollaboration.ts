// useCollaboration.ts
//
// This hook provides access to the collaboration context,
// making it easy to use collaboration features throughout the application.
// It also adds optimizations like batching and retry logic.

import { useContext, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { CollaborationContext } from '../components/collaboration/context/CollaborationContext';
import { createRetryFunction } from '../utils/retry';
import { createBatcher } from '../utils/batch';
import { throttle } from '../utils/throttle';

// Create retry-enabled versions of invoke for critical operations
const invokeWithRetry = createRetryFunction(invoke, {
  maxRetries: 3,
  retryDelay: 500,
  backoffFactor: 1.5,
  // Only retry on network-related errors, not validation errors
  retryCondition: (error) => {
    const errorStr = String(error).toLowerCase();
    return (
      errorStr.includes('network') ||
      errorStr.includes('timeout') ||
      errorStr.includes('connection') ||
      errorStr.includes('server')
    );
  },
});

// Enhanced useCollaboration hook with optimizations
export const useCollaboration = () => {
  const context = useContext(CollaborationContext);
  
  if (!context) {
    throw new Error('useCollaboration must be used within a CollaborationProvider');
  }
  
  // Create optimized versions of critical functions
  
  // Throttled cursor position updates
  const throttledUpdateCursor = useCallback(
    throttle(context.updateCursorPosition, 50), // 50ms throttle
    [context.updateCursorPosition]
  );
  
  // Setup a batch processor for selection updates
  const batchedSelectionUpdater = useCallback(() => {
    // Create a function to process batches of selection updates
    const processSelectionBatch = async (selectionUpdates: any[]) => {
      // Group updates by selection range (startId/endId/etc)
      // and only process the latest update for each range
      const latestUpdates = new Map();
      
      for (const update of selectionUpdates) {
        const key = `${update.startId}-${update.endId}`;
        latestUpdates.set(key, update);
      }
      
      // Process each unique update
      for (const update of latestUpdates.values()) {
        try {
          await context.updateSelection(
            update.startId,
            update.endId,
            update.startOffset,
            update.endOffset
          );
        } catch (error) {
          console.error('Error updating selection:', error);
        }
      }
    };
    
    // Create a batcher that will group selection updates
    return createBatcher(processSelectionBatch, {
      maxBatchSize: 5,
      maxDelayMs: 100,
    });
  }, [context.updateSelection]);
  
  // Use retry for critical operations
  const createSessionWithRetry = useCallback(
    async (name: string, conversationId: string) => {
      return invokeWithRetry('create_session', { name, conversationId })
        .then((session) => {
          context.refreshUsers();
          return session;
        });
    },
    [context.refreshUsers]
  );
  
  const joinSessionWithRetry = useCallback(
    async (sessionId: string) => {
      return invokeWithRetry('join_session', { sessionId })
        .then((session) => {
          context.refreshUsers();
          return session;
        });
    },
    [context.refreshUsers]
  );
  
  // Enhanced context with optimized functions
  const enhancedContext = {
    ...context,
    updateCursorPosition: throttledUpdateCursor,
    batchUpdateSelection: batchedSelectionUpdater(),
    createSessionWithRetry,
    joinSessionWithRetry,
  };
  
  return enhancedContext;
};

export default useCollaboration;
