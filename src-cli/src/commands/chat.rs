use dialoguer::Input;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::CliResult;
use crate::display::{format_message, print_error, print_info, MessageFormat, show_spinner};
use mcp_common::{error::McpResult, models::Message, service::ChatService};

/// Run the chat command
pub async fn run(
    chat_service: Arc<ChatService>,
    conversation_id: Option<String>,
    message: Option<String>,
    stream: bool,
) -> CliResult<()> {
    // Get conversation ID
    let conversation_id = match conversation_id {
        Some(id) => id,
        None => {
            // List available conversations
            let conversations = chat_service.list_conversations().await?;
            
            if conversations.is_empty() {
                print_info("No conversations found. Creating a new one...");
                let new_conversation = chat_service.create_conversation("New Conversation", None).await?;
                new_conversation.id
            } else {
                // Let user select from available conversations
                let mut options: Vec<String> = conversations
                    .iter()
                    .map(|c| format!("{} ({})", c.title, c.id))
                    .collect();
                
                options.push("Create a new conversation".to_string());
                
                let selection = dialoguer::Select::new()
                    .with_prompt("Select a conversation")
                    .items(&options)
                    .default(0)
                    .interact()?;
                
                if selection == conversations.len() {
                    // Create a new conversation
                    let title: String = Input::new()
                        .with_prompt("Enter a title for the new conversation")
                        .default("New Conversation".into())
                        .interact_text()?;
                    
                    let new_conversation = chat_service.create_conversation(&title, None).await?;
                    new_conversation.id
                } else {
                    // Extract ID from selected conversation
                    conversations[selection].id.clone()
                }
            }
        }
    };
    
    // Get message content
    let message_content = match message {
        Some(content) => content,
        None => {
            Input::new()
                .with_prompt("Enter your message")
                .interact_text()?
        }
    };
    
    // Send message
    let spinner = show_spinner();
    spinner.set_message("Sending message...");
    
    if stream {
        // Stream response
        let mut stream = chat_service
            .send_message_streaming(&conversation_id, &message_content)
            .await?;
        
        spinner.info("Response:");
        
        // Print user message
        println!("{}", format_message(&Message::user(&message_content), MessageFormat::Colored));
        println!();
        
        // Print assistant response as it streams
        let mut full_message = String::new();
        
        while let Some(result) = stream.recv().await {
            match result {
                Ok(message) => {
                    // Get the new text
                    let text = message.text();
                    
                    // Only print the new part since the last update
                    if text.len() > full_message.len() {
                        let new_text = &text[full_message.len()..];
                        print!("{}", new_text);
                        io::stdout().flush()?;
                        full_message = text;
                    }
                }
                Err(e) => {
                    print_error(&format!("Error receiving message: {}", e));
                    break;
                }
            }
        }
        
        println!("\n");
    } else {
        // Regular response
        match chat_service.send_message(&conversation_id, &message_content).await {
            Ok(response) => {
                spinner.success("Response received");
                
                // Print user message
                println!("{}", format_message(&Message::user(&message_content), MessageFormat::Colored));
                println!();
                
                // Print assistant response
                println!("{}", format_message(&response, MessageFormat::Colored));
                println!();
            }
            Err(e) => {
                spinner.error(&format!("Failed to send message: {}", e));
                return Err(e.into());
            }
        }
    }
    
    Ok(())
}
