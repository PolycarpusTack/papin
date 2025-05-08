use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode};

/// Draw the user interface
pub fn draw(f: &mut Frame, app: &App) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Status bar
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Input box
        ])
        .split(f.size());
    
    // Draw the status bar
    draw_status_bar(f, app, chunks[0]);
    
    // Draw the main area
    draw_main_area(f, app, chunks[1]);
    
    // Draw the input box
    draw_input_box(f, app, chunks[2]);
    
    // Draw help screen if enabled
    if app.show_help {
        draw_help_screen(f, app);
    }
    
    // Draw settings screen if enabled
    if app.settings_open {
        draw_settings_screen(f, app);
    }
}

/// Draw the status bar
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![];
    
    // App mode
    let mode_str = match app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Chatting => "CHAT",
        AppMode::Command => "COMMAND",
        AppMode::Help => "HELP",
        AppMode::Settings => "SETTINGS",
    };
    
    spans.push(Span::styled(
        format!(" {} ", mode_str),
        Style::default().bg(Color::Blue).fg(Color::White),
    ));
    
    // Current conversation
    if let Some(conversation) = &app.current_conversation {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            &conversation.title,
            Style::default().fg(Color::Green),
        ));
        
        if let Some(model) = &conversation.model {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                &model.name,
                Style::default().fg(Color::Yellow),
            ));
        }
    }
    
    // Streaming indicator
    if app.is_streaming {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            " STREAMING ",
            Style::default().bg(Color::LightGreen).fg(Color::Black),
        ));
    }
    
    // Status message
    if let Some((message, is_error)) = &app.status_message {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            message,
            Style::default().fg(if *is_error { Color::Red } else { Color::Green }),
        ));
    }
    
    // Create the paragraph
    let paragraph = Paragraph::new(Line::from(spans));
    
    // Render the status bar
    f.render_widget(paragraph, area);
}

/// Draw the main content area
fn draw_main_area(f: &mut Frame, app: &App, area: Rect) {
    // Split into conversations list and chat area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Conversations list
            Constraint::Percentage(80), // Chat area
        ])
        .split(area);
    
    // Draw the conversations list
    draw_conversations_list(f, app, chunks[0]);
    
    // Draw the chat area
    draw_chat_area(f, app, chunks[1]);
}

