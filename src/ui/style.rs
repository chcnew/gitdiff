use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::Rgb(20, 184, 166);
pub const RED: Color = Color::Rgb(239, 68, 68);
pub const GREEN: Color = Color::Rgb(34, 197, 94);
pub const AMBER: Color = Color::Rgb(245, 158, 11);
pub const FG: Color = Color::Rgb(226, 232, 240);
pub const DIM: Color = Color::Rgb(100, 116, 139);
pub const HEADER_BG: Color = Color::Rgb(30, 41, 59);
pub const INK: Color = Color::Rgb(15, 23, 42);

pub fn title() -> Style {
    Style::default()
        .fg(INK)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn border(focused: bool) -> Style {
    if focused {
        Style::default().fg(ACCENT)
    } else {
        Style::default().fg(DIM)
    }
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn fg() -> Style {
    Style::default().fg(FG)
}

pub fn selected() -> Style {
    Style::default()
        .bg(ACCENT)
        .fg(INK)
        .add_modifier(Modifier::BOLD)
}

pub fn info() -> Style {
    Style::default().fg(ACCENT)
}

pub fn error() -> Style {
    Style::default().fg(RED)
}

pub fn status(c: char) -> Style {
    match c {
        'A' => Style::default().fg(GREEN),
        'D' => Style::default().fg(RED),
        'M' | 'R' | 'C' => Style::default().fg(AMBER),
        'U' => Style::default().fg(RED),
        '?' => Style::default().fg(DIM),
        _ => Style::default().fg(FG),
    }
}
