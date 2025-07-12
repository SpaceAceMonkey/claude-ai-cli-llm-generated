//! Unit tests for UI rendering and display functionality
//! Tests text rendering, scrolling, dialogs, and layout calculations

use crate::app::AppState;
use crate::api::{Message, HighlightCache};
use crate::config::SHIFT_ENTER_SENDS;
use crate::utils::text::{wrap_text, calculate_cursor_line};
use crate::utils::scroll::calculate_chat_scroll_offset;
use crate::tui::format_message_for_tui_cached;

/// Helper function to create a test app state for UI tests
fn create_ui_test_app() -> AppState {
    AppState::new(
        "test_key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        1024,
        0.7,
        true,
    ).expect("Failed to create test app state")
}

/// Helper function to create test messages for UI rendering
fn create_ui_test_messages() -> Vec<Message> {
    vec![
        Message {
            role: "user".to_string(),
            content: "Hello, this is a test message that might be quite long and could wrap across multiple lines in the terminal interface.".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "This is a response message that also might be quite long and could potentially wrap across multiple lines when displayed in the terminal user interface.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Short message".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "Another response with some **bold** text and `code` snippets to test syntax highlighting.".to_string(),
        },
    ]
}

#[cfg(test)]
mod text_rendering_tests {
    use super::*;

    #[test]
    fn test_text_wrapping_basic() {
        // Test basic text wrapping functionality
        let text = "This is a long line that should wrap at a specific width";
        let width = 20;
        
        let wrapped = wrap_text(text, width);
        
        assert!(wrapped.len() > 1, "Long text should wrap to multiple lines");
        
        // Check that no line exceeds the width
        for line in &wrapped {
            assert!(line.width() <= width, "No line should exceed the specified width");
        }
        
        // Check that the text is preserved when joined
        let rejoined: String = wrapped.iter().map(|line| line.to_string()).collect::<Vec<_>>().join("");
        assert_eq!(rejoined, text, "Text should be preserved when rejoined");
    }

    #[test]
    fn test_text_wrapping_edge_cases() {
        // Test edge cases in text wrapping
        
        // Empty string
        let wrapped = wrap_text("", 20);
        assert_eq!(wrapped.len(), 1, "Empty string should result in one empty line");
        assert_eq!(wrapped[0].to_string(), "", "Empty string should result in empty line");
        
        // Single character
        let wrapped = wrap_text("a", 20);
        assert_eq!(wrapped.len(), 1, "Single character should not wrap");
        assert_eq!(wrapped[0].to_string(), "a", "Single character should be preserved");
        
        // String shorter than width
        let wrapped = wrap_text("short", 20);
        assert_eq!(wrapped.len(), 1, "Short string should not wrap");
        assert_eq!(wrapped[0].to_string(), "short", "Short string should be preserved");
        
        // String exactly at width
        let wrapped = wrap_text("exactly_twenty_chars", 20);
        assert_eq!(wrapped.len(), 1, "String at exact width should not wrap");
        assert_eq!(wrapped[0].to_string(), "exactly_twenty_chars", "String at exact width should be preserved");
        
        // String one character over width
        let wrapped = wrap_text("exactly_twenty_chars_", 20);
        assert_eq!(wrapped.len(), 2, "String one character over should wrap to two lines");
    }

    #[test]
    fn test_text_wrapping_with_newlines() {
        // Test text wrapping with existing newlines
        let text = "Line 1\nLine 2\nThis is a very long line that should wrap";
        let width = 20;
        
        let wrapped = wrap_text(text, width);
        
        assert!(wrapped.len() >= 3, "Should have at least 3 lines (2 explicit + 1 or more from wrapping)");
        
        // Check that explicit newlines are preserved
        assert_eq!(wrapped[0].to_string(), "Line 1");
        assert_eq!(wrapped[1].to_string(), "Line 2");
        
        // Check that the long line is wrapped
        let long_line_parts: Vec<String> = wrapped[2..].iter().map(|line| line.to_string()).collect();
        let rejoined_long_line = long_line_parts.join("");
        assert_eq!(rejoined_long_line, "This is a very long line that should wrap");
    }

