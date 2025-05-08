# MCP CLI - Command Line Interface for Claude MCP

A fast, powerful command-line interface for interacting with Claude AI models through the Model Context Protocol (MCP).

## Features

- **Complete Command Set**: Full set of commands for managing conversations and interacting with Claude
- **Interactive Mode**: Real-time streaming chat with Claude in your terminal
- **Conversation Management**: Create, list, show, and delete conversations
- **Export/Import**: Export conversations to various formats (JSON, Markdown, plain text)
- **System Messages**: Set system messages for conversation context
- **Model Management**: List models, set default model, change models for conversations
- **Streaming Responses**: Real-time streaming of Claude's responses
- **Rich Formatting**: Color-coded output with various formatting options
- **MCP Protocol**: Full implementation of the Model Context Protocol

## Usage

```bash
# Send a message to Claude
mcp chat -m "What's the weather like today?"

# Start an interactive session
mcp interactive

# Create a new conversation
mcp new -t "Weather Discussion"

# List all conversations
mcp list

# Show a specific conversation
mcp show CONVERSATION_ID

# Delete a conversation
mcp delete CONVERSATION_ID

# Export a conversation to markdown
mcp export CONVERSATION_ID -f markdown -o conversation.md

# Set a system message for a conversation
mcp system CONVERSATION_ID -m "You are a weather expert"

# List available models
mcp model list

# Set default model
mcp model set-default claude-3-opus-20240229

# Change model for a conversation
mcp model set-for-conversation CONVERSATION_ID claude-3-opus-20240229
```

## Interactive Mode

Interactive mode provides a REPL-like interface for conversing with Claude:

```bash
$ mcp interactive
Welcome to Claude MCP Interactive Mode
Type '.help' to see available commands

You> What's the capital of France?
Claude> The capital of France is Paris.

You> .help

===== Available Commands =====
.history    - Show conversation history
.switch     - Switch to another conversation
.new        - Create a new conversation
.system     - Set a system message
.help       - Show this help
.quit       - Exit interactive mode
============================

You> .quit
Goodbye!
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-username/claude-mcp.git
cd claude-mcp

# Build the CLI
cd src-cli
cargo build --release

# Install (optional)
cargo install --path .
```

### Using Pre-built Binaries

Download the appropriate binary for your platform from the [Releases](https://github.com/your-username/claude-mcp/releases) page.

## Configuration

On first run, the CLI will prompt you to configure your API key and other settings. 
You can also manually configure these settings by running:

```bash
mcp setup
```

Configuration is stored in the following location:
- Linux: `~/.config/mcp-cli/config.json`
- macOS: `~/Library/Application Support/mcp-cli/config.json`
- Windows: `%APPDATA%\mcp-cli\config.json`

## Environment Variables

- `MCP_API_KEY`: Your Claude API key (overrides config file)
- `MCP_DEFAULT_MODEL`: Default model to use (overrides config file)
- `MCP_CONFIG_PATH`: Custom path to config file

## Integration with Other Tools

The CLI is designed to work well with other command-line tools:

```bash
# Pipe content to Claude
cat document.txt | mcp chat

# Process Claude's response
mcp chat -m "Summarize this article" -c CONVERSATION_ID | jq .summary > summary.txt

# Use in scripts
mcp chat -m "Translate to French: Hello world" --no-stream > french.txt
```

## License

[MIT License](LICENSE)
