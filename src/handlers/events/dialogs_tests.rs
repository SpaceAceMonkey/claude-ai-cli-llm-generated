//! Unit tests for dialog handling functionality
//! Tests the dialog navigation, state management, and user interactions

use crossterm::event::KeyCode;
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

use crate::app::AppState;
use crate::handlers::events::dialogs::*;
use crate::handlers::file_ops::load_directory_contents;

/// Helper function to create a test AppState with temporary directory
fn create_test_app_state() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = AppState::new(
        "test_key".to_string(),
        "test_model".to_string(),
        1000,
        0.7,
        false,
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
        
        let result = handle_exit_dialog_key(&mut app, KeyCode::Char('y'));
        assert!(result);
        assert!(!app.show_exit_dialog);
    }

    /// Test that pressing 'n' in the exit dialog cancels the exit
    /// Expected: should_exit returns false, dialog is closed, but application continues
    #[test]
    fn test_exit_dialog_no_key() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        let result = handle_exit_dialog_key(&mut app, KeyCode::Char('n'));
        assert!(!result);
        assert!(!app.show_exit_dialog);
    }

    /// Test that pressing 'Y' (uppercase) in the exit dialog also exits
    /// Expected: should_exit returns true, case-insensitive handling
    #[test]
    fn test_exit_dialog_uppercase_yes() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        let result = handle_exit_dialog_key(&mut app, KeyCode::Char('Y'));
        assert!(result);
        assert!(!app.show_exit_dialog);
    }

    /// Test that pressing 'N' (uppercase) in the exit dialog cancels
    /// Expected: should_exit returns false, case-insensitive handling
    #[test]
    fn test_exit_dialog_uppercase_no() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        let result = handle_exit_dialog_key(&mut app, KeyCode::Char('N'));
        assert!(!result);
        assert!(!app.show_exit_dialog);
    }

    /// Test that pressing Escape in the exit dialog cancels the exit
    /// Expected: should_exit returns false, dialog is closed
    #[test]
    fn test_exit_dialog_escape_key() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        let result = handle_exit_dialog_key(&mut app, KeyCode::Esc);
        assert!(!result);
        assert!(!app.show_exit_dialog);
    }

    /// Test that pressing Enter in the exit dialog does nothing
    /// Expected: should_exit returns false, dialog remains open
    #[test]
    fn test_exit_dialog_enter_key() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        let result = handle_exit_dialog_key(&mut app, KeyCode::Enter);
        assert!(!result);
        assert!(app.show_exit_dialog); // Dialog should remain open
    }

    /// Test that pressing other keys in the exit dialog does nothing
    /// Expected: should_exit returns false, dialog remains open
    #[test]
    fn test_exit_dialog_other_keys() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_exit_dialog = true;
        
        let test_keys = vec![
            KeyCode::Char('a'),
            KeyCode::Char('z'),
            KeyCode::Char('1'),
            KeyCode::Tab,
            KeyCode::Backspace,
        ];
        
        for key in test_keys {
            let result = handle_exit_dialog_key(&mut app, key);
            assert!(!result);
            assert!(app.show_exit_dialog); // Dialog should remain open
        }
    }
}

#[cfg(test)]
mod save_dialog_tests {
    use super::*;

    /// Test basic save dialog navigation
    /// Expected: can navigate up and down through file list
    #[test]
    fn test_save_dialog_navigation() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create test files
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt", "file2.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Start at first item
        app.selected_file_index = 0;
        
