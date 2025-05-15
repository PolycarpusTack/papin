# Papin - MCP Client

Papin is a cross-platform desktop application that provides access to the Model Context Protocol (MCP) system with robust offline capabilities and excellent performance.

## Features

- **Multiple Interfaces**: GUI, CLI, and TUI interfaces for flexibility
- **Rich Conversation UI**: Fully-featured conversation interface with markdown, code blocks, and more
- **Offline Mode**: Continue working even without internet connectivity using embedded LLMs
- **Cross-Platform**: Runs natively on Windows, macOS, and Linux
- **Performance Optimizations**: Efficient memory usage and platform-specific optimizations
- **Plugin System**: Extendable functionality through WASM-based plugins
- **Comprehensive Observability**: Detailed metrics, logging, and telemetry

## Getting Started

### Installation

#### Windows

- Download the MSI installer from the [releases page](https://github.com/your-org/papin/releases)
- Run the installer and follow the on-screen instructions

#### macOS

- Download the DMG file from the [releases page](https://github.com/your-org/papin/releases)
- Open the DMG file and drag the application to your Applications folder

#### Linux

For Debian/Ubuntu:
```bash
sudo apt install ./papin_1.0.0_amd64.deb
```

For Fedora/RHEL:
```bash
sudo rpm -i papin-1.0.0.x86_64.rpm
```

For other distributions, use the AppImage:
```bash
chmod +x Papin-1.0.0.AppImage
./Papin-1.0.0.AppImage
```

### Using Different Interfaces

Papin provides three interfaces to suit different workflows:

#### GUI (Desktop Application)

The GUI provides a rich, user-friendly interface for desktop environments.

```bash
# Launch the GUI application
papin-gui
```

#### CLI (Command Line Interface)

The CLI provides a powerful command-line interface for terminal operations.

```bash
# Launch interactive mode
papin

# Create a new conversation
papin new "My conversation"

# Send a message to a conversation
papin chat 123456 "Hello, how are you?"

# List all conversations
papin list
```

#### TUI (Text User Interface)

The TUI provides a full-screen terminal interface with interactive capabilities.

```bash
# Launch the TUI
papin-tui
```

## Offline Capabilities

Papin supports offline mode with local LLM providers:

1. Enable offline mode in the settings
2. Configure your preferred local LLM provider (Ollama, LocalAI, llama.cpp)
3. Download models for offline use

See the [Offline Mode Documentation](docs/local_llm_integration.md) for details.

## Development

### Prerequisites

- Node.js 16 or later
- Rust 1.65 or later
- Tauri CLI

### Setup

1. Clone the repository:
```bash
git clone https://github.com/your-org/papin.git
cd papin
```

2. Install dependencies:
```bash
cargo install --path .
npm install
```

3. Start the development server:
```bash
npm run tauri:dev
```

### Building

```bash
# For all platforms
make build-all

# For specific components
make build-cli
make build-tui
make build-gui

# Platform-specific builds
./scripts/build-windows.ps1  # Windows
./scripts/build-macos.sh     # macOS
./scripts/build-linux.sh     # Linux
```

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Installation Guide](docs/INSTALLATION.md)
- [User Guide](docs/USER_GUIDE.md)
- [Local LLM Integration](docs/local_llm_integration.md)
- [Plugin System Design](docs/plugin_system_design.md)
- [MCP Protocol](MCP_README.md)
- [Current Project Status](PROJECT_STATUS.md)

## Performance Benchmarks

Run the performance benchmarks:

```bash
cargo bench --features benchmarking
```

## Contributing

We welcome contributions to Papin! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- [Tauri](https://tauri.app/) - Framework for building desktop applications
- [React](https://reactjs.org/) - UI framework
- [Rust](https://www.rust-lang.org/) - Systems programming language