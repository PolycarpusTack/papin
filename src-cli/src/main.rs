mod commands;
mod display;
mod error;

use clap::Parser;
use log::LevelFilter;
use std::sync::Arc;

use commands::{Cli, Commands};
use error::CliResult;
use mcp_common::{get_mcp_service, init_mcp_service, service::ChatService};

#[tokio::main]
async fn main() -> CliResult<()> {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Configure logger level
    if cli.verbose {
        log::set_max_level(LevelFilter::Debug);
    } else if cli.quiet {
        log::set_max_level(LevelFilter::Error);
    } else {
        log::set_max_level(LevelFilter::Info);
    }
    
    // Initialize MCP service
    let mcp_service = init_mcp_service();
    let chat_service = Arc::new(ChatService::new(mcp_service));
    
    // Process command
    match cli.command {
        Commands::Chat {
            conversation_id,
            message,
            no_stream,
        } => {
            commands::chat::run(chat_service, conversation_id, message, !no_stream).await?;
        }
        Commands::List => {
            commands::list::run(chat_service).await?;
        }
        Commands::New { title, model } => {
            commands::new::run(chat_service, title, model).await?;
        }
        Commands::Delete { conversation_id } => {
            commands::delete::run(chat_service, conversation_id).await?;
        }
        Commands::Show { conversation_id } => {
            commands::show::run(chat_service, conversation_id).await?;
        }
        Commands::Setup => {
            commands::setup::run().await?;
        }
        Commands::Export { conversation_id, format, output } => {
            commands::export::run(chat_service, conversation_id, format, output).await?;
        }
        Commands::System { conversation_id, message } => {
            commands::system::run(chat_service, conversation_id, message).await?;
        }
        Commands::Interactive { conversation_id } => {
            commands::interactive::run(chat_service, conversation_id).await?;
        }
    }
    
    Ok(())
}
