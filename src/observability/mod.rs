// Observability module for provider systems
pub mod metrics;

use std::sync::Arc;
use log::{debug, info, warn, error};

use crate::src_common::observability::telemetry::TelemetryClient;
use crate::src_common::observability::ObservabilityConfig;
use crate::offline::llm::discovery::DiscoveryService;

/// Initialize observability for the provider system
pub fn init_provider_observability(
    config: &ObservabilityConfig,
    discovery_service: Option<Arc<DiscoveryService>>,
) -> Result<(), String> {
    info!("Initializing provider observability systems");
    
    // Get telemetry client
    let telemetry_client = match TelemetryClient::get_instance() {
        Ok(client) => Some(client),
        Err(e) => {
            warn!("Failed to get telemetry client: {}", e);
            None
        }
    };
    
    // Initialize LLM metrics
    let llm_metrics = metrics::init_llm_metrics(telemetry_client);
    
    // Start monitoring if discovery service is provided
    if let Some(discovery) = discovery_service {
        llm_metrics.start_monitoring(discovery);
    }
    
    info!("Provider observability systems initialized");
    
    Ok(())
}

/// Shutdown observability for the provider system
pub fn shutdown_provider_observability() -> Result<(), String> {
    info!("Shutting down provider observability systems");
    
    // Get LLM metrics manager
    if let Some(manager) = metrics::llm::get_llm_metrics_manager() {
        // Stop monitoring
        manager.stop_monitoring();
    }
    
    info!("Provider observability systems shut down");
    
    Ok(())
}

/// Get the privacy notice HTML for the LLM metrics
pub fn get_llm_metrics_privacy_notice() -> String {
    r#"
    <h1>LLM Metrics Collection Privacy Notice</h1>
    
    <p>To improve your experience with local LLM providers, we collect anonymous metrics about
    performance and usage. This data helps us optimize the application and provide better
    recommendations for provider configuration.</p>
    
    <h2>What We Collect</h2>
    
    <ul>
        <li><strong>Performance metrics:</strong> Model latency, throughput, and resource usage</li>
        <li><strong>Usage statistics:</strong> Success/failure rates, model and provider preferences</li>
        <li><strong>Error information:</strong> Categories of errors to help us improve reliability</li>
    </ul>
    
    <h2>What We DON'T Collect</h2>
    
    <ul>
        <li><strong>Your prompts or inputs:</strong> The content you send to models is never collected</li>
        <li><strong>Generated outputs:</strong> The responses from models are never collected</li>
        <li><strong>Personal information:</strong> We don't collect names, addresses, or other PII</li>
        <li><strong>API keys or credentials:</strong> Your authentication information is never collected</li>
    </ul>
    
    <h2>How We Use This Data</h2>
    
    <ul>
        <li>Improve provider detection and configuration</li>
        <li>Optimize resource usage for different models</li>
        <li>Identify common issues and develop solutions</li>
        <li>Provide better recommendations for model selection</li>
    </ul>
    
    <h2>Your Choices</h2>
    
    <p>Metrics collection is <strong>completely optional and off by default</strong>. You can:</p>
    
    <ul>
        <li>Choose your desired anonymization level</li>
        <li>Enable or disable specific categories of metrics</li>
        <li>Turn off metrics collection at any time</li>
        <li>Request deletion of any data we've collected</li>
    </ul>
    
    <p>By accepting this notice, you allow us to collect the metrics described above according to
    your configuration choices.</p>
    "#.to_string()
}
