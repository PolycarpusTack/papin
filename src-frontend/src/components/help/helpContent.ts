export const helpTopics = [
  {
    id: 'getting-started',
    title: 'Getting Started with Papin',
    category: 'Basics',
    summary: 'Learn the basics of Papin, from installation to your first conversation.',
    intro: {
      beginner: "Welcome to Papin! This guide will help you get started with the basics. Papin is a desktop application that helps you communicate with AI models, even when you're offline.",
      intermediate: "This guide covers the initial setup and configuration of Papin, including customization options and account management.",
      advanced: "This technical guide details Papin's architecture, setup procedures, and configuration options for optimal performance."
    },
    content: [
      {
        title: 'System Requirements',
        beginner: "Papin works on most modern computers. You'll need Windows 10 or newer, macOS 10.15 or newer, or a recent version of Linux. Your computer should have at least 4GB of RAM (8GB recommended) and 500MB of free disk space for the application itself, plus extra space for offline models if you choose to use them.",
        intermediate: "Papin is compatible with Windows 10/11, macOS 10.15+, and major Linux distributions. Minimum requirements include 4GB RAM, 500MB storage for the application, and additional space for local models and conversation history. For optimal performance with local models, 16GB RAM and a multi-core CPU are recommended.",
        advanced: "Papin's system requirements vary based on usage patterns and enabled features. Base installation requires 500MB of storage and 4GB RAM minimum. Local model usage increases requirements substantially: small models require 2GB additional RAM, medium models 4GB, and large models 8-16GB. Multi-threading is leveraged for inference acceleration on systems with >4 cores. GPU acceleration is available for CUDA-compatible NVIDIA GPUs (minimum 4GB VRAM) and Apple Silicon."
      },
      {
        title: 'Installation Process',
        beginner: [
          {
            title: 'Windows Installation',
            content: "To install Papin on Windows, download the installer from the official website, run the .msi file, and follow the on-screen instructions. Once installed, you can find Papin in your Start menu.",
            steps: [
              "Download the installer from the official website",
              "Run the .msi installer file",
              "Follow the on-screen instructions",
              "Launch Papin from the Start menu or desktop shortcut"
            ]
          },
          {
            title: 'macOS Installation',
            content: "For Mac users, download the .dmg file from the official website, open it, and drag Papin to your Applications folder. Then you can launch it from your Applications folder.",
            steps: [
              "Download the .dmg file from the official website",
              "Open the file and drag Papin to your Applications folder",
              "Launch Papin from your Applications folder",
              "If prompted about security, go to System Preferences > Security & Privacy to allow the app"
            ]
          },
          {
            title: 'Linux Installation',
            content: "For Linux, download the appropriate package for your distribution (.deb, .rpm, or .AppImage) and install it using your package manager or by making the AppImage executable.",
            steps: [
              "Download the appropriate package for your distribution",
              "For .deb packages: sudo dpkg -i papin_1.0.0.deb",
              "For .rpm packages: sudo rpm -i papin_1.0.0.rpm",
              "For .AppImage: Make the file executable with 'chmod +x Papin-1.0.0.AppImage' and run it"
            ]
          }
        ],
        intermediate: [
          {
            title: 'Installation Options',
            content: "Papin offers several installation options across platforms, including portable installations and silent deployment options for enterprise environments.",
            note: "Enterprise users can use the --silent flag for automated deployment across multiple workstations."
          },
          {
            title: 'Cross-Platform Considerations',
            content: "When installing on multiple platforms, be aware that user data synchronization requires the same account across devices. Platform-specific features may vary slightly, particularly around system integration points.",
            tip: "For consistent cross-platform experiences, use the cloud synchronization feature to keep your settings and conversations in sync."
          },
          {
            title: 'Verification and Updates',
            content: "After installation, verify application integrity using the built-in diagnostic tool. Papin supports both automatic and manual updates, configurable in the settings panel.",
            code: "# Check application integrity\npapin --verify\n\n# Check for updates manually\npapin --check-update"
          }
        ],
        advanced: [
          {
            title: 'Command-Line Installation',
            content: "Advanced users can leverage command-line installation and configuration options.",
            code: "# Windows (PowerShell)\n$env:PAPIN_INSTALL_DIR = \"C:\\CustomPath\"\n$env:PAPIN_CONFIG_PRESET = \"developer\"\n.\\papin-installer.exe /S\n\n# macOS/Linux\nPAPIN_INSTALL_DIR=\"/opt/papin\" PAPIN_CONFIG_PRESET=\"developer\" ./papin-installer.sh"
          },
          {
            title: 'Enterprise Deployment',
            content: "For enterprise deployment, Papin supports system-wide installation with customizable policy configurations.",
            code: "# Deploy with custom policy file\npapin-installer --system-wide --policy-file=/path/to/policy.json"
          },
          {
            title: 'Installation Verification',
            content: "Cryptographic verification of installation artifacts ensures integrity.",
            code: "# Verify package signature\ngpg --verify papin-1.0.0.AppImage.sig papin-1.0.0.AppImage\n\n# Verify checksum\nsha256sum -c papin-1.0.0.AppImage.sha256"
          }
        ]
      },
      {
        title: 'Initial Configuration',
        beginner: "When you first open Papin, you'll need to create an account or sign in with an existing one. Follow the welcome wizard to set up your preferences, like which AI models you want to use and whether you want to enable offline mode. Don't worry, you can always change these settings later!",
        intermediate: "The initial configuration process includes account creation, preference setting, and optional local model downloads. Consider your typical usage pattern when selecting which offline capabilities to enable, as larger models require significant disk space. Authentication supports standard email/password, SSO, and OAuth integration with popular identity providers.",
        advanced: "Initial configuration can be automated via CLI flags or configuration files. Enterprise deployments can leverage LDAP/SAML integration and policy-based configuration management. Advanced users should consider configuring custom model endpoints, network proxy settings, and fine-tuned resource allocation based on hardware specifications."
      },
      {
        title: 'Your First Conversation',
        beginner: "Now that you're set up, let's start your first conversation! Click the 'New Conversation' button, choose which AI model you'd like to talk to, and type your message in the box at the bottom. Press Enter or click the Send button, and the AI will respond in the conversation window. It's that simple!",
        intermediate: "Initiating a conversation involves selecting an appropriate model based on your needs. Consider the context and complexity of your query when selecting a model. For technical queries, the specialized models may yield better results than general-purpose ones. Initial conversations are automatically titled based on content, but you can rename them for better organization.",
        advanced: "Conversations are backed by persistent storage with automatic checkpointing. Each message exchange generates a complete context window evaluation with adjustable parameters including temperature, top_p, and frequency penalty. Advanced users can modify these parameters via the API or configuration files to optimize for specific use cases."
      }
    ],
    faq: [
      {
        question: "Do I need an internet connection to use Papin?",
        beginner: "While Papin works best with an internet connection, you can still use it offline if you've set up offline mode. Some advanced features might not be available without internet, but you can still have conversations with AI models you've downloaded.",
        intermediate: "Papin functions in both online and offline modes. For offline use, you'll need to download models in advance and enable offline mode in settings. Syncing and cloud features require connectivity, but all core conversation functionality works offline with downloaded models.",
        advanced: "Papin implements a connection-aware state machine that automatically transitions between online and offline operation modes. Online operation enables cloud model access, synchronization, and telemetry collection. Offline operation leverages local model inference, persistent storage, and queues changes for future synchronization. Network requirements can be fine-tuned using the bandwidth management settings."
      },
      {
        question: "How do I update Papin?",
        beginner: "Papin checks for updates automatically when you're connected to the internet. When an update is available, you'll see a notification. Just click 'Update Now' to get the latest version!",
        intermediate: "Updates are managed through the built-in updater system accessible via Help > Check for Updates. You can configure update behavior in Settings > Application > Updates, including automatic download, silent installation, and update channel selection (stable, beta, etc.).",
        advanced: "Papin leverages a differential update system to minimize bandwidth usage. Updates are cryptographically signed and verified before installation. Enterprise environments can control updates through policy files and can stage updates for controlled rollout using the canary system. Custom update endpoints are supported for air-gapped networks."
      }
    ],
    relatedTopics: ['offline-capabilities', 'ui-navigation', 'account-management']
  },
  {
    id: 'conversations',
    title: 'Managing Conversations',
    category: 'Core Features',
    summary: 'Learn how to create, organize, and get the most out of your AI conversations.',
    intro: {
      beginner: "Conversations are at the heart of Papin. This guide will show you how to create, manage, and organize your conversations with AI models.",
      intermediate: "This comprehensive guide covers conversation management techniques, organization strategies, and effective prompt engineering.",
      advanced: "This technical overview explains the conversation architecture, storage mechanisms, and advanced conversation manipulation capabilities."
    },
    content: [
      {
        title: 'Creating New Conversations',
        beginner: "To start a new conversation, click the 'New Conversation' button in the top-left corner of the app or use the keyboard shortcut Ctrl+N (Cmd+N on Mac). You'll be able to choose which AI model you want to talk to before starting the conversation.",
        intermediate: "When creating new conversations, consider the specific model capabilities needed for your task. Specialized models offer better performance for domain-specific tasks, while general models excel at versatility. Conversation templates provide standardized starting points for common workflows.",
        advanced: "Conversation initialization triggers a series of template resolution steps, model capability verification, and resource allocation processes. Advanced users can script conversation creation using the API with pre-specified parameters, system prompts, and initial message sequences."
      },
      {
        title: 'Conversation History and Navigation',
        beginner: "All your conversations are saved automatically and appear in the sidebar on the left side of the app. Click any conversation to continue where you left off. You can use the search bar at the top of the sidebar to find specific conversations by typing keywords.",
        intermediate: "Conversation history leverages efficient storage mechanisms with full-text search capabilities. The contextual search supports filtering by date ranges, models used, and specific content types like code or data requests. Organization features include folders, tags, and starring important conversations.",
        advanced: "The conversation history implements a hybrid storage architecture utilizing SQLite for metadata and structured information with file-system based storage for conversation content. This enables efficient indexing while maintaining readability of raw data. The search system uses an inverted index with BM25 ranking for relevance-based retrieval."
      },
      {
        title: 'Organizing with Folders and Tags',
        beginner: "Keep your conversations organized by using folders and tags. To create a folder, right-click in the sidebar and select 'New Folder'. You can drag conversations into folders. To add tags, right-click on a conversation and select 'Add Tags'.",
        intermediate: [
          {
            title: 'Creating an Organization System',
            content: "Design an organization system that matches your workflow. Project-based folders work well for focused work, while tags can cross-cut across projects for thematic organization."
          },
          {
            title: 'Smart Folders',
            content: "Create smart folders that automatically collect conversations matching specific criteria. Configure rules based on content, models, dates, or tags."
          },
          {
            title: 'Bulk Organization',
            content: "Use multi-select (Ctrl+click or Shift+click) to organize multiple conversations at once. Bulk operations include moving to folders, adding tags, and exporting."
          }
        ],
        advanced: [
          {
            title: 'Custom Organization Schemes',
            content: "Create custom organization schemes using the advanced filtering API. Schemes can combine multiple metadata attributes with content-based filters.",
            code: "const filter = {\n  metadata: {\n    models: ['gpt4', 'claude-3'],\n    after: '2023-06-01',\n    tags: ['research', 'machine-learning']\n  },\n  content: {\n    contains: ['neural networks', 'transformer architecture'],\n    excludes: ['personal', 'confidential']\n  }\n};"
          },
          {
            title: 'Programmatic Organization',
            content: "Leverage the API for programmatic organization of conversations based on content analysis.",
            code: "// Auto-categorize conversations based on content analysis\nconst conversations = await api.getConversations();\nfor (const conv of conversations) {\n  const category = await analyzer.categorize(conv.content);\n  await api.addTag(conv.id, category);\n}"
          }
        ]
      },
      {
        title: 'Conversation Settings and Preferences',
        beginner: "Each conversation has its own settings you can adjust. Click the gear icon in the top-right corner of a conversation to change the AI model, adjust how the AI responds, or enable features like code highlighting.",
        intermediate: "Conversation settings include model selection, parameter tuning, and feature toggles. Model parameters control response characteristics like creativity, precision, and length. Context settings manage how much conversation history is included in each request. Accessibility settings include text-to-speech and alternative text rendering options.",
        advanced: "Each conversation maintains a complete set of configurable parameters that influence model behavior, rendering, and storage. These can be accessed and modified programmatically through the API. Parameter modifications during a conversation create change points that are tracked in the conversation metadata, enabling analysis of parameter impact on model outputs."
      },
      {
        title: 'Effective Prompt Engineering',
        beginner: "To get the best responses, try to be clear and specific in your messages. If you want the AI to respond in a certain way, don't be afraid to ask directly. For example, 'Please explain this like I'm 10 years old' or 'Can you format your response as a bullet-point list?'",
        intermediate: [
          {
            title: 'Structuring Effective Prompts',
            content: "Structure prompts with clear instructions, relevant context, and specific output formats. Break complex tasks into smaller steps for better results. Prime the model with examples of the desired output format."
          },
          {
            title: 'Role-Based Prompting',
            content: "Assign specific roles to the AI to obtain specialized perspectives. For example, 'Respond as a cybersecurity expert' or 'Answer from the perspective of a historian specializing in ancient Rome.'"
          },
          {
            title: 'Chain-of-Thought Prompting',
            content: "For complex reasoning tasks, encourage the model to work through the problem step-by-step by explicitly asking it to show its reasoning process."
          }
        ],
        advanced: [
          {
            title: 'Model-Specific Optimization',
            content: "Different model architectures respond optimally to different prompting techniques. Instruction-tuned models benefit from clear directives, while knowledge-heavy models may need careful context priming.",
            code: "// Example system prompt structure for optimal performance\nconst systemPrompt = {\n  role: \"system\",\n  content: `You are an expert in ${domain} with the following characteristics:\n  - ${characteristic1}\n  - ${characteristic2}\n  - ${characteristic3}\n\nRespond in the following format:\n${formatExample}`\n};"
          },
          {
            title: 'Retrieval-Augmented Generation',
            content: "Implement RAG techniques by augmenting prompts with relevant external knowledge, either from local document stores or integrated knowledge bases.",
            code: "// RAG implementation pseudo-code\nasync function enhancedPrompt(userQuery) {\n  const relevantDocs = await vectorStore.similaritySearch(userQuery);\n  return `Answer based on the following information:\\n\\n${relevantDocs.join('\\n\\n')}\\n\\nUser question: ${userQuery}`;\n}"
          },
          {
            title: 'Prompt Optimization',
            content: "Systematically optimize prompts through iterative refinement and testing against benchmark tasks.",
            code: "// Prompt testing framework\nconst promptVariants = [\n  { template: 'Explain ${concept} simply', name: 'Simple' },\n  { template: 'Explain ${concept} step by step', name: 'Steps' },\n  { template: 'Explain ${concept} with analogies', name: 'Analogy' }\n];\n\nconst results = await testPrompts(promptVariants, testCases);"
          }
        ]
      }
    ],
    examples: [
      {
        title: "Creating a Project-Based Folder Structure",
        description: "This example shows how to organize conversations for a research project:",
        steps: [
          "Right-click in the sidebar and select 'New Folder', name it 'Research Project'",
          "Inside that folder, create subfolders for 'Literature Review', 'Methodology', and 'Analysis'",
          "Start new conversations for specific research questions and drag them to appropriate folders",
          "Add tags like 'urgent', 'to-review', or 'completed' to track progress"
        ]
      },
      {
        title: "Effective Prompt for Complex Explanation",
        description: "Example of how to structure a prompt for a complex topic explanation:",
        code: "I need an explanation of quantum computing.\n\nPlease structure your response in the following way:\n1. Start with a simple analogy that a high school student would understand\n2. Explain the key principles and terminology\n3. Describe 2-3 practical applications\n4. Suggest resources for learning more\n\nKeep technical jargon to a minimum and explain any specialized terms."
      }
    ],
    relatedTopics: ['ui-navigation', 'offline-capabilities', 'local-llm']
  },
  {
    id: 'troubleshooting',
    title: 'Troubleshooting',
    category: 'Support',
    summary: 'Find solutions to common problems and learn how to diagnose issues with Papin.',
    intro: {
      beginner: "Having trouble with Papin? This guide will help you solve common problems and get back to using the app quickly.",
      intermediate: "This comprehensive troubleshooting guide covers diagnostic processes, common issues, and advanced resolution techniques for Papin-related problems.",
      advanced: "This technical guide details Papin's diagnostics architecture, error handling mechanisms, and advanced debugging methodologies for resolving complex issues."
    },
    content: [
      {
        title: 'Common Issues and Solutions',
        beginner: [
          {
            title: 'Application Won\'t Start',
            content: "If Papin won't start, try these steps: 1) Restart your computer, 2) Make sure your computer meets the minimum requirements, 3) Reinstall Papin with a fresh download from the official website, 4) Check if your antivirus might be blocking the app."
          },
          {
            title: 'Slow Performance',
            content: "If Papin is running slowly, try: 1) Close other applications to free up memory, 2) Use a smaller or cloud-based model instead of a large local model, 3) Check your internet connection if using cloud models, 4) Enable 'Performance Mode' in Settings."
          },
          {
            title: 'Offline Mode Not Working',
            content: "If offline mode isn't working properly: 1) Make sure you've downloaded local models, 2) Check that offline mode is enabled in Settings, 3) Verify you have enough storage space for models, 4) Try restarting the application."
          },
          {
            title: 'Sync Problems',
            content: "If your conversations aren't syncing between devices: 1) Check your internet connection, 2) Make sure you're signed in to the same account on all devices, 3) Try manually triggering a sync with Ctrl+Shift+S, 4) Check if there are sync conflicts to resolve in Settings > Sync > Conflicts."
          }
        ],
        intermediate: [
          {
            title: 'Diagnostic Process',
            content: "When troubleshooting issues, follow this systematic diagnostic process: 1) Identify the specific symptoms and when they occur, 2) Check application logs for relevant errors, 3) Verify system requirements and resource availability, 4) Test in different environments or configurations to isolate variables, 5) Apply targeted solutions based on diagnostic findings."
          },
          {
            title: 'Application Crashes',
            content: "For application crashes, examine crash logs located in your user directory under .papin/logs. Key indicators include memory access violations (suggesting memory issues), uncaught exceptions (indicating bug or edge case), or resource exhaustion (showing insufficient system resources). The log analysis tool in Help > Diagnostics > Log Analysis can help interpret these files."
          },
          {
            title: 'Model Loading Failures',
            content: "Model loading failures typically manifest as timeout errors, validation errors, or memory allocation failures. These can be resolved by verifying file integrity, ensuring sufficient resources (particularly RAM), and confirming compatibility between model versions and application versions. The model verification tool can diagnose specific model issues."
          },
          {
            title: 'Network-Related Issues',
            content: "Network problems often present as timeout errors, authentication failures, or sync conflicts. Diagnosing network issues involves checking connectivity to required endpoints, verifying API authentication, and examining network traffic patterns. The network diagnostic tool in Help > Diagnostics can perform comprehensive connectivity testing."
          }
        ],
        advanced: [
          {
            title: 'Telemetry Analysis',
            content: "Leverage built-in telemetry for advanced diagnostics and troubleshooting.",
            code: "// Retrieve detailed telemetry for specific subsystem\nconst telemetry = await api.diagnostics.getTelemetry({\n  subsystem: 'inference',\n  timeRange: { start: '-1h', end: 'now' },\n  includeMetrics: true,\n  includeLogs: true,\n  includeTraces: true,\n  samplingRate: 1.0 // Collect everything\n});\n\n// Analyze for anomalies\nconst anomalies = api.diagnostics.detectAnomalies(telemetry, {\n  algorithm: 'isolation-forest',\n  sensitivity: 0.8\n});"
          },
          {
            title: 'System State Inspection',
            content: "Inspect application state for inconsistencies and corruption using the state debugging tools.",
            code: "// Inspect and verify state integrity\nconst stateReport = await api.diagnostics.inspectState({\n  components: ['models', 'conversations', 'settings', 'sync'],\n  verifyIntegrity: true,\n  repairMode: 'report-only' // vs. 'auto-repair'\n});\n\n// Check for specific state corruption patterns\nif (stateReport.corruptedEntities.length > 0) {\n  // Handle corrupted state entities\n  const fixOptions = api.diagnostics.generateRepairPlan(stateReport);\n  console.log(fixOptions);\n}"
          },
          {
            title: 'Debugging Mode',
            content: "Enable comprehensive debugging mode for troubleshooting complex issues.",
            code: "// Enable debugging mode\napi.debug.enable({\n  level: 'trace', // vs. 'debug', 'info', etc.\n  subsystems: ['all'],\n  file: true,\n  console: true,\n  telemetry: false,\n  retention: '7d',\n  includeSymbolTables: true\n});\n\n// Inject diagnostic hooks\napi.debug.injectHooks({\n  beforeInference: (params) => {\n    console.log('Inference params:', params);\n    return params; // Can modify params here\n  },\n  afterInference: (result, metrics) => {\n    console.log('Inference metrics:', metrics);\n    return result; // Can modify result here\n  }\n});"
          }
        ]
      },
      {
        title: 'Using the Log Viewer',
        beginner: "The Log Viewer helps you (or support staff) figure out what's going wrong. To access it, go to Help > View Logs. You'll see a list of events that have happened in the app. If you're reporting a problem to support, they might ask you to share these logs to help diagnose the issue.",
        intermediate: "The Log Viewer provides a structured interface for examining application logs with filtering, searching, and export capabilities. Log entries are categorized by severity (debug, info, warning, error, critical) and subsystem (UI, models, sync, etc.). The contextual information includes timestamps, session IDs, and relevant state details. Use the export function to generate log packages for support tickets, which automatically redacts sensitive information.",
        advanced: "The logging infrastructure implements structured logging with contextual enrichment, correlation IDs, and causality tracking. Log rotation policies balance diagnostic value with storage efficiency, while real-time filtering leverages indexes for rapid query execution. Advanced filtering allows complex expressions combining predicates on multiple fields, with support for regular expressions and temporal constraints."
      },
      {
        title: 'Diagnostic Tools',
        beginner: "Papin includes several tools to help diagnose problems. Go to Help > Diagnostics to find tools for checking your network connection, testing model loading, and analyzing system compatibility. These tools can automatically detect common issues and suggest solutions.",
        intermediate: [
          {
            title: 'Network Diagnostics',
            content: "The network diagnostics tool performs comprehensive connectivity testing for all required services. It validates DNS resolution, endpoint accessibility, authentication, and throughput to identify specific network-related issues. The tool can distinguish between general connectivity problems and service-specific issues."
          },
          {
            title: 'System Compatibility Check',
            content: "The compatibility checker validates system specifications against application requirements, testing CPU capabilities, memory subsystems, storage performance, and GPU compatibility. It identifies potential hardware bottlenecks and suggests optimizations specific to your configuration."
          },
          {
            title: 'Model Verification',
            content: "The model verification tool validates local model files for integrity, ensuring they haven't been corrupted or tampered with. It verifies checksums, tests loading into memory, and performs basic inference tests to confirm functionality."
          },
          {
            title: 'Database Integrity Check',
            content: "The database checker validates the integrity of conversation storage and application settings, identifying and repairing corruption when possible. It verifies referential integrity, checks for orphaned data, and validates schema compliance."
          }
        ],
        advanced: [
          {
            title: 'Diagnostic API',
            content: "Access the diagnostic system programmatically for automated troubleshooting.",
            code: "// Comprehensive system diagnostics\nconst diagnosticSuite = api.diagnostics.createSuite({\n  modules: ['system', 'network', 'storage', 'models', 'database'],\n  depth: 'comprehensive', // vs. 'basic' or 'extended'\n  repair: false, // Don't auto-repair issues\n  timeout: 300000 // 5 minutes\n});\n\nconst results = await diagnosticSuite.run();\nconst report = diagnosticSuite.generateReport('markdown');"
          },
          {
            title: 'Performance Profiling',
            content: "Use built-in profiling tools to identify performance bottlenecks.",
            code: "// CPU profiling\nconst cpuProfile = await api.diagnostics.profileCPU({\n  duration: 30000, // 30 seconds\n  sampleInterval: 1, // 1ms\n  includeNative: true,\n  threads: 'all'\n});\n\n// Memory profiling\nconst memProfile = await api.diagnostics.profileMemory({\n  trackAllocations: true,\n  stackDepth: 20,\n  gcBefore: true\n});\n\n// Generate flame graph\nconst flamegraph = api.diagnostics.generateFlameGraph(cpuProfile);"
          },
          {
            title: 'Custom Diagnostic Modules',
            content: "Extend the diagnostic system with custom modules for specialized troubleshooting.",
            code: "// Define custom diagnostic module\napi.diagnostics.registerModule({\n  id: 'custom-pipeline-validator',\n  name: 'Custom Pipeline Validation',\n  description: 'Validates custom processing pipeline configuration',\n  run: async (options) => {\n    const results = { issues: [] };\n    // Custom validation logic...\n    return results;\n  },\n  interpretResults: (results) => {\n    // Generate human-readable interpretation\n    return { summary: '...', details: '...', severity: 'warning' };\n  }\n});"
          }
        ]
      },
      {
        title: 'Contacting Support',
        beginner: "If you can't solve a problem yourself, you can contact support for help. Go to Help > Contact Support or email support@papin.app. When reporting an issue, try to include: 1) What you were doing when the problem occurred, 2) What exactly happened, 3) Any error messages you saw, and 4) Your system information (Help > About).",
        intermediate: "When contacting support, provide comprehensive diagnostic information for faster resolution. Use the Support Package Generator (Help > Generate Support Package) to automatically collect relevant logs, system information, and diagnostic reports in a privacy-preserving format. Include reproducible steps for the issue, noting both the expected and actual behavior. For complex issues, consider scheduling a live troubleshooting session using the in-app booking tool.",
        advanced: "For advanced issues requiring engineering involvement, prepare a technical case file using the Support Debug Environment. This creates an isolated instance that captures a complete snapshot of the application state, execution traces, and environmental context. Configure the debugging telemetry level to balance diagnostic detail with privacy requirements. Enterprise customers can leverage the dedicated support channel with SLA-backed response times and escalation paths to engineering teams."
      }
    ],
    faq: [
      {
        question: "Why is Papin using so much memory?",
        beginner: "Papin uses memory for running AI models, especially local models that run on your computer. Large models need more memory to work properly. If memory usage is a concern, try using smaller models or cloud models instead of large local models.",
        intermediate: "Memory usage in Papin is primarily driven by model inference, particularly when using local models. The memory footprint includes model weights, KV caches, and temporary tensors required during inference. Memory usage scales with model size, context length, and batch size. To reduce memory consumption, consider using smaller models, quantized models, or enabling memory-efficient inference options in Settings > Performance > Advanced.",
        advanced: "Memory utilization in Papin follows a multi-tier allocation strategy optimized for inference workloads. The primary contributors to memory usage include model weights (static allocation based on model size and quantization), KV cache (dynamic allocation scaling with context length), inference tensors (temporary allocations dependent on batch size and model architecture), and application overhead. Advanced memory management techniques include weight sharing, disk offloading for inactive models, and dynamic precision adjustment based on available resources."
      },
      {
        question: "How can I make Papin run faster?",
        beginner: "To improve performance, try: 1) Close other applications to free up memory and CPU, 2) Use smaller models or cloud models instead of large local models, 3) Enable 'Performance Mode' in Settings > Performance, 4) Limit the number of open conversations, and 5) Make sure your computer meets the recommended system requirements.",
        intermediate: "Performance optimization involves multiple factors. Consider adjusting the following: 1) Model loading strategy (preload vs. on-demand), 2) Thread allocation for inference, 3) GPU acceleration settings if available, 4) Memory limits for model caching, and 5) Background task scheduling. For the most significant improvement, focus on selecting the right model size for your hardware capabilities.",
        advanced: "Comprehensive performance optimization involves a systems approach. Analyze bottlenecks using the profiling tools to identify whether the constraints are compute, memory, or I/O related. Consider implementing custom inference configurations tuned to your specific hardware, enabling specialized acceleration libraries, and leveraging quantized models with hardware-specific optimizations. For multi-model workflows, implement strategic model unloading and resource arbitration to prevent resource contention."
      }
    ],
    relatedTopics: ['performance-monitoring', 'local-llm', 'offline-capabilities']
  },
  {
    id: 'offline-capabilities',
    title: 'Offline Capabilities',
    category: 'Core Features',
    summary: 'Learn how to use Papin even when you don\'t have an internet connection.',
    intro: {
      beginner: "One of Papin's best features is that you can use it without an internet connection. This guide will show you how to set up and use offline mode.",
      intermediate: "This guide covers Papin's comprehensive offline capabilities, including local model management, synchronization, and offline workflow optimization.",
      advanced: "This technical documentation details the architecture of Papin's offline systems, including local inference engines, state management, and synchronization protocols."
    },
    content: [
      {
        title: 'Understanding Offline Mode',
        beginner: "Offline mode lets you use Papin even when you don't have internet access. It works by downloading AI models to your computer, so you can have conversations without connecting to the cloud. Your conversations are saved locally and can sync when you're back online.",
        intermediate: "Papin's offline capabilities leverage a local-first architecture that prioritizes availability and responsiveness. The system maintains functional equivalence between online and offline modes, with graceful degradation for features that require connectivity. This approach ensures consistent user experience regardless of network status.",
        advanced: "The offline architecture implements a sophisticated state machine that manages transitions between connectivity states. It employs optimistic concurrency control for operations, persistent queue management for deferred synchronization, and differential reconciliation algorithms to resolve conflicts. The system maintains a comprehensive event log to ensure data consistency across state transitions."
      },
      {
        title: 'Setting Up Offline Mode',
        beginner: [
          {
            title: 'Enabling Offline Mode',
            content: "To enable offline mode, go to Settings > Offline and toggle the 'Enable Offline Mode' switch. You'll then be asked to choose which local models you want to download."
          },
          {
            title: 'Choosing Models',
            content: "Start with smaller models if you're concerned about disk space. They download faster and use less storage, but might not be as capable as larger models."
          },
          {
            title: 'Download Process',
            content: "Downloading models might take some time depending on your internet speed and the model size. You can continue using Papin while downloads are in progress."
          }
        ],
        intermediate: [
          {
            title: 'Optimizing Model Selection',
            content: "Select models based on your typical offline usage patterns. Consider the trade-offs between model size, capability, and resource requirements. For general use, a medium-sized general model and a small specialized model often provide good coverage."
          },
          {
            title: 'Storage Management',
            content: "Monitor storage usage through the Models dashboard. Consider external storage options for larger models if your system has limited internal storage. Compression options can reduce storage requirements at the cost of slightly increased load times."
          },
          {
            title: 'Background Downloads',
            content: "Configure download scheduling to optimize for network conditions. Enable background downloading to continue model updates even when the application is closed. Set bandwidth limits to prevent network congestion during downloads."
          }
        ],
        advanced: [
          {
            title: 'Custom Model Integration',
            content: "Integrate custom or fine-tuned models into the offline system using the model registry API.",
            code: "// Register a custom local model\nconst modelConfig = {\n  id: 'custom-model-v1',\n  path: '/path/to/model/weights',\n  quantization: 'int8',\n  contextSize: 8192,\n  inferenceParams: {\n    threads: 4,\n    batchSize: 512\n  }\n};\napi.models.registerLocalModel(modelConfig);"
          },
          {
            title: 'Advanced Resource Allocation',
            content: "Configure fine-grained resource allocation for offline inference to optimize performance based on hardware capabilities.",
            code: "// Optimized resource configuration\nconst resourceConfig = {\n  memoryLimits: {\n    maxRam: '8GB',\n    maxVram: '4GB',\n    swapBuffer: '2GB'\n  },\n  computeAllocation: {\n    cpuThreads: 6,\n    cudaDevices: [0, 1],\n    prioritization: 'efficiency' // or 'speed'\n  }\n};"
          },
          {
            title: 'Model Sharding',
            content: "Implement model sharding to distribute large models across limited resources.",
            code: "// Configure model sharding\nconst shardingConfig = {\n  enabled: true,\n  shardSize: '2GB',\n  strategy: 'balanced', // 'memory-optimized' or 'latency-optimized'\n  diskCache: true,\n  prefetchWindow: 3\n};"
          }
        ]
      },
      {
        title: 'Working in Offline Mode',
        beginner: "When you're offline, Papin will automatically switch to using local models. You'll see an indicator in the top-right corner showing your connection status. Simply continue your conversations as normal! New conversations and changes are saved on your device.",
        intermediate: "During offline operation, Papin maintains the full feature set with local computational resources. The system automatically manages model loading and unloading based on usage patterns and available resources. Conversations are stored in a local database with automatic checkpointing for reliability. When connectivity is restored, the synchronization process reconciles local and cloud states.",
        advanced: "Offline operation relies on a complete local infrastructure including inference engines, storage services, and state management systems. The inference subsystem implements dynamic load balancing across available compute resources, with prioritization mechanisms for interactive requests. The storage layer employs a multi-tier architecture with optimized caching policies for high-throughput operation."
      },
      {
        title: 'Synchronization Process',
        beginner: "When you reconnect to the internet, Papin automatically syncs your offline conversations with your cloud account. You can see the sync status in the status bar at the bottom of the app. If you want to manually trigger a sync, click the sync icon or use Ctrl+Shift+S.",
        intermediate: [
          {
            title: 'Understanding the Sync Process',
            content: "Synchronization manages bidirectional data flow between local storage and cloud services. Changes are tracked through a distributed version control system and reconciled using intelligent conflict resolution strategies. The process prioritizes user data integrity while minimizing bandwidth usage."
          },
          {
            title: 'Managing Sync Conflicts',
            content: "When the same conversation is modified in multiple locations, conflict resolution intelligently merges changes. For direct conflicts, the system presents options to keep either version or merge manually. Conflict resolution preferences can be configured in advanced settings."
          },
          {
            title: 'Selective Synchronization',
            content: "Configure which content gets synchronized based on tags, folders, or conversation properties. Sensitive conversations can be marked as local-only to prevent cloud storage. Bandwidth usage can be optimized through selective sync policies."
          }
        ],
        advanced: [
          {
            title: 'Sync Protocol Architecture',
            content: "The synchronization system implements a CRDT-based protocol for eventual consistency across devices. Vector clocks track causality between events, enabling accurate conflict detection even with unreliable connectivity.",
            code: "// Simplified representation of the sync protocol\ninterface SyncOperation {\n  id: string;\n  vectorClock: Map<DeviceId, number>;\n  operation: 'create' | 'update' | 'delete';\n  resource: string;\n  payload: any;\n  parentOperations: string[];\n}"
          },
          {
            title: 'Efficient Delta Synchronization',
            content: "To minimize bandwidth usage, the system transmits only differential changes using an optimized binary format. This includes structural diffing for conversation content and model-specific optimizations for large assets.",
            code: "// Configure delta sync options\nconst deltaOptions = {\n  compression: 'zstd',\n  diffAlgorithm: 'structural', // vs. 'naive' or 'hybrid'\n  chunkSize: 262144, // bytes\n  deduplicate: true,\n  verifyChecksums: true\n};"
          },
          {
            title: 'Background Synchronization',
            content: "The system implements sophisticated background sync strategies with progressive backoff, network awareness, and battery optimization.",
            code: "// Advanced sync strategy configuration\nconst syncStrategy = {\n  initialRetryDelay: 1000, // ms\n  maxRetryDelay: 3600000, // 1 hour\n  backoffMultiplier: 1.5,\n  networkConditions: {\n    requireWifi: false,\n    meterednessAware: true,\n    minimumBandwidth: 0.5 // Mbps\n  }\n};"
          }
        ]
      }
    ],
    examples: [
      {
        title: "Optimizing for Travel Use",
        description: "Configure Papin for optimal offline use while traveling:",
        steps: [
          "Download small and medium-sized models before your trip",
          "Enable 'Aggressive caching' in Settings > Offline > Advanced",
          "Configure 'Bandwidth-aware sync' to prevent large syncs on hotel Wi-Fi",
          "Enable 'Power-saving mode' to optimize battery usage",
          "Set up scheduled synchronization for when you expect to have good connectivity"
        ]
      },
      {
        title: "Handling Specialized Tasks Offline",
        description: "Create a customized offline setup for specialized development tasks:",
        code: "// Custom offline configuration for code-focused workflows\nconst codeOptimizedConfig = {\n  models: ['papin-code-assistant-medium'],\n  contextSize: 16384,\n  templateOverrides: {\n    systemPrompts: {\n      codingAssistant: 'You are a helpful programming assistant...',\n      documentationHelper: 'You are a documentation specialist...',\n      bugDebugger: 'You are an expert in debugging code issues...'\n    }\n  },\n  caching: {\n    preferLocalExamples: true,\n    storeCodeSnippets: true,\n    indexingLevel: 'comprehensive'\n  }\n};"
      }
    ],
    relatedTopics: ['local-llm', 'performance-monitoring', 'troubleshooting']
  }
];
