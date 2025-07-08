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
use api::Message;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph, Wrap, Clear, List, ListItem, ListState},
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
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use config::{SCROLL_ON_USER_INPUT, SCROLL_ON_API_RESPONSE, SHIFT_ENTER_SENDS, PROGRESS_FRAMES};
use utils::text::*;
use handlers::history::{navigate_history_up, navigate_history_down};
use std::time::Duration;
use tui::format_message_for_tui;
use rustyline::Editor;

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

#[derive(Serialize, Deserialize, Clone)]
struct SavedConversation {
    version: String,
    timestamp: String,
    model: String,
    total_input_tokens: u32,
    total_output_tokens: u32,
    messages: Vec<Message>,
}

impl SavedConversation {
    fn new(client: &ConversationClient) -> Self {
        Self {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            model: client.model.clone(),
            total_input_tokens: client.total_input_tokens,
            total_output_tokens: client.total_output_tokens,
            messages: client.messages.clone(),
        }
    }

    fn validate(&self) -> bool {
        // Validate the conversation file format
        self.version == "1.0" && !self.messages.is_empty()
    }
}

// Replace the get_saves_directory function:
fn get_saves_directory() -> PathBuf {
    // Start from current working directory where the executable is
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn load_directory_contents(files: &mut Vec<String>, current_dir: &PathBuf, is_save_dialog: bool) {
    files.clear();
    
    // Add parent directory unless we're at root
    if current_dir.parent().is_some() {
        files.push("../".to_string());
    }
    
    // Add option to create new directory only for save dialog
    if is_save_dialog {
        files.push("[ Create New Directory ]".to_string());
    }
    
    if let Ok(entries) = fs::read_dir(current_dir) {
        let mut dirs = Vec::new();
        let mut regular_files = Vec::new();
        
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                // Show hidden directories starting with '.' but skip hidden files
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(format!("{}/", filename));
                } else if !filename.starts_with('.') {
                    regular_files.push(filename.to_string());
                }
            }
        }
        
        // Sort directories and files separately
        dirs.sort();
        regular_files.sort();
        
        // Add directories first, then files
        files.extend(dirs);
        files.extend(regular_files);
    }
    
    // If directory is empty, show a message
    let expected_count = if current_dir.parent().is_some() { 1 } else { 0 } + if is_save_dialog { 1 } else { 0 };
    if files.len() <= expected_count {
        files.push("(Empty directory)".to_string());
    }
}

fn save_conversation(client: &ConversationClient, filepath: &PathBuf) -> Result<()> {
    let conversation = SavedConversation::new(client);
    let json = serde_json::to_string_pretty(&conversation)?;
    fs::write(filepath, json)?;
    Ok(())
}

