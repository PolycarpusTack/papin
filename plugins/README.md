# MCP Client Plugin System

This directory contains example plugins for the MCP Client. The plugin system enables extending the functionality of the MCP Client through WebAssembly (WASM) modules that run in a secure sandbox.

## Overview

The MCP Client plugin system allows developers to create custom extensions that can:

- Process messages before they are sent to Claude
- Process responses from Claude
- Add new commands and functionality
- Interact with external services
- Customize the UI

Plugins are implemented in Rust or other languages that compile to WebAssembly, and run in a secure sandbox with limited access to system resources.

## Plugin Architecture

Each plugin consists of:

1. A **manifest file** (manifest.json) that describes the plugin, its capabilities, and required permissions
2. A **WebAssembly module** (*.wasm) that contains the plugin code
3. Optional **assets** such as images, stylesheets, or other resources

Plugins interact with the host application through a defined API that provides:

- Hook registration for responding to events
- Permission-based access to resources
- API access for interacting with Claude and other services
- Configuration management
- Logging facilities

## Example Plugins

This directory contains the following example plugins:

### Translation Plugin

A plugin that translates messages between different languages:

- Features automatic translation of incoming/outgoing messages
- Supports multiple languages 
- Configurable translation settings

### GitHub Code Snippets

A plugin that fetches code snippets from GitHub repositories:

- Use `/github` command to fetch code
- Specify repositories, file paths, and line ranges
- Syntax highlighting and formatting options

### Meeting Summarizer

A plugin that generates structured summaries from meeting transcripts:

- Extract action items, decisions, and participants
- Multiple summary formats (detailed, concise, bullet points)
- Automatic detection of meeting content

## Plugin Development

To create your own plugin:

1. Create a new directory for your plugin
2. Create a manifest.json file with plugin metadata
3. Implement your plugin code in Rust or another language that compiles to WebAssembly
4. Build your plugin to produce a .wasm file
5. Test your plugin in the MCP Client

## Plugin Manifest

Each plugin requires a manifest.json file with the following structure:

```json
{
  "name": "plugin-name",
  "displayName": "Plugin Display Name",
  "version": "1.0.0",
  "description": "Description of the plugin",
  "author": "Your Name",
  "license": "MIT",
  "main": "plugin.wasm",
  "permissions": [
    "permissions:needed"
  ],
  "hooks": [
    "hooks:to:register"
  ],
  "config": {
    "settings": [
      {
        "name": "settingName",
        "type": "string",
        "label": "Setting Label",
        "description": "Setting Description",
        "default": "default value"
      }
    ]
  }
}
```

## Security Considerations

Plugins run in a secure WebAssembly sandbox with the following restrictions:

- Limited memory access (only to their own memory)
- No direct file system access
- No direct network access
- No access to system resources
- Permission-based access to APIs

All plugins must request appropriate permissions, and users must approve these permissions before they can be used.

## Available Permissions

Plugins can request the following permissions:

- `conversations:read` - Read conversations and messages
- `conversations:write` - Create/modify conversations and messages
- `models:read` - Read available models
- `models:use` - Use specific models
- `system:settings` - Access system settings
- `user:preferences` - Access user preferences
- `network:{domain}` - Connect to a specific domain
- `ui:display` - Show UI elements
- `ui:interact` - Interact with user through UI elements

## Available Hooks

Plugins can register handlers for the following hooks:

- `message:pre-process` - Process a message before sending to Claude
- `message:post-process` - Process a message after receiving from Claude
- `conversation:create` - Called when a new conversation is created
- `conversation:open` - Called when a conversation is opened
- `conversation:close` - Called when a conversation is closed
- `application:start` - Called when the application starts
- `application:shutdown` - Called when the application shuts down
- `ui:render` - Custom UI rendering

## Building Plugins

To build a plugin:

1. Set up a Rust project with appropriate dependencies:
   ```toml
   [package]
   name = "my-plugin"
   version = "0.1.0"
   edition = "2021"
   
   [lib]
   crate-type = ["cdylib"]
   
   [dependencies]
   wasm-bindgen = "0.2"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   ```

2. Implement your plugin code:
   ```rust
   use wasm_bindgen::prelude::*;
   
   #[wasm_bindgen]
   pub fn init() {
       // Register the plugin
       host::register_plugin();
       
       // Register hooks
       host::register_hook("message:pre-process", pre_process_message);
       
       host::log_message("info", "My plugin initialized");
   }
   
   fn pre_process_message(context_ptr: i32) -> i32 {
       // Process message
       0
   }
   
   #[wasm_bindgen]
   extern "C" {
       pub mod host {
           pub fn register_plugin() -> i32;
           pub fn register_hook(hook_name: &str, callback_ptr: fn(i32) -> i32) -> i32;
           pub fn log_message(level: &str, message: &str) -> i32;
           // Other host functions
       }
   }
   ```

3. Build your plugin:
   ```sh
   cargo build --target wasm32-unknown-unknown --release
   ```

4. Package your plugin:
   ```sh
   mkdir -p my-plugin
   cp manifest.json my-plugin/
   cp target/wasm32-unknown-unknown/release/my_plugin.wasm my-plugin/plugin.wasm
   zip -r my-plugin.zip my-plugin/
   ```

## Installing Plugins

To install a plugin in the MCP Client:

1. Open the Plugin Manager in the MCP Client
2. Click "Install from File" or "Browse Plugin Repository"
3. Select your plugin package (.zip file)
4. Review the requested permissions
5. Confirm installation

## Plugin API Reference

### Host Functions

The following host functions are available to plugins:

- `register_plugin()` - Register the plugin with the host
- `register_hook(hook_name, callback_ptr)` - Register a hook handler
- `log_message(level, message)` - Log a message
- `read_memory(ptr)` - Read from memory
- `write_memory(ptr, len)` - Write to memory
- `get_settings()` - Get plugin settings
- `request_permission(permission)` - Request additional permissions
- `http_request(url, method, headers, body)` - Make an HTTP request

### Hook Handlers

Hook handlers receive a pointer to a context object and return a status code:

- Return 0 to indicate no changes
- Return a pointer to modified context to replace the original
