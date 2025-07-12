// Test for color configuration functionality
#[cfg(test)]
mod color_tests {
    use crate::config::{AnsiColor, ColorConfig};

    #[test]
    fn test_ansi_color_from_string() {
        assert_eq!(AnsiColor::from_str("black").unwrap(), AnsiColor::Black);
        assert_eq!(AnsiColor::from_str("red").unwrap(), AnsiColor::Red);
        assert_eq!(AnsiColor::from_str("bright-blue").unwrap(), AnsiColor::BrightBlue);
        assert_eq!(AnsiColor::from_str("bright-white").unwrap(), AnsiColor::BrightWhite);
        
        // Test invalid color
        assert!(AnsiColor::from_str("invalid-color").is_err());
    }

    #[test]
    fn test_color_config_default() {
        let config = ColorConfig::default();
        assert_eq!(config.background, AnsiColor::Black);
        assert_eq!(config.border, AnsiColor::White);
        assert_eq!(config.text, AnsiColor::White);
        assert_eq!(config.user_name, AnsiColor::BrightBlue);
        assert_eq!(config.assistant_name, AnsiColor::BrightGreen);
    }

    #[test]
    fn test_ansi_color_to_ratatui() {
        use ratatui::style::Color;
        
        assert_eq!(AnsiColor::Black.to_ratatui_color(), Color::Black);
        assert_eq!(AnsiColor::Red.to_ratatui_color(), Color::Red);
        assert_eq!(AnsiColor::BrightBlue.to_ratatui_color(), Color::LightBlue);
        assert_eq!(AnsiColor::BrightWhite.to_ratatui_color(), Color::Gray);
    }

    #[test]
    fn test_color_config_from_args() {
        use crate::config::Args;
        use clap::Parser;
        
        let args = Args::parse_from(&[
            "claudecli",
            "--api-key", "test-key",
            "--background-color", "blue",
            "--border-color", "bright-white",
            "--text-color", "green",
            "--user-name-color", "bright-yellow",
            "--assistant-name-color", "bright-cyan"
        ]);
        
        let config = ColorConfig::from_args(&args).unwrap();
        assert_eq!(config.background, AnsiColor::Blue);
        assert_eq!(config.border, AnsiColor::BrightWhite);
        assert_eq!(config.text, AnsiColor::Green);
        assert_eq!(config.user_name, AnsiColor::BrightYellow);
        assert_eq!(config.assistant_name, AnsiColor::BrightCyan);
    }
}
