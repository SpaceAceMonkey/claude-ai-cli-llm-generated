use crossterm::event::KeyModifiers;
use crate::app::AppState;
use crate::api::Message;
use crate::config::SHIFT_ENTER_SENDS;
use crate::handlers::{
    api::send_message_to_api,
    file_ops::{get_saves_directory, load_directory_contents},
};
use tokio::sync::mpsc;
use anyhow::Result;

pub async fn handle_enter_key(
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
    } else if app.input == "/colors" || app.input == "/color" {
        app.show_color_dialog = true;
        app.color_dialog_selection = 0;
        app.color_dialog_option = 0;
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

pub fn handle_backspace(app: &mut AppState) {
    if app.cursor_position > 0 {
        let mut chars: Vec<char> = app.input.chars().collect();
        if app.cursor_position <= chars.len() {
            chars.remove(app.cursor_position - 1);
            app.input = chars.into_iter().collect();
            app.cursor_position -= 1;
        }
    }
}

pub fn handle_delete(app: &mut AppState) {
    let mut chars: Vec<char> = app.input.chars().collect();
    if app.cursor_position < chars.len() {
        chars.remove(app.cursor_position);
        app.input = chars.into_iter().collect();
    }
}

pub fn handle_char_input(app: &mut AppState, c: char) {
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
    } else if (app.input == "/colors" || app.input == "/color") && c == ' ' {
        app.show_color_dialog = true;
        app.color_dialog_selection = 0;
        app.color_dialog_option = 0;
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