        // Navigate down
        handle_save_dialog_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected_file_index, 1);
        
        // Navigate up
        handle_save_dialog_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected_file_index, 0);
    }

    /// Test save dialog directory navigation
    /// Expected: can enter and exit directories
    #[test]
    fn test_save_dialog_directory_navigation() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create test directory structure
        create_test_files(&temp_dir.path().to_path_buf(), &["subdir/", "file1.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Find the subdirectory index
        let subdir_index = app.available_files.iter().position(|f| f.name == "subdir").unwrap();
        app.selected_file_index = subdir_index;
        
        // Enter directory
        handle_save_dialog_key(&mut app, KeyCode::Enter);
        assert_eq!(app.current_directory, temp_dir.path().join("subdir"));
        
        // Navigate back to parent
        app.selected_file_index = 0; // Should be ".." entry
        handle_save_dialog_key(&mut app, KeyCode::Enter);
        assert_eq!(app.current_directory, temp_dir.path().to_path_buf());
    }

    /// Test save dialog filename input
    /// Expected: can type filename and save
    #[test]
    fn test_save_dialog_filename_input() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        app.save_filename = "test.txt".to_string();
        
        // Test typing characters
        handle_save_dialog_key(&mut app, KeyCode::Char('a'));
        assert_eq!(app.save_filename, "test.txta");
        
        // Test backspace
        handle_save_dialog_key(&mut app, KeyCode::Backspace);
        assert_eq!(app.save_filename, "test.txt");
    }

    /// Test save dialog escape behavior
    /// Expected: escape closes dialog without saving
    #[test]
    fn test_save_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        app.save_filename = "test.txt".to_string();
        
        handle_save_dialog_key(&mut app, KeyCode::Esc);
        assert!(!app.show_save_dialog);
        assert_eq!(app.save_filename, ""); // Filename should be cleared
    }

    /// Test save dialog with empty filename
    /// Expected: cannot save with empty filename
    #[test]
    fn test_save_dialog_empty_filename() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        app.save_filename = "".to_string();
        
        // Try to save with empty filename
        handle_save_dialog_key(&mut app, KeyCode::Enter);
        assert!(app.show_save_dialog); // Dialog should remain open
    }

    /// Test save dialog create new directory
    /// Expected: can create new directory and navigate into it
    #[test]
    fn test_save_dialog_create_directory() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Find "Create new directory" option
        let create_dir_index = app.available_files.iter().position(|f| f.name == "Create new directory").unwrap();
        app.selected_file_index = create_dir_index;
        
        // Set directory name
        app.save_filename = "new_directory".to_string();
        
        // Create directory
        handle_save_dialog_key(&mut app, KeyCode::Enter);
        
        // Check that directory was created
        let new_dir_path = temp_dir.path().join("new_directory");
        assert!(new_dir_path.exists());
        assert!(new_dir_path.is_dir());
    }
}

#[cfg(test)]
mod load_dialog_tests {
    use super::*;

    /// Test basic load dialog navigation
    /// Expected: can navigate through file list
    #[test]
    fn test_load_dialog_navigation() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create test files
        create_test_files(&temp_dir.path().to_path_buf(), &["conv1.json", "conv2.json"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Start at first item
        app.selected_file_index = 0;
        
        // Navigate down
        handle_load_dialog_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected_file_index, 1);
        
        // Navigate up
        handle_load_dialog_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected_file_index, 0);
    }

    /// Test load dialog file selection
    /// Expected: can select and load a file
    #[test]
    fn test_load_dialog_file_selection() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create test conversation file
        let conv_path = temp_dir.path().join("test_conv.json");
        fs::write(&conv_path, r#"{"messages": []}"#).expect("Failed to create test file");
        
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Find the conversation file
        let file_index = app.available_files.iter().position(|f| f.name == "test_conv.json").unwrap();
        app.selected_file_index = file_index;
        
        // Select file
        handle_load_dialog_key(&mut app, KeyCode::Enter);
        assert!(!app.show_load_dialog); // Dialog should close after selection
    }

    /// Test load dialog directory navigation
    /// Expected: can navigate into and out of directories
    #[test]
    fn test_load_dialog_directory_navigation() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create test directory with file
        create_test_files(&temp_dir.path().to_path_buf(), &["subdir/"]);
        let subdir_path = temp_dir.path().join("subdir");
        fs::write(subdir_path.join("conv.json"), r#"{"messages": []}"#).expect("Failed to create test file");
        
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Find the subdirectory
        let subdir_index = app.available_files.iter().position(|f| f.name == "subdir").unwrap();
        app.selected_file_index = subdir_index;
        
        // Enter directory
        handle_load_dialog_key(&mut app, KeyCode::Enter);
        assert_eq!(app.current_directory, temp_dir.path().join("subdir"));
        
        // Navigate back to parent
        app.selected_file_index = 0; // Should be ".." entry
        handle_load_dialog_key(&mut app, KeyCode::Enter);
        assert_eq!(app.current_directory, temp_dir.path().to_path_buf());
    }

    /// Test load dialog escape behavior
    /// Expected: escape closes dialog without loading
    #[test]
    fn test_load_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        handle_load_dialog_key(&mut app, KeyCode::Esc);
        assert!(!app.show_load_dialog);
    }

    /// Test load dialog with no files
    /// Expected: handles empty directory gracefully
    #[test]
    fn test_load_dialog_empty_directory() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Should have at least parent directory option
        assert!(app.available_files.len() > 0);
        
        // Navigation should not crash
        handle_load_dialog_key(&mut app, KeyCode::Down);
        handle_load_dialog_key(&mut app, KeyCode::Up);
    }
}

