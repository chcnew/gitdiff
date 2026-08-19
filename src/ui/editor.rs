use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::ui::style;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let e = &app.editor;
    let dirty = if e.dirty { " *" } else { "" };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 编辑 {}{} ", e.path, dirty))
        .border_style(style::border(true));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let mut scroll = e.scroll;
    if e.cursor_line < scroll {
        scroll = e.cursor_line;
    }
    if height > 0 && e.cursor_line >= scroll + height {
        scroll = e.cursor_line + 1 - height;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (i, text) in e.lines.iter().enumerate().skip(scroll).take(height) {
        if i == e.cursor_line {
            let (before, at, after) = split_at_cursor(text, e.cursor_col);
            lines.push(Line::from(vec![
                Span::styled(before, style::fg()),
                Span::styled(if at.is_empty() { " ".to_string() } else { at }, Style::default().bg(style::ACCENT).fg(Color::Black)),
                Span::styled(after, style::fg()),
            ]));
        } else {
            lines.push(Line::from(Span::styled(text.clone(), style::fg())));
        }
    }
    f.render_widget(Paragraph::new(lines), inner);
}

fn split_at_cursor(s: &str, col: usize) -> (String, String, String) {
    let col = col.min(s.len());
    let before = s[..col].to_string();
    let after = s[col..].to_string();
    let mut chars = after.chars();
    let at = chars.next().map(|c| c.to_string()).unwrap_or_default();
    let rest = chars.collect();
    (before, at, rest)
}
