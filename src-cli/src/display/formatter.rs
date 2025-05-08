use console::{style, Style};
use mcp_common::models::{Conversation, Message, MessageRole};

/// Message format options
pub enum MessageFormat {
    Plain,
    Colored,
    Markdown,
    Json,
}

/// Format a conversation based on the selected format
pub fn format_conversation(conversation: &Conversation, format: MessageFormat) -> String {
    match format {
        MessageFormat::Plain => format_conversation_plain(conversation),
        MessageFormat::Colored => format_conversation_colored(conversation),
        MessageFormat::Markdown => format_conversation_markdown(conversation),
        MessageFormat::Json => format_conversation_json(conversation),
    }
}

/// Format a message based on the selected format
pub fn format_message(message: &Message, format: MessageFormat) -> String {
    match format {
        MessageFormat::Plain => format_message_plain(message),
        MessageFormat::Colored => format_message_colored(message),
        MessageFormat::Markdown => format_message_markdown(message),
        MessageFormat::Json => format_message_json(message),
    }
}

/// Format metadata as a string
pub fn format_metadata(metadata: &serde_json::Value) -> String {
    let formatted = serde_json::to_string_pretty(metadata).unwrap_or_default();
    
    if formatted == "null" {
        String::new()
    } else {
        formatted
    }
}

// Format a conversation in plain text
fn format_conversation_plain(conversation: &Conversation) -> String {
    let mut result = String::new();
    
    result.push_str(&format!("Conversation: {}\n", conversation.title));
    result.push_str(&format!("Model: {}\n", conversation.model.name));
    result.push_str(&format!("ID: {}\n", conversation.id));
    result.push_str(&format!("Messages: {}\n\n", conversation.messages.len()));
    
    for message in &conversation.messages {
        result.push_str(&format_message_plain(message));
        result.push_str("\n\n");
    }
    
    result
}

// Format a conversation with colors
fn format_conversation_colored(conversation: &Conversation) -> String {
    let mut result = String::new();
    
    let title_style = Style::new().cyan().bold();
    let model_style = Style::new().yellow();
    let id_style = Style::new().dim();
    
    result.push_str(&format!("{}: {}\n", title_style.apply_to("Conversation"), conversation.title));
    result.push_str(&format!("{}: {}\n", style("Model").yellow(), model_style.apply_to(&conversation.model.name)));
    result.push_str(&format!("{}: {}\n", style("ID").dim(), id_style.apply_to(&conversation.id)));
    result.push_str(&format!("{}: {}\n\n", style("Messages").dim(), conversation.messages.len()));
    
    for message in &conversation.messages {
        result.push_str(&format_message_colored(message));
        result.push_str("\n\n");
    }
    
    result
}

// Format a conversation in markdown
fn format_conversation_markdown(conversation: &Conversation) -> String {
    let mut result = String::new();
    
    result.push_str(&format!("# {}\n\n", conversation.title));
    result.push_str(&format!("**Model**: {}\n\n", conversation.model.name));
    result.push_str(&format!("**ID**: {}\n\n", conversation.id));
    result.push_str(&format!("**Messages**: {}\n\n", conversation.messages.len()));
    
    for message in &conversation.messages {
        result.push_str(&format_message_markdown(message));
        result.push_str("\n\n");
    }
    
    result
}

// Format a conversation as JSON
fn format_conversation_json(conversation: &Conversation) -> String {
    match serde_json::to_string_pretty(conversation) {
        Ok(json) => json,
        Err(_) => String::from("Error: Could not serialize conversation to JSON"),
    }
}

// Format a message in plain text
fn format_message_plain(message: &Message) -> String {
    let role = match message.role {
        MessageRole::User => "User",
        MessageRole::Assistant => "Assistant",
        MessageRole::System => "System",
    };
    
    format!("[{}] {}\n{}", role, message.timestamp(), message.text())
}

// Format a message with colors
fn format_message_colored(message: &Message) -> String {
    let (role, style) = match message.role {
        MessageRole::User => ("User", Style::new().green().bold()),
        MessageRole::Assistant => ("Assistant", Style::new().blue().bold()),
        MessageRole::System => ("System", Style::new().yellow().bold()),
    };
    
    let timestamp = Style::new().dim().apply_to(message.timestamp());
    
    format!(
        "[{}] {}\n{}",
        style.apply_to(role),
        timestamp,
        message.text()
    )
}

// Format a message in markdown
fn format_message_markdown(message: &Message) -> String {
    let heading = match message.role {
        MessageRole::User => "## 👤 User",
        MessageRole::Assistant => "## 🤖 Assistant",
        MessageRole::System => "## ⚙️ System",
    };
    
    format!(
        "{} ({})\n\n{}",
        heading,
        message.timestamp(),
        message.text()
    )
}

// Format a message as JSON
fn format_message_json(message: &Message) -> String {
    match serde_json::to_string_pretty(message) {
        Ok(json) => json,
        Err(_) => String::from("Error: Could not serialize message to JSON"),
    }
}
