mod api;
mod client;
mod syntax;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use client::ConversationClient;
use api::{ApiRequest, ApiResponse, ErrorResponse, Message};
use reqwest::Client;
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

use std::thread;
use std::time::Duration;
use tui::format_message_for_tui;
use rustyline::Editor;
use rustyline::history::History;

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

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
    let mut input = String::new();
    let mut status = String::new();
    let mut waiting = false;
    let mut progress_i = 0;
    let progress_frames = ["    ", ".   ", "..  ", "... ", "...."];
    let mut history_index: Option<usize> = None;
    let mut chat_scroll_offset: u16 = 0;
    let mut auto_scroll = true;
    let mut last_message_count: usize = 0;

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
                    Constraint::Length(3),  // Input
                    Constraint::Length(3),  // Status
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
                
                // Calculate the actual number of visual lines after wrapping
                let mut total_visual_lines: u16 = 0;
                for line in &chat_spans {
                    let line_width = line.width() as u16;
                    if line_width > chat_width {
                        // This line will wrap - calculate how many visual lines it needs
                        total_visual_lines += (line_width + chat_width - 1) / chat_width;
                    } else {
                        total_visual_lines += 1;
                    }
                }
                
                if total_visual_lines > chat_height {
                    chat_scroll_offset = total_visual_lines - chat_height;
                } else {
                    chat_scroll_offset = 0;
                }
            }

            // Chat/messages area
            let chat = Paragraph::new(Text::from(chat_spans))
                .block(Block::default().borders(Borders::ALL).title("Conversation"))
                .wrap(Wrap { trim: false })
                .scroll((chat_scroll_offset, 0));
            f.render_widget(chat, layout[0]);

            // Input area (middle)
            let input_bar = Paragraph::new(input.as_str())
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input_bar, layout[1]);
            f.set_cursor(
                layout[1].x + input.len() as u16 + 1,
                layout[1].y + 1,
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
                    progress_frames[progress_i % progress_frames.len()]
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
                    KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => {
                        break;
                    }
                    KeyCode::Enter => {
                        if !input.is_empty() {
                            waiting = true;
                            status = "Sending to Claude...".to_string();
                            progress_i = 0;

                            let user_input = input.clone();
                            input.clear();
                            history_index = None; // Reset history index

                            // Add to rustyline history
                            rl.add_history_entry(&user_input).ok();

                            // Immediately add the user message for display
                            client.messages.push(Message {
                                role: "user".to_string(),
                                content: user_input.clone(),
                            });

                            // Update the message count tracker to prevent double-detection
                            last_message_count = client.messages.len();

                            // Force auto-scroll if the feature flag is enabled
                            if SCROLL_ON_USER_INPUT {
                                auto_scroll = true;
                            }

                            // Clone what you need for the API call
                            let messages = client.messages.clone();
                            let api_key = client.api_key.clone();
                            let model = client.model.clone();
                            let max_tokens = client.max_tokens;
                            let temperature = client.temperature;

                            let mut handle = Box::pin(tokio::spawn(async move {
                                // Build the request using the messages (already includes the user message)
                                let request = ApiRequest {
                                    model: model.clone(),
                                    max_tokens,
                                    temperature,
                                    messages: messages.clone(),
                                };

                                let client_http = Client::new();
                                let response = client_http
                                    .post("https://api.anthropic.com/v1/messages")
                                    .header("Content-Type", "application/json")
                                    .header("x-api-key", &api_key)
                                    .header("anthropic-version", "2023-06-01")
                                    .json(&request)
                                    .send()
                                    .await
                                    .context("Failed to send request to API")?;

                                let status = response.status();
                                let response_text = response.text().await?;

                                if !status.is_success() {
                                    let error_response: ErrorResponse = serde_json::from_str(&response_text)
                                        .context("Failed to parse error response")?;
                                    anyhow::bail!(
                                        "API Error ({}): {}",
                                        error_response.error.error_type,
                                        error_response.error.message
                                    );
                                }

                                let api_response: ApiResponse = serde_json::from_str(&response_text)
                                    .context("Failed to parse API response")?;

                                let total_input_tokens = api_response.usage.input_tokens;
                                let total_output_tokens = api_response.usage.output_tokens;
                                let mut messages = messages.clone();

                                let assistant_response = api_response
                                    .content
                                    .iter()
                                    .filter(|block| block.content_type == "text")
                                    .map(|block| block.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join("");

                                messages.push(Message {
                                    role: "assistant".to_string(),
                                    content: assistant_response.clone(),
                                });

                                Ok((assistant_response, total_input_tokens, total_output_tokens, messages))
                            }));

                            loop {
                                use futures::FutureExt;
                                if let Some(result) = handle.as_mut().now_or_never() {
                                    waiting = false;
                                    match result {
                                        Ok(Ok((response, input_tokens, output_tokens, messages))) => {
                                            client.total_input_tokens = input_tokens;
                                            client.total_output_tokens = output_tokens;
                                            client.messages = messages;
                                            status = format!("Received response ({} tokens)", response.len());
                                        }
                                        Ok(Err(e)) => {
                                            status = format!("Error: {}", e);
                                        }
                                        Err(e) => {
                                            status = format!("Task join error: {}", e);
                                        }
                                    }
                                    break;
                                }

                                // Draw UI with animated progress bar
                                terminal.draw(|f| {
                                    let size = f.size();
                                    let layout = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints([
                                            Constraint::Min(3),
                                            Constraint::Length(3),
                                            Constraint::Length(3),
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
                    
                                        // Calculate the actual number of visual lines after wrapping
                                        let mut total_visual_lines: u16 = 0;
                                        for line in &chat_spans {
                                            let line_width = line.width() as u16;
                                            if line_width > chat_width {
                                                // This line will wrap - calculate how many visual lines it needs
                                                total_visual_lines += (line_width + chat_width - 1) / chat_width;
                                            } else {
                                                total_visual_lines += 1;
                                            }
                                        }
                                        
                                        if total_visual_lines > chat_height {
                                            chat_scroll_offset = total_visual_lines - chat_height;
                                        } else {
                                            chat_scroll_offset = 0;
                                        }
                                    }
                                    
                                    let chat = Paragraph::new(Text::from(chat_spans))
                                        .block(Block::default().borders(Borders::ALL).title("Conversation"))
                                        .wrap(Wrap { trim: false })
                                        .scroll((chat_scroll_offset, 0));
                                    f.render_widget(chat, layout[0]);

                                    let input_bar = Paragraph::new("")
                                        .block(Block::default().borders(Borders::ALL).title("Input"));
                                    f.render_widget(input_bar, layout[1]);

                                    let bottom_chunks = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints([
                                            Constraint::Percentage(70),
                                            Constraint::Percentage(30),
                                        ])
                                        .split(layout[2]);

                                    let status_text = format!(
                                        "Waiting for Claude {}",
                                        progress_frames[progress_i % progress_frames.len()]
                                    );
                                    let status_bar = Paragraph::new(status_text)
                                        .block(Block::default().borders(Borders::ALL).title("Status"));
                                    f.render_widget(status_bar, bottom_chunks[0]);

                                    let token_usage_text = format!(
                                        "Input tokens: {}, Output tokens: {}, Total tokens: {}",
                                        client.total_input_tokens,
                                        client.total_output_tokens,
                                        client.total_tokens()
                                    );
                                    let token_usage = Paragraph::new(token_usage_text)
                                        .block(Block::default().borders(Borders::ALL).title("Token Usage"));
                                    f.render_widget(token_usage, bottom_chunks[1]);
                                })?;

                                progress_i += 1;
                                tokio::time::sleep(Duration::from_millis(250)).await;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Up => {
                        let history = rl.history();
                        if history.len() == 0 {
                            // No history
                        } else {
                            history_index = Some(match history_index {
                                None => history.len().saturating_sub(1),
                                Some(0) => 0,
                                Some(i) => i.saturating_sub(1),
                            });
                            if let Some(i) = history_index {
                                let entries: Vec<String> = history.iter().map(|s| s.to_string()).collect();
                                if i < entries.len() {
                                    input = entries[i].clone();
                                }
                            }
                        }
                    }
                    KeyCode::Down => {
                        let history = rl.history();
                        if let Some(i) = history_index {
                            if i + 1 < history.len() {
                                history_index = Some(i + 1);
                                let entries: Vec<String> = history.iter().map(|s| s.to_string()).collect();
                                if i + 1 < entries.len() {
                                    input = entries[i + 1].clone();
                                }
                            } else {
                                history_index = None;
                                input.clear();
                            }
                        }
                    }
                    KeyCode::PageUp => {
                        auto_scroll = false; // Disable auto-scroll when manually scrolling
                        chat_scroll_offset = chat_scroll_offset.saturating_sub(5);
                    }
                    KeyCode::PageDown => {
                        // Calculate max scroll without drawing
                        let size = terminal.size()?;
                        let layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Min(3),
                                Constraint::Length(3),
                                Constraint::Length(3),
                            ])
                            .split(size);
                        
                        let chat_height = layout[0].height.saturating_sub(2);
                        let chat_width = layout[0].width.saturating_sub(2);
                        let mut chat_spans = Vec::new();
                        for msg in &client.messages {
                            chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
                        }

                        // Calculate visual lines with wrapping
                        let mut total_visual_lines: u16 = 0;
                        for line in &chat_spans {
                            let line_width = line.width() as u16;
                            if line_width > chat_width {
                                total_visual_lines += (line_width + chat_width - 1) / chat_width;
                            } else {
                                total_visual_lines += 1;
                            }
                        }

                        let max_scroll = if total_visual_lines > chat_height {
                            total_visual_lines - chat_height
                        } else {
                            0
                        };
                        
                        let new_offset = chat_scroll_offset.saturating_add(5);
                        chat_scroll_offset = new_offset.min(max_scroll);
                        
                        // If we've reached the bottom, re-enable auto-scroll
                        if chat_scroll_offset >= max_scroll {
                            auto_scroll = true;
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::F(2) => {
                        // Clear conversation
                        client.clear_conversation();
                        input.clear();
                        status.clear();
                    }
                    _ => {}
                }
            }
        }

        // Small sleep to prevent busy waiting when no events
        thread::sleep(Duration::from_millis(10));
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