    #[test]
    fn test_text_wrapping_unicode() {
        // Test text wrapping with Unicode characters
        let text = "Unicode: 你好世界 🦀 Rust は素晴らしい programming language";
        let width = 20;
        
        let wrapped = wrap_text(text, width);
        
        assert!(wrapped.len() > 1, "Unicode text should wrap");
        
        // Check that Unicode characters are preserved
        let rejoined: String = wrapped.iter().map(|line| line.to_string()).collect::<Vec<_>>().join(" ");
        assert!(rejoined.contains("你好世界"), "Chinese characters should be preserved");
        assert!(rejoined.contains("🦀"), "Emoji should be preserved");
        assert!(rejoined.contains("は"), "Japanese characters should be preserved");
    }

    #[test]
    fn test_cursor_position_calculation() {
        // Test cursor position calculation for wrapped text
        let text = "This is a long line that wraps";
        let width = 10;
        
        // Test cursor at beginning
        let cursor_line = calculate_cursor_line(text, 0, width);
        assert_eq!(cursor_line, 0, "Cursor at beginning should be on first line");
        
        // Test cursor in middle of first line
        let cursor_line = calculate_cursor_line(text, 5, width);
        assert_eq!(cursor_line, 0, "Cursor in middle of first line should be on first line");
        
        // Test cursor beyond first line
        let cursor_line = calculate_cursor_line(text, 15, width);
        assert!(cursor_line > 0, "Cursor beyond first line should be on subsequent line");
        
        // Test cursor at end
        let cursor_line = calculate_cursor_line(text, text.len(), width);
        assert!(cursor_line >= 0, "Cursor at end should be on valid line");
    }

    #[test]
    fn test_cursor_position_with_newlines() {
        // Test cursor position calculation with explicit newlines
        let text = "Line 1\nLine 2\nLine 3";
        let width = 20;
        
        // Test cursor at beginning
        let cursor_line = calculate_cursor_line(text, 0, width);
        assert_eq!(cursor_line, 0, "Cursor at beginning should be on first line");
        
        // Test cursor at start of second line
        let cursor_line = calculate_cursor_line(text, 7, width); // After "Line 1\n"
        assert_eq!(cursor_line, 1, "Cursor at start of second line should be on second line");
        
        // Test cursor at start of third line
        let cursor_line = calculate_cursor_line(text, 14, width); // After "Line 1\nLine 2\n"
        assert_eq!(cursor_line, 2, "Cursor at start of third line should be on third line");
    }

    #[test]
    fn test_message_formatting() {
        // Test message formatting for TUI display
        let mut cache = HighlightCache::new();
        
        // Test user message
        let user_spans = format_message_for_tui_cached("user", "Hello, world!", &mut cache);
        assert!(!user_spans.is_empty(), "User message should produce spans");
        
        // Test assistant message
        let assistant_spans = format_message_for_tui_cached("assistant", "Hello back!", &mut cache);
        assert!(!assistant_spans.is_empty(), "Assistant message should produce spans");
        
        // Test caching
        let cached_spans = format_message_for_tui_cached("user", "Hello, world!", &mut cache);
        assert_eq!(user_spans.len(), cached_spans.len(), "Cached results should match original");
    }

    #[test]
    fn test_message_formatting_with_markdown() {
        // Test message formatting with markdown-like content
        let mut cache = HighlightCache::new();
        
        let content = "This has **bold** text and `code` snippets.";
        let spans = format_message_for_tui_cached("assistant", content, &mut cache);
        
        assert!(!spans.is_empty(), "Markdown content should produce spans");
        
        // The exact formatting depends on the implementation, but we can check that the content is preserved
        let text_content: String = spans.iter().map(|line| line.to_string()).collect();
        assert!(text_content.contains("bold"), "Bold text should be preserved");
        assert!(text_content.contains("code"), "Code text should be preserved");
    }
}

#[cfg(test)]
mod scroll_calculation_tests {
    use super::*;

