# Model Management System

<!-- Table of Contents -->
<div class="toc">
  <ul>
    <li><a href="#overview">Overview</a></li>
    <li><a href="#key-features">Key Features</a></li>
    <li><a href="#getting-started">Getting Started</a></li>
    <li><a href="#advanced-features">Advanced Features</a></li>
    <li><a href="#technical-reference">Technical Reference</a></li>
    <li><a href="#troubleshooting">Troubleshooting</a></li>
    <li><a href="#best-practices">Best Practices</a></li>
    <li><a href="#faq">FAQ</a></li>
    <li><a href="#glossary">Glossary</a></li>
  </ul>
</div>

<span class="badge feature-new">New Feature</span>

## Overview {#overview}

The Model Management System in Papin provides a comprehensive solution for discovering, downloading, and managing local LLM models. It features a Netflix-inspired browsing interface, robust backend, and real-time updates for model operations.

<div class="screenshots">
  <img src="../assets/model-management-screenshot.png" alt="Model Management UI" />
  <div class="caption">The Netflix-inspired model browsing interface</div>
</div>

## Key Features

- **Discover Models**: Browse available models from different providers
- **Download Management**: Download models with real-time progress tracking
- **Model Organization**: Categorize and filter models by various properties
- **Disk Space Management**: Set limits and optimize storage usage
- **Real-time Updates**: Get instant feedback on model operations
- **Provider Integration**: Seamless integration with Ollama, LocalAI, and llama.cpp

## Getting Started

### Accessing Model Management

You can access the Model Management interface in three ways:

1. **GUI**: Open the main application and navigate to "Models" in the sidebar
2. **CLI**: Use `papin models list` or other model commands
3. **TUI**: Select "Model Management" from the main menu

### Basic Operations

#### Browsing Available Models

The main interface displays models in a Netflix-style card layout, organized by categories:

- **Recommended Models**: Tailored to your usage patterns
- **Recently Used**: Models you've used recently
- **Popular Models**: Most downloaded models
- **New Arrivals**: Recently added models
- **By Provider**: Models grouped by provider (Ollama, LocalAI, llama.cpp)

Hover over a model card to see additional details and action buttons.

#### Downloading a Model

To download a model:

1. Hover over the model card
2. Click the "Download" button
3. (Optional) Select any configuration options in the dialog
4. Click "Start Download"

A progress indicator will appear, showing:
- Download progress percentage
- Estimated time remaining
- Download speed
- Total size

You can cancel a download at any time by clicking the "Cancel" button.

#### Managing Downloaded Models

For models that are already downloaded:

- **Run**: Use the model for inference
- **Update**: Check for and download updates
- **Delete**: Remove the model from your device
- **Export**: Export the model to an external location
- **Settings**: Configure model-specific settings

### Advanced Features

#### Model Comparison

To compare multiple models:

1. Select the models you want to compare by clicking the checkbox on their cards
2. Click the "Compare" button in the toolbar
3. View side-by-side comparison of specifications and capabilities

#### Model Search and Filtering

Use the search bar to find specific models. Apply filters to narrow results by:

- Architecture (Llama, Mistral, etc.)
- Parameter size (7B, 13B, etc.)
- Quantization (4-bit, 5-bit, 8-bit)
- Capabilities (text generation, embeddings, vision, etc.)
- Provider compatibility

#### Disk Space Management

To manage disk space:

1. Go to Settings > Model Storage
2. Set maximum disk space allocation
3. Configure auto-cleanup policies:
   - Delete least recently used models
   - Keep favorite models
   - Minimum free space threshold

#### Model Registry Events

The system provides real-time notifications for various model events:

- Download started/completed/failed
- Model update available
- Low disk space warnings
- Model loading/unloading

## Technical Reference {#technical-reference}

