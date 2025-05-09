// LLM provider metrics module
pub mod llm;

// Re-export common types
pub use crate::src_common::observability::metrics::{
    Metric, MetricType, TimerStats, HistogramStats, MetricsHistory, 
    record_counter, increment_counter, record_gauge, record_histogram, time_operation,
    get_timers_report, get_counters_report, get_gauges_report, get_histograms_report
};

// Re-export LLM metrics
pub use self::llm::{
    LLMMetricsManager, LLMMetricsConfig, ProviderPerformanceMetrics, ModelPerformanceMetrics,
    AnonymizationLevel, ProviderEventType, ModelEventType, GenerationStatus,
    time_generation, track_provider_event, track_model_event,
    get_provider_metrics, get_model_metrics, init_llm_metrics
};
