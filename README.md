# Claude MCP Client

A fast, native client for Claude MCP built with Tauri (Rust) and React, featuring a complete implementation of the Model Context Protocol (MCP) with intelligent model routing and offline capability.

This project includes three interfaces:
1. A graphical desktop application (Tauri/React)
2. A command-line interface (CLI)
3. A terminal user interface (TUI)

All three interfaces share the same core functionality through a common library.

## Features

- **Ultra-fast startup** (<500ms to visible UI)
- **Model Context Protocol** implementation for real-time AI communication
- **Native performance** through Tauri/Rust backend
- **WebSocket streaming** for real-time responses
- **Local model fallback** for offline operation
- **Intelligent model router** for switching between cloud and local models
- **Feature flag system** for granular control
- **Multiple interfaces** (GUI, CLI, TUI) with shared functionality
- **Modern UI** with light/dark theme support

## Components

### Desktop Application (GUI)

The primary graphical interface built with Tauri and React. Features include:

- Fast-loading shell UI
- Real-time streaming responses
- Conversation management
- Settings and configuration
- Theme support
- Local model integration

See [src-tauri](src-tauri) and [src-frontend](src-frontend) for implementation details.

### Command Line Interface (CLI)

A powerful command-line tool for interacting with Claude models:

- Complete command set
- Interactive mode
- Streaming response support
- Piping and redirection support
- Integration with other CLI tools

See [src-cli](src-cli) for implementation details.

### Text User Interface (TUI)

A rich terminal-based UI for conversing with Claude:

- Full-featured terminal interface
- Keyboard navigation
- Command mode
- Real-time streaming
- Conversation management

See [src-tui](src-tui) for implementation details.

### Common Library

A shared library that provides functionality to all three interfaces:

- MCP protocol implementation
- Service layer for AI interactions
- Data models
- Configuration management
- Offline support

See [src-common](src-common) for implementation details.

## Architecture Overview

The application follows a layered architecture:

```
┌─────────────────────────────┐
│                             │
│ INTERFACES                  │
│ (GUI, CLI, TUI)             │
│                             │
└──────────────┬──────────────┘
               │
               │ Commands/Events
               │
┌──────────────▼──────────────┐
│                             │
│ COMMAND LAYER (Tauri/CLI)   │
│                             │
└──────────────┬──────────────┘
               │
               │ Service API
               │
┌──────────────▼──────────────┐
│                             │
│ SERVICE LAYER               │
│ (AI, Chat, Auth Services)   │
│                             │
└──────────────┬──────────────┘
               │
               │ Provider API
               │
┌──────────────▼──────────────┐
│                             │
│ PROVIDER LAYER              │
│ (Claude, Local Providers)   │
│                             │
└──────────────┬──────────────┘
               │
               │ Protocol API
               │
┌──────────────▼──────────────┐
│                             │
│ PROTOCOL LAYER              │
│ (MCP, REST, WebSocket)      │
│                             │
└─────────────────────────────┘
```

## Claude AI Integration

The client provides a sophisticated integration with Claude AI:

- **Multiple API Options**:
  - WebSocket-based Model Context Protocol (MCP)
  - RESTful API with streaming support
  - Automatic fallback between protocols

- **Real-time Streaming**: Message tokens are streamed in real-time for faster user experience

- **Offline Operation**: Automatically switches to local models when network is unavailable

- **Model Router**: Intelligent routing between Claude cloud models and local models based on:
  - Network availability
  - Configuration preferences
  - Custom routing rules

## Local Model Support

For offline operation, the client includes a local model system:

- **Local Inference**: Run small language models directly on your device
- **Model Management**: Automatic download and management of models
- **Streaming Generation**: Real-time token streaming for local models
- **Low Resource Usage**: Optimized for running on standard hardware

The local models serve as fallbacks when Claude is unavailable, ensuring you can always get a response.

## Installation

### Building from Source

1. Clone the repository:
   ```
   git clone https://github.com/your-username/claude-mcp.git
   cd claude-mcp
   ```

2. Build all components:
   ```
   make build-all
   ```

3. Install binaries (optional):
   ```
   make install-all
   ```

### Individual Components

You can build and install individual components:

```bash
# Build and install CLI
make build-cli
make install-cli

# Build and install TUI
make build-tui
make install-tui

# Build GUI
make build-gui
```

### Using Pre-built Binaries

Download the appropriate binaries for your platform from the [Releases](https://github.com/your-username/claude-mcp/releases) page.

## Development Setup

### Prerequisites

- Rust (1.62+)
- Node.js (16+)
- npm or yarn
- An Anthropic API key

### Configuration

Create a config file at one of these locations:
- Linux: `~/.config/mcp/config.json`
- macOS: `~/Library/Application Support/mcp/config.json`
- Windows: `%APPDATA%\mcp\config.json`

With the following content:

```json
{
  "api": {
    "key": "YOUR_API_KEY"
  },
  "model": {
    "default": "claude-3-opus-20240229"
  }
}
```

Or run the setup command:

```bash
mcp setup
```

## Feature Flag System

The application uses a feature flag system to enable/disable functionality:

- `LAZY_LOAD`: Enable lazy loading of non-essential components
- `PLUGINS`: Enable plugin system
- `HISTORY`: Enable conversation history features
- `ADVANCED_UI`: Enable advanced UI components
- `EXPERIMENTAL`: Enable experimental features
- `DEV_FEATURES`: Enable development-only features
- `ANALYTICS`: Enable analytics and telemetry
- `AUTO_UPDATE`: Enable auto-updates

## Project Status

For a detailed overview of the current project status, see [PROJECT_STATUS.md](PROJECT_STATUS.md).

## License

[MIT License](LICENSE)
