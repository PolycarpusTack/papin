/**
 * Creates a throttled function that only invokes func at most once per
 * every wait milliseconds.
 *
 * @param func The function to throttle
 * @param wait The number of milliseconds to throttle invocations to
 * @returns The throttled function
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let lastCallTime = 0;
  let lastArgs: Parameters<T> | null = null;
  let timeout: ReturnType<typeof setTimeout> | null = null;
  
  return function throttled(...args: Parameters<T>): void {
    const now = Date.now();
    const timeSinceLastCall = now - lastCallTime;
    
    // Update the last args for deferred execution
    lastArgs = args;
    
    // If enough time has elapsed, call the function immediately
    if (timeSinceLastCall >= wait) {
      lastCallTime = now;
      func(...args);
      return;
    }
    
    // Otherwise, schedule a deferred execution
    if (timeout === null) {
      timeout = setTimeout(() => {
        if (lastArgs !== null) {
          lastCallTime = Date.now();
          func(...lastArgs);
          lastArgs = null;
          timeout = null;
        }
      }, wait - timeSinceLastCall);
    }
  };
}
