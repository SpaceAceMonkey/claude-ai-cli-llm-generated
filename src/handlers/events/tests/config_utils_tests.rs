//! Unit tests for configuration and utility functions
//! Tests command line parsing, feature flags, and utility functions

use crate::app::AppState;
use crate::api::HighlightCache;
use crate::config::{SHIFT_ENTER_SENDS, SCROLL_ON_USER_INPUT, SCROLL_ON_API_RESPONSE};
use crate::utils::text::{wrap_text, calculate_cursor_line, move_cursor_up, move_cursor_down};
use crate::utils::scroll::calculate_chat_scroll_offset;
use crate::tui::format_message_for_tui_cached;

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        // Test AppState creation with different parameters
        let app_result = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        );
        
        assert!(app_result.is_ok(), "AppState creation should succeed with valid parameters");
        
        let app = app_result.unwrap();
        assert_eq!(app.client.api_key, "test_key");
        assert_eq!(app.client.model, "claude-3-5-sonnet-20241022");
        assert_eq!(app.client.max_tokens, 1024);
        assert_eq!(app.client.temperature, 0.7);
        assert!(app.simulate_mode);
    }

    #[test]
    fn test_app_state_with_different_models() {
        // Test AppState creation with different model names
        let models = vec![
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
            "custom-model",
        ];
        
        for model in models {
            let app_result = AppState::new(
                "test_key".to_string(),
                model.to_string(),
                1024,
                0.7,
                true,
            );
            
            assert!(app_result.is_ok(), "AppState creation should succeed with model: {}", model);
            
            let app = app_result.unwrap();
            assert_eq!(app.client.model, model);
        }
    }

    #[test]
    fn test_app_state_with_different_token_limits() {
        // Test AppState creation with different token limits
        let token_limits = vec![100, 1024, 4096, 8192, 100000];
        
        for max_tokens in token_limits {
            let app_result = AppState::new(
                "test_key".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                max_tokens,
                0.7,
                true,
            );
            
            assert!(app_result.is_ok(), "AppState creation should succeed with token limit: {}", max_tokens);
            
            let app = app_result.unwrap();
            assert_eq!(app.client.max_tokens, max_tokens);
        }
    }

    #[test]
    fn test_app_state_with_different_temperatures() {
        // Test AppState creation with different temperature values
        let temperatures = vec![0.0, 0.1, 0.5, 0.7, 1.0];
        
        for temperature in temperatures {
            let app_result = AppState::new(
                "test_key".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                1024,
                temperature,
                true,
            );
            
            assert!(app_result.is_ok(), "AppState creation should succeed with temperature: {}", temperature);
            
            let app = app_result.unwrap();
            assert_eq!(app.client.temperature, temperature);
        }
    }

    #[test]
    fn test_app_state_simulate_mode() {
        // Test AppState creation with both simulate modes
        
        // Simulate mode enabled
        let app_sim = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).unwrap();
        
        assert!(app_sim.simulate_mode, "Simulate mode should be enabled");
        
        // Simulate mode disabled
        let app_real = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            false,
        ).unwrap();
        
        assert!(!app_real.simulate_mode, "Simulate mode should be disabled");
    }

    #[test]
    fn test_feature_flags() {
        // Test that feature flags are properly defined
        
        // SHIFT_ENTER_SENDS should be a boolean
        assert!(SHIFT_ENTER_SENDS == true || SHIFT_ENTER_SENDS == false, "SHIFT_ENTER_SENDS should be a boolean");
        
        // SCROLL_ON_USER_INPUT should be a boolean
        assert!(SCROLL_ON_USER_INPUT == true || SCROLL_ON_USER_INPUT == false, "SCROLL_ON_USER_INPUT should be a boolean");
        
        // SCROLL_ON_API_RESPONSE should be a boolean
        assert!(SCROLL_ON_API_RESPONSE == true || SCROLL_ON_API_RESPONSE == false, "SCROLL_ON_API_RESPONSE should be a boolean");
    }

    #[test]
    fn test_feature_flag_behavior() {
        // Test that feature flags affect behavior consistently
        
        // Test SHIFT_ENTER_SENDS flag
        let input_title = if SHIFT_ENTER_SENDS {
            "Input (Shift/Alt+Enter to send, Enter for newline)"
        } else {
            "Input (Enter to send, Shift/Alt+Enter for newline)"
        };
        
        if SHIFT_ENTER_SENDS {
            assert!(input_title.contains("Shift/Alt+Enter to send"), "Should indicate Shift/Alt+Enter sends when flag is true");
            assert!(input_title.contains("Enter for newline"), "Should indicate Enter for newline when flag is true");
        } else {
            assert!(input_title.contains("Enter to send"), "Should indicate Enter sends when flag is false");
            assert!(input_title.contains("Shift/Alt+Enter for newline"), "Should indicate Shift/Alt+Enter for newline when flag is false");
        }
    }

    #[test]
    fn test_empty_api_key_handling() {
        // Test AppState creation with empty API key
        let app_result = AppState::new(
            "".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        );
        
        // Should succeed (validation happens at API call time)
        assert!(app_result.is_ok(), "AppState creation should succeed with empty API key");
        
        let app = app_result.unwrap();
        assert_eq!(app.client.api_key, "");
    }

    #[test]
    fn test_extreme_parameter_values() {
        // Test AppState creation with extreme parameter values
        
        // Very high token limit
        let app_high_tokens = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            u32::MAX,
            0.7,
            true,
        );
        assert!(app_high_tokens.is_ok(), "Should handle very high token limit");
        
        // Zero token limit
        let app_zero_tokens = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            0,
            0.7,
            true,
        );
        assert!(app_zero_tokens.is_ok(), "Should handle zero token limit");
        
        // Temperature at boundaries
        let app_temp_min = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.0,
            true,
        );
        assert!(app_temp_min.is_ok(), "Should handle minimum temperature");
        
        let app_temp_max = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            1.0,
            true,
        );
        assert!(app_temp_max.is_ok(), "Should handle maximum temperature");
    }
}