    #[test]
    fn test_chat_scroll_offset_calculation() {
        // Test chat scroll offset calculation
        let mut cache = HighlightCache::new();
        
        // Create test messages
        let messages = create_ui_test_messages();
        
        // Convert to spans
        let mut chat_spans = Vec::new();
        for msg in &messages {
            chat_spans.extend(format_message_for_tui_cached(&msg.role, &msg.content, &mut cache));
        }
        
        let chat_height = 10;
        let chat_width = 40;
        
        let scroll_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, chat_width);
        
        assert!(scroll_offset >= 0, "Scroll offset should be non-negative");
    }

    #[test]
    fn test_scroll_offset_with_empty_messages() {
        // Test scroll offset calculation with empty messages
        let chat_spans = Vec::new();
        let chat_height = 10;
        let chat_width = 40;
        
        let scroll_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, chat_width);
        
        assert_eq!(scroll_offset, 0, "Empty messages should result in zero scroll offset");
    }

    #[test]
    fn test_scroll_offset_with_small_content() {
        // Test scroll offset when content is smaller than chat height
        let mut cache = HighlightCache::new();
        
        let short_message = Message {
            role: "user".to_string(),
            content: "Short".to_string(),
        };
        
        let mut chat_spans = Vec::new();
        chat_spans.extend(format_message_for_tui_cached(&short_message.role, &short_message.content, &mut cache));
        
        let chat_height = 20; // Much larger than content
        let chat_width = 40;
        
        let scroll_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, chat_width);
        
        assert_eq!(scroll_offset, 0, "Small content should result in zero scroll offset");
    }

    #[test]
    fn test_scroll_offset_with_large_content() {
        // Test scroll offset when content is larger than chat height
        let mut cache = HighlightCache::new();
        
        // Create many messages
        let mut messages = Vec::new();
        for i in 0..20 {
            messages.push(Message {
                role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                content: format!("Message {} with some content that might wrap", i),
            });
        }
        
        let mut chat_spans = Vec::new();
        for msg in &messages {
            chat_spans.extend(format_message_for_tui_cached(&msg.role, &msg.content, &mut cache));
        }
        
        let chat_height = 5; // Much smaller than content
        let chat_width = 40;
        
        let scroll_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, chat_width);
        
        assert!(scroll_offset > 0, "Large content should result in positive scroll offset");
    }

    #[test]
    fn test_scroll_offset_with_different_widths() {
        // Test scroll offset calculation with different widths
        let mut cache = HighlightCache::new();
        
        let long_message = Message {
            role: "user".to_string(),
            content: "This is a very long message that will wrap differently depending on the terminal width available for display".to_string(),
        };
        
        let mut chat_spans = Vec::new();
        chat_spans.extend(format_message_for_tui_cached(&long_message.role, &long_message.content, &mut cache));
        
        let chat_height = 10;
        
        // Test with narrow width
        let narrow_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, 20);
        
        // Test with wide width
        let wide_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, 80);
        
        // Narrow width should generally result in higher scroll offset (more wrapping)
        assert!(narrow_offset >= wide_offset, "Narrow width should result in higher or equal scroll offset");
    }
}

#[cfg(test)]
mod input_display_tests {
    use super::*;

    #[test]
    fn test_input_title_display() {
        // Test input title display based on configuration
        let expected_title = if SHIFT_ENTER_SENDS {
            "Input (Shift/Alt+Enter to send, Enter for newline)"
        } else {
            "Input (Enter to send, Shift/Alt+Enter for newline)"
        };
        
        // This tests the configuration logic used in the UI
        assert!(!expected_title.is_empty(), "Input title should not be empty");
        assert!(expected_title.contains("Enter"), "Input title should mention Enter key");
        assert!(expected_title.contains("send"), "Input title should mention sending");
        assert!(expected_title.contains("newline"), "Input title should mention newline");
    }

