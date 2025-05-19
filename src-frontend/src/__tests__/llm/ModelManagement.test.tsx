import React from 'react';
import { render, screen, act, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ModelManagement from '../../components/llm/ModelManagement';

// Mock the modelManager module
jest.mock('../../api/EnhancedModelManager', () => ({
  __esModule: true,
  default: {
    getProviders: jest.fn(),
    getActiveProvider: jest.fn(),
    getAllModels: jest.fn(),
    getStorageInfo: jest.fn(),
    subscribeToModelEvents: jest.fn(),
    subscribeToDownloadEvents: jest.fn(),
    setActiveProvider: jest.fn(),
    downloadModel: jest.fn(),
    cancelDownload: jest.fn(),
    deleteModel: jest.fn(),
    importModel: jest.fn(),
    cleanupUnusedModels: jest.fn()
  }
}));

// Import after mocking
import modelManager from '../../api/EnhancedModelManager';

describe('ModelManagement Component', () => {
  // Mock data
  const mockProviders = [
    {
      id: 'ollama',
      name: 'Ollama',
      description: 'Run open-source LLMs locally',
      version: '0.1.0',
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: true,
        imageGeneration: false
      },
      status: 'active',
      requiresApiKey: false,
      supportsLocalModels: true
    }
  ];

  const mockModels = [
    {
      id: 'model1',
      name: 'Test Model 1',
      description: 'Test model description',
      architecture: 'Llama',
      format: 'GGUF',
      parameterCount: 7000000000,
      contextLength: 4096,
      quantization: 'Q4_0',
      sizeMb: 3900,
      provider: 'ollama',
      isInstalled: true,
      isLoaded: false,
      license: 'Test License',
      tags: ['test', 'model'],
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: false,
        imageGeneration: false
      },
      metadata: {}
    },
    {
      id: 'model2',
      name: 'Test Model 2',
      description: 'Another test model',
      architecture: 'Mistral',
      format: 'GGUF',
      parameterCount: 7000000000,
      contextLength: 8192,
      quantization: 'Q4_K_M',
      sizeMb: 4200,
      provider: 'ollama',
      isInstalled: false,
      isLoaded: false,
      license: 'Test License',
      tags: ['test', 'model', 'mistral'],
      capabilities: {
        textGeneration: true,
        chat: true,
        embeddings: false,
        imageGeneration: false
      },
      metadata: {}
    }
  ];

  const mockStorageInfo = {
    totalBytes: 100000000000,
    usedBytes: 35000000000,
    availableBytes: 65000000000,
    maxAllowedBytes: 100000000000,
    percentUsed: 35,
    modelCount: 1
  };

  // Setup default mocks for each test
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Default mock return values
    (modelManager.getProviders as jest.Mock).mockResolvedValue(mockProviders);
    (modelManager.getActiveProvider as jest.Mock).mockResolvedValue(mockProviders[0]);
    (modelManager.getAllModels as jest.Mock).mockResolvedValue(mockModels);
    (modelManager.getStorageInfo as jest.Mock).mockResolvedValue(mockStorageInfo);
    (modelManager.subscribeToModelEvents as jest.Mock).mockResolvedValue(jest.fn());
    (modelManager.subscribeToDownloadEvents as jest.Mock).mockResolvedValue(jest.fn());
  });

  test('renders component and loads initial data', async () => {
    render(<ModelManagement />);
    
    // Verify loading state
    expect(screen.getByText(/Loading models.../i)).toBeInTheDocument();
    
    // Verify calls to model manager
    await waitFor(() => {
      expect(modelManager.getProviders).toHaveBeenCalled();
      expect(modelManager.getActiveProvider).toHaveBeenCalled();
      expect(modelManager.getAllModels).toHaveBeenCalled();
      expect(modelManager.getStorageInfo).toHaveBeenCalled();
      expect(modelManager.subscribeToModelEvents).toHaveBeenCalled();
      expect(modelManager.subscribeToDownloadEvents).toHaveBeenCalled();
    });
  });

  test('cleanup checks for memory leak', async () => {
    const unsubscribeMock = jest.fn();
    (modelManager.subscribeToModelEvents as jest.Mock).mockResolvedValue(unsubscribeMock);
    (modelManager.subscribeToDownloadEvents as jest.Mock).mockResolvedValue(unsubscribeMock);
    
    const { unmount } = render(<ModelManagement />);
    
    // Wait for subscriptions to be set up
    await waitFor(() => {
      expect(modelManager.subscribeToModelEvents).toHaveBeenCalled();
      expect(modelManager.subscribeToDownloadEvents).toHaveBeenCalled();
    });
    
    // Unmount component to trigger cleanup
    unmount();
    
    // Verify unsubscribe was called - this helps confirm our isMountedRef is working
    expect(unsubscribeMock).toHaveBeenCalled();
  });
});