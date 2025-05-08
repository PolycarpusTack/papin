use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rand::Rng;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RolloutStrategy {
    AllUsers,
    PercentageRollout(f64),
    UserGroups(Vec<String>),
    CanaryGroup(String, f64),
    DeviceTypes(Vec<String>),
    DateRange { start: i64, end: Option<i64> },
    Expression(String),
}

impl std::fmt::Display for RolloutStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RolloutStrategy::AllUsers => write!(f, "All Users"),
            RolloutStrategy::PercentageRollout(pct) => write!(f, "{}% of Users", pct * 100.0),
            RolloutStrategy::UserGroups(groups) => write!(f, "User Groups: {}", groups.join(", ")),
            RolloutStrategy::CanaryGroup(group, pct) => write!(f, "Canary Group: {} ({}%)", group, pct * 100.0),
            RolloutStrategy::DeviceTypes(devices) => write!(f, "Device Types: {}", devices.join(", ")),
            RolloutStrategy::DateRange { start, end } => {
                if let Some(end_date) = end {
                    write!(f, "Date Range: {} to {}", start, end_date)
                } else {
                    write!(f, "Date Range: {} onwards", start)
                }
            },
            RolloutStrategy::Expression(expr) => write!(f, "Custom: {}", expr),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub rollout_strategy: RolloutStrategy,
    pub dependencies: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagConfig {
    pub client_id: String,
    pub user_id: Option<String>,
    pub user_groups: Vec<String>,
    pub device_type: String,
    pub canary_groups: HashMap<String, bool>,
    pub canary_percentage: f64,
    pub environment: String,
    pub version: String,
}

pub struct FeatureFlagManager {
    pub config: RwLock<FeatureFlagConfig>,
    pub flags: RwLock<HashMap<String, FeatureFlag>>,
}

