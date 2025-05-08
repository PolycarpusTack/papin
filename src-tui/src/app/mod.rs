use std::sync::Arc;
use crossterm::event::{KeyEvent, MouseEvent, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use tui_textarea::TextArea;
use tokio::sync::mpsc;

use crate::error::AppError;
use mcp_common::{
    models::{Conversation, Message, Model},
    service::ChatService,
};

// Result type used in the application
pub type AppResult<T> = std::result::Result<T, AppError>;

// Application mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,      // Navigation, list selection
    Chatting,    // Active chat input
    Command,     // Command input
    Help,        // Help screen
    Settings,    // Settings screen
}

// Application state
pub struct App {
    // Services
    pub chat_service: Arc<ChatService>,
    
    // Application state
    pub should_quit: bool,
    pub mode: AppMode,
    pub size: Rect,
    
    // Conversations
    pub conversations: Vec<Conversation>,
    pub selected_conversation_idx: Option<usize>,
    pub current_conversation: Option<Conversation>,
    pub message_offset: usize,
    
    // Streaming state
    pub is_streaming: bool,
    pub stream_receiver: Option<mpsc::Receiver<Result<Message, String>>>,
    pub current_response: String,
    
    // Input fields
    pub input: TextArea<'static>,
    pub command_input: TextArea<'static>,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    
    // Help
    pub show_help: bool,
    
    // Settings
    pub settings_open: bool,
    pub settings_idx: usize,
}

impl App {
    // Create a new application instance
    pub fn new(chat_service: Arc<ChatService>) -> Self {
        let mut app = Self {
            chat_service,
            should_quit: false,
            mode: AppMode::Normal,
            size: Rect::default(),
            conversations: Vec::new(),
            selected_conversation_idx: None,
            current_conversation: None,
            message_offset: 0,
            is_streaming: false,
            stream_receiver: None,
            current_response: String::new(),
            input: TextArea::default(),
            command_input: TextArea::default(),
            status_message: None,
            show_help: false,
            settings_open: false,
            settings_idx: 0,
        };
        
        // Configure TextArea
        app.input.set_placeholder_text("Type a message...");
        app.input.set_cursor_line_style(ratatui::style::Style::default());
        app.input.set_block(ratatui::widgets::Block::default());
        
        // Configure command input
        app.command_input.set_placeholder_text("Type a command...");
        app.command_input.set_cursor_line_style(ratatui::style::Style::default());
        app.command_input.set_block(ratatui::widgets::Block::default());
        
        app
    }
    
    // Initialize the application
    pub async fn initialize(&mut self) -> AppResult<()> {
        // Load conversations
        self.load_conversations().await?;
        
        // Set status message
        self.set_status("Welcome to Claude MCP TUI", false);
        
        Ok(())
    }
    
    // Handle application tick (time-based updates)
    pub fn tick(&mut self) {
        // Process streaming responses
        if self.is_streaming {
            if let Some(receiver) = &mut self.stream_receiver {
                // Try to receive new message chunks
                if let Ok(Some(result)) = receiver.try_recv() {
                    match result {
                        Ok(message) => {
                            // Update the current response
                            self.current_response = message.text();
                            
                            // Update the conversation with the response
                            if let Some(conversation) = &mut self.current_conversation {
                                // Check if there's already an assistant message at the end
                                let last_message_is_assistant = conversation.messages.last()
                                    .map(|m| m.role == "assistant")
                                    .unwrap_or(false);
                                
                                if last_message_is_assistant {
                                    // Update the last message
                                    if let Some(last) = conversation.messages.last_mut() {
                                        last.content = message.content.clone();
                                    }
                                } else {
                                    // Add a new message
                                    conversation.messages.push(message);
                                }
                            }
                        }
                        Err(e) => {
                            // Show error
                            self.set_status(&format!("Error: {}", e), true);
                            self.is_streaming = false;
                        }
                    }
                }
            } else {
                // No receiver, streaming should be false
                self.is_streaming = false;
            }
        }
        
        // Clear status message after a period of time
        if let Some((_, _)) = &self.status_message {
            // In a real implementation, we'd check against a timestamp
            // and clear after a certain duration
        }
    }
    
