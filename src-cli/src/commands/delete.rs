use dialoguer::Confirm;
use std::sync::Arc;

use crate::display::{print_error, print_success, show_spinner};
use crate::error::CliResult;
use mcp_common::service::ChatService;

/// Run the delete command
pub async fn run(chat_service: Arc<ChatService>, conversation_id: String) -> CliResult<()> {
    // Confirm deletion
    if !Confirm::new()
        .with_prompt(format!("Are you sure you want to delete conversation {}?", conversation_id))
        .default(false)
        .interact()?
    {
        print_error("Operation cancelled");
        return Ok(());
    }
    
    let spinner = show_spinner();
    spinner.set_message(&format!("Deleting conversation {}...", conversation_id));
    
    // Delete the conversation
    match chat_service.delete_conversation(&conversation_id).await {
        Ok(_) => {
            spinner.success("Conversation deleted");
            print_success(&format!("Deleted conversation {}", conversation_id));
            Ok(())
        }
        Err(e) => {
            spinner.error(&format!("Failed to delete conversation: {}", e));
            Err(e.into())
        }
    }
}
