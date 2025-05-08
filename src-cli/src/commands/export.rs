use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use crate::display::{format_conversation, print_error, print_success, show_spinner, MessageFormat};
use crate::error::CliResult;
use mcp_common::service::ChatService;

/// Run the export command
pub async fn run(
    chat_service: Arc<ChatService>,
    conversation_id: String,
    format: String,
    output: Option<String>,
) -> CliResult<()> {
    let spinner = show_spinner();
    spinner.set_message(&format!("Loading conversation {}...", conversation_id));
    
    // Load the conversation
    let conversation = match chat_service.get_conversation(&conversation_id).await {
        Ok(conv) => {
            spinner.success("Conversation loaded");
            conv
        }
        Err(e) => {
            spinner.error(&format!("Failed to load conversation: {}", e));
            return Err(e.into());
        }
    };
    
    // Determine format
    let format_mode = match format.to_lowercase().as_str() {
        "json" => MessageFormat::Json,
        "markdown" | "md" => MessageFormat::Markdown,
        "text" | "txt" => MessageFormat::Plain,
        _ => {
            print_error(&format!("Unknown format: {}", format));
            return Ok(());
        }
    };
    
    // Format the conversation
    let formatted = format_conversation(&conversation, format_mode);
    
    // Output the formatted conversation
    match output {
        Some(path) => {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(&path).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            
            // Write to file
            fs::write(&path, formatted)?;
            print_success(&format!("Conversation exported to {}", path));
        }
        None => {
            // Write to stdout
            io::stdout().write_all(formatted.as_bytes())?;
            println!();
        }
    }
    
    Ok(())
}
