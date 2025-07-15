use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph, Wrap, Clear, List, ListItem},
    layout::{Layout, Constraint, Direction, Rect},
    style::{Color, Style},
};
use crate::app::AppState;
use crate::config::AnsiColor;

/// Helper function to create a block with the configured border style
fn create_dialog_block(app: &AppState) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(app.colors.border_style.to_ratatui_border_set())
}

pub fn draw_dialogs(f: &mut Frame, app: &mut AppState, size: Rect) {
    // Save dialog overlay
    if app.show_save_dialog {
        draw_save_dialog(f, app, size);
    }
    
    // Load dialog overlay
    if app.show_load_dialog {
        draw_load_dialog(f, app, size);
    }
    
    // Create directory dialog overlay
    if app.show_create_dir_dialog {
        draw_create_dir_dialog(f, app, size);
    }

    // Color configuration dialog overlay
    if app.show_color_dialog {
        draw_color_dialog(f, app, size);
    }

    // Exit confirmation dialog overlay (render last so it appears on top)
    if app.show_exit_dialog {
        draw_exit_dialog(f, app, size);
    }

    // Error dialog overlay (render last so it appears on top)
    if app.show_error_dialog {
        draw_error_dialog(f, app, size);
    }
}

fn draw_save_dialog(f: &mut Frame, app: &mut AppState, size: Rect) {
    let dialog_area = Rect {
        x: size.width / 6,
        y: size.height / 4,
        width: (size.width * 2) / 3,
        height: size.height / 2,
    };
    
    // Create outer border area (slightly larger than dialog)
    let outer_border_area = Rect {
        x: dialog_area.x.saturating_sub(1),
        y: dialog_area.y.saturating_sub(1),
        width: dialog_area.width + 2,
        height: dialog_area.height + 2,
    };
    
    // Render outer border for visual separation
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(outer_border, outer_border_area);
    f.render_widget(Clear, dialog_area);
    
    // Split the dialog area to reserve space for filename input
    let dialog_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),     // File list (minimum 5 lines)
            Constraint::Length(3),  // Filename input area (1 line + 2 borders)
        ])
        .split(dialog_area);
    
    // Render file list in the top section
    let file_items: Vec<ListItem> = app.available_files.iter().map(|f| ListItem::new(f.as_str())).collect();
    
    let file_list = List::new(file_items)
        .block(create_dialog_block(app)
            .title(format!("Save Conversation - {} (↑↓ to select, Enter to save/navigate, Tab to copy filename)", app.current_directory.display())))
        .highlight_style(Style::default().bg(Color::Blue))
        .style(Style::default().bg(Color::Black));
    
    f.render_stateful_widget(file_list, dialog_layout[0], &mut app.file_list_state);
    
    // Render filename input in the bottom section
    let filename_input = Paragraph::new(format!("Filename: {}", app.save_filename))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Enter filename (Esc to cancel)"))
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(filename_input, dialog_layout[1]);
    
    // Set cursor in the filename input area
    f.set_cursor(
        dialog_layout[1].x + "Filename: ".len() as u16 + app.save_filename.len() as u16 + 1,
        dialog_layout[1].y + 1,
    );
}

