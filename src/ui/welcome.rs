use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, InputMode};
use crate::ui::{style, window_start};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(6),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(" GitDiff TUI ", style::title())),
        Line::from(""),
        Line::from("本地轻量 Git 终端客户端"),
    ])
    .alignment(Alignment::Left);
    f.render_widget(title, chunks[0]);

    let list_area = chunks[1];
    let height = list_area.height as usize;
    let mut lines: Vec<Line> = Vec::new();
    if app.recent_list.is_empty() {
        lines.push(Line::from(Span::styled("（无最近项目，按 o 打开仓库）", style::dim())));
    } else {
        let start = window_start(app.recent_list.len(), app.recent_selected, height);
        for (i, r) in app.recent_list.iter().enumerate().skip(start).take(height) {
            let is_sel = i == app.recent_selected;
            let row = if is_sel { style::selected() } else { style::fg() };
            lines.push(Line::from(vec![
                Span::styled(if is_sel { "› " } else { "  " }, style::dim()),
                Span::styled(r.name.clone(), row),
                Span::styled(format!("  {}", r.path), style::dim()),
            ]));
        }
    }
    let list = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 最近项目 ")
            .border_style(style::border(true)),
    );
    f.render_widget(list, list_area);

    let hints = Paragraph::new(Line::from(vec![
        Span::styled("o", style::info()),
        Span::styled(" 打开仓库   ", style::dim()),
        Span::styled("Enter", style::info()),
        Span::styled(" 打开选中   ", style::dim()),
        Span::styled("d", style::info()),
        Span::styled(" 移除   ", style::dim()),
        Span::styled("?", style::info()),
        Span::styled(" 帮助   ", style::dim()),
        Span::styled("q", style::info()),
        Span::styled(" 退出", style::dim()),
    ]));
    f.render_widget(hints, chunks[2]);

    if app.input_mode == InputMode::Path {
        f.render_widget(
            Paragraph::new(format!(" 仓库路径: {}", app.input)).style(style::info()),
            chunks[3],
        );
    } else if let Some(err) = &app.error_msg {
        f.render_widget(Paragraph::new(err.clone()).style(style::error()), chunks[3]);
    } else {
        f.render_widget(
            Paragraph::new(Span::styled(app.git_version.clone(), style::dim())),
            chunks[3],
        );
    }
}
