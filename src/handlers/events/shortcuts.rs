use crossterm::event::{KeyCode, KeyModifiers};
use crate::app::AppState;
use crate::handlers::file_ops::{get_saves_directory, load_directory_contents};
use super::navigation::{handle_chat_scroll_up, handle_chat_scroll_down};

pub fn handle_keyboard_shortcuts(
    app: &mut AppState,
    code: KeyCode,
    modifiers: KeyModifiers,
    terminal_size: (u16, u16),
) -> bool {
    match code {
        // NOTE: macOS Terminal Issues with Modifier Keys
        // On macOS, Alt+Arrow and Shift+Arrow combinations are often intercepted by:
        // 1. Terminal applications for word jumping (Alt+Arrow)
        // 2. System for text selection (Shift+Arrow)
        // 3. Some terminals convert these to escape sequences that crossterm doesn't recognize
        // This is why we provide multiple cross-platform alternatives below.
        
        KeyCode::Up if modifiers.contains(KeyModifiers::CONTROL) || 
                       modifiers.contains(KeyModifiers::ALT) || 
                       modifiers.contains(KeyModifiers::SHIFT) => {
            handle_chat_scroll_up(app);
            true
        }
        KeyCode::Down if modifiers.contains(KeyModifiers::CONTROL) || 
                         modifiers.contains(KeyModifiers::ALT) || 
                         modifiers.contains(KeyModifiers::SHIFT) => {
            handle_chat_scroll_down(app, terminal_size);
            true
        }
        // Cross-platform alternatives for chat scrolling (especially reliable on macOS)
        KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
            handle_chat_scroll_up(app);
            true
        }
        KeyCode::Char('j') if modifiers.contains(KeyModifiers::CONTROL) => {
            handle_chat_scroll_down(app, terminal_size);
            true
        }
        // Vi-style half-page scrolling
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..5 {
                handle_chat_scroll_up(app);
            }
            true
        }
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
            for _ in 0..5 {
                handle_chat_scroll_down(app, terminal_size);
            }
            true
        }
        // Additional cross-platform alternatives
        KeyCode::Char('[') if modifiers.contains(KeyModifiers::CONTROL) => {
            handle_chat_scroll_up(app);
            true
        }
        KeyCode::Char(']') if modifiers.contains(KeyModifiers::CONTROL) => {
            handle_chat_scroll_down(app, terminal_size);
            true
        }
        KeyCode::Char('-') if modifiers.contains(KeyModifiers::CONTROL) => {
            handle_chat_scroll_up(app);
            true
        }
        KeyCode::Char('=') if modifiers.contains(KeyModifiers::CONTROL) => {
            handle_chat_scroll_down(app, terminal_size);
            true
        }
        // Function keys for cross-platform compatibility
        KeyCode::F(1) => {
            handle_chat_scroll_up(app);
            true
        }
        KeyCode::F(2) => {
            handle_chat_scroll_down(app, terminal_size);
            true
        }
        // File operation shortcuts
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_save_dialog = true;
            app.save_filename.clear();
            app.dialog_cursor_pos = 0;
            app.current_directory = get_saves_directory();
            load_directory_contents(&mut app.available_files, &app.current_directory, true);
            app.file_list_state.select(Some(0));
            true
        }
        KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_load_dialog = true;
            app.current_directory = get_saves_directory();
            load_directory_contents(&mut app.available_files, &app.current_directory, false);
            app.file_list_state.select(Some(0));
            true
        }
        KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_exit_dialog = true;
            app.exit_selected = 0;
            true
        }
        // Color configuration shortcut
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_color_dialog = true;
            app.color_dialog_selection = 0;
            app.color_dialog_option = 0;
            true
        }
        _ => false
    }
}
