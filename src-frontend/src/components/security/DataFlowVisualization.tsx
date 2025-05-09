import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import * as d3 from 'd3';
import './DataFlowVisualization.css';

// Types
interface DataFlowNode {
  id: string;
  name: string;
  node_type: string;
  internal: boolean;
  location: string;
  metadata: Record<string, string>;
}

interface DataFlowGraph {
  nodes: DataFlowNode[];
  edges: [string, string, string[]][];
  data_items: Record<string, string>;
}

interface DataFlowEvent {
  id: string;
  operation: string;
  data_item: string;
  classification: string;
  source: string;
  destination: string;
  consent_obtained: boolean;
  timestamp: string;
  encrypted: boolean;
  metadata: Record<string, string>;
}

interface DataFlowStatistics {
  total_events: number;
  node_count: number;
  edge_count: number;
  data_item_count: number;
  classification_counts: Record<string, number>;
  destination_counts: Record<string, number>;
  operation_counts: Record<string, number>;
  external_destinations: string[];
}

// Classification colors
const classificationColors: Record<string, string> = {
  Public: '#4caf50',
  Personal: '#2196f3',
  Sensitive: '#ff9800',
  Confidential: '#f44336',
};

// Graph node types and colors
const nodeTypeColors: Record<string, string> = {
  application: '#8e24aa',
  storage: '#0288d1',
  memory: '#00796b',
  secure_storage: '#c62828',
  api: '#fb8c00',
  model: '#9c27b0',
  unknown: '#757575',
};

// Simulation node for d3
interface SimulationNode extends d3.SimulationNodeDatum {
  id: string;
  name: string;
  type: string;
  internal: boolean;
  location: string;
  radius: number;
  color: string;
}

// Simulation link for d3
interface SimulationLink extends d3.SimulationLinkDatum<SimulationNode> {
  source: string | SimulationNode;
  target: string | SimulationNode;
  classifications: string[];
  value: number;
}

