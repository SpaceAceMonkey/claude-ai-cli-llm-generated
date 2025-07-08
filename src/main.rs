mod api;
mod client;
mod syntax;
mod tui;
mod app;
mod config;
mod utils;
mod handlers;

use anyhow::Result;
use clap::Parser;
use client::ConversationClient;
use api::Message;  // Only keep Message, remove unused API types
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction},
};
use ratatui::text::Text;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use handlers::api::send_message_to_api;
use tokio::sync::mpsc;

// Remove unused imports: app::AppState, config::*, scroll::*
use config::{SCROLL_ON_USER_INPUT, SCROLL_ON_API_RESPONSE, SHIFT_ENTER_SENDS, PROGRESS_FRAMES};
use utils::text::*;  // Keep text utilities
use handlers::history::{navigate_history_up, navigate_history_down};

// Remove unused imports: std::thread
use std::time::Duration;
use tui::format_message_for_tui;
use rustyline::Editor;
// Remove unused import: rustyline::history::History

// Remove the unused SHOW_DEBUG_MESSAGES constant (line 69):
// const SHOW_DEBUG_MESSAGES: bool = true;  // Remove this line

// Change the mutable progress_frames to use the one from config (line 91):
// Remove: let progress_frames = ["    ", ".   ", "..  ", "... ", "...."];
// It's already imported from config::PROGRESS_FRAMES

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Your Anthropic API key
    #[arg(short, long)]
    #[arg(env = "ANTHROPIC_API_KEY")]
    api_key: String,

    /// Model to use (default: claude-3-5-sonnet-20241022)
    #[arg(short, long, default_value = "claude-3-5-sonnet-20241022")]
    model: String,

    /// Maximum tokens for response
    #[arg(short = 't', long, default_value = "1024")]
    max_tokens: u32,

    /// Temperature (0.0 to 1.0)
    #[arg(long, default_value = "0.7")]
    temperature: f32,

    /// Simulate API calls without actually sending requests
    #[arg(long)]
    simulate: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Feature flag constants
    const SCROLL_ON_USER_INPUT: bool = true;  // Feature flag for scrolling on user input
    const SCROLL_ON_API_RESPONSE: bool = true; // Feature flag for scrolling on API response

    // Setup TUI
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = ConversationClient::new(
        args.api_key,
        args.model,
        args.max_tokens,
        args.temperature,
    );
    let simulate_mode = args.simulate;  // Store the flag value

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
    let mut input = String::new();
    let mut status = String::new();
    let mut waiting = false;
    let mut progress_i = 0;
    let mut history_index: Option<usize> = None;
    let mut chat_scroll_offset: u16 = 0;
    let mut auto_scroll = true;
    let mut last_message_count: usize = 0;

    // New state variables
    let mut cursor_position: usize = 0;  // Track cursor position in input
    let mut input_scroll_offset: u16 = 0;  // Track scroll position for input
    let mut input_draft: Option<String> = None;  // Save current input when browsing history

    // Channel for API responses
    let (tx, mut rx) = mpsc::channel::<Result<(String, u32, u32, Vec<Message>), String>>(10);

    // Add these state variables after the other state variables (around line 90):
    let mut show_error_dialog = false;
    let mut error_message = String::new();

    loop {
        // Check for new messages BEFORE drawing
        let current_message_count = client.messages.len();
        if current_message_count != last_message_count {
            let is_user_message = client.messages.last()
                .map(|m| m.role == "user")
                .unwrap_or(false);
            
            last_message_count = current_message_count;
            
            // Apply feature flags to control when to enable auto-scroll
            if (is_user_message && SCROLL_ON_USER_INPUT) || 
               (!is_user_message && SCROLL_ON_API_RESPONSE) {
                auto_scroll = true;
            }
        }

        // Draw UI
        terminal.draw(|f| {
            let size = f.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),      // Conversation
                    Constraint::Length(6),   // Input (4 lines + 2 for borders)
                    Constraint::Length(3),   // Status
                ])
                .split(size);

            let mut chat_spans = Vec::new();
            for msg in &client.messages {
                chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
            }

            // Calculate proper scroll offset if auto_scroll is enabled
            if auto_scroll && !chat_spans.is_empty() {
                let chat_height = layout[0].height.saturating_sub(2); // subtract borders
                let chat_width = layout[0].width.saturating_sub(2); // subtract borders
                
                // Calculate total visual lines after wrapping
                let mut total_visual_lines: u16 = 0;
                for line in &chat_spans {
                    let line_width = line.width();
                    if line_width == 0 {
                        total_visual_lines += 1; // Empty line
                    } else {
                        // Calculate how many visual lines this will take after wrapping
                        let wrapped_lines = ((line_width as u16 + chat_width - 1) / chat_width).max(1);
                        total_visual_lines += wrapped_lines;
                    }
                }
                
                // Add a small buffer to ensure the last line is visible
                total_visual_lines += 1;
                
                if total_visual_lines > chat_height {
                    chat_scroll_offset = total_visual_lines - chat_height;
                } else {
                    chat_scroll_offset = 0;
                }
            }

            // Chat/messages area
            let chat_title = if simulate_mode {
                "Conversation (SIMULATE MODE)"
            } else {
                "Conversation"
            };
            let chat = Paragraph::new(Text::from(chat_spans))
                .block(Block::default().borders(Borders::ALL).title(chat_title))
                .wrap(Wrap { trim: false })
                .scroll((chat_scroll_offset, 0));
            f.render_widget(chat, layout[0]);

            // Input area (middle) - with wrapping and scroll
            let input_lines = wrap_text(&input, layout[1].width.saturating_sub(2) as usize);
            let cursor_line = calculate_cursor_line(&input, cursor_position, layout[1].width.saturating_sub(2) as usize);
            let input_height = layout[1].height.saturating_sub(2); // subtract borders

            // Auto-scroll input to keep cursor visible
            if cursor_line >= (input_scroll_offset as usize + input_height as usize) {
                input_scroll_offset = (cursor_line + 1).saturating_sub(input_height as usize) as u16;
            } else if cursor_line < input_scroll_offset as usize {
                input_scroll_offset = cursor_line as u16;
            }

            let input_title = if SHIFT_ENTER_SENDS {
                "Input (Shift/Alt+Enter to send, Enter for newline)"
            } else {
                "Input (Enter to send, Shift/Alt+Enter for newline)"
            };
            let input_bar = Paragraph::new(Text::from(input_lines))
                .block(Block::default().borders(Borders::ALL).title(input_title))
                .wrap(Wrap { trim: false })
                .scroll((input_scroll_offset, 0));
            f.render_widget(input_bar, layout[1]);

            // Calculate cursor position for rendering
            let (cursor_x, cursor_y) = calculate_cursor_position(
                &input,
                cursor_position,
                layout[1].width.saturating_sub(2) as usize,
                input_scroll_offset as usize,
            );
            f.set_cursor(
                layout[1].x + cursor_x as u16 + 1,
                layout[1].y + cursor_y as u16 + 1,
            );

            // Bottom section: split into status and token usage
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70), // Status
                    Constraint::Percentage(30), // Token usage
                ])
                .split(layout[2]);

            // Status/progress bar (bottom left)
            let status_text = if waiting {
                format!(
                    "Waiting for Claude {}",
                    PROGRESS_FRAMES[progress_i % PROGRESS_FRAMES.len()]  // Use PROGRESS_FRAMES from config
                )
            } else {
                status.clone()
            };
            let status_bar = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status_bar, bottom_chunks[0]);

            // Token usage (bottom right)
            let token_usage_text = format!(
                "Input tokens: {}, Output tokens: {}, Total tokens: {}",
                client.total_input_tokens,
                client.total_output_tokens,
                client.total_tokens()
            );
            let token_usage = Paragraph::new(token_usage_text)
                .block(Block::default().borders(Borders::ALL).title("Token Usage"));
            f.render_widget(token_usage, bottom_chunks[1]);

            // Error dialog overlay (render last so it appears on top)
            if show_error_dialog {
                let error_area = ratatui::layout::Rect {
                    x: size.width / 4,
                    y: size.height / 4,
                    width: size.width / 2,
                    height: size.height / 4,
                };
                
                // Clear the area first
                f.render_widget(
                    ratatui::widgets::Clear,
                    error_area
                );
                
                // Render the error dialog
                let error_dialog = Paragraph::new(error_message.clone())
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title("Error")
                        .title_style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                    )
                    .wrap(Wrap { trim: false })
                    .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
                
                f.render_widget(error_dialog, error_area);
            }
        })?;

        // Event handling
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                ..
            }) = event::read()?
            {
                match code {
                    // Handle error dialog dismissal first
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') if show_error_dialog => {
                        show_error_dialog = false;
                        error_message.clear();
                    }
                    // Only process other keys if error dialog is not shown
                    _ if show_error_dialog => {
                        // Ignore all other input when error dialog is shown
                    }
                    KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => {
                        break;
                    }
                    KeyCode::Enter => {
                        if modifiers.contains(KeyModifiers::SHIFT) || modifiers.contains(KeyModifiers::ALT) {
                            if SHIFT_ENTER_SENDS {
                                // Shift/Alt+Enter sends the message
                                if !input.is_empty() {
                                    waiting = true;
                                    status = "Sending to Claude...".to_string();
                                    progress_i = 0;

                                    let user_input = input.clone();
                                    input.clear();
                                    cursor_position = 0;
                                    history_index = None;
                                    input_draft = None;

                                    // Add to rustyline history
                                    rl.add_history_entry(&user_input).ok();

                                    // Add user message
                                    client.messages.push(Message {
                                        role: "user".to_string(),
                                        content: user_input.clone(),
                                    });

                                    // Spawn API call with channel
                                    let api_key = client.api_key.clone();
                                    let model = client.model.clone();
                                    let max_tokens = client.max_tokens;
                                    let temperature = client.temperature;
                                    let messages = client.messages.clone();
                                    let simulate = simulate_mode;
                                    let tx_clone = tx.clone();

                                    tokio::spawn(async move {
                                        match send_message_to_api(
                                            user_input,
                                            messages,
                                            api_key,
                                            model,
                                            max_tokens,
                                            temperature,
                                            simulate,
                                        ).await {
                                            Ok((response, input_tokens, output_tokens, updated_messages)) => {
                                                tx_clone.send(Ok((response, input_tokens, output_tokens, updated_messages))).await.ok();
                                            }
                                            Err(e) => {
                                                let error_msg = format!("API Error: {}", e);
                                                tx_clone.send(Err(error_msg)).await.ok();
                                            }
                                        }
                                    });
                                }
                            } else {
                                // Shift/Alt+Enter inserts a newline
                                input.insert(cursor_position, '\n');
                                cursor_position += 1;
                            }
                        } else if modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+Enter always sends
                            if !input.is_empty() {
                                waiting = true;
                                status = "Sending to Claude...".to_string();
                                progress_i = 0;

                                let user_input = input.clone();
                                input.clear();
                                cursor_position = 0;
                                history_index = None;
                                input_draft = None;

                                // Add to rustyline history
                                rl.add_history_entry(&user_input).ok();

                                // Add user message
                                client.messages.push(Message {
                                    role: "user".to_string(),
                                    content: user_input.clone(),
                                });

                                // Spawn API call with channel
                                let api_key = client.api_key.clone();
                                let model = client.model.clone();
                                let max_tokens = client.max_tokens;
                                let temperature = client.temperature;
                                let messages = client.messages.clone();
                                let simulate = simulate_mode;
                                let tx_clone = tx.clone();

                                tokio::spawn(async move {
                                    match send_message_to_api(
                                        user_input,
                                        messages,
                                        api_key,
                                        model,
                                        max_tokens,
                                        temperature,
                                        simulate,
                                    ).await {
                                        Ok((response, inputTokens, outputTokens, updated_messages)) => {
                                            tx_clone.send(Ok((response, inputTokens, outputTokens, updated_messages))).await.ok();
                                        }
                                        Err(e) => {
                                            let error_msg = format!("API Error: {}", e);
                                            tx_clone.send(Err(error_msg)).await.ok();
                                        }
                                    }
                                });
                            }
                        } else {
                            // Regular Enter behavior depends on the feature flag
                            if SHIFT_ENTER_SENDS {
                                // Regular Enter inserts a newline
                                input.insert(cursor_position, '\n');
                                cursor_position += 1;
                            } else {
                                // Regular Enter sends the message
                                if !input.is_empty() {
                                    waiting = true;
                                    status = "Sending to Claude...".to_string();
                                    progress_i = 0;

                                    let user_input = input.clone();
                                    input.clear();
                                    cursor_position = 0;
                                    history_index = None;
                                    input_draft = None;

                                    // Add to rustyline history
                                    rl.add_history_entry(&user_input).ok();

                                    // Add user message
                                    client.messages.push(Message {
                                        role: "user".to_string(),
                                        content: user_input.clone(),
                                    });

                                    // Spawn API call with channel
                                    let api_key = client.api_key.clone();
                                    let model = client.model.clone();
                                    let max_tokens = client.max_tokens;
                                    let temperature = client.temperature;
                                    let messages = client.messages.clone();
                                    let simulate = simulate_mode;
                                    let tx_clone = tx.clone();

                                    tokio::spawn(async move {
                                        match send_message_to_api(
                                            user_input,
                                            messages,
                                            api_key,
                                            model,
                                            max_tokens,
                                            temperature,
                                            simulate,
                                        ).await {
                                            Ok((response, inputTokens, outputTokens, updated_messages)) => {
                                                tx_clone.send(Ok((response, inputTokens, outputTokens, updated_messages))).await.ok();
                                            }
                                            Err(e) => {
                                                let error_msg = format!("API Error: {}", e);
                                                tx_clone.send(Err(error_msg)).await.ok();
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                    KeyCode::Char('v') if modifiers == KeyModifiers::CONTROL => {
                        // TODO: Implement proper clipboard handling
                        // For now, just skip it
                    }
                    KeyCode::Backspace => {
                        if cursor_position > 0 {
                            input.remove(cursor_position - 1);
                            cursor_position -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        input.remove(cursor_position);
                    }
                    KeyCode::Left => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if cursor_position < input.len() {
                            cursor_position += 1;
                        }
                    }
                    KeyCode::Up if modifiers.contains(KeyModifiers::CONTROL) || 
                                   modifiers.contains(KeyModifiers::ALT) || 
                                   modifiers.contains(KeyModifiers::SHIFT) => {
                        // Scroll chat up one line
                        if chat_scroll_offset > 0 {
                            chat_scroll_offset -= 1;
                            auto_scroll = false;
                        }
                    }
                    KeyCode::Down if modifiers.contains(KeyModifiers::CONTROL) || 
                                     modifiers.contains(KeyModifiers::ALT) || 
                                     modifiers.contains(KeyModifiers::SHIFT) => {
                        // Scroll chat down one line
                        let chat_height = terminal.size()?.height.saturating_sub(8);
                        
                        // Calculate max scroll
                        let mut chat_spans = Vec::new();
                        for msg in &client.messages {
                            chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
                        }
                        
                        if !chat_spans.is_empty() {
                            let chat_width = terminal.size()?.width.saturating_sub(4);
                            let mut total_visual_lines: u16 = 0;
                            
                            for line in &chat_spans {
                                let line_width = line.width();
                                if line_width == 0 {
                                    total_visual_lines += 1;
                                } else {
                                    let wrapped_lines = ((line_width as u16 + chat_width - 1) / chat_width).max(1);
                                    total_visual_lines += wrapped_lines;
                                }
                            }
                            
                            let max_scroll = total_visual_lines.saturating_sub(chat_height);
                            if chat_scroll_offset < max_scroll {
                                chat_scroll_offset += 1;
                            }
                            
                            // Re-enable auto-scroll if we're at the bottom
                            if chat_scroll_offset >= max_scroll {
                                auto_scroll = true;
                            }
                        }
                    }
                    KeyCode::Up => {
                        let input_width = terminal.size()?.width.saturating_sub(4) as usize;
                        let is_multiline = input.contains('\n') || input.len() > input_width;
                        
                        if is_multiline {
                            let new_pos = move_cursor_up(&input, cursor_position, input_width);
                            if new_pos != cursor_position {
                                cursor_position = new_pos;
                            } else {
                                navigate_history_up(&mut input, &mut cursor_position, &mut history_index, &mut input_draft, &rl);
                            }
                        } else {
                            navigate_history_up(&mut input, &mut cursor_position, &mut history_index, &mut input_draft, &rl);
                        }
                    }
                    KeyCode::Down => {
                        let input_width = terminal.size()?.width.saturating_sub(4) as usize;
                        let is_multiline = input.contains('\n') || input.len() > input_width;
                        
                        if is_multiline {
                            let new_pos = move_cursor_down(&input, cursor_position, input_width);
                            if new_pos != cursor_position {
                                cursor_position = new_pos;
                            } else {
                                navigate_history_down(&mut input, &mut cursor_position, &mut history_index, &mut input_draft, &rl);
                            }
                        } else {
                            navigate_history_down(&mut input, &mut cursor_position, &mut history_index, &mut input_draft, &rl);
                        }
                    }
                    KeyCode::Home => {
                        cursor_position = 0;
                    }
                    KeyCode::End => {
                        cursor_position = input.len();
                    }
                    KeyCode::PageUp => {
                        // Scroll chat up
                        if chat_scroll_offset > 0 {
                            let page_size = terminal.size()?.height.saturating_sub(12); // leave 2-3 lines for context
                            chat_scroll_offset = chat_scroll_offset.saturating_sub(page_size);
                            auto_scroll = false; // Disable auto-scroll when user manually scrolls
                        }
                    }
                    KeyCode::PageDown => {
                        // Scroll chat down
                        let chat_height = terminal.size()?.height.saturating_sub(8); // rough estimate
                        let page_size = chat_height.saturating_sub(4); // leave 2-3 lines for context
                        
                        // Calculate max scroll based on content
                        let mut chat_spans = Vec::new();
                        for msg in &client.messages {
                            chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
                        }
                        
                        if !chat_spans.is_empty() {
                            let chat_width = terminal.size()?.width.saturating_sub(4);
                            let mut total_visual_lines: u16 = 0;
                            
                            for line in &chat_spans {
                                let line_width = line.width();
                                if line_width == 0 {
                                    total_visual_lines += 1;
                                } else {
                                    let wrapped_lines = ((line_width as u16 + chat_width - 1) / chat_width).max(1);
                                    total_visual_lines += wrapped_lines;
                                }
                            }
                            
                            let max_scroll = total_visual_lines.saturating_sub(chat_height);
                            chat_scroll_offset = (chat_scroll_offset + page_size).min(max_scroll);
                            
                            // Re-enable auto-scroll if we're at the bottom
                            if chat_scroll_offset >= max_scroll {
                                auto_scroll = true;
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        input.insert(cursor_position, c);
                        cursor_position += 1;
                    }
                    _ => {}
                }
            }
        }

        // Check for API responses
        if let Ok(result) = rx.try_recv() {
            waiting = false;
            status = "Ready".to_string();
            
            match result {
                Ok((response, input_tokens, output_tokens, updated_messages)) => {
                    // Normal response handling
                    if let Some(assistant_msg) = updated_messages.last() {
                        if assistant_msg.role == "assistant" {
                            client.messages.push(assistant_msg.clone());
                        }
                    }
                    
                    client.total_input_tokens += input_tokens;
                    client.total_output_tokens += output_tokens;
                }
                Err(error_msg) => {
                    // Show the actual error message
                    show_error_dialog = true;
                    error_message = error_msg;
                }
            }
        }

        // Update progress animation for waiting state
        if waiting {
            // Slow down progress animation - only increment every 4th iteration
            static mut FRAME_COUNTER: u32 = 0;
            unsafe {
                FRAME_COUNTER += 1;
                if FRAME_COUNTER % 4 == 0 {
                    progress_i += 1;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // In simulate mode, we can add additional behavior if needed
        if simulate_mode {
            // Any simulate-specific behavior can go here
            // But the progress animation is now handled above for both modes
        }
    }

    // Cleanup: leave alternate screen and disable raw mode
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}