    // Handle keyboard events
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<bool> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key).await?,
            AppMode::Chatting => self.handle_chat_mode_key(key).await?,
            AppMode::Command => self.handle_command_mode_key(key).await?,
            AppMode::Help => self.handle_help_mode_key(key)?,
            AppMode::Settings => self.handle_settings_mode_key(key).await?,
        }
        
        Ok(self.should_quit)
    }
    
    // Handle mouse events
    pub fn handle_mouse_event(&mut self, _event: MouseEvent) {
        // Mouse handling code would go here
    }
    
    // Handle window resize
    pub fn resize(&mut self, width: u16, height: u16) {
        self.size = Rect::new(0, 0, width, height);
    }
    
    // Set status message
    pub fn set_status(&mut self, message: &str, is_error: bool) {
        self.status_message = Some((message.to_string(), is_error));
    }
    
    // Load conversations from the service
    async fn load_conversations(&mut self) -> AppResult<()> {
        match self.chat_service.list_conversations().await {
            Ok(conversations) => {
                self.conversations = conversations;
                
                // Select the first conversation if available
                if !self.conversations.is_empty() && self.selected_conversation_idx.is_none() {
                    self.selected_conversation_idx = Some(0);
                }
                
                Ok(())
            }
            Err(e) => {
                self.set_status(&format!("Failed to load conversations: {}", e), true);
                Err(AppError::Service(format!("Failed to load conversations: {}", e)))
            }
        }
    }
    
    // Load a specific conversation
    async fn load_conversation(&mut self, conversation_id: &str) -> AppResult<()> {
        match self.chat_service.get_conversation(conversation_id).await {
            Ok(conversation) => {
                self.current_conversation = Some(conversation);
                self.message_offset = 0;
                Ok(())
            }
            Err(e) => {
                self.set_status(&format!("Failed to load conversation: {}", e), true);
                Err(AppError::Service(format!("Failed to load conversation: {}", e)))
            }
        }
    }
    
    // Send a message in the current conversation
    async fn send_message(&mut self, content: &str) -> AppResult<()> {
        // Get the current conversation ID
        let conversation_id = if let Some(conversation) = &self.current_conversation {
            conversation.id.clone()
        } else if let Some(idx) = self.selected_conversation_idx {
            if let Some(conversation) = self.conversations.get(idx) {
                conversation.id.clone()
            } else {
                return Err(AppError::App("No conversation selected".to_string()));
            }
        } else {
            return Err(AppError::App("No conversation selected".to_string()));
        };
        
        // Add the user message to the conversation
        if let Some(conversation) = &mut self.current_conversation {
            let message = Message {
                id: format!("temp_{}", uuid::Uuid::new_v4()),
                role: "user".to_string(),
                content: vec![mcp_common::models::MessageContent {
                    content_type: "text".to_string(),
                    text: Some(content.to_string()),
                    source: None,
                }],
                model: None,
                stop_reason: None,
                stop_sequence: None,
                usage: None,
            };
            conversation.messages.push(message);
        }
        
        // Start streaming response
        match self.chat_service.send_message_streaming(&conversation_id, content).await {
            Ok(receiver) => {
                self.stream_receiver = Some(receiver);
                self.is_streaming = true;
                self.current_response = String::new();
                Ok(())
            }
            Err(e) => {
                self.set_status(&format!("Failed to send message: {}", e), true);
                Err(AppError::Service(format!("Failed to send message: {}", e)))
            }
        }
    }
    
    // Create a new conversation
    async fn create_conversation(&mut self, title: &str) -> AppResult<()> {
        match self.chat_service.create_conversation(title, None).await {
            Ok(conversation) => {
                // Add to list and select it
                self.conversations.insert(0, conversation.clone());
                self.selected_conversation_idx = Some(0);
                self.current_conversation = Some(conversation);
                self.set_status(&format!("Created conversation: {}", title), false);
                Ok(())
            }
            Err(e) => {
                self.set_status(&format!("Failed to create conversation: {}", e), true);
                Err(AppError::Service(format!("Failed to create conversation: {}", e)))
            }
        }
    }
    
    // Delete the current conversation
    async fn delete_conversation(&mut self) -> AppResult<()> {
        if let Some(idx) = self.selected_conversation_idx {
            if let Some(conversation) = self.conversations.get(idx) {
                let id = conversation.id.clone();
                let title = conversation.title.clone();
                
                match self.chat_service.delete_conversation(&id).await {
                    Ok(_) => {
                        // Remove from list
                        self.conversations.remove(idx);
                        
                        // Update selection
                        if self.conversations.is_empty() {
                            self.selected_conversation_idx = None;
                            self.current_conversation = None;
                        } else {
                            let new_idx = if idx >= self.conversations.len() {
                                self.conversations.len() - 1
                            } else {
                                idx
                            };
                            self.selected_conversation_idx = Some(new_idx);
                            
                            // Load the newly selected conversation
                            if let Some(conversation) = self.conversations.get(new_idx) {
                                self.load_conversation(&conversation.id).await?;
                            }
                        }
                        
                        self.set_status(&format!("Deleted conversation: {}", title), false);
                        Ok(())
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to delete conversation: {}", e), true);
                        Err(AppError::Service(format!("Failed to delete conversation: {}", e)))
                    }
                }
            } else {
                Err(AppError::App("Invalid conversation index".to_string()))
            }
        } else {
            Err(AppError::App("No conversation selected".to_string()))
        }
    }
    
    // Handle keys in normal mode (conversation navigation)
    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            // Quit application
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            
            // Help screen
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.show_help = true;
                self.mode = AppMode::Help;
            }
            
            // Settings screen
            KeyCode::Char('s') => {
                self.settings_open = true;
                self.mode = AppMode::Settings;
            }
            
            // Navigation - up/down
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(idx) = self.selected_conversation_idx {
                    if idx > 0 {
                        self.selected_conversation_idx = Some(idx - 1);
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(idx) = self.selected_conversation_idx {
                    if idx < self.conversations.len() - 1 {
                        self.selected_conversation_idx = Some(idx + 1);
                    }
                }
            }
            
            // Select conversation
            KeyCode::Enter => {
                if let Some(idx) = self.selected_conversation_idx {
                    if let Some(conversation) = self.conversations.get(idx) {
                        self.load_conversation(&conversation.id).await?;
                        self.mode = AppMode::Chatting;
                    }
                }
            }
            
            // Create new conversation
            KeyCode::Char('n') => {
                // Default name with timestamp
                let title = format!("Conversation {}", chrono::Local::now().format("%Y-%m-%d %H:%M"));
                self.create_conversation(&title).await?;
                self.mode = AppMode::Chatting;
            }
            
            // Delete conversation
            KeyCode::Char('d') => {
                if let Some(idx) = self.selected_conversation_idx {
                    if let Some(conversation) = self.conversations.get(idx) {
                        // In a real implementation, we'd ask for confirmation
                        self.delete_conversation().await?;
                    }
                }
            }
            
            // Command mode
            KeyCode::Char(':') => {
                self.command_input = TextArea::default();
                self.command_input.set_placeholder_text("Type a command...");
                self.mode = AppMode::Command;
            }
            
            // Scroll through conversation history
            KeyCode::PageUp => {
                if self.message_offset > 0 {
                    self.message_offset -= 1;
                }
            }
            KeyCode::PageDown => {
                if let Some(conversation) = &self.current_conversation {
                    if self.message_offset < conversation.messages.len() {
                        self.message_offset += 1;
                    }
                }
            }
            
            // Reload conversations
            KeyCode::Char('r') => {
                self.load_conversations().await?;
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    // Handle keys in chat mode (message input)
    async fn handle_chat_mode_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            // Send message on Ctrl+Enter
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let content = self.input.lines().join("\n");
                if !content.is_empty() {
                    self.send_message(&content).await?;
                    self.input = TextArea::default();
                    self.input.set_placeholder_text("Type a message...");
                }
            }
            
            // Exit chat mode on Escape
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            
            // Pass other keys to the text area
            _ => {
                self.input.input(key);
            }
        }
        
        Ok(())
    }
    
    // Handle keys in command mode
    async fn handle_command_mode_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            // Execute command on Enter
            KeyCode::Enter => {
                let command = self.command_input.lines().join(" ").trim().to_string();
                self.mode = AppMode::Normal;
                
                if !command.is_empty() {
                    self.execute_command(&command).await?;
                }
            }
            
            // Exit command mode on Escape
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            
            // Pass other keys to the text area
            _ => {
                self.command_input.input(key);
            }
        }
        
        Ok(())
    }
    
    // Handle keys in help mode
    fn handle_help_mode_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            // Exit help mode on Escape or q
            KeyCode::Esc | KeyCode::Char('q') => {
                self.show_help = false;
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    // Handle keys in settings mode
    async fn handle_settings_mode_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            // Exit settings mode on Escape
            KeyCode::Esc => {
                self.settings_open = false;
                self.mode = AppMode::Normal;
            }
            
            // Navigate settings
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_idx > 0 {
                    self.settings_idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // In a real implementation, we'd check against max settings
                self.settings_idx += 1;
            }
            
            // Toggle or modify settings
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Toggle or modify the selected setting
                // In a real implementation, we'd handle different setting types
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    // Execute a command from the command prompt
    async fn execute_command(&mut self, command: &str) -> AppResult<()> {
        // Parse command
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        
        match parts[0] {
            "quit" | "q" => {
                self.should_quit = true;
            }
            "new" | "n" => {
                let title = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    format!("Conversation {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))
                };
                self.create_conversation(&title).await?;
                self.mode = AppMode::Chatting;
            }
            "delete" | "d" => {
                self.delete_conversation().await?;
            }
            "reload" | "r" => {
                self.load_conversations().await?;
            }
            "help" | "h" => {
                self.show_help = true;
                self.mode = AppMode::Help;
            }
            "settings" | "s" => {
                self.settings_open = true;
                self.mode = AppMode::Settings;
            }
            _ => {
                self.set_status(&format!("Unknown command: {}", parts[0]), true);
            }
        }
        
        Ok(())
    }
}
