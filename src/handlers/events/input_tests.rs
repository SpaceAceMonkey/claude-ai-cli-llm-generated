//! Unit tests for input handling functionality
//! Tests character input, cursor movement, newline handling, and command processing

use crossterm::event::KeyModifiers;
use tokio::sync::mpsc;
use tempfile::TempDir;

use crate::app::AppState;
use crate::config::get_default_colors;
use crate::handlers::events::input::*;
use crate::api::Message;

/// Helper function to create a test AppState with minimal setup
fn create_test_app_state() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = AppState::new(
        "test_key".to_string(),
        "test_model".to_string(),
        1000,
        0.7,
        false,
        get_default_colors(),
    ).expect("Failed to create AppState");
    
    app.current_directory = temp_dir.path().to_path_buf();
    (app, temp_dir)
}

/// Helper function to create a test channel for async operations
fn create_test_channel() -> (mpsc::Sender<Result<(String, u32, u32, Vec<Message>), String>>, mpsc::Receiver<Result<(String, u32, u32, Vec<Message>), String>>) {
    mpsc::channel(10)
}

#[cfg(test)]
mod character_input_tests {
    use super::*;

    /// Test basic character input appends to input string and advances cursor
    /// Expected: Character is added at cursor position, cursor moves forward
    #[test]
    fn test_basic_character_input() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Test inserting character at beginning
        handle_char_input(&mut app, 'h');
        assert_eq!(app.input, "h");
        assert_eq!(app.cursor_position, 1);
        
        // Test inserting more characters
        handle_char_input(&mut app, 'e');
        handle_char_input(&mut app, 'l');
        handle_char_input(&mut app, 'l');
        handle_char_input(&mut app, 'o');
        
        assert_eq!(app.input, "hello");
        assert_eq!(app.cursor_position, 5);
    }

    /// Test character insertion at middle of existing text
    /// Expected: Character is inserted at cursor position, existing text shifts right
    #[test]
    fn test_character_insertion_middle() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up initial text
        app.input = "hello".to_string();
        app.cursor_position = 2; // Between 'e' and 'l'
        
        // Insert character in middle
        handle_char_input(&mut app, 'X');
        
        assert_eq!(app.input, "heXllo");
        assert_eq!(app.cursor_position, 3);
    }

    /// Test character insertion at beginning of existing text
    /// Expected: Character is inserted at position 0, all text shifts right
    #[test]
    fn test_character_insertion_beginning() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up initial text
        app.input = "world".to_string();
        app.cursor_position = 0;
        
        // Insert character at beginning
        handle_char_input(&mut app, 'X');
        
        assert_eq!(app.input, "Xworld");
        assert_eq!(app.cursor_position, 1);
    }

    /// Test Unicode character input handling
    /// Expected: Unicode characters are handled correctly in input and cursor positioning
    #[test]
    fn test_unicode_character_input() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Test various Unicode characters
        handle_char_input(&mut app, '你');
        handle_char_input(&mut app, '好');
        handle_char_input(&mut app, '🚀');
        handle_char_input(&mut app, 'é');
        
        assert_eq!(app.input, "你好🚀é");
        assert_eq!(app.cursor_position, 4);
    }
}

#[cfg(test)]
mod backspace_delete_tests {
    use super::*;

    /// Test backspace removes character before cursor and moves cursor back
    /// Expected: Character at cursor-1 is removed, cursor moves left
    #[test]
    fn test_backspace_basic() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text
        app.input = "hello".to_string();
        app.cursor_position = 5; // At end
        
        // Backspace should remove 'o'
        handle_backspace(&mut app);
        assert_eq!(app.input, "hell");
        assert_eq!(app.cursor_position, 4);
        
