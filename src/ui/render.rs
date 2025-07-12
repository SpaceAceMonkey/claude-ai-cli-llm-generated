// src/ui/render.rs
use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction, Rect},
    text::Text,
    style::Style,
};
use crate::{
    app::AppState,
    config::*,
    tui::format_message_for_tui_cached,
    utils::{text::*, scroll::*},
    ui::dialogs::draw_dialogs,
};

pub fn draw_ui(
    f: &mut Frame,
    app: &mut AppState,
    layout: &[Rect],
) {
    // Draw chat area
    draw_chat(f, app, layout[0]);
    
    // Draw input area
    draw_input(f, app, layout[1]);
    
    // Draw status area
    draw_status(f, app, layout[2]);
    
    // Draw dialogs (overlays)
    draw_dialogs(f, app, f.size());
}

fn draw_chat(
    f: &mut Frame,
    app: &mut AppState,
    area: Rect,
) {
    let mut chat_spans = Vec::new();
    for msg in &app.client.messages {
        chat_spans.extend(format_message_for_tui_cached(
            &msg.role, 
            &msg.content, 
            &mut app.highlight_cache,
            app.colors.user_name,
            app.colors.assistant_name,
        ));
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
        .block(Block::default()
            .borders(Borders::ALL)
            .border_set(app.colors.border_style.to_ratatui_border_set())
            .title(chat_title)
            .border_style(Style::default().fg(app.colors.border.to_ratatui_color()))
            .title_style(Style::default().fg(app.colors.border.to_ratatui_color())))
        .wrap(Wrap { trim: false })
        .scroll((app.chat_scroll_offset, 0))
        .style(Style::default()
            .bg(app.colors.background.to_ratatui_color())
            .fg(app.colors.text.to_ratatui_color()));
    f.render_widget(chat, area);
}

fn draw_input(
    f: &mut Frame,
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
        "Input (Shift/Alt+Enter to send, Enter for newline)"
    } else {
        "Input (Enter to send, Shift/Alt+Enter for newline)"
    };
    
    let input_bar = Paragraph::new(Text::from(input_lines))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_set(app.colors.border_style.to_ratatui_border_set())
            .title(input_title)
            .border_style(Style::default().fg(app.colors.border.to_ratatui_color()))
            .title_style(Style::default().fg(app.colors.border.to_ratatui_color())))
        .wrap(Wrap { trim: false })
        .scroll((app.input_scroll_offset, 0))
        .style(Style::default()
            .bg(app.colors.background.to_ratatui_color())
            .fg(app.colors.text.to_ratatui_color()));
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

fn draw_status(
    f: &mut Frame,
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
        .block(Block::default()
            .borders(Borders::ALL)
            .border_set(app.colors.border_style.to_ratatui_border_set())
            .title("Status")
            .border_style(Style::default().fg(app.colors.border.to_ratatui_color()))
            .title_style(Style::default().fg(app.colors.border.to_ratatui_color())))
        .style(Style::default()
            .bg(app.colors.background.to_ratatui_color())
            .fg(app.colors.text.to_ratatui_color()));
    f.render_widget(status_bar, bottom_chunks[0]);

    // Token usage
    let token_usage_text = format!(
        "Input tokens: {}, Output tokens: {}, Total tokens: {}",
        app.client.total_input_tokens,
        app.client.total_output_tokens,
        app.client.total_tokens()
    );
    let token_usage = Paragraph::new(token_usage_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_set(app.colors.border_style.to_ratatui_border_set())
            .title("Token Usage")
            .border_style(Style::default().fg(app.colors.border.to_ratatui_color()))
            .title_style(Style::default().fg(app.colors.border.to_ratatui_color())))
        .style(Style::default()
            .bg(app.colors.background.to_ratatui_color())
            .fg(app.colors.text.to_ratatui_color()));
    f.render_widget(token_usage, bottom_chunks[1]);
}