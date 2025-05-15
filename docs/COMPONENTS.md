# Papin Component Architecture

This document provides an overview of the key components in the Papin MCP Client and how they interact with each other.

## Core Architecture

Papin follows a multi-interface architecture with a shared common library:

```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│                 │  │                 │  │                 │
│  GUI (Tauri)    │  │  CLI            │  │  TUI            │
│                 │  │                 │  │                 │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────┐
│                                                         │
│               Common Library (mcp-common)               │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. Frontend Interface (GUI)

**Purpose**: Provides a rich desktop interface using Tauri and React.

**Key Components**:
- **React UI**: User interface components and state management
- **Tauri Commands**: Bridge between frontend and backend
- **Event System**: Real-time updates from backend to frontend

**Location**: `src-frontend/` directory

### 2. Command Line Interface (CLI)

**Purpose**: Provides a terminal-based interface for command-line operations.

**Key Components**:
- **Command Handlers**: Process user commands
- **Display Formatter**: Format output for the terminal
- **Interactive Mode**: REPL for conversation

**Location**: `src-cli/` directory

### 3. Text User Interface (TUI)

**Purpose**: Provides a full-screen terminal interface with interactive capabilities.

**Key Components**:
- **App Logic**: Core application logic
- **UI Renderer**: Terminal-based UI components
- **Event Handler**: Keyboard and mouse input processing

**Location**: `src-tui/` directory

### 4. Common Library

**Purpose**: Shared functionality used by all interfaces.

**Key Components**:
- **MCP Protocol**: Model Context Protocol implementation
- **Service Layer**: Core services for chat, authentication, etc.
- **Models**: Shared data models
- **Config**: Configuration management
- **Error Handling**: Centralized error system

**Location**: `src-common/` directory

### 5. Platform Layer

**Purpose**: Provides platform-specific optimizations and compatibility.

**Key Components**:
- **File System**: Cross-platform file operations
- **Hardware Detection**: CPU/GPU capabilities detection
- **Resource Monitoring**: System resource tracking
- **Performance Optimization**: Platform-specific performance tuning

**Location**: `src-common/src/platform/` and various platform-specific modules

### 6. Offline LLM System

**Purpose**: Enables using the application without internet connectivity.

**Key Components**:
- **Provider Interface**: Common interface for LLM providers
- **Provider Implementations**: Specific implementations (Ollama, LocalAI, etc.)
- **Model Management**: Model discovery, download, and selection
- **Inference Engine**: Text generation without cloud APIs

**Location**: `src/offline/llm/` directory

### 7. Plugin System

**Purpose**: Extends the application with custom functionality.

**Key Components**:
- **Plugin Registry**: Manages installed plugins
- **Plugin Loader**: Loads and initializes plugins in a sandbox
- **Permission System**: Controls resource access for plugins
- **SDK**: Development tools for plugin creation

**Location**: `src/plugins/` directory

### 8. Observability System

**Purpose**: Monitors and reports on application state and performance.

**Key Components**:
- **Metrics**: Performance metrics collection
- **Logging**: Structured logging system
- **Telemetry**: Anonymous usage data collection (opt-in)
- **Dashboards**: Visualization of metrics

**Location**: `src-common/src/observability/` and `src/observability/` directories

## Component Interaction Examples

### GUI Interaction Example

```
┌────────────┐          ┌────────────────┐          ┌─────────────┐          ┌─────────────┐
│            │  Command │                │  Service │             │  Protocol│             │
│ React UI   ├─────────►│ Tauri Command  ├─────────►│ ChatService ├─────────►│ MCP Client  │
│            │          │                │          │             │          │             │
└────────────┘          └────────────────┘          └─────────────┘          └──────┬──────┘
                                                                                     │
                                                                                     │
┌────────────┐          ┌────────────────┐          ┌─────────────┐          ┌──────▼──────┐
│            │  Update  │                │  Event   │             │  Response│             │
│ React UI   │◄─────────┤ Event Listener │◄─────────┤ ChatService │◄─────────┤ MCP Server  │
│            │          │                │          │             │          │             │
└────────────┘          └────────────────┘          └─────────────┘          └─────────────┘
```

### CLI Interaction Example

```
┌────────────┐          ┌────────────────┐          ┌─────────────┐          ┌─────────────┐
│            │  Command │                │  Service │             │  Protocol│             │
│ CLI Input  ├─────────►│ Command Parser ├─────────►│ ChatService ├─────────►│ MCP Client  │
│            │          │                │          │             │          │             │
└────────────┘          └────────────────┘          └─────────────┘          └──────┬──────┘
                                                                                     │
                                                                                     │
┌────────────┐          ┌────────────────┐          ┌─────────────┐          ┌──────▼──────┐
│            │  Output  │                │  Result  │             │  Response│             │
│ Terminal   │◄─────────┤ Output Format  │◄─────────┤ ChatService │◄─────────┤ MCP Server  │
│            │          │                │          │             │          │             │
└────────────┘          └────────────────┘          └─────────────┘          └─────────────┘
```

## Cross-Cutting Concerns

### 1. Error Handling

Error handling is implemented at multiple levels:

- **Application Level**: High-level error reporting to users
- **Service Level**: Service-specific error types and recovery
- **Protocol Level**: Network and communication errors
- **Platform Level**: OS-specific error handling

### 2. Configuration Management

Configuration is managed through a layered approach:

- **User Settings**: User-configurable options
- **Application Config**: Application-wide settings
- **Platform Config**: Platform-specific settings
- **Runtime Config**: Dynamic configuration based on environment

### 3. Security

Security is implemented through multiple mechanisms:

- **API Key Management**: Secure storage of authentication tokens
- **Permission System**: Granular control of resource access
- **Plugin Sandbox**: Isolation of third-party code
- **Data Encryption**: Protection of sensitive data

## Component Status

| Component | Status | Next Steps |
|-----------|--------|------------|
| GUI Interface | 🟡 Partial | Complete React component implementation |
| CLI Interface | ✅ Complete | Add more advanced commands |
| TUI Interface | ✅ Complete | Enhanced keyboard shortcuts |
| Common Library | ✅ Complete | Ongoing maintenance and extensions |
| Platform Layer | ✅ Complete | Extend hardware detection |
| Offline LLM | 🟡 Partial | Implement real inference engine |
| Plugin System | 🟡 Partial | Complete WASM sandbox |
| Observability | ✅ Complete | Add more visualization options |

## Conclusion

The component architecture of Papin is designed for flexibility, maintainability, and performance. The multi-interface approach with a shared common library ensures consistent behavior across different interfaces while optimizing for each interface's specific requirements.