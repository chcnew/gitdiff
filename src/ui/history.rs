use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};
use crate::ui::{style, window_start};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Panel::History;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 历史 ")
        .border_style(style::border(focused));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.history.in_files {
        draw_files(f, app, inner);
    } else {
        draw_commits(f, app, inner);
    }
}

fn draw_commits(f: &mut Frame, app: &App, area: Rect) {
    let commits = &app.history.commits;
    let height = area.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    if commits.is_empty() {
        lines.push(Line::from(Span::styled("（无提交）", style::dim())));
    } else {
        let vis = (height / 2).max(1);
        let start = window_start(commits.len(), app.history.selected, vis);
        for (i, c) in commits.iter().enumerate().skip(start).take(vis) {
            let is_sel = i == app.history.selected && app.focus == Panel::History;
            let row = if is_sel { style::selected() } else { style::fg() };
            lines.push(Line::from(vec![
                Span::styled(if is_sel { "› " } else { "  " }, style::dim()),
                Span::styled(format!("{} ", c.date), style::dim()),
                Span::styled(c.subject.clone(), row),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    ", style::dim()),
                Span::styled(
                    format!("{}  {}", c.hash.get(..8).unwrap_or(&c.hash), c.author),
                    style::dim(),
                ),
            ]));
        }
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_files(f: &mut Frame, app: &App, area: Rect) {
    let files = &app.history.files;
    let height = area.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    if files.is_empty() {
        lines.push(Line::from(Span::styled("（无文件变更）", style::dim())));
    } else {
        let start = window_start(files.len(), app.history.files_selected, height);
        for (i, cf) in files.iter().enumerate().skip(start).take(height) {
            let is_sel = i == app.history.files_selected && app.focus == Panel::History;
            let st = cf.status.chars().next().unwrap_or(' ');
            let row = if is_sel { style::selected() } else { style::fg() };
            lines.push(Line::from(vec![
                Span::styled(if is_sel { "› " } else { "  " }, style::dim()),
                Span::styled(format!("{st} "), style::status(st)),
                Span::styled(cf.path.clone(), row),
            ]));
        }
    }
    f.render_widget(Paragraph::new(lines), area);
}
