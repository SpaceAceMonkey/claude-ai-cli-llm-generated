use ratatui::{
    layout::{Layout, Constraint, Direction, Rect},
};

pub fn create_main_layout(size: Rect) -> Vec<Rect> {
    // Handle very small terminal sizes gracefully
    if size.height < 10 {
        // For very small terminals, allocate minimum space for each section
        let min_conversation = std::cmp::min(size.height.saturating_sub(3), 1);
        let min_input = std::cmp::min(3, size.height.saturating_sub(min_conversation));
        let min_status = size.height.saturating_sub(min_conversation + min_input);
        
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(min_conversation),
                Constraint::Length(min_input),
                Constraint::Length(min_status),
            ])
            .split(size)
            .to_vec()
    } else {
        // Normal layout for reasonable terminal sizes
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
}
