use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct StatusBar {
    left: String,
    center: String,
    right: String,
    error_message: Option<String>,
    visible: bool,
    theme: Theme,
}

impl StatusBar {
    pub fn new(theme: Theme) -> Self {
        Self {
            left: String::new(),
            center: String::new(),
            right: String::new(),
            error_message: None,
            visible: true,
            theme,
        }
    }

    /// Set the left section
    pub fn set_left(&mut self, text: impl Into<String>) {
        self.left = text.into();
    }

    /// Set the center section
    pub fn set_center(&mut self, text: impl Into<String>) {
        self.center = text.into();
    }

    /// Set the right section
    pub fn set_right(&mut self, text: impl Into<String>) {
        self.right = text.into();
    }

    /// Set the status bar content
    pub fn set_content(
        &mut self,
        left: impl Into<String>,
        center: impl Into<String>,
        right: impl Into<String>,
    ) {
        self.left = left.into();
        self.center = center.into();
        self.right = right.into();
    }

    /// Set an error message (displayed in red, overrides center section)
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    /// Clear the error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Clear all content
    pub fn clear(&mut self) {
        self.left.clear();
        self.center.clear();
        self.right.clear();
        self.error_message = None;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Create a status bar for the Patient list context
    pub fn patient_list(theme: Theme) -> Self {
        let mut bar = Self::new(theme);
        bar.set_left("Patients");
        bar.set_center("Search: / | New: n | Refresh: Ctrl+R");
        bar.set_right("F1 Help");
        bar
    }

    /// Create a status bar for the Patient form context
    pub fn patient_form(theme: Theme) -> Self {
        let mut bar = Self::new(theme);
        bar.set_left("New Patient");
        bar.set_center("Tab: Next Field | Ctrl+S: Submit | Esc: Cancel");
        bar.set_right("Esc: Cancel");
        bar
    }

    /// Create a status bar for the Calendar context
    pub fn calendar(theme: Theme) -> Self {
        let mut bar = Self::new(theme);
        bar.set_left("Calendar");
        bar.set_center("h/l: Day | j/k: Week | t: Today | Enter: Select");
        bar.set_right("F1 Help");
        bar
    }

    /// Create a status bar for the Schedule context
    pub fn schedule(theme: Theme) -> Self {
        let mut bar = Self::new(theme);
        bar.set_left("Schedule");
        bar.set_center("h/l: Column | j/k: Time | n: New | Enter: Select");
        bar.set_right("F1 Help");
        bar
    }

    /// Create a status bar for the Clinical context
    pub fn clinical(theme: Theme) -> Self {
        let mut bar = Self::new(theme);
        bar.set_left("Clinical Notes");
        bar.set_center("n: New | /: Search | 1-7: Views | Up/Down/j/k: Navigate");
        bar.set_right("F1 Help");
        bar
    }

    /// Create a status bar for the Billing context
    pub fn billing(theme: Theme) -> Self {
        let mut bar = Self::new(theme);
        bar.set_left("Billing");
        bar.set_center("Enter: View | n: New Invoice");
        bar.set_right("F1 Help");
        bar
    }
}

/// Render the status bar
impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || !self.visible {
            return;
        }

        // Use a block with top border
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.render(area, buf);

        // Calculate sections
        let width = area.width as usize;
        let center_width = width / 3;
        let side_width = (width - center_width) / 2;

        // Render left section
        if !self.left.is_empty() {
            let left_text = if self.left.len() > side_width {
                &self.left[..side_width]
            } else {
                &self.left
            };
            buf.set_string(
                area.x,
                area.y,
                left_text,
                Style::default().fg(self.theme.colors.foreground),
            );
        }

        let center_start = area.x + (side_width as u16);
        if let Some(ref error) = self.error_message {
            let error_text = if error.len() > center_width {
                &error[..center_width]
            } else {
                error.as_str()
            };
            buf.set_string(
                center_start,
                area.y,
                error_text,
                Style::default().fg(self.theme.colors.error),
            );
        } else if !self.center.is_empty() {
            let center_text = if self.center.len() > center_width {
                &self.center[..center_width]
            } else {
                &self.center
            };
            buf.set_string(
                center_start,
                area.y,
                center_text,
                Style::default().fg(self.theme.colors.disabled),
            );
        }

        // Render right section
        if !self.right.is_empty() {
            let right_start = area.x + (area.width.saturating_sub(self.right.len() as u16));
            buf.set_string(
                right_start,
                area.y,
                &self.right,
                Style::default().fg(self.theme.colors.border),
            );
        }
    }
}

/// Height of the status bar
pub const STATUS_BAR_HEIGHT: u16 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_creation() {
        let bar = StatusBar::new(Theme::dark());
        assert!(bar.is_visible());
        assert!(bar.left.is_empty());
    }

    #[test]
    fn test_status_bar_patient_list() {
        let bar = StatusBar::patient_list(Theme::dark());
        assert!(bar.is_visible());
        assert_eq!(bar.left, "Patients");
        assert!(bar.right.contains("Help"));
    }

    #[test]
    fn test_status_bar_visibility() {
        let mut bar = StatusBar::new(Theme::dark());
        assert!(bar.is_visible());

        bar.set_visible(false);
        assert!(!bar.is_visible());

        bar.set_visible(true);
        assert!(bar.is_visible());
    }

    #[test]
    fn test_status_bar_snapshot_normal_state() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(80, 3)).unwrap();
        let bar = StatusBar::patient_list(Theme::dark());

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(bar, rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_status_bar_snapshot_with_error() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(80, 3)).unwrap();
        let mut bar = StatusBar::new(Theme::dark());
        bar.set_left("Patients");
        bar.set_error("Error: Patient not found");
        bar.set_right("F1 Help");

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(bar, rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_status_bar_snapshot_with_custom_content() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(80, 3)).unwrap();
        let mut bar = StatusBar::new(Theme::dark());
        bar.set_left("Custom Section");
        bar.set_center("Custom Center");
        bar.set_right("Custom Right");

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(bar, rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }
}
