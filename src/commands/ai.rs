use crate::ai::router::NetworkStatus;
use crate::models::messages::{Message, MessageError};
use crate::models::Model;
use crate::services::ai::get_ai_service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

/// Get available models
#[tauri::command]
pub async fn get_available_models() -> Result<Vec<Model>, String> {
    Ok(get_ai_service().available_models().await)
}

/// Set network status
#[tauri::command]
pub fn set_network_status(status: String) -> Result<(), String> {
    let network_status = match status.as_str() {
        "connected" => NetworkStatus::Connected,
        "disconnected" => NetworkStatus::Disconnected,
        "unstable" => NetworkStatus::Unstable,
        _ => NetworkStatus::Unknown,
    };
    
    get_ai_service().set_network_status(network_status);
    Ok(())
}

/// Send a message to a model
#[tauri::command]
pub async fn send_message(
    conversation_id: String,
    model_id: String,
    content: String,
) -> Result<serde_json::Value, String> {
    // Create a message
    let message = Message::new_user_text(content);
    
    // Send message
    match get_ai_service()
        .send_message(&conversation_id, &model_id, message)
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

/// Stream a message to a model
#[tauri::command]
pub async fn stream_message(
    window: tauri::Window,
    conversation_id: String,
    model_id: String,
    content: String,
) -> Result<String, String> {
    // Create a message
    let message = Message::new_user_text(content);
    
    // Generate a unique stream ID
    let stream_id = uuid::Uuid::new_v4().to_string();
    
    // Start streaming
    match get_ai_service()
        .stream_message(&conversation_id, &model_id, message)
        .await
    {
        Ok(mut stream) => {
            // Process stream in a separate task
            let window_clone = window.clone();
            let stream_id_clone = stream_id.clone();
            
            tauri::async_runtime::spawn(async move {
                while let Some(response) = stream.recv().await {
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
                    
                    // Add stream ID
                    map.insert(
                        "stream_id".to_string(),
                        serde_json::to_value(&stream_id_clone).unwrap(),
                    );
                    
                    // Emit event to window
                    let _ = window_clone.emit("stream-update", serde_json::Value::Object(map));
                }
                
                // Emit stream end event
                let _ = window_clone.emit(
                    "stream-end",
                    serde_json::json!({
                        "stream_id": stream_id_clone
                    }),
                );
            });
            
            Ok(stream_id)
        }
        Err(e) => Err(format!("Failed to start streaming: {}", e)),
    }
}

/// Cancel a streaming message
#[tauri::command]
pub async fn cancel_streaming(
    conversation_id: String,
    message_id: String,
) -> Result<(), String> {
    match get_ai_service()
        .cancel_streaming(&conversation_id, &message_id)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to cancel streaming: {}", e)),
    }
}

/// Get conversation messages
#[tauri::command]
pub fn get_messages(conversation_id: String) -> Result<Vec<serde_json::Value>, String> {
    let messages = get_ai_service().get_messages(&conversation_id);
    
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

/// Create a conversation
#[tauri::command]
pub fn create_conversation(title: String, model_id: String) -> Result<serde_json::Value, String> {
    // Find model by ID
    let models = get_ai_service().available_models().await;
    let model = models
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Model with ID {} not found", model_id))?;
    
    // Create conversation
    let conversation = get_ai_service().create_conversation(&title, model);
    
    Ok(serde_json::to_value(conversation).unwrap())
}

/// Delete a conversation
#[tauri::command]
pub fn delete_conversation(id: String) -> Result<(), String> {
    get_ai_service().delete_conversation(&id)
}
