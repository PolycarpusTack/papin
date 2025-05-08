# MCP Client Observability System

This document provides an overview of the observability features implemented in the MCP Client.

## Overview

The MCP Client includes a comprehensive observability system with the following components:

1. **Metrics Collection System**: Collects performance metrics from all parts of the application
2. **Structured Logging System**: Provides detailed, context-rich logging with multiple severity levels
3. **Telemetry System**: Sends anonymized usage data to help improve the client (with strict privacy controls)
4. **Resource Monitoring Dashboard**: Visualizes system resource usage and application performance
5. **Canary Release Infrastructure**: Enables safe rollout of new features to small user groups

All components are integrated across the three interfaces (GUI, CLI, TUI) to provide a consistent observability experience.

## Components

### Metrics Collection System

The metrics collection system tracks various performance indicators throughout the application:

- **Counter Metrics**: Track occurrences of specific events (e.g., API requests)
- **Gauge Metrics**: Track values that can go up and down (e.g., memory usage)
- **Histogram Metrics**: Track the distribution of values (e.g., response times)
- **Timer Metrics**: Measure the duration of operations

Metrics are collected with configurable sampling rates to minimize performance impact.

**Files**:
- `src-common/src/observability/metrics.rs`: Core metrics implementation
- `src-tauri/src/monitoring/resources.rs`: Resource metrics collection

### Structured Logging System

The logging system provides rich, context-aware logs with different severity levels:

- **Trace**: Extremely detailed information for debugging
- **Debug**: Detailed information for developers
- **Info**: General information about application operation
- **Warn**: Potential issues that don't affect operation
- **Error**: Issues that affect operation but don't cause failure
- **Fatal**: Critical issues that cause application failure

Logs include structured context data and can be viewed in the application's log viewer.

**Files**:
- `src-common/src/observability/logging.rs`: Logging implementation

### Telemetry System

The telemetry system collects anonymized usage data to help improve the application:

- All telemetry is **opt-in** by default
- Users have **granular control** over what data is collected
- Data is **anonymized** with a random client ID
- Users can **delete their data** at any time

The telemetry system includes the following collection categories:

- **App Lifecycle**: Application startup and shutdown events
- **Feature Usage**: Which features are used and how often
- **Errors**: Error reports to help improve stability
- **Performance**: Performance metrics to optimize the application
- **User Actions**: UI interactions and workflow patterns
- **System Info**: Anonymous system configuration data
- **Logs**: Application logs for troubleshooting (opt-in only)

**Files**:
- `src-common/src/observability/telemetry.rs`: Telemetry implementation
- `src-frontend/src/components/settings/PrivacySettings.tsx`: Privacy controls UI

### Resource Monitoring Dashboard

The resource monitoring dashboard provides real-time visibility into system resource usage:

- **CPU Usage**: Real-time and historical CPU usage
- **Memory Usage**: Real-time and historical memory usage
- **API Latency**: Response times for API requests
- **Message Counts**: Number of messages processed
- **FPS**: Frames per second for UI rendering

The dashboard is available in the GUI interface and requires the `performance_dashboard` feature.

**Files**:
- `src-frontend/src/components/dashboard/ResourceDashboard.tsx`: Dashboard UI
- `src-tauri/src/monitoring/resources.rs`: Resource metrics collection

### Canary Release Infrastructure

The canary release infrastructure enables safe rollout of new features:

- **Feature Flags**: Control feature availability based on various criteria
- **Canary Groups**: Alpha, Beta, and Early Access user groups
- **Metrics Comparison**: Compare metrics between control and canary groups
- **Promotion Workflow**: Safely promote features from canary to general availability
- **Rollback Capability**: Quickly disable problematic features

**Files**:
- `src-common/src/feature_flags/mod.rs`: Feature flag system
- `src-common/src/observability/canary.rs`: Canary release infrastructure
- `src-frontend/src/components/canary/CanaryDashboard.tsx`: Canary UI
- `src-frontend/src/contexts/FeatureFlagContext.tsx`: Feature flag React context

## Configuration

The observability system is configurable through the application settings and feature flags:

- **Metrics Collection**: Configure sampling rates and buffer sizes
- **Logging**: Set log levels, file rotation, and console output
- **Telemetry**: Enable/disable collection categories and set privacy level
- **Canary Release**: Join canary groups and configure rollout percentages

## Feature Flags

The following feature flags control observability features:

- `advanced_telemetry`: Enable advanced telemetry collection (Canary Alpha)
- `performance_dashboard`: Enable the performance monitoring dashboard (Canary Beta)
- `debug_logging`: Enable verbose debug logging (Canary Alpha)
- `resource_monitoring`: Enable system resource monitoring (All Users)
- `crash_reporting`: Enable automatic crash reporting (50% Rollout)

## Implementation Notes

The observability system is designed with the following principles:

1. **Privacy First**: All user data collection is opt-in and transparent
2. **Low Overhead**: Sampling and buffering minimize performance impact
3. **Cross-Interface**: Consistent experience across GUI, CLI, and TUI
4. **Extensible**: Easy to add new metrics and logging contexts
5. **Safe Rollouts**: Feature flags and canary releases enable safe deployment

## Future Enhancements

Planned enhancements to the observability system:

1. **Custom Dashboards**: User-defined dashboards for specific metrics
2. **Alerting**: Proactive alerts for performance issues
3. **Distributed Tracing**: End-to-end request tracing
4. **Metrics Export**: Export metrics to external monitoring systems
5. **Advanced A/B Testing**: More sophisticated canary testing capabilities