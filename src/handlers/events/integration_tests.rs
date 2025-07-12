//! Integration tests for dialog functionality
//! Tests complete workflows and interactions between components

use crossterm::event::KeyCode;
use std::fs;
use tempfile::TempDir;

use crate::app::AppState;
use crate::handlers::events::dialogs::*;
use crate::handlers::file_ops::*;
use crate::client::ConversationClient;
use crate::api::Message;
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

/// Helper function to create a conversation with test data
fn add_test_conversation(app: &mut AppState) {
    app.client.messages.push(Message {
        role: "user".to_string(),
        content: "What is the capital of France?".to_string(),
    });
    
    app.client.messages.push(Message {
        role: "assistant".to_string(),
        content: "The capital of France is Paris.".to_string(),
    });
    
    app.client.total_input_tokens = 25;
    app.client.total_output_tokens = 50;
}

#[cfg(test)]
mod complete_workflow_tests {
    use super::*;

    #[test]
    fn test_complete_save_workflow() {
        let (mut app, temp_dir) = create_test_app_state();
        add_test_conversation(&mut app);
        
        // Start save dialog
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Enter filename
        let filename = "test_conversation.json";
        for c in filename.chars() {
            handle_save_dialog(&mut app, KeyCode::Char(c));
        }
        
        // Confirm save
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Verify save was successful
        assert!(!app.show_save_dialog);
        assert!(app.status.contains("Conversation saved"));
        
        // Verify file exists
        let saved_file = temp_dir.path().join(filename);
        assert!(saved_file.exists());
        
        // Verify file content
        let loaded_conversation = load_conversation(&saved_file)
            .expect("Failed to load saved conversation");
        assert_eq!(loaded_conversation.messages.len(), 2);
        assert_eq!(loaded_conversation.total_input_tokens, 25);
        assert_eq!(loaded_conversation.total_output_tokens, 50);
    }

    #[test]
    fn test_complete_load_workflow() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create a conversation file first
        let original_conversation = create_test_conversation();
        let test_file = temp_dir.path().join("test_load.json");
        save_conversation(&original_conversation, &test_file)
            .expect("Failed to save test conversation");
        
        // Start load dialog
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Find and select the test file
        let file_index = app.available_files.iter()
            .position(|f| f == "test_load.json")
            .expect("Test file not found");
        app.file_list_state.select(Some(file_index));
        
        // Load the file
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Verify load was successful
        assert!(!app.show_load_dialog);
        assert!(app.status.contains("Conversation loaded"));
        
