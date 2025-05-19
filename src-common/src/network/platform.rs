// src-common/src/network/platform.rs
// Cross-platform network connectivity detection

use std::time::Duration;
use std::process::Command;
use crate::platform::fs::{platform_fs, Platform};

/// Error type for network operations
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Network timeout")]
    Timeout,
    #[error("Network connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Platform-specific network API error: {0}")]
    PlatformSpecific(String),
    #[error("Platform not supported: {0:?}")]
    PlatformNotSupported(Platform),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Network connectivity checker
pub struct NetworkConnectivity {
    /// Timeout for connectivity checks (in milliseconds)
    timeout_ms: u64,
    /// URL to check for connectivity
    check_url: String,
    /// Platform-specific implementation
    platform: Platform,
}

impl Default for NetworkConnectivity {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkConnectivity {
    /// Create a new NetworkConnectivity instance
    pub fn new() -> Self {
        Self {
            timeout_ms: 5000,
            check_url: "https://api.anthropic.com".to_string(),
            platform: platform_fs().platform(),
        }
    }

    /// Set timeout for connectivity checks
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set URL to check for connectivity
    pub fn with_check_url(mut self, url: String) -> Self {
        self.check_url = url;
        self
    }

    /// Check if network is available using platform-specific methods
    pub fn is_connected(&self) -> Result<bool, NetworkError> {
        match self.platform {
            Platform::Windows => self.check_windows_connectivity(),
            Platform::MacOS => self.check_macos_connectivity(),
            Platform::Linux => self.check_linux_connectivity(),
            Platform::Unknown => Err(NetworkError::PlatformNotSupported(Platform::Unknown)),
        }
    }

    /// Check network connectivity with fallback methods
    pub fn is_connected_with_fallback(&self) -> Result<bool, NetworkError> {
        // Try platform-specific method first
        match self.is_connected() {
            Ok(result) => Ok(result),
            Err(e) => {
                // Log error and try fallback method
                log::warn!("Platform-specific connectivity check failed: {}", e);
                log::info!("Falling back to generic connectivity check");
                self.check_generic_connectivity()
            }
        }
    }

    /// Windows-specific connectivity check
    fn check_windows_connectivity(&self) -> Result<bool, NetworkError> {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::iter::once;
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::wininet::{InternetCheckConnectionW, FLAG_ICC_FORCE_CONNECTION};

            // Convert URL to wide string
            let wide_url: Vec<u16> = OsStr::new(&self.check_url)
                .encode_wide()
                .chain(once(0)) // Add null terminator
                .collect();

            let result = unsafe {
                InternetCheckConnectionW(
                    wide_url.as_ptr(),
                    FLAG_ICC_FORCE_CONNECTION,
                    0,
                )
            };

            if result != 0 {
                return Ok(true);
            }

            // If that fails, try WinHTTP API
            // The actual implementation would use WinHTTP API directly
            // For now, we'll use our generic check as fallback
            self.check_generic_connectivity()
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Fall back to generic method if we're not on Windows
            self.check_generic_connectivity()
        }
    }

    /// macOS-specific connectivity check
    fn check_macos_connectivity(&self) -> Result<bool, NetworkError> {
        #[cfg(target_os = "macos")]
        {
            // On macOS, we would use the SystemConfiguration framework
            // This requires linking against the SystemConfiguration framework
            // and using unsafe FFI bindings
            // 
            // For a simplified implementation, we'll use the networksetup command
            // that's available on macOS systems
            let output = Command::new("networksetup")
                .args(["-getnetworkserviceenabled", "Wi-Fi"])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() == "Enabled" {
                    // Wi-Fi is enabled, now check if we have connectivity
                    return self.check_generic_connectivity();
                }
            }

            // Check Ethernet as well
            let output = Command::new("networksetup")
                .args(["-getnetworkserviceenabled", "Ethernet"])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() == "Enabled" {
                    // Ethernet is enabled, check connectivity
                    return self.check_generic_connectivity();
                }
            }

