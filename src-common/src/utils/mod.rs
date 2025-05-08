pub mod security;
pub mod text;

use std::env;

/// Check if a feature flag is enabled
pub fn is_feature_enabled(name: &str) -> bool {
    env::var(format!("MCP_FEATURE_{}", name.to_uppercase()))
        .map(|val| val.to_lowercase() == "true" || val == "1")
        .unwrap_or(false)
}

/// Get environment variable or default
pub fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Get application name
pub fn app_name() -> String {
    env_or("MCP_APP_NAME", "Claude MCP Client")
}

/// Get application version
pub fn app_version() -> String {
    env_or("MCP_APP_VERSION", "0.1.0")
}

/// Get application platform
pub fn app_platform() -> String {
    env_or("MCP_APP_PLATFORM", std::env::consts::OS)
}
