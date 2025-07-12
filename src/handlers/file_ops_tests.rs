//! Tests for file operations in dialogs
//! Focus on save/load functionality, error handling, and file system interactions

use std::fs;
use tempfile::TempDir;
use serde_json;

use crate::app::AppState;
use crate::handlers::file_ops::*;
use crate::client::ConversationClient;
use crate::api::Message;

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

/// Helper function to create a test conversation with messages
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

#[cfg(test)]
mod save_conversation_tests {
    use super::*;

    #[test]
    fn test_save_conversation_success() {
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        
        let save_path = temp_dir.path().join("test_conversation.json");
        
        let result = save_conversation(&client, &save_path);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(save_path.exists());
        
        // Verify file content
        let content = fs::read_to_string(&save_path).expect("Failed to read saved file");
        let saved_conversation: SavedConversation = serde_json::from_str(&content)
            .expect("Failed to parse saved conversation");
        
        assert_eq!(saved_conversation.version, "1.0");
        assert_eq!(saved_conversation.model, "test_model");
        assert_eq!(saved_conversation.total_input_tokens, 50);
        assert_eq!(saved_conversation.total_output_tokens, 100);
        assert_eq!(saved_conversation.messages.len(), 2);
    }

    #[test]
    fn test_save_conversation_empty_messages() {
        let (_app, temp_dir) = create_test_app_state();
        let client = ConversationClient::new(
            "test_key".to_string(),
            "test_model".to_string(),
            1000,
            0.7,
        );
        
        let save_path = temp_dir.path().join("empty_conversation.json");
        
        let result = save_conversation(&client, &save_path);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(save_path.exists());
        
        // Verify empty conversation is valid
        let content = fs::read_to_string(&save_path).expect("Failed to read saved file");
        let saved_conversation: SavedConversation = serde_json::from_str(&content)
            .expect("Failed to parse saved conversation");
        
        assert_eq!(saved_conversation.messages.len(), 0);
        assert!(saved_conversation.validate());
    }

    #[test]
    fn test_save_conversation_invalid_path() {
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        
        // Try to save to a non-existent directory
        let invalid_path = temp_dir.path().join("nonexistent").join("test.json");
        
        let result = save_conversation(&client, &invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_conversation_readonly_directory() {
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        
        // Create a readonly directory (Unix-like systems)
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir_all(&readonly_dir).expect("Failed to create readonly directory");
        
        // Set readonly permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&readonly_dir, perms).expect("Failed to set readonly permissions");
        }
        
        let save_path = readonly_dir.join("test.json");
        
        let result = save_conversation(&client, &save_path);
        
        // Should fail on Unix systems
        #[cfg(unix)]
        assert!(result.is_err());
        
        // On Windows, this might succeed depending on the file system
        #[cfg(windows)]
        {
            // Just ensure it doesn't panic
            let _ = result;
        }
    }

    #[test]
    fn test_save_conversation_with_unicode() {
        let (_app, temp_dir) = create_test_app_state();
        let mut client = ConversationClient::new(
            "test_key".to_string(),
            "test_model".to_string(),
            1000,
            0.7,
        );
        
        // Add message with Unicode content
        client.messages.push(Message {
            role: "user".to_string(),
            content: "Hello 世界! 🌍 Testing unicode: αβγ".to_string(),
        });
        
        let save_path = temp_dir.path().join("unicode_conversation.json");
        
        let result = save_conversation(&client, &save_path);
        assert!(result.is_ok());
        
        // Verify Unicode content is preserved
        let content = fs::read_to_string(&save_path).expect("Failed to read saved file");
        assert!(content.contains("世界"));
        assert!(content.contains("🌍"));
        assert!(content.contains("αβγ"));
    }
}

#[cfg(test)]
mod load_conversation_tests {
    use super::*;

