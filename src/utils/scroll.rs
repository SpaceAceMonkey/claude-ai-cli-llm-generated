// src/utils/scroll.rs
use ratatui::text::Line;

pub fn calculate_chat_scroll_offset(
    chat_spans: &[Line],
    chat_height: u16,
    chat_width: u16,
) -> u16 {
    let mut total_visual_lines: u16 = 0;
    
    for line in chat_spans {
        let line_width = line.width() as u16;
        if line_width > chat_width {
            total_visual_lines += (line_width + chat_width - 1) / chat_width;
        } else {
            total_visual_lines += 1;
        }
    }
    
    if total_visual_lines > chat_height {
        total_visual_lines - chat_height
    } else {
        0
    }
}