use ratatui::text::{Span, Line};
use ratatui::style::{Style as TuiStyle, Color as TuiColor};
use ratatui::style::Stylize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::syntax::highlight_code_block;
use crate::api::HighlightCache;
use crate::config::SHOW_DEBUG_MESSAGES;

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

// New cached version - optimized for performance with syntax highlighting
// This version caches the formatted and highlighted content to avoid expensive
// re-computation on every frame, which dramatically improves performance when
// displaying conversations with lots of code blocks.
pub fn format_message_for_tui_cached(role: &str, content: &str, cache: &mut HighlightCache) -> Vec<Line<'static>> {
    // Calculate hash for the entire message content
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    role.hash(&mut hasher);
    let content_hash = hasher.finish();
    
    // Check if we have cached result
    if let Some(cached_lines) = cache.get(content_hash) {
        if SHOW_DEBUG_MESSAGES {
            eprintln!("Cache HIT for message hash: {}", content_hash);
        }
        return cached_lines.clone();
    }
    
    if SHOW_DEBUG_MESSAGES {
        eprintln!("Cache MISS for message hash: {}, formatting...", content_hash);
    }
    
    // Format the message (same logic as original function)
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
    
    // Cache the result
    cache.insert(content_hash, lines.clone());
    
    if SHOW_DEBUG_MESSAGES {
        eprintln!("Cached {} lines for message hash: {}, cache size: {}", lines.len(), content_hash, cache.len());
    }
    
    lines
}