impl FeatureFlagManager {
    pub fn new(config: FeatureFlagConfig) -> Self {
        Self {
            config: RwLock::new(config),
            flags: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn load_flags(&self, flags: Vec<FeatureFlag>) {
        let mut flag_map = self.flags.write().unwrap();
        *flag_map = flags.into_iter().map(|flag| (flag.id.clone(), flag)).collect();
    }
    
    pub fn is_enabled(&self, flag_id: &str) -> bool {
        let flags = self.flags.read().unwrap();
        let config = self.config.read().unwrap();
        
        if let Some(flag) = flags.get(flag_id) {
            if !flag.enabled {
                return false;
            }
            
            // Check dependencies
            for dep_id in &flag.dependencies {
                if !self.is_enabled(dep_id) {
                    return false;
                }
            }
            
            // Check rollout strategy
            match &flag.rollout_strategy {
                RolloutStrategy::AllUsers => true,
                
                RolloutStrategy::PercentageRollout(percentage) => {
                    // Use client ID as seed for deterministic randomness
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash_slice(config.client_id.as_bytes(), &mut hasher);
                    std::hash::Hash::hash_slice(flag_id.as_bytes(), &mut hasher);
                    let hash = std::hash::Hasher::finish(&hasher);
                    
                    // Convert to 0-1 range
                    let normalized = (hash as f64) / (u64::MAX as f64);
                    
                    normalized < *percentage
                },
                
                RolloutStrategy::UserGroups(groups) => {
                    if let Some(user_id) = &config.user_id {
                        for group in groups {
                            if config.user_groups.contains(group) {
                                return true;
                            }
                        }
                    }
                    false
                },
                
                RolloutStrategy::CanaryGroup(group_name, percentage) => {
                    if let Some(is_in_group) = config.canary_groups.get(group_name) {
                        if *is_in_group {
                            // Use client ID as seed for deterministic randomness
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            std::hash::Hash::hash_slice(config.client_id.as_bytes(), &mut hasher);
                            std::hash::Hash::hash_slice(flag_id.as_bytes(), &mut hasher);
                            let hash = std::hash::Hasher::finish(&hasher);
                            
                            // Convert to 0-1 range
                            let normalized = (hash as f64) / (u64::MAX as f64);
                            
                            return normalized < *percentage;
                        }
                    }
                    false
                },
                
                RolloutStrategy::DeviceTypes(device_types) => {
                    device_types.contains(&config.device_type)
                },
                
                RolloutStrategy::DateRange { start, end } => {
                    let now = Utc::now().timestamp();
                    now >= *start && (end.is_none() || now <= end.unwrap())
                },
                
                RolloutStrategy::Expression(expr) => {
                    // In a real implementation, this would evaluate a condition expression
                    // For this example, we'll just return false
                    false
                },
            }
        } else {
            false
        }
    }
    
    pub fn get_flag(&self, flag_id: &str) -> Option<FeatureFlag> {
        let flags = self.flags.read().unwrap();
        flags.get(flag_id).cloned()
    }
    
    pub fn update_config(&self, new_config: FeatureFlagConfig) {
        let mut config = self.config.write().unwrap();
        *config = new_config;
    }
    
    pub fn get_enabled_flags(&self) -> Vec<FeatureFlag> {
        let flags = self.flags.read().unwrap();
        flags.values()
            .filter(|flag| self.is_enabled(&flag.id))
            .cloned()
            .collect()
    }
    
    pub fn opt_into_canary_group(&self, group_name: &str) {
        let mut config = self.config.write().unwrap();
        config.canary_groups.insert(group_name.to_string(), true);
    }
    
    pub fn opt_out_of_canary_group(&self, group_name: &str) {
        let mut config = self.config.write().unwrap();
        config.canary_groups.insert(group_name.to_string(), false);
    }
    
    pub fn set_canary_percentage(&self, percentage: f64) {
        let mut config = self.config.write().unwrap();
        config.canary_percentage = percentage.clamp(0.0, 1.0);
    }
    
    pub fn create_flag(&self, name: &str, description: &str, 
                       rollout_strategy: RolloutStrategy, 
                       dependencies: Vec<String>) -> FeatureFlag {
        let flag = FeatureFlag {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            enabled: true,
            rollout_strategy,
            dependencies,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        };
        
        let mut flags = self.flags.write().unwrap();
        flags.insert(flag.id.clone(), flag.clone());
        
        flag
    }
    
    pub fn update_flag(&self, flag: FeatureFlag) -> Result<(), String> {
        let mut flags = self.flags.write().unwrap();
        
        if !flags.contains_key(&flag.id) {
            return Err(format!("Flag with ID {} not found", flag.id));
        }
        
        flags.insert(flag.id.clone(), flag);
        Ok(())
    }
    
    pub fn delete_flag(&self, flag_id: &str) -> Result<(), String> {
        let mut flags = self.flags.write().unwrap();
        
        if !flags.contains_key(flag_id) {
            return Err(format!("Flag with ID {} not found", flag_id));
        }
        
        flags.remove(flag_id);
        Ok(())
    }
    
    pub fn toggle_flag(&self, flag_id: &str, enabled: bool) -> Result<(), String> {
        let mut flags = self.flags.write().unwrap();
        
        if let Some(flag) = flags.get_mut(flag_id) {
            flag.enabled = enabled;
            flag.updated_at = Utc::now().timestamp();
            Ok(())
        } else {
            Err(format!("Flag with ID {} not found", flag_id))
        }
    }
    
    pub fn get_all_flags(&self) -> Vec<FeatureFlag> {
        let flags = self.flags.read().unwrap();
        flags.values().cloned().collect()
    }
}

// Create a global instance
lazy_static::lazy_static! {
    pub static ref FEATURE_FLAG_MANAGER: Arc<FeatureFlagManager> = {
        // Create default config
        let config = FeatureFlagConfig {
            client_id: Uuid::new_v4().to_string(),
            user_id: None,
            user_groups: Vec::new(),
            device_type: detect_device_type(),
            canary_groups: HashMap::new(),
            canary_percentage: 0.0, // By default, not in canary program
            environment: "production".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        Arc::new(FeatureFlagManager::new(config))
    };
}

fn detect_device_type() -> String {
    #[cfg(target_os = "windows")]
    return "windows".to_string();
    
    #[cfg(target_os = "macos")]
    return "macos".to_string();
    
    #[cfg(target_os = "linux")]
    return "linux".to_string();
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown".to_string();
}

// Feature flag check macro
#[macro_export]
macro_rules! feature_enabled {
    ($flag_id:expr) => {
        crate::feature_flags::FEATURE_FLAG_MANAGER.is_enabled($flag_id)
    };
}

// Predefined canary groups
pub const CANARY_GROUP_ALPHA: &str = "alpha";
pub const CANARY_GROUP_BETA: &str = "beta";
pub const CANARY_GROUP_EARLY_ACCESS: &str = "early_access";
pub const CANARY_GROUP_INTERNAL: &str = "internal";

// Feature flag IDs for observability features
pub const FLAG_ADVANCED_TELEMETRY: &str = "advanced_telemetry";
pub const FLAG_PERFORMANCE_DASHBOARD: &str = "performance_dashboard";
pub const FLAG_DEBUG_LOGGING: &str = "debug_logging";
pub const FLAG_RESOURCE_MONITORING: &str = "resource_monitoring";
pub const FLAG_CRASH_REPORTING: &str = "crash_reporting";

// Tauri commands
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_feature_flags() -> Vec<FeatureFlag> {
    FEATURE_FLAG_MANAGER.get_all_flags()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn toggle_feature_flag(flag_id: String, enabled: bool) -> Result<(), String> {
    FEATURE_FLAG_MANAGER.toggle_flag(&flag_id, enabled)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn create_feature_flag(
    name: String, 
    description: String, 
    rollout_strategy: RolloutStrategy, 
    dependencies: Option<Vec<String>>
) -> FeatureFlag {
    FEATURE_FLAG_MANAGER.create_flag(
        &name, 
        &description, 
        rollout_strategy, 
        dependencies.unwrap_or_default()
    )
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn delete_feature_flag(flag_id: String) -> Result<(), String> {
    FEATURE_FLAG_MANAGER.delete_flag(&flag_id)
}