const DataFlowVisualization: React.FC = () => {
  const [graph, setGraph] = useState<DataFlowGraph | null>(null);
  const [events, setEvents] = useState<DataFlowEvent[]>([]);
  const [statistics, setStatistics] = useState<DataFlowStatistics | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<[string, string] | null>(null);
  const [filterClassification, setFilterClassification] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState<boolean>(false);
  const [viewType, setViewType] = useState<'graph' | 'list' | 'stats'>('graph');
  
  const svgRef = useRef<SVGSVGElement>(null);

  // Load data on mount
  useEffect(() => {
    loadData();
    
    // Set up auto-refresh if enabled
    let intervalId: number;
    if (autoRefresh) {
      intervalId = window.setInterval(loadData, 5000);
    }
    
    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [autoRefresh, filterClassification]);

  // Function to load data from backend
  const loadData = async () => {
    try {
      setLoading(true);
      
      // Fetch data flow graph
      const graphData = await invoke<DataFlowGraph>('get_data_flow_graph');
      setGraph(graphData);
      
      // Fetch recent events
      const eventsData = await invoke<DataFlowEvent[]>('get_recent_data_flow_events', { limit: 100 });
      setEvents(eventsData);
      
      // Fetch statistics
      const statsData = await invoke<DataFlowStatistics>('get_data_flow_statistics');
      setStatistics(statsData);
      
      // Draw graph if in graph view
      if (viewType === 'graph' && graphData) {
        drawGraph(graphData);
      }
      
      setError(null);
    } catch (err) {
      console.error('Error loading data flow data:', err);
      setError(`Failed to load data flow information: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // Clear data flow events
  const clearEvents = async () => {
    try {
      await invoke('clear_data_flow_events');
      
      // Reload data
      loadData();
    } catch (err) {
      console.error('Error clearing data flow events:', err);
      setError(`Failed to clear data flow events: ${err}`);
    }
  };

  // Draw the force-directed graph using D3
  const drawGraph = (graphData: DataFlowGraph) => {
    if (!svgRef.current) return;
    
    // Clear previous graph
    d3.select(svgRef.current).selectAll('*').remove();
    
    const svg = d3.select(svgRef.current);
    const width = svgRef.current.clientWidth;
    const height = svgRef.current.clientHeight;
    
    // Create simulation nodes from graph data
    const nodes: SimulationNode[] = graphData.nodes.map(node => ({
      id: node.id,
      name: node.name,
      type: node.node_type,
      internal: node.internal,
      location: node.location,
      radius: node.internal ? 20 : 15,
      color: nodeTypeColors[node.node_type] || nodeTypeColors.unknown,
    }));
    
    // Create links from edges
    const links: SimulationLink[] = graphData.edges.map(edge => ({
      source: edge[0],
      target: edge[1],
      classifications: edge[2],
      value: edge[2].length,
    }));
    
    // Filter by classification if needed
    const filteredLinks = filterClassification 
      ? links.filter(link => link.classifications.includes(filterClassification))
      : links;
      
    // Create force simulation
    const simulation = d3.forceSimulation<SimulationNode, SimulationLink>(nodes)
      .force('link', d3.forceLink<SimulationNode, SimulationLink>(filteredLinks)
        .id(d => d.id)
        .distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(d => d.radius + 10));
    
    // Create arrow marker for links
    svg.append('defs').selectAll('marker')
      .data(['end'])
      .enter().append('marker')
      .attr('id', 'arrow')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 25)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#999');
    
    // Create links
    const link = svg.append('g')
      .selectAll('path')
      .data(filteredLinks)
      .enter().append('path')
      .attr('class', 'link')
      .attr('marker-end', 'url(#arrow)')
      .attr('stroke-width', d => Math.sqrt(d.value) * 2)
      .attr('stroke', d => {
        if (d.classifications.length === 1) {
          return classificationColors[d.classifications[0]] || '#999';
        } else {
          return '#999';
        }
      })
      .on('click', (event, d) => {
        setSelectedEdge([d.source.id || d.source as string, d.target.id || d.target as string]);
        setSelectedNode(null);
      });
    
    // Create nodes
    const node = svg.append('g')
      .selectAll('circle')
      .data(nodes)
      .enter().append('circle')
      .attr('class', 'node')
      .attr('r', d => d.radius)
      .attr('fill', d => d.color)
      .attr('stroke', d => d.internal ? '#fff' : '#333')
      .attr('stroke-width', 2)
      .on('click', (event, d) => {
        setSelectedNode(d.id);
        setSelectedEdge(null);
      })
      .call(d3.drag<SVGCircleElement, SimulationNode>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        }) as any); // Type cast to any due to d3 typing issues
    
    // Create node labels
    const labels = svg.append('g')
      .selectAll('text')
      .data(nodes)
      .enter().append('text')
      .attr('class', 'node-label')
      .text(d => d.name)
      .attr('text-anchor', 'middle')
      .attr('dy', '0.35em')
      .attr('fill', d => d.internal ? '#fff' : '#333')
      .attr('pointer-events', 'none');
    
    // Update positions on simulation tick
    simulation.on('tick', () => {
      link.attr('d', d => {
        const source = d.source as SimulationNode;
        const target = d.target as SimulationNode;
        
        return `M${source.x},${source.y}L${target.x},${target.y}`;
      });
      
      node.attr('cx', d => Math.max(d.radius, Math.min(width - d.radius, d.x || 0)))
          .attr('cy', d => Math.max(d.radius, Math.min(height - d.radius, d.y || 0)));
          
      labels.attr('x', d => Math.max(d.radius, Math.min(width - d.radius, d.x || 0)))
            .attr('y', d => Math.max(d.radius + 20, Math.min(height - d.radius, (d.y || 0) + 30)));
    });
  };

  // Format timestamp for display
  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  // Render selected node details
  const renderNodeDetails = () => {
    if (!selectedNode || !graph) return null;
    
    const node = graph.nodes.find(n => n.id === selectedNode);
    if (!node) return null;
    
    // Find all edges connected to this node
    const connectedEdges = graph.edges.filter(
      edge => edge[0] === selectedNode || edge[1] === selectedNode
    );
    
    return (
      <div className="details-panel">
        <h3>Node Details: {node.name}</h3>
        
        <div className="details-content">
          <div className="detail-item">
            <span className="detail-label">ID:</span>
            <span className="detail-value">{node.id}</span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">Type:</span>
            <span className="detail-value">{node.node_type}</span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">Location:</span>
            <span className="detail-value">{node.location}</span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">Internal:</span>
            <span className="detail-value">{node.internal ? 'Yes' : 'No'}</span>
          </div>
          
          {Object.keys(node.metadata).length > 0 && (
            <>
              <div className="detail-section">Metadata</div>
              {Object.entries(node.metadata).map(([key, value]) => (
                <div key={key} className="detail-item">
                  <span className="detail-label">{key}:</span>
                  <span className="detail-value">{value}</span>
                </div>
              ))}
            </>
          )}
          
          <div className="detail-section">Connections</div>
          {connectedEdges.length === 0 ? (
            <div className="detail-empty">No connections</div>
          ) : (
            <>
              <div className="detail-subtitle">Incoming:</div>
              {connectedEdges.filter(edge => edge[1] === selectedNode).length === 0 ? (
                <div className="detail-empty">None</div>
              ) : (
                connectedEdges
                  .filter(edge => edge[1] === selectedNode)
                  .map(edge => (
                    <div key={`${edge[0]}-${edge[1]}`} className="detail-item">
                      <span className="detail-label">From:</span>
                      <span className="detail-value">{graph.nodes.find(n => n.id === edge[0])?.name || edge[0]}</span>
                      <div className="detail-classifications">
                        {edge[2].map(cls => (
                          <span 
                            key={cls} 
                            className="classification-tag" 
                            style={{ backgroundColor: classificationColors[cls] || '#999' }}
                          >
                            {cls}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))
              )}
              
              <div className="detail-subtitle">Outgoing:</div>
              {connectedEdges.filter(edge => edge[0] === selectedNode).length === 0 ? (
                <div className="detail-empty">None</div>
              ) : (
                connectedEdges
                  .filter(edge => edge[0] === selectedNode)
                  .map(edge => (
                    <div key={`${edge[0]}-${edge[1]}`} className="detail-item">
                      <span className="detail-label">To:</span>
                      <span className="detail-value">{graph.nodes.find(n => n.id === edge[1])?.name || edge[1]}</span>
                      <div className="detail-classifications">
                        {edge[2].map(cls => (
                          <span 
                            key={cls} 
                            className="classification-tag" 
                            style={{ backgroundColor: classificationColors[cls] || '#999' }}
                          >
                            {cls}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))
              )}
            </>
          )}
        </div>
      </div>
    );
  };

  // Render selected edge details
  const renderEdgeDetails = () => {
    if (!selectedEdge || !graph) return null;
    
    const [sourceId, targetId] = selectedEdge;
    
    // Find the edge in the graph
    const edge = graph.edges.find(e => e[0] === sourceId && e[1] === targetId);
    if (!edge) return null;
    
    const sourceNode = graph.nodes.find(n => n.id === sourceId);
    const targetNode = graph.nodes.find(n => n.id === targetId);
    
    // Find events related to this edge
    const relatedEvents = events.filter(
      event => event.source === sourceId && event.destination === targetId
    );
    
    return (
      <div className="details-panel">
        <h3>Data Flow Details</h3>
        
        <div className="details-content">
          <div className="detail-item">
            <span className="detail-label">From:</span>
            <span className="detail-value">{sourceNode?.name || sourceId}</span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">To:</span>
            <span className="detail-value">{targetNode?.name || targetId}</span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">Data Classifications:</span>
            <div className="detail-classifications">
              {edge[2].map(cls => (
                <span 
                  key={cls} 
                  className="classification-tag" 
                  style={{ backgroundColor: classificationColors[cls] || '#999' }}
                >
                  {cls}
                </span>
              ))}
            </div>
          </div>
          
          <div className="detail-section">Recent Events</div>
          {relatedEvents.length === 0 ? (
            <div className="detail-empty">No events found</div>
          ) : (
            <div className="events-list">
              {relatedEvents.slice(0, 5).map(event => (
                <div key={event.id} className="event-item">
                  <div className="event-header">
                    <span 
                      className="event-classification"
                      style={{ backgroundColor: classificationColors[event.classification] || '#999' }}
                    >
                      {event.classification}
                    </span>
                    <span className="event-timestamp">{formatTimestamp(event.timestamp)}</span>
                  </div>
                  <div className="event-details">
                    <div className="event-operation">{event.operation}</div>
                    <div className="event-data-item">{event.data_item}</div>
                    <div className="event-encryption">
                      {event.encrypted ? (
                        <span className="encryption-enabled">🔒 Encrypted</span>
                      ) : (
                        <span className="encryption-disabled">🔓 Unencrypted</span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
              
              {relatedEvents.length > 5 && (
                <div className="more-events">
                  + {relatedEvents.length - 5} more events
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    );
  };

  // Render statistics view
  const renderStatistics = () => {
    if (!statistics) return null;
    
    return (
      <div className="statistics-view">
        <div className="stats-row">
          <div className="stat-card">
            <div className="stat-value">{statistics.total_events}</div>
            <div className="stat-label">Total Events</div>
          </div>
          
          <div className="stat-card">
            <div className="stat-value">{statistics.node_count}</div>
            <div className="stat-label">System Components</div>
          </div>
          
          <div className="stat-card">
            <div className="stat-value">{statistics.edge_count}</div>
            <div className="stat-label">Data Flows</div>
          </div>
          
          <div className="stat-card">
            <div className="stat-value">{statistics.data_item_count}</div>
            <div className="stat-label">Data Items</div>
          </div>
        </div>
        
        <div className="stats-grid">
          <div className="stat-chart-card">
            <h3>Data Classification</h3>
            <div className="classification-chart">
              {Object.entries(statistics.classification_counts).map(([classification, count]) => (
                <div key={classification} className="chart-bar-container">
                  <div className="chart-label">{classification}</div>
                  <div className="chart-bar">
                    <div 
                      className="chart-bar-fill" 
                      style={{
                        width: `${(count / statistics.total_events) * 100}%`,
                        backgroundColor: classificationColors[classification] || '#999'
                      }}
                    ></div>
                  </div>
                  <div className="chart-value">{count}</div>
                </div>
              ))}
            </div>
          </div>
          
          <div className="stat-chart-card">
            <h3>Top Destinations</h3>
            <div className="destination-chart">
              {Object.entries(statistics.destination_counts)
                .sort((a, b) => b[1] - a[1])
                .slice(0, 5)
                .map(([destination, count]) => (
                  <div key={destination} className="chart-bar-container">
                    <div className="chart-label">
                      {graph?.nodes.find(n => n.id === destination)?.name || destination}
                    </div>
                    <div className="chart-bar">
                      <div 
                        className="chart-bar-fill" 
                        style={{
                          width: `${(count / statistics.total_events) * 100}%`,
                          backgroundColor: '#2196f3'
                        }}
                      ></div>
                    </div>
                    <div className="chart-value">{count}</div>
                  </div>
                ))}
            </div>
          </div>
          
          <div className="stat-chart-card">
            <h3>Top Operations</h3>
            <div className="operation-chart">
              {Object.entries(statistics.operation_counts)
                .sort((a, b) => b[1] - a[1])
                .slice(0, 5)
                .map(([operation, count]) => (
                  <div key={operation} className="chart-bar-container">
                    <div className="chart-label">{operation}</div>
                    <div className="chart-bar">
                      <div 
                        className="chart-bar-fill" 
                        style={{
                          width: `${(count / statistics.total_events) * 100}%`,
                          backgroundColor: '#ff9800'
                        }}
                      ></div>
                    </div>
                    <div className="chart-value">{count}</div>
                  </div>
                ))}
            </div>
          </div>
          
          <div className="stat-chart-card">
            <h3>External Data Destinations</h3>
            {statistics.external_destinations.length === 0 ? (
              <div className="no-external">No external data destinations found</div>
            ) : (
              <div className="external-destinations">
                {statistics.external_destinations.map(destination => {
                  const node = graph?.nodes.find(n => n.id === destination);
                  return (
                    <div key={destination} className="external-destination">
                      <div className="external-name">{node?.name || destination}</div>
                      <div className="external-type">{node?.node_type || 'unknown'}</div>
                      <div className="external-location">{node?.location || 'unknown'}</div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

  // Render events list view
  const renderEventsList = () => {
    return (
      <div className="events-view">
        <div className="events-list-header">
          <h3>Recent Data Flow Events</h3>
          <div className="events-actions">
            <button 
              className="clear-events-button" 
              onClick={clearEvents}
              disabled={events.length === 0}
            >
              Clear Events
            </button>
          </div>
        </div>
        
        {events.length === 0 ? (
          <div className="no-events">No data flow events recorded</div>
        ) : (
          <div className="events-table">
            <div className="events-table-header">
              <div className="event-col timestamp">Timestamp</div>
              <div className="event-col operation">Operation</div>
              <div className="event-col data-item">Data Item</div>
              <div className="event-col classification">Classification</div>
              <div className="event-col source">Source</div>
              <div className="event-col destination">Destination</div>
              <div className="event-col encrypted">Encrypted</div>
            </div>
            
            <div className="events-table-body">
              {events.map(event => (
                <div key={event.id} className="events-table-row">
                  <div className="event-col timestamp">{formatTimestamp(event.timestamp)}</div>
                  <div className="event-col operation">{event.operation}</div>
                  <div className="event-col data-item">{event.data_item}</div>
                  <div className="event-col classification">
                    <span 
                      className="classification-tag" 
                      style={{ backgroundColor: classificationColors[event.classification] || '#999' }}
                    >
                      {event.classification}
                    </span>
                  </div>
                  <div className="event-col source">
                    {graph?.nodes.find(n => n.id === event.source)?.name || event.source}
                  </div>
                  <div className="event-col destination">
                    {graph?.nodes.find(n => n.id === event.destination)?.name || event.destination}
                  </div>
                  <div className="event-col encrypted">
                    {event.encrypted ? (
                      <span className="encryption-enabled">🔒</span>
                    ) : (
                      <span className="encryption-disabled">🔓</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="data-flow-visualization">
      <div className="visualization-header">
        <h2>Data Flow Visualization</h2>
        
        <div className="view-controls">
          <button 
            className={viewType === 'graph' ? 'active' : ''}
            onClick={() => setViewType('graph')}
          >
            Graph View
          </button>
          <button 
            className={viewType === 'list' ? 'active' : ''}
            onClick={() => setViewType('list')}
          >
            Events List
          </button>
          <button 
            className={viewType === 'stats' ? 'active' : ''}
            onClick={() => setViewType('stats')}
          >
            Statistics
          </button>
        </div>
        
        <div className="visualization-controls">
          {viewType === 'graph' && (
            <div className="filter-control">
              <label>Filter by Classification:</label>
              <select 
                value={filterClassification || ''} 
                onChange={e => setFilterClassification(e.target.value || null)}
              >
                <option value="">All Classifications</option>
                <option value="Public">Public</option>
                <option value="Personal">Personal</option>
                <option value="Sensitive">Sensitive</option>
                <option value="Confidential">Confidential</option>
              </select>
            </div>
          )}
          
          <div className="refresh-control">
            <label className="auto-refresh-label">
              <input 
                type="checkbox" 
                checked={autoRefresh} 
                onChange={e => setAutoRefresh(e.target.checked)} 
              />
              Auto-refresh
            </label>
            <button 
              className="refresh-button" 
              onClick={loadData}
              disabled={loading}
            >
              {loading ? 'Loading...' : 'Refresh'}
            </button>
          </div>
        </div>
      </div>
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      <div className="visualization-content">
        {viewType === 'graph' && (
          <div className="graph-view">
            <div className="graph-container">
              <svg ref={svgRef} className="graph-svg"></svg>
              
              {loading && (
                <div className="loading-overlay">
                  <div className="loading-spinner"></div>
                  <div className="loading-text">Loading data flow graph...</div>
                </div>
              )}
              
              {!loading && graph && graph.nodes.length === 0 && (
                <div className="no-data-overlay">
                  <div className="no-data-message">
                    No data flow information available.
                    <br />
                    Start using the application to see data flows.
                  </div>
                </div>
              )}
            </div>
            
            <div className="details-container">
              {selectedNode && renderNodeDetails()}
              {selectedEdge && renderEdgeDetails()}
              {!selectedNode && !selectedEdge && (
                <div className="empty-details">
                  <p>Select a node or edge to see details</p>
                </div>
              )}
            </div>
            
            <div className="legend">
              <div className="legend-section">
                <div className="legend-title">Node Types</div>
                {Object.entries(nodeTypeColors).map(([type, color]) => (
                  <div key={type} className="legend-item">
                    <div 
                      className="legend-color" 
                      style={{ backgroundColor: color }}
                    ></div>
                    <div className="legend-label">{type}</div>
                  </div>
                ))}
              </div>
              
              <div className="legend-section">
                <div className="legend-title">Data Classifications</div>
                {Object.entries(classificationColors).map(([type, color]) => (
                  <div key={type} className="legend-item">
                    <div 
                      className="legend-color" 
                      style={{ backgroundColor: color }}
                    ></div>
                    <div className="legend-label">{type}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
        
        {viewType === 'list' && renderEventsList()}
        
        {viewType === 'stats' && renderStatistics()}
      </div>
    </div>
  );
};

export default DataFlowVisualization;
