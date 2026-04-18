//! Disabled subtab view component for feature-gated subtabs.
//!
//! Renders a centred message when a subtab is not enabled (feature-gated).

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::ui::theme::Theme;

/// Displays a disabled subtab message for feature-gated subtabs.
#[derive(Clone)]
pub struct DisabledSubtabView {
    /// Name of the disabled subtab
    subtab_name: String,
    /// Theme for styling
    theme: Theme,
}

impl DisabledSubtabView {
    /// Create a new disabled subtab view
    pub fn new(subtab_name: impl Into<String>, theme: Theme) -> Self {
        Self {
            subtab_name: subtab_name.into(),
            theme,
        }
    }
}

impl Widget for DisabledSubtabView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let main_message = format!("{} is not enabled", self.subtab_name);
        let secondary_message = "Enable this feature in Cargo.toml to use this subtab";

        let content = vec![
            Line::from(main_message),
            Line::from(""),
            Line::from(secondary_message),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(format!(" {} ", self.subtab_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.colors.disabled)),
            )
            .style(Style::default().fg(self.theme.colors.disabled))
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_subtab_view_creation() {
        let theme = Theme::dark();
        let view = DisabledSubtabView::new("Pathology", theme);
        assert_eq!(view.subtab_name, "Pathology");
    }

    #[test]
    fn test_disabled_subtab_view_render() {
        let theme = Theme::dark();
        let view = DisabledSubtabView::new("Prescription", theme);

        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        view.render(Rect::new(0, 0, 50, 10), &mut buf);

        // Verify buffer was written to
        assert!(!buf.content.is_empty());
    }

    #[test]
    fn test_disabled_subtab_view_empty_area() {
        let theme = Theme::dark();
        let view = DisabledSubtabView::new("Referral", theme);

        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        view.render(Rect::new(0, 0, 0, 0), &mut buf);

        // Should handle empty area gracefully
        assert!(buf.content.is_empty());
    }
}
