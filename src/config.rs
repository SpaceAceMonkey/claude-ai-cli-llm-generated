// src/config.rs
use clap::Parser;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Feature flags and configuration constants
pub const SCROLL_ON_USER_INPUT: bool = true;
pub const SCROLL_ON_API_RESPONSE: bool = true;
pub const SHIFT_ENTER_SENDS: bool = false;
pub const SHOW_DEBUG_MESSAGES: bool = false;

pub const PROGRESS_FRAMES: [&str; 5] = ["    ", ".   ", "..  ", "... ", "...."];

/// Available ANSI colors for user selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl AnsiColor {
    pub fn to_ratatui_color(self) -> Color {
        match self {
            AnsiColor::Black => Color::Black,
            AnsiColor::Red => Color::Red,
            AnsiColor::Green => Color::Green,
            AnsiColor::Yellow => Color::Yellow,
            AnsiColor::Blue => Color::Blue,
            AnsiColor::Magenta => Color::Magenta,
            AnsiColor::Cyan => Color::Cyan,
            AnsiColor::White => Color::White,
            AnsiColor::BrightBlack => Color::DarkGray,
            AnsiColor::BrightRed => Color::LightRed,
            AnsiColor::BrightGreen => Color::LightGreen,
            AnsiColor::BrightYellow => Color::LightYellow,
            AnsiColor::BrightBlue => Color::LightBlue,
            AnsiColor::BrightMagenta => Color::LightMagenta,
            AnsiColor::BrightCyan => Color::LightCyan,
            AnsiColor::BrightWhite => Color::Gray,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AnsiColor::Black => "Black",
            AnsiColor::Red => "Red",
            AnsiColor::Green => "Green",
            AnsiColor::Yellow => "Yellow",
            AnsiColor::Blue => "Blue",
            AnsiColor::Magenta => "Magenta",
            AnsiColor::Cyan => "Cyan",
            AnsiColor::White => "White",
            AnsiColor::BrightBlack => "Bright Black",
            AnsiColor::BrightRed => "Bright Red",
            AnsiColor::BrightGreen => "Bright Green",
            AnsiColor::BrightYellow => "Bright Yellow",
            AnsiColor::BrightBlue => "Bright Blue",
            AnsiColor::BrightMagenta => "Bright Magenta",
            AnsiColor::BrightCyan => "Bright Cyan",
            AnsiColor::BrightWhite => "Bright White",
        }
    }

    pub fn all() -> Vec<AnsiColor> {
        vec![
            AnsiColor::Black,
            AnsiColor::Red,
            AnsiColor::Green,
            AnsiColor::Yellow,
            AnsiColor::Blue,
            AnsiColor::Magenta,
            AnsiColor::Cyan,
            AnsiColor::White,
            AnsiColor::BrightBlack,
            AnsiColor::BrightRed,
            AnsiColor::BrightGreen,
            AnsiColor::BrightYellow,
            AnsiColor::BrightBlue,
            AnsiColor::BrightMagenta,
            AnsiColor::BrightCyan,
            AnsiColor::BrightWhite,
        ]
    }

    pub fn from_string(s: &str) -> Option<AnsiColor> {
        match s.to_lowercase().as_str() {
            "black" => Some(AnsiColor::Black),
            "red" => Some(AnsiColor::Red),
            "green" => Some(AnsiColor::Green),
            "yellow" => Some(AnsiColor::Yellow),
            "blue" => Some(AnsiColor::Blue),
            "magenta" => Some(AnsiColor::Magenta),
            "cyan" => Some(AnsiColor::Cyan),
            "white" => Some(AnsiColor::White),
            "bright-black" => Some(AnsiColor::BrightBlack),
            "bright-red" => Some(AnsiColor::BrightRed),
            "bright-green" => Some(AnsiColor::BrightGreen),
            "bright-yellow" => Some(AnsiColor::BrightYellow),
            "bright-blue" => Some(AnsiColor::BrightBlue),
            "bright-magenta" => Some(AnsiColor::BrightMagenta),
            "bright-cyan" => Some(AnsiColor::BrightCyan),
            "bright-white" => Some(AnsiColor::BrightWhite),
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match Self::from_string(s) {
            Some(color) => Ok(color),
            None => Err(anyhow::anyhow!("Invalid color name: {}", s)),
        }
    }
}

impl Default for AnsiColor {
    fn default() -> Self {
        AnsiColor::White
    }
}

/// Color configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub background: AnsiColor,
    pub border: AnsiColor,
    pub text: AnsiColor,
    pub user_name: AnsiColor,
    pub assistant_name: AnsiColor,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            background: AnsiColor::Black,
            border: AnsiColor::White,
            text: AnsiColor::White,
            user_name: AnsiColor::BrightBlue,
            assistant_name: AnsiColor::BrightGreen,
        }
    }
}

impl ColorConfig {
    pub fn from_args(args: &Args) -> anyhow::Result<Self> {
        Ok(Self {
            background: AnsiColor::from_str(&args.background_color)?,
            border: AnsiColor::from_str(&args.border_color)?,
            text: AnsiColor::from_str(&args.text_color)?,
            user_name: AnsiColor::from_str(&args.user_name_color)?,
            assistant_name: AnsiColor::from_str(&args.assistant_name_color)?,
        })
    }
}

// Helper function for tests
#[cfg(test)]
pub fn default_test_colors() -> ColorConfig {
    ColorConfig::default()
}

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

    /// Background color (default: black)
    #[arg(long, default_value = "black")]
    pub background_color: String,

    /// Border color (default: white)
    #[arg(long, default_value = "white")]
    pub border_color: String,

    /// Text color (default: white)
    #[arg(long, default_value = "white")]
    pub text_color: String,

    /// User name color (default: bright-blue)
    #[arg(long, default_value = "bright-blue")]
    pub user_name_color: String,

    /// Assistant name color (default: bright-green)
    #[arg(long, default_value = "bright-green")]
    pub assistant_name_color: String,
}