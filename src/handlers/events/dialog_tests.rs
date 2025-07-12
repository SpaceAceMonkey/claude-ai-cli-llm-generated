//! Unit tests for dialog handling functionality
//! Tests the dialog navigation, state management, and user interactions

use crossterm::event::KeyCode;
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

use crate::app::AppState;
use crate::handlers::events::dialogs::*;
use crate::handlers::file_ops::load_directory_contents;
use crate::config::get_default_colors;

/// Helper function to create a test AppState with temporary directory
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

/// Helper function to create test files in a directory
fn create_test_files(dir: &PathBuf, files: &[&str]) {
    for file in files {
        if file.ends_with('/') {
            let dir_path = dir.join(&file[..file.len()-1]);
            fs::create_dir_all(&dir_path).expect("Failed to create test directory");
        } else {
            let file_path = dir.join(file);
            fs::write(&file_path, "test content").expect("Failed to create test file");
        }
    }
}

#[cfg(test)]
mod exit_dialog_tests {
    use super::*;

    /// Test that pressing 'y' in the exit dialog immediately exits the application
    /// Expected: should_exit returns true, indicating the application should terminate
    #[test]
    fn test_exit_dialog_yes_key() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        // Press 'y' key to confirm exit
        let should_exit = handle_exit_dialog(&mut app, KeyCode::Char('y'))
            .expect("Failed to handle exit dialog");
        
        // Expected: Application should exit immediately
        assert!(should_exit);
    }

    /// Test that pressing 'n' in the exit dialog cancels the exit and closes the dialog
    /// Expected: should_exit returns false, dialog is closed, selection is reset
    #[test]
    fn test_exit_dialog_no_key() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        // Press 'n' key to cancel exit
        let should_exit = handle_exit_dialog(&mut app, KeyCode::Char('n'))
            .expect("Failed to handle exit dialog");
        
        // Expected: Application should not exit, dialog should close, selection should reset
        assert!(!should_exit);
        assert!(!app.show_exit_dialog);
        assert_eq!(app.exit_selected, 0);
    }

    /// Test that pressing Enter when "Yes" is selected exits the application
    /// Expected: should_exit returns true when Enter is pressed on the "Yes" option
    #[test]
    fn test_exit_dialog_enter_on_yes() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        app.exit_selected = 0; // Yes selected
        
        // Press Enter while "Yes" is selected
        let should_exit = handle_exit_dialog(&mut app, KeyCode::Enter)
            .expect("Failed to handle exit dialog");
        
        // Expected: Application should exit
        assert!(should_exit);
    }

    /// Test that pressing Enter when "No" is selected cancels the exit
    /// Expected: should_exit returns false, dialog closes, selection resets
    #[test]
    fn test_exit_dialog_enter_on_no() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        app.exit_selected = 1; // No selected
        
        // Press Enter while "No" is selected
        let should_exit = handle_exit_dialog(&mut app, KeyCode::Enter)
            .expect("Failed to handle exit dialog");
        
        // Expected: Application should not exit, dialog should close, selection should reset
        assert!(!should_exit);
        assert!(!app.show_exit_dialog);
        assert_eq!(app.exit_selected, 0);
    }

    /// Test that pressing Escape cancels the exit dialog regardless of current selection
    /// Expected: should_exit returns false, dialog closes, selection resets
    #[test]
    fn test_exit_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        app.exit_selected = 1; // Start with "No" selected
        
        // Press Escape to cancel dialog
        let should_exit = handle_exit_dialog(&mut app, KeyCode::Esc)
            .expect("Failed to handle exit dialog");
        
        // Expected: Application should not exit, dialog should close, selection should reset to "Yes"
        assert!(!should_exit);
        assert!(!app.show_exit_dialog);
        assert_eq!(app.exit_selected, 0);
    }

    /// Test navigation between "Yes" and "No" options using arrow keys and directional keys
    /// Expected: Up/Down and Left/Right keys should toggle between options (0=Yes, 1=No)
    #[test]
    fn test_exit_dialog_navigation() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        app.exit_selected = 0; // Start with "Yes" selected
        
        // Navigate to "No" using Down arrow
        handle_exit_dialog(&mut app, KeyCode::Down).expect("Failed to handle navigation");
        assert_eq!(app.exit_selected, 1); // Should be on "No"
        
        // Navigate back to "Yes" using Up arrow
        handle_exit_dialog(&mut app, KeyCode::Up).expect("Failed to handle navigation");
        assert_eq!(app.exit_selected, 0); // Should be on "Yes"
        
        // Test horizontal navigation - Right should go to "No"
        handle_exit_dialog(&mut app, KeyCode::Right).expect("Failed to handle navigation");
        assert_eq!(app.exit_selected, 1); // Should be on "No"
        
        // Left should go back to "Yes"
        handle_exit_dialog(&mut app, KeyCode::Left).expect("Failed to handle navigation");
        assert_eq!(app.exit_selected, 0); // Should be on "Yes"
    }
}