<div class="tabbed-section">
  <div class="tabs">
    <button class="tab active" data-target="provider-system">Provider System</button>
    <button class="tab" data-target="model-registry">Model Registry</button>
    <button class="tab" data-target="model-formats">Model Formats</button>
    <button class="tab" data-target="api-reference">API Reference</button>
  </div>

  <div class="tab-content active" id="provider-system">
    <h3>Provider System</h3>

    <p>The Model Management System interfaces with multiple LLM providers, each with different capabilities and format support:</p>

    <table class="data-table">
      <thead>
        <tr>
          <th>Provider</th>
          <th>Description</th>
          <th>Format Support</th>
          <th>Pros</th>
          <th>Cons</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Ollama</td>
          <td>Local model runner</td>
          <td>GGUF, GGML</td>
          <td>
            <ul>
              <li>Easy to use API</li>
              <li>Built-in model registry</li>
              <li>Active community</li>
            </ul>
          </td>
          <td>
            <ul>
              <li>Less fine-grained control</li>
              <li>Higher memory usage</li>
            </ul>
          </td>
        </tr>
        <tr>
          <td>LocalAI</td>
          <td>OpenAI-compatible API</td>
          <td>GGUF, GGML, ONNX</td>
          <td>
            <ul>
              <li>OpenAI API compatibility</li>
              <li>Wide format support</li>
              <li>Multi-model serving</li>
            </ul>
          </td>
          <td>
            <ul>
              <li>More complex setup</li>
              <li>Requires separate installation</li>
            </ul>
          </td>
        </tr>
        <tr>
          <td>llama.cpp</td>
          <td>Direct inference</td>
          <td>GGUF, GGML</td>
          <td>
            <ul>
              <li>Maximum performance</li>
              <li>Fine-grained control</li>
              <li>Lower memory usage</li>
            </ul>
          </td>
          <td>
            <ul>
              <li>More technical to configure</li>
              <li>Fewer high-level features</li>
            </ul>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</div>

### Model Registry

The Model Registry is the core component that tracks all models and their metadata. It provides:

- Centralized model catalog
- Versioning support
- File management
- Event broadcasting
- Disk space optimization

### Model Formats

The system supports multiple model formats:

- **GGUF**: Latest format optimized for quantized models (recommended)
- **GGML**: Legacy format for older models
- **ONNX**: Standardized format for machine learning models
- **PyTorch**: Native PyTorch format (limited support)

### API Reference

For developers, the system exposes these key API endpoints:

#### Tauri Commands
```typescript
// Get all models
getAllModels(): Promise<ModelInfo[]>

// Get installed models
getInstalledModels(): Promise<ModelInfo[]>

// Download a model
downloadModel(modelId: string, url: string, provider: string): Promise<DownloadStatus>

// Get download status
getDownloadStatus(modelId: string): Promise<DownloadStatus>

// Register for events
registerModelRegistryEvents(callback: (event: ModelRegistryEvent) => void): Promise<() => void>
```

## Troubleshooting {#troubleshooting}

<div class="troubleshooting-wizard">
  <div class="problem-selector">
    <label for="issue-selector">What issue are you experiencing?</label>
    <select id="issue-selector" class="dropdown">
      <option value="download-fails">Download fails to start</option>
      <option value="download-interrupted">Download starts but fails later</option>
      <option value="model-wont-run">Model is downloaded but won't run</option>
      <option value="performance-issues">Performance issues with model</option>
      <option value="other">Other issue</option>
    </select>
    <button class="button secondary">Get Help</button>
  </div>
  
  <div class="diagnostic-tool">
    <span class="badge tool">Diagnostic Tool</span>
    <h4>Model System Diagnostics</h4>
    <p>Our built-in diagnostic tool can check your system and identify common issues.</p>
    <button class="button primary">Run Diagnostics</button>
  </div>
</div>

### Common Issues

<div class="issue-card">
  <div class="issue-header">
    <h4>Download Fails Immediately</h4>
    <span class="severity high">Common Issue</span>
  </div>
  <div class="issue-content">
    <p>When a download fails to start, this is typically related to connectivity or resource issues.</p>
    
    <div class="checklist">
      <div class="check-item">
        <input type="checkbox" id="check1" class="checkbox">
        <label for="check1">Verify your internet connection is working</label>
      </div>
      <div class="check-item">
        <input type="checkbox" id="check2" class="checkbox">
        <label for="check2">Check that the model URL is still valid</label>
      </div>
      <div class="check-item">
        <input type="checkbox" id="check3" class="checkbox">
        <label for="check3">Ensure you have sufficient disk space (<span class="diag-disk-free">4.2GB</span> free)</label>
      </div>
      <div class="check-item">
        <input type="checkbox" id="check4" class="checkbox">
        <label for="check4">Verify your firewall isn't blocking downloads</label>
      </div>
    </div>
    
    <div class="help-action">
      <button class="button link">Still having trouble? Contact support</button>
    </div>
  </div>
</div>

