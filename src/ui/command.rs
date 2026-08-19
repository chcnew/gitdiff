use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};
use crate::ui::style;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Panel::Terminal;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 终端 ")
        .border_style(style::border(focused));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let output = &app.runner.output;
    let total = output.len();
    let body_h = height.saturating_sub(1);
    let start = if total > body_h { total - body_h } else { 0 };

    let mut lines: Vec<Line> = output
        .iter()
        .skip(start)
        .map(|s| Line::from(Span::styled(s.clone(), style::fg())))
        .collect();

    let prompt = if app.runner.running { " ⟳ " } else { " $ " };
    lines.push(Line::from(vec![
        Span::styled(prompt, style::info()),
        Span::styled(app.input.clone(), style::fg()),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}