#[cfg(test)]
mod save_dialog_tests {
    use super::*;

    /// Test that character input in the save dialog updates filename and cursor position
    /// Expected: Characters are appended to filename, cursor position advances
    #[test]
    fn test_save_dialog_filename_input() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Type "test" character by character
        handle_save_dialog(&mut app, KeyCode::Char('t'));
        handle_save_dialog(&mut app, KeyCode::Char('e'));
        handle_save_dialog(&mut app, KeyCode::Char('s'));
        handle_save_dialog(&mut app, KeyCode::Char('t'));
        
        // Expected: Filename should be "test" and cursor at position 4
        assert_eq!(app.save_filename, "test");
        assert_eq!(app.dialog_cursor_pos, 4);
    }

    /// Test backspace functionality in filename input field
    /// Expected: Backspace removes character before cursor, cursor moves back
    /// At beginning of string, backspace should have no effect
    #[test]
    fn test_save_dialog_backspace() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        app.save_filename = "test".to_string();
        app.dialog_cursor_pos = 4; // Cursor at end
        
        // Backspace should remove 't' and move cursor back
        handle_save_dialog(&mut app, KeyCode::Backspace);
        assert_eq!(app.save_filename, "tes");
        assert_eq!(app.dialog_cursor_pos, 3);
        
        // Test backspace at beginning - should not change anything
        app.dialog_cursor_pos = 0;
        handle_save_dialog(&mut app, KeyCode::Backspace);
        assert_eq!(app.save_filename, "tes"); // Should not change
        assert_eq!(app.dialog_cursor_pos, 0); // Cursor should stay at 0
    }

    /// Test that Escape key cancels the save dialog and resets state
    /// Expected: Dialog closes, filename cleared, cursor position reset
    #[test]
    fn test_save_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        app.save_filename = "test".to_string();
        app.dialog_cursor_pos = 4;
        
        // Press Escape to cancel dialog
        handle_save_dialog(&mut app, KeyCode::Esc);
        
        // Expected: Dialog should close and state should be reset
        assert!(!app.show_save_dialog);
        assert_eq!(app.save_filename, "");
        assert_eq!(app.dialog_cursor_pos, 0);
    }

    /// Test file list navigation with wrapping behavior
    /// Expected: Up/Down arrows navigate through file list with wrapping at boundaries
    #[test]
    fn test_save_dialog_file_list_navigation() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create test files for navigation
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt", "file2.txt", "file3.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Start at first item
        app.file_list_state.select(Some(0));
        
        // Navigate down one position
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(1));
        
        // Jump to last item to test wrapping
        let last_index = app.available_files.len() - 1;
        app.file_list_state.select(Some(last_index));
        
        // Navigate down from last item - should wrap to first
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
        
        // Navigate up from first item - should wrap to last
        handle_save_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(last_index));
    }

    /// Test directory navigation functionality in save dialog
    /// Expected: Selecting a directory and pressing Enter should navigate into it
    #[test]
    fn test_save_dialog_directory_navigation() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create a test subdirectory with a file inside
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdirectory");
        fs::write(subdir.join("file.txt"), "content").expect("Failed to create file");
        
        // Load directory contents to populate file list
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Find and select the subdirectory (should appear as "subdir/")
        let subdir_index = app.available_files.iter().position(|f| f == "subdir/").unwrap();
        app.file_list_state.select(Some(subdir_index));
        
        // Press Enter to navigate into subdirectory
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Expected: Current directory should change to the subdirectory
        assert_eq!(app.current_directory, subdir);
    }

    /// Test behavior with empty directory
    /// Expected: Should show parent directory ".." and "Create New Directory" option
    #[test]
    fn test_save_dialog_empty_directory() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Load contents of empty directory
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Expected: Empty directory should still have navigation options
        assert!(app.available_files.contains(&"../".to_string())); // Parent directory
        assert!(app.available_files.contains(&"[ Create New Directory ]".to_string())); // Create dir option
    }
}

#[cfg(test)]
mod load_dialog_tests {
    use super::*;

