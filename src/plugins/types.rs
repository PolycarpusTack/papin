use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

/// Plugin manifest definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier
    pub name: String,
    /// Display name for the plugin
    pub display_name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin license
    pub license: String,
    /// Main WASM file
    pub main: String,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Plugin hooks
    pub hooks: Vec<String>,
    /// Plugin configuration
    #[serde(default)]
    pub config: PluginConfig,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// Plugin settings
    #[serde(default)]
    pub settings: Vec<PluginSetting>,
}

/// Plugin setting definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSetting {
    /// Setting name
    pub name: String,
    /// Setting type
    pub r#type: String,
    /// Setting label
    pub label: String,
    /// Setting description
    pub description: String,
    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Whether this is a secret value
    #[serde(default)]
    pub secret: bool,
    /// Possible values for enum types
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<PluginEnumValue>,
}

/// Enum value for settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEnumValue {
    /// Value
    pub value: String,
    /// Display label
    pub label: String,
}

/// Plugin instance
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Plugin path
    pub path: PathBuf,
    /// Active status
    pub active: bool,
    /// Installed timestamp
    pub installed_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Current settings
    pub settings: serde_json::Value,
    /// Plugin instance ID (UUID)
    pub instance_id: String,
}

/// Plugin information for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// Active status
    pub active: bool,
    /// Installed timestamp
    pub installed_at: String,
    /// Last updated timestamp
    pub updated_at: String,
}

/// Detailed plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDetails {
    /// Basic plugin info
    pub info: PluginInfo,
    /// Plugin license
    pub license: String,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Plugin hooks
    pub hooks: Vec<String>,
    /// Plugin settings
    pub settings: Vec<PluginSetting>,
    /// Current settings values
    pub current_settings: serde_json::Value,
}

/// Permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin name
    pub plugin_name: String,
    /// Requested permissions
    pub permissions: Vec<String>,
    /// Request reason
    pub reason: String,
}

/// Plugin hook context
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Hook name
    pub hook_name: String,
    /// Context data
    pub data: HashMap<String, serde_json::Value>,
}

/// Repository plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPlugin {
    /// Plugin ID
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Latest version
    pub version: String,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// Downloads count
    pub downloads: u64,
    /// Rating (0-5)
    pub rating: f32,
    /// Repository URL
    pub repo_url: String,
    /// Download URL
    pub download_url: String,
    /// Screenshot URLs
    pub screenshots: Vec<String>,
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Local plugin package file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackage {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Package location
    pub path: PathBuf,
    /// Package checksums
    pub checksums: HashMap<String, String>,
}
