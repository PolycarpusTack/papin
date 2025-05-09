use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use log::{debug, info, warn, error};

use crate::feature_flags::{FeatureFlags, FeatureManager};
use crate::observability::metrics::{MetricsCollector, Metric, MetricType, HistogramStats, TimerStats};
use crate::observability::telemetry::{TelemetryClient, track_feature_usage};
use crate::error::Result;

/// Canary release system for safely rolling out new features to users
/// 
/// This module provides a way to gradually roll out new features to users,
/// monitor their performance, and if necessary, roll back problematic features.

/// Canary group definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanaryGroup {
    /// Internal development and testing (employees only)
    Internal,
    
    /// Alpha testers (opt-in, limited set of users)
    Alpha, 
    
    /// Beta testers (broader group, opt-in)
    Beta,
    
    /// Early access (production-ready features, limited rollout)
    EarlyAccess,
    
    /// All users (full availability)
    AllUsers,
}

impl CanaryGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            CanaryGroup::Internal => "internal",
            CanaryGroup::Alpha => "alpha",
            CanaryGroup::Beta => "beta",
            CanaryGroup::EarlyAccess => "early_access",
            CanaryGroup::AllUsers => "all_users",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "internal" => Some(CanaryGroup::Internal),
            "alpha" => Some(CanaryGroup::Alpha),
            "beta" => Some(CanaryGroup::Beta),
            "early_access" | "earlyaccess" => Some(CanaryGroup::EarlyAccess),
            "all_users" | "allusers" | "all" => Some(CanaryGroup::AllUsers),
            _ => None,
        }
    }
}

/// Feature rollout status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RolloutStatus {
    /// Feature is not available to any users
    Disabled,
    
    /// Feature is available to a specific canary group
    CanaryGroup(CanaryGroup),
    
    /// Feature is available to a percentage of users
    PercentRollout(u8),
    
    /// Feature is available to all users
    FullyEnabled,
    
    /// Feature was rolled back due to issues
    RolledBack,
}

impl RolloutStatus {
    pub fn is_enabled_for_group(&self, group: CanaryGroup) -> bool {
        match self {
            RolloutStatus::Disabled => false,
            RolloutStatus::CanaryGroup(canary_group) => {
                // If the user's group has same or higher access level
                match (group, canary_group) {
                    // Internal has access to everything
                    (CanaryGroup::Internal, _) => true,
                    
                    // Alpha has access to Alpha, Beta, Early Access
                    (CanaryGroup::Alpha, CanaryGroup::Alpha) => true,
                    (CanaryGroup::Alpha, CanaryGroup::Beta) => true,
                    (CanaryGroup::Alpha, CanaryGroup::EarlyAccess) => true,
                    (CanaryGroup::Alpha, CanaryGroup::Internal) => false,
                    
                    // Beta has access to Beta, Early Access
                    (CanaryGroup::Beta, CanaryGroup::Beta) => true,
                    (CanaryGroup::Beta, CanaryGroup::EarlyAccess) => true,
                    (CanaryGroup::Beta, _) => false,
                    
                    // Early Access has access to Early Access only
                    (CanaryGroup::EarlyAccess, CanaryGroup::EarlyAccess) => true,
                    (CanaryGroup::EarlyAccess, _) => false,
                    
                    // All Users only have access to features available to all users
                    (CanaryGroup::AllUsers, CanaryGroup::AllUsers) => true,
                    (CanaryGroup::AllUsers, _) => false,
                }
            },
            RolloutStatus::PercentRollout(_) => {
                // Internal, Alpha, and Beta users always get percent rollouts
                matches!(group, CanaryGroup::Internal | CanaryGroup::Alpha | CanaryGroup::Beta)
            },
            RolloutStatus::FullyEnabled => true,
            RolloutStatus::RolledBack => false,
        }
    }
    
