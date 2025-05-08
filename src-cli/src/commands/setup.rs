use dialoguer::{Confirm, Input, Select};
use std::sync::Arc;

use crate::display::{print_error, print_info, print_success, show_spinner};
use crate::error::CliResult;
use mcp_common::config::{get_settings, get_storage_manager};

/// Run the setup command
pub async fn run() -> CliResult<()> {
    print_info("MCP Client Setup");
    println!();
    
    // Get settings
    let settings = get_settings();
    let mut settings_guard = settings.lock().unwrap();
    
    // API key setup
    let current_api_key = settings_guard.get_api_key().unwrap_or(Ok(None)).unwrap_or(None);
    
    if current_api_key.is_some() {
        print_info("An API key is already configured");
        
        if !Confirm::new()
            .with_prompt("Do you want to replace it?")
            .default(false)
            .interact()?
        {
            print_info("Keeping existing API key");
        } else {
            // Replace API key
            let new_api_key: String = Input::new()
                .with_prompt("Enter your Anthropic API key")
                .interact_text()?;
            
            if new_api_key.is_empty() {
                print_error("API key cannot be empty");
            } else {
                settings_guard.set_api_key(&new_api_key)?;
                print_success("API key updated");
            }
        }
    } else {
        // No API key configured
        print_info("No API key configured");
        
        let new_api_key: String = Input::new()
            .with_prompt("Enter your Anthropic API key")
            .interact_text()?;
        
        if new_api_key.is_empty() {
            print_error("API key cannot be empty");
        } else {
            settings_guard.set_api_key(&new_api_key)?;
            print_success("API key saved");
        }
    }
    
    // Model settings
    print_info("\nModel Settings");
    
    // Default model
    let model_options = [
        "claude-3-opus-20240229",
        "claude-3-sonnet-20240229",
        "claude-3-haiku-20240307",
    ];
    
    let default_model_index = model_options
        .iter()
        .position(|&m| m == settings_guard.api.model)
        .unwrap_or(1); // Default to sonnet
    
    let model_selection = Select::new()
        .with_prompt("Select default model")
        .items(&model_options)
        .default(default_model_index)
        .interact()?;
    
    settings_guard.api.model = model_options[model_selection].to_string();
    
    // Temperature
    let temperature: f32 = Input::new()
        .with_prompt("Default temperature (0.0-1.0)")
        .default(settings_guard.model.temperature)
        .interact_text()?;
    
    settings_guard.model.temperature = temperature.clamp(0.0, 1.0);
    
    // Max tokens
    let max_tokens: u32 = Input::new()
        .with_prompt("Default max tokens")
        .default(settings_guard.model.max_tokens)
        .interact_text()?;
    
    settings_guard.model.max_tokens = max_tokens;
    
    // System prompt
    print_info("\nSystem Prompt");
    
    if let Some(prompt) = &settings_guard.model.system_prompt {
        println!("Current system prompt: {}", prompt);
        
        if Confirm::new()
            .with_prompt("Do you want to change it?")
            .default(false)
            .interact()?
        {
            let new_prompt: String = Input::new()
                .with_prompt("Enter new system prompt (leave empty to clear)")
                .interact_text()?;
            
            if new_prompt.is_empty() {
                settings_guard.model.system_prompt = None;
                print_info("System prompt cleared");
            } else {
                settings_guard.model.system_prompt = Some(new_prompt);
                print_success("System prompt updated");
            }
        }
    } else {
        print_info("No system prompt configured");
        
        if Confirm::new()
            .with_prompt("Do you want to set a system prompt?")
            .default(false)
            .interact()?
        {
            let new_prompt: String = Input::new()
                .with_prompt("Enter system prompt")
                .interact_text()?;
            
            if !new_prompt.is_empty() {
                settings_guard.model.system_prompt = Some(new_prompt);
                print_success("System prompt set");
            }
        }
    }
    
    // Save settings
    settings_guard.save()?;
    print_success("\nSettings saved");
    
    // Return success
    Ok(())
}
