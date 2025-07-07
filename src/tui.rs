use ratatui::text::{Span, Line};
use ratatui::style::{Style as TuiStyle, Color as TuiColor};
use ratatui::style::Stylize;

use crate::syntax::highlight_code_block;

pub fn format_message_for_tui(role: &str, content: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let (role_color, _) = match role {
        "assistant" => (TuiColor::Cyan, "\x1b[0m"),
        "user" => (TuiColor::Magenta, "\x1b[0m"),
        _ => (TuiColor::Yellow, "\x1b[0m"),
    };

    let mut in_code = false;
    let mut code_lang = "rust";
    let mut code_buf = String::new();

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            if in_code {
                let highlighted_lines = highlight_code_block(&code_buf, code_lang);
                lines.extend(highlighted_lines);
                code_buf.clear();
                in_code = false;
            } else {
                let after = line.trim_start().trim_start_matches("```").trim();
                code_lang = if !after.is_empty() { after } else { "rust" };
                in_code = true;
            }
            continue;
        }
        if in_code {
            code_buf.push_str(line);
            code_buf.push('\n');
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}]: ", role),
                    TuiStyle::default().fg(role_color).bold(),
                ),
                Span::raw(line.to_string()),
            ]));
        }
    }
    if in_code && !code_buf.is_empty() {
        let highlighted_lines = highlight_code_block(&code_buf, code_lang);
        lines.extend(highlighted_lines);
    }
    
    lines
}