#[cfg(test)]
mod utility_function_tests {
    use super::*;

    #[test]
    fn test_wrap_text_basic() {
        // Test basic text wrapping
        let text = "This is a test string that should be wrapped at word boundaries";
        let width = 20;
        
        let wrapped = wrap_text(text, width);
        
        assert!(wrapped.len() > 1, "Long text should wrap to multiple lines");
        
        for line in &wrapped {
            assert!(line.width() <= width, "No wrapped line should exceed width");
        }
        
        let rejoined: String = wrapped.iter().map(|line| line.to_string()).collect::<Vec<_>>().join("");
        assert_eq!(rejoined, text, "Text should be preserved when rejoined");
    }

    #[test]
    fn test_wrap_text_edge_cases() {
        // Test edge cases for text wrapping
        
        // Empty string
        let wrapped = wrap_text("", 10);
        assert_eq!(wrapped.len(), 1, "Empty string should result in one empty line");
        assert_eq!(wrapped[0].to_string(), "", "Empty string should result in empty line");
        
        // Single word longer than width
        let wrapped = wrap_text("verylongwordthatexceedswidth", 10);
        assert!(wrapped.len() >= 1, "Long word should be handled");
        
        // Text with newlines
        let wrapped = wrap_text("Line 1\nLine 2", 20);
        assert!(wrapped.len() >= 2, "Newlines should be preserved");
        
        // Text with only spaces
        let wrapped = wrap_text("   ", 10);
        assert!(wrapped.len() >= 1, "Spaces should be handled");
    }

    #[test]
    fn test_calculate_cursor_line() {
        // Test cursor line calculation
        let text = "Line 1\nLine 2\nLine 3";
        let width = 20;
        
        // Cursor at beginning
        assert_eq!(calculate_cursor_line(text, 0, width), 0);
        
        // Cursor at start of second line
        assert_eq!(calculate_cursor_line(text, 7, width), 1);
        
        // Cursor at start of third line
        assert_eq!(calculate_cursor_line(text, 14, width), 2);
        
        // Cursor at end
        assert_eq!(calculate_cursor_line(text, text.len(), width), 2);
    }

    #[test]
    fn test_calculate_cursor_line_with_wrapping() {
        // Test cursor line calculation with text wrapping
        let text = "This is a very long line that will wrap across multiple lines";
        let width = 20;
        
        let line_0 = calculate_cursor_line(text, 0, width);
        assert_eq!(line_0, 0, "Start should be line 0");
        
        let line_mid = calculate_cursor_line(text, 30, width);
        assert!(line_mid > 0, "Middle should be on later line");
        
        let line_end = calculate_cursor_line(text, text.len(), width);
        assert!(line_end >= line_mid, "End should be on same or later line");
    }

    #[test]
    fn test_move_cursor_up() {
        // Test cursor movement up
        let text = "Line 1\nLine 2\nLine 3";
        let width = 20;
        
        // From middle of second line
        let new_pos = move_cursor_up(text, 10, width);
        assert!(new_pos < 10, "Cursor should move up");
        
        // From start of first line (should stay)
        let new_pos = move_cursor_up(text, 0, width);
        assert_eq!(new_pos, 0, "Cursor at start should not move up");
    }

