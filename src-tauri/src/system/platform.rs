// src-tauri/src/system/platform.rs
//
// Platform-specific system integration

use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};
use tauri::{
    AppHandle,
    Manager,
    Runtime,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    CustomMenuItem,
    Wry,
};
use log::{debug, info, warn, error};

/// Platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

/// Platform information struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Platform type
    pub platform: Platform,
    /// Platform version (e.g., Windows 10, macOS 12.0)
    pub version: String,
    /// Platform architecture (e.g., x86_64, aarch64)
    pub architecture: String,
    /// Additional platform-specific details
    pub details: std::collections::HashMap<String, String>,
}

impl PlatformInfo {
    /// Get current platform information
    pub fn current() -> Self {
        let platform = detect_platform();
        let version = get_platform_version();
        let architecture = std::env::consts::ARCH.to_string();
        let details = get_platform_details();
        
        Self {
            platform,
            version,
            architecture,
            details,
        }
    }
    
    /// Get the platform type as a string
    pub fn platform_name(&self) -> &'static str {
        match self.platform {
            Platform::Windows => "windows",
            Platform::MacOS => "macos",
            Platform::Linux => "linux",
            Platform::Unknown => "unknown",
        }
    }
    
    /// Check if this is a Windows platform
    pub fn is_windows(&self) -> bool {
        self.platform == Platform::Windows
    }
    
    /// Check if this is a macOS platform
    pub fn is_macos(&self) -> bool {
        self.platform == Platform::MacOS
    }
    
    /// Check if this is a Linux platform
    pub fn is_linux(&self) -> bool {
        self.platform == Platform::Linux
    }
}

/// Get current platform
fn detect_platform() -> Platform {
    #[cfg(target_os = "windows")]
    return Platform::Windows;
    
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
    
    #[cfg(target_os = "linux")]
    return Platform::Linux;
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return Platform::Unknown;
}

/// Get platform version
fn get_platform_version() -> String {
    #[cfg(target_os = "windows")]
    {
        // Try to get Windows version using PowerShell
        let output = Command::new("powershell")
            .args(["-Command", "(Get-WmiObject -Class Win32_OperatingSystem).Version"])
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version = version.trim();
                if !version.is_empty() {
                    return version.to_string();
                }
            }
        }
        
        // Fall back to os_info crate
        if let Ok(info) = os_info::get() {
            return format!("{} {}", info.os_type(), info.version());
        }
        
        "Windows (unknown version)".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        // Try to get macOS version using sw_vers
        let output = Command::new("sw_vers")
            .args(["-productVersion"])
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version = version.trim();
                if !version.is_empty() {
                    return version.to_string();
                }
            }
        }
        
        // Fall back to os_info crate
        if let Ok(info) = os_info::get() {
            return format!("{} {}", info.os_type(), info.version());
        }
        
        "macOS (unknown version)".to_string()
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try to get Linux distribution using lsb_release
        let output = Command::new("lsb_release")
            .args(["-ds"])
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version = version.trim();
                if !version.is_empty() {
                    return version.to_string();
                }
            }
        }
        
        // Fall back to os_info crate
        if let Ok(info) = os_info::get() {
            return format!("{} {}", info.os_type(), info.version());
        }
        
        "Linux (unknown distribution)".to_string()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Fall back to os_info crate
        if let Ok(info) = os_info::get() {
            return format!("{} {}", info.os_type(), info.version());
        }
        
        "Unknown".to_string()
    }
}