<div class="issue-card">
  <div class="issue-header">
    <h4>Download Starts But Fails Later</h4>
    <span class="severity medium">Occasional Issue</span>
  </div>
  <div class="issue-content">
    <p>Downloads that start but fail to complete are usually caused by connection instability or interruptions.</p>
    
    <ul class="solution-list">
      <li>Check for unstable internet connection</li>
      <li>Verify the download wasn't cancelled by another process</li>
      <li>Check if antivirus software is blocking the download</li>
      <li>Try using a wired connection instead of Wi-Fi</li>
      <li>Consider using the "Resume Download" feature for interrupted downloads</li>
    </ul>
    
    <div class="code-example">
      <div class="code-header">CLI Recovery Command</div>
      <code>papin models download --resume &lt;model-id&gt;</code>
    </div>
  </div>
</div>

<div class="issue-card">
  <div class="issue-header">
    <h4>Model Appears Downloaded But Won't Run</h4>
    <span class="severity medium">Occasional Issue</span>
  </div>
  <div class="issue-content">
    <p>If a model shows as downloaded but fails to run, it could be due to compatibility issues or incomplete files.</p>
    
    <ul class="solution-list">
      <li>Check if all model files are complete and not corrupted</li>
      <li>Verify you have the correct provider selected</li>
      <li>Check hardware compatibility with the model's requirements</li>
      <li>Try repairing the model through Settings > Model Management > Repair</li>
    </ul>
    
    <div class="terminal-example">
      <div class="terminal-header">Model Verification</div>
      <div class="terminal-output">
        $ papin models verify mistral-7b<br>
        Verifying model 'mistral-7b'...<br>
        Checking file integrity: ✓<br>
        Validating model format: ✓<br>
        Testing provider compatibility: ✗ Error: Model requires 16GB RAM, 8GB available<br>
        Recommendation: Try using a more quantized version of this model.
      </div>
    </div>
  </div>
</div>

<div class="issue-card">
  <div class="issue-header">
    <h4>Performance Issues With Model</h4>
    <span class="severity low">Common Issue</span>
  </div>
  <div class="issue-content">
    <p>Performance problems are typically related to hardware constraints or resource competition.</p>
    
    <ul class="solution-list">
      <li>Try a more quantized version (e.g., 4-bit instead of 8-bit)</li>
      <li>Check if your hardware meets the minimum requirements</li>
      <li>Close other resource-intensive applications</li>
      <li>Reduce the context length for faster responses</li>
      <li>Enable hardware acceleration if available</li>
    </ul>
    
    <div class="performance-tip">
      <div class="tip-header">Performance Tip</div>
      <p>For the best balance of quality and performance, 4-bit quantized models on GPU provide excellent results on most systems.</p>
    </div>
  </div>
</div>

### Error Messages

<table class="error-table">
  <thead>
    <tr>
      <th>Error Code</th>
      <th>Message</th>
      <th>Meaning</th>
      <th>Solution</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>ERR_DISK_SPACE</code></td>
      <td>"Insufficient disk space"</td>
      <td>Not enough storage for the model</td>
      <td>
        <button class="solution-btn">Free up space</button> or
        <button class="solution-btn">Select a smaller model</button>
      </td>
    </tr>
    <tr>
      <td><code>ERR_FORMAT</code></td>
      <td>"Model incompatible with provider"</td>
      <td>The model format isn't supported</td>
      <td>
        <button class="solution-btn">Choose different model</button> or
        <button class="solution-btn">Switch provider</button>
      </td>
    </tr>
    <tr>
      <td><code>ERR_TIMEOUT</code></td>
      <td>"Download timeout"</td>
      <td>Download took too long and was cancelled</td>
      <td>
        <button class="solution-btn">Check internet speed</button> or
        <button class="solution-btn">Try again</button>
      </td>
    </tr>
    <tr>
      <td><code>ERR_REGISTRY</code></td>
      <td>"Model registry error"</td>
      <td>Internal system error</td>
      <td>
        <button class="solution-btn">Restart application</button> or
        <button class="solution-btn">Contact support</button>
      </td>
    </tr>
  </tbody>
</table>

<div class="live-support">
  <h4>Still Having Issues?</h4>
  <p>Our team is available to help troubleshoot any problems you encounter.</p>
  <div class="support-options">
    <button class="button primary">Live Chat Support</button>
    <button class="button secondary">Submit Bug Report</button>
    <button class="button tertiary">View Knowledge Base</button>
  </div>
</div>

## Best Practices {#best-practices}

