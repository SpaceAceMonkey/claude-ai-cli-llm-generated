//! Tests for navigation behavior in dialogs
//! Focus on wrapping behavior and edge cases

use crossterm::event::KeyCode;
use std::path::PathBuf;
use tempfile::TempDir;

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
            std::fs::create_dir_all(&dir_path).expect("Failed to create test directory");
        } else {
            let file_path = dir.join(file);
            std::fs::write(&file_path, "test content").expect("Failed to create test file");
        }
    }
}

#[cfg(test)]
mod navigation_wrapping_tests {
    use super::*;

    #[test]
    fn test_save_dialog_wrapping_empty_list() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Empty directory should still have parent and create dir options
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Initially no selection
        assert!(app.file_list_state.selected().is_none());
        
        // Navigate down should select first item
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
        
        // Navigate up should select last item
        handle_save_dialog(&mut app, KeyCode::Up);
        let last_index = app.available_files.len() - 1;
        assert_eq!(app.file_list_state.selected(), Some(last_index));
    }

    #[test]
    fn test_save_dialog_wrapping_single_item() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create single file
        create_test_files(&temp_dir.path().to_path_buf(), &["single.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Select first item
        app.file_list_state.select(Some(0));
        
        // Navigate up should wrap to last
        handle_save_dialog(&mut app, KeyCode::Up);
        let last_index = app.available_files.len() - 1;
        assert_eq!(app.file_list_state.selected(), Some(last_index));
        
        // Navigate down should wrap to first
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
    }

    #[test]
    fn test_save_dialog_wrapping_multiple_items() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create multiple files
        create_test_files(&temp_dir.path().to_path_buf(), &[
            "file1.txt", "file2.txt", "file3.txt", "dir1/", "dir2/"
        ]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        let total_items = app.available_files.len();
        assert!(total_items > 3); // Should have files plus parent dir and create dir option
        
        // Test wrapping from first to last
        app.file_list_state.select(Some(0));
        handle_save_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(total_items - 1));
        
        // Test wrapping from last to first
        app.file_list_state.select(Some(total_items - 1));
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
        
        // Test normal navigation
        app.file_list_state.select(Some(1));
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(2));
        
        handle_save_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(1));
    }

    #[test]
    fn test_load_dialog_wrapping_behavior() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_load_dialog = true;
        
        // Create test files
        create_test_files(&temp_dir.path().to_path_buf(), &[
            "load1.json", "load2.json", "subdir/"
        ]);
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        let total_items = app.available_files.len();
        
        // Test wrapping from first to last
        app.file_list_state.select(Some(0));
        handle_load_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(total_items - 1));
        
        // Test wrapping from last to first
        app.file_list_state.select(Some(total_items - 1));
        handle_load_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation_with_no_selection() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt", "file2.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Start with no selection
        app.file_list_state.select(None);
        
        // Down should select first item
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
        
        // Reset to no selection
        app.file_list_state.select(None);
        
        // Up should select last item
        handle_save_dialog(&mut app, KeyCode::Up);
        let last_index = app.available_files.len() - 1;
        assert_eq!(app.file_list_state.selected(), Some(last_index));
    }

    #[test]
    fn test_navigation_boundary_conditions() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Test with selection at exact boundary
        let last_index = app.available_files.len() - 1;
        app.file_list_state.select(Some(last_index));
        
        // Should wrap to first
        handle_save_dialog(&mut app, KeyCode::Down);
        assert_eq!(app.file_list_state.selected(), Some(0));
        
        // Test with selection at zero
        app.file_list_state.select(Some(0));
        
        // Should wrap to last
        handle_save_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(last_index));
    }

    #[test]
    fn test_navigation_with_empty_files_list() {
        let (mut app, _temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Manually create empty files list (shouldn't happen in practice)
        app.available_files.clear();
        app.file_list_state.select(None);
        
        // Navigation should not crash
        handle_save_dialog(&mut app, KeyCode::Up);
        assert!(app.file_list_state.selected().is_none());
        
        handle_save_dialog(&mut app, KeyCode::Down);
        assert!(app.file_list_state.selected().is_none());
    }

    #[test]
    fn test_navigation_with_out_of_bounds_selection() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Manually set selection to out of bounds (shouldn't happen in practice)
        let total_items = app.available_files.len();
        app.file_list_state.select(Some(total_items + 10));
        
        // Navigation should handle this gracefully
        handle_save_dialog(&mut app, KeyCode::Down);
        // Should wrap to first item
        assert_eq!(app.file_list_state.selected(), Some(0));
    }
}

#[cfg(test)]
mod cross_platform_navigation_tests {
    use super::*;

    #[test]
    fn test_navigation_consistency_across_dialogs() {
        let (mut app, temp_dir) = create_test_app_state();
        
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt", "file2.txt", "file3.txt"]);
        
        // Test save dialog navigation
        app.show_save_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        let total_items = app.available_files.len();
        app.file_list_state.select(Some(0));
        
        // Test wrapping behavior
        handle_save_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(total_items - 1));
        
        // Switch to load dialog
        app.show_save_dialog = false;
        app.show_load_dialog = true;
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        
        let total_items = app.available_files.len();
        app.file_list_state.select(Some(0));
        
        // Test same wrapping behavior
        handle_load_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(total_items - 1));
    }

    #[test]
    fn test_navigation_with_special_entries() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create subdirectory structure
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir_all(&subdir).expect("Failed to create subdirectory");
        
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        // Should have parent directory, create dir option, and subdirectory
        assert!(app.available_files.contains(&"../".to_string()));
        assert!(app.available_files.contains(&"[ Create New Directory ]".to_string()));
        assert!(app.available_files.contains(&"subdir/".to_string()));
        
        // Test navigation through special entries
        let total_items = app.available_files.len();
        for i in 0..total_items {
            app.file_list_state.select(Some(i));
            
            // Navigation should work for all entries
            handle_save_dialog(&mut app, KeyCode::Up);
            handle_save_dialog(&mut app, KeyCode::Down);
            
            // Should be back to original position
            assert_eq!(app.file_list_state.selected(), Some(i));
        }
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_navigation_with_large_file_list() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        // Create many files
        let mut files = Vec::new();
        for i in 0..1000 {
            files.push(format!("file_{:04}.txt", i));
        }
        
        for file in &files {
            std::fs::write(temp_dir.path().join(file), "content").expect("Failed to create file");
        }
        
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        let total_items = app.available_files.len();
        
        // Test wrapping with large list
        app.file_list_state.select(Some(0));
        handle_save_dialog(&mut app, KeyCode::Up);
        assert_eq!(app.file_list_state.selected(), Some(total_items - 1));
        
        // Test multiple rapid navigation
        for _ in 0..100 {
            handle_save_dialog(&mut app, KeyCode::Down);
        }
        
        // Should still be valid
        assert!(app.file_list_state.selected().unwrap() < total_items);
    }

    #[test]
    fn test_rapid_navigation_changes() {
        let (mut app, temp_dir) = create_test_app_state();
        app.show_save_dialog = true;
        
        create_test_files(&temp_dir.path().to_path_buf(), &["file1.txt", "file2.txt", "file3.txt"]);
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        
        let total_items = app.available_files.len();
        app.file_list_state.select(Some(0));
        
        // Rapid up/down navigation
        for _ in 0..50 {
            handle_save_dialog(&mut app, KeyCode::Up);
            handle_save_dialog(&mut app, KeyCode::Down);
        }
        
        // Should still be valid
        assert!(app.file_list_state.selected().unwrap() < total_items);
    }
}