fn draw_load_dialog(f: &mut Frame, app: &mut AppState, size: Rect) {
    let dialog_area = Rect {
        x: size.width / 6,
        y: size.height / 4,
        width: (size.width * 2) / 3,
        height: size.height / 2,
    };
    
    // Create outer border area (slightly larger than dialog)
    let outer_border_area = Rect {
        x: dialog_area.x.saturating_sub(1),
        y: dialog_area.y.saturating_sub(1),
        width: dialog_area.width + 2,
        height: dialog_area.height + 2,
    };
    
    // Render outer border for visual separation
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(outer_border, outer_border_area);
    f.render_widget(Clear, dialog_area);
    
    let file_items: Vec<ListItem> = app.available_files.iter().map(|f| ListItem::new(f.as_str())).collect();
    
    let file_list = List::new(file_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Load Conversation - {} (↑↓ to select, Enter to open, Esc to cancel)", app.current_directory.display())))
        .highlight_style(Style::default().bg(Color::Blue))
        .style(Style::default().bg(Color::Black));
    
    f.render_stateful_widget(file_list, dialog_area, &mut app.file_list_state);
}

fn draw_create_dir_dialog(f: &mut Frame, app: &AppState, size: Rect) {
    let dialog_area = Rect {
        x: size.width / 4,
        y: size.height / 3,
        width: size.width / 2,
        height: 5,
    };
    
    // Create outer border area (slightly larger than dialog)
    let outer_border_area = Rect {
        x: dialog_area.x.saturating_sub(1),
        y: dialog_area.y.saturating_sub(1),
        width: dialog_area.width + 2,
        height: dialog_area.height + 2,
    };
    
    // Render outer border for visual separation
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(outer_border, outer_border_area);
    f.render_widget(Clear, dialog_area);
    
    let create_dialog = Paragraph::new(format!("Enter directory name: {}", app.new_dir_name))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Create Directory in {}", app.current_directory.display())))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(create_dialog, dialog_area);
    
    // Fix cursor positioning - place it right after "Enter directory name: "
    let prompt_len = "Enter directory name: ".len();
    f.set_cursor(
        dialog_area.x + 1 + prompt_len as u16 + app.new_dir_name.len() as u16,
        dialog_area.y + 1,
    );
}

fn draw_exit_dialog(f: &mut Frame, app: &AppState, size: Rect) {
    // Calculate optimal dialog width based on content
    let main_text = "Exit the program?";
    let instruction_text = "Use ↑↓ or Y/N to select, Enter to confirm.";
    let title_text = "Confirm Exit";
    let options_text = "  [Yes]     [No]  ";
    
    // Find the longest line to determine minimum width needed
    let text_lines = [main_text, instruction_text, title_text, options_text];
    let max_content_width = text_lines.iter()
        .map(|line| line.len())
        .max()
        .unwrap_or(0);
    
    // Add margins: 2 for borders + 4 for internal padding
    let min_width = max_content_width + 6;
    
    // Limit to 90% of screen width but ensure it's at least the minimum needed
    let max_allowed_width = (size.width * 90) / 100;
    let dialog_width = std::cmp::min(max_allowed_width, std::cmp::max(min_width as u16, 30));
    
    // Center the dialog horizontally
    let dialog_x = (size.width.saturating_sub(dialog_width)) / 2;
    
    let dialog_area = Rect {
        x: dialog_x,
        y: size.height / 2 - 3,
        width: dialog_width,
        height: 6,
    };
    
    // Create outer border area (slightly larger than dialog)
    let outer_border_area = Rect {
        x: dialog_area.x.saturating_sub(1),
        y: dialog_area.y.saturating_sub(1),
        width: dialog_area.width + 2,
        height: dialog_area.height + 2,
    };
    
    // Render outer border for visual separation
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(outer_border, outer_border_area);
    f.render_widget(Clear, dialog_area);
    
    let exit_dialog = Paragraph::new("Exit the program?\n\nUse ↑↓ or Y/N to select, Enter to confirm.")
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Confirm Exit"))
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });
    
    f.render_widget(exit_dialog, dialog_area);
    
    // Render Yes/No options
    let options_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 4,
        width: dialog_area.width - 4,
        height: 1,
    };
    
    let options = Paragraph::new("  [Yes]     [No]  ")
        .style(Style::default());
    f.render_widget(options, options_area);
    
    // Highlight the selected option
    let highlight_area = if app.exit_selected == 0 {
        Rect {
            x: options_area.x + 2,
            y: options_area.y,
            width: 5,
            height: 1,
        }
    } else {
        Rect {
            x: options_area.x + 12,
            y: options_area.y,
            width: 4,
            height: 1,
        }
    };
    
    let highlight_text = if app.exit_selected == 0 { "[Yes]" } else { "[No]" };
    let highlight = Paragraph::new(highlight_text)
        .style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(highlight, highlight_area);
}

