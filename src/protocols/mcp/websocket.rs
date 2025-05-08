use crate::protocols::mcp::error::McpError;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex as AsyncMutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};

/// WebSocket client for MCP
pub struct WebSocketClient {
    /// The WebSocket URL
    url: String,
    
    /// Sender half of the channel to send messages to WebSocket
    tx: Arc<Mutex<Option<Sender<String>>>>,
    
    /// WebSocket connection
    ws_stream: Arc<AsyncMutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    
    /// Internal message receiver
    rx: Arc<AsyncMutex<Option<Receiver<String>>>>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            tx: Arc::new(Mutex::new(None)),
            ws_stream: Arc::new(AsyncMutex::new(None)),
            rx: Arc::new(AsyncMutex::new(None)),
        }
    }
    
    /// Connect to the WebSocket server
    pub async fn connect(&self) -> Result<(), McpError> {
        let url = url::Url::parse(&self.url)
            .map_err(|e| McpError::ConnectionError(format!("Invalid URL: {}", e)))?;
        
        // Connect to the WebSocket server
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| McpError::ConnectionError(format!("Failed to connect: {}", e)))?;
        
        // Create a channel for sending messages
        let (tx, rx) = mpsc::channel(32);
        
        {
            let mut tx_guard = self.tx.lock().unwrap();
            *tx_guard = Some(tx);
        }
        
        {
            let mut rx_guard = self.rx.lock().await;
            *rx_guard = Some(rx);
        }
        
        // Store the WebSocket stream
        {
            let mut ws_guard = self.ws_stream.lock().await;
            *ws_guard = Some(ws_stream);
        }
        
        // Spawn message sender task
        let ws_clone = self.ws_stream.clone();
        let rx_clone = self.rx.clone();
        
        tokio::spawn(async move {
            let mut rx_guard = rx_clone.lock().await;
            if let Some(mut rx) = rx_guard.take() {
                while let Some(message) = rx.recv().await {
                    let mut ws_guard = ws_clone.lock().await;
                    if let Some(ws) = ws_guard.as_mut() {
                        if let Err(e) = ws.send(WsMessage::Text(message)).await {
                            error!("Failed to send WebSocket message: {}", e);
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Disconnect from the WebSocket server
    pub async fn disconnect(&self) -> Result<(), McpError> {
        let mut ws_guard = self.ws_stream.lock().await;
        if let Some(mut ws) = ws_guard.take() {
            // Send close message
            if let Err(e) = ws.close(None).await {
                warn!("Error while closing WebSocket: {}", e);
            }
        }
        
        // Drop the sender to terminate the message sender task
        let mut tx_guard = self.tx.lock().unwrap();
        *tx_guard = None;
        
        Ok(())
    }
    
    /// Send a message to the WebSocket server
    pub async fn send(&self, message: String) -> Result<(), McpError> {
        let tx_guard = self.tx.lock().unwrap();
        if let Some(tx) = tx_guard.as_ref() {
            tx.send(message)
                .await
                .map_err(|e| McpError::WebSocketError(format!("Failed to send message: {}", e)))?;
            Ok(())
        } else {
            Err(McpError::ConnectionClosed)
        }
    }
    
    /// Receive a message from the WebSocket server
    pub async fn receive(&self) -> Result<String, McpError> {
        let mut ws_guard = self.ws_stream.lock().await;
        if let Some(ws) = ws_guard.as_mut() {
            match ws.next().await {
                Some(Ok(WsMessage::Text(text))) => Ok(text),
                Some(Ok(WsMessage::Binary(bin))) => {
                    String::from_utf8(bin)
                        .map_err(|e| McpError::ProtocolError(format!("Invalid UTF-8: {}", e)))
                }
                Some(Ok(WsMessage::Ping(_))) => {
                    // Respond with pong - many WebSocket libraries handle this automatically
                    Err(McpError::ProtocolError("Received ping message".to_string()))
                }
                Some(Ok(WsMessage::Pong(_))) => {
                    // Respond to pong - usually a response to our ping
                    Err(McpError::ProtocolError("Received pong message".to_string()))
                }
                Some(Ok(WsMessage::Close(_))) => Err(McpError::ConnectionClosed),
                Some(Err(e)) => Err(McpError::WebSocketError(e.to_string())),
                None => Err(McpError::ConnectionClosed),
            }
        } else {
            Err(McpError::ConnectionClosed)
        }
    }
    
    /// Check if the WebSocket is connected
    pub async fn is_connected(&self) -> bool {
        let ws_guard = self.ws_stream.lock().await;
        ws_guard.is_some()
    }
}
