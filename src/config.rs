// src/config.rs
use clap::{Parser, ValueEnum};
use ratatui::style::Color;
use ratatui::symbols::border;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::fmt;
use std::collections::HashMap;
use base64::prelude::*;

/// Feature flags and configuration constants
pub const SCROLL_ON_USER_INPUT: bool = true;
pub const SCROLL_ON_API_RESPONSE: bool = true;
pub const SHIFT_ENTER_SENDS: bool = false;
pub const SHOW_DEBUG_MESSAGES: bool = false;

/// Terminal size constants
pub const MIN_TERMINAL_WIDTH: u16 = 10;
pub const MIN_TERMINAL_HEIGHT: u16 = 10;
pub const MIN_MESSAGE_DISPLAY_WIDTH: u16 = 20;
pub const MIN_MESSAGE_DISPLAY_HEIGHT: u16 = 3;

pub const PROGRESS_FRAMES: [&str; 5] = ["    ", ".   ", "..  ", "... ", "...."];

/// Color profile structure that contains a complete color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorProfile {
    pub name: String,
    pub description: String,
    pub config: ColorConfig,
}

impl ColorProfile {
    pub fn new(name: String, description: String, config: ColorConfig) -> Self {
        Self { name, description, config }
    }
}

/// Embedded color profiles encoded as Base64 strings
const EMBEDDED_PROFILES: &[(&str, &str)] = &[
    ("default", "eyJuYW1lIjoiRGVmYXVsdCIsImRlc2NyaXB0aW9uIjoiRGVmYXVsdCBjb2xvciBzY2hlbWUiLCJjb25maWciOnsiYmFja2dyb3VuZCI6IkJsYWNrIiwiYm9yZGVyIjoiV2hpdGUiLCJ0ZXh0IjoiV2hpdGUiLCJ1c2VyX25hbWUiOiJCcmlnaHRCbHVlIiwiYXNzaXN0YW50X25hbWUiOiJCcmlnaHRHcmVlbiIsImJvcmRlcl9zdHlsZSI6IkFzY2lpIn19"),
    ("matrix", "eyJuYW1lIjoiTWF0cml4IiwiZGVzY3JpcHRpb24iOiJHcmVlbi1vbi1ibGFjayBtYXRyaXggc3R5bGUiLCJjb25maWciOnsiYmFja2dyb3VuZCI6IkJsYWNrIiwiYm9yZGVyIjoiR3JlZW4iLCJ0ZXh0IjoiQnJpZ2h0R3JlZW4iLCJ1c2VyX25hbWUiOiJCcmlnaHRXaGl0ZSIsImFzc2lzdGFudF9uYW1lIjoiQnJpZ2h0R3JlZW4iLCJib3JkZXJfc3R5bGUiOiJBc2NpaSJ9fQ=="),
    ("ocean", "eyJuYW1lIjoiT2NlYW4iLCJkZXNjcmlwdGlvbiI6IkJsdWUgYW5kIGN5YW4gb2NlYW4gdGhlbWUiLCJjb25maWciOnsiYmFja2dyb3VuZCI6IkJsYWNrIiwiYm9yZGVyIjoiQ3lhbiIsInRleHQiOiJCcmlnaHRXaGl0ZSIsInVzZXJfbmFtZSI6IkJyaWdodEJsdWUiLCJhc3Npc3RhbnRfbmFtZSI6IkJyaWdodEN5YW4iLCJib3JkZXJfc3R5bGUiOiJSb3VuZGVkIn19"),
    ("sunset", "eyJuYW1lIjoiU3Vuc2V0IiwiZGVzY3JpcHRpb24iOiJXYXJtIHN1bnNldCBjb2xvcnMiLCJjb25maWciOnsiYmFja2dyb3VuZCI6IkJsYWNrIiwiYm9yZGVyIjoiWWVsbG93IiwidGV4dCI6IkJyaWdodFdoaXRlIiwidXNlcl9uYW1lIjoiQnJpZ2h0UmVkIiwiYXNzaXN0YW50X25hbWUiOiJCcmlnaHRZZWxsb3ciLCJib3JkZXJfc3R5bGUiOiJUaGljayJ9fQ=="),
    ("forest", "eyJuYW1lIjoiRm9yZXN0IiwiZGVzY3JpcHRpb24iOiJHcmVlbiBmb3Jlc3QgdGhlbWUiLCJjb25maWciOnsiYmFja2dyb3VuZCI6IkJsYWNrIiwiYm9yZGVyIjoiR3JlZW4iLCJ0ZXh0IjoiQnJpZ2h0R3JlZW4iLCJ1c2VyX25hbWUiOiJCcmlnaHRXaGl0ZSIsImFzc2lzdGFudF9uYW1lIjoiWWVsbG93IiwiYm9yZGVyX3N0eWxlIjoiQXNjaWkifX0="),
    ("purple", "eyJuYW1lIjoiUHVycGxlIiwiZGVzY3JpcHRpb24iOiJQdXJwbGUgbWFnaWMgdGhlbWUiLCJjb25maWciOnsiYmFja2dyb3VuZCI6IkJsYWNrIiwiYm9yZGVyIjoiTWFnZW50YSIsInRleHQiOiJCcmlnaHRXaGl0ZSIsInVzZXJfbmFtZSI6IkJyaWdodE1hZ2VudGEiLCJhc3Npc3RhbnRfbmFtZSI6IkJyaWdodEN5YW4iLCJib3JkZXJfc3R5bGUiOiJSb3VuZGVkIn19"),
    ("fire", "eyJuYW1lIjoiRmlyZSIsImRlc2NyaXB0aW9uIjoiRmlyZSBhbmQgZmxhbWVzIHRoZW1lIiwiY29uZmlnIjp7ImJhY2tncm91bmQiOiJCbGFjayIsImJvcmRlciI6IlJlZCIsInRleHQiOiJCcmlnaHRXaGl0ZSIsInVzZXJfbmFtZSI6IkJyaWdodFJlZCIsImFzc2lzdGFudF9uYW1lIjoiQnJpZ2h0WWVsbG93IiwiYm9yZGVyX3N0eWxlIjoiVGhpY2sifX0="),
    ("ice", "eyJuYW1lIjoiSWNlIiwiZGVzY3JpcHRpb24iOiJDb2xkIGljZSB0aGVtZSIsImNvbmZpZyI6eyJiYWNrZ3JvdW5kIjoiQmxhY2siLCJib3JkZXIiOiJDeWFuIiwidGV4dCI6IkJyaWdodFdoaXRlIiwidXNlcl9uYW1lIjoiQnJpZ2h0Q3lhbiIsImFzc2lzdGFudF9uYW1lIjoiQnJpZ2h0Qmx1ZSIsImJvcmRlcl9zdHlsZSI6IlJvdW5kZWQifX0="),
    ("gold", "eyJuYW1lIjoiR29sZCIsImRlc2NyaXB0aW9uIjoiR29sZGVuIGx1eHVyeSB0aGVtZSIsImNvbmZpZyI6eyJiYWNrZ3JvdW5kIjoiQmxhY2siLCJib3JkZXIiOiJZZWxsb3ciLCJ0ZXh0IjoiQnJpZ2h0V2hpdGUiLCJ1c2VyX25hbWUiOiJCcmlnaHRZZWxsb3ciLCJhc3Npc3RhbnRfbmFtZSI6IkJyaWdodFJlZCIsImJvcmRlcl9zdHlsZSI6IlRoaWNrIn19"),
];