/// Get additional platform-specific details
fn get_platform_details() -> std::collections::HashMap<String, String> {
    let mut details = std::collections::HashMap::new();
    
    // Add common details
    details.insert("os_family".to_string(), std::env::consts::FAMILY.to_string());
    details.insert("hostname".to_string(), hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string()));
    
    #[cfg(target_os = "windows")]
    {
        // Add Windows-specific details
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "(Get-ComputerInfo).WindowsProductName"])
            .output() {
            if output.status.success() {
                let product_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                details.insert("product_name".to_string(), product_name);
            }
        }
        
        // Check if running under WSL
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "(Get-ComputerInfo).HyperVisorPresent"])
            .output() {
            if output.status.success() {
                let hypervisor = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
                if hypervisor == "true" {
                    details.insert("hypervisor".to_string(), "present".to_string());
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Add macOS-specific details
        if let Ok(output) = Command::new("sw_vers")
            .args(["-buildVersion"])
            .output() {
            if output.status.success() {
                let build = String::from_utf8_lossy(&output.stdout).trim().to_string();
                details.insert("build_version".to_string(), build);
            }
        }
        
        // Check if running on Apple Silicon
        if let Ok(output) = Command::new("uname")
            .args(["-m"])
            .output() {
            if output.status.success() {
                let arch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if arch == "arm64" {
                    details.insert("apple_silicon".to_string(), "true".to_string());
                } else {
                    details.insert("apple_silicon".to_string(), "false".to_string());
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Add Linux-specific details
        if let Ok(output) = Command::new("uname")
            .args(["-r"])
            .output() {
            if output.status.success() {
                let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
                details.insert("kernel_version".to_string(), kernel);
            }
        }
        
        // Try to get desktop environment
        for env_var in &["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION"] {
            if let Ok(value) = std::env::var(env_var) {
                if !value.is_empty() {
                    details.insert("desktop_environment".to_string(), value);
                    break;
                }
            }
        }
        
        // Check if running in a container
        if std::path::Path::new("/.dockerenv").exists() {
            details.insert("container".to_string(), "docker".to_string());
        }
    }
    
    details
}

/// Platform-specific system integration
pub struct PlatformIntegration {
    platform_info: PlatformInfo,
}

impl PlatformIntegration {
    /// Create a new platform integration
    pub fn new() -> Self {
        Self {
            platform_info: PlatformInfo::current(),
        }
    }
    
    /// Get platform information
    pub fn get_platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }
    
    /// Initialize platform-specific features
    pub fn initialize<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize platform-specific features
        match self.platform_info.platform {
            Platform::Windows => self.initialize_windows(app),
            Platform::MacOS => self.initialize_macos(app),
            Platform::Linux => self.initialize_linux(app),
            Platform::Unknown => Ok(()),
        }
    }
    
    /// Initialize Windows-specific features
    fn initialize_windows<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Windows platform integration");
        
        // Set up system tray
        self.setup_system_tray(app)?;
        
        // Register with Windows startup
        if let Err(e) = self.register_startup_windows() {
            warn!("Failed to register with Windows startup: {}", e);
        }
        
        // Set up Windows notifications
        self.setup_windows_notifications(app)?;
        
        Ok(())
    }
    
    /// Initialize macOS-specific features
    fn initialize_macos<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing macOS platform integration");
        
        // Set up macOS dock menu
        self.setup_macos_dock_menu(app)?;
        
        // Set up macOS notifications
        self.setup_macos_notifications(app)?;
        
        // Register with macOS startup
        if let Err(e) = self.register_startup_macos() {
            warn!("Failed to register with macOS startup: {}", e);
        }
        
        Ok(())
    }
    
    /// Initialize Linux-specific features
    fn initialize_linux<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Linux platform integration");
        
        // Set up system tray
        self.setup_system_tray(app)?;
        
        // Set up Linux notifications
        self.setup_linux_notifications(app)?;
        
        // Register with Linux startup
        if let Err(e) = self.register_startup_linux() {
            warn!("Failed to register with Linux startup: {}", e);
        }
        
        Ok(())
    }
    
    /// Set up system tray
    fn setup_system_tray<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        // Create system tray menu
        let tray_menu = SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("show".to_string(), "Show"))
            .add_item(CustomMenuItem::new("offline".to_string(), "Go Offline"))
            .add_item(CustomMenuItem::new("sync".to_string(), "Sync Now"))
            .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
        
        // Set up system tray
        let system_tray = SystemTray::new().with_menu(tray_menu);
        app.tray_handle().set_menu(tray_menu)?;
        
        // Set up event handler for system tray events
        app.listen_global("tauri://system-tray", move |event| {
            if let Some(SystemTrayEvent::MenuItemClick { id, .. }) = event
                .payload()
                .and_then(|payload| serde_json::from_str::<SystemTrayEvent>(payload).ok()) {
                
                match id.as_str() {
                    "show" => {
                        // Show the application window
                        if let Some(window) = app.get_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    },
                    "offline" => {
                        // Toggle offline mode
                        // This should be handled by the application logic
                        app.emit_all("toggle-offline", ()).ok();
                    },
                    "sync" => {
                        // Trigger synchronization
                        // This should be handled by the application logic
                        app.emit_all("force-sync", ()).ok();
                    },
                    "quit" => {
                        // Quit the application
                        app.exit(0);
                    },
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    /// Set up Windows notifications
    fn setup_windows_notifications<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            // Register notification handler
            app.listen_global("send-notification", move |event| {
                if let Some(payload) = event.payload() {
                    let notification: Notification = serde_json::from_str(payload).unwrap_or_default();
                    
                    // Send notification using Windows API
                    // This would use the windows_notification crate in a real implementation
                    info!("Sending Windows notification: {:?}", notification);
                }
            });
        }
        
        Ok(())
    }
    
    /// Set up macOS notifications
    fn setup_macos_notifications<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            // Register notification handler
            app.listen_global("send-notification", move |event| {
                if let Some(payload) = event.payload() {
                    let notification: Notification = serde_json::from_str(payload).unwrap_or_default();
                    
                    // Send notification using macOS Notification Center
                    // This would use the mac_notification_sys crate in a real implementation
                    info!("Sending macOS notification: {:?}", notification);
                }
            });
        }
        
        Ok(())
    }
    
    /// Set up Linux notifications
    fn setup_linux_notifications<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            // Register notification handler
            app.listen_global("send-notification", move |event| {
                if let Some(payload) = event.payload() {
                    let notification: Notification = serde_json::from_str(payload).unwrap_or_default();
                    
                    // Send notification using D-Bus
                    // This would use the notify-rust crate in a real implementation
                    info!("Sending Linux notification: {:?}", notification);
                }
            });
        }
        
        Ok(())
    }
    
    /// Set up macOS dock menu
    fn setup_macos_dock_menu<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            // Set up dock menu
            // This would use tauri::Menu and app.set_menu() in a real implementation
            info!("Setting up macOS dock menu");
        }
        
        Ok(())
    }
    
    /// Register with Windows startup
    fn register_startup_windows(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            // Register with Windows startup using registry
            // This would use the winreg crate in a real implementation
            info!("Registering with Windows startup");
        }
        
        Ok(())
    }
    
    /// Register with macOS startup
    fn register_startup_macos(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            // Register with macOS startup using Launch Agents
            // This would create a plist file in ~/Library/LaunchAgents in a real implementation
            info!("Registering with macOS startup");
        }
        
        Ok(())
    }
    
    /// Register with Linux startup
    fn register_startup_linux(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            // Register with Linux startup using desktop entry
            // This would create a .desktop file in ~/.config/autostart in a real implementation
            info!("Registering with Linux startup");
        }
        
        Ok(())
    }
    
    /// Handle minimizing the window
    pub fn handle_minimize<R: Runtime>(&self, app: &AppHandle<R>, minimize_to_tray: bool) {
        if minimize_to_tray {
            // Hide the window instead of minimizing
            if let Some(window) = app.get_window("main") {
                window.hide().ok();
            }
        } else {
            // Regular minimize
            if let Some(window) = app.get_window("main") {
                window.minimize().ok();
            }
        }
    }
    
    /// Handle window close request
    pub fn handle_close_requested<R: Runtime>(&self, app: &AppHandle<R>, minimize_on_close: bool) -> bool {
        if minimize_on_close {
            // Minimize to tray instead of closing
            self.handle_minimize(app, true);
            false // Prevent the default close behavior
        } else {
            // Allow the window to close normally
            true
        }
    }
    
    /// Create and show a native notification
    pub fn show_notification<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        title: &str,
        body: &str,
        urgency: NotificationUrgency,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let notification = Notification {
            title: title.to_string(),
            body: body.to_string(),
            urgency,
        };
        
        app.emit_all("send-notification", notification)?;
        
        Ok(())
    }
}

