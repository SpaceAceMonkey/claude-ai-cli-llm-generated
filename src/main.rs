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
use ratatui::text::{Span, Line, Text};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use std::io::Write;
use std::thread;
use std::time::Duration;
use tui::format_message_for_tui;
use rustyline::Editor;
use futures::future::FutureExt;

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

    loop {
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

            // Chat/messages area
            let mut chat_spans = Vec::new();
            for msg in &client.messages {
                chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
            }
            let chat = Paragraph::new(Text::from(chat_spans))
                .block(Block::default().borders(Borders::ALL).title("Conversation"))
                .wrap(Wrap { trim: false });
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
        if event::poll(Duration::from_millis(1000))? {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
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

                            // Immediately add the user message for display
                            client.messages.push(Message {
                                role: "user".to_string(),
                                content: user_input.clone(),
                            });

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

                                let mut total_input_tokens = api_response.usage.input_tokens;
                                let mut total_output_tokens = api_response.usage.output_tokens;
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
                                    let chat = Paragraph::new(Text::from(chat_spans))
                                        .block(Block::default().borders(Borders::ALL).title("Conversation"))
                                        .wrap(Wrap { trim: false });
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
                    KeyCode::Char(ch) => {
                        input.push(ch);
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

        thread::sleep(Duration::from_millis(100));
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
