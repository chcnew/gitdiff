pub mod branches;
pub mod changes;
pub mod command;
pub mod diff;
pub mod editor;
pub mod filetree;
pub mod help;
pub mod history;
pub mod style;
pub mod welcome;
pub mod workspace;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.mode {
        Mode::Welcome => welcome::draw(f, app),
        Mode::Workspace => workspace::draw(f, app),
    }

    if app.help_open {
        help::draw(f);
    }
}

/// 带 `Borders::ALL` 边框的块，其内容区为向内收缩 1 格。
pub fn bordered_inner(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

/// 计算列表可见窗口的起始下标，保证选中项位于窗口中部附近。
pub fn window_start(total: usize, selected: usize, height: usize) -> usize {
    if total == 0 || height == 0 {
        return 0;
    }
    if total <= height {
        return 0;
    }
    let mut start = selected.saturating_sub(height / 2);
    if start + height > total {
        start = total - height;
    }
    start
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
