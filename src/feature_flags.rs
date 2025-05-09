use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

bitflags! {
    /// Feature flags to control application behavior and enable/disable features
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct FeatureFlags: u32 {
        /// Enable experimental features (may be unstable)
        const EXPERIMENTAL = 0b0000_0000_0001;
        
        /// Enable development-only features (for testing)
        const DEV_FEATURES = 0b0000_0000_0010;
        
        /// Enable lazy loading of non-essential components
        const LAZY_LOAD = 0b0000_0000_0100;
        
        /// Enable plugin system
        const PLUGINS = 0b0000_0000_1000;
        
        /// Enable conversation history features
        const HISTORY = 0b0000_0001_0000;
        
        /// Enable advanced UI components
        const ADVANCED_UI = 0b0000_0010_0000;
        
        /// Enable analytics and telemetry
        const ANALYTICS = 0b0000_0100_0000;
        
        /// Enable auto-updates
        const AUTO_UPDATE = 0b0000_1000_0000;
        
        /// Enable real-time collaboration features
        const COLLABORATION = 0b0001_0000_0000;
        
        /// Default configuration for production builds
        const DEFAULT = Self::LAZY_LOAD.bits() | Self::PLUGINS.bits() | 
                        Self::HISTORY.bits() | Self::ADVANCED_UI.bits() | 
                        Self::AUTO_UPDATE.bits() | Self::COLLABORATION.bits();
        
        /// Minimal configuration for fastest startup
        const MINIMAL = Self::LAZY_LOAD.bits();
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        FeatureFlags::DEFAULT
    }
}

impl FromStr for FeatureFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = FeatureFlags::empty();
        
        for flag_str in s.split(',') {
            let flag_str = flag_str.trim().to_uppercase();
            match flag_str.as_str() {
                "EXPERIMENTAL" => flags |= FeatureFlags::EXPERIMENTAL,
                "DEV" | "DEV_FEATURES" => flags |= FeatureFlags::DEV_FEATURES,
                "LAZY_LOAD" => flags |= FeatureFlags::LAZY_LOAD,
                "PLUGINS" => flags |= FeatureFlags::PLUGINS,
                "HISTORY" => flags |= FeatureFlags::HISTORY,
                "ADVANCED_UI" => flags |= FeatureFlags::ADVANCED_UI,
                "ANALYTICS" => flags |= FeatureFlags::ANALYTICS,
                "AUTO_UPDATE" => flags |= FeatureFlags::AUTO_UPDATE,
                "COLLABORATION" => flags |= FeatureFlags::COLLABORATION,
                "DEFAULT" => flags |= FeatureFlags::DEFAULT,
                "MINIMAL" => flags |= FeatureFlags::MINIMAL,
                "" => continue,
                _ => return Err(format!("Unknown feature flag: {}", flag_str)),
            }
        }
        
        Ok(flags)
    }
}

/// FeatureManager handles the runtime management of feature flags
pub struct FeatureManager {
    flags: FeatureFlags,
}

impl FeatureManager {
    /// Create a new feature manager with the given flags
    pub fn new(flags: FeatureFlags) -> Self {
        FeatureManager { flags }
    }
    
    /// Create a new feature manager with default flags
    pub fn default() -> Self {
        FeatureManager { flags: FeatureFlags::default() }
    }
    
    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: FeatureFlags) -> bool {
        self.flags.contains(feature)
    }
    
    /// Enable a feature
    pub fn enable(&mut self, feature: FeatureFlags) {
        self.flags |= feature;
    }
    
    /// Disable a feature
    pub fn disable(&mut self, feature: FeatureFlags) {
        self.flags &= !feature;
    }
    
    /// Get the current feature flags
    pub fn flags(&self) -> FeatureFlags {
        self.flags
    }
    
    /// Load feature flags from environment
    pub fn from_env() -> Self {
        let env_flags = std::env::var("CLAUDE_MCP_FEATURES").unwrap_or_default();
        let flags = FeatureFlags::from_str(&env_flags).unwrap_or_else(|e| {
            eprintln!("Error parsing feature flags: {}", e);
            FeatureFlags::default()
        });
        
        Self::new(flags)
    }
    
    /// Get a minimal configuration for fastest startup
    pub fn minimal() -> Self {
        Self::new(FeatureFlags::MINIMAL)
    }
}

/// Helper function for reading features from config
pub fn parse_feature_config(config_str: &str) -> FeatureFlags {
    FeatureFlags::from_str(config_str).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flags() {
        let flags = FeatureFlags::default();
        assert!(flags.contains(FeatureFlags::LAZY_LOAD));
        assert!(flags.contains(FeatureFlags::PLUGINS));
        assert!(flags.contains(FeatureFlags::HISTORY));
        assert!(flags.contains(FeatureFlags::ADVANCED_UI));
        assert!(flags.contains(FeatureFlags::AUTO_UPDATE));
        assert!(flags.contains(FeatureFlags::COLLABORATION));
        
        assert!(!flags.contains(FeatureFlags::EXPERIMENTAL));
        assert!(!flags.contains(FeatureFlags::DEV_FEATURES));
    }
    
    #[test]
    fn test_minimal_flags() {
        let flags = FeatureFlags::MINIMAL;
        assert!(flags.contains(FeatureFlags::LAZY_LOAD));
        
        assert!(!flags.contains(FeatureFlags::PLUGINS));
        assert!(!flags.contains(FeatureFlags::HISTORY));
        assert!(!flags.contains(FeatureFlags::ADVANCED_UI));
        assert!(!flags.contains(FeatureFlags::AUTO_UPDATE));
        assert!(!flags.contains(FeatureFlags::COLLABORATION));
        assert!(!flags.contains(FeatureFlags::EXPERIMENTAL));
        assert!(!flags.contains(FeatureFlags::DEV_FEATURES));
    }
    
    #[test]
    fn test_from_str() {
        let flags = FeatureFlags::from_str("EXPERIMENTAL,LAZY_LOAD,COLLABORATION").unwrap();
        assert!(flags.contains(FeatureFlags::EXPERIMENTAL));
        assert!(flags.contains(FeatureFlags::LAZY_LOAD));
        assert!(flags.contains(FeatureFlags::COLLABORATION));
        assert!(!flags.contains(FeatureFlags::PLUGINS));
    }
    
    #[test]
    fn test_feature_manager() {
        let mut manager = FeatureManager::default();
        assert!(manager.is_enabled(FeatureFlags::LAZY_LOAD));
        assert!(manager.is_enabled(FeatureFlags::COLLABORATION));
        
        manager.disable(FeatureFlags::COLLABORATION);
        assert!(!manager.is_enabled(FeatureFlags::COLLABORATION));
        
        manager.enable(FeatureFlags::EXPERIMENTAL);
        assert!(manager.is_enabled(FeatureFlags::EXPERIMENTAL));
    }
}