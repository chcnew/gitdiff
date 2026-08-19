use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::diff::{DiffRow, DiffTag};
use crate::ui::style;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let d = &app.diff;
    let title = if d.untracked {
        format!(" 差异 {} [未跟踪] ", d.path)
    } else if d.staged {
        format!(" 差异 {} [暂存] ", d.path)
    } else {
        format!(" 差异 {} [工作区] ", d.path)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(style::border(true));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if d.loading {
        f.render_widget(Paragraph::new(" 加载中…").style(style::dim()), inner);
        return;
    }
    if d.sides.binary {
        f.render_widget(
            Paragraph::new(" 二进制文件，无法展示文本差异").style(style::dim()),
            inner,
        );
        return;
    }

    if d.side_by_side {
        render_side_by_side(f, app, inner);
    } else {
        render_unified(f, app, inner);
    }
}

pub fn draw_history(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 历史详情 ")
        .border_style(style::border(true));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some((path, content)) = &app.history.detail {
        let title = Line::from(Span::styled(format!(" {path}"), style::info()));
        f.render_widget(Paragraph::new(title), inner);

        let body = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner)[1];

        let lines: Vec<Line> = content.lines().map(colorize_line).collect();
        let height = body.height as usize;
        let total = lines.len();
        let scroll = app.history.detail_scroll.min(total.saturating_sub(height));
        let visible: Vec<Line> = lines.into_iter().skip(scroll).take(height).collect();
        f.render_widget(Paragraph::new(visible), body);
    } else {
        f.render_widget(
            Paragraph::new(" 在右侧选中提交文件后查看 patch").style(style::dim()),
            inner,
        );
    }
}

fn render_unified(f: &mut Frame, app: &App, inner: Rect) {
    let d = &app.diff;
    let lines: Vec<Line> = d.unified.lines().map(colorize_line).collect();
    let height = inner.height as usize;
    let total = lines.len();
    let scroll = d.scroll.min(total.saturating_sub(height));
    let visible: Vec<Line> = lines.into_iter().skip(scroll).take(height).collect();
    f.render_widget(Paragraph::new(visible), inner);
}

fn render_side_by_side(f: &mut Frame, app: &App, inner: Rect) {
    let d = &app.diff;
    let height = inner.height as usize;
    let total = d.rows.len();
    let scroll = d.scroll.min(total.saturating_sub(height));
    let active_row = d.hunks.get(d.hunk_idx).copied();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let colw = (inner.width.saturating_sub(2)) / 2;
    let body_height = height.saturating_sub(1);

    let mut left_body: Vec<Line> = Vec::new();
    let mut right_body: Vec<Line> = Vec::new();
    for (i, row) in d.rows.iter().enumerate().skip(scroll).take(body_height) {
        let is_active = active_row == Some(i);
        let (l, r) = row_to_spans(row, colw as usize, is_active);
        left_body.push(l);
        right_body.push(r);
    }

    let left_label = d.sides.left_label.clone();
    let right_label = d.sides.right_label.clone();

    let col_pairs = [
        (cols[0], left_label, left_body),
        (cols[1], right_label, right_body),
    ];

    for (col, label, body) in col_pairs {
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(col);
        let header = Line::from(Span::styled(format!(" {label} "), style::title()));
        f.render_widget(Paragraph::new(header), v[0]);
        f.render_widget(Paragraph::new(body), v[1]);
    }
}

fn colorize_line(s: &str) -> Line<'_> {
    let st = if s.starts_with("+++") || s.starts_with("---") {
        style::dim()
    } else if s.starts_with("@@") {
        style::info()
    } else if s.starts_with('+') {
        Style::default().fg(style::GREEN)
    } else if s.starts_with('-') {
        Style::default().fg(style::RED)
    } else {
        style::fg()
    };
    Line::from(Span::styled(s.to_string(), st))
}

fn row_to_spans(row: &DiffRow, colw: usize, is_active: bool) -> (Line<'_>, Line<'_>) {
    let active = if is_active {
        Style::default().bg(style::HEADER_BG)
    } else {
        Style::default()
    };

    match row.tag {
        DiffTag::Equal => {
            let t = row.left.clone().unwrap_or_default();
            let st = style::fg().patch(active);
            (
                Line::from(Span::styled(pad(&t, colw), st)),
                Line::from(Span::styled(pad(&t, colw), st)),
            )
        }
        DiffTag::Mod => {
            let l = row.left.clone().unwrap_or_default();
            let r = row.right.clone().unwrap_or_default();
            (
                Line::from(Span::styled(
                    pad(&format!("-{l}"), colw),
                    Style::default().fg(style::RED).patch(active),
                )),
                Line::from(Span::styled(
                    pad(&format!("+{r}"), colw),
                    Style::default().fg(style::GREEN).patch(active),
                )),
            )
        }
        DiffTag::Del => {
            let l = row.left.clone().unwrap_or_default();
            (
                Line::from(Span::styled(
                    pad(&format!("-{l}"), colw),
                    Style::default().fg(style::RED).patch(active),
                )),
                Line::from(Span::styled(pad("", colw), active)),
            )
        }
        DiffTag::Add => {
            let r = row.right.clone().unwrap_or_default();
            (
                Line::from(Span::styled(pad("", colw), active)),
                Line::from(Span::styled(
                    pad(&format!("+{r}"), colw),
                    Style::default().fg(style::GREEN).patch(active),
                )),
            )
        }
    }
}

fn pad(s: &str, w: usize) -> String {
    let count = s.chars().count();
    if count >= w {
        s.chars().take(w).collect()
    } else {
        let mut out = String::from(s);
        for _ in 0..(w - count) {
            out.push(' ');
        }
        out
    }
}
