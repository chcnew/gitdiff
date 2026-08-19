use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};
use crate::ui::{style, window_start};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Panel::Left;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 文件 ")
        .border_style(style::border(focused));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let flat = app.filetree.flat();
    let height = inner.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    if flat.is_empty() {
        lines.push(Line::from(Span::styled("（空）", style::dim())));
    } else {
        let start = window_start(flat.len(), app.filetree.selected, height);
        for (i, row) in flat.iter().enumerate().skip(start).take(height) {
            let is_sel = i == app.filetree.selected && focused;
            let indent = "  ".repeat(row.depth);
            let arrow = if row.is_dir {
                if app.filetree.open.contains(&row.path) {
                    "▾"
                } else {
                    "▸"
                }
            } else {
                " "
            };
            let st = if is_sel {
                style::selected()
            } else if row.is_dir {
                Style::default().fg(style::ACCENT)
            } else {
                style::fg()
            };
            lines.push(Line::from(vec![
                Span::styled(if is_sel { "› " } else { "  " }, style::dim()),
                Span::styled(indent, style::dim()),
                Span::styled(arrow, style::dim()),
                Span::styled(format!(" {}", row.name), st),
            ]));
        }
    }
    f.render_widget(Paragraph::new(lines), inner);
}
