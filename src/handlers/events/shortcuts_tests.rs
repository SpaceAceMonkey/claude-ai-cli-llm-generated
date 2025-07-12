//! Unit tests for keyboard shortcuts functionality
//! Tests various keyboard combinations for chat scrolling, file operations, and cross-platform compatibility

use crossterm::event::{KeyCode, KeyModifiers};
use tempfile::TempDir;

use crate::app::AppState;
use crate::handlers::events::shortcuts::*;
use crate::config::get_default_colors;

/// Helper function to create a test AppState with minimal setup
fn create_test_app_state() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut app = AppState::new(
        "test_key".to_string(),
        "test_model".to_string(),
        1000,
        0.7,
        true, // simulate_mode
        get_default_colors(),
    ).expect("Failed to create app state");
    
    // Add some test messages to enable scrolling
    app.client.messages.push(crate::api::Message {
        role: "user".to_string(),
        content: "Test message 1".to_string(),
    });
    app.client.messages.push(crate::api::Message {
        role: "assistant".to_string(),
        content: "Test response 1".to_string(),
    });
    app.client.messages.push(crate::api::Message {
        role: "user".to_string(),
        content: "Test message 2".to_string(),
    });
    
    (app, temp_dir)
}

#[test]
fn test_chat_scroll_up_keyboard_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset
    app.chat_scroll_offset = 5;
    app.auto_scroll = true;
    
    // Test Ctrl+Up
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Up, KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+Up should be handled");
    assert_eq!(app.chat_scroll_offset, 4, "Should scroll up by 1");
    assert!(!app.auto_scroll, "Auto-scroll should be disabled");
    
    // Test Alt+Up
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Up, KeyModifiers::ALT, terminal_size);
    assert!(result, "Alt+Up should be handled");
    assert_eq!(app.chat_scroll_offset, 3, "Should scroll up by 1");
    
    // Test Shift+Up
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Up, KeyModifiers::SHIFT, terminal_size);
    assert!(result, "Shift+Up should be handled");
    assert_eq!(app.chat_scroll_offset, 2, "Should scroll up by 1");
}

#[test]
fn test_chat_scroll_down_keyboard_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset
    app.chat_scroll_offset = 0;
    app.auto_scroll = false;
    
    // Test Ctrl+Down
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Down, KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+Down should be handled");
    assert!(app.chat_scroll_offset >= 0, "Should maintain valid scroll offset");
    
    // Test Alt+Down
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Down, KeyModifiers::ALT, terminal_size);
    assert!(result, "Alt+Down should be handled");
    
    // Test Shift+Down
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Down, KeyModifiers::SHIFT, terminal_size);
    assert!(result, "Shift+Down should be handled");
}

#[test]
fn test_vi_style_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset
    app.chat_scroll_offset = 5;
    
    // Test Ctrl+k (vi-style up)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('k'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+k should be handled");
    assert_eq!(app.chat_scroll_offset, 4, "Should scroll up by 1");
    
    // Test Ctrl+j (vi-style down)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('j'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+j should be handled");
    assert!(app.chat_scroll_offset >= 4, "Should maintain valid scroll offset");
}

#[test]
fn test_vi_style_half_page_scrolling() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset high enough for half-page up
    app.chat_scroll_offset = 10;
    let initial_offset = app.chat_scroll_offset;
    
    // Test Ctrl+u (vi-style half-page up)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('u'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+u should be handled");
    assert_eq!(app.chat_scroll_offset, initial_offset - 5, "Should scroll up by 5 lines");
    
    // Test Ctrl+d (vi-style half-page down)
    let current_offset = app.chat_scroll_offset;
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('d'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+d should be handled");
    assert!(app.chat_scroll_offset >= current_offset, "Should maintain or increase scroll offset");
}

#[test]
fn test_bracket_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset
    app.chat_scroll_offset = 5;
    
    // Test Ctrl+[ (up)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('['), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+[ should be handled");
    assert_eq!(app.chat_scroll_offset, 4, "Should scroll up by 1");
    
    // Test Ctrl+] (down)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char(']'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+] should be handled");
    assert!(app.chat_scroll_offset >= 4, "Should maintain valid scroll offset");
}

#[test]
fn test_minus_plus_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset
    app.chat_scroll_offset = 5;
    
    // Test Ctrl+- (up)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('-'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+- should be handled");
    assert_eq!(app.chat_scroll_offset, 4, "Should scroll up by 1");
    
    // Test Ctrl+= (down)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('='), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+= should be handled");
    assert!(app.chat_scroll_offset >= 4, "Should maintain valid scroll offset");
}

#[test]
fn test_function_key_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Set initial scroll offset
    app.chat_scroll_offset = 5;
    
    // Test F1 (up)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::F(1), KeyModifiers::empty(), terminal_size);
    assert!(result, "F1 should be handled");
    assert_eq!(app.chat_scroll_offset, 4, "Should scroll up by 1");
    
    // Test F2 (down)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::F(2), KeyModifiers::empty(), terminal_size);
    assert!(result, "F2 should be handled");
    assert!(app.chat_scroll_offset >= 4, "Should maintain valid scroll offset");
}

#[test]
fn test_file_operation_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Ensure dialogs are initially closed
    assert!(!app.show_save_dialog, "Save dialog should initially be closed");
    assert!(!app.show_load_dialog, "Load dialog should initially be closed");
    assert!(!app.show_exit_dialog, "Exit dialog should initially be closed");
    
    // Test Ctrl+s (save)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+s should be handled");
    assert!(app.show_save_dialog, "Save dialog should be shown");
    assert!(app.save_filename.is_empty(), "Save filename should be cleared");
    assert_eq!(app.dialog_cursor_pos, 0, "Dialog cursor should be at position 0");
    
    // Reset for next test
    app.show_save_dialog = false;
    
    // Test Ctrl+l (load)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('l'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+l should be handled");
    assert!(app.show_load_dialog, "Load dialog should be shown");
    assert!(app.file_list_state.selected().is_some(), "File list should have selection");
    
    // Reset for next test
    app.show_load_dialog = false;
    
    // Test Ctrl+q (quit)
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+q should be handled");
    assert!(app.show_exit_dialog, "Exit dialog should be shown");
    assert_eq!(app.exit_selected, 0, "Exit selection should default to 0 (Yes)");
}

