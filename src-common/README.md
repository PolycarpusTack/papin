# MCP Common Library

A shared Rust library that provides common functionality for all MCP client interfaces (GUI, CLI, and TUI). This library implements the Model Context Protocol (MCP) and provides services for interacting with Claude AI models.

## Components

### MCP Protocol Implementation

- Full implementation of the Model Context Protocol (MCP)
- WebSocket-based real-time communication
- Message streaming support
- Authentication and session management
- Error handling and recovery

### Service Layer

- **McpService**: Core service for MCP interactions
- **ChatService**: High-level service for conversation management
- **AuthService**: Authentication and API key management
- **ModelService**: Model selection and management

### Data Models

- Conversation model with messages
- User, assistant, and system message types
- Model configurations
- Metadata structures

### Configuration Management

- Configuration file handling
- Environment variable support
- User settings management

### Utilities

- Error handling
- Logging
- Token management
- Path resolution

## Usage

This library is designed to be used as a dependency by the various MCP client interfaces. It provides a consistent API for interacting with Claude models, regardless of the interface being used.

```rust
use mcp_common::{init_mcp_service, service::ChatService};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize the MCP service
    let mcp_service = init_mcp_service();
    
    // Create a chat service
    let chat_service = Arc::new(ChatService::new(mcp_service));
    
    // Create a new conversation
    let conversation = chat_service
        .create_conversation("New Conversation", None)
        .await
        .expect("Failed to create conversation");
    
    // Send a message
    let response = chat_service
        .send_message(&conversation.id, "Hello, Claude!")
        .await
        .expect("Failed to send message");
    
    println!("Claude: {}", response.text());
}
```

## Message Streaming

The library supports real-time streaming of responses from Claude:

```rust
// Send a message with streaming response
let mut stream = chat_service
    .send_message_streaming(&conversation_id, "Tell me a story")
    .await
    .expect("Failed to start streaming");

// Process streaming updates
while let Some(result) = stream.recv().await {
    match result {
        Ok(message) => {
            println!("New content: {}", message.text());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            break;
        }
    }
}
```

## Offline Support

The library includes support for local models when offline:

```rust
// The router will automatically select a local model if disconnected
let response = chat_service
    .send_message(&conversation_id, "Are you there?")
    .await
    .expect("Failed to send message");

// Check if the response came from a local model
if let Some(model) = &response.model {
    println!("Response from model: {}", model.name);
}
```

## Thread Safety

All services are designed to be thread-safe and can be wrapped in an `Arc` for sharing between threads:

```rust
let chat_service = Arc::new(ChatService::new(mcp_service));

// Clone to share between threads
let service_clone = chat_service.clone();

tokio::spawn(async move {
    let conversations = service_clone.list_conversations().await.unwrap();
    // ...
});
```

## Error Handling

The library provides a comprehensive error type hierarchy:

```rust
match chat_service.send_message(&conversation_id, "Hello").await {
    Ok(response) => {
        println!("Claude: {}", response.text());
    }
    Err(e) => match e {
        McpError::Network(err) => {
            eprintln!("Network error: {}", err);
            // Handle network errors (try offline mode)
        }
        McpError::Authentication(err) => {
            eprintln!("Authentication error: {}", err);
            // Handle auth errors (prompt for API key)
        }
        McpError::Service(err) => {
            eprintln!("Service error: {}", err);
            // Handle service errors
        }
        _ => {
            eprintln!("Error: {}", e);
        }
    },
}
```

## Configuration

The library uses a shared configuration system that works across all interfaces:

```rust
use mcp_common::config::{Config, load_config, save_config};

// Load configuration
let mut config = load_config().unwrap_or_default();

// Update configuration
config.api_key = Some("your-api-key".to_string());
config.default_model = Some("claude-3-opus-20240229".to_string());

// Save configuration
save_config(&config).expect("Failed to save config");
```
