# MCP Client

The MCP Client is a cross-platform desktop application for accessing and interacting with the MCP system. It provides a rich set of features, robust offline capabilities, and excellent performance.

## Features

### Core Features

- **Rich Text Conversations**: Fully-featured conversation UI with support for markdown, code blocks, and more
- **Offline Mode**: Continue working even without an internet connection
- **Local LLM Support**: Use embedded models for inference when offline
- **Performance Optimizations**: Efficient memory usage and caching for responsive experience
- **Auto-Updates**: Seamless background updates to keep the application current

### Performance and Optimization

- **Memory Management**: Intelligent memory usage with configurable limits and garbage collection
- **Caching System**: API and resource caching for faster response times
- **Resource Monitoring**: Real-time monitoring of system resources
- **Checkpointing**: Save and restore conversation state
- **Synchronization**: Two-way sync mechanism for seamless transitions between online and offline modes

### Observability

- **Metrics Collection**: Comprehensive metrics for monitoring application performance
- **Structured Logging**: Enhanced logging system with filtering and search capabilities
- **Telemetry**: Privacy-focused telemetry system for tracking feature usage and errors
- **Canary Releases**: Gradual rollout of new features with automatic rollback
- **Resource Dashboard**: Visual monitoring of system resources and performance

## System Requirements

- **Windows**: Windows 10 or later (64-bit)
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Ubuntu 20.04 or later, or equivalent

## Installation

### Windows

1. Download the MSI installer from the releases page
2. Run the installer and follow the on-screen instructions
3. Alternatively, download the portable EXE if you prefer not to install

### macOS

1. Download the DMG file from the releases page
2. Open the DMG file and drag the application to your Applications folder
3. First-time users may need to approve the application in System Preferences > Security & Privacy

### Linux

#### Debian/Ubuntu

```bash
sudo apt install ./mcp-client_1.0.0_amd64.deb
```

#### Fedora/RHEL

```bash
sudo rpm -i mcp-client-1.0.0.x86_64.rpm
```

#### AppImage

```bash
chmod +x MCP-Client-1.0.0.AppImage
./MCP-Client-1.0.0.AppImage
```

## Getting Started

1. Launch the MCP Client from your applications menu or desktop shortcut
2. Sign in with your MCP account credentials
3. For offline usage, go to Settings > Offline and enable offline mode
4. Configure auto-update settings in Settings > Updates

## Development

### Prerequisites

- Node.js 16 or later
- Rust 1.65 or later
- Tauri CLI

### Setup

1. Clone the repository:

```bash
git clone https://github.com/your-org/mcp-client.git
cd mcp-client
```

2. Install dependencies:

```bash
npm install
```

3. Start the development server:

```bash
npm run tauri dev
```

### Building

#### For all platforms:

```bash
npm run tauri build
```

#### For specific platforms:

- Windows: `./installers/windows-build.ps1`
- macOS: `./installers/macos-build.sh`
- Linux: `./installers/linux-build.sh`

## Architecture

The MCP Client is built with a hybrid architecture:

- **Frontend**: React with TypeScript
- **Backend**: Rust with Tauri
- **Bridge**: Tauri commands for communication between frontend and backend

Key components:

- **Observability**: Metrics, logging, and telemetry systems
- **Offline**: Local-first architecture with embedded LLMs
- **Auto-Update**: Background update system with progressive rollouts
- **Optimization**: Memory management and caching
- **Distribution**: Platform-specific installers and updates

## Configuration

### Memory Management

Memory management can be configured through the Settings > Performance menu or via the configuration file:

```json
{
  "memory": {
    "maxMemoryMb": 1024,
    "thresholdMemoryMb": 768,
    "maxContextTokens": 8192,
    "enabled": true,
    "checkIntervalSecs": 30
  }
}
```

### Caching

Caching can be configured through the Settings > Performance menu:

```json
{
  "apiCache": {
    "maxEntries": 1000,
    "ttlSeconds": 300,
    "persist": true,
    "enabled": true,
    "cleanupIntervalSecs": 60
  },
  "resourceCache": {
    "maxEntries": 200,
    "ttlSeconds": 3600,
    "persist": true,
    "enabled": true,
    "cleanupIntervalSecs": 300
  }
}
```

### Auto-Update

Auto-update can be configured through the Settings > Updates menu:

```json
{
  "updater": {
    "enabled": true,
    "checkInterval": 24,
    "autoDownload": true,
    "autoInstall": false
  }
}
```

## Troubleshooting

### Common Issues

#### Application Won't Start

1. Check system requirements
2. Verify installation integrity
3. Check system logs for errors

#### High Memory Usage

1. Adjust memory limits in Settings > Performance
2. Close other memory-intensive applications
3. Restart the application to clear caches

#### Offline Mode Not Working

1. Ensure offline mode is enabled in Settings > Offline
2. Check that local models are downloaded
3. Verify storage permissions

#### Updates Not Installing

1. Check internet connection
2. Verify update settings in Settings > Updates
3. Try manual update by downloading latest version

### Logs

Log files are located at:

- Windows: `%APPDATA%\MCP-Client\logs`
- macOS: `~/Library/Application Support/MCP-Client/logs`
- Linux: `~/.config/MCP-Client/logs`

## License

MCP Client is licensed under the [MIT License](LICENSE).