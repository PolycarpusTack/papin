use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::error::{McpError, McpResult};
use crate::models::Conversation;
use super::{data_path, get_data_dir};

/// Storage manager
pub struct StorageManager {
    /// Conversations directory
    conversations_dir: PathBuf,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new() -> Self {
        let mut conversations_dir = get_data_dir();
        conversations_dir.push("conversations");
        
        // Create if it doesn't exist
        if !conversations_dir.exists() {
            fs::create_dir_all(&conversations_dir).expect("Failed to create conversations directory");
        }
        
        Self {
            conversations_dir,
        }
    }
    
    /// Get path for a conversation file
    pub fn conversation_path(&self, conversation_id: &str) -> PathBuf {
        let mut path = self.conversations_dir.clone();
        path.push(format!("{}.json", conversation_id));
        path
    }
    
    /// Save a conversation
    pub fn save_conversation(&self, conversation: &Conversation) -> McpResult<()> {
        let path = self.conversation_path(&conversation.id);
        
        let content = serde_json::to_string_pretty(conversation)
            .map_err(|e| McpError::Serialization(e))?;
            
        fs::write(path, content)
            .map_err(|e| McpError::Io(e))?;
            
        Ok(())
    }
    
    /// Load a conversation
    pub fn load_conversation(&self, conversation_id: &str) -> McpResult<Conversation> {
        let path = self.conversation_path(conversation_id);
        
        if !path.exists() {
            return Err(McpError::Unknown(format!("Conversation {} not found", conversation_id)));
        }
        
        let content = fs::read_to_string(&path)
            .map_err(|e| McpError::Io(e))?;
            
        let conversation = serde_json::from_str(&content)
            .map_err(|e| McpError::Serialization(e))?;
            
        Ok(conversation)
    }
    
    /// Delete a conversation
    pub fn delete_conversation(&self, conversation_id: &str) -> McpResult<()> {
        let path = self.conversation_path(conversation_id);
        
        if path.exists() {
            fs::remove_file(path)
                .map_err(|e| McpError::Io(e))?;
        }
        
        Ok(())
    }
    
    /// List all conversations
    pub fn list_conversations(&self) -> McpResult<Vec<Conversation>> {
        let mut conversations = Vec::new();
        
        for entry in fs::read_dir(&self.conversations_dir)
            .map_err(|e| McpError::Io(e))?
        {
            let entry = entry.map_err(|e| McpError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                // Read the conversation file
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(conversation) = serde_json::from_str::<Conversation>(&content) {
                        conversations.push(conversation);
                    }
                }
            }
        }
        
        // Sort by last updated
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(conversations)
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}
