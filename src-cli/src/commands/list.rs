use console::Style;
use std::sync::Arc;

use crate::display::{print_info, show_spinner, TableColumn, print_table};
use crate::error::CliResult;
use mcp_common::service::ChatService;

/// Run the list command
pub async fn run(chat_service: Arc<ChatService>) -> CliResult<()> {
    let spinner = show_spinner();
    spinner.set_message("Loading conversations...");
    
    let conversations = chat_service.list_conversations().await?;
    
    if conversations.is_empty() {
        spinner.info("No conversations found");
        return Ok(());
    }
    
    spinner.success(&format!("Found {} conversations", conversations.len()));
    
    // Define table columns
    let columns = vec![
        TableColumn {
            title: "ID".to_string(),
            width: 12,
            style: Some(Style::new().dim()),
        },
        TableColumn {
            title: "Title".to_string(),
            width: 30,
            style: Some(Style::new().cyan()),
        },
        TableColumn {
            title: "Model".to_string(),
            width: 20,
            style: Some(Style::new().yellow()),
        },
        TableColumn {
            title: "Messages".to_string(),
            width: 8,
            style: None,
        },
        TableColumn {
            title: "Last Updated".to_string(),
            width: 20,
            style: None,
        },
    ];
    
    // Prepare rows
    let mut rows = Vec::new();
    
    for conversation in conversations {
        let updated_at = chrono::DateTime::<chrono::Local>::from(conversation.updated_at)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        
        rows.push(vec![
            conversation.id[0..10].to_string() + "..",
            conversation.title,
            conversation.model.name,
            conversation.messages.len().to_string(),
            updated_at,
        ]);
    }
    
    // Print table
    print_table(&columns, &rows)?;
    
    Ok(())
}