/// Draw the conversations list
fn draw_conversations_list(f: &mut Frame, app: &App, area: Rect) {
    // Create list items
    let items: Vec<ListItem> = app
        .conversations
        .iter()
        .enumerate()
        .map(|(i, conversation)| {
            let style = if Some(i) == app.selected_conversation_idx {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            
            ListItem::new(conversation.title.clone()).style(style)
        })
        .collect();
    
    // Create the list
    let list = List::new(items)
        .block(Block::default().title("Conversations").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    
    // Render the list
    f.render_widget(list, area);
}

/// Draw the chat area
fn draw_chat_area(f: &mut Frame, app: &App, area: Rect) {
    // Create the chat box
    let chat_box = Block::default()
        .title("Chat")
        .borders(Borders::ALL);
    
    // Render the chat box
    f.render_widget(chat_box, area);
    
    // Inner area for messages
    let inner_area = chat_box.inner(area);
    
    // Display conversation messages
    if let Some(conversation) = &app.current_conversation {
        let messages = &conversation.messages;
        
        if !messages.is_empty() {
            let mut text_spans = Vec::new();
            
            for message in messages {
                // Determine style based on role
                let (prefix, style) = match message.role.as_str() {
                    "user" => (
                        "You: ",
                        Style::default().fg(Color::Green),
                    ),
                    "assistant" => (
                        "Claude: ",
                        Style::default().fg(Color::Blue),
                    ),
                    "system" => (
                        "System: ",
                        Style::default().fg(Color::Yellow),
                    ),
                    _ => (
                        "Unknown: ",
                        Style::default(),
                    ),
                };
                
                // Add sender with style
                text_spans.push(Line::from(Span::styled(
                    prefix,
                    style.add_modifier(Modifier::BOLD),
                )));
                
                // Add message content
                for content in &message.content {
                    if let Some(text) = &content.text {
                        // Split by lines and add each as a span
                        for line in text.lines() {
                            text_spans.push(Line::from(line));
                        }
                    }
                }
                
                // Add separator
                text_spans.push(Line::from(""));
            }
            
            // Create the text widget
            let text = Text::from(text_spans);
            let paragraph = Paragraph::new(text)
                .wrap(Wrap { trim: false });
            
            // Render the messages
            f.render_widget(paragraph, inner_area);
        }
    }
}

/// Draw the input box
fn draw_input_box(f: &mut Frame, app: &App, area: Rect) {
    // Create the input box
    let input_box = Block::default()
        .title(match app.mode {
            AppMode::Chatting => "Message",
            AppMode::Command => "Command",
            _ => "Input",
        })
        .borders(Borders::ALL);
    
    // Set the block
    match app.mode {
        AppMode::Chatting => {
            app.input.set_block(input_box);
            f.render_widget(app.input.widget(), area);
        }
        AppMode::Command => {
            app.command_input.set_block(input_box);
            f.render_widget(app.command_input.widget(), area);
        }
        _ => {
            let text = match app.mode {
                AppMode::Normal => "Press Enter to chat, n for new, d to delete",
                AppMode::Help => "Press q to exit help",
                AppMode::Settings => "Press Esc to exit settings",
                _ => "",
            };
            
            let paragraph = Paragraph::new(text)
                .block(input_box);
            
            f.render_widget(paragraph, area);
        }
    }
}

/// Draw the help screen
fn draw_help_screen(f: &mut Frame, app: &App) {
    // Create a centered popup
    let area = centered_rect(60, 60, f.size());
    
    // Create the help box
    let help_box = Block::default()
        .title("Help")
        .borders(Borders::ALL);
    
    // Render the help box
    f.render_widget(help_box, area);
    
    // Inner area for help content
    let inner_area = help_box.inner(area);
    
    // Help text
    let text = Text::from(vec![
        Line::from("Claude MCP TUI Commands"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  q         - Quit application"),
        Line::from("  ?         - Show this help"),
        Line::from("  :         - Enter command mode"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  j/k       - Move up/down in lists"),
        Line::from("  Enter     - Select conversation"),
        Line::from("  Esc       - Return to normal mode"),
        Line::from(""),
        Line::from("Conversations:"),
        Line::from("  n         - Create new conversation"),
        Line::from("  d         - Delete current conversation"),
        Line::from("  r         - Reload conversations"),
        Line::from(""),
        Line::from("Chat:"),
        Line::from("  Ctrl+Enter - Send message"),
        Line::from("  PageUp/Down - Scroll through history"),
        Line::from(""),
        Line::from("Settings:"),
        Line::from("  s         - Open settings"),
    ]);
    
    // Create the text widget
    let paragraph = Paragraph::new(text);
    
    // Render the help content
    f.render_widget(paragraph, inner_area);
}

/// Draw the settings screen
fn draw_settings_screen(f: &mut Frame, app: &App) {
    // Create a centered popup
    let area = centered_rect(60, 60, f.size());
    
    // Create the settings box
    let settings_box = Block::default()
        .title("Settings")
        .borders(Borders::ALL);
    
    // Render the settings box
    f.render_widget(settings_box, area);
    
    // Inner area for settings content
    let inner_area = settings_box.inner(area);
    
    // Settings list
    let items = vec![
        ListItem::new("API Key Configuration"),
        ListItem::new("Default Model: Claude-3-Opus"),
        ListItem::new("Enable Message Streaming: Yes"),
        ListItem::new("Dark Mode: Enabled"),
        ListItem::new("Show System Messages: Yes"),
    ];
    
    // Create the list
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    
    // Render the settings list
    f.render_stateful_widget(
        list,
        inner_area,
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.settings_idx)),
    );
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
