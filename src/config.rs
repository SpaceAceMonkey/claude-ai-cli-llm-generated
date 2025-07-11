// src/config.rs
use clap::Parser;

/// Feature flags and configuration constants
pub const SCROLL_ON_USER_INPUT: bool = true;
pub const SCROLL_ON_API_RESPONSE: bool = true;
pub const SHIFT_ENTER_SENDS: bool = false;
pub const SHOW_DEBUG_MESSAGES: bool = true;

pub const PROGRESS_FRAMES: [&str; 5] = ["    ", ".   ", "..  ", "... ", "...."];

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Your Anthropic API key
    #[arg(short, long)]
    #[arg(env = "ANTHROPIC_API_KEY")]
    pub api_key: String,

    /// Model to use (default: claude-3-5-sonnet-20241022)
    #[arg(short, long, default_value = "claude-3-5-sonnet-20241022")]
    pub model: String,

    /// Maximum tokens for response
    #[arg(short = 't', long, default_value = "1024")]
    pub max_tokens: u32,

    /// Temperature (0.0 to 1.0)
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,

    /// Simulate API calls without actually sending requests
    #[arg(long)]
    pub simulate: bool,
}