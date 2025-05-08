use dialoguer::Input;
use std::sync::Arc;

use crate::display::{print_success, show_spinner};
use crate::error::CliResult;
use mcp_common::service::ChatService;

/// Run the system command
pub async fn run(
    chat_service: Arc<ChatService>,
    conversation_id: String,
    message: Option<String>,
) -> CliResult<()> {
    // Get system message content
    let content = match message {
        Some(text) => text,
        None => {
            Input::new()
                .with_prompt("Enter system message")
                .interact_text()?
        }
    };
    
    let spinner = show_spinner();
    spinner.set_message("Setting system message...");
    
    // Set the system message
    match chat_service.set_system_message(&conversation_id, &content).await {
        Ok(_) => {
            spinner.success("System message set");
            print_success(&format!(
                "System message set for conversation {}",
                conversation_id
            ));
            Ok(())
        }
        Err(e) => {
            spinner.error(&format!("Failed to set system message: {}", e));
            Err(e.into())
        }
    }
}