    pub fn is_enabled_for_user(&self, user_id: &str, user_group: CanaryGroup) -> bool {
        match self {
            RolloutStatus::Disabled => false,
            RolloutStatus::CanaryGroup(group) => self.is_enabled_for_group(user_group),
            RolloutStatus::PercentRollout(percent) => {
                // Always enable for internal, alpha, and beta users
                if matches!(user_group, CanaryGroup::Internal | CanaryGroup::Alpha | CanaryGroup::Beta) {
                    return true;
                }
                
                // For others, check the percentage
                let percent = *percent as u32;
                if percent >= 100 {
                    return true;
                }
                
                // Use a hash of the user ID to determine if they're in the rollout
                let mut seed_bytes = [0u8; 32];
                
                // Use the user ID as a seed for deterministic randomness
                let user_bytes = user_id.as_bytes();
                for (i, &byte) in user_bytes.iter().enumerate() {
                    seed_bytes[i % 32] ^= byte;
                }
                
                // Create a deterministic RNG
                let mut rng = StdRng::from_seed(seed_bytes);
                
                // Generate a number between 0 and 99
                let user_value = rng.gen_range(0..100);
                
                // If the user's value is less than the percentage, they get the feature
                user_value < percent
            },
            RolloutStatus::FullyEnabled => true,
            RolloutStatus::RolledBack => false,
        }
    }
}

/// Feature rollout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRollout {
    pub feature_name: String,
    pub feature_flag: Option<FeatureFlags>,
    pub description: String,
    pub status: RolloutStatus,
    pub version_introduced: String,
    pub metrics: Vec<String>,
    pub rollout_schedule: Option<HashMap<DateTime<Utc>, RolloutStatus>>,
    pub automatic: bool,
    pub rollback_threshold: Option<MetricThreshold>,
    pub owners: Vec<String>,
}

/// Metric threshold for automatic rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    pub metric_name: String,
    pub metric_type: MetricType,
    pub operator: ThresholdOperator,
    pub value: f64,
    pub duration_minutes: u32,
}

/// Threshold operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

impl ThresholdOperator {
    pub fn evaluate(&self, a: f64, b: f64) -> bool {
        match self {
            ThresholdOperator::GreaterThan => a > b,
            ThresholdOperator::LessThan => a < b,
            ThresholdOperator::GreaterThanOrEqual => a >= b,
            ThresholdOperator::LessThanOrEqual => a <= b,
            ThresholdOperator::Equal => (a - b).abs() < f64::EPSILON,
            ThresholdOperator::NotEqual => (a - b).abs() >= f64::EPSILON,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ThresholdOperator::GreaterThan => ">",
            ThresholdOperator::LessThan => "<",
            ThresholdOperator::GreaterThanOrEqual => ">=",
            ThresholdOperator::LessThanOrEqual => "<=",
            ThresholdOperator::Equal => "==",
            ThresholdOperator::NotEqual => "!=",
        }
    }
}

/// Canary release configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub enabled: bool,
    pub user_id: String,
    pub user_group: CanaryGroup,
    pub opt_in_features: Vec<String>,
    pub feature_rollouts: HashMap<String, FeatureRollout>,
    pub metrics_comparison_enabled: bool,
    pub auto_rollback_enabled: bool,
    pub monitoring_interval_minutes: u32,
}

impl Default for CanaryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            user_id: Uuid::new_v4().to_string(),
            user_group: CanaryGroup::AllUsers,
            opt_in_features: Vec::new(),
            feature_rollouts: HashMap::new(),
            metrics_comparison_enabled: true,
            auto_rollback_enabled: true,
            monitoring_interval_minutes: 15,
        }
    }
}

/// Metrics comparison between control and canary groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsComparison {
    pub feature_name: String,
    pub timestamp: DateTime<Utc>,
    pub control_metrics: HashMap<String, MetricValue>,
    pub canary_metrics: HashMap<String, MetricValue>,
    pub difference_percent: HashMap<String, f64>,
    pub alerts: Vec<MetricAlert>,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(f64),
    Gauge(f64),
    Histogram(HistogramStats),
    Timer(TimerStats),
}

/// Metric alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAlert {
    pub metric_name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub threshold: String,
    pub actual_value: String,
    pub timestamp: DateTime<Utc>,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Main canary release manager
pub struct CanaryManager {
    config: Arc<RwLock<CanaryConfig>>,
    feature_manager: Arc<RwLock<FeatureManager>>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    telemetry_client: Option<Arc<TelemetryClient>>,
    metrics_history: Arc<RwLock<HashMap<String, Vec<MetricsComparison>>>>,
    last_check: Arc<RwLock<DateTime<Utc>>>,
}

impl CanaryManager {
    pub fn new(
        config: CanaryConfig,
        feature_manager: Arc<RwLock<FeatureManager>>,
        metrics_collector: Option<Arc<MetricsCollector>>,
        telemetry_client: Option<Arc<TelemetryClient>>,
    ) -> Self {
        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            feature_manager,
            metrics_collector,
            telemetry_client,
            metrics_history: Arc::new(RwLock::new(HashMap::new())),
            last_check: Arc::new(RwLock::new(Utc::now())),
        };
        