        // Verify conversation was loaded
        assert_eq!(app.client.messages.len(), 2);
        assert_eq!(app.client.total_input_tokens, 50);
        assert_eq!(app.client.total_output_tokens, 100);
    }

    #[test]
    fn test_save_then_load_workflow() {
        let (mut app, _temp_dir) = create_test_app_state();
        add_test_conversation(&mut app);
        
        let filename = "roundtrip_test.json";
        
        // Save the conversation
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        for c in filename.chars() {
            handle_save_dialog(&mut app, KeyCode::Char(c));
        }
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Clear the conversation
        app.client.messages.clear();
        app.client.total_input_tokens = 0;
        app.client.total_output_tokens = 0;
        
        // Load the conversation back
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        let file_index = app.available_files.iter()
            .position(|f| f == filename)
            .expect("Saved file not found");
        app.file_list_state.select(Some(file_index));
        
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Verify the conversation was restored
        assert_eq!(app.client.messages.len(), 2);
        assert_eq!(app.client.messages[0].content, "What is the capital of France?");
        assert_eq!(app.client.messages[1].content, "The capital of France is Paris.");
        assert_eq!(app.client.total_input_tokens, 25);
        assert_eq!(app.client.total_output_tokens, 50);
    }

    #[test]
    fn test_directory_navigation_workflow() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create subdirectory structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdirectory");
        
        let subsubdir = subdir.join("subsubdir");
        fs::create_dir_all(&subsubdir).expect("Failed to create subsubdirectory");
        
        // Create a file in the subdirectory
        fs::write(subsubdir.join("deep_file.txt"), "content")
            .expect("Failed to create deep file");
        
        // Start in root directory
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Navigate to subdir
        let subdir_index = app.available_files.iter()
            .position(|f| f == "subdir/")
            .expect("Subdir not found");
        app.file_list_state.select(Some(subdir_index));
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should be in subdir now
        assert_eq!(app.current_directory, subdir);
        
        // Navigate to subsubdir
        let subsubdir_index = app.available_files.iter()
            .position(|f| f == "subsubdir/")
            .expect("Subsubdir not found");
        app.file_list_state.select(Some(subsubdir_index));
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should be in subsubdir now
        assert_eq!(app.current_directory, subsubdir);
        
        // Should see the deep file
        assert!(app.available_files.contains(&"deep_file.txt".to_string()));
        
        // Navigate back using parent directory
        let parent_index = app.available_files.iter()
            .position(|f| f == "../")
            .expect("Parent dir not found");
        app.file_list_state.select(Some(parent_index));
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should be back in subdir
        assert_eq!(app.current_directory, subdir);
    }

    #[test]
    fn test_create_directory_workflow() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Start save dialog
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Select create directory option
        let create_dir_index = app.available_files.iter()
            .position(|f| f == "[ Create New Directory ]")
            .expect("Create directory option not found");
        app.file_list_state.select(Some(create_dir_index));
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should open create directory dialog
        assert!(app.show_create_dir_dialog);
        
        // Enter directory name
        let dirname = "new_test_dir";
        for c in dirname.chars() {
            handle_create_dir_dialog(&mut app, KeyCode::Char(c));
        }
        
        // Create the directory
        handle_create_dir_dialog(&mut app, KeyCode::Enter);
        
        // Should have created directory and navigated to it
        assert!(!app.show_create_dir_dialog);
        assert_eq!(app.current_directory, temp_dir.path().join(dirname));
        
        // Directory should exist
        assert!(app.current_directory.exists());
        assert!(app.current_directory.is_dir());
    }

    fn create_test_conversation() -> ConversationClient {
        let mut client = ConversationClient::new(
            "test_key".to_string(),
            "test_model".to_string(),
            1000,
            0.7,
        );
        
        client.messages.push(Message {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
        });
        
        client.messages.push(Message {
            role: "assistant".to_string(),
            content: "Hello! How can I help you today?".to_string(),
        });
        
        client.total_input_tokens = 50;
        client.total_output_tokens = 100;
        
        client
    }
}

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_save_error_recovery() {
        let (mut app, _temp_dir) = create_test_app_state();
        add_test_conversation(&mut app);
        
        // Start save dialog
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Try to save with invalid filename (contains path separator)
        let invalid_filename = "invalid/filename.json";
        for c in invalid_filename.chars() {
            handle_save_dialog(&mut app, KeyCode::Char(c));
        }
        
        // Attempt to save
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should show error and keep dialog open
        assert!(app.status.contains("Save failed") || app.status.contains("error"));
        
        // Clear filename and try again with valid name
        app.save_filename.clear();
        app.dialog_cursor_pos = 0;
        
        let valid_filename = "valid_filename.json";
        for c in valid_filename.chars() {
            handle_save_dialog(&mut app, KeyCode::Char(c));
        }
        
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should succeed this time
        assert!(!app.show_save_dialog);
        assert!(app.status.contains("Conversation saved"));
    }

    #[test]
    fn test_load_error_recovery() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create invalid JSON file
        let invalid_file = temp_dir.path().join("invalid.json");
        fs::write(&invalid_file, "{ invalid json content")
            .expect("Failed to create invalid file");
        
        // Create valid JSON file
        let valid_conversation = create_test_conversation();
        let valid_file = temp_dir.path().join("valid.json");
        save_conversation(&valid_conversation, &valid_file)
            .expect("Failed to save valid conversation");
        
        // Start load dialog
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Try to load invalid file
        let invalid_index = app.available_files.iter()
            .position(|f| f == "invalid.json")
            .expect("Invalid file not found");
        app.file_list_state.select(Some(invalid_index));
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Should show error and keep dialog open
        assert!(app.status.contains("Load failed") || app.status.contains("error"));
        assert!(app.show_load_dialog);
        
        // Try to load valid file
        let valid_index = app.available_files.iter()
            .position(|f| f == "valid.json")
            .expect("Valid file not found");
        app.file_list_state.select(Some(valid_index));
        handle_load_dialog(&mut app, KeyCode::Enter);
        
        // Should succeed this time
        assert!(!app.show_load_dialog);
        assert!(app.status.contains("Conversation loaded"));
    }

    #[test]
    fn test_directory_navigation_error_recovery() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create directory without read permissions
        let no_read_dir = temp_dir.path().join("no_read");
        fs::create_dir_all(&no_read_dir).expect("Failed to create no_read directory");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&no_read_dir).unwrap().permissions();
            perms.set_mode(0o000); // No permissions
            fs::set_permissions(&no_read_dir, perms).expect("Failed to set no permissions");
        }
        
        // Start save dialog
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Try to navigate to directory without read permissions
        if let Some(no_read_index) = app.available_files.iter().position(|f| f == "no_read/") {
            app.file_list_state.select(Some(no_read_index));
            handle_save_dialog(&mut app, KeyCode::Enter);
            
            // On Unix, the directory change may still succeed but the load will fail
            // The behavior depends on the specific dialog implementation
            #[cfg(unix)]
            {
                // The dialog should still be functional, regardless of directory state
                assert!(app.show_save_dialog);
            }
        }
        
        // Create accessible directory
        let accessible_dir = temp_dir.path().join("accessible");
        fs::create_dir_all(&accessible_dir).expect("Failed to create accessible directory");
        
        // Reset to original directory to ensure test is deterministic
        app.current_directory = temp_dir.path().to_path_buf();
        
        // Reload directory contents
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Navigate to accessible directory
        let accessible_index = app.available_files.iter()
            .position(|f| f == "accessible/")
            .expect("Accessible directory not found");
        app.file_list_state.select(Some(accessible_index));
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Should succeed
        assert_eq!(app.current_directory, accessible_dir);
    }

    fn create_test_conversation() -> ConversationClient {
        let mut client = ConversationClient::new(
            "test_key".to_string(),
            "test_model".to_string(),
            1000,
            0.7,
        );
        
        client.messages.push(Message {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
        });
        
        client.messages.push(Message {
            role: "assistant".to_string(),
            content: "Hello! How can I help you today?".to_string(),
        });
        
        client.total_input_tokens = 50;
        client.total_output_tokens = 100;
        
        client
    }
}

