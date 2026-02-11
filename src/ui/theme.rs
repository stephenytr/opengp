use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub normal: Style,
    pub selected: Style,
    pub highlight: Style,
    pub error: Style,
    pub warning: Style,
    pub success: Style,
    pub header: Style,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            normal: Style::default().fg(Color::White),
            selected: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            highlight: Style::default().bg(Color::DarkGray),
            error: Style::default().fg(Color::Red),
            warning: Style::default().fg(Color::Yellow),
            success: Style::default().fg(Color::Green),
            header: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