fn draw_error_dialog(f: &mut Frame, app: &AppState, size: Rect) {
    let error_area = Rect {
        x: size.width / 4,
        y: size.height / 4,
        width: size.width / 2,
        height: size.height / 4,
    };
    
    // Create outer border area
    let outer_border_area = Rect {
        x: error_area.x.saturating_sub(1),
        y: error_area.y.saturating_sub(1),
        width: error_area.width + 2,
        height: error_area.height + 2,
    };
    
    // Render outer border
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    f.render_widget(outer_border, outer_border_area);
    
    f.render_widget(Clear, error_area);
    
    let error_dialog = Paragraph::new(app.error_message.clone())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Error")
            .title_style(Style::default().fg(Color::Red)))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(error_dialog, error_area);
}

fn draw_color_dialog(f: &mut Frame, app: &mut AppState, size: Rect) {
    let dialog_area = Rect {
        x: size.width / 6,
        y: size.height / 6,
        width: (size.width * 2) / 3,
        height: (size.height * 2) / 3,
    };
    
    // Create outer border area (slightly larger than dialog)
    let outer_border_area = Rect {
        x: dialog_area.x.saturating_sub(1),
        y: dialog_area.y.saturating_sub(1),
        width: dialog_area.width + 2,
        height: dialog_area.height + 2,
    };
    
    // Render outer border for visual separation
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(outer_border, outer_border_area);
    f.render_widget(Clear, dialog_area);
    
    // Clone the colors to avoid borrowing issues
    let background_color = app.colors.background;
    let border_color = app.colors.border;
    let text_color = app.colors.text;
    let user_name_color = app.colors.user_name;
    let assistant_name_color = app.colors.assistant_name;
    
    let color_options = [
        ("Background", background_color),
        ("Border", border_color),
        ("Text", text_color),
        ("User Name", user_name_color),
        ("Assistant Name", assistant_name_color),
    ];
    
    let available_colors = AnsiColor::all();
    
    // Create layout for the dialog
    let dialog_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Color options (at least 1 line)
            Constraint::Length(3),  // Instructions
        ])
        .split(dialog_area);
    
    // Title
    let title = Paragraph::new("Color Configuration")
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Colors")
            .title_style(Style::default().fg(Color::Yellow)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(title, dialog_layout[0]);
    
    // Color options area
    let options_area = dialog_layout[1];
    let options_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Color type list
            Constraint::Percentage(60),  // Color selection
        ])
        .split(options_area);
    
    // Left side - color type selection with scrolling
    let left_available_height = options_layout[0].height.saturating_sub(2); // subtract borders
    let left_visible_count = std::cmp::max(1, left_available_height as usize); // Ensure at least 1 item is visible
    
    // Update scroll offset for left pane with actual available height
    crate::handlers::events::update_color_dialog_selection_scroll_with_height(app, color_options.len(), left_visible_count);
    
    let left_scroll_offset = app.color_dialog_selection_scroll_offset;
    let left_max_scroll = color_options.len().saturating_sub(left_visible_count);
    let left_clamped_scroll_offset = std::cmp::min(left_scroll_offset, left_max_scroll);
    let left_end_index = std::cmp::min(left_clamped_scroll_offset + left_visible_count, color_options.len());
    
    let mut color_type_items = Vec::new();
    for (i, (name, current_color)) in color_options.iter().enumerate().skip(left_clamped_scroll_offset).take(left_end_index - left_clamped_scroll_offset) {
        let style = if i == app.color_dialog_selection {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };
        
        let display_text = format!("{}: {}", name, current_color.name());
        color_type_items.push(ListItem::new(display_text).style(style));
    }
    
    // Add scroll indicators for left pane if needed
    let mut left_title = "Color Type".to_string();
    if left_clamped_scroll_offset > 0 {
        left_title = format!("{} ↑", left_title);
    }
    if left_end_index < color_options.len() {
        left_title = format!("{} ↓", left_title);
    }
    
    let color_type_list = List::new(color_type_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(left_title)
            .title_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(color_type_list, options_layout[0]);
    
    // Right side - color selection with scrolling
    let right_available_height = options_layout[1].height.saturating_sub(2); // subtract borders
    let right_visible_count = std::cmp::max(1, right_available_height as usize); // Ensure at least 1 item is visible
    
    // Update scroll offset for right pane with actual available height
    crate::handlers::events::update_color_dialog_scroll_with_height(app, &available_colors, right_visible_count);
    
    let right_scroll_offset = app.color_dialog_scroll_offset;
    let right_max_scroll = available_colors.len().saturating_sub(right_visible_count);
    let right_clamped_scroll_offset = std::cmp::min(right_scroll_offset, right_max_scroll);
    let right_end_index = std::cmp::min(right_clamped_scroll_offset + right_visible_count, available_colors.len());
    
    let mut color_items = Vec::new();
    for (i, color) in available_colors.iter().enumerate().skip(right_clamped_scroll_offset).take(right_end_index - right_clamped_scroll_offset) {
        let style = if i == app.color_dialog_option {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(color.to_ratatui_color())
        };
        
        let display_text = format!("● {}", color.name());
        color_items.push(ListItem::new(display_text).style(style));
    }
    
    // Add scroll indicators for right pane if needed
    let mut right_title = "Available Colors".to_string();
    if right_clamped_scroll_offset > 0 {
        right_title = format!("{} ↑", right_title);
    }
    if right_end_index < available_colors.len() {
        right_title = format!("{} ↓", right_title);
    }
    
    let color_list = List::new(color_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(right_title)
            .title_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(color_list, options_layout[1]);
    
    // Instructions
    let instructions = Paragraph::new("←→: Select color type | ↑↓: Select color | Enter: Apply | Esc: Cancel")
<<<<<<< Updated upstream
=======
        .block(create_enhanced_dialog_block("Instructions"))
        .style(Style::default().bg(Color::Black).fg(Color::White));
    
    f.render_widget(instructions, dialog_layout[2]);
}

fn draw_profile_dialog(f: &mut Frame, app: &AppState, size: Rect) {
    // Get the number of profiles to calculate content-based size
    let profiles = crate::config::get_all_profiles();
    let profile_count = profiles.len();
    
    // Calculate dynamic dialog size based on content
    let min_width = 50;  // Minimum width for profile names and descriptions
    let min_height = 10; // Title (3) + Min content (4) + Instructions (3)
    
    // Calculate preferred size based on profile count
    let preferred_width = std::cmp::max(min_width, 70);
    let preferred_height = std::cmp::max(min_height, 6 + profile_count); // Title + profiles + instructions + padding
    
    // Apply 90% maximum constraint
    let max_width = (size.width * 9) / 10;
    let max_height = (size.height * 9) / 10;
    
    // Use the smaller of preferred or maximum size
    let dialog_width = std::cmp::min(preferred_width, max_width);
    let dialog_height = std::cmp::min(preferred_height as u16, max_height);
    
    // Center the dialog
    let dialog_area = Rect {
        x: (size.width.saturating_sub(dialog_width)) / 2,
        y: (size.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };
    
    // Create outer border area for visual separation
    let outer_border_area = Rect {
        x: dialog_area.x.saturating_sub(1),
        y: dialog_area.y.saturating_sub(1),
        width: dialog_area.width + 2,
        height: dialog_area.height + 2,
    };
    
    // Render outer border
    let outer_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    f.render_widget(outer_border, outer_border_area);
    
    f.render_widget(Clear, dialog_area);
    
    // Create layout for the dialog
    let dialog_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Profile list
            Constraint::Length(3),  // Instructions
        ])
        .split(dialog_area);
    
    // Title
    let title = Paragraph::new("Color Profiles")
        .block(create_enhanced_dialog_block("Color Profiles"))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(title, dialog_layout[0]);
    
    // Profile list
    let profile_area = dialog_layout[1];
    let profiles: Vec<_> = app.available_profiles.values().collect();
    
    // Calculate visible area for scrolling
    let visible_height = profile_area.height.saturating_sub(2) as usize; // Account for borders
    let scroll_offset = app.profile_dialog_scroll_offset;
    
    let mut profile_items = Vec::new();
    for (i, profile) in profiles.iter().enumerate() {
        if i >= scroll_offset && i < scroll_offset + visible_height {
            let style = if i == app.profile_dialog_selection {
                Style::default().fg(Color::Yellow).bg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            
            let display_text = format!("{} - {}", profile.name, profile.description);
            profile_items.push(ListItem::new(display_text).style(style));
        }
    }
    
    let profile_list = List::new(profile_items)
>>>>>>> Stashed changes
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Instructions")
            .title_style(Style::default().fg(Color::Green)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(instructions, dialog_layout[2]);
}
