// src-tauri/src/notification/mod.rs
//
// Platform-specific notification system

use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};
use crate::system::platform::{PlatformInfo, Platform, NotificationUrgency};

/// Notification manager
pub struct NotificationManager {
    platform_info: PlatformInfo,
    enabled: bool,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable notifications
    pub enabled: bool,
    /// Sound enabled
    pub sound_enabled: bool,
    /// Urgency level (low, normal, critical)
    pub urgency: NotificationUrgency,
    /// Group similar notifications
    pub group_similar: bool,
    /// Maximum number of notifications to show at once
    pub max_notifications: usize,
    /// Duration to show notifications (in milliseconds)
    pub duration_ms: u64,
    /// Position of notifications
    pub position: NotificationPosition,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            urgency: NotificationUrgency::Normal,
            group_similar: true,
            max_notifications: 5,
            duration_ms: 5000,
            position: NotificationPosition::default(),
        }
    }
}

/// Notification position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::TopRight
    }
}

/// Notification category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationCategory {
    /// General notification
    General,
    /// System notification
    System,
    /// Network notification
    Network,
    /// Offline mode notification
    Offline,
    /// Synchronization notification
    Sync,
    /// Error notification
    Error,
}

impl Default for NotificationCategory {
    fn default() -> Self {
        Self::General
    }
}

/// Notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    /// Notification ID
    pub id: String,
    /// Notification title
    pub title: String,
    /// Notification body
    pub body: String,
    /// Notification urgency
    pub urgency: NotificationUrgency,
    /// Notification category
    pub category: NotificationCategory,
    /// Notification timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Notification actions
    pub actions: Vec<NotificationAction>,
    /// Notification is dismissible
    pub dismissible: bool,
    /// Notification icon
    pub icon: Option<String>,
    /// Notification sound
    pub sound: Option<String>,
    /// Notification should play sound
    pub play_sound: bool,
    /// Notification group
    pub group: Option<String>,
}

impl Default for NotificationData {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: String::new(),
            body: String::new(),
            urgency: NotificationUrgency::Normal,
            category: NotificationCategory::General,
            timestamp: chrono::Utc::now(),
            actions: Vec::new(),
            dismissible: true,
            icon: None,
            sound: None,
            play_sound: true,
            group: None,
        }
    }
}

/// Notification action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Action ID
    pub id: String,
    /// Action title
    pub title: String,
    /// Action icon
    pub icon: Option<String>,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new() -> Self {
        Self {
            platform_info: PlatformInfo::current(),
            enabled: true,
        }
    }
    
    /// Enable or disable notifications
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if notifications are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Send a notification
    pub fn send_notification(&self, notification: NotificationData) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        // Log the notification
        info!("Sending notification: {:?}", notification);
        
        // Send the notification using platform-specific method
        match self.platform_info.platform {
            Platform::Windows => self.send_windows_notification(notification),
            Platform::MacOS => self.send_macos_notification(notification),
            Platform::Linux => self.send_linux_notification(notification),
            Platform::Unknown => {
                // Fall back to generic notification
                self.send_generic_notification(notification)
            }
        }
    }
    
    /// Send a Windows notification
    fn send_windows_notification(&self, notification: NotificationData) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            use windows_notification::{Toast, ToastManager};
            
            // Create a toast notification
            let toast = Toast::new(windows_notification::POWERSHELL_APP_ID)
                .title(&notification.title)
                .text1(&notification.body);
            
            // Add actions
            let toast_with_actions = notification.actions.iter().fold(toast, |toast, action| {
                toast.action(&action.title, &action.id)
            });
            
            // Show the notification
            ToastManager::get()
                .show(toast_with_actions)?;
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            self.send_generic_notification(notification)?;
        }
        
        Ok(())
    }
    
    /// Send a macOS notification
    fn send_macos_notification(&self, notification: NotificationData) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            use mac_notification_sys::{self, Notification, NotificationBuilder};
            
            // Create a notification
            let mut builder = NotificationBuilder::new()
                .title(&notification.title)
                .message(&notification.body)
                .sound(notification.play_sound);
            
            // Add actions
            for action in &notification.actions {
                builder = builder.button(&action.title);
            }
            
            // Build and send the notification
            let notification = builder.build()?;
            notification.send()?;
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            self.send_generic_notification(notification)?;
        }
        
        Ok(())
    }
    
    /// Send a Linux notification
    fn send_linux_notification(&self, notification: NotificationData) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            use notify_rust::{Notification, Urgency};
            
            // Map urgency
            let urgency = match notification.urgency {
                NotificationUrgency::Low => Urgency::Low,
                NotificationUrgency::Normal => Urgency::Normal,
                NotificationUrgency::Critical => Urgency::Critical,
            };
            
            // Create and send notification
            let mut notif = Notification::new()
                .summary(&notification.title)
                .body(&notification.body)
                .urgency(urgency)
                .timeout(notification.play_sound);
            
            // Add actions
            for action in &notification.actions {
                notif = notif.action(&action.title, &action.id);
            }
            
            // If we have an icon, use it
            if let Some(icon) = notification.icon {
                notif = notif.icon(&icon);
            }
            
            // Send the notification
            notif.show()?;
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            self.send_generic_notification(notification)?;
        }
        
        Ok(())
    }
    
    /// Send a generic notification (fallback)
    fn send_generic_notification(&self, notification: NotificationData) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simple fallback that logs the notification
        info!("Generic notification: {} - {}", notification.title, notification.body);
        
        // In a real implementation, this might use Tauri's notification API
        // or another cross-platform solution
        
        Ok(())
    }
    
    /// Create a new notification
    pub fn create_notification(
        &self,
        title: &str,
        body: &str,
        category: NotificationCategory,
    ) -> NotificationData {
        NotificationData {
            title: title.to_string(),
            body: body.to_string(),
            category,
            ..Default::default()
        }
    }
    
    /// Create a system notification
    pub fn create_system_notification(&self, title: &str, body: &str) -> NotificationData {
        self.create_notification(title, body, NotificationCategory::System)
    }
    
    /// Create a network notification
    pub fn create_network_notification(&self, title: &str, body: &str) -> NotificationData {
        self.create_notification(title, body, NotificationCategory::Network)
    }
    
    /// Create an offline notification
    pub fn create_offline_notification(&self, title: &str, body: &str) -> NotificationData {
        self.create_notification(title, body, NotificationCategory::Offline)
    }
    
    /// Create a sync notification
    pub fn create_sync_notification(&self, title: &str, body: &str) -> NotificationData {
        self.create_notification(title, body, NotificationCategory::Sync)
    }
    
    /// Create an error notification
    pub fn create_error_notification(&self, title: &str, body: &str) -> NotificationData {
        let mut notification = self.create_notification(title, body, NotificationCategory::Error);
        notification.urgency = NotificationUrgency::Critical;
        notification
    }
}