#[cfg(test)]
mod dialog_interaction_tests {
    use super::*;

    #[test]
    fn test_multiple_dialog_interactions() {
        let (mut app, _temp_dir) = create_test_app_state();
        add_test_conversation(&mut app);
        
        // Save conversation
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        let filename = "multi_dialog_test.json";
        for c in filename.chars() {
            handle_save_dialog(&mut app, KeyCode::Char(c));
        }
        handle_save_dialog(&mut app, KeyCode::Enter);
        
        // Open load dialog
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        // Cancel load dialog
        handle_load_dialog(&mut app, KeyCode::Esc);
        assert!(!app.show_load_dialog);
        
        // Open save dialog again
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Cancel save dialog
        handle_save_dialog(&mut app, KeyCode::Esc);
        assert!(!app.show_save_dialog);
        
        // Open exit dialog
        app.show_exit_dialog = true;
        
        // Choose No
        handle_exit_dialog(&mut app, KeyCode::Char('n')).expect("Failed to handle exit dialog");
        assert!(!app.show_exit_dialog);
        
        // All dialogs should be closed
        assert!(!app.show_save_dialog);
        assert!(!app.show_load_dialog);
        assert!(!app.show_exit_dialog);
        assert!(!app.show_create_dir_dialog);
    }

    #[test]
    fn test_dialog_state_isolation() {
        let (mut app, _temp_dir) = create_test_app_state();
        
        // Set up save dialog state
        app.show_save_dialog = true;
        app.save_filename = "test_save.json".to_string();
        app.dialog_cursor_pos = 5;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        app.file_list_state.select(Some(2));
        
        // Close save dialog
        handle_save_dialog(&mut app, KeyCode::Esc);
        
        // Verify save dialog state was reset
        assert!(!app.show_save_dialog);
        assert_eq!(app.save_filename, "");
        assert_eq!(app.dialog_cursor_pos, 0);
        
        // Open load dialog
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        app.file_list_state.select(Some(1));
        
        // Close load dialog
        handle_load_dialog(&mut app, KeyCode::Esc);
        
        // Verify load dialog state was reset
        assert!(!app.show_load_dialog);
        
        // Open create directory dialog
        app.show_create_dir_dialog = true;
        app.new_dir_name = "test_dir".to_string();
        
        // Close create directory dialog
        handle_create_dir_dialog(&mut app, KeyCode::Esc);
        
        // Verify create directory dialog state was reset
        assert!(!app.show_create_dir_dialog);
        assert_eq!(app.new_dir_name, "");
    }