    #[test]
    fn test_load_conversation_success() {
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        
        let save_path = temp_dir.path().join("test_conversation.json");
        
        // First save a conversation
        save_conversation(&client, &save_path).expect("Failed to save conversation");
        
        // Then load it
        let result = load_conversation(&save_path);
        assert!(result.is_ok());
        
        let loaded_conversation = result.unwrap();
        assert_eq!(loaded_conversation.model, "test_model");
        assert_eq!(loaded_conversation.total_input_tokens, 50);
        assert_eq!(loaded_conversation.total_output_tokens, 100);
        assert_eq!(loaded_conversation.messages.len(), 2);
        assert_eq!(loaded_conversation.messages[0].role, "user");
        assert_eq!(loaded_conversation.messages[0].content, "Hello, world!");
    }

    #[test]
    fn test_load_conversation_nonexistent_file() {
        let (_app, temp_dir) = create_test_app_state();
        let nonexistent_path = temp_dir.path().join("nonexistent.json");
        
        let result = load_conversation(&nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_conversation_invalid_json() {
        let (_app, temp_dir) = create_test_app_state();
        let invalid_path = temp_dir.path().join("invalid.json");
        
        // Write invalid JSON
        fs::write(&invalid_path, "{ invalid json }").expect("Failed to write invalid JSON");
        
        let result = load_conversation(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_conversation_invalid_format() {
        let (_app, temp_dir) = create_test_app_state();
        let invalid_path = temp_dir.path().join("invalid_format.json");
        
        // Write valid JSON but invalid conversation format
        let invalid_conversation = r#"{
            "version": "2.0",
            "model": "test_model",
            "messages": []
        }"#;
        
        fs::write(&invalid_path, invalid_conversation).expect("Failed to write invalid conversation");
        
        let result = load_conversation(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_conversation_missing_fields() {
        let (_app, temp_dir) = create_test_app_state();
        let incomplete_path = temp_dir.path().join("incomplete.json");
        
        // Write JSON missing required fields
        let incomplete_conversation = r#"{
            "version": "1.0",
            "model": "test_model"
        }"#;
        
        fs::write(&incomplete_path, incomplete_conversation).expect("Failed to write incomplete conversation");
        
        let result = load_conversation(&incomplete_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_conversation_with_unicode() {
        let (_app, temp_dir) = create_test_app_state();
        let mut client = ConversationClient::new(
            "test_key".to_string(),
            "test_model".to_string(),
            1000,
            0.7,
        );
        
        // Add message with Unicode content
        client.messages.push(Message {
            role: "user".to_string(),
            content: "Hello 世界! 🌍 Testing unicode: αβγ".to_string(),
        });
        
        let save_path = temp_dir.path().join("unicode_conversation.json");
        
        // Save and load
        save_conversation(&client, &save_path).expect("Failed to save conversation");
        let loaded_conversation = load_conversation(&save_path).expect("Failed to load conversation");
        
        // Verify Unicode content is preserved
        assert_eq!(loaded_conversation.messages[0].content, "Hello 世界! 🌍 Testing unicode: αβγ");
    }
}

#[cfg(test)]
mod directory_operations_tests {
    use super::*;

    #[test]
    fn test_load_directory_contents_empty() {
        let (_app, temp_dir) = create_test_app_state();
        let mut files = Vec::new();
        
        load_directory_contents(&mut files, &temp_dir.path().to_path_buf(), true);
        
        // Should have parent directory and create directory option
        assert!(files.contains(&"../".to_string()));
        assert!(files.contains(&"[ Create New Directory ]".to_string()));
        assert!(files.contains(&"(Empty directory)".to_string()));
    }

    #[test]
    fn test_load_directory_contents_with_files() {
        let (_app, temp_dir) = create_test_app_state();
        let mut files = Vec::new();
        
        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "content1").expect("Failed to create file1");
        fs::write(temp_dir.path().join("file2.txt"), "content2").expect("Failed to create file2");
        fs::create_dir_all(temp_dir.path().join("subdir")).expect("Failed to create subdir");
        
        load_directory_contents(&mut files, &temp_dir.path().to_path_buf(), true);
        
        // Should have parent directory, create directory option, subdirectory, and files
        assert!(files.contains(&"../".to_string()));
        assert!(files.contains(&"[ Create New Directory ]".to_string()));
        assert!(files.contains(&"subdir/".to_string()));
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));
        assert!(!files.contains(&"(Empty directory)".to_string()));
    }

    #[test]
    fn test_load_directory_contents_hidden_files() {
        let (_app, temp_dir) = create_test_app_state();
        let mut files = Vec::new();
        
        // Create hidden files and directories
        fs::write(temp_dir.path().join(".hidden_file"), "content").expect("Failed to create hidden file");
        fs::create_dir_all(temp_dir.path().join(".hidden_dir")).expect("Failed to create hidden dir");
        fs::write(temp_dir.path().join("visible_file.txt"), "content").expect("Failed to create visible file");
        
        load_directory_contents(&mut files, &temp_dir.path().to_path_buf(), true);
        
        // Should show hidden directories but not hidden files
        assert!(files.contains(&".hidden_dir/".to_string()));
        assert!(!files.contains(&".hidden_file".to_string()));
        assert!(files.contains(&"visible_file.txt".to_string()));
    }

    #[test]
    fn test_load_directory_contents_sorting() {
        let (_app, temp_dir) = create_test_app_state();
        let mut files = Vec::new();
        
        // Create files and directories in random order
        fs::write(temp_dir.path().join("z_file.txt"), "content").expect("Failed to create z_file");
        fs::write(temp_dir.path().join("a_file.txt"), "content").expect("Failed to create a_file");
        fs::create_dir_all(temp_dir.path().join("z_dir")).expect("Failed to create z_dir");
        fs::create_dir_all(temp_dir.path().join("a_dir")).expect("Failed to create a_dir");
        
        load_directory_contents(&mut files, &temp_dir.path().to_path_buf(), true);
        
        // Find indices of directories and files
        let a_dir_idx = files.iter().position(|f| f == "a_dir/").unwrap();
        let z_dir_idx = files.iter().position(|f| f == "z_dir/").unwrap();
        let a_file_idx = files.iter().position(|f| f == "a_file.txt").unwrap();
        let z_file_idx = files.iter().position(|f| f == "z_file.txt").unwrap();
        
        // Directories should come before files
        assert!(a_dir_idx < a_file_idx);
        assert!(z_dir_idx < z_file_idx);
        
        // Items should be sorted within their categories
        assert!(a_dir_idx < z_dir_idx);
        assert!(a_file_idx < z_file_idx);
    }

    #[test]
    fn test_load_directory_contents_save_vs_load() {
        let (_app, temp_dir) = create_test_app_state();
        let mut save_files = Vec::new();
        let mut load_files = Vec::new();
        
        // Create test files
        fs::write(temp_dir.path().join("file.txt"), "content").expect("Failed to create file");
        
        load_directory_contents(&mut save_files, &temp_dir.path().to_path_buf(), true);
        load_directory_contents(&mut load_files, &temp_dir.path().to_path_buf(), false);
        
        // Save dialog should have create directory option
        assert!(save_files.contains(&"[ Create New Directory ]".to_string()));
        assert!(!load_files.contains(&"[ Create New Directory ]".to_string()));
        
        // Both should have parent directory and file
        assert!(save_files.contains(&"../".to_string()));
        assert!(load_files.contains(&"../".to_string()));
        assert!(save_files.contains(&"file.txt".to_string()));
        assert!(load_files.contains(&"file.txt".to_string()));
    }

    #[test]
    fn test_get_saves_directory() {
        let saves_dir = get_saves_directory();
        
        // Should return a valid path
        assert!(saves_dir.is_absolute() || saves_dir.as_os_str() == ".");
        
        // Should not panic
        let _ = saves_dir.exists();
    }

    #[test]
    fn test_load_directory_contents_permission_error() {
        let (_app, temp_dir) = create_test_app_state();
        let mut files = Vec::new();
        
        // Create a directory with no read permissions (Unix-like systems)
        let no_read_dir = temp_dir.path().join("no_read");
        fs::create_dir_all(&no_read_dir).expect("Failed to create no_read directory");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&no_read_dir).unwrap().permissions();
            perms.set_mode(0o000); // No permissions
            fs::set_permissions(&no_read_dir, perms).expect("Failed to set no permissions");
        }
        
        // Should not panic, just return empty list
        load_directory_contents(&mut files, &no_read_dir, true);
        
        // Should have at least the parent directory
        assert!(files.contains(&"../".to_string()));
    }
}

#[cfg(test)]
mod saved_conversation_tests {
    use super::*;

