// src/utils/text.rs
use ratatui::text::Line;

pub fn wrap_text(text: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    if text.is_empty() {
        lines.push(Line::from(""));
        return lines;
    }
    
    let text_lines: Vec<&str> = text.split('\n').collect();
    
    for (i, line) in text_lines.iter().enumerate() {
        if line.is_empty() {
            lines.push(Line::from(""));
        } else if line.len() <= width {
            lines.push(Line::from(line.to_string()));
        } else {
            let mut start = 0;
            while start < line.len() {
                let end = (start + width).min(line.len());
                lines.push(Line::from(line[start..end].to_string()));
                start = end;
            }
        }
    }
    
    lines
}

pub fn calculate_cursor_line(text: &str, cursor_pos: usize, width: usize) -> usize {
    let mut line = 0;
    let mut col = 0;
    
    for (i, ch) in text.chars().enumerate() {
        if i >= cursor_pos {
            break;
        }
        
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            if col >= width {
                line += 1;
                col = 0;
            }
            col += 1;
        }
    }
    line
}

pub fn calculate_cursor_position(
    text: &str,
    cursor_pos: usize,
    width: usize,
    scroll_offset: usize,
) -> (usize, usize) {
    let mut line: usize = 0;
    let mut col: usize = 0;
    
    for (i, ch) in text.chars().enumerate() {
        if i >= cursor_pos {
            break;
        }
        
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            if col >= width {
                line += 1;
                col = 0;
            }
            col += 1;
        }
    }
    
    let visible_line = line.saturating_sub(scroll_offset);
    (col % width, visible_line)
}

pub fn move_cursor_up(text: &str, cursor_pos: usize, width: usize) -> usize {
    if cursor_pos == 0 {
        return 0;
    }
    
    let mut current_line = 0;
    let mut current_col = 0;
    let mut line_start_positions = vec![0];
    
    for (i, ch) in text.chars().enumerate() {
        if i == cursor_pos {
            break;
        }
        
        if ch == '\n' {
            current_line += 1;
            current_col = 0;
            line_start_positions.push(i + 1);
        } else {
            if current_col >= width {
                current_line += 1;
                current_col = 0;
                line_start_positions.push(i);
            }
            current_col += 1;
        }
    }
    
    if current_line == 0 {
        return cursor_pos;
    }
    
    let prev_line_start = line_start_positions[current_line - 1];
    let mut target_pos = prev_line_start;
    let mut col = 0;
    
    for (i, ch) in text[prev_line_start..].chars().enumerate() {
        if col >= current_col || ch == '\n' || col >= width {
            break;
        }
        target_pos = prev_line_start + i + 1;
        col += 1;
    }
    
    target_pos.min(cursor_pos - 1)
}

pub fn move_cursor_down(text: &str, cursor_pos: usize, width: usize) -> usize {
    if cursor_pos >= text.len() {
        return cursor_pos;
    }
    
    let mut current_line = 0;
    let mut current_col = 0;
    let mut line_start_positions = vec![0];
    let mut found_cursor = false;
    let mut cursor_line = 0;
    let mut cursor_col = 0;
    
    for (i, ch) in text.chars().enumerate() {
        if i == cursor_pos {
            found_cursor = true;
            cursor_line = current_line;
            cursor_col = current_col;
        }
        
        if ch == '\n' {
            current_line += 1;
            current_col = 0;
            line_start_positions.push(i + 1);
        } else {
            if current_col >= width {
                current_line += 1;
                current_col = 0;
                line_start_positions.push(i);
            }
            current_col += 1;
        }
    }
    
    if !found_cursor || cursor_line >= current_line {
        return cursor_pos;
    }
    
    if cursor_line + 1 >= line_start_positions.len() {
        return cursor_pos;
    }
    
    let next_line_start = line_start_positions[cursor_line + 1];
    let mut target_pos = next_line_start;
    let mut col = 0;
    
    for (i, ch) in text[next_line_start..].chars().enumerate() {
        if col >= cursor_col || ch == '\n' || (i > 0 && col >= width) {
            break;
        }
        target_pos = next_line_start + i + 1;
        col += 1;
    }
    
    target_pos.max(cursor_pos + 1)
}