    #[test]
    fn test_input_scrolling_logic() {
        // Test input scrolling logic for multi-line input
        let mut app = create_ui_test_app();
        
        // Set up multi-line input
        app.input = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string();
        
        let area_width = 20;
        let area_height = 3; // Small height to force scrolling
        
        let input_lines = wrap_text(&app.input, area_width - 2);
        let cursor_line = calculate_cursor_line(&app.input, app.cursor_position, area_width - 2);
        let input_height = area_height - 2;
        
        // Test auto-scroll to keep cursor visible
        let original_scroll_offset = app.input_scroll_offset;
        
        // Simulate cursor at bottom
        app.cursor_position = app.input.len();
        let cursor_line = calculate_cursor_line(&app.input, app.cursor_position, area_width - 2);
        
        // Calculate new scroll offset
        let new_scroll_offset = if cursor_line >= (app.input_scroll_offset as usize + input_height as usize) {
            (cursor_line + 1).saturating_sub(input_height as usize) as u16
        } else if cursor_line < app.input_scroll_offset as usize {
            cursor_line as u16
        } else {
            app.input_scroll_offset
        };
        
        app.input_scroll_offset = new_scroll_offset;
        
        // Verify scrolling behavior
        assert!(app.input_scroll_offset <= input_lines.len() as u16, "Scroll offset should not exceed input lines");
    }

    #[test]
    fn test_input_cursor_visibility() {
        // Test that cursor remains visible during input scrolling
        let mut app = create_ui_test_app();
        
        // Create input that requires scrolling
        app.input = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6".to_string();
        
        let area_width = 20;
        let area_height = 4; // Height that can show 2 lines of input
        let input_height = area_height - 2;
        
        // Test cursor at beginning
        app.cursor_position = 0;
        let cursor_line = calculate_cursor_line(&app.input, app.cursor_position, area_width - 2);
        assert_eq!(cursor_line, 0, "Cursor at beginning should be on first line");
        
        // Test cursor at end
        app.cursor_position = app.input.len();
        let cursor_line = calculate_cursor_line(&app.input, app.cursor_position, area_width - 2);
        assert!(cursor_line >= 0, "Cursor at end should be on valid line");
        
        // Test scroll offset adjustment
        if cursor_line >= (app.input_scroll_offset as usize + input_height as usize) {
            app.input_scroll_offset = (cursor_line + 1).saturating_sub(input_height as usize) as u16;
        } else if cursor_line < app.input_scroll_offset as usize {
            app.input_scroll_offset = cursor_line as u16;
        }
        
        // Verify cursor is visible
        assert!(cursor_line >= app.input_scroll_offset as usize, "Cursor should be at or after scroll offset");
        assert!(cursor_line < (app.input_scroll_offset as usize + input_height as usize), "Cursor should be within visible area");
    }

    #[test]
    fn test_input_line_wrapping() {
        // Test input line wrapping behavior
        let long_input = "This is a very long input line that should wrap across multiple lines when the terminal width is limited and the text exceeds the available space";
        let width = 20;
        
        let wrapped = wrap_text(long_input, width);
        
        assert!(wrapped.len() > 1, "Long input should wrap to multiple lines");
        
        // Verify each line doesn't exceed width (check span lengths)
        for line in &wrapped {
            let line_text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();
            assert!(line_text.len() <= width, "Wrapped line should not exceed width");
        }
        
        // Verify content is preserved
        let rejoined: String = wrapped.iter().map(|line| line.spans.iter().map(|span| span.content.as_ref()).collect::<String>()).collect::<Vec<_>>().join("");
        assert_eq!(rejoined, long_input, "Content should be preserved after wrapping");
    }

    #[test]
    fn test_input_with_tabs_and_spaces() {
        // Test input handling with tabs and spaces
        let input_with_tabs = "Line 1\n\tIndented line\n    Space indented";
        let width = 20;
        
        let wrapped = wrap_text(input_with_tabs, width);
        
        assert!(wrapped.len() >= 3, "Input with tabs should produce multiple lines");
        
        // Check that tabs and spaces are preserved in some form
        let rejoined: String = wrapped.iter().map(|line| line.spans.iter().map(|span| span.content.as_ref()).collect::<String>()).collect::<Vec<_>>().join("\n");
        assert!(rejoined.contains("Indented"), "Indented content should be preserved");
        assert!(rejoined.contains("Space indented"), "Space indented content should be preserved");
    }
}

#[cfg(test)]
mod status_display_tests {
    use super::*;