        // Backspace should remove 'l'
        handle_backspace(&mut app);
        assert_eq!(app.input, "hel");
        assert_eq!(app.cursor_position, 3);
    }

    /// Test backspace at beginning of text does nothing
    /// Expected: No change to input or cursor position
    #[test]
    fn test_backspace_at_beginning() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text with cursor at beginning
        app.input = "hello".to_string();
        app.cursor_position = 0;
        
        // Backspace should do nothing
        handle_backspace(&mut app);
        assert_eq!(app.input, "hello");
        assert_eq!(app.cursor_position, 0);
    }

    /// Test backspace in middle of text removes correct character
    /// Expected: Character at cursor-1 is removed, cursor moves left
    #[test]
    fn test_backspace_middle() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text with cursor in middle
        app.input = "hello".to_string();
        app.cursor_position = 3; // Between second 'l' and 'o'
        
        // Backspace should remove second 'l'
        handle_backspace(&mut app);
        assert_eq!(app.input, "helo");
        assert_eq!(app.cursor_position, 2);
    }

    /// Test delete removes character at cursor position
    /// Expected: Character at cursor position is removed, cursor stays in place
    #[test]
    fn test_delete_basic() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text with cursor at beginning
        app.input = "hello".to_string();
        app.cursor_position = 0;
        
        // Delete should remove 'h'
        handle_delete(&mut app);
        assert_eq!(app.input, "ello");
        assert_eq!(app.cursor_position, 0);
    }

    /// Test delete at end of text does nothing
    /// Expected: No change to input or cursor position
    #[test]
    fn test_delete_at_end() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text with cursor at end
        app.input = "hello".to_string();
        app.cursor_position = 5;
        
        // Delete should do nothing
        handle_delete(&mut app);
        assert_eq!(app.input, "hello");
        assert_eq!(app.cursor_position, 5);
    }

    /// Test delete in middle of text removes correct character
    /// Expected: Character at cursor position is removed, cursor stays in place
    #[test]
    fn test_delete_middle() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text with cursor in middle
        app.input = "hello".to_string();
        app.cursor_position = 2; // At second 'l'
        
        // Delete should remove second 'l'
        handle_delete(&mut app);
        assert_eq!(app.input, "helo");
        assert_eq!(app.cursor_position, 2);
    }

    /// Test backspace and delete with Unicode characters
    /// Expected: Unicode characters are handled correctly as single units
    #[test]
    fn test_unicode_backspace_delete() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up Unicode text
        app.input = "你好🚀".to_string();
        app.cursor_position = 3; // After emoji
        
        // Backspace should remove emoji
        handle_backspace(&mut app);
        assert_eq!(app.input, "你好");
        assert_eq!(app.cursor_position, 2);
        
        // Delete should remove nothing (at end)
        handle_delete(&mut app);
        assert_eq!(app.input, "你好");
        assert_eq!(app.cursor_position, 2);
        
        // Move cursor and test delete
        app.cursor_position = 1;
        handle_delete(&mut app);
        assert_eq!(app.input, "你");
        assert_eq!(app.cursor_position, 1);
    }
}

#[cfg(test)]
mod enter_key_tests {
    use super::*;

    /// Test regular Enter key sends message when SHIFT_ENTER_SENDS is false
    /// Expected: Message is sent, input is cleared, cursor reset
    #[tokio::test]
    async fn test_regular_enter_sends_message() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Set up input
        app.input = "test message".to_string();
        app.cursor_position = 12;
        
        // Handle Enter key with no modifiers
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        
        // Should have cleared input and reset cursor
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert!(app.waiting);
        assert_eq!(app.status, "Sending to Claude...");
        