/// Notification urgency level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

impl Default for NotificationUrgency {
    fn default() -> Self {
        Self::Normal
    }
}

/// Notification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub urgency: NotificationUrgency,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            urgency: NotificationUrgency::default(),
        }
    }
}

// Platform integration commands
#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo::current()
}

#[tauri::command]
pub fn get_platform_name() -> String {
    let platform = detect_platform();
    match platform {
        Platform::Windows => "windows".to_string(),
        Platform::MacOS => "macos".to_string(),
        Platform::Linux => "linux".to_string(),
        Platform::Unknown => "unknown".to_string(),
    }
}

#[tauri::command]
pub fn show_platform_notification(
    app_handle: tauri::AppHandle,
    title: String,
    body: String,
    urgency: String,
) -> Result<(), String> {
    let urgency = match urgency.to_lowercase().as_str() {
        "low" => NotificationUrgency::Low,
        "normal" => NotificationUrgency::Normal,
        "critical" => NotificationUrgency::Critical,
        _ => NotificationUrgency::Normal,
    };
    
    let platform = PlatformIntegration::new();
    platform.show_notification(&app_handle, &title, &body, urgency)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        let platform = detect_platform();
        
        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);
        
        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);
        
        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }
    
    #[test]
    fn test_platform_info() {
        let info = PlatformInfo::current();
        
        assert!(!info.version.is_empty());
        assert!(!info.architecture.is_empty());
        
        #[cfg(target_os = "windows")]
        assert!(info.is_windows());
        
        #[cfg(target_os = "macos")]
        assert!(info.is_macos());
        
        #[cfg(target_os = "linux")]
        assert!(info.is_linux());
    }
    
    #[test]
    fn test_platform_details() {
        let details = get_platform_details();
        
        assert!(!details.is_empty());
        assert!(details.contains_key("os_family"));
        assert!(details.contains_key("hostname"));
    }
}
