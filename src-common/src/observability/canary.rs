use crate::feature_flags::{
    FeatureFlag, FeatureFlagManager, RolloutStrategy, FEATURE_FLAG_MANAGER,
    CANARY_GROUP_ALPHA, CANARY_GROUP_BETA, CANARY_GROUP_EARLY_ACCESS
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryGroup {
    pub name: String,
    pub description: String,
    pub percentage: f64,
    pub active_features: Vec<String>,
    pub user_count: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryMetrics {
    pub feature_id: String,
    pub group_name: String,
    pub metrics: CanaryMetricsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryMetricsData {
    pub error_rate: CanaryComparison<f64>,
    pub performance: CanaryComparison<Vec<f64>>,
    pub usage: CanaryComparison<f64>,
    pub timestamp: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryComparison<T> {
    pub control: T,
    pub canary: T,
}

pub struct CanaryReleaseService {
    groups: Arc<Mutex<Vec<CanaryGroup>>>,
    metrics: Arc<Mutex<HashMap<String, CanaryMetrics>>>,
}

impl CanaryReleaseService {
    pub fn new() -> Self {
        let default_groups = vec![
            CanaryGroup {
                name: CANARY_GROUP_ALPHA.to_string(),
                description: "Alpha testers - early testing of experimental features".to_string(),
                percentage: 0.05, // 5%
                active_features: Vec::new(),
                user_count: 0,
                enabled: true,
            },
            CanaryGroup {
                name: CANARY_GROUP_BETA.to_string(),
                description: "Beta testers - testing of nearly-complete features".to_string(),
                percentage: 0.1, // 10%
                active_features: Vec::new(),
                user_count: 0,
                enabled: true,
            },
            CanaryGroup {
                name: CANARY_GROUP_EARLY_ACCESS.to_string(),
                description: "Early access users - preview of completed features".to_string(),
                percentage: 0.2, // 20%
                active_features: Vec::new(),
                user_count: 0,
                enabled: true,
            },
        ];
        
        Self {
            groups: Arc::new(Mutex::new(default_groups)),
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn get_canary_groups(&self) -> Vec<CanaryGroup> {
        let groups = self.groups.lock().unwrap();
        groups.clone()
    }
    
    pub fn get_canary_group(&self, name: &str) -> Option<CanaryGroup> {
        let groups = self.groups.lock().unwrap();
        groups.iter().find(|group| group.name == name).cloned()
    }
    
    pub fn toggle_canary_group(&self, group_name: &str, enabled: bool) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();
        if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
            group.enabled = enabled;
            Ok(())
        } else {
            Err(format!("Canary group '{}' not found", group_name))
        }
    }
    
    pub fn update_canary_percentage(&self, group_name: &str, percentage: f64) -> Result<(), String> {
        let mut groups = self.groups.lock().unwrap();
        if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
            group.percentage = percentage.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(format!("Canary group '{}' not found", group_name))
        }
    }
    
    pub fn create_canary_feature(&self, name: &str, description: &str, group_name: &str, percentage: f64) -> Result<FeatureFlag, String> {
        // Ensure canary group exists
        {
            let groups = self.groups.lock().unwrap();
            if !groups.iter().any(|group| group.name == group_name) {
                return Err(format!("Canary group '{}' not found", group_name));
            }
        }
        
        // Create feature flag with canary rollout strategy
        let feature_flag = FeatureFlag {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            enabled: true,
            rollout_strategy: RolloutStrategy::CanaryGroup(group_name.to_string(), percentage.clamp(0.0, 1.0)),
            dependencies: Vec::new(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            metadata: HashMap::new(),
        };
        
        // Add feature to canary group
        {
            let mut groups = self.groups.lock().unwrap();
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.active_features.push(feature_flag.id.clone());
            }
        }
        
        // Create initial metrics
        let _ = self.initialize_feature_metrics(&feature_flag.id, group_name);
        
        // Return the created feature
        Ok(feature_flag)
    }
    
    pub fn toggle_canary_feature(&self, feature_id: &str, enabled: bool) -> Result<(), String> {
        // Get feature flag
        let flag_manager = FEATURE_FLAG_MANAGER.clone();
        let flag = flag_manager.get_flag(feature_id)
            .ok_or_else(|| format!("Feature flag '{}' not found", feature_id))?;
        
        // Check if it's a canary feature
        if let RolloutStrategy::CanaryGroup(_, _) = flag.rollout_strategy {
            // Toggle the feature
            flag_manager.toggle_flag(feature_id, enabled)
        } else {
            Err("Not a canary feature".to_string())
        }
    }
    
    pub fn promote_canary_feature(&self, feature_id: &str) -> Result<(), String> {
        // Get feature flag
        let flag_manager = FEATURE_FLAG_MANAGER.clone();
        let flag = flag_manager.get_flag(feature_id)
            .ok_or_else(|| format!("Feature flag '{}' not found", feature_id))?;
        
        // Only allow promoting canary features
        if let RolloutStrategy::CanaryGroup(group_name, _) = &flag.rollout_strategy {
            // Create a new flag with AllUsers strategy
            let new_flag = FeatureFlag {
                id: flag.id.clone(),
                name: flag.name.clone(),
                description: flag.description.clone(),
                enabled: flag.enabled,
                rollout_strategy: RolloutStrategy::AllUsers,
                dependencies: flag.dependencies.clone(),
                created_at: flag.created_at,
                updated_at: Utc::now().timestamp(),
                metadata: flag.metadata.clone(),
            };
            
            // Update feature flag
            flag_manager.update_flag(new_flag)?;
            
            // Remove from canary group
            {
                let mut groups = self.groups.lock().unwrap();
                if let Some(group) = groups.iter_mut().find(|g| g.name == *group_name) {
                    group.active_features.retain(|id| id != feature_id);
                }
            }
            
            Ok(())
        } else {
            Err("Only canary features can be promoted".to_string())
        }
    }
    
    pub fn rollback_canary_feature(&self, feature_id: &str) -> Result<(), String> {
        // Get feature flag
        let flag_manager = FEATURE_FLAG_MANAGER.clone();
        let flag = flag_manager.get_flag(feature_id)
            .ok_or_else(|| format!("Feature flag '{}' not found", feature_id))?;
        
        // Only allow rolling back canary features
        if let RolloutStrategy::CanaryGroup(group_name, _) = &flag.rollout_strategy {
            // Disable the feature flag
            flag_manager.toggle_flag(feature_id, false)?;
            
            // Remove from canary group
            {
                let mut groups = self.groups.lock().unwrap();
                if let Some(group) = groups.iter_mut().find(|g| g.name == *group_name) {
                    group.active_features.retain(|id| id != feature_id);
                }
            }
            
            Ok(())
        } else {
            Err("Only canary features can be rolled back".to_string())
        }
    }
    
    pub fn get_user_canary_group(&self) -> Option<String> {
        let flag_manager = FEATURE_FLAG_MANAGER.clone();
        let config = flag_manager.config.read().unwrap();
        
        // Check each group to see if user is enrolled
        for (group_name, enrolled) in &config.canary_groups {
            if *enrolled {
                return Some(group_name.clone());
            }
        }
        
        None
    }
    
    pub fn opt_into_canary_group(&self, group_name: &str) -> Result<(), String> {
        // Ensure canary group exists
        {
            let groups = self.groups.lock().unwrap();
            if !groups.iter().any(|group| group.name == group_name) {
                return Err(format!("Canary group '{}' not found", group_name));
            }
        }
        
        // Opt into canary group
        let flag_manager = FEATURE_FLAG_MANAGER.clone();
        flag_manager.opt_into_canary_group(group_name);
        
        // Update user count
        {
            let mut groups = self.groups.lock().unwrap();
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.user_count += 1;
            }
        }
        
        Ok(())
    }
    
    pub fn opt_out_of_canary_group(&self) -> Result<(), String> {
        // Get current canary group
        let current_group = self.get_user_canary_group();
        
        if let Some(group_name) = current_group {
            // Opt out of canary group
            let flag_manager = FEATURE_FLAG_MANAGER.clone();
            flag_manager.opt_out_of_canary_group(&group_name);
            
            // Update user count
            {
                let mut groups = self.groups.lock().unwrap();
                if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                    if group.user_count > 0 {
                        group.user_count -= 1;
                    }
                }
            }
            
            Ok(())
        } else {
            Err("User is not enrolled in any canary group".to_string())
        }
    }
    
    pub fn get_canary_metrics(&self, feature_id: &str, group_name: &str) -> Option<CanaryMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(&format!("{}:{}", feature_id, group_name)).cloned()
    }
    
    pub fn update_canary_metrics(&self, feature_id: &str, group_name: &str, is_canary: bool, 
                                  error_rate: Option<f64>, performance: Option<f64>, used: bool) {
        let mut metrics = self.metrics.lock().unwrap();
        let key = format!("{}:{}", feature_id, group_name);
        
        let entry = metrics.entry(key).or_insert_with(|| {
            self.initialize_feature_metrics(feature_id, group_name).unwrap_or_else(|_| {
                // Create default metrics if initialization fails
                CanaryMetrics {
                    feature_id: feature_id.to_string(),
                    group_name: group_name.to_string(),
                    metrics: CanaryMetricsData {
                        error_rate: CanaryComparison {
                            control: 0.0,
                            canary: 0.0,
                        },
                        performance: CanaryComparison {
                            control: vec![],
                            canary: vec![],
                        },
                        usage: CanaryComparison {
                            control: 0.0,
                            canary: 0.0,
                        },
                        timestamp: vec![],
                    },
                }
            })
        });
        
        let now = Utc::now().timestamp();
        
        // Update metrics
        if let Some(rate) = error_rate {
            if is_canary {
                entry.metrics.error_rate.canary = rate;
            } else {
                entry.metrics.error_rate.control = rate;
            }
        }
        
        if let Some(perf) = performance {
            // Add new performance data point
            if is_canary {
                entry.metrics.performance.canary.push(perf);
                
                // Keep only last N points
                if entry.metrics.performance.canary.len() > 20 {
                    entry.metrics.performance.canary.remove(0);
                }
            } else {
                entry.metrics.performance.control.push(perf);
                
                // Keep only last N points
                if entry.metrics.performance.control.len() > 20 {
                    entry.metrics.performance.control.remove(0);
                }
            }
            
            // Add timestamp if needed
            if entry.metrics.timestamp.len() < entry.metrics.performance.canary.len() {
                entry.metrics.timestamp.push(now);
                
                // Keep only last N timestamps
                if entry.metrics.timestamp.len() > 20 {
                    entry.metrics.timestamp.remove(0);
                }
            }
        }
        
        // Update usage if feature was used
        if used {
            // In a real implementation, this would track usage over time
            // For simplicity, we're just setting a static value
            if is_canary {
                entry.metrics.usage.canary = 75.0; // Example value
            } else {
                entry.metrics.usage.control = 80.0; // Example value
            }
        }
    }
    
    fn initialize_feature_metrics(&self, feature_id: &str, group_name: &str) -> Result<CanaryMetrics, String> {
        // Create initial metrics structure
        let metrics = CanaryMetrics {
            feature_id: feature_id.to_string(),
            group_name: group_name.to_string(),
            metrics: CanaryMetricsData {
                error_rate: CanaryComparison {
                    control: 0.0,
                    canary: 0.0,
                },
                performance: CanaryComparison {
                    control: vec![],
                    canary: vec![],
                },
                usage: CanaryComparison {
                    control: 0.0,
                    canary: 0.0,
                },
                timestamp: vec![],
            },
        };
        
        // Store metrics
        let mut metrics_map = self.metrics.lock().unwrap();
        metrics_map.insert(format!("{}:{}", feature_id, group_name), metrics.clone());
        
        Ok(metrics)
    }
}

