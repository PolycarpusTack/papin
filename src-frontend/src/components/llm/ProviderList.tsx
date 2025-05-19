import React from 'react';
import { Provider } from '../../api/EnhancedModelManager';
import './ProviderList.css';

interface ProviderListProps {
  providers: Provider[];
  activeProviderId?: string;
  onSelectProvider: (providerId: string) => void;
  isLoading?: boolean;
}

/**
 * ProviderList component displays a list of LLM providers with their status
 */
const ProviderList: React.FC<ProviderListProps> = ({
  providers,
  activeProviderId,
  onSelectProvider,
  isLoading = false
}) => {
  // Provider logos mapping (fallback icons)
  const getProviderIcon = (providerId: string): React.ReactNode => {
    const defaultIcons: Record<string, string> = {
      'ollama': '🦙',
      'localai': '🤖',
      'llamacpp': '🔧',
      'openai': '📝',
      'anthropic': '👤',
      'mistral': '🌪️',
      'together': '🤝',
      'default': '🧠'
    };
    
    // Use matching icon or default
    const iconKey = Object.keys(defaultIcons).find(key => 
      providerId.toLowerCase().includes(key)
    ) || 'default';
    
    return defaultIcons[iconKey];
  };
  
  // Render loading state
  if (isLoading) {
    return (
      <div className="provider-list-loading">
        <div className="provider-list-spinner"></div>
      </div>
    );
  }
  
  // Render empty state
  if (providers.length === 0) {
    return (
      <div className="provider-list-empty">
        <p>No providers available</p>
      </div>
    );
  }
  
  return (
    <div className="provider-list">
      {providers.map(provider => (
        <div
          key={provider.id}
          className={`provider-item ${activeProviderId === provider.id ? 'active' : ''} ${provider.status}`}
          onClick={() => provider.status !== 'unavailable' && onSelectProvider(provider.id)}
        >
          <div className="provider-icon">
            {provider.logoUrl ? (
              <img src={provider.logoUrl} alt={provider.name} />
            ) : (
              getProviderIcon(provider.id)
            )}
          </div>
          <div className="provider-details">
            <div className="provider-name">{provider.name}</div>
            <div className="provider-status">
              {provider.status === 'active' && 'Active'}
              {provider.status === 'available' && 'Available'}
              {provider.status === 'unavailable' && 'Unavailable'}
            </div>
          </div>
          <div className="provider-indicator">
            <div className={`status-dot ${provider.status}`}></div>
          </div>
        </div>
      ))}
    </div>
  );
};

export default ProviderList;