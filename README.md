# Claude MCP Client for Linux

A fast, native Linux client for Claude MCP built with Tauri (Rust) and React, featuring a complete implementation of the Model Context Protocol (MCP).

## Features

- **Ultra-fast startup** (<500ms to visible UI)
- **Model Context Protocol** implementation for real-time AI communication
- **Native performance** through Tauri/Rust backend
- **WebSocket streaming** for real-time responses
- **Feature flag system** for granular control
- **Lazy loading** of non-essential components
- **Modern UI** with light/dark theme support

## Project Structure

The project is organized as follows:

```
claude-mcp/
├── src/                     # Rust backend code
│   ├── main.rs              # Tauri app entry point
│   ├── shell_loader.rs      # Fast bootstrap loader
│   ├── feature_flags.rs     # Feature flag system
│   ├── models/              # Data models
│   │   ├── mod.rs           # Base models
│   │   └── messages.rs      # Message definitions
│   ├── protocols/           # Protocol implementations
│   │   ├── mod.rs           # Protocol abstractions
│   │   └── mcp/             # MCP protocol implementation
│   ├── services/            # Backend services
│   │   ├── api.rs           # API service
│   │   ├── auth.rs          # Authentication service
│   │   ├── chat.rs          # Chat service
│   │   └── mcp.rs           # MCP service
│   ├── commands/            # Tauri commands
│   │   ├── auth.rs          # Auth commands
│   │   ├── chat.rs          # Chat commands
│   │   └── mcp.rs           # MCP commands
│   └── utils/               # Utility functions
│       ├── config.rs        # Configuration management
│       ├── events.rs        # Event system
│       └── lazy_loader.rs   # Lazy loading implementation
├── src-tauri/               # Tauri configuration
├── src-frontend/            # React frontend
    ├── src/
    │   ├── main.tsx         # React entry point
    │   ├── App.tsx          # Main React component
    │   ├── components/      # UI components
    │   └── lazy/            # Lazy-loaded components
```

## Model Context Protocol (MCP)

The client implements the complete Model Context Protocol for communicating with AI models over WebSockets:

- **Real-time communication** with Anthropic's Claude models
- **Message streaming** for faster responses
- **Bi-directional protocol** supporting request-response and events
- **Efficient serialization** for minimal latency
- **Reconnection handling** for resilient connections
- **Authentication** with API key and session management

For detailed information on the MCP implementation, see [MCP_README.md](MCP_README.md).

## Fast Startup Architecture

This project implements a multi-stage loading process to achieve <500ms startup time:

1. **Shell Loading** (~20ms): Initial minimal UI frame
2. **Shell Ready** (~100ms): Basic UI visible to user
3. **Core Services** (~200ms): Essential backend services initialized
4. **Secondary Loading**: Non-essential components loaded in background

The architecture uses a combination of techniques:
- Lightweight shell UI that loads first
- Lazy loading of non-essential components
- Feature flags to control what gets loaded
- Optimized build configuration for small initial bundle

## Development Setup

### Prerequisites

- Rust (1.62+)
- Node.js (16+)
- npm or yarn
- An Anthropic API key

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/your-username/claude-mcp.git
   cd claude-mcp
   ```

2. Install dependencies:
   ```
   # Install frontend dependencies
   cd src-frontend
   npm install
   
   # Return to root and install Rust dependencies
   cd ..
   cargo build
   ```

3. Configure your API key:
   ```
   # Create a config directory
   mkdir -p ~/.config/claude-mcp
   
   # Create a config file (replace YOUR_API_KEY with your actual API key)
   echo '{"api": {"key": "YOUR_API_KEY"}}' > ~/.config/claude-mcp/config.json
   ```

4. Run the development version:
   ```
   npm run tauri dev
   ```

### Building for Production

```
npm run tauri build
```

This will create optimized production builds in the `src-tauri/target/release` directory.

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

You can control these features:
- At build time through Cargo features
- At runtime through environment variables
- Through the config file

## Cross-Platform Considerations

Although this project is primarily designed for Linux, the code is structured to be cross-platform compatible. When pushing to GitHub and pulling on your Linux server:

1. Make sure to run `cargo update` to update dependencies for the Linux environment
2. Check for any platform-specific path issues in the config paths
3. Use the appropriate build targets when building on Linux (`deb` and `appimage`)

## License

[MIT License](LICENSE)