        // Initialize feature flags based on config
        manager.update_feature_flags();
        
        // Start monitoring scheduled rollouts
        manager.start_rollout_scheduler();
        
        manager
    }
    
    /// Check if a feature is enabled for the current user
    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        let config = self.config.read().unwrap();
        
        // Check if canary system is enabled
        if !config.enabled {
            // Fallback to feature flag system
            if let Some(rollout) = config.feature_rollouts.get(feature_name) {
                if let Some(flag) = rollout.feature_flag {
                    return self.feature_manager.read().unwrap().is_enabled(flag);
                }
            }
            return false;
        }
        
        // Check if this feature is in the rollout config
        if let Some(rollout) = config.feature_rollouts.get(feature_name) {
            // Check if feature is enabled based on rollout status
            let is_enabled = rollout.status.is_enabled_for_user(&config.user_id, config.user_group);
            
            // Check if user has explicitly opted in
            let opted_in = config.opt_in_features.contains(&feature_name.to_string());
            
            // Track feature check
            if let Some(client) = &self.telemetry_client {
                let mut properties = HashMap::new();
                properties.insert("feature".to_string(), feature_name.to_string());
                properties.insert("enabled".to_string(), is_enabled.to_string());
                properties.insert("opted_in".to_string(), opted_in.to_string());
                properties.insert("user_group".to_string(), config.user_group.as_str().to_string());
                
                track_feature_usage(&format!("canary_check_{}", feature_name), Some(properties));
            }
            
            return is_enabled || opted_in;
        }
        
        // If feature not in canary system, fall back to feature flag system
        let flag_result = if let Some(rollout) = config.feature_rollouts.get(feature_name) {
            if let Some(flag) = rollout.feature_flag {
                self.feature_manager.read().unwrap().is_enabled(flag)
            } else {
                false
            }
        } else {
            false
        };
        
        flag_result
    }
    
    /// Get the user's canary group
    pub fn get_user_group(&self) -> CanaryGroup {
        self.config.read().unwrap().user_group
    }
    
    /// Set the user's canary group
    pub fn set_user_group(&self, group: CanaryGroup) {
        let mut config = self.config.write().unwrap();
        config.user_group = group;
        
        // Update feature flags based on new group
        drop(config); // Release lock before calling update_feature_flags
        self.update_feature_flags();
        
        // Track group change
        if let Some(client) = &self.telemetry_client {
            let mut properties = HashMap::new();
            properties.insert("group".to_string(), group.as_str().to_string());
            
            track_feature_usage("canary_group_change", Some(properties));
        }
    }
    
    /// Opt in to a specific feature
    pub fn opt_in_feature(&self, feature_name: &str) -> Result<()> {
        let mut config = self.config.write().unwrap();
        
        // Check if feature exists
        if !config.feature_rollouts.contains_key(feature_name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Feature '{}' not found in canary system", feature_name),
            ).into());
        }
        
        // Add to opt-in list if not already there
        if !config.opt_in_features.contains(&feature_name.to_string()) {
            config.opt_in_features.push(feature_name.to_string());
        }
        
        // Update feature flag if needed
        if let Some(rollout) = config.feature_rollouts.get(feature_name) {
            if let Some(flag) = rollout.feature_flag {
                drop(config); // Release lock before modifying feature manager
                self.feature_manager.write().unwrap().enable(flag);
            } else {
                drop(config); // Release lock
            }
        } else {
            drop(config); // Release lock
        }
        
        // Track opt-in
        if let Some(client) = &self.telemetry_client {
            let mut properties = HashMap::new();
            properties.insert("feature".to_string(), feature_name.to_string());
            
            track_feature_usage("canary_opt_in", Some(properties));
        }
        
        Ok(())
    }
    
    /// Opt out of a specific feature
    pub fn opt_out_feature(&self, feature_name: &str) -> Result<()> {
        let mut config = self.config.write().unwrap();
        
        // Remove from opt-in list
        config.opt_in_features.retain(|f| f != feature_name);
        
        // Store values we need after dropping the lock
        let should_disable = if let Some(rollout) = config.feature_rollouts.get(feature_name) {
            if let Some(flag) = rollout.feature_flag {
                // Only disable if not otherwise enabled through user's group
                let should_disable = !rollout.status.is_enabled_for_user(&config.user_id, config.user_group);
                
                if should_disable {
                    Some(flag)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        drop(config); // Release lock
        
        // Update feature flag if needed
        if let Some(flag) = should_disable {
            self.feature_manager.write().unwrap().disable(flag);
        }
        
        // Track opt-out
        if let Some(client) = &self.telemetry_client {
            let mut properties = HashMap::new();
            properties.insert("feature".to_string(), feature_name.to_string());
            
            track_feature_usage("canary_opt_out", Some(properties));
        }
        
        Ok(())
    }
    
    /// Add a new feature rollout
    pub fn add_feature_rollout(&self, rollout: FeatureRollout) -> Result<()> {
        let mut config = self.config.write().unwrap();
        
        // Add to rollouts map
        config.feature_rollouts.insert(rollout.feature_name.clone(), rollout.clone());
        
        // Store values we need after dropping the lock
        let feature_flag = rollout.feature_flag;
        let status = rollout.status;
        let user_id = config.user_id.clone();
        let user_group = config.user_group;
        let opt_in = config.opt_in_features.contains(&rollout.feature_name);
        
        drop(config); // Release lock
        
        // Update feature flag if needed
        if let Some(flag) = feature_flag {
            let mut feature_manager = self.feature_manager.write().unwrap();
            
            if status.is_enabled_for_user(&user_id, user_group) || opt_in {
                feature_manager.enable(flag);
            } else {
                feature_manager.disable(flag);
            }
        }
        
        // Track new rollout
        if let Some(client) = &self.telemetry_client {
            let mut properties = HashMap::new();
            properties.insert("feature".to_string(), rollout.feature_name.clone());
            properties.insert("status".to_string(), format!("{:?}", rollout.status));
            
            track_feature_usage("canary_rollout_added", Some(properties));
        }
        
        Ok(())
    }
    
    /// Update a feature rollout
    pub fn update_feature_rollout(&self, feature_name: &str, new_status: RolloutStatus) -> Result<()> {
        // Get values under a read lock first to minimize write lock time
        let (old_status, flag, automatic, user_id, user_group, opt_in) = {
            let config = self.config.read().unwrap();
            
            if let Some(rollout) = config.feature_rollouts.get(feature_name) {
                let old_status = rollout.status;
                let flag = rollout.feature_flag;
                let automatic = rollout.automatic;
                let user_id = config.user_id.clone();
                let user_group = config.user_group;
                let opt_in = config.opt_in_features.contains(&rollout.feature_name);
                
                (Some(old_status), flag, automatic, user_id, user_group, opt_in)
            } else {
                (None, None, false, String::new(), CanaryGroup::AllUsers, false)
            }
        };
        
        // If feature doesn't exist, return error
        if old_status.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Feature '{}' not found in canary system", feature_name),
            ).into());
        }
        
        // Update status
        {
            let mut config = self.config.write().unwrap();
            if let Some(rollout) = config.feature_rollouts.get_mut(feature_name) {
                rollout.status = new_status;
            }
        }
        
        // Update feature flag if needed
        if let Some(flag) = flag {
            let mut feature_manager = self.feature_manager.write().unwrap();
            
            if new_status.is_enabled_for_user(&user_id, user_group) || opt_in {
                feature_manager.enable(flag);
            } else {
                feature_manager.disable(flag);
            }
        }
        
        // Track status change
        if let Some(client) = &self.telemetry_client {
            let mut properties = HashMap::new();
            properties.insert("feature".to_string(), feature_name.to_string());
            properties.insert("old_status".to_string(), format!("{:?}", old_status.unwrap()));
            properties.insert("new_status".to_string(), format!("{:?}", new_status));
            
            track_feature_usage("canary_rollout_updated", Some(properties));
        }
        
        Ok(())
    }
    
    /// Remove a feature rollout
    pub fn remove_feature_rollout(&self, feature_name: &str) -> Result<()> {
        // Extract values under read lock first
        let flag = {
            let config = self.config.read().unwrap();
            
            if let Some(rollout) = config.feature_rollouts.get(feature_name) {
                rollout.feature_flag
            } else {
                None
            }
        };
        
        // Update with write lock
        {
            let mut config = self.config.write().unwrap();
            
            // Remove if it exists
            if !config.feature_rollouts.contains_key(feature_name) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Feature '{}' not found in canary system", feature_name),
                ).into());
            }
            
            config.feature_rollouts.remove(feature_name);
            
            // Remove from opt-in list
            config.opt_in_features.retain(|f| f != feature_name);
        }
        
        // Disable feature flag if needed
        if let Some(flag) = flag {
            self.feature_manager.write().unwrap().disable(flag);
        }
        
        // Track removal
        if let Some(client) = &self.telemetry_client {
            let mut properties = HashMap::new();
            properties.insert("feature".to_string(), feature_name.to_string());
            
            track_feature_usage("canary_rollout_removed", Some(properties));
        }
        
        Ok(())
    }
    
    /// Update feature flags based on canary configuration
    fn update_feature_flags(&self) {
        let config = self.config.read().unwrap();
        let mut feature_manager = self.feature_manager.write().unwrap();
        
        for (name, rollout) in &config.feature_rollouts {
            if let Some(flag) = rollout.feature_flag {
                let is_enabled = rollout.status.is_enabled_for_user(&config.user_id, config.user_group) || 
                                config.opt_in_features.contains(name);
                
                if is_enabled {
                    feature_manager.enable(flag);
                } else {
                    feature_manager.disable(flag);
                }
            }
        }
    }
    
    /// Start scheduler for automatic rollouts based on schedule
    fn start_rollout_scheduler(&self) {
        let config_arc = Arc::clone(&self.config);
        let manager_arc = Arc::new(self.clone());
        
        std::thread::spawn(move || {
            loop {
                // Sleep for a while between checks
                std::thread::sleep(std::time::Duration::from_secs(60));
                
                // Check if there are any scheduled rollouts
                let now = Utc::now();
                let mut updates = Vec::new();
                
                {
                    let config = config_arc.read().unwrap();
                    
                    // Skip if canary system is disabled
                    if !config.enabled {
                        continue;
                    }
                    
                    // Check all rollouts with schedules
                    for (name, rollout) in &config.feature_rollouts {
                        if let Some(schedule) = &rollout.rollout_schedule {
                            // Find scheduled updates that should have happened by now
                            for (scheduled_time, status) in schedule {
                                if scheduled_time <= &now && *status != rollout.status {
                                    updates.push((name.clone(), *status));
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Apply any needed updates
                for (name, status) in updates {
                    debug!("Applying scheduled rollout update for feature '{}': {:?}", name, status);
                    if let Err(e) = manager_arc.update_feature_rollout(&name, status) {
                        error!("Failed to apply scheduled rollout update: {}", e);
                    }
                }
            }
        });
    }
    
    /// Check metrics for automatic rollbacks
    pub fn check_metrics(&self) -> Vec<MetricAlert> {
        let config = self.config.read().unwrap();
        
        // Check if auto-rollback is enabled
        if !config.enabled || !config.auto_rollback_enabled {
            return Vec::new();
        }
        
        // Check if it's time to check metrics based on interval
        let now = Utc::now();
        let mut last_check = self.last_check.write().unwrap();
        let minutes_since_last_check = (now - *last_check).num_minutes() as u32;
        
        if minutes_since_last_check < config.monitoring_interval_minutes {
            return Vec::new();
        }
        
        // Update last check time
        *last_check = now;
        
        // Collect all alerts
        let mut all_alerts = Vec::new();
        
        // Check metrics for each feature that has a rollback threshold
        let metrics_collector = match &self.metrics_collector {
            Some(collector) => collector,
            None => return Vec::new(), // No metrics collector available
        };
        
        // Get metrics reports
        let counters = metrics_collector.get_counters_report().unwrap_or_default();
        let gauges = metrics_collector.get_gauges_report().unwrap_or_default();
        let histograms = metrics_collector.get_histograms_report().unwrap_or_default();
        let timers = metrics_collector.get_timers_report().unwrap_or_default();
        
        for (feature_name, rollout) in &config.feature_rollouts {
            if let Some(threshold) = &rollout.rollback_threshold {
                // Skip features that are already rolled back or fully enabled
                if matches!(rollout.status, RolloutStatus::RolledBack | RolloutStatus::FullyEnabled) {
                    continue;
                }
                
                // Check the appropriate metric based on type
                let mut threshold_exceeded = false;
                let mut actual_value = String::new();
                
                match threshold.metric_type {
                    MetricType::Counter => {
                        if let Some(value) = counters.get(&threshold.metric_name) {
                            if threshold.operator.evaluate(*value, threshold.value) {
                                threshold_exceeded = true;
                                actual_value = value.to_string();
                            }
                        }
                    },
                    MetricType::Gauge => {
                        if let Some(value) = gauges.get(&threshold.metric_name) {
                            if threshold.operator.evaluate(*value, threshold.value) {
                                threshold_exceeded = true;
                                actual_value = value.to_string();
                            }
                        }
                    },
                    MetricType::Histogram => {
                        if let Some(stats) = histograms.get(&threshold.metric_name) {
                            // Compare based on average value by default
                            if threshold.operator.evaluate(stats.avg, threshold.value) {
                                threshold_exceeded = true;
                                actual_value = stats.avg.to_string();
                            }
                        }
                    },
                    MetricType::Timer => {
                        if let Some(stats) = timers.get(&threshold.metric_name) {
                            // Compare based on average value by default
                            if threshold.operator.evaluate(stats.avg_ms, threshold.value) {
                                threshold_exceeded = true;
                                actual_value = stats.avg_ms.to_string();
                            }
                        }
                    },
                }
                
                if threshold_exceeded {
                    // Create alert
                    let alert = MetricAlert {
                        metric_name: threshold.metric_name.clone(),
                        severity: AlertSeverity::Critical,
                        message: format!(
                            "Metric '{}' threshold exceeded for feature '{}'. Threshold: {} {}, actual value: {}",
                            threshold.metric_name, 
                            feature_name,
                            threshold.operator.as_str(), 
                            threshold.value,
                            actual_value
                        ),
                        threshold: format!("{} {}", threshold.operator.as_str(), threshold.value),
                        actual_value,
                        timestamp: now,
                    };
                    
                    all_alerts.push(alert);
                    
                    // Auto-rollback if needed
                    if rollout.automatic {
                        debug!("Auto-rolling back feature '{}' due to threshold violation", feature_name);
                        drop(config); // Release read lock before acquiring write lock
                        if let Err(e) = self.update_feature_rollout(feature_name, RolloutStatus::RolledBack) {
                            error!("Failed to auto-rollback feature: {}", e);
                        }
                        return all_alerts; // Return early since we modified the config
                    }
                }
            }
        }
        
        all_alerts
    }
    
    /// Compare metrics between control and canary groups
    pub fn compare_metrics(&self, feature_name: &str) -> Option<MetricsComparison> {
        let config = self.config.read().unwrap();
        
        if !config.enabled || !config.metrics_comparison_enabled {
            return None;
        }
        
        // Check if the feature exists
        let rollout = match config.feature_rollouts.get(feature_name) {
            Some(r) => r,
            None => return None,
        };
        
        // Skip if feature is not in an active canary state
        if !matches!(rollout.status, RolloutStatus::CanaryGroup(_) | RolloutStatus::PercentRollout(_)) {
            return None;
        }
        
        // Check if we have a metrics collector
        let metrics_collector = match &self.metrics_collector {
            Some(collector) => collector,
            None => return None,
        };
        
        // Get metric reports
        let counters = metrics_collector.get_counters_report().unwrap_or_default();
        let gauges = metrics_collector.get_gauges_report().unwrap_or_default();
        let histograms = metrics_collector.get_histograms_report().unwrap_or_default();
        let timers = metrics_collector.get_timers_report().unwrap_or_default();
        
        // Collect metrics for this feature
        let mut control_metrics = HashMap::new();
        let mut canary_metrics = HashMap::new();
        let mut diff_percent = HashMap::new();
        let mut alerts = Vec::new();
        
        // Check each metric associated with this feature
        for metric_name in &rollout.metrics {
            // First, check if we have control and canary metrics
            let control_name = format!("{}_control", metric_name);
            let canary_name = format!("{}_canary", metric_name);
            
            // Check counters
            if let (Some(control), Some(canary)) = (counters.get(&control_name), counters.get(&canary_name)) {
                control_metrics.insert(metric_name.clone(), MetricValue::Counter(*control));
                canary_metrics.insert(metric_name.clone(), MetricValue::Counter(*canary));
                
                // Calculate percent difference if control is not zero
                if *control > 0.0 {
                    let diff = (canary - control) / control * 100.0;
                    diff_percent.insert(metric_name.clone(), diff);
                    
                    // Generate alert if difference is significant
                    if diff.abs() > 20.0 {
                        alerts.push(MetricAlert {
                            metric_name: metric_name.clone(),
                            severity: if diff.abs() > 50.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                            message: format!(
                                "Counter metric '{}' for feature '{}' shows {}% {} in canary group",
                                metric_name, 
                                feature_name,
                                diff.abs(),
                                if diff > 0.0 { "increase" } else { "decrease" }
                            ),
                            threshold: "20% difference".to_string(),
                            actual_value: format!("{}% difference", diff),
                            timestamp: Utc::now(),
                        });
                    }
                }
                continue;
            }
            
            // Check gauges
            if let (Some(control), Some(canary)) = (gauges.get(&control_name), gauges.get(&canary_name)) {
                control_metrics.insert(metric_name.clone(), MetricValue::Gauge(*control));
                canary_metrics.insert(metric_name.clone(), MetricValue::Gauge(*canary));
                
                // Calculate percent difference if control is not zero
                if *control > 0.0 {
                    let diff = (canary - control) / control * 100.0;
                    diff_percent.insert(metric_name.clone(), diff);
                    
                    // Generate alert if difference is significant
                    if diff.abs() > 20.0 {
                        alerts.push(MetricAlert {
                            metric_name: metric_name.clone(),
                            severity: if diff.abs() > 50.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                            message: format!(
                                "Gauge metric '{}' for feature '{}' shows {}% {} in canary group",
                                metric_name, 
                                feature_name,
                                diff.abs(),
                                if diff > 0.0 { "increase" } else { "decrease" }
                            ),
                            threshold: "20% difference".to_string(),
                            actual_value: format!("{}% difference", diff),
                            timestamp: Utc::now(),
                        });
                    }
                }
                continue;
            }
            
            // Check histograms
            if let (Some(control), Some(canary)) = (histograms.get(&control_name), histograms.get(&canary_name)) {
                control_metrics.insert(metric_name.clone(), MetricValue::Histogram(control.clone()));
                canary_metrics.insert(metric_name.clone(), MetricValue::Histogram(canary.clone()));
                
                // Calculate percent difference if control average is not zero
                if control.avg > 0.0 {
                    let diff = (canary.avg - control.avg) / control.avg * 100.0;
                    diff_percent.insert(metric_name.clone(), diff);
                    
                    // Generate alert if difference is significant
                    if diff.abs() > 20.0 {
                        alerts.push(MetricAlert {
                            metric_name: metric_name.clone(),
                            severity: if diff.abs() > 50.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                            message: format!(
                                "Histogram metric '{}' for feature '{}' shows {}% {} in canary group",
                                metric_name, 
                                feature_name,
                                diff.abs(),
                                if diff > 0.0 { "increase" } else { "decrease" }
                            ),
                            threshold: "20% difference".to_string(),
                            actual_value: format!("{}% difference", diff),
                            timestamp: Utc::now(),
                        });
                    }
                }
                continue;
            }
            
            // Check timers
            if let (Some(control), Some(canary)) = (timers.get(&control_name), timers.get(&canary_name)) {
                control_metrics.insert(metric_name.clone(), MetricValue::Timer(control.clone()));
                canary_metrics.insert(metric_name.clone(), MetricValue::Timer(canary.clone()));
                
                // Calculate percent difference if control average is not zero
                if control.avg_ms > 0.0 {
                    let diff = (canary.avg_ms - control.avg_ms) / control.avg_ms * 100.0;
                    diff_percent.insert(metric_name.clone(), diff);
                    
                    // Generate alert if difference is significant
                    if diff.abs() > 20.0 {
                        alerts.push(MetricAlert {
                            metric_name: metric_name.clone(),
                            severity: if diff.abs() > 50.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                            message: format!(
                                "Timer metric '{}' for feature '{}' shows {}% {} in canary group",
                                metric_name, 
                                feature_name,
                                diff.abs(),
                                if diff > 0.0 { "increase" } else { "decrease" }
                            ),
                            threshold: "20% difference".to_string(),
                            actual_value: format!("{}% difference", diff),
                            timestamp: Utc::now(),
                        });
                    }
                }
                continue;
            }
        }
        
        // Create comparison result
        let comparison = MetricsComparison {
            feature_name: feature_name.to_string(),
            timestamp: Utc::now(),
            control_metrics,
            canary_metrics,
            difference_percent: diff_percent,
            alerts,
        };
        
        // Save to history
        {
            let mut history = self.metrics_history.write().unwrap();
            let feature_history = history.entry(feature_name.to_string()).or_insert_with(Vec::new);
            feature_history.push(comparison.clone());
            
            // Limit history size
            if feature_history.len() > 100 {
                feature_history.remove(0);
            }
        }
        
        Some(comparison)
    }
    
    /// Get metrics history for a feature
    pub fn get_metrics_history(&self, feature_name: &str) -> Vec<MetricsComparison> {
        let history = self.metrics_history.read().unwrap();
        
        if let Some(feature_history) = history.get(feature_name) {
            feature_history.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Get all feature rollouts
    pub fn get_feature_rollouts(&self) -> HashMap<String, FeatureRollout> {
        self.config.read().unwrap().feature_rollouts.clone()
    }
    
    /// Get a specific feature rollout
    pub fn get_feature_rollout(&self, feature_name: &str) -> Option<FeatureRollout> {
        self.config.read().unwrap().feature_rollouts.get(feature_name).cloned()
    }
    
    /// Get all opt-in features
    pub fn get_opt_in_features(&self) -> Vec<String> {
        self.config.read().unwrap().opt_in_features.clone()
    }
}

// Enable deep cloning for the manager to use in the rollout scheduler
impl Clone for CanaryManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            feature_manager: Arc::clone(&self.feature_manager),
            metrics_collector: self.metrics_collector.clone(),
            telemetry_client: self.telemetry_client.clone(),
            metrics_history: Arc::clone(&self.metrics_history),
            last_check: Arc::clone(&self.last_check),
        }
    }
}

// Create a global canary manager
lazy_static::lazy_static! {
    pub static ref CANARY_MANAGER: Arc<RwLock<Option<CanaryManager>>> = Arc::new(RwLock::new(None));
}

// Initialize canary manager
pub fn init_canary_manager(
    config: CanaryConfig,
    feature_manager: Arc<RwLock<FeatureManager>>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    telemetry_client: Option<Arc<TelemetryClient>>,
) -> Result<()> {
    let manager = CanaryManager::new(
        config,
        feature_manager,
        metrics_collector,
        telemetry_client,
    );
    
    let mut global_manager = CANARY_MANAGER.write().unwrap();
    *global_manager = Some(manager);
    
    info!("Canary release manager initialized");
    
    Ok(())
}

// Helper functions to interact with the global canary manager
pub fn is_feature_enabled(feature_name: &str) -> bool {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.is_feature_enabled(feature_name)
    } else {
        false
    }
}

pub fn check_metrics() -> Vec<MetricAlert> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.check_metrics()
    } else {
        Vec::new()
    }
}

