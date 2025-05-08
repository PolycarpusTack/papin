# MCP TUI - Text User Interface for Claude MCP

A terminal-based user interface for interacting with Claude AI models through the Model Context Protocol (MCP).

## Features

- **Rich Terminal UI**: Full-featured interface built with Ratatui and Crossterm
- **Conversation Management**: View, create, select, and delete conversations
- **Real-time Streaming**: See Claude's responses as they're generated
- **Keyboard Navigation**: Vim-inspired keyboard shortcuts
- **Command Mode**: Quick access to commands via command palette
- **Multiple Views**: Conversations list, chat view, settings, help
- **Theming Support**: Customizable colors and styling
- **MCP Protocol**: Full implementation of the Model Context Protocol
- **Offline Support**: Automatic fallback to local models when offline

## Usage

```bash
# Start the TUI
mcp-tui

# Start with a specific conversation
mcp-tui --conversation CONVERSATION_ID
```

## Keyboard Shortcuts

### Global

- `q` - Quit application
- `?` - Show help screen
- `:` - Enter command mode
- `s` - Open settings

### Navigation

- `j/k` or Arrow keys - Navigate up/down in lists
- `Enter` - Select conversation
- `Esc` - Return to normal mode
- `Tab` - Cycle through sections

### Conversation Management

- `n` - Create new conversation
- `d` - Delete current conversation
- `r` - Reload conversations
- `PageUp/PageDown` - Scroll through history

### Chat Mode

- `Ctrl+Enter` - Send message
- `Esc` - Exit chat mode

## Command Mode

The TUI features a vim-like command mode activated with `:`. Available commands:

- `:quit` or `:q` - Quit the application
- `:new [title]` or `:n [title]` - Create a new conversation
- `:delete` or `:d` - Delete the current conversation
- `:reload` or `:r` - Reload conversations
- `:help` or `:h` - Show help screen
- `:settings` or `:s` - Open settings

## User Interface

```
┌─ Status Bar ───────────────────────────────────────────────────────────────┐
│ NORMAL | My Conversation | claude-3-opus-20240229                          │
├─ Conversations ──────────┬─ Chat ─────────────────────────────────────────┐
│                          │                                                 │
│ > Project Ideas          │ You: What are some project ideas for Rust?      │
│   Meeting Notes          │                                                 │
│   Brainstorming          │ Claude: Here are some project ideas for Rust:   │
│   Research               │                                                 │
│   Claude Exploration     │ 1. Command-line tools                           │
│   Coding Help            │ 2. WebAssembly applications                     │
│   Travel Planning        │ 3. Embedded systems                             │
│                          │ 4. Network services                             │
│                          │ 5. Game development                             │
│                          │                                                 │
│                          │ Would you like me to elaborate on any of these? │
│                          │                                                 │
│                          │                                                 │
│                          │                                                 │
├──────────────────────────┴─────────────────────────────────────────────────┤
│ Type a message...                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-username/claude-mcp.git
cd claude-mcp

# Build the TUI
cd src-tui
cargo build --release

# Install (optional)
cargo install --path .
```

### Using Pre-built Binaries

Download the appropriate binary for your platform from the [Releases](https://github.com/your-username/claude-mcp/releases) page.

## Configuration

The TUI shares configuration with the main application and CLI. Configuration is stored in:

- Linux: `~/.config/mcp/config.json`
- macOS: `~/Library/Application Support/mcp/config.json`
- Windows: `%APPDATA%\mcp\config.json`

## Environment Variables

- `MCP_API_KEY`: Your Claude API key (overrides config file)
- `MCP_DEFAULT_MODEL`: Default model to use (overrides config file)
- `MCP_CONFIG_PATH`: Custom path to config file
- `MCP_TUI_COLORS`: Set to `true` to enable colors, `false` to disable
- `MCP_TUI_THEME`: Set the theme (`default`, `dark`, `light`, `high-contrast`)

## Advanced Features

### Custom Key Bindings

Create a file at `~/.config/mcp/tui-keybindings.json` to customize key bindings.

### Themes

The TUI supports multiple themes that can be selected in the settings screen or via the `MCP_TUI_THEME` environment variable.

### Local Model Integration

When offline, the TUI automatically switches to available local models, providing a seamless experience even without internet access.

## License

[MIT License](LICENSE)
