pub mod chat;
pub mod delete;
pub mod export;
pub mod interactive;
pub mod list;
pub mod new;
pub mod setup;
pub mod show;
pub mod system;

use clap::{Parser, Subcommand};

/// MCP Client Command Line Interface
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets the level of verbosity
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Suppresses most output
    #[arg(short, long)]
    pub quiet: bool,
    
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Send a message in a conversation
    Chat {
        /// Conversation ID
        #[arg(short, long)]
        conversation_id: Option<String>,
        
        /// Message content
        #[arg(short, long)]
        message: Option<String>,
        
        /// Disable streaming mode
        #[arg(long)]
        no_stream: bool,
    },
    
    /// List conversations
    List,
    
    /// Create a new conversation
    New {
        /// Conversation title
        #[arg(short, long)]
        title: Option<String>,
        
        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },
    
    /// Delete a conversation
    Delete {
        /// Conversation ID
        conversation_id: String,
    },
    
    /// Show conversation details
    Show {
        /// Conversation ID
        conversation_id: String,
    },
    
    /// Configure API settings
    Setup,
    
    /// Export a conversation
    Export {
        /// Conversation ID
        conversation_id: String,
        
        /// Export format (json, markdown, txt)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Set system message for a conversation
    System {
        /// Conversation ID
        conversation_id: String,
        
        /// System message content
        #[arg(short, long)]
        message: Option<String>,
    },
    
    /// Start interactive mode
    Interactive {
        /// Conversation ID (optional)
        #[arg(short, long)]
        conversation_id: Option<String>,
    },
}