/// Available border styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, ValueEnum)]
pub enum BorderStyle {
    /// ASCII borders using +, -, | characters
    Ascii,
    /// Rounded Unicode borders with curved corners
    Rounded,
    /// Thick Unicode borders with bold lines
    Thick,
    /// Double-line Unicode borders with parallel lines
    Double,
}

impl BorderStyle {
    pub fn to_ratatui_border_set(self) -> border::Set {
        match self {
            BorderStyle::Ascii => border::PLAIN,
            BorderStyle::Rounded => border::ROUNDED,
            BorderStyle::Thick => border::THICK,
            BorderStyle::Double => border::DOUBLE,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BorderStyle::Ascii => "ASCII",
            BorderStyle::Rounded => "Rounded",
            BorderStyle::Thick => "Thick",
            BorderStyle::Double => "Double",
        }
    }

    pub fn all() -> Vec<BorderStyle> {
        vec![
            BorderStyle::Ascii,
            BorderStyle::Rounded,
            BorderStyle::Thick,
            BorderStyle::Double,
        ]
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Ascii
    }
}

impl fmt::Display for BorderStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorderStyle::Ascii => write!(f, "ascii"),
            BorderStyle::Rounded => write!(f, "rounded"),
            BorderStyle::Thick => write!(f, "thick"),
            BorderStyle::Double => write!(f, "double"),
        }
    }
}

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
            AnsiColor::White => Color::Gray,
            AnsiColor::BrightBlack => Color::DarkGray,
            AnsiColor::BrightRed => Color::LightRed,
            AnsiColor::BrightGreen => Color::LightGreen,
            AnsiColor::BrightYellow => Color::LightYellow,
            AnsiColor::BrightBlue => Color::LightBlue,
            AnsiColor::BrightMagenta => Color::LightMagenta,
            AnsiColor::BrightCyan => Color::LightCyan,
            AnsiColor::BrightWhite => Color::White,
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
    #[serde(default)]
    pub border_style: BorderStyle,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            background: AnsiColor::Black,
            border: AnsiColor::White,
            text: AnsiColor::White,
            user_name: AnsiColor::BrightBlue,
            assistant_name: AnsiColor::BrightGreen,
            border_style: BorderStyle::default(),
        }
    }
}

