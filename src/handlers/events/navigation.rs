use crate::app::AppState;
use crate::handlers::history::{navigate_history_up, navigate_history_down};
use crate::utils::text::{move_cursor_up, move_cursor_down};
use crate::tui::format_message_for_tui_cached;

pub fn handle_chat_scroll_up(app: &mut AppState) {
    if app.chat_scroll_offset > 0 {
        app.chat_scroll_offset -= 1;
    }
    // Disable auto-scroll when user manually scrolls up
    app.auto_scroll = false;
}

pub fn handle_chat_scroll_down(app: &mut AppState, terminal_size: (u16, u16)) {
    let chat_height = terminal_size.1.saturating_sub(8);
    
    // Calculate max scroll
    let mut chat_spans = Vec::new();
    for msg in &app.client.messages {
        chat_spans.extend(format_message_for_tui_cached(&msg.role, &msg.content, &mut app.highlight_cache, app.colors.user_name, app.colors.assistant_name));
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

pub fn handle_up_key(app: &mut AppState, terminal_size: (u16, u16)) {
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

pub fn handle_down_key(app: &mut AppState, terminal_size: (u16, u16)) {
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

pub fn handle_page_up(app: &mut AppState, terminal_size: (u16, u16)) {
    // Scroll chat up
    if app.chat_scroll_offset > 0 {
        let page_size = terminal_size.1.saturating_sub(12); // leave 2-3 lines for context
        app.chat_scroll_offset = app.chat_scroll_offset.saturating_sub(page_size);
        app.auto_scroll = false; // Disable auto-scroll when user manually scrolls
    }
}

pub fn handle_page_down(app: &mut AppState, terminal_size: (u16, u16)) {
    // Scroll chat down
    let chat_height = terminal_size.1.saturating_sub(8); // rough estimate
    let page_size = chat_height.saturating_sub(4); // leave 2-3 lines for context
    
    // Calculate max scroll based on content
    let mut chat_spans = Vec::new();
    for msg in &app.client.messages {
        chat_spans.extend(format_message_for_tui_cached(&msg.role, &msg.content, &mut app.highlight_cache, app.colors.user_name, app.colors.assistant_name));
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