#[test]
fn test_unhandled_shortcuts() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Test various unhandled key combinations
    let unhandled_keys = vec![
        (KeyCode::Char('a'), KeyModifiers::CONTROL),
        (KeyCode::Char('b'), KeyModifiers::CONTROL),
        (KeyCode::Char('c'), KeyModifiers::CONTROL),
        (KeyCode::Char('x'), KeyModifiers::ALT),
        (KeyCode::Char('y'), KeyModifiers::SHIFT),
        (KeyCode::Tab, KeyModifiers::empty()),
        (KeyCode::Char(' '), KeyModifiers::CONTROL),
        (KeyCode::F(5), KeyModifiers::empty()),
        (KeyCode::Insert, KeyModifiers::empty()),
    ];
    
    for (key, modifiers) in unhandled_keys {
        let result = handle_keyboard_shortcuts(&mut app, key, modifiers, terminal_size);
        assert!(!result, "Key {:?} with modifiers {:?} should not be handled", key, modifiers);
    }
}

#[test]
fn test_scroll_boundary_conditions() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Test scrolling up when already at top
    app.chat_scroll_offset = 0;
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('k'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+k should be handled");
    assert_eq!(app.chat_scroll_offset, 0, "Should remain at top when already at top");
    
    // Test auto-scroll behavior
    app.auto_scroll = true;
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('k'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+k should be handled");
    assert!(!app.auto_scroll, "Auto-scroll should be disabled when scrolling up");
}

#[test]
fn test_modifier_combinations() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Test combinations of modifiers
    app.chat_scroll_offset = 5;
    
    // Test Ctrl+Shift+Up (should still work)
    let combined_modifiers = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Up, combined_modifiers, terminal_size);
    assert!(result, "Ctrl+Shift+Up should be handled");
    assert_eq!(app.chat_scroll_offset, 4, "Should scroll up by 1");
    
    // Test Ctrl+Alt+Down (should still work)
    let combined_modifiers = KeyModifiers::CONTROL | KeyModifiers::ALT;
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Down, combined_modifiers, terminal_size);
    assert!(result, "Ctrl+Alt+Down should be handled");
}

#[test]
fn test_empty_message_list_scrolling() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Clear all messages
    app.client.messages.clear();
    app.chat_scroll_offset = 0;
    
    // Test scrolling with empty message list
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('k'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+k should be handled even with empty messages");
    assert_eq!(app.chat_scroll_offset, 0, "Should remain at 0 with empty messages");
    
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('j'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+j should be handled even with empty messages");
    assert_eq!(app.chat_scroll_offset, 0, "Should remain at 0 with empty messages");
}

#[test]
fn test_dialog_state_preservation() {
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    // Open save dialog
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+s should be handled");
    assert!(app.show_save_dialog, "Save dialog should be shown");
    
    // Try to open load dialog while save dialog is open
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('l'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+l should be handled");
    assert!(app.show_load_dialog, "Load dialog should be shown");
    // Note: The implementation allows multiple dialogs to be open simultaneously
    // This might be a design consideration to review
    
    // Try to open exit dialog while other dialogs are open
    let result = handle_keyboard_shortcuts(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL, terminal_size);
    assert!(result, "Ctrl+q should be handled");
    assert!(app.show_exit_dialog, "Exit dialog should be shown");
}

#[test]
fn test_cross_platform_compatibility() {
    // This test documents the cross-platform shortcuts designed for macOS compatibility
    let (mut app, _temp_dir) = create_test_app_state();
    let terminal_size = (80, 24);
    
    app.chat_scroll_offset = 5;
    
    // Test all the alternative shortcuts that work around macOS Terminal limitations
    let cross_platform_up_shortcuts = vec![
        (KeyCode::Char('k'), KeyModifiers::CONTROL),
        (KeyCode::Char('['), KeyModifiers::CONTROL),
        (KeyCode::Char('-'), KeyModifiers::CONTROL),
        (KeyCode::F(1), KeyModifiers::empty()),
    ];
    
    for (key, modifiers) in cross_platform_up_shortcuts {
        let initial_offset = app.chat_scroll_offset;
        let result = handle_keyboard_shortcuts(&mut app, key, modifiers, terminal_size);
        assert!(result, "Cross-platform up shortcut {:?} with {:?} should be handled", key, modifiers);
        assert_eq!(app.chat_scroll_offset, initial_offset - 1, "Should scroll up by 1");
        // Reset for next test
        app.chat_scroll_offset = initial_offset;
    }
    
    let cross_platform_down_shortcuts = vec![
        (KeyCode::Char('j'), KeyModifiers::CONTROL),
        (KeyCode::Char(']'), KeyModifiers::CONTROL),
        (KeyCode::Char('='), KeyModifiers::CONTROL),
        (KeyCode::F(2), KeyModifiers::empty()),
    ];
    
    for (key, modifiers) in cross_platform_down_shortcuts {
        let result = handle_keyboard_shortcuts(&mut app, key, modifiers, terminal_size);
        assert!(result, "Cross-platform down shortcut {:?} with {:?} should be handled", key, modifiers);
        // We don't check exact scroll values for down scrolling as it depends on content
    }
}
