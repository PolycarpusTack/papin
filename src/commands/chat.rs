use crate::models::messages::{Message, MessageError};
use crate::models::{Conversation, Model};
use crate::services::chat::get_chat_service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

/// Get available models
#[tauri::command]
pub fn get_available_models() -> Vec<Model> {
    get_chat_service().available_models()
}

/// Create a new conversation
#[tauri::command]
pub fn create_conversation(title: String, model_id: String) -> Result<Conversation, String> {
    // Find model by ID
    let model = get_chat_service()
        .available_models()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Model with ID {} not found", model_id))?;
    
    // Create conversation
    let conversation = get_chat_service().create_conversation(&title, model);
    Ok(conversation)
}

/// Get a conversation by ID
#[tauri::command]
pub fn get_conversation(id: String) -> Result<Conversation, String> {
    get_chat_service()
        .get_conversation(&id)
        .ok_or_else(|| format!("Conversation with ID {} not found", id))
}

/// Get all conversations
#[tauri::command]
pub fn get_conversations() -> Vec<Conversation> {
    let service = get_chat_service();
    
    // In a real implementation, we would fetch all conversations
    // For now, we'll return an empty list
    Vec::new()
}

/// Delete a conversation
#[tauri::command]
pub fn delete_conversation(id: String) -> Result<(), String> {
    get_chat_service().delete_conversation(&id)
}

/// Get conversation message history
#[tauri::command]
pub fn get_messages(conversation_id: String) -> Result<Vec<serde_json::Value>, String> {
    let messages = get_chat_service().get_messages(&conversation_id);
    
    // Convert to serde_json::Value for serialization
    let messages_json = messages
        .into_iter()
        .map(|msg| {
            let mut map = serde_json::Map::new();
            
            // Convert message
            map.insert(
                "message".to_string(),
                serde_json::to_value(&msg.message).unwrap(),
            );
            
            // Convert parent IDs
            map.insert(
                "parent_ids".to_string(),
                serde_json::to_value(&msg.parent_ids).unwrap(),
            );
            
            // Convert completed_at
            map.insert(
                "completed_at".to_string(),
                if let Some(time) = msg.completed_at {
                    serde_json::to_value(time).unwrap()
                } else {
                    serde_json::Value::Null
                },
            );
            
            // Convert partial_content
            map.insert(
                "partial_content".to_string(),
                if let Some(content) = msg.partial_content {
                    serde_json::to_value(content).unwrap()
                } else {
                    serde_json::Value::Null
                },
            );
            
            // Convert status
            map.insert(
                "status".to_string(),
                serde_json::to_value(match msg.status {
                    crate::models::messages::MessageStatus::Sending => "sending",
                    crate::models::messages::MessageStatus::Streaming => "streaming",
                    crate::models::messages::MessageStatus::Complete => "complete",
                    crate::models::messages::MessageStatus::Failed => "failed",
                    crate::models::messages::MessageStatus::Cancelled => "cancelled",
                })
                .unwrap(),
            );
            
            serde_json::Value::Object(map)
        })
        .collect();
    
    Ok(messages_json)
}

/// Send a message in a conversation
#[tauri::command]
pub async fn send_message(
    conversation_id: String,
    content: String,
) -> Result<serde_json::Value, String> {
    // Create a message
    let message = Message::new_user_text(content);
    
    // Send message
    match get_chat_service()
        .send_message(&conversation_id, message)
        .await
    {
        Ok(response) => {
            // Convert to json
            let mut map = serde_json::Map::new();
            
            // Convert message
            map.insert(
                "message".to_string(),
                serde_json::to_value(&response.message).unwrap(),
            );
            
            // Convert parent IDs
            map.insert(
                "parent_ids".to_string(),
                serde_json::to_value(&response.parent_ids).unwrap(),
            );
            
            // Convert completed_at
            map.insert(
                "completed_at".to_string(),
                if let Some(time) = response.completed_at {
                    serde_json::to_value(time).unwrap()
                } else {
                    serde_json::Value::Null
                },
            );
            
            // Convert partial_content
            map.insert(
                "partial_content".to_string(),
                if let Some(content) = response.partial_content {
                    serde_json::to_value(content).unwrap()
                } else {
                    serde_json::Value::Null
                },
            );
            
            // Convert status
            map.insert(
                "status".to_string(),
                serde_json::to_value(match response.status {
                    crate::models::messages::MessageStatus::Sending => "sending",
                    crate::models::messages::MessageStatus::Streaming => "streaming",
                    crate::models::messages::MessageStatus::Complete => "complete",
                    crate::models::messages::MessageStatus::Failed => "failed",
                    crate::models::messages::MessageStatus::Cancelled => "cancelled",
                })
                .unwrap(),
            );
            
            Ok(serde_json::Value::Object(map))
        }
        Err(e) => Err(format!("Failed to send message: {}", e)),
    }
}
