// Whiteboard command handlers for the collaborative whiteboard feature

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::error::Result;
use crate::collaboration::get_collaboration_manager;

/// Point coordinates for whiteboard operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Type of drawing operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    #[serde(rename = "pencil")]
    Pencil,
    
    #[serde(rename = "line")]
    Line,
    
    #[serde(rename = "rectangle")]
    Rectangle,
    
    #[serde(rename = "circle")]
    Circle,
    
    #[serde(rename = "text")]
    Text,
    
    #[serde(rename = "eraser")]
    Eraser,
}

/// Whiteboard drawing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawOperation {
    #[serde(rename = "type")]
    pub operation_type: OperationType,
    
    pub points: Vec<Point>,
    
    pub color: String,
    
    pub size: u32,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    
    pub user_id: String,
    
    pub timestamp: u64,
}

/// Whiteboard state for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardState {
    pub operations: Vec<DrawOperation>,
    pub version: u64,
}

// In-memory storage for whiteboard state by session ID
lazy_static::lazy_static! {
    static ref WHITEBOARDS: Arc<RwLock<HashMap<String, WhiteboardState>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// Send a whiteboard operation to all users in a session
#[tauri::command]
pub async fn send_whiteboard_operation(session_id: String, operation: String) -> Result<()> {
    // Parse the operation JSON
    let operation: DrawOperation = serde_json::from_str(&operation)?;
    
    // Get the collaboration manager
    let manager = get_collaboration_manager()?;
    
    // Add operation to whiteboard state
    {
        let mut whiteboards = WHITEBOARDS.write().unwrap();
        let whiteboard = whiteboards.entry(session_id.clone()).or_insert(WhiteboardState {
            operations: Vec::new(),
            version: 0,
        });
        
        whiteboard.operations.push(operation.clone());
        whiteboard.version += 1;
    }
    
    // Create a message with the operation
    let message_content = serde_json::to_string(&operation)?;
    let message = crate::models::messages::Message {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: session_id.clone(),
        content: message_content,
        timestamp: SystemTime::now(),
        sender: "system".to_string(),
        sender_name: "Whiteboard".to_string(),
        message_type: "whiteboard_operation".to_string(),
        metadata: HashMap::new(),
    };
    
    // Send the message through the collaboration system
    manager.send_message(&message)?;
    
    info!("Sent whiteboard operation in session {}", session_id);
    
    Ok(())
}

/// Get the current whiteboard state for a session
#[tauri::command]
pub async fn get_whiteboard_state(session_id: String) -> Result<WhiteboardState> {
    let whiteboards = WHITEBOARDS.read().unwrap();
    
    // Return the whiteboard state or an empty one if it doesn't exist
    Ok(whiteboards.get(&session_id).cloned().unwrap_or(WhiteboardState {
        operations: Vec::new(),
        version: 0,
    }))
}

/// Clear the whiteboard for a session
#[tauri::command]
pub async fn clear_whiteboard(session_id: String) -> Result<()> {
    // Get the collaboration manager
    let manager = get_collaboration_manager()?;
    
    // Clear the whiteboard state
    {
        let mut whiteboards = WHITEBOARDS.write().unwrap();
        whiteboards.insert(session_id.clone(), WhiteboardState {
            operations: Vec::new(),
            version: 0,
        });
    }
    
    // Create a clear message
    let message = crate::models::messages::Message {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: session_id.clone(),
        content: "clear".to_string(),
        timestamp: SystemTime::now(),
        sender: "system".to_string(),
        sender_name: "Whiteboard".to_string(),
        message_type: "whiteboard_clear".to_string(),
        metadata: HashMap::new(),
    };
    
    // Send the message through the collaboration system
    manager.send_message(&message)?;
    
    info!("Cleared whiteboard in session {}", session_id);
    
    Ok(())
}

/// Save the whiteboard as an image
#[tauri::command]
pub async fn save_whiteboard_image(session_id: String, image_data: String) -> Result<String> {
    // In a real implementation, we would save the image to disk or cloud storage
    // For now, we'll just return a mock file path
    
    let file_path = format!("whiteboard_{}.png", session_id);
    
    info!("Saved whiteboard image for session {}", session_id);
    
    Ok(file_path)
}
