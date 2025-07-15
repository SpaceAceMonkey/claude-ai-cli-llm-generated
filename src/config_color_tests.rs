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
        assert_eq!(AnsiColor::White.to_ratatui_color(), Color::Gray);
        assert_eq!(AnsiColor::BrightBlue.to_ratatui_color(), Color::LightBlue);
        assert_eq!(AnsiColor::BrightWhite.to_ratatui_color(), Color::White);
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

    #[test]
    fn test_color_profile_embedded_loading() {
        let profiles = crate::config::load_embedded_profiles().unwrap();
        
        // Check that all expected profiles are loaded
        assert!(profiles.contains_key("default"));
        assert!(profiles.contains_key("matrix"));
        assert!(profiles.contains_key("ocean"));
        assert!(profiles.contains_key("sunset"));
        
        // Check default profile structure
        let default_profile = &profiles["default"];
        assert_eq!(default_profile.name, "Default");
        assert_eq!(default_profile.description, "Default color scheme");
        assert_eq!(default_profile.config.background, AnsiColor::Black);
        assert_eq!(default_profile.config.user_name, AnsiColor::BrightBlue);
        assert_eq!(default_profile.config.assistant_name, AnsiColor::BrightGreen);
        
        // Check matrix profile structure
        let matrix_profile = &profiles["matrix"];
        assert_eq!(matrix_profile.name, "Matrix");
        assert_eq!(matrix_profile.description, "Green-on-black matrix style");
        assert_eq!(matrix_profile.config.background, AnsiColor::Black);
        assert_eq!(matrix_profile.config.border, AnsiColor::Green);
        assert_eq!(matrix_profile.config.text, AnsiColor::BrightGreen);
    }

    #[test]
    fn test_color_profile_creation() {
        let config = ColorConfig {
            background: AnsiColor::Black,
            border: AnsiColor::Red,
            text: AnsiColor::White,
            user_name: AnsiColor::BrightBlue,
            assistant_name: AnsiColor::BrightGreen,
            border_style: crate::config::BorderStyle::Rounded,
        };
        
        let profile = crate::config::ColorProfile::new(
            "Test Profile".to_string(),
            "A test color profile".to_string(),
            config.clone(),
        );
        
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.description, "A test color profile");
        assert_eq!(profile.config.background, config.background);
        assert_eq!(profile.config.border, config.border);
        assert_eq!(profile.config.text, config.text);
        assert_eq!(profile.config.user_name, config.user_name);
        assert_eq!(profile.config.assistant_name, config.assistant_name);
        assert_eq!(profile.config.border_style, config.border_style);
    }

    #[test]
    fn test_get_all_profiles() {
        let all_profiles = crate::config::get_all_profiles();
        
        // Should contain at least the embedded profiles
        assert!(all_profiles.len() >= 4);
        assert!(all_profiles.contains_key("default"));
        assert!(all_profiles.contains_key("matrix"));
        assert!(all_profiles.contains_key("ocean"));
        assert!(all_profiles.contains_key("sunset"));
    }
}
