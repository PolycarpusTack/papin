use chrono::{DateTime, Local};
use mcp_common::models::Message;
use ratatui::style::{Color, Style};

/// Format a timestamp as a readable string
pub fn format_timestamp(timestamp: &DateTime<Local>) -> String {
    timestamp.format("%Y-%m-%d %H:%M").to_string()
}

/// Get the color for a message role
pub fn get_role_color(role: &str) -> Color {
    match role {
        "user" => Color::Green,
        "assistant" => Color::Blue,
        "system" => Color::Yellow,
        _ => Color::White,
    }
}

/// Format a message for display
pub fn format_message(message: &Message) -> String {
    let prefix = match message.role.as_str() {
        "user" => "You",
        "assistant" => "Claude",
        "system" => "System",
        _ => "Unknown",
    };
    
    let content = message.content.iter()
        .filter_map(|c| c.text.clone())
        .collect::<Vec<String>>()
        .join("\n");
    
    format!("{}: {}", prefix, content)
}

/// Get the style for a message role
pub fn get_role_style(role: &str) -> Style {
    Style::default().fg(get_role_color(role))
}

/// Truncate a string to a maximum length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Generate a unique ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Parse a command string into tokens
pub fn parse_command(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;
    
    for c in command.chars() {
        if escape_next {
            current_token.push(c);
            escape_next = false;
        } else if c == '\\' {
            escape_next = true;
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current_token.is_empty() {
                tokens.push(current_token);
                current_token = String::new();
            }
        } else {
            current_token.push(c);
        }
    }
    
    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    
    tokens
}
