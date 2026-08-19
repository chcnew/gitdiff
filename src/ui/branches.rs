use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::ui::{centered_rect, style};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if !app.branches.visible {
        return;
    }

    let popup = centered_rect(50, 70, area);
    let items = &app.branches.items;
    let mut lines: Vec<Line> = Vec::new();
    if items.is_empty() {
        lines.push(Line::from(Span::styled("（无分支）", style::dim())));
    } else {
        for (i, b) in items.iter().enumerate() {
            let is_sel = i == app.branches.selected;
            let row = if is_sel { style::selected() } else { style::fg() };
            lines.push(Line::from(vec![
                Span::styled(if b.is_current { "* " } else { "  " }, style::info()),
                Span::styled(format!(" {}{}", b.name, if is_sel { " ◄" } else { "" }), row),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 分支 ")
        .border_style(style::border(true));
    let para = Paragraph::new(lines).block(block);

    f.render_widget(Clear, popup);
    f.render_widget(para, popup);
}