            // If we get here, none of the main interfaces are enabled
            Ok(false)
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Fall back to generic method if we're not on macOS
            self.check_generic_connectivity()
        }
    }

    /// Linux-specific connectivity check
    fn check_linux_connectivity(&self) -> Result<bool, NetworkError> {
        #[cfg(target_os = "linux")]
        {
            // On Linux, we can use NetworkManager or similar
            // This would require DBus bindings to communicate with NetworkManager
            //
            // For a simplified implementation, first we'll check if we have NetworkManager
            let nm_check = Command::new("which")
                .arg("nmcli")
                .output();

            if let Ok(output) = nm_check {
                if output.status.success() {
                    // We have NetworkManager, use it to check connectivity
                    let nm_output = Command::new("nmcli")
                        .args(["networking", "connectivity"])
                        .output()?;

                    if nm_output.status.success() {
                        let stdout = String::from_utf8_lossy(&nm_output.stdout);
                        let status = stdout.trim();
                        
                        // NetworkManager connectivity states
                        return Ok(status == "full" || status == "limited");
                    }
                }
            }

            // If NetworkManager is not available or fails, try ip route
            let route_check = Command::new("ip")
                .args(["route", "get", "8.8.8.8"])
                .output()?;

            if route_check.status.success() {
                // We have a route to the internet, but that doesn't guarantee connectivity
                // Let's do a generic check
                return self.check_generic_connectivity();
            }

            // No route to the internet
            Ok(false)
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fall back to generic method if we're not on Linux
            self.check_generic_connectivity()
        }
    }

    /// Generic connectivity check that works on all platforms
    fn check_generic_connectivity(&self) -> Result<bool, NetworkError> {
        // Try with ping first
        if let Ok(connected) = self.check_with_ping() {
            return Ok(connected);
        }

        // If ping fails, try HTTP request
        self.check_with_http()
    }

    /// Check connectivity using ping
    fn check_with_ping(&self) -> Result<bool, NetworkError> {
        #[cfg(target_os = "windows")]
        let ping_args = ["-n", "1", "-w", &self.timeout_ms.to_string(), "8.8.8.8"];

        #[cfg(not(target_os = "windows"))]
        let ping_args = ["-c", "1", "-W", &(self.timeout_ms / 1000).to_string(), "8.8.8.8"];

        let result = Command::new("ping")
            .args(ping_args)
            .output()?;

        Ok(result.status.success())
    }

    /// Check connectivity using HTTP request
    fn check_with_http(&self) -> Result<bool, NetworkError> {
        // This would normally use a lightweight HTTP client
        // For simplicity, we'll use curl which is available on most platforms
        let result = Command::new("curl")
            .args([
                "--max-time", &format!("{}", self.timeout_ms as f64 / 1000.0),
                "--silent",
                "--head",
                &self.check_url,
            ])
            .output()?;

        Ok(result.status.success())
    }

    /// Asynchronous connectivity check (just an example, would normally use async-std or tokio)
    pub async fn is_connected_async(&self) -> Result<bool, NetworkError> {
        // In a real async implementation, we would use tokio's Command or reqwest
        // For this example, we'll just wrap the synchronous method in a future
        let timeout_duration = Duration::from_millis(self.timeout_ms);
        
        // This would be replaced with a proper async implementation
        // Here we're just simulating it
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        
        struct ConnectivityFuture<'a>(&'a NetworkConnectivity);
        
        impl<'a> Future for ConnectivityFuture<'a> {
            type Output = Result<bool, NetworkError>;
            
            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                Poll::Ready(self.0.is_connected_with_fallback())
            }
        }
        
        ConnectivityFuture(self).await
    }
}

/// Create a global network connectivity checker instance
pub fn create_network_checker() -> NetworkConnectivity {
    NetworkConnectivity::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_checker_creation() {
        let checker = NetworkConnectivity::new();
        assert_eq!(checker.timeout_ms, 5000);
        assert_eq!(checker.check_url, "https://api.anthropic.com");
    }
    
    #[test]
    fn test_custom_configuration() {
        let checker = NetworkConnectivity::new()
            .with_timeout(10000)
            .with_check_url("https://example.com".to_string());
            
        assert_eq!(checker.timeout_ms, 10000);
        assert_eq!(checker.check_url, "https://example.com");
    }
}