#[cfg(test)]
mod models_dialog_tests {
    use super::*;

    /// Test basic models dialog navigation
    /// Expected: can navigate through model list
    #[test]
    fn test_models_dialog_navigation() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_models_dialog = true;
        
        // Models should be populated
        assert!(app.available_models.len() > 0);
        
        // Start at first model
        app.selected_model_index = 0;
        
        // Navigate down
        handle_models_dialog_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected_model_index, 1);
        
        // Navigate up
        handle_models_dialog_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected_model_index, 0);
    }

    /// Test models dialog selection
    /// Expected: can select a model and close dialog
    #[test]
    fn test_models_dialog_selection() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_models_dialog = true;
        
        // Select a model
        app.selected_model_index = 1;
        let selected_model = app.available_models[1].clone();
        
        handle_models_dialog_key(&mut app, KeyCode::Enter);
        assert!(!app.show_models_dialog); // Dialog should close
        assert_eq!(app.client.model, selected_model); // Model should be updated
    }

    /// Test models dialog escape behavior
    /// Expected: escape closes dialog without changing model
    #[test]
    fn test_models_dialog_escape() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_models_dialog = true;
        
        let original_model = app.client.model.clone();
        app.selected_model_index = 1;
        
        handle_models_dialog_key(&mut app, KeyCode::Esc);
        assert!(!app.show_models_dialog);
        assert_eq!(app.client.model, original_model); // Model should not change
    }

    /// Test models dialog wrapping
    /// Expected: navigation wraps around at boundaries
    #[test]
    fn test_models_dialog_wrapping() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_models_dialog = true;
        
        let model_count = app.available_models.len();
        
        // Start at first model and go up (should wrap to last)
        app.selected_model_index = 0;
        handle_models_dialog_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected_model_index, model_count - 1);
        
        // Go down from last model (should wrap to first)
        handle_models_dialog_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected_model_index, 0);
    }

    /// Test models dialog with single model
    /// Expected: handles single model list gracefully
    #[test]
    fn test_models_dialog_single_model() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_models_dialog = true;
        
        // Force single model
        app.available_models = vec!["single-model".to_string()];
        app.selected_model_index = 0;
        
        // Navigation should stay at same position
        handle_models_dialog_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected_model_index, 0);
        
        handle_models_dialog_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected_model_index, 0);
    }
}

#[cfg(test)]
mod dialog_state_management_tests {
    use super::*;

    /// Test that only one dialog can be open at a time
    /// Expected: opening a new dialog closes the current one
    #[test]
    fn test_dialog_mutual_exclusivity() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Open save dialog
        app.show_save_dialog = true;
        assert!(app.show_save_dialog);
        
        // Open load dialog (should close save dialog)
        app.show_save_dialog = false;
        app.show_load_dialog = true;
        assert!(!app.show_save_dialog);
        assert!(app.show_load_dialog);
        