    #[test]
    fn test_saved_conversation_new() {
        let client = create_test_conversation();
        let saved_conversation = SavedConversation::new(&client);
        
        assert_eq!(saved_conversation.version, "1.0");
        assert_eq!(saved_conversation.model, "test_model");
        assert_eq!(saved_conversation.total_input_tokens, 50);
        assert_eq!(saved_conversation.total_output_tokens, 100);
        assert_eq!(saved_conversation.messages.len(), 2);
        assert!(!saved_conversation.timestamp.is_empty());
    }

    #[test]
    fn test_saved_conversation_validate() {
        let client = create_test_conversation();
        let saved_conversation = SavedConversation::new(&client);
        
        assert!(saved_conversation.validate());
        
        // Test invalid version
        let mut invalid_conversation = saved_conversation.clone();
        invalid_conversation.version = "2.0".to_string();
        assert!(!invalid_conversation.validate());
    }

    #[test]
    fn test_saved_conversation_empty_messages() {
        let client = ConversationClient::new(
            "test_key".to_string(),
            "test_model".to_string(),
            1000,
            0.7,
        );
        let saved_conversation = SavedConversation::new(&client);
        
        assert!(saved_conversation.validate());
        assert_eq!(saved_conversation.messages.len(), 0);
    }

    #[test]
    fn test_saved_conversation_serialization() {
        let client = create_test_conversation();
        let saved_conversation = SavedConversation::new(&client);
        
        // Test serialization
        let json = serde_json::to_string(&saved_conversation).expect("Failed to serialize");
        assert!(json.contains("version"));
        assert!(json.contains("1.0"));
        assert!(json.contains("test_model"));
        
        // Test deserialization
        let deserialized: SavedConversation = serde_json::from_str(&json)
            .expect("Failed to deserialize");
        
        assert_eq!(deserialized.version, saved_conversation.version);
        assert_eq!(deserialized.model, saved_conversation.model);
        assert_eq!(deserialized.messages.len(), saved_conversation.messages.len());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_file_operations_with_long_paths() {
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        
        // Create a very long path
        let long_name = "a".repeat(100);
        let long_path = temp_dir.path().join(&long_name).with_extension("json");
        
        // Should handle long paths gracefully
        let result = save_conversation(&client, &long_path);
        
        // May succeed or fail depending on filesystem limits
        // But should not panic
        let _ = result;
    }

    #[test]
    fn test_file_operations_with_special_characters() {
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        
        // Test with various special characters in filename
        let special_chars = vec![
            "test with spaces.json",
            "test-with-dashes.json",
            "test_with_underscores.json",
            "test.with.dots.json",
        ];
        
        for filename in special_chars {
            let path = temp_dir.path().join(filename);
            let result = save_conversation(&client, &path);
            
            // Should not panic
            let _ = result;
        }
    }

    #[test]
    fn test_concurrent_file_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let (_app, temp_dir) = create_test_app_state();
        let client = create_test_conversation();
        let temp_path = Arc::new(temp_dir.path().to_path_buf());
        
        // Spawn multiple threads trying to save to different files
        let mut handles = vec![];
        
        for i in 0..10 {
            let client = client.clone();
            let temp_path = temp_path.clone();
            
            let handle = thread::spawn(move || {
                let filename = format!("concurrent_test_{}.json", i);
                let path = temp_path.join(filename);
                save_conversation(&client, &path)
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().expect("Thread panicked");
            // Each thread should succeed or fail gracefully
            let _ = result;
        }
    }
}
