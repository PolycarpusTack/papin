// Re-export platform-specific monitoring implementations
pub mod platform;

// Re-export important types and functions
pub use self::platform::{
    get_resource_monitor,
    ResourceMonitor,
    ResourceMetrics,
    ResourceThresholds,
    MonitoringError,
};
