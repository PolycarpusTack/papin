import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import styled from 'styled-components';
import { motion, AnimatePresence } from 'framer-motion';

// Types
interface LLMModel {
  id: string;
  name: string;
  size_mb: number;
  architecture: string;
  parameters: number;
  installed: boolean;
  family: string;
  quantization: string;
  format: string;
}

interface StorageInfo {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
}

interface ProviderInfo {
  name: string;
  status: 'active' | 'available' | 'unavailable';
  model_count: number;
  icon: React.ReactNode;
}

// Styled Components
const Container = styled.div`
  padding: 20px;
  background-color: #141414;
  border-radius: 10px;
  color: #f5f5f5;
  font-family: 'Netflix Sans', 'Helvetica Neue', Arial, sans-serif;
`;

const Header = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
`;

const Title = styled.h2`
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 10px;
`;

const Stats = styled.div`
  display: flex;
  gap: 16px;
`;

const StatItem = styled.div`
  background-color: rgba(255, 255, 255, 0.1);
  padding: 10px 16px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  min-width: 120px;
`;

const StatValue = styled.div`
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 4px;
`;

const StatLabel = styled.div`
  font-size: 0.8rem;
  color: #b3b3b3;
`;

const Grid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
  margin-top: 24px;
`;

const Card = styled.div`
  background: linear-gradient(135deg, #1f1f1f 0%, #0a0a0a 100%);
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  height: 100%;
`;

const CardHeader = styled.div`
  margin-bottom: 16px;
  position: relative;
`;

const CardTitle = styled.h3`
  font-size: 1.2rem;
  margin: 0 0 4px;
  display: flex;
  align-items: center;
  gap: 8px;
`;

const CardSubtitle = styled.div`
  color: #b3b3b3;
  font-size: 0.9rem;
`;

const Badge = styled.span<{ $type: 'success' | 'warning' | 'info' | 'error' }>`
  position: absolute;
  top: 0;
  right: 0;
  font-size: 0.7rem;
  padding: 3px 8px;
  border-radius: 4px;
  font-weight: 600;
  background-color: ${props => {
    switch(props.$type) {
      case 'success': return 'rgba(0, 200, 83, 0.2)';
      case 'warning': return 'rgba(255, 160, 0, 0.2)';
      case 'info': return 'rgba(0, 113, 235, 0.2)';
      case 'error': return 'rgba(229, 9, 20, 0.2)';
    }
  }};
  color: ${props => {
    switch(props.$type) {
      case 'success': return '#00C853';
      case 'warning': return '#FFA000';
      case 'info': return '#0071EB';
      case 'error': return '#E50914';
    }
  }};
`;

const ProgressBar = styled.div`
  height: 4px;
  background-color: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  margin: 8px 0;
  overflow: hidden;
`;

const ProgressFill = styled.div<{ $percentage: number; $color?: string }>`
  height: 100%;
  width: ${props => props.$percentage}%;
  background-color: ${props => props.$color || '#0071EB'};
  transition: width 0.3s ease;
`;

const ModelList = styled.div`
  margin-top: 12px;
`;

const ModelItem = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  &:last-child {
    border-bottom: none;
  }
`;

const ModelName = styled.span`
  font-size: 0.9rem;
`;

const ModelMeta = styled.span`
  font-size: 0.8rem;
  color: #b3b3b3;
`;

const Button = styled.button`
  background-color: rgba(255, 255, 255, 0.08);
  color: white;
  border: none;
  border-radius: 4px;
  padding: 6px 12px;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
  
  &:hover {
    background-color: rgba(255, 255, 255, 0.15);
  }
`;

const ActionButton = styled(Button)<{ $primary?: boolean }>`
  background-color: ${props => props.$primary ? '#E50914' : 'rgba(255, 255, 255, 0.08)'};
  
  &:hover {
    background-color: ${props => props.$primary ? '#F40612' : 'rgba(255, 255, 255, 0.15)'};
  }
