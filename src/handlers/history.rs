// src/handlers/history.rs
use rustyline::Editor;
use rustyline::history::History;

pub fn navigate_history_up(
    input: &mut String,
    cursor_position: &mut usize,
    history_index: &mut Option<usize>,
    input_draft: &mut Option<String>,
    rl: &Editor<(), rustyline::history::DefaultHistory>,
) {
    let history = rl.history();
    if history.len() == 0 {
        return;
    }
    
    // Save current input as draft if we're just starting to browse history
    if history_index.is_none() && !input.is_empty() {
        *input_draft = Some(input.clone());
    }
    
    *history_index = Some(match *history_index {
        None => history.len().saturating_sub(1),
        Some(0) => 0,
        Some(i) => i.saturating_sub(1),
    });
    
    if let Some(i) = *history_index {
        let entries: Vec<String> = history.iter().map(|s| s.to_string()).collect();
        if i < entries.len() {
            *input = entries[i].clone();
            *cursor_position = input.len();
        }
    }
}

pub fn navigate_history_down(
    input: &mut String,
    cursor_position: &mut usize,
    history_index: &mut Option<usize>,
    input_draft: &mut Option<String>,
    rl: &Editor<(), rustyline::history::DefaultHistory>,
) {
    let history = rl.history();
    if let Some(i) = *history_index {
        if i + 1 < history.len() {
            *history_index = Some(i + 1);
            let entries: Vec<String> = history.iter().map(|s| s.to_string()).collect();
            if i + 1 < entries.len() {
                *input = entries[i + 1].clone();
                *cursor_position = input.len();
            }
        } else {
            // We've reached the end of history, restore the draft if we have one
            *history_index = None;
            if let Some(draft) = input_draft.take() {
                *input = draft;
                *cursor_position = input.len();
            } else {
                input.clear();
                *cursor_position = 0;
            }
        }
    }
}