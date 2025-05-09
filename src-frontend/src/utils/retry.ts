/**
 * Executes a function with retry logic for handling transient failures.
 * 
 * @param operation The function to execute with retries
 * @param options Configuration options for the retry mechanism
 * @returns A promise that resolves with the result of the operation
 */
export async function withRetry<T>(
  operation: () => Promise<T>,
  options: {
    maxRetries?: number;
    retryDelay?: number;
    backoffFactor?: number;
    retryCondition?: (error: any) => boolean;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    retryDelay = 500,
    backoffFactor = 2,
    retryCondition = () => true,
  } = options;
  
  let retries = 0;
  let delay = retryDelay;
  
  while (true) {
    try {
      return await operation();
    } catch (error) {
      // Check if we've exceeded the maximum number of retries
      if (retries >= maxRetries || !retryCondition(error)) {
        throw error;
      }
      
      // Increment the retry count
      retries++;
      
      // Log the retry
      console.warn(`Operation failed, retrying (${retries}/${maxRetries})...`, error);
      
      // Wait before retrying
      await new Promise(resolve => setTimeout(resolve, delay));
      
      // Increase the delay for the next retry using exponential backoff
      delay *= backoffFactor;
    }
  }
}

/**
 * Creates a version of a function that includes retry logic.
 * 
 * @param fn The function to wrap with retry logic
 * @param options Configuration options for the retry mechanism
 * @returns A new function that includes retry logic
 */
export function createRetryFunction<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  options: {
    maxRetries?: number;
    retryDelay?: number;
    backoffFactor?: number;
    retryCondition?: (error: any) => boolean;
  } = {}
): T {
  return ((...args: Parameters<T>) => {
    return withRetry(() => fn(...args), options);
  }) as T;
}
