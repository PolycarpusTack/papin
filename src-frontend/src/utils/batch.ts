/**
 * Creates a function that batches multiple calls into one and processes them together.
 * 
 * @param processBatch The function to process the batch of items
 * @param options Configuration options
 * @returns A function that can be called to add items to the batch
 */
export function createBatcher<T>(
  processBatch: (items: T[]) => Promise<void>,
  options: {
    maxBatchSize?: number;
    maxDelayMs?: number;
  } = {}
): (item: T) => void {
  const { maxBatchSize = 10, maxDelayMs = 100 } = options;
  
  let batch: T[] = [];
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  
  // Process the current batch
  const flush = async () => {
    if (batch.length === 0) return;
    
    const itemsToProcess = [...batch];
    batch = [];
    timeoutId = null;
    
    try {
      await processBatch(itemsToProcess);
    } catch (error) {
      console.error('Error processing batch:', error);
    }
  };
  
  // Return a function that adds items to the batch
  return (item: T) => {
    batch.push(item);
    
    // If we've reached the max batch size, flush immediately
    if (batch.length >= maxBatchSize) {
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeoutId = null;
      }
      flush();
      return;
    }
    
    // Otherwise, set a timeout to flush after the max delay
    if (!timeoutId) {
      timeoutId = setTimeout(flush, maxDelayMs);
    }
  };
}