        // Should have added user message
        assert_eq!(app.client.messages.len(), 1);
        assert_eq!(app.client.messages[0].role, "user");
        assert_eq!(app.client.messages[0].content, "test message");
    }

    /// Test Shift+Enter inserts newline when SHIFT_ENTER_SENDS is false
    /// Expected: Newline character is inserted at cursor position
    #[tokio::test]
    async fn test_shift_enter_inserts_newline() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Set up input
        app.input = "hello".to_string();
        app.cursor_position = 5;
        
        // Handle Shift+Enter
        handle_enter_key(&mut app, KeyModifiers::SHIFT, &tx).await.unwrap();
        
        // Should have inserted newline
        assert_eq!(app.input, "hello\n");
        assert_eq!(app.cursor_position, 6);
        assert!(!app.waiting); // Should not be sending
    }

    /// Test Alt+Enter inserts newline when SHIFT_ENTER_SENDS is false
    /// Expected: Newline character is inserted at cursor position
    #[tokio::test]
    async fn test_alt_enter_inserts_newline() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Set up input
        app.input = "hello".to_string();
        app.cursor_position = 3; // In middle
        
        // Handle Alt+Enter
        handle_enter_key(&mut app, KeyModifiers::ALT, &tx).await.unwrap();
        
        // Should have inserted newline in middle
        assert_eq!(app.input, "hel\nlo");
        assert_eq!(app.cursor_position, 4);
        assert!(!app.waiting); // Should not be sending
    }

    /// Test Ctrl+Enter sends message even with text
    /// Expected: Message is sent regardless of SHIFT_ENTER_SENDS setting
    #[tokio::test]
    async fn test_ctrl_enter_sends_message() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Set up input
        app.input = "test message".to_string();
        app.cursor_position = 12;
        
        // Handle Ctrl+Enter
        handle_enter_key(&mut app, KeyModifiers::CONTROL, &tx).await.unwrap();
        
        // Should have sent message
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert!(app.waiting);
        assert_eq!(app.client.messages.len(), 1);
        assert_eq!(app.client.messages[0].content, "test message");
    }

    /// Test Ctrl+Enter with empty input does nothing
    /// Expected: No message sent, no state changes
    #[tokio::test]
    async fn test_ctrl_enter_empty_input() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Empty input
        app.input = "".to_string();
        app.cursor_position = 0;
        
        // Handle Ctrl+Enter
        handle_enter_key(&mut app, KeyModifiers::CONTROL, &tx).await.unwrap();
        
        // Should not have sent anything
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert!(!app.waiting);
        assert_eq!(app.client.messages.len(), 0);
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;

    /// Test /save command opens save dialog
    /// Expected: Save dialog opens, input is cleared, directory is set
    #[tokio::test]
    async fn test_save_command() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Set up /save command
        app.input = "/save".to_string();
        app.cursor_position = 5;
        
        // Handle Enter key
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        
        // Should have opened save dialog
        assert!(app.show_save_dialog);
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert_eq!(app.save_filename, "");
        assert_eq!(app.dialog_cursor_pos, 0);
        assert!(app.available_files.len() > 0); // Should have loaded directory
    }

    /// Test /load command opens load dialog
    /// Expected: Load dialog opens, input is cleared, directory is set
    #[tokio::test]
    async fn test_load_command() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Set up /load command
        app.input = "/load".to_string();
        app.cursor_position = 5;
        
        // Handle Enter key
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        
        // Should have opened load dialog
        assert!(app.show_load_dialog);
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert!(app.available_files.len() > 0); // Should have loaded directory
    }

    /// Test /save command with space character triggers dialog
    /// Expected: Dialog opens when space is pressed after /save
    #[test]
    fn test_save_command_with_space() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up /save command
        app.input = "/save".to_string();
        app.cursor_position = 5;
        
        // Handle space character
        handle_char_input(&mut app, ' ');
        
        // Should have opened save dialog
        assert!(app.show_save_dialog);
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert_eq!(app.save_filename, "");
        assert_eq!(app.dialog_cursor_pos, 0);
    }

    /// Test /load command with space character triggers dialog
    /// Expected: Dialog opens when space is pressed after /load
    #[test]
    fn test_load_command_with_space() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up /load command
        app.input = "/load".to_string();
        app.cursor_position = 5;
        
        // Handle space character
        handle_char_input(&mut app, ' ');
        
        // Should have opened load dialog
        assert!(app.show_load_dialog);
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
    }

    /// Test slash character starts command mode
    /// Expected: Forward slash is inserted at beginning of empty input
    #[test]
    fn test_slash_command_start() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Empty input, cursor at beginning
        app.input = "".to_string();
        app.cursor_position = 0;
        
        // Handle slash character
        handle_char_input(&mut app, '/');
        
        // Should have inserted slash
        assert_eq!(app.input, "/");
        assert_eq!(app.cursor_position, 1);
    }

    /// Test regular text after slash doesn't trigger commands
    /// Expected: Text is treated as regular input, no dialogs open
    #[test]
    fn test_non_command_with_slash() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up text that starts with slash but isn't a command
        app.input = "/hello".to_string();
        app.cursor_position = 6;
        
        // Handle space character
        handle_char_input(&mut app, ' ');
        
        // Should not have opened any dialogs
        assert!(!app.show_save_dialog);
        assert!(!app.show_load_dialog);
        assert_eq!(app.input, "/hello ");
        assert_eq!(app.cursor_position, 7);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete input flow: type, edit, send message
    /// Expected: All operations work together correctly
    #[tokio::test]
    async fn test_complete_input_flow() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Type some text
        handle_char_input(&mut app, 'H');
        handle_char_input(&mut app, 'e');
        handle_char_input(&mut app, 'l');
        handle_char_input(&mut app, 'l');
        handle_char_input(&mut app, 'o');
        
        assert_eq!(app.input, "Hello");
        assert_eq!(app.cursor_position, 5);
        
        // Add newline with Shift+Enter
        handle_enter_key(&mut app, KeyModifiers::SHIFT, &tx).await.unwrap();
        
        assert_eq!(app.input, "Hello\n");
        assert_eq!(app.cursor_position, 6);
        
        // Add more text
        handle_char_input(&mut app, 'W');
        handle_char_input(&mut app, 'o');
        handle_char_input(&mut app, 'r');
        handle_char_input(&mut app, 'l');
        handle_char_input(&mut app, 'd');
        
        assert_eq!(app.input, "Hello\nWorld");
        assert_eq!(app.cursor_position, 11);
        
        // Send message
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        
        // Should have sent multi-line message
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
        assert_eq!(app.client.messages.len(), 1);
        assert_eq!(app.client.messages[0].content, "Hello\nWorld");
    }

    /// Test cursor positioning with complex edits
    /// Expected: Cursor remains in correct position through various operations
    #[test]
    fn test_cursor_positioning_complex() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Build text: "Hello World"
        app.input = "Hello World".to_string();
        app.cursor_position = 6; // Between "Hello" and "World"
        
        // Insert text in middle
        handle_char_input(&mut app, 'X');
        handle_char_input(&mut app, 'Y');
        handle_char_input(&mut app, 'Z');
        
        assert_eq!(app.input, "Hello XYZWorld");
        assert_eq!(app.cursor_position, 9);
        
        // Backspace a few characters
        handle_backspace(&mut app);
        handle_backspace(&mut app);
        
        assert_eq!(app.input, "Hello XWorld");
        assert_eq!(app.cursor_position, 7);
        
        // Delete forward
        handle_delete(&mut app);
        
        assert_eq!(app.input, "Hello Xorld");
        assert_eq!(app.cursor_position, 7);
    }

    /// Test multiple commands in sequence
    /// Expected: Each command is processed independently
    #[tokio::test]
    async fn test_multiple_commands_sequence() {
        let (mut app, _temp_dir) = create_test_app_state();
        let (tx, _rx) = create_test_channel();
        
        // Test /save command
        app.input = "/save".to_string();
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        assert!(app.show_save_dialog);
        
        // Reset state
        app.show_save_dialog = false;
        
        // Test /load command
        app.input = "/load".to_string();
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        assert!(app.show_load_dialog);
        
        // Reset state
        app.show_load_dialog = false;
        
        // Test regular message
        app.input = "regular message".to_string();
        handle_enter_key(&mut app, KeyModifiers::NONE, &tx).await.unwrap();
        assert_eq!(app.client.messages.len(), 1);
        assert_eq!(app.client.messages[0].content, "regular message");
    }
}
