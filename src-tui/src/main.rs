mod app;
mod error;
mod event;
mod ui;
mod util;

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, AppResult};
use event::{Event, EventHandler};
use mcp_common::{get_mcp_service, init_mcp_service, service::ChatService};

// Entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Initialize services
    let mcp_service = init_mcp_service();
    let chat_service = Arc::new(ChatService::new(mcp_service));
    
    // Create app and run it
    let app = App::new(chat_service);
    let res = run_app(&mut terminal, app).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    // Print any error
    if let Err(err) = res {
        eprintln!("{}", err);
    }
    
    Ok(())
}

// Run the application
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> AppResult<()> {
    // Create an event handler
    let mut event_handler = EventHandler::new(Duration::from_millis(100));
    
    // Initialize the app
    app.initialize().await?;
    
    // Main loop
    loop {
        // Render the UI
        terminal.draw(|f| ui::draw(f, &app))?;
        
        // Handle events
        match event_handler.next()? {
            Event::Tick => {
                app.tick();
            }
            Event::Key(key_event) => {
                // Pass the key event to the app
                if app.handle_key_event(key_event).await? {
                    // If the app returns true, exit the application
                    return Ok(());
                }
            }
            Event::Mouse(mouse_event) => {
                app.handle_mouse_event(mouse_event);
            }
            Event::Resize(width, height) => {
                app.resize(width, height);
            }
        }
    }
}