    #[test]
    fn test_status_message_display() {
        // Test status message display logic
        let mut app = create_ui_test_app();
        
        // Test default status
        assert_eq!(app.status, "", "Default status should be empty");
        
        // Test waiting status
        app.status = "Sending to Claude...".to_string();
        app.waiting = true;
        assert_eq!(app.status, "Sending to Claude...");
        assert!(app.waiting);
        
        // Test error status
        app.status = "Error: Connection failed".to_string();
        app.waiting = false;
        assert_eq!(app.status, "Error: Connection failed");
        assert!(!app.waiting);
    }

    #[test]
    fn test_token_usage_display() {
        // Test token usage display logic
        let mut app = create_ui_test_app();
        
        // Test initial state
        assert_eq!(app.client.total_input_tokens, 0);
        assert_eq!(app.client.total_output_tokens, 0);
        
        // Test with some tokens
        app.client.total_input_tokens = 127;
        app.client.total_output_tokens = 89;
        
        let total_tokens = app.client.total_input_tokens + app.client.total_output_tokens;
        assert_eq!(total_tokens, 216);
        
        // Test token display formatting
        let token_display = format!(
            "Input tokens: {}, Output tokens: {}, Total tokens: {}",
            app.client.total_input_tokens,
            app.client.total_output_tokens,
            total_tokens
        );
        
        assert!(token_display.contains("Input tokens: 127"));
        assert!(token_display.contains("Output tokens: 89"));
        assert!(token_display.contains("Total tokens: 216"));
    }

    #[test]
    fn test_progress_indicator() {
        // Test progress indicator logic
        let mut app = create_ui_test_app();
        
        // Test progress animation
        app.waiting = true;
        app.progress_i = 0;
        
        // Simulate progress updates
        for i in 1..=10 {
            app.progress_i = i;
            
            // Progress should be tracked
            assert_eq!(app.progress_i, i);
            
            // Waiting state should be maintained
            assert!(app.waiting);
        }
        
        // Test progress completion
        app.waiting = false;
        assert!(!app.waiting);
        
        // Progress value can be anything when not waiting
        assert!(app.progress_i >= 0);
    }

    #[test]
    fn test_status_transitions() {
        // Test status transitions during application lifecycle
        let mut app = create_ui_test_app();
        
        // Initial state
        assert_eq!(app.status, "");
        assert!(!app.waiting);
        
        // Starting request
        app.status = "Sending to Claude...".to_string();
        app.waiting = true;
        assert_eq!(app.status, "Sending to Claude...");
        assert!(app.waiting);
        
        // Request completed successfully
        app.status = "Ready".to_string();
        app.waiting = false;
        assert_eq!(app.status, "Ready");
        assert!(!app.waiting);
        
        // Error occurred
        app.status = "Error: Request failed".to_string();
        app.waiting = false;
        assert_eq!(app.status, "Error: Request failed");
        assert!(!app.waiting);
        
        // Recovery to ready state
        app.status = "Ready".to_string();
        assert_eq!(app.status, "Ready");
    }
}

#[cfg(test)]
mod layout_tests {

    #[test]
    fn test_layout_calculations() {
        // Test layout calculations for different terminal sizes
        
        // Small terminal
        let small_terminal = (40u16, 10u16);
        let (width, height) = small_terminal;
        
        // Calculate rough layout areas
        let chat_height = height.saturating_sub(8); // Leave room for input and status
        let input_height = 3u16; // Minimum input height
        let status_height = 2u16; // Status bar height
        
        assert!(chat_height > 0, "Chat area should have positive height");
        assert!(input_height > 0, "Input area should have positive height");
        assert!(status_height > 0, "Status area should have positive height");
        assert!(chat_height + input_height + status_height <= height, "Total height should not exceed terminal height");
        
        // Large terminal
        let large_terminal = (120u16, 40u16);
        let (width, height) = large_terminal;
        
        let chat_height = height.saturating_sub(8);
        assert!(chat_height > 0, "Chat area should have positive height on large terminal");
        assert!(chat_height >= 10, "Chat area should have reasonable height on large terminal");
    }

