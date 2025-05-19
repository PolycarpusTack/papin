# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Papin is a cross-platform desktop application that provides access to the MCP system with robust offline capabilities and excellent performance. It's built with a hybrid architecture using:

- **Frontend**: React with TypeScript
- **Backend**: Rust with Tauri
- **Bridge**: Tauri commands for communication between frontend and backend

## Project Roadmap and JIRA Overview

### Current Project Roadmap
- Implement core offline AI capabilities
- Enhance performance optimization system
- Develop comprehensive observability infrastructure
- Expand cross-platform support and compatibility
- Implement advanced security features

### JIRA Project Structure
- **Epics**: Major project milestones and architectural improvements
- **Stories**: Specific feature implementations and enhancements
- **Sprints**: 2-week iterative development cycles
- **Boards**: 
  - Backend Development
  - Frontend Development
  - DevOps and Infrastructure
  - AI and Offline Capabilities

### Key JIRA Tracking Areas
- Performance benchmarks and optimization
- Offline system development
- Security enhancements
- Cross-platform compatibility
- AI integration and local model support

## Common Commands

### Build Commands

```bash
# Build all components
make build-all

# Build specific components
make build-cli
make build-tui
make build-gui

# Build and install all components
make install-all

# Install specific components
make install-cli
make install-tui
make install-gui
```

### Frontend Development

```bash
# Start frontend development server
cd src-frontend && npm run dev

# Start Tauri development
npm run tauri:dev

# Build frontend
cd src-frontend && npm run build
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run unit tests
cargo test --test "*_test"

# Run integration tests
cargo test --test "*_integration_test"

# Run end-to-end tests
cd tests/e2e && npx playwright test

# Run performance benchmarks
cargo bench --features benchmarking
```

### Linting

```bash
# Rust linting
cargo clippy --all-targets --all-features

# Frontend linting
cd src-frontend && npm run lint
```

### Building for Specific Platforms

```bash
# Windows
./installers/windows-build.ps1

# macOS
./installers/macos-build.sh

# Linux
./installers/linux-build.sh
```

## Project Structure

The codebase is organized into several key components:

- `src/` - Core Rust backend code
  - `ai/` - AI capabilities including Claude integration and local models
  - `commands/` - Tauri commands for frontend communication
  - `offline/` - Offline capabilities and local LLM support
  - `observability/` - Metrics, logging, and telemetry
  - `services/` - Core application services
  - `protocols/` - API clients and protocol implementations
  - `security/` - Security features

- `src-common/` - Shared Rust code used across components
  - `config/` - Configuration systems
  - `models/` - Data models
  - `observability/` - Shared observability infrastructure
  - `utils/` - Shared utilities

- `src-frontend/` - React/TypeScript frontend
  - `components/` - React UI components
  - `hooks/` - Custom React hooks
  - `contexts/` - React context providers

- `src-cli/` - Command-line interface
- `src-tui/` - Text user interface
- `src-tauri/` - Tauri configuration and platform integration

## Key Architecture Components

### Observability System

The observability system includes three main components:
- Metrics collection for performance and usage statistics
- Enhanced logging with structured logs
- Privacy-focused telemetry system

### Offline System

The offline system enables operation without internet:
- Local LLM support for offline AI capabilities
- Checkpointing for saving/restoring conversation state
- Synchronization for two-way data syncing

### Optimization System

Performance optimizations include:
- Memory management for efficient resource usage
- Caching system for improved response times

## Feature Flags

The project uses feature flags to enable/disable functionality:

```
[features]
default = []
memory-optimizations = ["mimalloc"]
benchmarking = ["criterion"]
telemetry = []
canary = []
custom-protocol = ["tauri/custom-protocol"]
```

## Testing Strategy

The codebase employs multiple testing approaches:
- Unit tests for individual components
- Integration tests for component interactions
- End-to-end tests for complete user workflows
- Performance benchmarks for critical components

Tests are organized in the `tests/` directory:
- `tests/unit/` - Unit tests
- `tests/integration/` - Integration tests
- `tests/e2e/` - End-to-end tests
- `benches/` - Performance benchmarks