use ratatui::{
    layout::{Layout, Constraint, Direction, Rect},
};

pub fn create_main_layout(size: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Conversation
            Constraint::Length(6),   // Input (4 lines + 2 for borders)
            Constraint::Length(3),   // Status
        ])
        .split(size)
        .to_vec()
}