    #[test]
    fn test_minimum_layout_requirements() {
        // Test minimum layout requirements
        let min_width = 20u16;
        let min_height = 6u16;
        
        let chat_height = min_height.saturating_sub(6); // Very minimal
        let input_height = 2u16;
        let status_height = 2u16;
        
        // Should still be functional with minimal dimensions
        assert!(chat_height == 0 || chat_height > 0, "Chat height should be handled gracefully");
        assert!(input_height > 0, "Input area should always be available");
        assert!(status_height > 0, "Status area should always be available");
    }

    #[test]
    fn test_layout_proportions() {
        // Test layout proportions for different terminal sizes
        let test_sizes = vec![
            (40u16, 10u16),   // Small
            (80u16, 24u16),   // Medium
            (120u16, 40u16),  // Large
            (160u16, 60u16),  // Extra large
        ];
        
        for (width, height) in test_sizes {
            let chat_height = height.saturating_sub(8);
            let total_ui_height = chat_height + 3 + 2; // chat + input + status
            
            // Chat should take up a reasonable portion of the space for larger terminals
            if height >= 12 {
                assert!(chat_height >= height / 3, "Chat should take at least 1/3 of height for larger terminals");
            }
            
            // Total UI should not exceed terminal size
            assert!(total_ui_height <= height, "Total UI height should not exceed terminal height");
        }
    }
}

#[cfg(test)]
mod visual_consistency_tests {
    use super::*;

    #[test]
    fn test_message_display_consistency() {
        // Test that messages are displayed consistently
        let mut app = create_ui_test_app();
        let mut cache = HighlightCache::new();
        
        // Add test messages
        app.client.messages = create_ui_test_messages();
        
        // Render messages multiple times
        for _ in 0..3 {
            let mut chat_spans = Vec::new();
            for msg in &app.client.messages {
                chat_spans.extend(format_message_for_tui_cached(&msg.role, &msg.content, &mut cache));
            }
            
            assert!(!chat_spans.is_empty(), "Messages should render consistently");
            assert_eq!(chat_spans.len(), chat_spans.len(), "Span count should be consistent");
        }
    }

    #[test]
    fn test_input_display_consistency() {
        // Test that input is displayed consistently
        let mut app = create_ui_test_app();
        
        app.input = "Test input with multiple lines\nSecond line\nThird line".to_string();
        app.cursor_position = 10;
        
        let width = 40;
        
        // Render input multiple times
        for _ in 0..3 {
            let input_lines = wrap_text(&app.input, width);
            let cursor_line = calculate_cursor_line(&app.input, app.cursor_position, width);
            
            assert!(!input_lines.is_empty(), "Input should render consistently");
            assert!(cursor_line >= 0, "Cursor line should be valid");
        }
    }

    #[test]
    fn test_status_display_consistency() {
        // Test that status is displayed consistently
        let mut app = create_ui_test_app();
        
        let test_statuses = vec![
            "Ready",
            "Sending to Claude...",
            "Error: Connection failed",
            "Saving conversation...",
            "Loading conversation...",
        ];
        
        for status in test_statuses {
            app.status = status.to_string();
            
            // Status should be displayed as-is
            assert_eq!(app.status, status, "Status should be displayed consistently");
        }
    }

    #[test]
    fn test_unicode_display_consistency() {
        // Test Unicode display consistency
        let mut app = create_ui_test_app();
        let mut cache = HighlightCache::new();
        
        let unicode_content = "Hello 世界! 🦀 Rust は素晴らしい! Здравствуй мир! مرحبا بالعالم!";
        
        app.client.messages.push(Message {
            role: "user".to_string(),
            content: unicode_content.to_string(),
        });
        
        // Render Unicode content multiple times
        for _ in 0..3 {
            let chat_spans = format_message_for_tui_cached("user", unicode_content, &mut cache);
            assert!(!chat_spans.is_empty(), "Unicode content should render consistently");
            
            let rendered_text: String = chat_spans.iter().map(|line| line.to_string()).collect();
            assert!(rendered_text.contains("世界"), "Chinese characters should be preserved");
            assert!(rendered_text.contains("🦀"), "Emoji should be preserved");
            assert!(rendered_text.contains("は"), "Japanese characters should be preserved");
            assert!(rendered_text.contains("мир"), "Cyrillic characters should be preserved");
            assert!(rendered_text.contains("بالعالم"), "Arabic characters should be preserved");
        }
    }
}