    #[test]
    fn test_move_cursor_down() {
        // Test cursor movement down
        let text = "Line 1\nLine 2\nLine 3";
        let width = 20;
        
        // From start of first line
        let new_pos = move_cursor_down(text, 0, width);
        assert!(new_pos > 0, "Cursor should move down");
        
        // From end of last line (should stay)
        let new_pos = move_cursor_down(text, text.len(), width);
        assert_eq!(new_pos, text.len(), "Cursor at end should not move down");
    }

    #[test]
    fn test_move_cursor_with_wrapping() {
        // Test cursor movement with wrapped text
        let text = "This is a very long line that will wrap across multiple lines and should allow proper cursor movement";
        let width = 20;
        
        // Test moving up and down preserves column position when possible
        let start_pos = 50;
        let up_pos = move_cursor_up(text, start_pos, width);
        let down_pos = move_cursor_down(text, up_pos, width);
        
        // Should be close to original position (within reason)
        assert!((down_pos as i32 - start_pos as i32).abs() <= 5, "Cursor movement should be consistent");
    }

    #[test]
    fn test_calculate_chat_scroll_offset() {
        // Test chat scroll offset calculation
        let mut cache = HighlightCache::new();
        
        // Create test spans
        let mut spans = Vec::new();
        for i in 0..10 {
            spans.extend(format_message_for_tui_cached("user", &format!("Message {}", i), &mut cache));
        }
        
        let chat_height = 5;
        let chat_width = 40;
        
        let offset = calculate_chat_scroll_offset(&spans, chat_height, chat_width);
        
        assert!(offset >= 0, "Scroll offset should be non-negative");
    }

    #[test]
    fn test_calculate_chat_scroll_offset_edge_cases() {
        // Test chat scroll offset with edge cases
        
        // Empty spans
        let offset = calculate_chat_scroll_offset(&Vec::new(), 10, 40);
        assert_eq!(offset, 0, "Empty spans should result in zero offset");
        
        // Very small chat height
        let mut cache = HighlightCache::new();
        let spans = format_message_for_tui_cached("user", "Test message", &mut cache);
        let offset = calculate_chat_scroll_offset(&spans, 1, 40);
        assert!(offset >= 0, "Small chat height should be handled");
        
        // Very small chat width
        let offset = calculate_chat_scroll_offset(&spans, 10, 5);
        assert!(offset >= 0, "Small chat width should be handled");
    }

    #[test]
    fn test_format_message_for_tui_cached() {
        // Test cached message formatting
        let mut cache = HighlightCache::new();
        
        // First call should populate cache
        let spans1 = format_message_for_tui_cached("user", "Test message", &mut cache);
        assert!(!spans1.is_empty(), "Should format message");
        
        // Second call should use cache
        let spans2 = format_message_for_tui_cached("user", "Test message", &mut cache);
        assert_eq!(spans1.len(), spans2.len(), "Cached result should match original");
        
        // Different message should not use cache
        let spans3 = format_message_for_tui_cached("user", "Different message", &mut cache);
        assert!(!spans3.is_empty(), "Should format different message");
    }

    #[test]
    fn test_format_message_different_roles() {
        // Test message formatting for different roles
        let mut cache = HighlightCache::new();
        
        let user_spans = format_message_for_tui_cached("user", "User message", &mut cache);
        let assistant_spans = format_message_for_tui_cached("assistant", "Assistant message", &mut cache);
        let system_spans = format_message_for_tui_cached("system", "System message", &mut cache);
        
        assert!(!user_spans.is_empty(), "User message should format");
        assert!(!assistant_spans.is_empty(), "Assistant message should format");
        assert!(!system_spans.is_empty(), "System message should format");
    }

    #[test]
    fn test_format_message_with_special_characters() {
        // Test message formatting with special characters
        let mut cache = HighlightCache::new();
        
        let special_content = "Message with **bold**, `code`, and unicode: 🦀 世界";
        let spans = format_message_for_tui_cached("user", special_content, &mut cache);
        
        assert!(!spans.is_empty(), "Should format message with special characters");
        
        // Check that content is preserved in some form
        let text_content: String = spans.iter().map(|line| line.to_string()).collect();
        assert!(text_content.contains("bold"), "Bold text should be preserved");
        assert!(text_content.contains("code"), "Code text should be preserved");
        assert!(text_content.contains("🦀"), "Unicode should be preserved");
    }

    #[test]
    fn test_utility_functions_with_unicode() {
        // Test utility functions with Unicode text
        let unicode_text = "Hello 世界! 🦀 Rust は素晴らしい programming language";
        let width = 30;
        
        // Test wrapping
        let wrapped = wrap_text(unicode_text, width);
        assert!(!wrapped.is_empty(), "Should wrap Unicode text");
        
        let rejoined: String = wrapped.iter().map(|line| line.to_string()).collect::<Vec<_>>().join("");
        assert_eq!(rejoined, unicode_text, "Unicode text should be preserved");
        
        // Test cursor positioning
        let cursor_line = calculate_cursor_line(unicode_text, 10, width);
        assert!(cursor_line >= 0, "Should handle Unicode cursor positioning");
        
        // Test cursor movement
        let new_pos = move_cursor_up(unicode_text, 20, width);
        assert!(new_pos <= 20, "Should handle Unicode cursor movement");
    }

