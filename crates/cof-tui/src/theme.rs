//! Shared colors and styles, so every screen looks like part of the same app.

use ratatui::style::{Color, Modifier, Style};

pub const BLOOD: Color = Color::Rgb(170, 20, 20);
pub const BONE: Color = Color::Rgb(220, 220, 200);
pub const ASH: Color = Color::Rgb(120, 120, 110);

pub fn text() -> Style {
    Style::new().fg(BONE)
}

pub fn dim() -> Style {
    Style::new().fg(ASH)
}

pub fn title() -> Style {
    Style::new().fg(BLOOD).add_modifier(Modifier::BOLD)
}

pub fn border() -> Style {
    Style::new().fg(BLOOD)
}

pub fn selected() -> Style {
    Style::new()
        .fg(BONE)
        .bg(Color::Rgb(70, 10, 10))
        .add_modifier(Modifier::BOLD)
}

pub fn key() -> Style {
    Style::new().fg(BONE).add_modifier(Modifier::BOLD)
}

pub fn info() -> Style {
    Style::new().fg(Color::Rgb(150, 190, 150))
}

pub fn error() -> Style {
    Style::new()
        .fg(Color::Rgb(255, 90, 90))
        .add_modifier(Modifier::BOLD)
}
