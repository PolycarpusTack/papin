use console::{style, Term};
use dialoguer::{Input, Select};
use std::io::{self, Write};
use std::sync::Arc;

use crate::commands;
use crate::display::{format_message, print_error, print_info, print_success, MessageFormat};
use crate::error::CliResult;
use mcp_common::{error::McpResult, models::Message, service::ChatService};

// Commands available in interactive mode
enum InteractiveCommand {
    SendMessage,
    ShowHistory,
    SwitchConversation,
    NewConversation,
    SystemMessage,
    Help,
    Quit,
}

/// Run the interactive command
pub async fn run(
    chat_service: Arc<ChatService>,
    conversation_id: Option<String>,
) -> CliResult<()> {
    // Clear screen
    let term = Term::stdout();
    term.clear_screen()?;
    
    print_info("Welcome to Claude MCP Interactive Mode");
    print_info("Type '.help' to see available commands");
    println!();
    
    // Get or create conversation
    let mut current_conversation_id = match conversation_id {
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
                
                let selection = Select::new()
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
    
    // Get conversation details
    let conversation = chat_service.get_conversation(&current_conversation_id).await?;
    print_success(&format!("Conversation: {} ({})", conversation.title, conversation.model.name));
    
    // Main interaction loop
    loop {
        // Get input from user
        let input: String = Input::new()
            .with_prompt(style("You").green().to_string())
            .interact_text()?;
        
        // Check if input is a command
        if input.starts_with('.') {
            match parse_command(&input) {
                InteractiveCommand::SendMessage => {
                    print_error("Not a valid command. Type '.help' to see available commands");
                }
                InteractiveCommand::ShowHistory => {
                    // Fetch and show conversation history
                    let conversation = chat_service.get_conversation(&current_conversation_id).await?;
                    
                    println!("\n===== Conversation History =====");
                    for message in &conversation.messages {
                        println!("{}\n", format_message(message, MessageFormat::Colored));
                    }
                    println!("================================\n");
                }
                InteractiveCommand::SwitchConversation => {
                    // List and select a conversation
                    let conversations = chat_service.list_conversations().await?;
                    
                    if conversations.is_empty() {
                        print_error("No conversations found");
                        continue;
                    }
                    
                    let options: Vec<String> = conversations
                        .iter()
                        .map(|c| format!("{} ({})", c.title, c.id))
                        .collect();
                    
                    let selection = Select::new()
                        .with_prompt("Select a conversation")
                        .items(&options)
                        .default(0)
                        .interact()?;
                    
                    // Switch to selected conversation
                    current_conversation_id = conversations[selection].id.clone();
                    print_success(&format!(
                        "Switched to conversation: {} ({})",
                        conversations[selection].title,
                        conversations[selection].model.name
                    ));
                }
                InteractiveCommand::NewConversation => {
                    // Create a new conversation
                    let result = commands::new::run(chat_service.clone(), None, None).await;
                    
                    if let Ok(()) = result {
                        // Get the newly created conversation
                        let conversations = chat_service.list_conversations().await?;
                        if let Some(newest) = conversations.first() {
                            current_conversation_id = newest.id.clone();
                            print_success(&format!(
                                "Now using conversation: {} ({})",
                                newest.title,
                                newest.model.name
                            ));
                        }
                    }
                }
                InteractiveCommand::SystemMessage => {
                    // Set system message
                    let content: String = Input::new()
                        .with_prompt("Enter system message")
                        .interact_text()?;
                    
                    chat_service
                        .set_system_message(&current_conversation_id, &content)
                        .await?;
                    
                    print_success("System message set");
                }
                InteractiveCommand::Help => {
                    show_help();
                }
                InteractiveCommand::Quit => {
                    print_info("Goodbye!");
                    break;
                }
            }
        } else {
            // Not a command, send as a message
            println!();
            
            match chat_service
                .send_message_streaming(&current_conversation_id, &input)
                .await
            {
                Ok(mut stream) => {
                    // Print assistant header
                    print!("{} ", style("Claude").blue().bold());
                    io::stdout().flush()?;
                    
                    // Print response as it streams
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
                }
                Err(e) => {
                    print_error(&format!("Failed to send message: {}", e));
                }
            }
        }
    }
    
    Ok(())
}

// Parse a command from user input
fn parse_command(input: &str) -> InteractiveCommand {
    let input = input.trim();
    
    match input {
        ".history" => InteractiveCommand::ShowHistory,
        ".switch" => InteractiveCommand::SwitchConversation,
        ".new" => InteractiveCommand::NewConversation,
        ".system" => InteractiveCommand::SystemMessage,
        ".help" => InteractiveCommand::Help,
        ".quit" | ".exit" => InteractiveCommand::Quit,
        _ => InteractiveCommand::SendMessage,
    }
}

// Display help information
fn show_help() {
    println!("\n===== Available Commands =====");
    println!(".history    - Show conversation history");
    println!(".switch     - Switch to another conversation");
    println!(".new        - Create a new conversation");
    println!(".system     - Set a system message");
    println!(".help       - Show this help");
    println!(".quit       - Exit interactive mode");
    println!("============================\n");
}
