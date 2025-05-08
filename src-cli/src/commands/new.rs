use dialoguer::Input;
use std::sync::Arc;

use crate::display::{print_success, show_spinner};
use crate::error::CliResult;
use mcp_common::{models::Model, service::ChatService};

/// Run the new command
pub async fn run(
    chat_service: Arc<ChatService>,
    title: Option<String>,
    model_id: Option<String>,
) -> CliResult<()> {
    // Get title
    let title = match title {
        Some(t) => t,
        None => {
            Input::new()
                .with_prompt("Enter a title for the new conversation")
                .default("New Conversation".into())
                .interact_text()?
        }
    };
    
    // Get model
    let model = if let Some(model_id) = model_id {
        // Get available models
        let models = chat_service.available_models().await?;
        
        // Find requested model
        models
            .into_iter()
            .find(|m| m.id == model_id)
            .unwrap_or_else(Model::default_claude)
    } else {
        // Get available models
        let spinner = show_spinner();
        spinner.set_message("Loading available models...");
        
        let models = chat_service.available_models().await?;
        
        if models.is_empty() {
            spinner.warning("No models available. Using default model.");
            Model::default_claude()
        } else {
            spinner.success(&format!("Found {} models", models.len()));
            
            // Let user select a model
            let options: Vec<String> = models
                .iter()
                .map(|m| format!("{} ({})", m.name, m.id))
                .collect();
            
            let selection = dialoguer::Select::new()
                .with_prompt("Select a model")
                .items(&options)
                .default(0)
                .interact()?;
            
            models[selection].clone()
        }
    };
    
    // Create the conversation
    let spinner = show_spinner();
    spinner.set_message(&format!("Creating conversation with {}...", model.name));
    
    let conversation = chat_service.create_conversation(&title, Some(model)).await?;
    
    spinner.success("Conversation created");
    print_success(&format!(
        "Created conversation '{}' with ID: {}",
        conversation.title, conversation.id
    ));
    
    Ok(())
}