// Singleton instance
lazy_static::lazy_static! {
    pub static ref CANARY_SERVICE: Arc<CanaryReleaseService> = {
        Arc::new(CanaryReleaseService::new())
    };
}

// Tauri commands
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_canary_groups() -> Vec<CanaryGroup> {
    CANARY_SERVICE.get_canary_groups()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_user_canary_group() -> Option<String> {
    CANARY_SERVICE.get_user_canary_group()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn opt_into_canary_group(group_name: String) -> Result<(), String> {
    CANARY_SERVICE.opt_into_canary_group(&group_name)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn opt_out_of_canary_group() -> Result<(), String> {
    CANARY_SERVICE.opt_out_of_canary_group()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn toggle_canary_group(group_name: String, enabled: bool) -> Result<(), String> {
    CANARY_SERVICE.toggle_canary_group(&group_name, enabled)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn update_canary_percentage(group_name: String, percentage: f64) -> Result<(), String> {
    CANARY_SERVICE.update_canary_percentage(&group_name, percentage)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_canary_metrics(feature_id: String, group_name: String) -> Option<CanaryMetrics> {
    CANARY_SERVICE.get_canary_metrics(&feature_id, &group_name)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn promote_canary_feature(feature_id: String) -> Result<(), String> {
    CANARY_SERVICE.promote_canary_feature(&feature_id)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn rollback_canary_feature(feature_id: String) -> Result<(), String> {
    CANARY_SERVICE.rollback_canary_feature(&feature_id)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn create_canary_feature(name: String, description: String, group_name: String, percentage: f64) -> Result<FeatureFlag, String> {
    CANARY_SERVICE.create_canary_feature(&name, &description, &group_name, percentage)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn toggle_canary_feature(feature_id: String, enabled: bool) -> Result<(), String> {
    CANARY_SERVICE.toggle_canary_feature(&feature_id, enabled)
}