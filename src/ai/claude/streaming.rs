use super::api::ClaudeDeltaResponse;
use crate::models::messages::{ContentType, Message, MessageError, MessageRole};
use crate::utils::events::{events, get_event_system};
use futures_util::StreamExt;
use log::{debug, error, warn};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::Stream;

/// Claude streaming handler
#[derive(Clone)]
pub struct ClaudeStreamHandler {
    /// Stream ID
    stream_id: String,
    
    /// Message sender
    tx: mpsc::Sender<Result<Message, MessageError>>,
    
    /// Original message ID
    original_message_id: String,
    
    /// Cancel channel
    cancel: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    
    /// Accumulated text
    accumulated_text: Arc<Mutex<String>>,
}

impl ClaudeStreamHandler {
    /// Create a new Claude streaming handler
    pub fn new(
        stream_id: String,
        tx: mpsc::Sender<Result<Message, MessageError>>,
        original_message_id: String,
    ) -> Self {
        Self {
            stream_id,
            tx,
            original_message_id,
            cancel: Arc::new(Mutex::new(None)),
            accumulated_text: Arc::new(Mutex::new(String::new())),
        }
    }
    
    /// Handle streaming response
    pub async fn handle_stream<S, E>(
        &self,
        stream: &mut S,
        message_id: &str,
    ) where
        S: Stream<Item = Result<ClaudeDeltaResponse, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Create cancel channel
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        
        {
            let mut cancel_guard = self.cancel.lock().unwrap();
            *cancel_guard = Some(cancel_tx);
        }
        
        let mut final_response = None;
        
        // Process stream
        loop {
            tokio::select! {
                // Check for cancellation
                _ = &mut cancel_rx => {
                    debug!("Stream cancelled: {}", self.stream_id);
                    break;
                }
                
                // Process next chunk
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(delta)) => {
                            // Process delta
                            self.process_delta(delta, message_id).await;
                        }
                        Some(Err(e)) => {
                            // Send error
                            let error_msg = format!("Stream error: {}", e);
                            error!("{}", error_msg);
                            
                            let _ = self.tx.send(Err(MessageError::NetworkError(error_msg))).await;
                            break;
                        }
                        None => {
                            // End of stream
                            debug!("End of stream: {}", self.stream_id);
                            
                            // Send final complete message
                            let text = {
                                let text_guard = self.accumulated_text.lock().unwrap();
                                text_guard.clone()
                            };
                            
                            let final_message = Message {
                                id: message_id.to_string(),
                                role: MessageRole::Assistant,
                                content: crate::models::messages::MessageContent {
                                    parts: vec![ContentType::Text { text }],
                                },
                                metadata: None,
                                created_at: SystemTime::now(),
                            };
                            
                            let _ = self.tx.send(Ok(final_message)).await;
                            break;
                        }
                    }
                }
            }
        }
    }
    
    /// Process delta response
    async fn process_delta(&self, delta: ClaudeDeltaResponse, message_id: &str) {
        match delta.response_type.as_str() {
            "message_delta" => {
                // Check if delta contains content
                if let Some(delta_content) = delta.delta.get("content") {
                    // Only process text content for now
                    if let Some(content) = delta_content.as_array() {
                        for part in content {
                            if let Some(part_type) = part.get("type") {
                                if part_type.as_str() == Some("text") {
                                    if let Some(text) = part.get("text") {
                                        if let Some(text_str) = text.as_str() {
                                            // Append to accumulated text
                                            {
                                                let mut text_guard = self.accumulated_text.lock().unwrap();
                                                text_guard.push_str(text_str);
                                            }
                                            
                                            // Send partial message
                                            let text = {
                                                let text_guard = self.accumulated_text.lock().unwrap();
                                                text_guard.clone()
                                            };
                                            
                                            let partial_message = Message {
                                                id: message_id.to_string(),
                                                role: MessageRole::Assistant,
                                                content: crate::models::messages::MessageContent {
                                                    parts: vec![ContentType::Text { text }],
                                                },
                                                metadata: None,
                                                created_at: SystemTime::now(),
                                            };
                                            
                                            let _ = self.tx.send(Ok(partial_message)).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "message_stop" => {
                // Final message, check if we have usage info
                if let Some(usage) = delta.usage {
                    // In a real implementation, we would include usage in metadata
                    debug!("Stream stopped with usage: {:?}", usage);
                }
            }
            _ => {
                // Ignore other message types
                debug!("Unknown delta type: {}", delta.response_type);
            }
        }
    }
    
    /// Cancel the stream
    pub async fn cancel(&self) {
        // Send cancel signal
        let mut cancel_tx = None;
        
        {
            let mut cancel_guard = self.cancel.lock().unwrap();
            cancel_tx = cancel_guard.take();
        }
        
        if let Some(tx) = cancel_tx {
            let _ = tx.send(());
        }
    }
}