<div class="best-practices-container">
  <div class="practice-card">
    <div class="practice-icon">🏃‍♂️</div>
    <h3>Start Small</h3>
    <p>Begin with smaller models (1-7B parameters) before trying larger ones.</p>
    <div class="practice-details">
      <p>Smaller models are:</p>
      <ul>
        <li>Faster to download</li>
        <li>Easier to get running</li>
        <li>Great for learning how the system works</li>
      </ul>
      <div class="recommendation">
        <strong>Recommended first model:</strong> Gemma 2B or Phi-2
      </div>
    </div>
  </div>
  
  <div class="practice-card">
    <div class="practice-icon">⚡</div>
    <h3>Use Quantized Models</h3>
    <p>4-bit and 5-bit models offer excellent performance with smaller size.</p>
    <div class="practice-details">
      <p>Quantization reduces precision but offers massive benefits:</p>
      <table class="mini-table">
        <tr>
          <th>Quantization</th>
          <th>Size Reduction</th>
          <th>Memory Usage</th>
          <th>Speed</th>
          <th>Quality Loss</th>
        </tr>
        <tr>
          <td>F16 (none)</td>
          <td>0%</td>
          <td>High</td>
          <td>Slowest</td>
          <td>None</td>
        </tr>
        <tr>
          <td>8-bit</td>
          <td>~50%</td>
          <td>Medium</td>
          <td>Medium</td>
          <td>Minimal</td>
        </tr>
        <tr>
          <td>4-bit</td>
          <td>~75%</td>
          <td>Low</td>
          <td>Fast</td>
          <td>Noticeable</td>
        </tr>
      </table>
      <p>For most users, 4-bit quantized models offer the best balance.</p>
    </div>
  </div>
  
  <div class="practice-card">
    <div class="practice-icon">🗂️</div>
    <h3>Organize by Use Case</h3>
    <p>Keep specialized models for specific tasks instead of using one for everything.</p>
    <div class="practice-details">
      <p>Match models to tasks for best results:</p>
      <ul>
        <li><strong>Creative writing:</strong> Llama models</li>
        <li><strong>Programming tasks:</strong> Code-specialized models</li>
        <li><strong>Structured data:</strong> Function-calling models</li>
        <li><strong>Quick responses:</strong> Small, efficient models</li>
      </ul>
      <button class="button secondary mini">Create Task-Based Collections</button>
    </div>
  </div>
  
  <div class="practice-card">
    <div class="practice-icon">🧪</div>
    <h3>Test Different Providers</h3>
    <p>Providers may offer better performance for certain models.</p>
    <div class="practice-details">
      <div class="provider-tips">
        <div class="tip"><strong>Ollama:</strong> Best for ease of use and quick setup</div>
        <div class="tip"><strong>LocalAI:</strong> Best for OpenAI API compatibility</div>
        <div class="tip"><strong>llama.cpp:</strong> Best for maximum performance</div>
      </div>
      <div class="performance-comparison">
        <div class="chart">
          <div class="chart-bar" style="width: 85%;">llama.cpp</div>
          <div class="chart-bar" style="width: 70%;">LocalAI</div>
          <div class="chart-bar" style="width: 65%;">Ollama</div>
        </div>
        <div class="chart-caption">Relative performance benchmark (7B model)</div>
      </div>
    </div>
  </div>
  
  <div class="practice-card">
    <div class="practice-icon">🧹</div>
    <h3>Regular Cleanup</h3>
    <p>Remove unused models to save space and maintain performance.</p>
    <div class="practice-details">
      <p>Setup automatic cleanup rules:</p>
      <div class="cleanup-settings">
        <div class="setting">
          <input type="checkbox" id="cleanup1" checked>
          <label for="cleanup1">Auto-remove models unused for 60+ days</label>
        </div>
        <div class="setting">
          <input type="checkbox" id="cleanup2">
          <label for="cleanup2">Keep minimum of 10GB free space</label>
        </div>
        <div class="setting">
          <input type="checkbox" id="cleanup3" checked>
          <label for="cleanup3">Never auto-remove favorited models</label>
        </div>
      </div>
      <button class="button primary mini">Schedule Monthly Cleanup</button>
    </div>
  </div>
</div>