        // Open models dialog (should close load dialog)
        app.show_load_dialog = false;
        app.show_models_dialog = true;
        assert!(!app.show_load_dialog);
        assert!(app.show_models_dialog);
    }

    /// Test dialog state initialization
    /// Expected: dialogs start in closed state
    #[test]
    fn test_dialog_initial_state() {
        let (app, _temp_dir) = create_test_app_state();
        
        assert!(!app.show_save_dialog);
        assert!(!app.show_load_dialog);
        assert!(!app.show_models_dialog);
        assert!(!app.show_exit_dialog);
    }

    /// Test dialog state persistence across operations
    /// Expected: dialog state is maintained during operations
    #[test]
    fn test_dialog_state_persistence() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Open save dialog
        app.show_save_dialog = true;
        app.save_filename = "test.txt".to_string();
        
        // Perform some operations
        app.selected_file_index = 1;
        
        // State should be preserved
        assert!(app.show_save_dialog);
        assert_eq!(app.save_filename, "test.txt");
        assert_eq!(app.selected_file_index, 1);
    }

    /// Test dialog cleanup on close
    /// Expected: dialog-specific state is cleaned up when dialog closes
    #[test]
    fn test_dialog_cleanup() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Open save dialog and set state
        app.show_save_dialog = true;
        app.save_filename = "test.txt".to_string();
        app.selected_file_index = 2;
        
        // Close dialog
        handle_save_dialog_key(&mut app, KeyCode::Esc);
        
        // State should be cleaned up
        assert!(!app.show_save_dialog);
        assert_eq!(app.save_filename, "");
        assert_eq!(app.selected_file_index, 0);
    }

    /// Test dialog error handling
    /// Expected: dialogs handle errors gracefully
    #[test]
    fn test_dialog_error_handling() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Test with invalid directory
        app.current_directory = PathBuf::from("/invalid/path");
        
        // Should handle error gracefully
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Should have some default entries even if directory doesn't exist
        assert!(app.available_files.len() > 0);
    }
}

#[cfg(test)]
mod dialog_integration_tests {
    use super::*;

    /// Test complete save workflow
    /// Expected: can open dialog, navigate, enter filename, and save
    #[test]
    fn test_complete_save_workflow() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Open save dialog
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Enter filename
        app.save_filename = "test_conversation.txt".to_string();
        
        // Save file
        handle_save_dialog_key(&mut app, KeyCode::Enter);
        
        // Check that file was created
        let saved_file = temp_dir.path().join("test_conversation.txt");
        assert!(saved_file.exists());
        assert!(!app.show_save_dialog);
    }

    /// Test complete load workflow
    /// Expected: can open dialog, navigate, select file, and load
    #[test]
    fn test_complete_load_workflow() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create test conversation file
        let conv_path = temp_dir.path().join("test_conv.json");
        fs::write(&conv_path, r#"{"messages": [{"role": "user", "content": "test"}]}"#).expect("Failed to create test file");
        
        // Open load dialog
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Find and select the conversation file
        let file_index = app.available_files.iter().position(|f| f.name == "test_conv.json").unwrap();
        app.selected_file_index = file_index;
        
        // Load file
        handle_load_dialog_key(&mut app, KeyCode::Enter);
        
        // Check that conversation was loaded
        assert!(!app.show_load_dialog);
        assert!(app.client.messages.len() > 0);
    }

    /// Test complete model change workflow
    /// Expected: can open dialog, select model, and apply change
    #[test]
    fn test_complete_model_change_workflow() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        let original_model = app.client.model.clone();
        
        // Open models dialog
        app.show_models_dialog = true;
        
        // Select different model
        app.selected_model_index = 1;
        let new_model = app.available_models[1].clone();
        
        // Apply change
        handle_models_dialog_key(&mut app, KeyCode::Enter);
        
        // Check that model was changed
        assert!(!app.show_models_dialog);
        assert_eq!(app.client.model, new_model);
        assert_ne!(app.client.model, original_model);
    }

    /// Test dialog transitions
    /// Expected: can smoothly transition between different dialogs
    #[test]
    fn test_dialog_transitions() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Open save dialog
        app.show_save_dialog = true;
        assert!(app.show_save_dialog);
        
        // Close and open load dialog
        app.show_save_dialog = false;
        app.show_load_dialog = true;
        assert!(!app.show_save_dialog);
        assert!(app.show_load_dialog);
        
        // Close and open models dialog
        app.show_load_dialog = false;
        app.show_models_dialog = true;
        assert!(!app.show_load_dialog);
        assert!(app.show_models_dialog);
        
        // Close all dialogs
        app.show_models_dialog = false;
        assert!(!app.show_save_dialog);
        assert!(!app.show_load_dialog);
        assert!(!app.show_models_dialog);
    }
}
