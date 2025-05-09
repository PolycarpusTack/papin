use serde::{Serialize, Deserialize};
use tauri::Wry;
use std::collections::HashMap;

pub mod whiteboard;

use crate::collaboration::{
    CollaborationConfig, 
    ConnectionStatus, 
    UserRole,
    Session,
    User,
    init_collaboration,
    get_collaboration_manager
};
use crate::collaboration::presence::{CursorPosition, Selection};
use crate::error::Result;
use crate::models::messages::{Conversation, Message};

/// Register collaboration commands with Tauri
pub fn register_commands(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        // General commands
        init_collaboration_system,
        get_collaboration_config,
        update_collaboration_config,
        get_connection_status,
        
        // Session commands
        create_session,
        join_session,
        leave_session,
        invite_user,
        remove_user,
        change_user_role,
        get_session_users,
        
        // Presence commands
        update_cursor_position,
        update_selection,
        get_cursors,
        get_selections,
        
        // Sync commands
        sync_conversation,
        send_message,
        
        // AV commands
        start_audio_call,
        start_video_call,
        end_call,
        toggle_mute,
        toggle_video,
        get_active_call,
        get_media_devices,
        
        // User commands
        update_username,
        update_avatar,
        
        // Statistics
        get_collaboration_statistics,
        
        // Whiteboard commands
        whiteboard::send_whiteboard_operation,
        whiteboard::get_whiteboard_state,
        whiteboard::clear_whiteboard,
        whiteboard::save_whiteboard_image,
    ])
}

/// Initialize the collaboration system
#[tauri::command]
pub async fn init_collaboration_system(config: Option<CollaborationConfig>) -> Result<()> {
    init_collaboration(config)
}

/// Get the current collaboration configuration
#[tauri::command]
pub async fn get_collaboration_config() -> Result<CollaborationConfig> {
    let manager = get_collaboration_manager()?;
    Ok(manager.get_config())
}

/// Update the collaboration configuration
#[tauri::command]
pub async fn update_collaboration_config(config: CollaborationConfig) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.update_config(config)
}

/// Get the current connection status
#[tauri::command]
pub async fn get_connection_status() -> Result<ConnectionStatus> {
    let manager = get_collaboration_manager()?;
    Ok(manager.get_connection_status())
}

/// Create a new collaboration session
#[tauri::command]
pub async fn create_session(name: String, conversation_id: String) -> Result<Session> {
    let manager = get_collaboration_manager()?;
    manager.create_session(&name, &conversation_id)
}

/// Join an existing collaboration session
#[tauri::command]
pub async fn join_session(session_id: String) -> Result<Session> {
    let manager = get_collaboration_manager()?;
    manager.join_session(&session_id)
}

/// Leave the current collaboration session
#[tauri::command]
pub async fn leave_session() -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.leave_session()
}

/// Invite a user to the current session
#[tauri::command]
pub async fn invite_user(email: String, role: UserRole) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.invite_user(&email, role)
}

/// Remove a user from the current session
#[tauri::command]
pub async fn remove_user(user_id: String) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.remove_user(&user_id)
}

/// Change a user's role in the current session
#[tauri::command]
pub async fn change_user_role(user_id: String, role: UserRole) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.change_user_role(&user_id, role)
}

/// Get all users in the current session
#[tauri::command]
pub async fn get_session_users() -> Result<Vec<User>> {
    let manager = get_collaboration_manager()?;
    manager.get_session_users()
}

/// Update cursor position
#[tauri::command]
pub async fn update_cursor_position(x: f32, y: f32, element_id: Option<String>) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.update_cursor_position(x, y, element_id.as_deref())
}

/// Update selection
#[tauri::command]
pub async fn update_selection(
    start_id: String, 
    end_id: String, 
    start_offset: usize, 
    end_offset: usize
) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.update_selection(&start_id, &end_id, start_offset, end_offset)
}

/// Get all cursors in the current session
#[tauri::command]
pub async fn get_cursors() -> Result<HashMap<String, CursorPosition>> {
    let manager = get_collaboration_manager()?;
    manager.get_cursors()
}

/// Get all selections in the current session
#[tauri::command]
pub async fn get_selections() -> Result<HashMap<String, Selection>> {
    let manager = get_collaboration_manager()?;
    manager.get_selections()
}

/// Synchronize a conversation
#[tauri::command]
pub async fn sync_conversation(conversation: Conversation) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.sync_conversation(&conversation)
}

/// Send a message in the collaborative session
#[tauri::command]
pub async fn send_message(message: Message) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.send_message(&message)
}

/// Start an audio call in the current session
#[tauri::command]
pub async fn start_audio_call() -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.start_audio_call()
}

/// Start a video call in the current session
#[tauri::command]
pub async fn start_video_call() -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.start_video_call()
}

/// End the current call
#[tauri::command]
pub async fn end_call() -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.end_call()
}

/// Toggle mute status
#[tauri::command]
pub async fn toggle_mute() -> Result<bool> {
    let manager = get_collaboration_manager()?;
    
    // Get current session ID
    let session_id = match manager.get_statistics()?.current_session_id {
        Some(id) => id,
        None => return Err("No active session".into()),
    };
    
    // Get RTC manager and toggle mute
    let rtc_manager = manager.rtc_manager.write().unwrap();
    rtc_manager.toggle_mute(&session_id)
}

/// Toggle video status
#[tauri::command]
pub async fn toggle_video() -> Result<bool> {
    let manager = get_collaboration_manager()?;
    
    // Get current session ID
    let session_id = match manager.get_statistics()?.current_session_id {
        Some(id) => id,
        None => return Err("No active session".into()),
    };
    
    // Get RTC manager and toggle video
    let rtc_manager = manager.rtc_manager.write().unwrap();
    rtc_manager.toggle_video(&session_id)
}

/// Get the active call in the current session
#[tauri::command]
pub async fn get_active_call() -> Result<Option<crate::collaboration::rtc::Call>> {
    let manager = get_collaboration_manager()?;
    
    // Get current session ID
    let session_id = match manager.get_statistics()?.current_session_id {
        Some(id) => id,
        None => return Ok(None),
    };
    
    // Get RTC manager and get active call
    let rtc_manager = manager.rtc_manager.read().unwrap();
    rtc_manager.get_active_call(&session_id)
}

/// Get available media devices
#[tauri::command]
pub async fn get_media_devices() -> Result<Vec<crate::collaboration::rtc::MediaDevice>> {
    let manager = get_collaboration_manager()?;
    
    // Get RTC manager and get devices
    let rtc_manager = manager.rtc_manager.read().unwrap();
    rtc_manager.get_media_devices()
}

/// Update username
#[tauri::command]
pub async fn update_username(name: String) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.update_username(&name)
}

/// Update avatar
#[tauri::command]
pub async fn update_avatar(avatar: Option<String>) -> Result<()> {
    let manager = get_collaboration_manager()?;
    manager.update_avatar(avatar.as_deref())
}

/// Get collaboration statistics
#[tauri::command]
pub async fn get_collaboration_statistics() -> Result<crate::collaboration::CollaborationStatistics> {
    let manager = get_collaboration_manager()?;
    manager.get_statistics()
}
