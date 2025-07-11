use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::AppState;
use crate::api::Message;
use crate::config::SHIFT_ENTER_SENDS;
use crate::handlers::{
    api::send_message_to_api,
    file_ops::{get_saves_directory, load_directory_contents, save_conversation, load_conversation},
    history::{navigate_history_up, navigate_history_down},
};
use crate::utils::text::{move_cursor_up, move_cursor_down};
use crate::tui::format_message_for_tui;
use tokio::sync::mpsc;
use anyhow::Result;

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
        KeyCode::Up if modifiers.contains(KeyModifiers::CONTROL) || 
                       modifiers.contains(KeyModifiers::ALT) || 
                       modifiers.contains(KeyModifiers::SHIFT) => {
            handle_chat_scroll_up(app);
        }
        KeyCode::Down if modifiers.contains(KeyModifiers::CONTROL) || 
                         modifiers.contains(KeyModifiers::ALT) || 
                         modifiers.contains(KeyModifiers::SHIFT) => {
            handle_chat_scroll_down(app, terminal_size);
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
        _ => {
            // Handle all other KeyCode variants
        }
    }
    
    Ok(false) // Continue running
}

fn handle_exit_dialog(app: &mut AppState, code: KeyCode) -> Result<bool> {
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

fn handle_create_dir_dialog(app: &mut AppState, code: KeyCode) {
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

fn handle_save_dialog(app: &mut AppState, code: KeyCode) {
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

fn handle_load_dialog(app: &mut AppState, code: KeyCode) {
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

async fn handle_enter_key(
    app: &mut AppState,
    modifiers: KeyModifiers,
    tx: &mpsc::Sender<Result<(String, u32, u32, Vec<Message>), String>>,
) -> Result<()> {
    // Check for commands first
    if app.input == "/save" {
        app.show_save_dialog = true;
        app.save_filename.clear();
        app.dialog_cursor_pos = 0;
        app.current_directory = get_saves_directory();
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        app.file_list_state.select(Some(0));
        app.input.clear();
        app.cursor_position = 0;
    } else if app.input == "/load" {
        app.show_load_dialog = true;
        app.current_directory = get_saves_directory();
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        app.file_list_state.select(Some(0));
        app.input.clear();
        app.cursor_position = 0;
    } else if modifiers.contains(KeyModifiers::SHIFT) || modifiers.contains(KeyModifiers::ALT) {
        if SHIFT_ENTER_SENDS && !app.input.is_empty() {
            send_message(app, tx).await?;
        } else {
            // Shift/Alt+Enter adds newline
            let mut chars: Vec<char> = app.input.chars().collect();
            chars.insert(app.cursor_position, '\n');
            app.input = chars.into_iter().collect();
            app.cursor_position += 1;
        }
    } else if modifiers.contains(KeyModifiers::CONTROL) && !app.input.is_empty() {
        send_message(app, tx).await?;
    } else {
        // Regular Enter behavior depends on the feature flag
        if SHIFT_ENTER_SENDS {
            // Regular Enter inserts a newline
            let mut chars: Vec<char> = app.input.chars().collect();
            chars.insert(app.cursor_position, '\n');
            app.input = chars.into_iter().collect();
            app.cursor_position += 1;
        } else {
            // Regular Enter sends the message
            if !app.input.is_empty() {
                send_message(app, tx).await?;
            }
        }
    }
    
    Ok(())
}

async fn send_message(
    app: &mut AppState,
    tx: &mpsc::Sender<Result<(String, u32, u32, Vec<Message>), String>>,
) -> Result<()> {
    app.waiting = true;
    app.status = "Sending to Claude...".to_string();
    app.progress_i = 0;

    let user_input = app.input.clone();
    app.input.clear();
    app.cursor_position = 0;
    app.history_index = None;
    app.input_draft = None;

    // Add to rustyline history
    app.rl.add_history_entry(&user_input).ok();

    // Add user message
    app.client.messages.push(Message {
        role: "user".to_string(),
        content: user_input.clone(),
    });

    // Spawn API call with channel
    let api_key = app.client.api_key.clone();
    let model = app.client.model.clone();
    let max_tokens = app.client.max_tokens;
    let temperature = app.client.temperature;
    let messages = app.client.messages.clone();
    let simulate = app.simulate_mode;
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        match send_message_to_api(
            user_input,
            messages,
            api_key,
            model,
            max_tokens,
            temperature,
            simulate,
        ).await {
            Ok((response, input_tokens, output_tokens, updated_messages)) => {
                tx_clone.send(Ok((response, input_tokens, output_tokens, updated_messages))).await.ok();
            }
            Err(e) => {
                let error_msg = format!("API Error: {}", e);
                tx_clone.send(Err(error_msg)).await.ok();
            }
        }
    });

    Ok(())
}

fn handle_backspace(app: &mut AppState) {
    if app.cursor_position > 0 {
        let mut chars: Vec<char> = app.input.chars().collect();
        if app.cursor_position <= chars.len() {
            chars.remove(app.cursor_position - 1);
            app.input = chars.into_iter().collect();
            app.cursor_position -= 1;
        }
    }
}

fn handle_delete(app: &mut AppState) {
    let mut chars: Vec<char> = app.input.chars().collect();
    if app.cursor_position < chars.len() {
        chars.remove(app.cursor_position);
        app.input = chars.into_iter().collect();
    }
}

fn handle_chat_scroll_up(app: &mut AppState) {
    if app.chat_scroll_offset > 0 {
        app.chat_scroll_offset -= 1;
        app.auto_scroll = false;
    }
}

fn handle_chat_scroll_down(app: &mut AppState, terminal_size: (u16, u16)) {
    let chat_height = terminal_size.1.saturating_sub(8);
    
    // Calculate max scroll
    let mut chat_spans = Vec::new();
    for msg in &app.client.messages {
        chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
    }
    
    if !chat_spans.is_empty() {
        let chat_width = terminal_size.0.saturating_sub(4);
        let mut total_visual_lines: u16 = 0;
        
        for line in &chat_spans {
            let line_width = line.width();
            if line_width == 0 {
                total_visual_lines += 1;
            } else {
                let wrapped_lines = ((line_width as u16 + chat_width - 1) / chat_width).max(1);
                total_visual_lines += wrapped_lines;
            }
        }
        
        let max_scroll = total_visual_lines.saturating_sub(chat_height);
        if app.chat_scroll_offset < max_scroll {
            app.chat_scroll_offset += 1;
        }
        
        // Re-enable auto-scroll if we're at the bottom
        if app.chat_scroll_offset >= max_scroll {
            app.auto_scroll = true;
        }
    }
}

fn handle_up_key(app: &mut AppState, terminal_size: (u16, u16)) {
    let input_width = terminal_size.0.saturating_sub(4) as usize;
    let is_multiline = app.input.contains('\n') || app.input.len() > input_width;
    
    if is_multiline {
        let new_pos = move_cursor_up(&app.input, app.cursor_position, input_width);
        if new_pos != app.cursor_position {
            app.cursor_position = new_pos;
        } else {
            navigate_history_up(&mut app.input, &mut app.cursor_position, &mut app.history_index, &mut app.input_draft, &app.rl);
        }
    } else {
        navigate_history_up(&mut app.input, &mut app.cursor_position, &mut app.history_index, &mut app.input_draft, &app.rl);
    }
}

fn handle_down_key(app: &mut AppState, terminal_size: (u16, u16)) {
    let input_width = terminal_size.0.saturating_sub(4) as usize;
    let is_multiline = app.input.contains('\n') || app.input.len() > input_width;
    
    if is_multiline {
        let new_pos = move_cursor_down(&app.input, app.cursor_position, input_width);
        if new_pos != app.cursor_position {
            app.cursor_position = new_pos;
        } else {
            navigate_history_down(&mut app.input, &mut app.cursor_position, &mut app.history_index, &mut app.input_draft, &app.rl);
        }
    } else {
        navigate_history_down(&mut app.input, &mut app.cursor_position, &mut app.history_index, &mut app.input_draft, &app.rl);
    }
}

fn handle_page_up(app: &mut AppState, terminal_size: (u16, u16)) {
    // Scroll chat up
    if app.chat_scroll_offset > 0 {
        let page_size = terminal_size.1.saturating_sub(12); // leave 2-3 lines for context
        app.chat_scroll_offset = app.chat_scroll_offset.saturating_sub(page_size);
        app.auto_scroll = false; // Disable auto-scroll when user manually scrolls
    }
}

fn handle_page_down(app: &mut AppState, terminal_size: (u16, u16)) {
    // Scroll chat down
    let chat_height = terminal_size.1.saturating_sub(8); // rough estimate
    let page_size = chat_height.saturating_sub(4); // leave 2-3 lines for context
    
    // Calculate max scroll based on content
    let mut chat_spans = Vec::new();
    for msg in &app.client.messages {
        chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
    }
    
    if !chat_spans.is_empty() {
        let chat_width = terminal_size.0.saturating_sub(4);
        let mut total_visual_lines: u16 = 0;
        
        for line in &chat_spans {
            let line_width = line.width();
            if line_width == 0 {
                total_visual_lines += 1;
            } else {
                let wrapped_lines = ((line_width as u16 + chat_width - 1) / chat_width).max(1);
                total_visual_lines += wrapped_lines;
            }
        }
        
        let max_scroll = total_visual_lines.saturating_sub(chat_height);
        app.chat_scroll_offset = (app.chat_scroll_offset + page_size).min(max_scroll);
        
        // Re-enable auto-scroll if we're at the bottom
        if app.chat_scroll_offset >= max_scroll {
            app.auto_scroll = true;
        }
    }
}

fn handle_char_input(app: &mut AppState, c: char) {
    // Check for commands at start of empty input
    if app.cursor_position == 0 && app.input.is_empty() && c == '/' {
        app.input.insert(app.cursor_position, c);
        app.cursor_position += 1;
    } else if app.input == "/save" && c == ' ' {
        app.show_save_dialog = true;
        app.save_filename.clear();
        app.dialog_cursor_pos = 0;
        app.current_directory = get_saves_directory();
        load_directory_contents(&mut app.available_files, &app.current_directory, true);
        app.file_list_state.select(Some(0));
        app.input.clear();
        app.cursor_position = 0;
    } else if app.input == "/load" && c == ' ' {
        app.show_load_dialog = true;
        app.current_directory = get_saves_directory();
        load_directory_contents(&mut app.available_files, &app.current_directory, false);
        app.file_list_state.select(Some(0));
        app.input.clear();
        app.cursor_position = 0;
    } else {
        // Regular character input
        let mut chars: Vec<char> = app.input.chars().collect();
        chars.insert(app.cursor_position, c);
        app.input = chars.into_iter().collect();
        app.cursor_position += 1;
    }
}
