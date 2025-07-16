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
                } else if !app.available_files.is_empty() {
                    // Wrap to bottom
                    app.file_list_state.select(Some(app.available_files.len() - 1));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(app.available_files.len() - 1));
            }
        }
        KeyCode::Down => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected < app.available_files.len().saturating_sub(1) {
                    app.file_list_state.select(Some(selected + 1));
                } else if !app.available_files.is_empty() {
                    // Wrap to top
                    app.file_list_state.select(Some(0));
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
                } else if !app.available_files.is_empty() {
                    // Wrap to bottom
                    app.file_list_state.select(Some(app.available_files.len() - 1));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(app.available_files.len() - 1));
            }
        }
        KeyCode::Down => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected < app.available_files.len().saturating_sub(1) {
                    app.file_list_state.select(Some(selected + 1));
                } else if !app.available_files.is_empty() {
                    // Wrap to top
                    app.file_list_state.select(Some(0));
                }
            } else if !app.available_files.is_empty() {
                app.file_list_state.select(Some(0));
            }
        }
        _ => {}
    }
}

pub fn handle_color_dialog(app: &mut AppState, code: KeyCode) {
    use crate::config::AnsiColor;
    
    let color_options_count = 5; // background, border, text, user_name, assistant_name
    let available_colors = AnsiColor::all();
    
    match code {
        KeyCode::Enter => {
            // Apply the selected color to the selected option
            let selected_color = available_colors[app.color_dialog_option];
            match app.color_dialog_selection {
                0 => app.colors.background = selected_color,
                1 => app.colors.border = selected_color,
                2 => app.colors.text = selected_color,
                3 => app.colors.user_name = selected_color,
                4 => app.colors.assistant_name = selected_color,
                _ => {}
            }
        }
        KeyCode::Esc => {
            // Close the dialog
            app.show_color_dialog = false;
            app.color_dialog_selection = 0;
            app.color_dialog_option = 0;
            app.color_dialog_scroll_offset = 0;
            app.color_dialog_selection_scroll_offset = 0;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            // Reset colors to defaults
            app.colors.reset_to_defaults();
        }
        KeyCode::Up => {
            if app.color_dialog_option > 0 {
                app.color_dialog_option -= 1;
            } else {
                app.color_dialog_option = available_colors.len() - 1;
            }
            // Update scroll offset to keep selection visible
            update_color_dialog_scroll(app, &available_colors);
        }
        KeyCode::Down => {
            if app.color_dialog_option < available_colors.len() - 1 {
                app.color_dialog_option += 1;
            } else {
                app.color_dialog_option = 0;
            }
            // Update scroll offset to keep selection visible
            update_color_dialog_scroll(app, &available_colors);
        }
        KeyCode::Left => {
            if app.color_dialog_selection > 0 {
                app.color_dialog_selection -= 1;
            } else {
                app.color_dialog_selection = color_options_count - 1;
            }
            // Update scroll offset for left pane to keep selection visible
            update_color_dialog_selection_scroll(app, color_options_count);
        }
        KeyCode::Right => {
            if app.color_dialog_selection < color_options_count - 1 {
                app.color_dialog_selection += 1;
            } else {
                app.color_dialog_selection = 0;
            }
            // Update scroll offset for left pane to keep selection visible
            update_color_dialog_selection_scroll(app, color_options_count);
        }
        _ => {}
    }
}

fn update_color_dialog_scroll(app: &mut AppState, available_colors: &[crate::config::AnsiColor]) {
    // This will be updated during rendering with the actual available height
    // For now, use a conservative minimum to prevent out-of-bounds access
    let visible_height = 1; // Will be updated by the render function
    
    let current_selection = app.color_dialog_option;
    let scroll_offset = &mut app.color_dialog_scroll_offset;
    
    // If selection is above the visible area, scroll up
    if current_selection < *scroll_offset {
        *scroll_offset = current_selection;
    }
    // If selection is below the visible area, scroll down
    else if current_selection >= *scroll_offset + visible_height {
        *scroll_offset = current_selection.saturating_sub(visible_height - 1);
    }
    
    // Ensure scroll offset doesn't go beyond the available range
    let max_scroll = available_colors.len().saturating_sub(visible_height);
    if *scroll_offset > max_scroll {
        *scroll_offset = max_scroll;
    }
}

fn update_color_dialog_selection_scroll(app: &mut AppState, total_options: usize) {
    // This will be updated during rendering with the actual available height
    // For now, use a conservative minimum to prevent out-of-bounds access
    let visible_height = 1; // Will be updated by the render function
    
    let current_selection = app.color_dialog_selection;
    let scroll_offset = &mut app.color_dialog_selection_scroll_offset;
    
    // If selection is above the visible area, scroll up
    if current_selection < *scroll_offset {
        *scroll_offset = current_selection;
    }
    // If selection is below the visible area, scroll down
    else if current_selection >= *scroll_offset + visible_height {
        *scroll_offset = current_selection.saturating_sub(visible_height - 1);
    }
    
    // Ensure scroll offset doesn't go beyond the available range
    let max_scroll = total_options.saturating_sub(visible_height);
    if *scroll_offset > max_scroll {
        *scroll_offset = max_scroll;
    }
}