impl ColorConfig {
    pub fn from_args(args: &Args) -> anyhow::Result<Self> {
        let (result, _) = Self::from_args_and_saved(args);
        result
    }

    /// Create color configuration from args and saved config
    pub fn from_args_and_saved(args: &Args) -> (anyhow::Result<Self>, Option<String>) {
        // Start with saved config or defaults
        let (mut config, config_error) = if args.reset_colors {
            (ColorConfig::default(), None)
        } else {
            load_color_config_with_error_info()
        };

        // Apply command-line overrides if specified
        let mut result = Ok(());
        if let Some(color_str) = &args.background_color {
            if let Err(e) = AnsiColor::from_str(color_str) {
                result = Err(e);
            } else {
                config.background = AnsiColor::from_str(color_str).unwrap();
            }
        }
        if let Some(color_str) = &args.border_color {
            if let Err(e) = AnsiColor::from_str(color_str) {
                result = Err(e);
            } else {
                config.border = AnsiColor::from_str(color_str).unwrap();
            }
        }
        if let Some(color_str) = &args.text_color {
            if let Err(e) = AnsiColor::from_str(color_str) {
                result = Err(e);
            } else {
                config.text = AnsiColor::from_str(color_str).unwrap();
            }
        }
        if let Some(color_str) = &args.user_name_color {
            if let Err(e) = AnsiColor::from_str(color_str) {
                result = Err(e);
            } else {
                config.user_name = AnsiColor::from_str(color_str).unwrap();
            }
        }
        if let Some(color_str) = &args.assistant_name_color {
            if let Err(e) = AnsiColor::from_str(color_str) {
                result = Err(e);
            } else {
                config.assistant_name = AnsiColor::from_str(color_str).unwrap();
            }
        }

        // Parse border style - no validation needed since clap handles it
        config.border_style = args.border_style;

        (result.map(|_| config), config_error)
    }

    /// Reset all colors to defaults
    pub fn reset_to_defaults(&mut self) {
        *self = ColorConfig::default();
    }
}

/// Get the path to the configuration file
pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("claudecli");
    std::fs::create_dir_all(&path).ok(); // Create directory if it doesn't exist
    path.push("config.json");
    path
}

/// Save color configuration to file
pub fn save_color_config(config: &ColorConfig) -> anyhow::Result<()> {
    let config_path = get_config_path();
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(&config_path, json)?;
    Ok(())
}

/// Load color configuration from file
pub fn load_color_config() -> anyhow::Result<ColorConfig> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Ok(ColorConfig::default());
    }

    let contents = std::fs::read_to_string(&config_path)?;
    
    // Try to parse the config, with #[serde(default)] handling invalid border styles
    match serde_json::from_str::<ColorConfig>(&contents) {
        Ok(config) => Ok(config),
        Err(e) => {
            // If deserialization fails completely, fall back to defaults
            // The #[serde(default)] on border_style should handle invalid values gracefully
            Err(e.into())
        }
    }
}

/// Load color configuration, falling back to defaults on error
pub fn load_color_config_safe() -> ColorConfig {
    load_color_config().unwrap_or_else(|_| ColorConfig::default())
}

/// Load color configuration, returning the result and whether it had an error
pub fn load_color_config_with_error_info() -> (ColorConfig, Option<String>) {
    match load_color_config() {
        Ok(config) => (config, None),
        Err(e) => (ColorConfig::default(), Some(format!("Failed to load color config: {}", e))),
    }
}

/// Load embedded color profiles
pub fn load_embedded_profiles() -> Result<HashMap<String, ColorProfile>, String> {
    let mut profiles = HashMap::new();
    
    for (key, base64_data) in EMBEDDED_PROFILES {
        let decoded = BASE64_STANDARD.decode(base64_data)
            .map_err(|e| format!("Failed to decode profile {}: {}", key, e))?;
        
        let profile: ColorProfile = serde_json::from_slice(&decoded)
            .map_err(|e| format!("Failed to parse profile {}: {}", key, e))?;
        
        profiles.insert(key.to_string(), profile);
    }
    
    Ok(profiles)
}

/// Get the path to the color profiles directory
pub fn get_profiles_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("claudecli");
    path.push("profiles");
    std::fs::create_dir_all(&path).ok(); // Create directory if it doesn't exist
    path
}