// Tauri command to send a notification
#[tauri::command]
pub fn send_notification(
    title: String,
    body: String,
    category: String,
    urgency: Option<String>,
) -> Result<(), String> {
    let manager = NotificationManager::new();
    
    // Parse category
    let category = match category.to_lowercase().as_str() {
        "general" => NotificationCategory::General,
        "system" => NotificationCategory::System,
        "network" => NotificationCategory::Network,
        "offline" => NotificationCategory::Offline,
        "sync" => NotificationCategory::Sync,
        "error" => NotificationCategory::Error,
        _ => NotificationCategory::General,
    };
    
    // Parse urgency
    let urgency = match urgency.as_deref().unwrap_or("normal").to_lowercase().as_str() {
        "low" => NotificationUrgency::Low,
        "normal" => NotificationUrgency::Normal,
        "critical" => NotificationUrgency::Critical,
        _ => NotificationUrgency::Normal,
    };
    
    // Create and send notification
    let mut notification = manager.create_notification(&title, &body, category);
    notification.urgency = urgency;
    
    manager.send_notification(notification)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_creation() {
        let manager = NotificationManager::new();
        
        // Test creating a general notification
        let notification = manager.create_notification("Test Title", "Test Body", NotificationCategory::General);
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.body, "Test Body");
        assert_eq!(notification.category, NotificationCategory::General);
        assert_eq!(notification.urgency, NotificationUrgency::Normal);
        
        // Test creating a system notification
        let notification = manager.create_system_notification("System Title", "System Body");
        assert_eq!(notification.title, "System Title");
        assert_eq!(notification.body, "System Body");
        assert_eq!(notification.category, NotificationCategory::System);
        
        // Test creating an error notification
        let notification = manager.create_error_notification("Error Title", "Error Body");
        assert_eq!(notification.title, "Error Title");
        assert_eq!(notification.body, "Error Body");
        assert_eq!(notification.category, NotificationCategory::Error);
        assert_eq!(notification.urgency, NotificationUrgency::Critical);
    }
    
    #[test]
    fn test_notification_settings() {
        // Test default settings
        let settings = NotificationSettings::default();
        assert!(settings.enabled);
        assert!(settings.sound_enabled);
        assert_eq!(settings.urgency, NotificationUrgency::Normal);
        assert_eq!(settings.position, NotificationPosition::TopRight);
    }
}