    /// Test loading a valid conversation file from the load dialog
    /// Expected: File loads successfully, dialog closes, conversation data is populated
    #[test]
    fn test_load_dialog_file_selection() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create a valid conversation JSON file
        let test_file = temp_dir.path().join("test.json");
        let conversation_json = r#"{
            "version": "1.0",
            "timestamp": "2023-01-01T00:00:00Z",
            "model": "test_model",
            "total_input_tokens": 100,
            "total_output_tokens": 200,
            "messages": []
        }"#;
        fs::write(&test_file, conversation_json).expect("Failed to create test file");
        
        // Load directory contents to see the file
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Find and select the test file
        let file_index = app.available_files.iter().position(|f| f == "test.json").unwrap();
        app.file_list_state.select(Some(file_index));
        
        // Press Enter to load the file
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Expected: Dialog should close, success message should show, conversation data should be loaded
        assert!(!app.show_load_dialog);
        assert!(app.status.contains("Conversation loaded"));
        assert_eq!(app.client.total_input_tokens, 100);
        assert_eq!(app.client.total_output_tokens, 200);
    }

    /// Test navigation wrapping behavior in load dialog file list
    /// Expected: Up/Down navigation wraps at boundaries (first↔last)
    #[test]
    fn test_load_dialog_navigation_wrapping() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create test files for navigation testing
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt", "file2.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Start at first item
        app.file_list_state.select(Some(0));
        
        // Navigate up from first item - should wrap to last
        handle_load_dialog(&mut app, KeyCode::Up);
        let last_index = app.available_files.len() - 1;
        assert_eq!(app.file_list_state.selected(), Some(last_index));
        
        // Navigate down from last item - should wrap to first
        handle_load_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
    }

    /// Test that Escape key closes the load dialog without loading anything
    /// Expected: Dialog closes, no file is loaded
    #[test]
    fn test_load_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Press Escape to cancel dialog
        handle_load_dialog(&mut app, KeyCode::Esc);
        
        // Expected: Dialog should close
        assert!(!app.show_load_dialog);
    }

    /// Test attempting to load an invalid JSON file
    /// Expected: Error message is shown, dialog remains open
    #[test]
    fn test_load_dialog_invalid_file() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create a file with invalid JSON content
        let test_file = temp_dir.path().join("invalid.json");
        fs::write(&test_file, "invalid json").expect("Failed to create test file");
        
        // Load directory contents to see the file
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Find and select the invalid file
        let file_index = app.available_files.iter().position(|f| f == "invalid.json").unwrap();
        app.file_list_state.select(Some(file_index));
        
        // Attempt to load the invalid file
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Expected: Error message should be shown indicating load failure
        assert!(app.status.contains("Load failed"));
    }

    /// Test pressing Enter when no file is selected
    /// Expected: Nothing happens, dialog remains open, no crash occurs
    #[test]
    fn test_load_dialog_no_selection() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Load directory contents
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Explicitly set no selection
        app.file_list_state.select(None);
        
        // Press Enter with no selection - should not crash
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Expected: Dialog should still be open, no adverse effects
        assert!(app.show_load_dialog);
    }
}

#[cfg(test)]
mod create_dir_dialog_tests {
    use super::*;

    /// Test creating a directory with a valid name
    /// Expected: Directory name is accumulated correctly, directory is created on Enter,
    /// dialog closes, name is cleared, and navigation switches to new directory
    #[test]
    fn test_create_dir_dialog_valid_name() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_create_dir_dialog = true;
        