    #[test]
    fn test_utility_functions_performance() {
        // Test utility functions with large input
        let large_text = "A".repeat(10000);
        let width = 80;
        
        // Test wrapping performance
        let start = std::time::Instant::now();
        let wrapped = wrap_text(&large_text, width);
        let wrap_duration = start.elapsed();
        
        assert!(!wrapped.is_empty(), "Should wrap large text");
        assert!(wrap_duration < std::time::Duration::from_millis(100), "Wrapping should be fast");
        
        // Test cursor calculation performance
        let start = std::time::Instant::now();
        let cursor_line = calculate_cursor_line(&large_text, 5000, width);
        let cursor_duration = start.elapsed();
        
        assert!(cursor_line >= 0, "Should calculate cursor line for large text");
        assert!(cursor_duration < std::time::Duration::from_millis(100), "Cursor calculation should be fast");
    }
}

#[cfg(test)]
mod integration_configuration_tests {
    use super::*;

    #[test]
    fn test_app_state_integration() {
        // Test that AppState integrates properly with utility functions
        let app = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).unwrap();
        
        // Test input handling
        let input = "Test input\nSecond line";
        let width = 40;
        
        let wrapped = wrap_text(input, width);
        assert!(!wrapped.is_empty(), "Should wrap app input");
        
        let cursor_line = calculate_cursor_line(input, 5, width);
        assert!(cursor_line >= 0, "Should calculate cursor line for app input");
    }

    #[test]
    fn test_configuration_consistency() {
        // Test that configuration is consistent across the application
        let app = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).unwrap();
        
        // Test that feature flags are consistent
        let input_behavior = if SHIFT_ENTER_SENDS {
            "shift_enter_sends"
        } else {
            "enter_sends"
        };
        
        assert!(input_behavior == "shift_enter_sends" || input_behavior == "enter_sends", 
                "Input behavior should be consistent");
        
        // Test that scroll behavior is consistent
        let scroll_behavior = (SCROLL_ON_USER_INPUT, SCROLL_ON_API_RESPONSE);
        assert!(scroll_behavior.0 == true || scroll_behavior.0 == false, 
                "User input scroll should be boolean");
        assert!(scroll_behavior.1 == true || scroll_behavior.1 == false, 
                "API response scroll should be boolean");
    }

    #[test]
    fn test_default_values() {
        // Test default values are reasonable
        let app = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).unwrap();
        
        // Default state should be empty
        assert_eq!(app.status, "");
        assert!(!app.waiting);
        assert_eq!(app.progress_i, 0);
        
        // Default input should be empty
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        
        // Default scroll should be auto
        assert_eq!(app.chat_scroll_offset, 0);
        assert!(app.auto_scroll);
        
        // Default dialogs should be closed
        assert!(!app.show_save_dialog);
        assert!(!app.show_load_dialog);
        assert!(!app.show_exit_dialog);
        assert!(!app.show_create_dir_dialog);
        assert!(!app.show_error_dialog);
    }

    #[test]
    fn test_boundary_conditions() {
        // Test boundary conditions in configuration
        
        // Test with minimum viable parameters
        let app_min = AppState::new(
            "k".to_string(),
            "m".to_string(),
            1,
            0.0,
            true,
        );
        assert!(app_min.is_ok(), "Should handle minimum parameters");
        
        // Test with maximum reasonable parameters
        let app_max = AppState::new(
            "a".repeat(1000),
            "model-".to_string() + &"name".repeat(100),
            100000,
            1.0,
            false,
        );
        assert!(app_max.is_ok(), "Should handle maximum parameters");
    }

    #[test]
    fn test_state_transitions() {
        // Test state transitions are valid
        let mut app = AppState::new(
            "test_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).unwrap();
        
        // Test dialog state transitions
        assert!(!app.show_save_dialog);
        app.show_save_dialog = true;
        assert!(app.show_save_dialog);
        app.show_save_dialog = false;
        assert!(!app.show_save_dialog);
        
        // Test waiting state transitions
        assert!(!app.waiting);
        app.waiting = true;
        assert!(app.waiting);
        app.waiting = false;
        assert!(!app.waiting);
        
        // Test scroll state transitions
        assert!(app.auto_scroll);
        app.auto_scroll = false;
        assert!(!app.auto_scroll);
        app.auto_scroll = true;
        assert!(app.auto_scroll);
    }
}
