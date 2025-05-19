import React, { useState } from 'react';
import { ModelFilter as FilterOptions, ModelArchitecture, ModelFormat, QuantizationType } from '../../api/EnhancedModelManager';
import './ModelFilter.css';

interface ModelFilterComponentProps {
  onFilterChange: (filter: FilterOptions) => void;
  initialFilter?: FilterOptions;
  showProviderFilter?: boolean;
  providers?: {id: string, name: string}[];
}

/**
 * ModelFilterComponent component for filtering model lists
 */
const ModelFilterComponent: React.FC<ModelFilterComponentProps> = ({
  onFilterChange,
  initialFilter = {},
  showProviderFilter = true,
  providers = []
}) => {
  const [filter, setFilter] = useState<FilterOptions>(initialFilter);
  const [showAdvanced, setShowAdvanced] = useState(false);
  
  // Available architectures
  const architectures: ModelArchitecture[] = [
    'Llama', 'Mistral', 'Falcon', 'GPT-J', 'MPT', 
    'Phi', 'Pythia', 'Cerebras', 'Claude', 'GPT', 
    'PaLM', 'Gemma', 'BERT', 'Other'
  ];
  
  // Available formats
  const formats: ModelFormat[] = [
    'GGUF', 'GGML', 'ONNX', 'PyTorch', 'TensorFlow', 
    'Safetensors', 'Other'
  ];
  
  // Available quantization types
  const quantizationTypes: QuantizationType[] = [
    'None', 'Q4_K_M', 'Q4_0', 'Q4_1', 'Q5_K_M', 
    'Q5_0', 'Q5_1', 'Q8_0', 'Q8_1', 'Int8', 
    'Int4', 'Other'
  ];
  
  // Update filter and notify parent
  const updateFilter = (newFilter: Partial<FilterOptions>) => {
    const updatedFilter = { ...filter, ...newFilter };
    setFilter(updatedFilter);
    onFilterChange(updatedFilter);
  };
  
  // Handle search input
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateFilter({ searchQuery: e.target.value });
  };
  
  // Handle provider select
  const handleProviderChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    updateFilter({ providerId: e.target.value || undefined });
  };
  
  // Handle installed filter toggle
  const handleInstalledChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateFilter({ isInstalled: e.target.checked || undefined });
  };
  
  // Handle capability filter toggle
  const handleCapabilityChange = (capability: keyof FilterOptions['capabilities'], value: boolean) => {
    const currentCapabilities = filter.capabilities || {};
    updateFilter({
      capabilities: {
        ...currentCapabilities,
        [capability]: value || undefined
      }
    });
  };
  
  // Handle architecture filter
  const handleArchitectureChange = (architecture: ModelArchitecture, checked: boolean) => {
    const currentArchitectures = filter.architecture || [];
    let newArchitectures: ModelArchitecture[];
    
    if (checked) {
      newArchitectures = [...currentArchitectures, architecture];
    } else {
      newArchitectures = currentArchitectures.filter(a => a !== architecture);
    }
    
    updateFilter({ architecture: newArchitectures.length > 0 ? newArchitectures : undefined });
  };
  
  // Handle format filter
  const handleFormatChange = (format: ModelFormat, checked: boolean) => {
    const currentFormats = filter.format || [];
    let newFormats: ModelFormat[];
    
    if (checked) {
      newFormats = [...currentFormats, format];
    } else {
      newFormats = currentFormats.filter(f => f !== format);
    }
    
    updateFilter({ format: newFormats.length > 0 ? newFormats : undefined });
  };
  
  // Handle quantization filter
  const handleQuantizationChange = (quantization: QuantizationType, checked: boolean) => {
    const currentQuantization = filter.quantization || [];
    let newQuantization: QuantizationType[];
    
    if (checked) {
      newQuantization = [...currentQuantization, quantization];
    } else {
      newQuantization = currentQuantization.filter(q => q !== quantization);
    }
    
    updateFilter({ quantization: newQuantization.length > 0 ? newQuantization : undefined });
  };
  
  // Handle size filter
  const handleMaxSizeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
    updateFilter({ maxSizeMb: value });
  };
  
  // Handle context length filter
  const handleMinContextChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
    updateFilter({ minContextLength: value });
  };
  
  // Reset all filters
  const resetFilters = () => {
    const emptyFilter: FilterOptions = {};
    setFilter(emptyFilter);
    onFilterChange(emptyFilter);
  };
  
  return (
    <div className="model-filter">
      <div className="filter-basic">
        {/* Search */}
        <div className="filter-search">
          <input
            type="text"
            placeholder="Search models..."
            value={filter.searchQuery || ''}
            onChange={handleSearchChange}
            className="search-input"
          />
        </div>
        
        {/* Basic filters row */}
        <div className="filter-row">
          {/* Provider filter */}
          {showProviderFilter && providers.length > 0 && (
            <div className="filter-group">
              <label htmlFor="provider-select">Provider</label>
              <select 
                id="provider-select"
                value={filter.providerId || ''} 
                onChange={handleProviderChange}
                className="filter-select"
              >
                <option value="">All Providers</option>
                {providers.map(provider => (
                  <option key={provider.id} value={provider.id}>
                    {provider.name}
                  </option>
                ))}
              </select>
            </div>
          )}
          
          {/* Installed filter */}
          <div className="filter-check">
            <label className="checkbox-label">
              <input 
                type="checkbox" 
                checked={filter.isInstalled || false}
                onChange={handleInstalledChange}
              />
              <span>Installed Only</span>
            </label>
          </div>
          
          {/* Capabilities filters */}
          <div className="filter-capabilities">
            <label>Capabilities:</label>
            <div className="capability-checks">
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={filter.capabilities?.textGeneration || false}
                  onChange={(e) => handleCapabilityChange('textGeneration', e.target.checked)}
                />
                <span>Text</span>
              </label>
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={filter.capabilities?.chat || false}
                  onChange={(e) => handleCapabilityChange('chat', e.target.checked)}
                />
                <span>Chat</span>
              </label>
              <label className="checkbox-label">
                <input 
                  type="checkbox" 
                  checked={filter.capabilities?.embeddings || false}
                  onChange={(e) => handleCapabilityChange('embeddings', e.target.checked)}
                />
                <span>Embeddings</span>
              </label>
            </div>
          </div>
          
          {/* Advanced toggle */}
          <button 
            className="advanced-toggle" 
            onClick={() => setShowAdvanced(!showAdvanced)}
            type="button"
            aria-expanded={showAdvanced}
            aria-controls="advanced-filters"
          >
            {showAdvanced ? 'Hide Advanced' : 'Show Advanced'}
          </button>
        </div>
      </div>
      
      {/* Advanced filters */}
      {showAdvanced && (
        <div id="advanced-filters" className="filter-advanced">
          <div className="filter-row">
            {/* Size filter */}
            <div className="filter-group">
              <label htmlFor="size-select">Max Size</label>
              <select 
                id="size-select"
                value={filter.maxSizeMb?.toString() || ''} 
                onChange={handleMaxSizeChange}
                className="filter-select"
              >
                <option value="">Any Size</option>
                <option value="1000">1 GB</option>
                <option value="3000">3 GB</option>
                <option value="5000">5 GB</option>
                <option value="10000">10 GB</option>
                <option value="20000">20 GB</option>
              </select>
            </div>
            
            {/* Context length filter */}
            <div className="filter-group">
              <label htmlFor="context-select">Min Context</label>
              <select 
                id="context-select"
                value={filter.minContextLength?.toString() || ''} 
                onChange={handleMinContextChange}
                className="filter-select"
              >
                <option value="">Any Context</option>
                <option value="2048">2K tokens</option>
                <option value="4096">4K tokens</option>
                <option value="8192">8K tokens</option>
                <option value="16384">16K tokens</option>
                <option value="32768">32K tokens</option>
                <option value="65536">64K tokens</option>
                <option value="131072">128K tokens</option>
              </select>
            </div>
          </div>
          
          <div className="filter-sections">
            {/* Architecture filter */}
            <div className="filter-section">
              <h4>Architectures</h4>
              <div className="filter-checkbox-group">
                {architectures.map(architecture => (
                  <label key={architecture} className="checkbox-label">
                    <input 
                      type="checkbox" 
                      checked={(filter.architecture || []).includes(architecture)}
                      onChange={(e) => handleArchitectureChange(architecture, e.target.checked)}
                    />
                    <span>{architecture}</span>
                  </label>
                ))}
              </div>
            </div>
            
            {/* Format filter */}
            <div className="filter-section">
              <h4>Formats</h4>
              <div className="filter-checkbox-group">
                {formats.map(format => (
                  <label key={format} className="checkbox-label">
                    <input 
                      type="checkbox" 
                      checked={(filter.format || []).includes(format)}
                      onChange={(e) => handleFormatChange(format, e.target.checked)}
                    />
                    <span>{format}</span>
                  </label>
                ))}
              </div>
            </div>
            
            {/* Quantization filter */}
            <div className="filter-section">
              <h4>Quantization</h4>
              <div className="filter-checkbox-group">
                {quantizationTypes.map(quantization => (
                  <label key={quantization} className="checkbox-label">
                    <input 
                      type="checkbox" 
                      checked={(filter.quantization || []).includes(quantization)}
                      onChange={(e) => handleQuantizationChange(quantization, e.target.checked)}
                    />
                    <span>{quantization}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>
          
          <div className="filter-actions">
            <button 
              className="filter-reset" 
              onClick={resetFilters}
              type="button"
            >
              Reset Filters
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default ModelFilterComponent;