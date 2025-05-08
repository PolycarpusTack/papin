use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Event type ID
pub type EventType = &'static str;

/// Event payload
pub type EventPayload = serde_json::Value;

/// Event handler function
pub type EventHandler = Box<dyn Fn(EventPayload) -> () + Send + Sync>;

/// Event system for handling application events
pub struct EventSystem {
    /// Registered event handlers
    handlers: Arc<Mutex<HashMap<EventType, Vec<EventHandler>>>>,
    
    /// Event sender
    tx: mpsc::UnboundedSender<(EventType, EventPayload)>,
    
    /// Event receiver
    rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<(EventType, EventPayload)>>>>,
}

impl EventSystem {
    /// Create a new event system
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
        }
    }
    
    /// Start the event processing loop
    pub fn start(&self) {
        let handlers = self.handlers.clone();
        let mut rx_guard = self.rx.lock().unwrap();
        
        if let Some(rx) = rx_guard.take() {
            tokio::spawn(async move {
                Self::process_events(handlers, rx).await;
            });
        }
    }
    
    /// Register an event handler
    pub fn on<F>(&self, event_type: EventType, handler: F)
    where
        F: Fn(EventPayload) -> () + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        let event_handlers = handlers.entry(event_type).or_insert_with(Vec::new);
        event_handlers.push(Box::new(handler));
    }
    
    /// Emit an event
    pub fn emit(&self, event_type: EventType, payload: EventPayload) {
        if let Err(e) = self.tx.send((event_type, payload)) {
            warn!("Failed to emit event {}: {}", event_type, e);
        }
    }
    
    /// Process events in the background
    async fn process_events(
        handlers: Arc<Mutex<HashMap<EventType, Vec<EventHandler>>>>,
        mut rx: mpsc::UnboundedReceiver<(EventType, EventPayload)>,
    ) {
        while let Some((event_type, payload)) = rx.recv().await {
            // Get handlers for this event type
            let handlers_clone = handlers.clone();
            let handlers_guard = handlers_clone.lock().unwrap();
            
            if let Some(event_handlers) = handlers_guard.get(event_type) {
                // Clone handlers and payload to avoid holding the lock during handler execution
                let handlers_copy = event_handlers.clone();
                let payload_copy = payload.clone();
                
                // Drop the guard before executing handlers
                drop(handlers_guard);
                
                // Execute handlers in a separate task
                tokio::spawn(async move {
                    for handler in handlers_copy {
                        handler(payload_copy.clone());
                    }
                });
            }
        }
    }
}

/// Global event system instance
static EVENT_SYSTEM: once_cell::sync::OnceCell<EventSystem> = once_cell::sync::OnceCell::new();

/// Get the global event system instance
pub fn get_event_system() -> &'static EventSystem {
    EVENT_SYSTEM.get_or_init(|| {
        let system = EventSystem::new();
        system.start();
        system
    })
}

/// Event types
pub mod events {
    /// Connection status changed
    pub const CONNECTION_STATUS_CHANGED: &str = "connection_status_changed";
    
    /// Message received
    pub const MESSAGE_RECEIVED: &str = "message_received";
    
    /// Message sent
    pub const MESSAGE_SENT: &str = "message_sent";
    
    /// Message status changed
    pub const MESSAGE_STATUS_CHANGED: &str = "message_status_changed";
    
    /// Conversation created
    pub const CONVERSATION_CREATED: &str = "conversation_created";
    
    /// Conversation deleted
    pub const CONVERSATION_DELETED: &str = "conversation_deleted";
    
    /// Authentication status changed
    pub const AUTH_STATUS_CHANGED: &str = "auth_status_changed";
}
