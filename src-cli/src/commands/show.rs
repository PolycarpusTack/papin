use std::sync::Arc;

use crate::display::{format_conversation, print_error, show_spinner, MessageFormat};
use crate::error::CliResult;
use mcp_common::service::ChatService;

/// Run the show command
pub async fn run(chat_service: Arc<ChatService>, conversation_id: String) -> CliResult<()> {
    let spinner = show_spinner();
    spinner.set_message(&format!("Loading conversation {}...", conversation_id));
    
    // Load the conversation
    match chat_service.get_conversation(&conversation_id).await {
        Ok(conversation) => {
            spinner.success("Conversation loaded");
            
            // Format and print the conversation
            let formatted = format_conversation(&conversation, MessageFormat::Colored);
            println!("{}", formatted);
            
            Ok(())
        }
        Err(e) => {
            spinner.error(&format!("Failed to load conversation: {}", e));
            Err(e.into())
        }
    }
}