pub fn compare_metrics(feature_name: &str) -> Option<MetricsComparison> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.compare_metrics(feature_name)
    } else {
        None
    }
}

pub fn opt_in_feature(feature_name: &str) -> Result<()> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.opt_in_feature(feature_name)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Canary manager not initialized",
        ).into())
    }
}

pub fn opt_out_feature(feature_name: &str) -> Result<()> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.opt_out_feature(feature_name)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Canary manager not initialized",
        ).into())
    }
}

pub fn set_user_group(group: CanaryGroup) -> Result<()> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.set_user_group(group);
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Canary manager not initialized",
        ).into())
    }
}

// Macro to check if a feature is enabled
#[macro_export]
macro_rules! feature_enabled {
    ($feature_name:expr) => {
        $crate::observability::canary::is_feature_enabled($feature_name)
    };
}

// Tauri commands
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_canary_group() -> String {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.get_user_group().as_str().to_string()
    } else {
        "all_users".to_string()
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn set_canary_group(group: String) -> Result<()> {
    let canary_group = CanaryGroup::from_str(&group)
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid canary group: {}", group),
        ))?;
    
    set_user_group(canary_group)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_feature_rollouts() -> HashMap<String, FeatureRollout> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.get_feature_rollouts()
    } else {
        HashMap::new()
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_canary_metrics(feature_name: String) -> Option<MetricsComparison> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.compare_metrics(&feature_name)
    } else {
        None
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn opt_in_canary_feature(feature_name: String) -> Result<()> {
    opt_in_feature(&feature_name)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn opt_out_canary_feature(feature_name: String) -> Result<()> {
    opt_out_feature(&feature_name)
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_canary_opt_in_features() -> Vec<String> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.get_opt_in_features()
    } else {
        Vec::new()
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn check_canary_metrics() -> Vec<MetricAlert> {
    check_metrics()
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_metrics_history(feature_name: String) -> Vec<MetricsComparison> {
    if let Some(manager) = CANARY_MANAGER.read().unwrap().as_ref() {
        manager.get_metrics_history(&feature_name)
    } else {
        Vec::new()
    }
}
