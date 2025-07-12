use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph, Wrap, Clear, List, ListItem},
    layout::{Layout, Constraint, Direction, Rect},
    style::{Color, Style},
};
use crate::app::AppState;
use crate::config::AnsiColor;

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
        .block(Block::default()
            .borders(Borders::ALL)
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

fn draw_color_dialog(f: &mut Frame, app: &AppState, size: Rect) {
    let dialog_area = Rect {
        x: size.width / 6,
        y: size.height / 6,
        width: (size.width * 2) / 3,
        height: (size.height * 2) / 3,
    };
    
    f.render_widget(Clear, dialog_area);
    
    let color_options = [
        ("Background", &app.colors.background),
        ("Border", &app.colors.border),
        ("Text", &app.colors.text),
        ("User Name", &app.colors.user_name),
        ("Assistant Name", &app.colors.assistant_name),
    ];
    
    let available_colors = AnsiColor::all();
    
    // Create layout for the dialog
    let dialog_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Color options
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
    
    // Left side - color type selection
    let mut color_type_items = Vec::new();
    for (i, (name, current_color)) in color_options.iter().enumerate() {
        let style = if i == app.color_dialog_selection {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };
        
        let display_text = format!("{}: {}", name, current_color.name());
        color_type_items.push(ListItem::new(display_text).style(style));
    }
    
    let color_type_list = List::new(color_type_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Color Type")
            .title_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(color_type_list, options_layout[0]);
    
    // Right side - color selection
    let mut color_items = Vec::new();
    for (i, color) in available_colors.iter().enumerate() {
        let style = if i == app.color_dialog_option {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(color.to_ratatui_color())
        };
        
        let display_text = format!("● {}", color.name());
        color_items.push(ListItem::new(display_text).style(style));
    }
    
    let color_list = List::new(color_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Available Colors")
            .title_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(color_list, options_layout[1]);
    
    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate | ←→: Switch panels | Enter: Select color | Esc: Cancel")
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Instructions")
            .title_style(Style::default().fg(Color::Green)))
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(instructions, dialog_layout[2]);
}
