//! Welcome panel for empty workspace state.
//!
//! Displays when no patients are currently open, showing OpenGP title,
//! empty state message, and keybind hints for common actions.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use crate::ui::theme::Theme;

/// Welcome panel widget for empty workspace state
#[derive(Clone)]
pub struct WelcomePanel {
    theme: Theme,
}

impl WelcomePanel {
    /// Create a new welcome panel
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}

impl Widget for WelcomePanel {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        // Calculate centered box dimensions
        let box_width = 50.min(area.width.saturating_sub(4));
        let box_height = 12.min(area.height.saturating_sub(2));

        let box_x = area.x + (area.width.saturating_sub(box_width)) / 2;
        let box_y = area.y + (area.height.saturating_sub(box_height)) / 2;

        let box_area = Rect {
            x: box_x,
            y: box_y,
            width: box_width,
            height: box_height,
        };

        // Create border block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.secondary));

        // Render block background
        block.render(box_area, buf);

        // Calculate inner area for content
        let inner = box_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 1,
        });

        // Build content lines
        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(vec![Span::styled(
            "OpenGP",
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::from(""));

        // Empty state message
        lines.push(Line::from(vec![Span::styled(
            "No patients currently open",
            Style::default().fg(self.theme.colors.foreground),
        )]));

        lines.push(Line::from(""));

        // Keybind hints
        lines.push(Line::from(vec![Span::styled(
            "F2",
            Style::default()
                .fg(self.theme.colors.secondary)
                .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — Search patients"),
        ]));

        lines.push(Line::from(vec![Span::styled(
            "F3",
            Style::default()
                .fg(self.theme.colors.secondary)
                .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — Appointments calendar"),
        ]));

        // Render content
        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.colors.foreground));

        paragraph.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_panel_creation() {
        let theme = Theme::dark();
        let panel = WelcomePanel::new(theme);
        assert_eq!(panel.theme.colors.primary, ratatui::style::Color::Cyan);
    }

    #[test]
    fn test_welcome_panel_renders() {
        let theme = Theme::dark();
        let panel = WelcomePanel::new(theme);

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let mut buf = ratatui::buffer::Buffer::empty(area);
        panel.render(area, &mut buf);

        // Verify buffer was modified (not empty)
        assert!(!buf.content.is_empty());
    }
}
