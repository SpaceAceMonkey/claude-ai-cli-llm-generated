use crate::app::AppState;
use crate::config::{AnsiColor, ColorConfig};
use crate::handlers::events::dialogs::handle_color_dialog;
use crossterm::event::KeyCode;

#[test]
fn test_color_dialog_scrolling() {
    let mut app = AppState {
        // Initialize with minimal required fields for testing
        input: String::new(),
        conversation: Vec::new(),
        is_api_loading: false,
        api_key: "test".to_string(),
        model: "test".to_string(),
        max_tokens: 1024,
        temperature: 0.7,
        scroll_offset: 0,
        exit_selected: 0,
        show_exit_dialog: false,
        show_save_dialog: false,
        save_filename: String::new(),
        show_load_dialog: false,
        show_create_dir_dialog: false,
        new_dir_name: String::new(),
        current_directory: std::path::PathBuf::from("/tmp"),
        available_files: Vec::new(),
        file_list_state: Default::default(),
        show_error_dialog: false,
        error_message: String::new(),
        simulate_api: false,
        syntax_highlighting: true,
        highlight_cache: Default::default(),
        colors: ColorConfig::default(),
        show_color_dialog: true,
        color_dialog_selection: 0,
        color_dialog_option: 0,
        color_dialog_scroll_offset: 0,
    };

    // Test initial state
    assert_eq!(app.color_dialog_option, 0);
    assert_eq!(app.color_dialog_scroll_offset, 0);

    // Test navigating down through all colors
    let total_colors = AnsiColor::all().len();
    for i in 0..total_colors {
        handle_color_dialog(&mut app, KeyCode::Down);
        
        // The selection should wrap around when reaching the end
        let expected_selection = (i + 1) % total_colors;
        assert_eq!(app.color_dialog_option, expected_selection);
        
        // The scroll offset should update to keep the selection visible
        // For the first 6 items, scroll offset should be 0
        // After that, it should start scrolling
        if expected_selection == 0 {
            // Wrapped around, scroll should reset to 0
            assert_eq!(app.color_dialog_scroll_offset, 0);
        } else if expected_selection < 6 {
            // Still within the first visible area
            assert_eq!(app.color_dialog_scroll_offset, 0);
        } else {
            // Should be scrolling
            assert!(app.color_dialog_scroll_offset > 0);
        }
    }

    // Test navigating up from the top (should wrap to bottom)
    app.color_dialog_option = 0;
    app.color_dialog_scroll_offset = 0;
    
    handle_color_dialog(&mut app, KeyCode::Up);
    
    // Should wrap to the last color
    assert_eq!(app.color_dialog_option, total_colors - 1);
    
    // Scroll should position to show the last color
    assert!(app.color_dialog_scroll_offset > 0);
}

#[test]
fn test_color_dialog_reset_on_escape() {
    let mut app = AppState {
        input: String::new(),
        conversation: Vec::new(),
        is_api_loading: false,
        api_key: "test".to_string(),
        model: "test".to_string(),
        max_tokens: 1024,
        temperature: 0.7,
        scroll_offset: 0,
        exit_selected: 0,
        show_exit_dialog: false,
        show_save_dialog: false,
        save_filename: String::new(),
        show_load_dialog: false,
        show_create_dir_dialog: false,
        new_dir_name: String::new(),
        current_directory: std::path::PathBuf::from("/tmp"),
        available_files: Vec::new(),
        file_list_state: Default::default(),
        show_error_dialog: false,
        error_message: String::new(),
        simulate_api: false,
        syntax_highlighting: true,
        highlight_cache: Default::default(),
        colors: ColorConfig::default(),
        show_color_dialog: true,
        color_dialog_selection: 2,
        color_dialog_option: 8,
        color_dialog_scroll_offset: 5,
    };

    // Test escape key resets everything
    handle_color_dialog(&mut app, KeyCode::Esc);
    
    assert_eq!(app.show_color_dialog, false);
    assert_eq!(app.color_dialog_selection, 0);
    assert_eq!(app.color_dialog_option, 0);
    assert_eq!(app.color_dialog_scroll_offset, 0);
}
