import React, { lazy, Suspense, ComponentType } from 'react';

interface LazyLoadOptions {
  fallback?: React.ReactNode;
  errorComponent?: React.ComponentType<{ error: Error }>;
}

// Default error boundary component
const DefaultErrorComponent: React.FC<{ error: Error }> = ({ error }) => (
  <div className="error-boundary">
    <h3>Something went wrong loading this component</h3>
    <p>{error.message}</p>
    <button onClick={() => window.location.reload()}>
      Reload Application
    </button>
  </div>
);

// Default loading fallback
const DefaultFallback = (
  <div className="loading-component">
    <div className="loading-spinner"></div>
    <p>Loading component...</p>
  </div>
);

// Error boundary component
class ErrorBoundary extends React.Component<
  { children: React.ReactNode; errorComponent: React.ComponentType<{ error: Error }> },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: React.ReactNode; errorComponent: React.ComponentType<{ error: Error }> }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError && this.state.error) {
      const ErrorComponent = this.props.errorComponent;
      return <ErrorComponent error={this.state.error} />;
    }

    return this.props.children;
  }
}

// Function to create a lazy-loaded component with error boundary
export function lazyLoad<T extends ComponentType<any>>(
  importFunc: () => Promise<{ default: T }>,
  options: LazyLoadOptions = {}
) {
  const LazyComponent = lazy(importFunc);
  const fallback = options.fallback || DefaultFallback;
  const ErrorComponent = options.errorComponent || DefaultErrorComponent;

  return (props: React.ComponentProps<T>) => (
    <ErrorBoundary errorComponent={ErrorComponent}>
      <Suspense fallback={fallback}>
        <LazyComponent {...props} />
      </Suspense>
    </ErrorBoundary>
  );
}

export default lazyLoad;
