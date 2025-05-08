use std::sync::Arc;

use crate::display::{print_error, print_info, print_success, print_table, TableColumn};
use crate::error::CliResult;
use mcp_common::service::ChatService;

/// List available models
pub async fn list(chat_service: Arc<ChatService>) -> CliResult<()> {
    print_info("Fetching available models...");
    
    match chat_service.list_models().await {
        Ok(models) => {
            if models.is_empty() {
                print_info("No models available");
                return Ok(());
            }
            
            // Define table columns
            let columns = vec![
                TableColumn {
                    header: "Name".to_string(),
                    width: 30,
                },
                TableColumn {
                    header: "Provider".to_string(),
                    width: 15,
                },
                TableColumn {
                    header: "Max Tokens".to_string(),
                    width: 15,
                },
                TableColumn {
                    header: "Available".to_string(),
                    width: 10,
                },
            ];
            
            // Convert models to rows
            let rows: Vec<Vec<String>> = models
                .iter()
                .map(|model| {
                    vec![
                        model.name.clone(),
                        model.provider.clone().unwrap_or_else(|| "Unknown".to_string()),
                        model.max_tokens.map(|t| t.to_string()).unwrap_or_else(|| "-".to_string()),
                        if model.available { "Yes".to_string() } else { "No".to_string() },
                    ]
                })
                .collect();
            
            // Print models table
            print_table(&columns, &rows);
            
            print_success(&format!("Found {} models", models.len()));
        }
        Err(e) => {
            print_error(&format!("Failed to fetch models: {}", e));
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// Set default model for new conversations
pub async fn set_default(chat_service: Arc<ChatService>, model_name: &str) -> CliResult<()> {
    print_info(&format!("Setting default model to '{}'...", model_name));
    
    match chat_service.set_default_model(model_name).await {
        Ok(_) => {
            print_success(&format!("Default model set to '{}'", model_name));
            Ok(())
        }
        Err(e) => {
            print_error(&format!("Failed to set default model: {}", e));
            Err(e.into())
        }
    }
}

/// Set model for an existing conversation
pub async fn set_for_conversation(
    chat_service: Arc<ChatService>,
    conversation_id: &str,
    model_name: &str,
) -> CliResult<()> {
    print_info(&format!(
        "Setting model for conversation '{}' to '{}'...",
        conversation_id, model_name
    ));
    
    match chat_service.set_conversation_model(conversation_id, model_name).await {
        Ok(_) => {
            print_success(&format!("Model set to '{}'", model_name));
            Ok(())
        }
        Err(e) => {
            print_error(&format!("Failed to set conversation model: {}", e));
            Err(e.into())
        }
    }
}