pub fn update_color_dialog_scroll_with_height(app: &mut AppState, available_colors: &[crate::config::AnsiColor], visible_height: usize) {
    let visible_height = std::cmp::max(1, visible_height); // Ensure at least 1 item is visible
    
    let current_selection = app.color_dialog_option;
    let scroll_offset = &mut app.color_dialog_scroll_offset;
    
    // If selection is above the visible area, scroll up
    if current_selection < *scroll_offset {
        *scroll_offset = current_selection;
    }
    // If selection is below the visible area, scroll down
    else if current_selection >= *scroll_offset + visible_height {
        *scroll_offset = current_selection.saturating_sub(visible_height - 1);
    }
    
    // Ensure scroll offset doesn't go beyond the available range
    let max_scroll = available_colors.len().saturating_sub(visible_height);
    if *scroll_offset > max_scroll {
        *scroll_offset = max_scroll;
    }
}

pub fn update_color_dialog_selection_scroll_with_height(app: &mut AppState, total_options: usize, visible_height: usize) {
    let visible_height = std::cmp::max(1, visible_height); // Ensure at least 1 item is visible
    
    let current_selection = app.color_dialog_selection;
    let scroll_offset = &mut app.color_dialog_selection_scroll_offset;
    
    // If selection is above the visible area, scroll up
    if current_selection < *scroll_offset {
        *scroll_offset = current_selection;
    }
    // If selection is below the visible area, scroll down
    else if current_selection >= *scroll_offset + visible_height {
        *scroll_offset = current_selection.saturating_sub(visible_height - 1);
    }
    
    // Ensure scroll offset doesn't go beyond the available range
    let max_scroll = total_options.saturating_sub(visible_height);
    if *scroll_offset > max_scroll {
        *scroll_offset = max_scroll;
    }
}

pub fn handle_profile_dialog(app: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            // Apply selected profile
            let mut profiles: Vec<_> = app.available_profiles.values().collect();
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            if app.profile_dialog_selection < profiles.len() {
                if let Some(profile) = profiles.get(app.profile_dialog_selection) {
                    app.colors = profile.config.clone();
                    // Save the applied profile as current config
                    if let Err(e) = crate::config::save_color_config(&app.colors) {
                        app.show_error_dialog = true;
                        app.error_message = format!("Failed to save color configuration: {}", e);
                    }
                }
            }
            // Note: Dialog remains open like the color dialog does
        }
        KeyCode::Esc => {
            // Cancel - close dialog
            app.show_profile_dialog = false;
            app.profile_dialog_selection = 0;
            app.profile_dialog_scroll_offset = 0;
        }
        KeyCode::Up => {
            let mut profiles: Vec<_> = app.available_profiles.values().collect();
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            
            if app.profile_dialog_selection > 0 {
                app.profile_dialog_selection -= 1;
            } else if !profiles.is_empty() {
                // Wrap to bottom
                app.profile_dialog_selection = profiles.len() - 1;
            }
            // Note: scroll update will be handled by the UI when it renders
        }
        KeyCode::Down => {
            let mut profiles: Vec<_> = app.available_profiles.values().collect();
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            
            if app.profile_dialog_selection < profiles.len().saturating_sub(1) {
                app.profile_dialog_selection += 1;
            } else if !profiles.is_empty() {
                // Wrap to top
                app.profile_dialog_selection = 0;
            }
            // Note: scroll update will be handled by the UI when it renders
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Save current config as custom profile
            // TODO: Implement custom profile saving dialog
            app.show_error_dialog = true;
            app.error_message = "Custom profile saving not yet implemented".to_string();
        }
        _ => {}
    }
}

pub fn update_profile_dialog_scroll_with_height(app: &mut AppState, visible_height: usize) {
    update_profile_dialog_scroll(app, visible_height);
}

fn update_profile_dialog_scroll(app: &mut AppState, visible_height: usize) {
    let total_profiles = app.available_profiles.len();
    
    // Safety check: if visible_height is 0, we can't display anything
    if visible_height == 0 {
        app.profile_dialog_scroll_offset = 0;
        return;
    }
    
    // If we have fewer profiles than visible height, no scrolling needed
    if total_profiles <= visible_height {
        app.profile_dialog_scroll_offset = 0;
        return;
    }
    
    let current_selection = app.profile_dialog_selection;
    let scroll_offset = &mut app.profile_dialog_scroll_offset;
    
    // Adjust scroll offset to keep selection visible
    if current_selection < *scroll_offset {
        *scroll_offset = current_selection;
    } else if current_selection >= *scroll_offset + visible_height {
        // Safe calculation: visible_height is guaranteed > 0 at this point
        *scroll_offset = current_selection.saturating_sub(visible_height - 1);
    }
    
    // Ensure scroll offset doesn't go beyond the available range
    let max_scroll = total_profiles.saturating_sub(visible_height);
    if *scroll_offset > max_scroll {
        *scroll_offset = max_scroll;
    }
}
