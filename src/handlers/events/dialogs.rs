use crossterm::event::KeyCode;
use crate::app::AppState;
use crate::handlers::file_ops::{load_directory_contents, save_conversation, load_conversation};
use anyhow::Result;

pub fn handle_exit_dialog(app: &mut AppState, code: KeyCode) -> Result<bool> {
    match code {
        KeyCode::Enter => {
            if app.exit_selected == 0 {
                // Yes - exit the program
                return Ok(true);
            } else {
                // No - close dialog and continue
                app.show_exit_dialog = false;
                app.exit_selected = 0;
            }
        }
        KeyCode::Esc => {
            // Cancel - close dialog
            app.show_exit_dialog = false;
            app.exit_selected = 0;
        }
        KeyCode::Up | KeyCode::Left => {
            app.exit_selected = 0; // Select Yes
        }
        KeyCode::Down | KeyCode::Right => {
            app.exit_selected = 1; // Select No
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Yes - exit immediately
            return Ok(true);
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            // No - close dialog and continue
            app.show_exit_dialog = false;
            app.exit_selected = 0;
        }
        _ => {}
    }
    
    Ok(false)
}

pub fn handle_create_dir_dialog(app: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if !app.new_dir_name.is_empty() {
                let mut new_dir_path = app.current_directory.clone();
                new_dir_path.push(&app.new_dir_name);
                match std::fs::create_dir_all(&new_dir_path) {
                    Ok(_) => {
                        app.status = format!("Directory created: {}", new_dir_path.display());
                        app.current_directory = new_dir_path;
                        load_directory_contents(&mut app.available_files, &app.current_directory, app.show_save_dialog);
                        app.file_list_state.select(Some(0));
                    }
                    Err(e) => {
                        app.status = format!("Failed to create directory: {}", e);
                    }
                }
            }
            app.show_create_dir_dialog = false;
            app.new_dir_name.clear();
        }
        KeyCode::Esc => {
            app.show_create_dir_dialog = false;
            app.new_dir_name.clear();
        }
        KeyCode::Backspace => {
            if !app.new_dir_name.is_empty() {
                let mut chars: Vec<char> = app.new_dir_name.chars().collect();
                chars.pop();
                app.new_dir_name = chars.into_iter().collect();
            }
        }
        KeyCode::Char(c) => {
            // Only allow valid directory name characters
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                app.new_dir_name.push(c);
            }
        }
        _ => {}
    }
}

pub fn handle_save_dialog(app: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if !app.save_filename.is_empty() {
                let mut filepath = app.current_directory.clone();
                filepath.push(&app.save_filename);
                match save_conversation(&app.client, &filepath) {
                    Ok(_) => app.status = format!("Conversation saved to {}", filepath.display()),
                    Err(e) => app.status = format!("Save failed: {}", e),
                }
                app.show_save_dialog = false;
                app.save_filename.clear();
                app.dialog_cursor_pos = 0;
            } else if let Some(selected) = app.file_list_state.selected() {
                if selected < app.available_files.len() {
                    let filename = &app.available_files[selected];
                    if filename == "../" {
                        if let Some(parent) = app.current_directory.parent() {
                            app.current_directory = parent.to_path_buf();
                            load_directory_contents(&mut app.available_files, &app.current_directory, true);
                            app.file_list_state.select(Some(0));
                        }
                    } else if filename == "[ Create New Directory ]" {
                        app.show_create_dir_dialog = true;
                        app.new_dir_name.clear();
                    } else if filename.ends_with('/') {
                        let dirname = &filename[..filename.len()-1];
                        app.current_directory.push(dirname);
                        load_directory_contents(&mut app.available_files, &app.current_directory, true);
                        app.file_list_state.select(Some(0));
                    } else if !filename.starts_with('(') {
                        app.save_filename = filename.clone();
                        app.dialog_cursor_pos = app.save_filename.len();
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.show_save_dialog = false;
            app.save_filename.clear();
            app.dialog_cursor_pos = 0;
        }
        KeyCode::Up => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected > 0 {
                    app.file_list_state.select(Some(selected - 1));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(app.available_files.len() - 1));
            }
        }
        KeyCode::Down => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected < app.available_files.len().saturating_sub(1) {
                    app.file_list_state.select(Some(selected + 1));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(0));
            }
        }
        KeyCode::Backspace => {
            if !app.save_filename.is_empty() && app.dialog_cursor_pos > 0 {
                let mut chars: Vec<char> = app.save_filename.chars().collect();
                chars.remove(app.dialog_cursor_pos - 1);
                app.save_filename = chars.into_iter().collect();
                app.dialog_cursor_pos -= 1;
            }
        }
        KeyCode::Char(c) => {
            let mut chars: Vec<char> = app.save_filename.chars().collect();
            chars.insert(app.dialog_cursor_pos, c);
            app.save_filename = chars.into_iter().collect();
            app.dialog_cursor_pos += 1;
        }
        _ => {}
    }
}

pub fn handle_load_dialog(app: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected < app.available_files.len() {
                    let filename = &app.available_files[selected];
                    if filename == "../" {
                        if let Some(parent) = app.current_directory.parent() {
                            app.current_directory = parent.to_path_buf();
                            load_directory_contents(&mut app.available_files, &app.current_directory, false);
                            app.file_list_state.select(Some(0));
                        }
                    } else if filename.ends_with('/') {
                        let dirname = &filename[..filename.len()-1];
                        app.current_directory.push(dirname);
                        load_directory_contents(&mut app.available_files, &app.current_directory, false);
                        app.file_list_state.select(Some(0));
                    } else if !filename.starts_with('(') {
                        let mut filepath = app.current_directory.clone();
                        filepath.push(filename);
                        match load_conversation(&filepath) {
                            Ok(conversation) => {
                                app.client.messages = conversation.messages;
                                app.client.total_input_tokens = conversation.total_input_tokens;
                                app.client.total_output_tokens = conversation.total_output_tokens;
                                app.status = format!("Conversation loaded from {}", filepath.display());
                                app.auto_scroll = true;
                                app.show_load_dialog = false;
                                // Clear the highlight cache since we have new messages
                                app.clear_highlight_cache();
                            }
                            Err(e) => app.status = format!("Load failed: {}", e),
                        }
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.show_load_dialog = false;
        }
        KeyCode::Up => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected > 0 {
                    app.file_list_state.select(Some(selected - 1));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(app.available_files.len() - 1));
            }
        }
        KeyCode::Down => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected < app.available_files.len().saturating_sub(1) {
                    app.file_list_state.select(Some(selected + 1));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(0));
            }
        }
        _ => {}
    }
}
