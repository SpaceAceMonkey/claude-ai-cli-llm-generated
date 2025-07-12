// src/ui/borders.rs
// Custom border styles for better text selection behavior

use ratatui::{
    widgets::{Block, Borders},
    symbols::border,
};

/// Creates a block with Unicode box drawing characters for better text selection
pub fn create_unicode_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
}

/// Creates a block with thick Unicode borders
pub fn create_thick_unicode_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
}

/// Creates a block with double-line Unicode borders
pub fn create_double_unicode_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
}

/// Creates a block with plain Unicode borders (default box drawing)
pub fn create_plain_unicode_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::PLAIN)
}
