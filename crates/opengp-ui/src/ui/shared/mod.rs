pub mod types;

#[cfg(test)]
mod tests;

pub use types::{FormAction, FormMode, ModalAction};

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};

use crate::ui::theme::Theme;

pub fn selected_style(theme: &Theme) -> Style {
    Style::default().fg(theme.colors.selected)
}

pub fn header_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.colors.primary)
        .add_modifier(Modifier::BOLD)
}

pub fn disabled_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.colors.disabled)
        .add_modifier(Modifier::DIM)
}

pub fn error_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.colors.error)
        .add_modifier(Modifier::BOLD)
}

pub fn hover_style(theme: &Theme) -> Style {
    Style::default()
        .fg(invert_color(theme.colors.highlight))
        .bg(theme.colors.highlight)
}

pub fn selected_hover_style(theme: &Theme) -> Style {
    Style::default()
        .fg(invert_color(theme.colors.highlight))
        .bg(theme.colors.highlight)
}

pub fn invert_color(color: Color) -> Color {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        Color::Black | Color::DarkGray => return Color::White,
        Color::White | Color::Gray => return Color::Black,
        Color::Red | Color::LightRed => (220.0, 50.0, 50.0),
        Color::Green | Color::LightGreen => (50.0, 205.0, 50.0),
        Color::Blue | Color::LightBlue => (70.0, 130.0, 180.0),
        Color::Yellow | Color::LightYellow => (220.0, 220.0, 0.0),
        Color::Cyan | Color::LightCyan => (0.0, 210.0, 210.0),
        Color::Magenta | Color::LightMagenta => (210.0, 0.0, 210.0),
        _ => return Color::White,
    };
    let luminance = 0.2126 * linearise(r) + 0.7152 * linearise(g) + 0.0722 * linearise(b);
    if luminance > 0.179 {
        Color::Black
    } else {
        Color::White
    }
}

fn linearise(channel: f32) -> f32 {
    let c = channel / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn border_block<'a>(title: &'a str, theme: &Theme, focused: bool) -> Block<'a> {
    let border_style = if focused {
        Style::default().fg(theme.colors.primary)
    } else {
        Style::default().fg(theme.colors.border)
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
}
