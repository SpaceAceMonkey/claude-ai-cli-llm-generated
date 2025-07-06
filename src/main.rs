use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use rustyline::Editor;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use tokio;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Your Anthropic API key
    /// sk-ant-api04-etc-etc-etc-etc
    #[arg(short, long, env = "ANTHROPIC_API_KEY")]
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

#[derive(Serialize, Debug, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: Usage,
}

#[derive(Deserialize, Debug)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Deserialize, Debug)]
struct ErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

struct ConversationClient {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<Message>,
}

impl ConversationClient {
    fn new(api_key: String, model: String, max_tokens: u32, temperature: f32) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            max_tokens,
            temperature,
            messages: Vec::new(),
        }
    }

    async fn send_message(&mut self, user_input: &str) -> Result<String> {
        // Add user message to conversation history
        self.messages.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let request = ApiRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            messages: self.messages.clone(),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
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

        // Extract the assistant's response text
        let assistant_response = api_response
            .content
            .iter()
            .filter(|block| block.content_type == "text")
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        // Add assistant's response to conversation history
        self.messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_response.clone(),
        });

        // Print token usage
        println!(
            "\n[Tokens - Input: {}, Output: {}, Total: {}]",
            api_response.usage.input_tokens,
            api_response.usage.output_tokens,
            api_response.usage.input_tokens + api_response.usage.output_tokens
        );

        Ok(assistant_response)
    }

    fn show_conversation_stats(&self) {
        println!(
            "\n[Conversation: {} messages, Model: {}]",
            self.messages.len(),
            self.model
        );
    }

    fn clear_conversation(&mut self) {
        self.messages.clear();
        println!("Conversation history cleared.");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let features = AppFeatures {
        less_available: is_tool_available("less"),
    };

    println!("🤖 Claude Interactive CLI Client");
    println!("Type 'quit' or 'exit' to quit");
    println!("Type 'clear' to clear conversation history");
    println!("Type 'stats' to show conversation statistics");
    println!("Type '/history' or '/hi' to show conversation history");
    println!("Type '/historyl' or '/hl' to show conversation history as piped through less");
    println!("Model: {}", args.model);
    println!("Max tokens: {}", args.max_tokens);
    println!("Temperature: {}", args.temperature);
    println!("{}", "=".repeat(50));

    if !features.less_available {
        println!("(Note: 'less' not found, '/historyl' and '/hl' will be unavailable)");
    }

    let mut client = ConversationClient::new(
        args.api_key,
        args.model,
        args.max_tokens,
        args.temperature,
    );

    // Initialize rustyline editor for input with history
    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();

    loop {
        let readline = rl.readline("\n> ");
        let input = match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    rl.add_history_entry(trimmed);
                }
                trimmed.to_string()
            }
            Err(_) => {
                println!("\nGoodbye! 👋");
                break;
            }
        };

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().trim() {
            "quit" | "exit" => {
                println!("Goodbye! 👋");
                break;
            }
            "clear" => {
                client.clear_conversation();
                continue;
            }
            "stats" => {
                client.show_conversation_stats();
                continue;
            }
            "/history" | "/hi" => {
                show_history_overlay(&client.messages);
                continue;
            }
            "/historyl" | "/hl" => {
                if features.less_available {
                    show_history_with_less(&client.messages);
                } else {
                    println!("'less' is not available on this system.");
                }
                continue;
            }
            _ => {}
        }

        println!("\nSending request...");

        match client.send_message(&input).await {
            Ok(response) => {
                println!("\n🤖 Claude:");
                print_message_with_highlighting("assistant", &response);
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = Message {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
        };
        
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello, world!\""));
    }

    #[test]
    fn test_api_request_serialization() {
        let request = ApiRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"claude-3-5-sonnet-20241022\""));
        assert!(json.contains("\"max_tokens\":1024"));
    }
}

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn show_history_overlay(messages: &[Message]) {
    print!("\x1B[2J\x1B[1;1H");
    println!("--- Conversation History ---\n");
    for msg in messages {
        print_message_with_highlighting(&msg.role, &msg.content);
    }
    println!("\nPress Enter, Space, or Escape to return...");

    // Enable raw mode to suppress escape sequences and echo
    enable_raw_mode().ok();

    // Swallow all keypresses except Enter, Space, or Escape
    loop {
        if let Ok(true) = event::poll(std::time::Duration::from_millis(500)) {
            if let Ok(Event::Key(key_event)) = event::read() {
                match key_event.code {
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => break,
                    _ => {} // Ignore all other keys
                }
            }
        }
    }

    // Always disable raw mode before returning
    disable_raw_mode().ok();

    print!("\x1B[2J\x1B[1;1H");
    println!("--- Conversation So Far ---\n");
    for msg in messages {
        print_message_with_highlighting(&msg.role, &msg.content);
    }
}

fn show_history_with_less(messages: &[Message]) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut history = String::new();
    for msg in messages {
        history.push_str(&format!("[{}]: {}\n", msg.role, msg.content));
    }

    let mut child = Command::new("less")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn less");
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(history.as_bytes());
    }
    let _ = child.wait();
}

use std::process::Command;
fn is_tool_available(tool: &str) -> bool {
    Command::new("which")
        .arg(tool)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

struct AppFeatures {
    less_available: bool,
}

use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

fn highlight_code_block(code: &str, language: &str) -> String {
    // Load syntax and theme sets once (could be optimized with lazy_static or similar)
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_token(language).unwrap_or_else(|| ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut highlighted = String::new();
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        highlighted.push_str(&as_24_bit_terminal_escaped(&ranges[..], false));
    }
    highlighted
}

fn print_message_with_highlighting(role: &str, content: &str) {
    let mut in_code = false;
    let mut code_lang = "rust"; // Default to rust, or parse from block
    let mut code_buf = String::new();

    // Choose color code for each role
    let (role_color_start, role_color_end) = match role {
        "assistant" => ("\x1b[1;36m", "\x1b[0m"), // Bold cyan
        "user" => ("\x1b[1;35m", "\x1b[0m"),      // Bold green
        _ => ("\x1b[1;33m", "\x1b[0m"),           // Bold yellow
    };

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            if in_code {
                // End of code block: print highlighted
                println!("{}", highlight_code_block(&code_buf, code_lang));
                code_buf.clear();
                in_code = false;
            } else {
                // Start of code block: parse language if present
                let after = line.trim_start().trim_start_matches("```").trim();
                if !after.is_empty() {
                    code_lang = after;
                } else {
                    code_lang = "rust";
                }
                in_code = true;
            }
            continue;
        }
        if in_code {
            code_buf.push_str(line);
            code_buf.push('\n');
        } else {
            // Print the colored role name
            println!(
                "[{}{}{}]: {}",
                role_color_start, role, role_color_end, line
            );
        }
    }
    // Print any trailing code block (if not closed)
    if in_code && !code_buf.is_empty() {
        println!("{}", highlight_code_block(&code_buf, code_lang));
    }
}
