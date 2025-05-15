// Re-export platform-specific performance implementations
pub mod platform;

// Re-export important types and functions
pub use self::platform::{
    get_performance_manager,
    PerformanceManager,
    HardwareCapabilities,
    CpuFeatures,
    MemoryInfo,
    GpuInfo,
    ThermalInfo,
    PlatformType,
};
