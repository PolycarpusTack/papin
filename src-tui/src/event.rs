use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

use crate::error::AppResult;

/// Events supported by the application
pub enum Event {
    /// Terminal tick (for regular updates)
    Tick,
    
    /// Key event
    Key(KeyEvent),
    
    /// Mouse event
    Mouse(MouseEvent),
    
    /// Terminal resize
    Resize(u16, u16),
}

/// Event handler
pub struct EventHandler {
    /// Event sender
    sender: mpsc::Sender<Event>,
    
    /// Event receiver
    receiver: mpsc::Receiver<Event>,
    
    /// Event handler thread
    thread: Option<thread::JoinHandle<()>>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let thread = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    // Timeout for polling events
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(Duration::from_secs(0));
                    
                    // Check for events
                    if event::poll(timeout).unwrap() {
                        match event::read().unwrap() {
                            CrosstermEvent::Key(key) => {
                                if sender.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if sender.send(Event::Mouse(mouse)).is_err() {
                                    break;
                                }
                            }
                            CrosstermEvent::Resize(width, height) => {
                                if sender.send(Event::Resize(width, height)).is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    // Check if tick rate elapsed
                    if last_tick.elapsed() >= tick_rate {
                        // Send tick event
                        if sender.send(Event::Tick).is_err() {
                            break;
                        }
                        // Reset last tick
                        last_tick = Instant::now();
                    }
                }
            })
        };
        
        Self {
            sender,
            receiver,
            thread: Some(thread),
        }
    }
    
    /// Get the next event from the handler
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv().unwrap())
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        // Close the channel to end the thread
        drop(self.sender.clone());
        
        // Join the thread
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
