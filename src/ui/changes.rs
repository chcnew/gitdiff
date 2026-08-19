use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};
use crate::git::types::{Area, FileChange};
use crate::ui::{style, window_start};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Panel::Left;
    let items = &app.changes.items;
    let height = area.height as usize;

    let mut lines: Vec<Line> = Vec::new();
    if items.is_empty() {
        lines.push(Line::from(Span::styled(" 工作区干净", style::dim())));
    } else {
        let start = window_start(items.len(), app.changes.selected, height);
        for (i, fc) in items.iter().enumerate().skip(start).take(height) {
            let is_sel = i == app.changes.selected && focused;
            let st = status_char(fc);
            let area_label = match fc.area {
                Area::Staged => "S",
                Area::Unstaged => "U",
                Area::Untracked => "?",
            };
            let row = if is_sel { style::selected() } else { style::fg() };
            lines.push(Line::from(vec![
                Span::styled(if is_sel { "› " } else { "  " }, style::dim()),
                Span::styled(format!("{st} "), style::status(st)),
                Span::styled(format!("[{area_label}] "), style::dim()),
                Span::styled(fc.path.clone(), row),
                Span::styled(
                    fc.old_path
                        .as_ref()
                        .map(|o| format!("  ← {o}"))
                        .unwrap_or_default(),
                    style::dim(),
                ),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 变更 ({}) ", items.len()))
        .border_style(style::border(focused));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn status_char(fc: &FileChange) -> char {
    match fc.area {
        Area::Staged => fc.index_status,
        _ => fc.worktree_status,
    }
}
