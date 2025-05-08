use std::time::Duration;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use anyhow::Result;

/// Terminal events
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal tick
    Tick,
    /// Key press
    Key(KeyEvent),
    /// Mouse click/scroll
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
}

/// Event handler
pub struct EventHandler {
    /// Polling interval
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }
    
    /// Get the next event (blocking with timeout)
    pub fn next(&self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Mouse(mouse) => Ok(Event::Mouse(mouse)),
                CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
                CrosstermEvent::FocusGained => self.next(),
                CrosstermEvent::FocusLost => self.next(),
                CrosstermEvent::Paste(_) => self.next(),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}
