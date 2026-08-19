use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, CenterView, InputMode, LeftTab, Panel};
use crate::ui::{branches, changes, command, diff, editor, filetree, history, style};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    let show_input = matches!(
        app.input_mode,
        InputMode::Commit | InputMode::Branch | InputMode::Command
    );
    let terminal_h = if app.runner.visible {
        Constraint::Percentage(app.runner.height_percent)
    } else {
        Constraint::Length(0)
    };
    let input_h = if show_input {
        Constraint::Length(1)
    } else {
        Constraint::Length(0)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(4),
            terminal_h,
            input_h,
            Constraint::Length(1),
        ])
        .split(area);

    draw_top_bar(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    if app.runner.visible {
        command::draw(f, app, chunks[2]);
    }
    if show_input {
        draw_input_bar(f, app, chunks[3]);
    }
    draw_status_bar(f, app, chunks[4]);

    branches::draw(f, app, area);
}

fn draw_top_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let line = Line::from(vec![
        Span::styled(" GitDiff TUI ", style::title()),
        Span::styled(
            format!(" {} ", app.repo_name),
            style::fg().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("[{}] ", app.current_branch), style::info()),
        Span::styled(" ?帮助 t终端 b分支 P推送 F拉取 x关闭 q退出", style::dim()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_main(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(1),
            Constraint::Percentage(30),
        ])
        .split(area);

    draw_left(f, app, chunks[0]);
    draw_center(f, app, chunks[1]);
    history::draw(f, app, chunks[2]);
}

fn draw_left(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let tabs = Line::from(vec![
        Span::styled(
            " 变更 ",
            if app.left_tab == LeftTab::Changes {
                style::selected()
            } else {
                style::dim()
            },
        ),
        Span::styled(
            " 文件 ",
            if app.left_tab == LeftTab::Files {
                style::selected()
            } else {
                style::dim()
            },
        ),
        Span::styled(" [变更 /]文件", style::dim()),
    ]);
    f.render_widget(Paragraph::new(tabs), v[0]);

    match app.left_tab {
        LeftTab::Changes => changes::draw(f, app, v[1]),
        LeftTab::Files => filetree::draw(f, app, v[1]),
    }
}

fn draw_center(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    match app.center_view {
        CenterView::Diff => diff::draw(f, app, area),
        CenterView::Editor => editor::draw(f, app, area),
        CenterView::HistoryDiff => diff::draw_history(f, app, area),
    }
}

fn draw_input_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (label, text) = match app.input_mode {
        InputMode::Commit => (" 提交信息 ", app.input.as_str()),
        InputMode::Branch => (" 新分支名 ", app.input.as_str()),
        InputMode::Command => (" $ ", app.input.as_str()),
        _ => ("", app.input.as_str()),
    };
    let line = Line::from(vec![
        Span::styled(label, style::title()),
        Span::styled(format!("{text}"), style::fg()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let line = if let Some(err) = &app.error_msg {
        Line::from(Span::styled(err.clone(), style::error()))
    } else if let Some(msg) = &app.status_msg {
        Line::from(Span::styled(msg.clone(), style::info()))
    } else {
        Line::from(Span::styled(hint(app), style::dim()))
    };
    f.render_widget(Paragraph::new(line), area);
}

fn hint(app: &App) -> String {
    match app.focus {
        Panel::Left => match app.left_tab {
            LeftTab::Changes => "space 暂存/取消  Enter 差异  a 全部  c 提交  u 取消暂存".to_string(),
            LeftTab::Files => "Enter 展开/打开  e 编辑  h 收起  ] 切到变更".to_string(),
        },
        Panel::Center => match app.center_view {
            CenterView::Diff => "v 左右/统一  n/N 差异点  j/k 滚动".to_string(),
            CenterView::Editor => "编辑中… Ctrl+S 保存  Esc 退出编辑".to_string(),
            CenterView::HistoryDiff => "j/k 滚动".to_string(),
        },
        Panel::History => "Enter 查看提交文件  Esc 返回".to_string(),
        Panel::Terminal => "i 输入命令  Ctrl+C 中断  t 收起".to_string(),
    }
}