fn load_conversation(filepath: &PathBuf) -> Result<SavedConversation> {
    let json = fs::read_to_string(filepath)?;
    let conversation: SavedConversation = serde_json::from_str(&json)?;
    if !conversation.validate() {
        return Err(anyhow::anyhow!("Invalid conversation file format"));
    }
    Ok(conversation)
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
    let mut cursor_position: usize = 0;
    let mut input_scroll_offset: u16 = 0;
    let mut input_draft: Option<String> = None;

    // Channel for API responses
    let (tx, mut rx) = mpsc::channel::<(String, u32, u32, Vec<Message>)>(10);

    // File dialog state
    let mut show_save_dialog = false;
    let mut show_load_dialog = false;
    let mut save_filename = String::new();
    let mut available_files: Vec<String> = Vec::new();
    let mut file_list_state = ListState::default();
    let mut dialog_cursor_pos = 0;
    let mut current_directory = get_saves_directory();

    // New state variables
    let mut show_create_dir_dialog = false;
    let mut new_dir_name = String::new();

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

            // Save dialog overlay
            if show_save_dialog {
                let dialog_area = ratatui::layout::Rect {
                    x: size.width / 6,
                    y: size.height / 4,
                    width: (size.width * 2) / 3,
                    height: size.height / 2,
                };
                
                f.render_widget(Clear, dialog_area);
                
                let file_items: Vec<ListItem> = available_files.iter().map(|f| ListItem::new(f.as_str())).collect();
                
                let file_list = List::new(file_items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Save Conversation - {} (↑↓ to select, Enter to save/navigate, Esc to cancel)", current_directory.display())))
                    .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue))
                    .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
                
                f.render_stateful_widget(file_list, dialog_area, &mut file_list_state);
                
                // Show filename input at the bottom of the dialog
                let input_area = ratatui::layout::Rect {
                    x: dialog_area.x + 1,
                    y: dialog_area.y + dialog_area.height - 3,
                    width: dialog_area.width - 2,
                    height: 1,
                };
                
                let filename_input = Paragraph::new(format!("Filename: {}", save_filename))
                    .style(ratatui::style::Style::default().bg(ratatui::style::Color::DarkGray));
                f.render_widget(filename_input, input_area);
                
                // Set cursor after filename prompt
                f.set_cursor(
                    input_area.x + "Filename: ".len() as u16 + save_filename.len() as u16,
                    input_area.y,
                );
            }
            
            // Load dialog overlay
            if show_load_dialog {
                let dialog_area = ratatui::layout::Rect {
                    x: size.width / 6,
                    y: size.height / 4,
                    width: (size.width * 2) / 3,
                    height: size.height / 2,
                };
                
                f.render_widget(Clear, dialog_area);
                
                let file_items: Vec<ListItem> = available_files.iter().map(|f| ListItem::new(f.as_str())).collect();
                
                let file_list = List::new(file_items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Load Conversation - {} (↑↓ to select, Enter to open, Esc to cancel)", current_directory.display())))
                    .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue))
                    .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
                
                f.render_stateful_widget(file_list, dialog_area, &mut file_list_state);
            }
            
            // Create directory dialog overlay
            if show_create_dir_dialog {
                let dialog_area = ratatui::layout::Rect {
                    x: size.width / 4,
                    y: size.height / 3,
                    width: size.width / 2,
                    height: 5,
                };
                
                f.render_widget(Clear, dialog_area);
                
                let create_dialog = Paragraph::new(format!("Enter directory name: {}", new_dir_name))
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Create Directory in {}", current_directory.display())))
                    .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
                
                f.render_widget(create_dialog, dialog_area);
                
                // Fix cursor positioning - place it right after "Enter directory name: "
                let prompt_len = "Enter directory name: ".len();
                f.set_cursor(
                    dialog_area.x + 1 + prompt_len as u16 + new_dir_name.len() as u16,
                    dialog_area.y + 1,
                );
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
                    // Handle dialog inputs first
                    _ if show_save_dialog => {
                        match code {
                            KeyCode::Enter => {
                                if let Some(selected) = file_list_state.selected() {
                                    if selected < available_files.len() {
                                        let filename = &available_files[selected];
                                        if filename == "../" {
                                            if let Some(parent) = current_directory.parent() {
                                                current_directory = parent.to_path_buf();
                                                load_directory_contents(&mut available_files, &current_directory, true);
                                                file_list_state.select(Some(0));
                                            }
                                        } else if filename == "[ Create New Directory ]" {
                                            show_create_dir_dialog = true;
                                            new_dir_name.clear();
                                        } else if filename.ends_with('/') {
                                            let dirname = &filename[..filename.len()-1];
                                            current_directory.push(dirname);
                                            load_directory_contents(&mut available_files, &current_directory, true);
                                            file_list_state.select(Some(0));
                                        } else if filename.starts_with('(') && filename.ends_with(')') {
                                            // Skip placeholder messages
                                        } else {
                                            // Pre-fill the filename from selected file
                                            save_filename = filename.clone();
                                            dialog_cursor_pos = save_filename.len();
                                        }
                                    }
                                } else if !save_filename.is_empty() {
                                    // Save with the typed filename
                                    let mut filepath = current_directory.clone();
                                    filepath.push(&save_filename);
                                    match save_conversation(&client, &filepath) {
                                        Ok(_) => status = format!("Conversation saved to {}", filepath.display()),
                                        Err(e) => status = format!("Save failed: {}", e),
                                    }
                                    show_save_dialog = false;
                                    save_filename.clear();
                                    dialog_cursor_pos = 0;
                                }
                            }
                            KeyCode::Esc => {
                                show_save_dialog = false;
                                save_filename.clear();
                                dialog_cursor_pos = 0;
                            }
                            KeyCode::Up => {
                                if let Some(selected) = file_list_state.selected() {
                                    if selected > 0 {
                                        file_list_state.select(Some(selected - 1));
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if let Some(selected) = file_list_state.selected() {
                                    if selected < available_files.len().saturating_sub(1) {
                                        file_list_state.select(Some(selected + 1));
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                if !save_filename.is_empty() && dialog_cursor_pos > 0 {
                                    let mut chars: Vec<char> = save_filename.chars().collect();
                                    chars.remove(dialog_cursor_pos - 1);
                                    save_filename = chars.into_iter().collect();
                                    dialog_cursor_pos -= 1;
                                }
                            }
                            KeyCode::Char(c) => {
                                let mut chars: Vec<char> = save_filename.chars().collect();
                                chars.insert(dialog_cursor_pos, c);
                                save_filename = chars.into_iter().collect();
                                dialog_cursor_pos += 1;
                            }
                            _ => {}
                        }
                    }
                    _ if show_create_dir_dialog => {
                        match code {
                            KeyCode::Enter => {
                                if !new_dir_name.is_empty() {
                                    let mut new_dir_path = current_directory.clone();
                                    new_dir_path.push(&new_dir_name);
                                    match std::fs::create_dir_all(&new_dir_path) {
                                        Ok(_) => {
                                            status = format!("Directory created: {}", new_dir_path.display());
                                            current_directory = new_dir_path;
                                            load_directory_contents(&mut available_files, &current_directory, true);
                                            file_list_state.select(Some(0));
                                        }
                                        Err(e) => {
                                            status = format!("Failed to create directory: {}", e);
                                        }
                                    }
                                }
                                show_create_dir_dialog = false;
                                new_dir_name.clear();
                            }
                            KeyCode::Esc => {
                                show_create_dir_dialog = false;
                                new_dir_name.clear();
                            }
                            KeyCode::Backspace => {
                                if !new_dir_name.is_empty() {
                                    let mut chars: Vec<char> = new_dir_name.chars().collect();
                                    chars.pop();
                                    new_dir_name = chars.into_iter().collect();
                                }
                            }
                            KeyCode::Char(c) => {
                                // Only allow valid directory name characters
                                if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                                    new_dir_name.push(c);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ if show_load_dialog => {
                        match code {
                            KeyCode::Enter => {
                                if let Some(selected) = file_list_state.selected() {
                                    if selected < available_files.len() {
                                        let filename = &available_files[selected];
                                        if filename == "../" {
                                            if let Some(parent) = current_directory.parent() {
                                                current_directory = parent.to_path_buf();
                                                load_directory_contents(&mut available_files, &current_directory, false);
                                                file_list_state.select(Some(0));
                                            }
                                        } else if filename.ends_with('/') {
                                            let dirname = &filename[..filename.len()-1];
                                            current_directory.push(dirname);
                                            load_directory_contents(&mut available_files, &current_directory, false);
                                            file_list_state.select(Some(0));
                                        } else if filename.starts_with('(') && filename.ends_with(')') {
                                            // Skip placeholder messages
                                        } else {
                                            let mut filepath = current_directory.clone();
                                            filepath.push(filename);
                                            match load_conversation(&filepath) {
                                                Ok(conversation) => {
                                                    client.messages = conversation.messages;
                                                    client.total_input_tokens = conversation.total_input_tokens;
                                                    client.total_output_tokens = conversation.total_output_tokens;
                                                    status = format!("Conversation loaded from {}", filepath.display());
                                                    auto_scroll = true;
                                                    show_load_dialog = false;
                                                }
                                                Err(e) => status = format!("Load failed: {}", e),
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => show_load_dialog = false,
                            KeyCode::Up => {
                                if let Some(selected) = file_list_state.selected() {
                                    if selected > 0 {
                                        file_list_state.select(Some(selected - 1));
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if let Some(selected) = file_list_state.selected() {
                                    if selected < available_files.len().saturating_sub(1) {
                                        file_list_state.select(Some(selected + 1));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => break,
                    KeyCode::Enter => {
                        // Check for commands first
                        if input == "/save" {
                            show_save_dialog = true;
                            save_filename.clear();
                            dialog_cursor_pos = 0;
                            current_directory = get_saves_directory();
                            load_directory_contents(&mut available_files, &current_directory, true);
                            file_list_state.select(Some(0));
                            input.clear();
                            cursor_position = 0;
                        } else if input == "/load" {
                            show_load_dialog = true;
                            current_directory = get_saves_directory();
                            load_directory_contents(&mut available_files, &current_directory, false);
                            file_list_state.select(Some(0));
                            input.clear();
                            cursor_position = 0;
                        } else if modifiers.contains(KeyModifiers::SHIFT) || modifiers.contains(KeyModifiers::ALT) {
                            if SHIFT_ENTER_SENDS && !input.is_empty() {
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
                                            tx_clone.send((response, input_tokens, output_tokens, updated_messages)).await.ok();
                                        }
                                        Err(e) => eprintln!("API Error: {}", e),
                                    }
                                });
                            } else {
                                // Shift/Alt+Enter sends the message
                                let mut chars: Vec<char> = input.chars().collect();
                                chars.insert(cursor_position, '\n');
                                input = chars.into_iter().collect();
                                cursor_position += 1;
                            }
                        } else if modifiers.contains(KeyModifiers::CONTROL) && !input.is_empty() {
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
                                        tx_clone.send((response, input_tokens, output_tokens, updated_messages)).await.ok();
                                    }
                                    Err(e) => eprintln!("API Error: {}", e),
                                }
                            });
                        } else {
                            // Regular Enter behavior depends on the feature flag
                            if SHIFT_ENTER_SENDS {
                                // Regular Enter inserts a newline
                                let mut chars: Vec<char> = input.chars().collect();
                                chars.insert(cursor_position, '\n');
                                input = chars.into_iter().collect();
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
                                                if let Err(_) = tx_clone.send((response, inputTokens, outputTokens, updated_messages)).await {
                                                    eprintln!("Failed to send response through channel");
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("API Error: {}", e);
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
                            let mut chars: Vec<char> = input.chars().collect();
                            if cursor_position <= chars.len() {
                                chars.remove(cursor_position - 1);
                                input = chars.into_iter().collect();
                                cursor_position -= 1;
                            }
                        }
                    }
                    KeyCode::Delete => {
                        let mut chars: Vec<char> = input.chars().collect();
                        if cursor_position < chars.len() {
                            chars.remove(cursor_position);
                            input = chars.into_iter().collect();
                        }
                    }
                    KeyCode::Left => {
                        if cursor_position > 0 { cursor_position -= 1; }
                    }
                    KeyCode::Right => {
                        if cursor_position < input.chars().count() { cursor_position += 1; }
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
                    KeyCode::Home => cursor_position = 0,
                    KeyCode::End => cursor_position = input.chars().count(),
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
                        // Check for commands at start of empty input
                        if cursor_position == 0 && input.is_empty() && c == '/' {
                            input.insert(cursor_position, c);
                            cursor_position += 1;
                        } else if input == "/save" && c == ' ' {
                            show_save_dialog = true;
                            save_filename.clear();
                            dialog_cursor_pos = 0;
                            current_directory = get_saves_directory();
                            load_directory_contents(&mut available_files, &current_directory, true);
                            file_list_state.select(Some(0));
                            input.clear();
                            cursor_position = 0;
                        } else if input == "/load" && c == ' ' {
                            show_load_dialog = true;
                            current_directory = get_saves_directory();
                            load_directory_contents(&mut available_files, &current_directory, false);
                            file_list_state.select(Some(0));
                            input.clear();
                            cursor_position = 0;
                        } else {
                            // Regular character input
                            let mut chars: Vec<char> = input.chars().collect();
                            chars.insert(cursor_position, c);
                            input = chars.into_iter().collect();
                            cursor_position += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check for API responses
        if let Ok((response, input_tokens, output_tokens, updated_messages)) = rx.try_recv() {
            waiting = false;
            status = "Ready".to_string();
            
            // Update messages
            // Only add the assistant's response (the last message in updated_messages)
            if let Some(assistant_msg) = updated_messages.last() {
                if assistant_msg.role == "assistant" {
                    client.messages.push(assistant_msg.clone());
                }
            }
            
            // Update token counts
            client.total_input_tokens += input_tokens;
            client.total_output_tokens += output_tokens;
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