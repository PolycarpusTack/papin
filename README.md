# MCP Client

The MCP Client is a cross-platform desktop application that provides access to the MCP system with robust offline capabilities and excellent performance.

## Features

- **Rich Conversation UI**: Fully-featured conversation interface with markdown, code blocks, and more
- **Offline Mode**: Continue working even without internet connectivity using embedded LLMs
- **Auto-Updates**: Seamlessly update the application with the latest features and fixes
- **Performance Optimizations**: Efficient memory usage and caching for responsive experience
- **Comprehensive Observability**: Detailed metrics, logging, and telemetry

## Getting Started

### Installation

#### Windows

- Download the MSI installer from the [releases page](https://github.com/your-org/mcp-client/releases)
- Run the installer and follow the on-screen instructions

#### macOS

- Download the DMG file from the [releases page](https://github.com/your-org/mcp-client/releases)
- Open the DMG file and drag the application to your Applications folder

#### Linux

For Debian/Ubuntu:
```bash
sudo apt install ./mcp-client_1.0.0_amd64.deb
```

For Fedora/RHEL:
```bash
sudo rpm -i mcp-client-1.0.0.x86_64.rpm
```

For other distributions, use the AppImage:
```bash
chmod +x MCP-Client-1.0.0.AppImage
./MCP-Client-1.0.0.AppImage
```

### Usage

Refer to the [User Guide](docs/USER_GUIDE.md) for detailed usage instructions.

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

```bash
# For all platforms
npm run tauri build

# For specific platforms
./installers/windows-build.ps1  # Windows
./installers/macos-build.sh     # macOS
./installers/linux-build.sh     # Linux
```

## Documentation

- [Installation Guide](docs/INSTALLATION.md)
- [User Guide](docs/USER_GUIDE.md)
- [Architecture Overview](docs/ARCHITECTURE.md)
- [API Documentation](docs/API.md)
- [Contribution Guidelines](CONTRIBUTING.md)

## Performance Benchmarks

Run the performance benchmarks:

```bash
cargo bench --features benchmarking
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- [Tauri](https://tauri.app/) - Framework for building desktop applications
- [React](https://reactjs.org/) - UI framework
- [Rust](https://www.rust-lang.org/) - Systems programming language
