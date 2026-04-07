pub mod types;

#[cfg(test)]
mod tests;

pub use types::{FormAction, FormMode, ModalAction};

use ratatui::style::{Modifier, Style};
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