`;

// Icons
const ModelIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M12 2L2 7L12 12L22 7L12 2Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M2 17L12 22L22 17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M2 12L12 17L22 12" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

const HardDriveIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M22 12H2" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <rect x="2" y="6" width="20" height="12" rx="2" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M6 12V16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M10 12V16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

const ServerIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="2" y="2" width="20" height="8" rx="2" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <rect x="2" y="14" width="20" height="8" rx="2" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M6 6H6.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M6 18H6.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

const OllamaIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9 9-4.03 9-9-4.03-9-9-9zm0 2c3.86 0 7 3.14 7 7s-3.14 7-7 7-7-3.14-7-7 3.14-7 7-7zm0 2c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zm0 2c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3z"/>
  </svg>
);

const LocalAIIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" strokeWidth="2"/>
    <path d="M3 10H21" stroke="currentColor" strokeWidth="2"/>
    <path d="M10 10V21" stroke="currentColor" strokeWidth="2"/>
    <path d="M8 7H8.01" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
  </svg>
);

const LlamaCppIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M5 6.2C5 5.07989 5 4.51984 5.21799 4.09202C5.40973 3.71569 5.71569 3.40973 6.09202 3.21799C6.51984 3 7.07989 3 8.2 3H15.8C16.9201 3 17.4802 3 17.908 3.21799C18.2843 3.40973 18.5903 3.71569 18.782 4.09202C19 4.51984 19 5.07989 19 6.2V21L12 17L5 21V6.2Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

const ArrowRightIcon = () => (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M5 12H19" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M12 5L19 12L12 19" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

// Format bytes to human readable string
const formatBytes = (bytes: number, decimals = 1) => {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

const ModelsOverview: React.FC = () => {
  const [models, setModels] = useState<LLMModel[]>([]);
  const [storageInfo, setStorageInfo] = useState<StorageInfo>({
    total_bytes: 0,
    used_bytes: 0,
    available_bytes: 0
  });
  const [providers, setProviders] = useState<ProviderInfo[]>([
    {
      name: 'Ollama',
      status: 'unavailable',
      model_count: 0,
      icon: <OllamaIcon />
    },
    {
      name: 'LocalAI',
      status: 'unavailable',
      model_count: 0,
      icon: <LocalAIIcon />
    },
    {
      name: 'llama.cpp',
      status: 'unavailable',
      model_count: 0,
      icon: <LlamaCppIcon />
    }
  ]);
  const [loading, setLoading] = useState(true);
  
  // Load data
  useEffect(() => {
    const loadData = async () => {
      try {
        setLoading(true);
        
        // Get installed models
        const installedModels: LLMModel[] = await invoke('get_all_models');
        setModels(installedModels);
        
        // Get storage info
        const storage: StorageInfo = await invoke('get_model_disk_space');
        setStorageInfo(storage);
        
        // Get provider information
        const providerStatus: Record<string, { available: boolean; model_count: number }> = 
          await invoke('get_provider_status');
        
        setProviders(prev => 
          prev.map(provider => ({
            ...provider,
            status: providerStatus[provider.name.toLowerCase()]?.available 
              ? (providerStatus[provider.name.toLowerCase()].model_count > 0 ? 'active' : 'available')
              : 'unavailable',
            model_count: providerStatus[provider.name.toLowerCase()]?.model_count || 0
          }))
        );
        
      } catch (error) {
        console.error('Failed to load model overview data:', error);
      } finally {
        setLoading(false);
      }
    };
    
    loadData();
  }, []);
  
  // Group models by architecture family
  const modelsByArchitecture = models.reduce((groups, model) => {
    const architecture = model.architecture || 'Unknown';
    if (!groups[architecture]) {
      groups[architecture] = [];
    }
    groups[architecture].push(model);
    return groups;
  }, {} as Record<string, LLMModel[]>);
  
  // Get model counts
  const installedCount = models.filter(m => m.installed).length;
  const totalModelsCount = models.length;
  
  return (
    <Container>
      <Header>
        <Title>
          <ModelIcon /> LLM Models Overview
        </Title>
        <Button>Manage Models <ArrowRightIcon /></Button>
      </Header>
      
      <Stats>
        <StatItem>
          <StatValue>{installedCount}</StatValue>
          <StatLabel>Installed Models</StatLabel>
        </StatItem>
        
        <StatItem>
          <StatValue>{totalModelsCount}</StatValue>
          <StatLabel>Available Models</StatLabel>
        </StatItem>
        
        <StatItem>
          <StatValue>{providers.filter(p => p.status !== 'unavailable').length}</StatValue>
          <StatLabel>Active Providers</StatLabel>
        </StatItem>
        
        <StatItem>
          <StatValue>{formatBytes(storageInfo.used_bytes)}</StatValue>
          <StatLabel>Storage Used</StatLabel>
        </StatItem>
      </Stats>
      
      <div style={{ marginTop: 20 }}>
        <CardSubtitle>Storage Usage</CardSubtitle>
        <ProgressBar>
          <ProgressFill 
            $percentage={(storageInfo.used_bytes / storageInfo.total_bytes) * 100}
            $color={storageInfo.used_bytes / storageInfo.total_bytes > 0.9 ? '#E50914' : undefined}
          />
        </ProgressBar>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.8rem', color: '#b3b3b3' }}>
          <span>Used: {formatBytes(storageInfo.used_bytes)}</span>
          <span>Available: {formatBytes(storageInfo.available_bytes)}</span>
          <span>Total: {formatBytes(storageInfo.total_bytes)}</span>
        </div>
      </div>
      
      <Grid>
        {/* Provider Cards */}
        {providers.map(provider => (
          <Card key={provider.name}>
            <CardHeader>
              <CardTitle>
                {provider.icon} {provider.name}
              </CardTitle>
              <CardSubtitle>
                LLM Provider
              </CardSubtitle>
              <Badge $type={
                provider.status === 'active' ? 'success' : 
                provider.status === 'available' ? 'info' : 'error'
              }>
                {provider.status === 'active' ? 'Active' : 
                 provider.status === 'available' ? 'Available' : 'Unavailable'}
              </Badge>
            </CardHeader>
            
            {provider.status !== 'unavailable' ? (
              <>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 10 }}>
                  <span style={{ fontSize: '0.9rem' }}>Models</span>
                  <span style={{ fontSize: '0.9rem', fontWeight: 'bold' }}>{provider.model_count}</span>
                </div>
                
                <ModelList>
                  {models
                    .filter(model => 
                      // Different providers support different model formats
                      (provider.name === 'Ollama' && ['GGUF', 'GGML'].includes(model.format)) ||
                      (provider.name === 'LocalAI' && ['GGUF', 'GGML', 'ONNX'].includes(model.format)) ||
                      (provider.name === 'llama.cpp' && ['GGUF', 'GGML'].includes(model.format))
                    )
                    .slice(0, 3)
                    .map(model => (
                      <ModelItem key={model.id}>
                        <div>
                          <ModelName>{model.name}</ModelName>
                          <div>
                            <ModelMeta>
                              {model.architecture} • {(model.parameters / 1000000000).toFixed(1)}B
                            </ModelMeta>
                          </div>
                        </div>
                        <Badge $type={model.installed ? 'success' : 'info'}>
                          {model.installed ? 'Installed' : 'Available'}
                        </Badge>
                      </ModelItem>
                    ))
                  }
                </ModelList>
                
                <div style={{ marginTop: 16, textAlign: 'right' }}>
                  <Button>View All</Button>
                </div>
              </>
            ) : (
              <div style={{ 
                display: 'flex', 
                flexDirection: 'column', 
                alignItems: 'center', 
                justifyContent: 'center',
                padding: '20px 0',
                color: '#b3b3b3'
              }}>
                <ServerIcon />
                <p style={{ marginTop: 10, fontSize: '0.9rem' }}>Provider not detected</p>
                <ActionButton $primary style={{ marginTop: 10 }}>Install</ActionButton>
              </div>
            )}
          </Card>
        ))}
        
        {/* Model Architectures Cards */}
        {Object.entries(modelsByArchitecture).map(([architecture, archModels]) => (
          <Card key={architecture}>
            <CardHeader>
              <CardTitle>
                <ModelIcon /> {architecture}
              </CardTitle>
              <CardSubtitle>
                {archModels.length} model{archModels.length !== 1 ? 's' : ''}
              </CardSubtitle>
              <Badge $type="info">
                {archModels.filter(m => m.installed).length} Installed
              </Badge>
            </CardHeader>
            
            <ModelList>
              {archModels.slice(0, 3).map(model => (
                <ModelItem key={model.id}>
                  <div>
                    <ModelName>{model.name}</ModelName>
                    <div>
                      <ModelMeta>
                        {(model.parameters / 1000000000).toFixed(1)}B • {model.quantization}
                      </ModelMeta>
                    </div>
                  </div>
                  <Badge $type={model.installed ? 'success' : 'info'}>
                    {model.installed ? 'Installed' : 'Available'}
                  </Badge>
                </ModelItem>
              ))}
            </ModelList>
            
            {archModels.length > 3 && (
              <div style={{ marginTop: 16, textAlign: 'right' }}>
                <Button>View All {archModels.length} Models</Button>
              </div>
            )}
          </Card>
        ))}
        
        {/* Storage Card */}
        <Card>
          <CardHeader>
            <CardTitle>
              <HardDriveIcon /> Storage
            </CardTitle>
            <CardSubtitle>
              Model Storage Management
            </CardSubtitle>
          </CardHeader>
          
          <div style={{ marginTop: 10 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.9rem' }}>
              <span>Used Space</span>
              <span>{formatBytes(storageInfo.used_bytes)}</span>
            </div>
            <ProgressBar style={{ margin: '8px 0' }}>
              <ProgressFill 
                $percentage={(storageInfo.used_bytes / storageInfo.total_bytes) * 100}
                $color={storageInfo.used_bytes / storageInfo.total_bytes > 0.9 ? '#E50914' : undefined}
              />
            </ProgressBar>
            
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.9rem' }}>
              <span>Free Space</span>
              <span>{formatBytes(storageInfo.available_bytes)}</span>
            </div>
            <ProgressBar style={{ margin: '8px 0' }}>
              <ProgressFill 
                $percentage={(storageInfo.available_bytes / storageInfo.total_bytes) * 100}
                $color="#00C853"
              />
            </ProgressBar>
            
            <div style={{ marginTop: 16, display: 'flex', gap: 10 }}>
              <ActionButton>Optimize Storage</ActionButton>
              <Button>Settings</Button>
            </div>
          </div>
        </Card>
      </Grid>
    </Container>
  );
};

export default ModelsOverview;