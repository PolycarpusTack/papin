use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::error::{McpError, McpResult};

/// WebSocket connection status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected to any server
    Disconnected,
    
    /// Currently establishing connection
    Connecting,
    
    /// Connected and authenticated
    Connected,
    
    /// Connection established but authentication failed
    AuthFailed,
    
    /// Connection dropped or experiencing issues
    Error(String),
}

/// WebSocket connection configuration
#[derive(Clone, Debug)]
pub struct WebSocketConfig {
    /// Server URL
    pub url: String,
    
    /// Optional headers
    pub headers: Vec<(String, String)>,
    
    /// Timeout for connection
    pub connect_timeout: Duration,
    
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    
    /// Reconnection attempts
    pub max_reconnect_attempts: u32,
    
    /// Reconnection delay
    pub reconnect_delay: Duration,
}

/// WebSocket client
pub struct WebSocketClient {
    /// Configuration
    config: WebSocketConfig,
    
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Message sender channel
    sender: Arc<Mutex<Option<mpsc::Sender<WsMessage>>>>,
    
    /// Message receiver channel
    receiver: Arc<Mutex<mpsc::Receiver<WsMessage>>>,
    
    /// Message sender for the connection task
    connection_sender: mpsc::Sender<WsMessage>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: WebSocketConfig) -> Self {
        // Create channels for message passing
        let (connection_sender, connection_receiver) = mpsc::channel::<WsMessage>(32);
        let (message_sender, message_receiver) = mpsc::channel::<WsMessage>(32);
        
        let client = Self {
            config,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            sender: Arc::new(Mutex::new(None)),
            receiver: Arc::new(Mutex::new(message_receiver)),
            connection_sender,
        };
        
        // Spawn task to handle messages
        let status_clone = client.status.clone();
        let sender_clone = client.sender.clone();
        let config_clone = client.config.clone();
        
        tokio::spawn(async move {
            Self::connection_task(
                status_clone,
                sender_clone,
                message_sender,
                connection_receiver,
                config_clone,
            )
            .await;
        });
        
        client
    }
    
    /// Get current connection status
    pub fn status(&self) -> ConnectionStatus {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.status.read().await.clone()
            })
        })
    }
    
    /// Connect to the server
    pub async fn connect(&self) -> McpResult<()> {
        // Update status
        *self.status.write().await = ConnectionStatus::Connecting;
        
        // Send connect message to the connection task
        self.connection_sender
            .send(WsMessage::Text("CONNECT".to_string()))
            .await
            .map_err(|e| McpError::Connection(format!("Failed to send connect message: {}", e)))?;
            
        // Wait for connection to be established
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let status = self.status.read().await.clone();
            
            match status {
                ConnectionStatus::Connected => return Ok(()),
                ConnectionStatus::AuthFailed => {
                    return Err(McpError::Authentication("Authentication failed".to_string()));
                }
                ConnectionStatus::Error(e) => {
                    return Err(McpError::Connection(e));
                }
                _ => continue,
            }
        }
        
        Err(McpError::Connection("Connection timed out".to_string()))
    }
    
    /// Disconnect from the server
    pub async fn disconnect(&self) -> McpResult<()> {
        // Send disconnect message to the connection task
        self.connection_sender
            .send(WsMessage::Text("DISCONNECT".to_string()))
            .await
            .map_err(|e| McpError::Connection(format!("Failed to send disconnect message: {}", e)))?;
            
        // Update status
        *self.status.write().await = ConnectionStatus::Disconnected;
        
        Ok(())
    }
    
    /// Send a message to the server
    pub async fn send(&self, message: WsMessage) -> McpResult<()> {
        // Check if connected
        let status = self.status.read().await.clone();
        if status != ConnectionStatus::Connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }
        
        // Get sender
        let sender = self.sender.lock().await.clone();
        if let Some(sender) = sender {
            // Send message
            sender
                .send(message)
                .await
                .map_err(|e| McpError::Connection(format!("Failed to send message: {}", e)))?;
                
            Ok(())
        } else {
            Err(McpError::Connection("No sender available".to_string()))
        }
    }
    
    /// Receive a message from the server with timeout
    pub async fn receive(&self, timeout_duration: Duration) -> McpResult<WsMessage> {
        // Check if connected
        let status = self.status.read().await.clone();
        if status != ConnectionStatus::Connected {
            return Err(McpError::Connection("Not connected".to_string()));
        }
        
        // Get receiver
        let mut receiver = self.receiver.lock().await;
        
        // Wait for message with timeout
        match timeout(timeout_duration, receiver.recv()).await {
            Ok(Some(message)) => Ok(message),
            Ok(None) => Err(McpError::Connection("Channel closed".to_string())),
            Err(_) => Err(McpError::Connection("Receive timed out".to_string())),
        }
    }
    
    /// Connection task
    async fn connection_task(
        status: Arc<RwLock<ConnectionStatus>>,
        sender: Arc<Mutex<Option<mpsc::Sender<WsMessage>>>>,
        message_sender: mpsc::Sender<WsMessage>,
        mut control_receiver: mpsc::Receiver<WsMessage>,
        config: WebSocketConfig,
    ) {
        // Websocket connection
        let mut ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>> = None;
        let mut reconnect_attempts = 0;
        
        loop {
            // Check for control messages
            if let Ok(msg) = control_receiver.try_recv() {
                if let WsMessage::Text(text) = &msg {
                    match text.as_str() {
                        "CONNECT" => {
                            // Try to connect
                            match Self::do_connect(&config).await {
                                Ok(stream) => {
                                    ws_stream = Some(stream);
                                    *status.write().await = ConnectionStatus::Connected;
                                    reconnect_attempts = 0;
                                    
                                    // Create message channels
                                    let (tx, mut rx) = mpsc::channel::<WsMessage>(32);
                                    *sender.lock().await = Some(tx);
                                    
                                    // Spawn task to handle websocket messages
                                    let stream_clone = ws_stream.as_mut().unwrap().get_mut();
                                    let message_sender_clone = message_sender.clone();
                                    let status_clone = status.clone();
                                    
                                    tokio::spawn(async move {
                                        Self::handle_websocket(
                                            stream_clone,
                                            message_sender_clone,
                                            &mut rx,
                                            status_clone,
                                        )
                                        .await;
                                    });
                                }
                                Err(e) => {
                                    *status.write().await = ConnectionStatus::Error(e.to_string());
                                }
                            }
                        }
                        "DISCONNECT" => {
                            // Close connection
                            if let Some(stream) = &mut ws_stream {
                                let _ = stream.close(None).await;
                            }
                            ws_stream = None;
                            *status.write().await = ConnectionStatus::Disconnected;
                            *sender.lock().await = None;
                        }
                        _ => {
                            // Ignore other messages
                        }
                    }
                }
            }
            
            // Sleep a bit to avoid busy loop
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Connect to the WebSocket server
    async fn do_connect(
        config: &WebSocketConfig,
    ) -> McpResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        // Parse URL
        let url = Url::parse(&config.url)
            .map_err(|e| McpError::Connection(format!("Invalid URL: {}", e)))?;
            
        // Connect with timeout
        let result = timeout(config.connect_timeout, connect_async(url)).await;
        
        match result {
            Ok(Ok((ws_stream, _))) => Ok(ws_stream),
            Ok(Err(e)) => Err(McpError::Connection(format!("WebSocket connect error: {}", e))),
            Err(_) => Err(McpError::Connection("Connection timed out".to_string())),
        }
    }
    
    /// Handle WebSocket messages
    async fn handle_websocket(
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        message_sender: mpsc::Sender<WsMessage>,
        message_receiver: &mut mpsc::Receiver<WsMessage>,
        status: Arc<RwLock<ConnectionStatus>>,
    ) {
        loop {
            tokio::select! {
                // Handle incoming messages from the WebSocket
                Some(msg) = ws_stream.next() => {
                    match msg {
                        Ok(msg) => {
                            if msg.is_close() {
                                debug!("WebSocket closed by server");
                                break;
                            }
                            
                            // Forward message to receiver
                            if let Err(e) = message_sender.send(msg).await {
                                error!("Failed to forward message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            *status.write().await = ConnectionStatus::Error(format!("WebSocket error: {}", e));
                            break;
                        }
                    }
                }
                
                // Handle outgoing messages to the WebSocket
                Some(msg) = message_receiver.recv() => {
                    if let Err(e) = ws_stream.send(msg).await {
                        error!("Failed to send message: {}", e);
                        *status.write().await = ConnectionStatus::Error(format!("Send error: {}", e));
                        break;
                    }
                }
                
                // Stop if all senders are dropped
                else => {
                    debug!("All message senders dropped");
                    break;
                }
            }
        }
        
        // Update status on exit
        let current_status = status.read().await.clone();
        if current_status == ConnectionStatus::Connected {
            *status.write().await = ConnectionStatus::Disconnected;
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "wss://api.anthropic.com/v1/messages".to_string(),
            headers: Vec::new(),
            connect_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(30),
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(2),
        }
    }
}
