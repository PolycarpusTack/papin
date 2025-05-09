// Data Flow Tracking and Visualization
//
// This module tracks the flow of data throughout the application and visualizes it for users.
// It shows users where their data goes, what systems process it, and provides transparency
// about how their information is handled.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::config::config_path;
use crate::error::Result;
use crate::observability::metrics::{record_counter, record_gauge};
use crate::security::DataClassification;

const DATA_FLOW_LOG_FILE: &str = "data_flow_log.json";
const MAX_LOG_ENTRIES: usize = 1000;

/// A data flow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowEvent {
    /// Unique ID for the event
    pub id: String,
    
    /// Type of operation (e.g., "api_request", "file_save", "encryption")
    pub operation: String,
    
    /// Data item name or description
    pub data_item: String,
    
    /// Classification level of the data
    pub classification: DataClassification,
    
    /// Source location/system
    pub source: String,
    
    /// Destination location/system
    pub destination: String,
    
    /// Whether user consent was obtained for this data flow
    pub consent_obtained: bool,
    
    /// When the data flow occurred
    pub timestamp: SystemTime,
    
    /// Whether the data was encrypted during transfer
    pub encrypted: bool,
    
    /// Additional metadata about the data flow
    pub metadata: HashMap<String, String>,
}

/// A node in the data flow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowNode {
    /// Unique ID for the node
    pub id: String,
    
    /// Name of the system or component
    pub name: String,
    
    /// Type of node (e.g., "local_storage", "remote_api", "memory")
    pub node_type: String,
    
    /// Whether the node is internal to the app
    pub internal: bool,
    
    /// Location (e.g., "local", "cloud", "third_party")
    pub location: String,
    
    /// Additional metadata about the node
    pub metadata: HashMap<String, String>,
}

/// Data flow graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowGraph {
    /// Nodes in the graph
    pub nodes: Vec<DataFlowNode>,
    
    /// Edges in the graph (source_id, destination_id, data_classifications)
    pub edges: Vec<(String, String, Vec<DataClassification>)>,
    
    /// Data items flowing through the graph
    pub data_items: HashMap<String, DataClassification>,
}

/// Data Flow Manager
pub struct DataFlowManager {
    /// Whether data flow tracking is enabled
    enabled: bool,
    
    /// Recent data flow events
    events: Arc<RwLock<VecDeque<DataFlowEvent>>>,
    
    /// Data flow graph
    graph: Arc<RwLock<DataFlowGraph>>,
    
    /// Known nodes in the system
    nodes: Arc<RwLock<HashMap<String, DataFlowNode>>>,
    
    /// Lock for file operations
    file_lock: Arc<Mutex<()>>,
}

