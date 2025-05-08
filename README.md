# Claude MCP Client for Linux

A fast, native Linux client for Claude MCP built with Tauri (Rust) and React.

## Features

- **Ultra-fast startup** (<500ms to visible UI)
- **Native performance** through Tauri/Rust backend
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
│   ├── services/            # Backend services
│   └── utils/               # Utility functions
├── src-tauri/               # Tauri configuration
├── src-frontend/            # React frontend
    ├── src/
    │   ├── main.tsx         # React entry point
    │   ├── App.tsx          # Main React component
    │   ├── components/      # UI components
    │   └── lazy/            # Lazy-loaded components
```

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

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/your-org/claude-mcp.git
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

3. Run the development version:
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

[MIT License](LICENSE)# papin