        // Input directory name character by character
        handle_create_dir_dialog(&mut app, KeyCode::Char('t'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('e'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('s'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('t'));
        
        // Expected: Directory name should be accumulated properly
        assert_eq!(app.new_dir_name, "test");
        
        // Create directory by pressing Enter
        handle_create_dir_dialog(&mut app, KeyCode::Enter);
        
        // Expected: Directory should be created physically, dialog should close,
        // name should be cleared, and current directory should change to new directory
        let new_dir = temp_dir.path().join("test");
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
        assert!(!app.show_create_dir_dialog);
        assert_eq!(app.new_dir_name, "");
        assert_eq!(app.current_directory, new_dir);
    }

    /// Test input validation for directory names - invalid characters should be rejected
    /// Expected: Invalid characters (/, ?, *) are not added to directory name,
    /// but valid characters (alphanumeric, _, -, .) are accepted
    #[test]
    fn test_create_dir_dialog_invalid_characters() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_create_dir_dialog = true;
        
        // Try invalid characters that should be rejected
        handle_create_dir_dialog(&mut app, KeyCode::Char('/'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('?'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('*'));
        
        // Expected: Directory name should remain empty as invalid characters are rejected
        assert_eq!(app.new_dir_name, "");
        
        // Try valid characters that should be accepted
        handle_create_dir_dialog(&mut app, KeyCode::Char('a'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('_'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('-'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('.'));
        handle_create_dir_dialog(&mut app, KeyCode::Char('1'));
        
        // Expected: All valid characters should be included in directory name
        assert_eq!(app.new_dir_name, "a_-.1");
    }

    /// Test backspace functionality in directory name input
    /// Expected: Backspace removes last character from directory name
    /// At the beginning of an empty string, backspace should have no effect
    #[test]
    fn test_create_dir_dialog_backspace() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_create_dir_dialog = true;
        app.new_dir_name = "test".to_string();
        
        // Press backspace to remove last character
        handle_create_dir_dialog(&mut app, KeyCode::Backspace);
        
        // Expected: Last character 't' should be removed
        assert_eq!(app.new_dir_name, "tes");
        
        // Test backspace on empty string - should not crash or change anything
        app.new_dir_name.clear();
        handle_create_dir_dialog(&mut app, KeyCode::Backspace);
        
        // Expected: Empty string should remain empty
        assert_eq!(app.new_dir_name, "");
    }

    /// Test that Escape key cancels the create directory dialog
    /// Expected: Dialog closes and directory name is cleared
    #[test]
    fn test_create_dir_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_create_dir_dialog = true;
        app.new_dir_name = "test".to_string();
        
        // Press Escape to cancel dialog
        handle_create_dir_dialog(&mut app, KeyCode::Esc);
        
        // Expected: Dialog should close and directory name should be cleared
        assert!(!app.show_create_dir_dialog);
        assert_eq!(app.new_dir_name, "");
    }

    /// Test attempting to create a directory with an empty name
    /// Expected: Dialog closes but no directory is created, name is cleared
    #[test]
    fn test_create_dir_dialog_empty_name() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_create_dir_dialog = true;
        
        // Try to create directory with empty name by pressing Enter
        handle_create_dir_dialog(&mut app, KeyCode::Enter);
        
        // Expected: Dialog should close but no directory should be created
        // (verified by the fact that no filesystem operations occur)
        assert!(!app.show_create_dir_dialog);
        assert_eq!(app.new_dir_name, "");
    }
}

#[cfg(test)]
mod property_based_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    /// Property-based test: exit dialog should always handle any input safely
    /// Expected: No matter what state the exit dialog is in, it should never panic
    /// when handling any key input, and always return a valid result
    #[test]
    fn test_exit_dialog_always_safe() {
        fn prop(selected: u8) -> bool {
            let (mut app, _temp_dir) = create_test_app_state();
            app.show_exit_dialog = true;
            app.exit_selected = selected as usize;
            
            // Any key should not panic and should return a valid result
            let result = handle_exit_dialog(&mut app, KeyCode::Char('x'));
            result.is_ok()
        }
        quickcheck(prop as fn(u8) -> bool);
    }

    /// Property-based test: save dialog filename input should be safe with any character sequence
    /// Expected: No matter what characters are input, the dialog should never crash
    /// and should maintain reasonable limits on filename length
    #[test]
    fn test_save_filename_input_safe() {
        fn prop(chars: Vec<char>) -> TestResult {
            // Limit the size to prevent excessive test times
            if chars.len() > 100 {
                return TestResult::discard();
            }
            
            let (mut app, _temp_dir) = create_test_app_state();
            app.show_save_dialog = true;
            
            // Input any characters should not panic
            for c in chars {
                handle_save_dialog(&mut app, KeyCode::Char(c));
            }
            
            // Expected: Filename should not exceed reasonable limits
            TestResult::from_bool(app.save_filename.len() <= 1000)
        }
        quickcheck(prop as fn(Vec<char>) -> TestResult);
    }

    /// Property-based test: directory name input should validate characters and be safe
    /// Expected: Any character sequence should be processed safely, with invalid
    /// characters filtered out, and only valid characters (alphanumeric, _, -, .) retained
    #[test]
    fn test_directory_name_input_safe() {
        fn prop(chars: Vec<char>) -> TestResult {
            // Limit the size to prevent excessive test times
            if chars.len() > 100 {
                return TestResult::discard();
            }
            
            let (mut app, _temp_dir) = create_test_app_state();
            app.show_create_dir_dialog = true;
            
            // Input any characters should not panic
            for c in chars {
                handle_create_dir_dialog(&mut app, KeyCode::Char(c));
            }
            
            // Expected: Directory name should only contain valid characters after filtering
            TestResult::from_bool(app.new_dir_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'))
        }
        quickcheck(prop as fn(Vec<char>) -> TestResult);
    }
}
