// src/ui/render.rs
use ratatui::{
    Frame,
    backend::Backend,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction, Rect},
    text::Text,
};
use crate::{
    app::AppState,
    config::*,
    tui::format_message_for_tui,
    utils::{text::*, scroll::*},
};

pub fn draw_ui<B: Backend>(
    f: &mut Frame<B>,
    app: &mut AppState,
    layout: &[Rect],
) {
    // Draw chat area
    draw_chat(f, app, layout[0]);
    
    // Draw input area
    draw_input(f, app, layout[1]);
    
    // Draw status area
    draw_status(f, app, layout[2]);
}

fn draw_chat<B: Backend>(
    f: &mut Frame<B>,
    app: &mut AppState,
    area: Rect,
) {
    let mut chat_spans = Vec::new();
    for msg in &app.client.messages {
        chat_spans.extend(format_message_for_tui(&msg.role, &msg.content));
    }

    // Calculate proper scroll offset if auto_scroll is enabled
    if app.auto_scroll && !chat_spans.is_empty() {
        let chat_height = area.height.saturating_sub(2);
        let chat_width = area.width.saturating_sub(2);
        app.chat_scroll_offset = calculate_chat_scroll_offset(&chat_spans, chat_height, chat_width);
    }

    let chat_title = if app.simulate_mode {
        "Conversation (SIMULATE MODE)"
    } else {
        "Conversation"
    };
    
    let chat = Paragraph::new(Text::from(chat_spans))
        .block(Block::default().borders(Borders::ALL).title(chat_title))
        .wrap(Wrap { trim: false })
        .scroll((app.chat_scroll_offset, 0));
    f.render_widget(chat, area);
}

fn draw_input<B: Backend>(
    f: &mut Frame<B>,
    app: &mut AppState,
    area: Rect,
) {
    let input_lines = wrap_text(&app.input, area.width.saturating_sub(2) as usize);
    let cursor_line = calculate_cursor_line(
        &app.input,
        app.cursor_position,
        area.width.saturating_sub(2) as usize,
    );
    let input_height = area.height.saturating_sub(2);

    // Auto-scroll input to keep cursor visible
    if cursor_line >= (app.input_scroll_offset as usize + input_height as usize) {
        app.input_scroll_offset = (cursor_line + 1).saturating_sub(input_height as usize) as u16;
    } else if cursor_line < app.input_scroll_offset as usize {
        app.input_scroll_offset = cursor_line as u16;
    }

    let input_title = if SHIFT_ENTER_SENDS {
        "Input (Shift/Alt+Enter to send)"
    } else {
        "Input (Ctrl+Enter to send, Shift/Alt+Enter for newline)"
    };
    
    let input_bar = Paragraph::new(Text::from(input_lines))
        .block(Block::default().borders(Borders::ALL).title(input_title))
        .wrap(Wrap { trim: false })
        .scroll((app.input_scroll_offset, 0));
    f.render_widget(input_bar, area);

    // Calculate cursor position for rendering
    let (cursor_x, cursor_y) = calculate_cursor_position(
        &app.input,
        app.cursor_position,
        area.width.saturating_sub(2) as usize,
        app.input_scroll_offset as usize,
    );
    f.set_cursor(
        area.x + cursor_x as u16 + 1,
        area.y + cursor_y as u16 + 1,
    );
}

fn draw_status<B: Backend>(
    f: &mut Frame<B>,
    app: &AppState,
    area: Rect,
) {
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Status
            Constraint::Percentage(30), // Token usage
        ])
        .split(area);

    // Status bar
    let status_text = if app.waiting {
        format!(
            "Waiting for Claude {}",
            PROGRESS_FRAMES[app.progress_i % PROGRESS_FRAMES.len()]
        )
    } else {
        app.status.clone()
    };
    
    let status_bar = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status_bar, bottom_chunks[0]);

    // Token usage
    let token_usage_text = format!(
        "Input tokens: {}, Output tokens: {}, Total tokens: {}",
        app.client.total_input_tokens,
        app.client.total_output_tokens,
        app.client.total_tokens()
    );
    let token_usage = Paragraph::new(token_usage_text)
        .block(Block::default().borders(Borders::ALL).title("Token Usage"));
    f.render_widget(token_usage, bottom_chunks[1]);
}