    #[test]
    fn test_dialog_navigation_consistency() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "content1").expect("Failed to create file1");
        fs::write(temp_dir.path().join("file2.txt"), "content2").expect("Failed to create file2");
        
        // Test save dialog navigation
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        let save_file_count = app.available_files.len();
        app.file_list_state.select(Some(0));
        
        // Navigate through all items
        for i in 0..save_file_count {
            assert_eq!(app.file_list_state.selected(), Some(i));
            handle_save_dialog(&mut app, KeyCode::Down);
        }
        
        // Should have wrapped to first
        assert_eq!(app.file_list_state.selected(), Some(0));
        
        // Switch to load dialog
        app.show_save_dialog = false;
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        let load_file_count = app.available_files.len();
        app.file_list_state.select(Some(0));
        
        // Navigate through all items
        for i in 0..load_file_count {
            assert_eq!(app.file_list_state.selected(), Some(i));
            handle_load_dialog(&mut app, KeyCode::Down);
        }
        
        // Should have wrapped to first
        assert_eq!(app.file_list_state.selected(), Some(0));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_directory_performance() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create many files
        for i in 0..1000 {
            let filename = format!("file_{:04}.txt", i);
            fs::write(temp_dir.path().join(&filename), format!("content {}", i))
                .expect("Failed to create file");
        }
        
        // Load directory contents
        let start = std::time::Instant::now();
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        let duration = start.elapsed();
        
        // Should complete within reasonable time
        assert!(duration.as_secs() < 5);
        
        // Should have loaded all files
        assert!(app.available_files.len() > 1000);
        
        // Test navigation performance
        app.show_save_dialog = true;
        app.file_list_state.select(Some(0));
        
        let start = std::time::Instant::now();
        for _ in 0..100 {
            handle_save_dialog(&mut app, KeyCode::Down);
        }
        let duration = start.elapsed();
        
        // Navigation should be fast
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_frequent_dialog_operations() {
        let (mut app, temp_dir) = create_test_app_state();
        
        // Create test files
        fs::write(temp_dir.path().join("test1.txt"), "content1").expect("Failed to create test1");
        fs::write(temp_dir.path().join("test2.txt"), "content2").expect("Failed to create test2");
        
        let start = std::time::Instant::now();
        
        // Perform many dialog operations
        for _i in 0..100 {
            // Open save dialog
            app.show_save_dialog = true;
            load_directory_contents(&mut app.available_files, &app.current_directory, true);
            
            // Navigate
            handle_save_dialog(&mut app, KeyCode::Down);
            handle_save_dialog(&mut app, KeyCode::Up);
            
            // Close dialog
            handle_save_dialog(&mut app, KeyCode::Esc);
            
            // Open load dialog
            app.show_load_dialog = true;
            load_directory_contents(&mut app.available_files, &app.current_directory, false);
            
            // Navigate
            handle_load_dialog(&mut app, KeyCode::Down);
            handle_load_dialog(&mut app, KeyCode::Up);
            
            // Close dialog
            handle_load_dialog(&mut app, KeyCode::Esc);
        }
        
        let duration = start.elapsed();
        
        // Should complete within reasonable time
        assert!(duration.as_secs() < 10);
    }
}
