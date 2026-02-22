//! Help Overlay Component
//!
//! F1 help overlay displaying keyboard shortcuts and context-sensitive help.

use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Padding, Widget};

use crate::ui::keybinds::KeyContext;

/// Help overlay state
#[derive(Debug, Clone)]
pub struct HelpOverlay {
    /// Whether the help overlay is visible
    visible: bool,
    /// Current context for context-sensitive help
    context: KeyContext,
}

impl HelpOverlay {
    /// Create a new help overlay
    pub fn new() -> Self {
        Self {
            visible: false,
            context: KeyContext::Global,
        }
    }

    /// Show the help overlay
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the help overlay
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle the help overlay
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if the help overlay is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set the current context for context-sensitive help
    pub fn set_context(&mut self, context: KeyContext) {
        self.context = context;
    }

    fn get_display_keybinds(&self) -> Vec<(&'static str, &'static str)> {
        let mut keybinds = vec![
            ("Global", ""),
            ("F1", "Toggle Help"),
            ("q", "Quit (or Ctrl+Q)"),
            ("Ctrl+N", "New Item"),
            ("Ctrl+F", "Search"),
            ("Ctrl+R", "Refresh"),
            ("Tab", "Next Focus"),
            ("Shift+Tab", "Previous Focus"),
            ("Esc", "Cancel / Back"),
        ];

        // Add context-specific keybinds
        match self.context {
            KeyContext::Global | KeyContext::PatientList => {
                keybinds.push(("Navigation", ""));
                keybinds.push(("j/↓", "Move Down"));
                keybinds.push(("k/↑", "Move Up"));
                keybinds.push(("/", "Search"));
                keybinds.push(("n", "New Patient"));
                keybinds.push(("Enter", "Open Patient"));
            }
            KeyContext::PatientForm => {
                keybinds.push(("Form", ""));
                keybinds.push(("Tab", "Next Field"));
                keybinds.push(("Shift+Tab", "Previous Field"));
                keybinds.push(("Enter", "Submit"));
                keybinds.push(("Ctrl+S", "Save"));
            }
            KeyContext::Calendar => {
                keybinds.push(("Calendar", ""));
                keybinds.push(("h/←", "Previous Day"));
                keybinds.push(("l/→", "Next Day"));
                keybinds.push(("j/↓", "Next Week"));
                keybinds.push(("k/↑", "Previous Week"));
                keybinds.push(("t", "Today"));
                keybinds.push(("Enter", "Select Date"));
            }
            KeyContext::Schedule => {
                keybinds.push(("Schedule", ""));
                keybinds.push(("h/←", "Prev Practitioner"));
                keybinds.push(("l/→", "Next Practitioner"));
                keybinds.push(("j/↓", "Next Time Slot"));
                keybinds.push(("k/↑", "Prev Time Slot"));
                keybinds.push(("n", "New Appointment"));
                keybinds.push(("Enter", "Select"));
            }
            KeyContext::Clinical => {
                keybinds.push(("Clinical", ""));
                keybinds.push(("Enter", "View Note"));
                keybinds.push(("n", "New Note"));
                keybinds.push(("e", "Edit Note"));
                keybinds.push(("Tab/Shift+Tab", "Cycle Views"));
                keybinds.push(("←/→", "Cycle Views"));
                keybinds.push(("1-7", "Jump to View"));
                keybinds.push(("a", "Allergies"));
                keybinds.push(("c", "Conditions"));
                keybinds.push(("v", "Vital Signs"));
                keybinds.push(("o", "Observations"));
                keybinds.push(("f", "Family History"));
                keybinds.push(("h", "Social History"));
            }
            KeyContext::Billing => {
                keybinds.push(("Billing", ""));
                keybinds.push(("Enter", "View Invoice"));
                keybinds.push(("n", "New Invoice"));
                keybinds.push(("p", "Process Payment"));
            }
            KeyContext::Search => {
                keybinds.push(("Search", ""));
                keybinds.push(("Enter", "Select"));
                keybinds.push(("Esc", "Close"));
            }
            KeyContext::Help => {
                keybinds.push(("Help", ""));
                keybinds.push(("Esc", "Close"));
                keybinds.push(("↑/↓", "Scroll"));
            }
        }

        keybinds
    }
}

impl Default for HelpOverlay {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the help overlay
impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.visible || area.is_empty() {
            return;
        }

        // Calculate the help box size (centered, roughly 60% of screen)
        let width = (area.width * 3 / 5).clamp(40, 80);
        let height = (area.height * 3 / 5).clamp(15, 30);

        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;

        let help_area = Rect::new(x, y, width, height);

        // Draw semi-transparent background overlay
        for row in area.y..area.y + area.height {
            for col in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut(Position::new(col, row)) {
                    cell.set_bg(Color::Black);
                }
            }
        }

        // Draw the help box
        let block = Block::default()
            .title(" Help (Press F1 to close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title_style(Style::default().fg(Color::White))
            .padding(Padding::new(1, 1, 1, 1));

        block.render(help_area, buf);

        // Calculate content area
        let content_area = help_area.inner(Margin::new(1, 1));

        // Get keybinds to display
        let keybinds = self.get_display_keybinds();

        // Render keybinds in two columns
        let col_width = content_area.width / 2;
        let mid_point = keybinds.len().div_ceil(2);

        for (i, (key, desc)) in keybinds.iter().enumerate() {
            let col = if i < mid_point { 0 } else { col_width as usize };
            let row = if i < mid_point { i } else { i - mid_point };

            let x = content_area.x + col as u16;
            let y = content_area.y + row as u16;

            if y < content_area.y + content_area.height {
                // Render key (left column)
                buf.set_string(x, y, *key, Style::default().fg(Color::Yellow));

                // Render description (with spacing after key)
                let desc_x = x + 12;
                if desc_x < content_area.x + content_area.width {
                    buf.set_string(desc_x, y, *desc, Style::default().fg(Color::White));
                }
            }
        }

        // Draw separator line in the middle
        let sep_x = content_area.x + col_width;
        for row in content_area.y..content_area.y + content_area.height.saturating_sub(1) {
            if let Some(cell) = buf.cell_mut(Position::new(sep_x, row)) {
                cell.set_char('│');
                cell.set_fg(Color::DarkGray);
            }
        }
    }
}

/// Height of the help overlay when visible
pub const HELP_OVERLAY_MIN_HEIGHT: u16 = 15;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_overlay_visibility() {
        let mut help = HelpOverlay::new();
        assert!(!help.is_visible());

        help.show();
        assert!(help.is_visible());

        help.hide();
        assert!(!help.is_visible());

        help.toggle();
        assert!(help.is_visible());

        help.toggle();
        assert!(!help.is_visible());
    }

    #[test]
    fn test_help_overlay_context() {
        let mut help = HelpOverlay::new();
        assert_eq!(help.context, KeyContext::Global);

        help.set_context(KeyContext::PatientList);
        assert_eq!(help.context, KeyContext::PatientList);
    }

    #[test]
    fn test_get_display_keybinds() {
        let help = HelpOverlay::new();
        let keybinds = help.get_display_keybinds();
        assert!(!keybinds.is_empty());
    }
}