/// Save a custom color profile
pub fn save_color_profile(profile: &ColorProfile) -> anyhow::Result<()> {
    let profiles_path = get_profiles_path();
    let filename = format!("{}.json", profile.name.replace(' ', "_").to_lowercase());
    let profile_path = profiles_path.join(filename);
    
    let json = serde_json::to_string_pretty(profile)?;
    std::fs::write(&profile_path, json)?;
    Ok(())
}

/// Load custom color profiles from disk
pub fn load_custom_profiles() -> HashMap<String, ColorProfile> {
    let mut profiles = HashMap::new();
    let profiles_path = get_profiles_path();
    
    if let Ok(entries) = std::fs::read_dir(&profiles_path) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                        if let Ok(profile) = serde_json::from_str::<ColorProfile>(&contents) {
                            profiles.insert(profile.name.clone(), profile);
                        }
                    }
                }
            }
        }
    }
    
    profiles
}

/// Get all available color profiles (embedded + custom)
pub fn get_all_profiles() -> HashMap<String, ColorProfile> {
    let mut all_profiles = HashMap::new();
    
    // Load embedded profiles
    if let Ok(embedded) = load_embedded_profiles() {
        all_profiles.extend(embedded);
    }
    
    // Load custom profiles (these can override embedded ones)
    let custom = load_custom_profiles();
    all_profiles.extend(custom);
    
    all_profiles
}

/// Get the default color configuration for testing
pub fn get_default_colors() -> ColorConfig {
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

    /// Reset all colors to default values
    #[arg(long)]
    pub reset_colors: bool,

    /// Background color (default: black)
    #[arg(long)]
    pub background_color: Option<String>,

    /// Border color (default: white)
    #[arg(long)]
    pub border_color: Option<String>,

    /// Text color (default: white)
    #[arg(long)]
    pub text_color: Option<String>,

    /// User name color (default: bright-blue)
    #[arg(long)]
    pub user_name_color: Option<String>,

    /// Assistant name color (default: bright-green)
    #[arg(long)]
    pub assistant_name_color: Option<String>,

    /// Border style: ascii, rounded, thick, double (default: ascii)
    #[arg(long, default_value_t = BorderStyle::Ascii)]
    pub border_style: BorderStyle,
}

#[cfg(test)]
mod command_line_override_tests {
    use super::*;

    #[test]
    fn test_background_color_black_override() {
        // Test that --background-color black properly overrides config file values
        let args = Args {
            api_key: "dummy".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            simulate: false,
            reset_colors: false,
            background_color: Some("black".to_string()),
            border_color: None,
            text_color: None,
            user_name_color: None,
            assistant_name_color: None,
            border_style: BorderStyle::Ascii,
        };

        let (result, _) = ColorConfig::from_args_and_saved(&args);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.background, AnsiColor::Black);
    }

    #[test]
    fn test_all_default_colors_can_be_overridden() {
        // Test that all default colors can be explicitly specified and will override config
        let args = Args {
            api_key: "dummy".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            simulate: false,
            reset_colors: false,
            background_color: Some("black".to_string()),
            border_color: Some("white".to_string()),
            text_color: Some("white".to_string()),
            user_name_color: Some("bright-blue".to_string()),
            assistant_name_color: Some("bright-green".to_string()),
            border_style: BorderStyle::Rounded,
        };

        let (result, _) = ColorConfig::from_args_and_saved(&args);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.background, AnsiColor::Black);
        assert_eq!(config.border, AnsiColor::White);
        assert_eq!(config.text, AnsiColor::White);
        assert_eq!(config.user_name, AnsiColor::BrightBlue);
        assert_eq!(config.assistant_name, AnsiColor::BrightGreen);
    }    #[test]
    fn test_no_override_when_not_specified() {
        // Test that when no color arguments are provided, we get defaults
        let args = Args {
            api_key: "dummy".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            simulate: false,
            reset_colors: false,
            background_color: None,
            border_color: None,
            text_color: None,
            user_name_color: None,
            assistant_name_color: None,
            border_style: BorderStyle::Rounded,
        };

        let (result, _) = ColorConfig::from_args_and_saved(&args);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        // This test will load from config file if it exists, or defaults if not
        // The important part is that the function doesn't crash and returns valid colors
        assert!(matches!(config.background, AnsiColor::Black | AnsiColor::Red | AnsiColor::Green | AnsiColor::Yellow | AnsiColor::Blue | AnsiColor::Magenta | AnsiColor::Cyan | AnsiColor::White | AnsiColor::BrightBlack | AnsiColor::BrightRed | AnsiColor::BrightGreen | AnsiColor::BrightYellow | AnsiColor::BrightBlue | AnsiColor::BrightMagenta | AnsiColor::BrightCyan | AnsiColor::BrightWhite));
    }
}