impl DataFlowManager {
    /// Create a new Data Flow Manager
    pub fn new(enabled: bool) -> Result<Self> {
        Ok(Self {
            enabled,
            events: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))),
            graph: Arc::new(RwLock::new(DataFlowGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
                data_items: HashMap::new(),
            })),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            file_lock: Arc::new(Mutex::new(())),
        })
    }
    
    /// Start the data flow tracking service
    pub fn start_service(&self) -> Result<()> {
        if !self.enabled {
            info!("Data flow tracking is disabled");
            return Ok(());
        }
        
        // Load previous data flow events if available
        self.load_events()?;
        
        // Initialize known nodes
        self.initialize_known_nodes()?;
        
        // Build the graph from events
        self.rebuild_graph()?;
        
        info!("Data flow tracking service started with {} events", self.events.read().unwrap().len());
        record_gauge("security.data_flow.event_count", self.events.read().unwrap().len() as f64, None);
        record_counter("security.data_flow.service_started", 1.0, None);
        
        Ok(())
    }
    
    /// Initialize known nodes in the system
    fn initialize_known_nodes(&self) -> Result<()> {
        let mut nodes = self.nodes.write().unwrap();
        
        // Add standard nodes
        self.add_node_internal(&mut nodes, "local_app", "Papin", "application", true, "local", HashMap::new())?;
        self.add_node_internal(&mut nodes, "local_storage", "Local Storage", "storage", true, "local", HashMap::new())?;
        self.add_node_internal(&mut nodes, "memory", "Application Memory", "memory", true, "local", HashMap::new())?;
        self.add_node_internal(&mut nodes, "secure_enclave", "Secure Enclave", "secure_storage", true, "local", HashMap::new())?;
        self.add_node_internal(&mut nodes, "cloud_sync", "Cloud Sync Service", "api", false, "cloud", HashMap::new())?;
        self.add_node_internal(&mut nodes, "local_llm", "Local LLM", "model", true, "local", HashMap::new())?;
        self.add_node_internal(&mut nodes, "cloud_llm", "Cloud LLM", "model", false, "cloud", HashMap::new())?;
        self.add_node_internal(&mut nodes, "telemetry", "Telemetry Service", "api", false, "cloud", HashMap::new())?;
        self.add_node_internal(&mut nodes, "file_system", "File System", "storage", false, "local", HashMap::new())?;
        self.add_node_internal(&mut nodes, "clipboard", "System Clipboard", "memory", false, "local", HashMap::new())?;
        
        Ok(())
    }
    
    /// Add a node to the graph
    fn add_node_internal(
        &self,
        nodes: &mut HashMap<String, DataFlowNode>,
        id: &str,
        name: &str,
        node_type: &str,
        internal: bool,
        location: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let node = DataFlowNode {
            id: id.to_string(),
            name: name.to_string(),
            node_type: node_type.to_string(),
            internal,
            location: location.to_string(),
            metadata,
        };
        
        nodes.insert(id.to_string(), node);
        
        Ok(())
    }
    
    /// Add a custom node to the graph
    pub fn add_node(
        &self,
        id: &str,
        name: &str,
        node_type: &str,
        internal: bool,
        location: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut nodes = self.nodes.write().unwrap();
        self.add_node_internal(&mut nodes, id, name, node_type, internal, location, metadata)?;
        
        // Rebuild the graph
        drop(nodes);
        self.rebuild_graph()?;
        
        Ok(())
    }
    
    /// Enable or disable data flow tracking
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        if self.enabled == enabled {
            return Ok(()); // No change needed
        }
        
        // Update enabled state
        let mut this = unsafe { &mut *(self as *const Self as *mut Self) };
        this.enabled = enabled;
        
        if enabled {
            // Start tracking
            info!("Enabling data flow tracking");
            self.load_events()?;
            self.initialize_known_nodes()?;
            self.rebuild_graph()?;
            
            record_counter("security.data_flow.enabled", 1.0, None);
        } else {
            // Stop tracking
            info!("Disabling data flow tracking");
            self.events.write().unwrap().clear();
            
            // Clear graph
            *self.graph.write().unwrap() = DataFlowGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
                data_items: HashMap::new(),
            };
            
            record_counter("security.data_flow.disabled", 1.0, None);
        }
        
        Ok(())
    }
    
    /// Load data flow events from disk
    fn load_events(&self) -> Result<()> {
        let log_path = config_path(DATA_FLOW_LOG_FILE);
        
        if !log_path.exists() {
            // No events to load
            return Ok(());
        }
        
        let log_data = fs::read_to_string(&log_path)
            .map_err(|e| format!("Failed to read data flow log: {}", e))?;
        
        let events: Vec<DataFlowEvent> = serde_json::from_str(&log_data)
            .map_err(|e| format!("Failed to parse data flow log: {}", e))?;
        
        let mut event_queue = self.events.write().unwrap();
        event_queue.clear();
        
        for event in events {
            event_queue.push_back(event);
        }
        
        // Limit size
        while event_queue.len() > MAX_LOG_ENTRIES {
            event_queue.pop_front();
        }
        
        debug!("Loaded {} data flow events", event_queue.len());
        
        Ok(())
    }
    
    /// Save data flow events to disk
    fn save_events(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let _lock = self.file_lock.lock().unwrap();
        let log_path = config_path(DATA_FLOW_LOG_FILE);
        
        let events: Vec<DataFlowEvent> = self.events.read().unwrap().iter().cloned().collect();
        
        let log_data = serde_json::to_string_pretty(&events)
            .map_err(|e| format!("Failed to serialize data flow log: {}", e))?;
        
        fs::write(&log_path, log_data)
            .map_err(|e| format!("Failed to write data flow log: {}", e))?;
        
        debug!("Saved {} data flow events", events.len());
        
        Ok(())
    }
    
    /// Track a data flow event
    pub fn track_data_flow(
        &self,
        operation: &str,
        data_item: &str,
        classification: DataClassification,
        destination: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Create event
        let event = DataFlowEvent {
            id: uuid::Uuid::new_v4().to_string(),
            operation: operation.to_string(),
            data_item: data_item.to_string(),
            classification,
            source: "local_app".to_string(), // Default source is the app itself
            destination: destination.to_string(),
            consent_obtained: true, // Default to true, should be overridden as needed
            timestamp: SystemTime::now(),
            encrypted: destination != "memory" && destination != "local_app", // Default to encrypted for external flows
            metadata: HashMap::new(),
        };
        
        // Add event to queue
        {
            let mut events = self.events.write().unwrap();
            events.push_back(event.clone());
            
            // Limit size
            while events.len() > MAX_LOG_ENTRIES {
                events.pop_front();
            }
        }
        
        // Update the graph
        self.update_graph(&event)?;
        
        // Save events periodically (every 10 events)
        if self.events.read().unwrap().len() % 10 == 0 {
            self.save_events()?;
        }
        
        record_counter("security.data_flow.event_tracked", 1.0, None);
        
        Ok(())
    }
    
    /// Track a data flow event with more details
    pub fn track_data_flow_detailed(
        &self,
        operation: &str,
        data_item: &str,
        classification: DataClassification,
        source: &str,
        destination: &str,
        consent_obtained: bool,
        encrypted: bool,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Create event
        let event = DataFlowEvent {
            id: uuid::Uuid::new_v4().to_string(),
            operation: operation.to_string(),
            data_item: data_item.to_string(),
            classification,
            source: source.to_string(),
            destination: destination.to_string(),
            consent_obtained,
            timestamp: SystemTime::now(),
            encrypted,
            metadata,
        };
        
        // Add event to queue
        {
            let mut events = self.events.write().unwrap();
            events.push_back(event.clone());
            
            // Limit size
            while events.len() > MAX_LOG_ENTRIES {
                events.pop_front();
            }
        }
        
        // Update the graph
        self.update_graph(&event)?;
        
        // Save events periodically (every 10 events)
        if self.events.read().unwrap().len() % 10 == 0 {
            self.save_events()?;
        }
        
        record_counter("security.data_flow.event_tracked", 1.0, None);
        
        Ok(())
    }
    
    /// Update the data flow graph with a new event
    fn update_graph(&self, event: &DataFlowEvent) -> Result<()> {
        let nodes = self.nodes.read().unwrap();
        let mut graph = self.graph.write().unwrap();
        
        // Add source node if needed
        if !graph.nodes.iter().any(|n| n.id == event.source) {
            if let Some(node) = nodes.get(&event.source) {
                graph.nodes.push(node.clone());
            } else {
                // Unknown node, create a generic one
                graph.nodes.push(DataFlowNode {
                    id: event.source.clone(),
                    name: event.source.clone(),
                    node_type: "unknown".to_string(),
                    internal: false,
                    location: "unknown".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Add destination node if needed
        if !graph.nodes.iter().any(|n| n.id == event.destination) {
            if let Some(node) = nodes.get(&event.destination) {
                graph.nodes.push(node.clone());
            } else {
                // Unknown node, create a generic one
                graph.nodes.push(DataFlowNode {
                    id: event.destination.clone(),
                    name: event.destination.clone(),
                    node_type: "unknown".to_string(),
                    internal: false,
                    location: "unknown".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Add or update edge
        let edge_exists = graph.edges.iter().any(|(src, dst, _)| 
            *src == event.source && *dst == event.destination);
            
        if edge_exists {
            // Update existing edge
            for (_, _, classifications) in graph.edges.iter_mut() {
                if !classifications.contains(&event.classification) {
                    classifications.push(event.classification);
                }
            }
        } else {
            // Add new edge
            graph.edges.push((
                event.source.clone(),
                event.destination.clone(),
                vec![event.classification],
            ));
        }
        
        // Add data item
        graph.data_items.insert(event.data_item.clone(), event.classification);
        
        Ok(())
    }
    
    /// Rebuild the data flow graph from all events
    fn rebuild_graph(&self) -> Result<()> {
        let nodes = self.nodes.read().unwrap();
        let events = self.events.read().unwrap();
        let mut graph = self.graph.write().unwrap();
        
        // Clear existing graph
        *graph = DataFlowGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
            data_items: HashMap::new(),
        };
        
        // Add all nodes (will deduplicate later)
        for (_, node) in nodes.iter() {
            graph.nodes.push(node.clone());
        }
        
        // Add all events to the graph
        for event in events.iter() {
            // Add source node if needed
            if !graph.nodes.iter().any(|n| n.id == event.source) {
                if let Some(node) = nodes.get(&event.source) {
                    graph.nodes.push(node.clone());
                } else {
                    // Unknown node, create a generic one
                    graph.nodes.push(DataFlowNode {
                        id: event.source.clone(),
                        name: event.source.clone(),
                        node_type: "unknown".to_string(),
                        internal: false,
                        location: "unknown".to_string(),
                        metadata: HashMap::new(),
                    });
                }
            }
            
            // Add destination node if needed
            if !graph.nodes.iter().any(|n| n.id == event.destination) {
                if let Some(node) = nodes.get(&event.destination) {
                    graph.nodes.push(node.clone());
                } else {
                    // Unknown node, create a generic one
                    graph.nodes.push(DataFlowNode {
                        id: event.destination.clone(),
                        name: event.destination.clone(),
                        node_type: "unknown".to_string(),
                        internal: false,
                        location: "unknown".to_string(),
                        metadata: HashMap::new(),
                    });
                }
            }
            
            // Add or update edge
            let edge_idx = graph.edges.iter().position(|(src, dst, _)| 
                *src == event.source && *dst == event.destination);
                
            if let Some(idx) = edge_idx {
                // Update existing edge
                let (_, _, classifications) = &mut graph.edges[idx];
                if !classifications.contains(&event.classification) {
                    classifications.push(event.classification);
                }
            } else {
                // Add new edge
                graph.edges.push((
                    event.source.clone(),
                    event.destination.clone(),
                    vec![event.classification],
                ));
            }
            
            // Add data item
            graph.data_items.insert(event.data_item.clone(), event.classification);
        }
        
        // Remove duplicates from nodes
        let mut unique_nodes = Vec::new();
        let mut seen_ids = HashSet::new();
        
        for node in graph.nodes.drain(..) {
            if !seen_ids.contains(&node.id) {
                seen_ids.insert(node.id.clone());
                unique_nodes.push(node);
            }
        }
        
        graph.nodes = unique_nodes;
        
        Ok(())
    }
    
    /// Get recent data flow events
    pub fn get_recent_events(&self, limit: Option<usize>) -> Result<Vec<DataFlowEvent>> {
        let events = self.events.read().unwrap();
        let limit = limit.unwrap_or(100).min(events.len());
        
        // Get the last 'limit' events
        let result: Vec<DataFlowEvent> = events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect();
            
        Ok(result)
    }
    
    /// Get the current data flow graph
    pub fn get_data_flow_graph(&self) -> Result<DataFlowGraph> {
        Ok(self.graph.read().unwrap().clone())
    }
    
    /// Clear all data flow events
    pub fn clear_events(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Clear events
        self.events.write().unwrap().clear();
        
        // Clear graph
        *self.graph.write().unwrap() = DataFlowGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
            data_items: HashMap::new(),
        };
        
        // Initialize graph with nodes
        self.rebuild_graph()?;
        
        // Save empty events
        self.save_events()?;
        
        info!("Cleared all data flow events");
        record_counter("security.data_flow.events_cleared", 1.0, None);
        
        Ok(())
    }
    
    /// Search for data flow events
    pub fn search_events(
        &self,
        data_item: Option<&str>,
        classification: Option<DataClassification>,
        source: Option<&str>,
        destination: Option<&str>,
        operation: Option<&str>,
        start_time: Option<SystemTime>,
        end_time: Option<SystemTime>,
    ) -> Result<Vec<DataFlowEvent>> {
        let events = self.events.read().unwrap();
        
        let results: Vec<DataFlowEvent> = events.iter()
            .filter(|event| {
                // Match all provided filters
                (data_item.is_none() || data_item.unwrap() == event.data_item) &&
                (classification.is_none() || classification.unwrap() == event.classification) &&
                (source.is_none() || source.unwrap() == event.source) &&
                (destination.is_none() || destination.unwrap() == event.destination) &&
                (operation.is_none() || operation.unwrap() == event.operation) &&
                (start_time.is_none() || start_time.unwrap() <= event.timestamp) &&
                (end_time.is_none() || end_time.unwrap() >= event.timestamp)
            })
            .cloned()
            .collect();
            
        Ok(results)
    }
    
    /// Get data flow statistics
    pub fn get_statistics(&self) -> Result<DataFlowStatistics> {
        let events = self.events.read().unwrap();
        let graph = self.graph.read().unwrap();
        
        // Count by classification
        let mut count_by_classification = HashMap::new();
        count_by_classification.insert(DataClassification::Public, 0);
        count_by_classification.insert(DataClassification::Personal, 0);
        count_by_classification.insert(DataClassification::Sensitive, 0);
        count_by_classification.insert(DataClassification::Confidential, 0);
        
        for event in events.iter() {
            *count_by_classification.entry(event.classification).or_insert(0) += 1;
        }
        
        // Count by destination
        let mut count_by_destination = HashMap::new();
        for event in events.iter() {
            *count_by_destination.entry(event.destination.clone()).or_insert(0) += 1;
        }
        
        // Count by operation
        let mut count_by_operation = HashMap::new();
        for event in events.iter() {
            *count_by_operation.entry(event.operation.clone()).or_insert(0) += 1;
        }
        
        // Calculate stats
        let stats = DataFlowStatistics {
            total_events: events.len(),
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            data_item_count: graph.data_items.len(),
            classification_counts: count_by_classification,
            destination_counts: count_by_destination,
            operation_counts: count_by_operation,
            external_destinations: graph.nodes.iter()
                .filter(|node| !node.internal)
                .map(|node| node.id.clone())
                .collect(),
        };
        
        Ok(stats)
    }
}

/// Statistics about data flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowStatistics {
    /// Total number of events
    pub total_events: usize,
    
    /// Number of nodes in the graph
    pub node_count: usize,
    
    /// Number of edges in the graph
    pub edge_count: usize,
    
    /// Number of unique data items
    pub data_item_count: usize,
    
    /// Count of events by classification
    pub classification_counts: HashMap<DataClassification, usize>,
    
    /// Count of events by destination
    pub destination_counts: HashMap<String, usize>,
    
    /// Count of events by operation
    pub operation_counts: HashMap<String, usize>,
    
    /// List of external destinations
    pub external_destinations: Vec<String>,
}
