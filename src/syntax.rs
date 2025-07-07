use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use ratatui::style::{Style as RatatuiStyle, Color as RatatuiColor};
use ratatui::text::{Span, Line};

pub fn highlight_code_block(code: &str, language: &str) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_token(language).unwrap_or_else(|| ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    
    let mut lines = Vec::new();
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let mut spans = Vec::new();
        
        for (style, text) in ranges {
            let fg_color = style.foreground;
            let ratatui_color = RatatuiColor::Rgb(fg_color.r, fg_color.g, fg_color.b);
            let ratatui_style = RatatuiStyle::default().fg(ratatui_color);
            spans.push(Span::styled(text.to_string(), ratatui_style));
        }
        
        lines.push(Line::from(spans));
    }
    lines
}