<div class="best-practices-wizard">
  <h3>Hardware-Optimized Recommendations</h3>
  <p>We can suggest the best practices based on your specific hardware.</p>
  
  <div class="wizard-form">
    <div class="form-group">
      <label>RAM Available:</label>
      <select class="dropdown">
        <option>Less than 8GB</option>
        <option>8-16GB</option>
        <option selected>16-32GB</option>
        <option>More than 32GB</option>
      </select>
    </div>
    
    <div class="form-group">
      <label>GPU Available:</label>
      <select class="dropdown">
        <option>None (CPU only)</option>
        <option>Integrated GPU</option>
        <option selected>NVIDIA GPU (4-8GB VRAM)</option>
        <option>NVIDIA GPU (8GB+ VRAM)</option>
        <option>AMD GPU</option>
      </select>
    </div>
    
    <button class="button primary">Get Custom Recommendations</button>
  </div>
</div>

## FAQ

**Q: How much disk space do models typically require?**

A: It varies widely:
- Small models (1-3B parameters): 500MB-2GB
- Medium models (7B parameters): 3-5GB
- Large models (13B+ parameters): 8-30GB+

Quantization can reduce these sizes substantially (e.g., a 4-bit quantized 7B model might be only 3-4GB).

**Q: Can I use my own custom models?**

A: Yes! Use the "Import Model" feature to add your own custom models. Ensure they're in a supported format (GGUF recommended).

**Q: How do I know which model is best for my needs?**

A: Check the model capabilities on the model detail page. Models have different strengths:

<div class="expandable-section">
  <div class="section-header">💻 Code-Specialized Models</div>
  <div class="section-content">
    <p>Optimized for programming tasks, these models excel at:</p>
    <ul>
      <li>Code completion and generation</li>
      <li>Debugging assistance</li>
      <li>Code explanation and documentation</li>
      <li>Language-specific knowledge</li>
    </ul>
    <p><strong>Popular examples:</strong> CodeLlama, WizardCoder, Deepseek Coder</p>
  </div>
</div>

<div class="expandable-section">
  <div class="section-header">🌐 Multilingual Models</div>
  <div class="section-content">
    <p>Support multiple languages beyond English, with capabilities for:</p>
    <ul>
      <li>Translation assistance</li>
      <li>Non-English content generation</li>
      <li>Cross-lingual understanding</li>
    </ul>
    <p><strong>Popular examples:</strong> BLOOM, XLM-RoBERTa variants</p>
  </div>
</div>

<div class="expandable-section">
  <div class="section-header">📊 Function-Calling Models</div>
  <div class="section-content">
    <p>Provide structured outputs and API integration capabilities:</p>
    <ul>
      <li>JSON/structured data generation</li>
      <li>Tool usage and external API calls</li>
      <li>Form-filling and data extraction</li>
    </ul>
    <p><strong>Popular examples:</strong> Mistral models with function calling, NexusRaven</p>
  </div>
</div>

<div class="expandable-section">
  <div class="section-header">🔍 Small but Efficient Models</div>
  <div class="section-content">
    <p>Smaller models that work well on modest hardware:</p>
    <ul>
      <li>Fast response times</li>
      <li>Lower memory requirements</li>
      <li>Often more focused on specific tasks</li>
    </ul>
    <p><strong>Popular examples:</strong> Phi-2, TinyLlama, Gemma 2B</p>
  </div>
</div>

<div class="model-wizard">
  <button class="button primary">Launch Model Selection Wizard</button>
  <p class="helper-text">Our interactive wizard can help you find the perfect model based on your hardware and needs.</p>
</div>

**Q: Can I run these models on modest hardware?**

A: Many models have quantized versions that run on modest hardware:
- 4-bit quantized 7B models can run on many laptops
- Lower context lengths require less memory
- CPU-only operation is possible but slower

## Glossary

| Term | Definition |
|------|------------|
| **Architecture** | The underlying neural network design (Llama, Mistral, etc.) |
| **Context Length** | The maximum number of tokens the model can consider at once |
| **GGUF** | Latest file format for quantized LLMs, successor to GGML |
| **GGML** | Legacy file format for quantized LLMs |
| **Inference** | The process of running the model to generate outputs |
| **Parameter** | The adjustable weights in the neural network (measured in billions) |
| **Provider** | Software that interfaces with models (Ollama, LocalAI, etc.) |
| **Quantization** | Technique to reduce model size by using lower precision numbers |
| **Token** | A piece of text processed by the model (roughly 4 characters in English) |

---

*This documentation was last updated on May 14, 2025. For the latest version, check the [official documentation](https://papin.docs).*
