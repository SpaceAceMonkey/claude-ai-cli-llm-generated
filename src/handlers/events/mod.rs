use crossterm::event::{KeyCode, KeyEvent};
use crate::app::AppState;
use crate::api::Message;
use tokio::sync::mpsc;
use anyhow::Result;

mod dialogs;
mod input;
mod navigation;
mod shortcuts;

use dialogs::{handle_exit_dialog, handle_create_dir_dialog, handle_save_dialog, handle_load_dialog};
use input::{handle_enter_key, handle_backspace, handle_delete, handle_char_input};
use navigation::{handle_up_key, handle_down_key, handle_page_up, handle_page_down};
use shortcuts::handle_keyboard_shortcuts;

pub async fn handle_key_event(
    app: &mut AppState,
    key_event: KeyEvent,
    tx: &mpsc::Sender<Result<(String, u32, u32, Vec<Message>), String>>,
    terminal_size: (u16, u16),
) -> Result<bool> {
    let KeyEvent { code, modifiers, .. } = key_event;
    
    match code {
        // Handle error dialog dismissal first
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') if app.show_error_dialog => {
            app.show_error_dialog = false;
            app.error_message.clear();
        }
        // Only process other keys if error dialog is not shown
        _ if app.show_error_dialog => {
            // Ignore all other input when error dialog is shown
        }
        // Handle exit dialog 
        _ if app.show_exit_dialog => {
            return handle_exit_dialog(app, code);
        }
        // Handle create directory dialog
        _ if app.show_create_dir_dialog => {
            handle_create_dir_dialog(app, code);
        }
        // Handle save dialog
        _ if app.show_save_dialog => {
            handle_save_dialog(app, code);
        }
        // Handle load dialog
        _ if app.show_load_dialog => {
            handle_load_dialog(app, code);
        }
        // Handle main interface - Escape shows exit dialog ONLY when no other dialogs are open
        KeyCode::Esc => {
            // Show exit confirmation dialog only when in main interface
            app.show_exit_dialog = true;
            app.exit_selected = 0; // Default to Yes
        }
        // Main interface key handling
        KeyCode::Enter => {
            handle_enter_key(app, modifiers, tx).await?;
        }
        KeyCode::Backspace => {
            handle_backspace(app);
        }
        KeyCode::Delete => {
            handle_delete(app);
        }
        KeyCode::Left => {
            if app.cursor_position > 0 { 
                app.cursor_position -= 1; 
            }
        }
        KeyCode::Right => {
            if app.cursor_position < app.input.chars().count() { 
                app.cursor_position += 1; 
            }
        }
        KeyCode::Up => {
            handle_up_key(app, terminal_size);
        }
        KeyCode::Down => {
            handle_down_key(app, terminal_size);
        }
        KeyCode::Home => {
            app.cursor_position = 0;
        }
        KeyCode::End => {
            app.cursor_position = app.input.chars().count();
        }
        KeyCode::PageUp => {
            handle_page_up(app, terminal_size);
        }
        KeyCode::PageDown => {
            handle_page_down(app, terminal_size);
        }
        KeyCode::Char(c) => {
            handle_char_input(app, c);
        }
        // Handle keyboard shortcuts and modified keys
        _ => {
            if handle_keyboard_shortcuts(app, code, modifiers, terminal_size) {
                // Shortcut was handled
            }
            // Handle all other KeyCode variants
        }
    }
    
    Ok(false) // Continue running
}
