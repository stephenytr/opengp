//! Help Overlay Component
//!
//! F1 help overlay displaying keyboard shortcuts and context-sensitive help.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Padding, Widget};

use crate::ui::keybinds::{KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;

/// Format a KeyEvent into a human-readable string
fn format_key_event(key: &KeyEvent) -> String {
    let mut parts = Vec::new();

    // Add modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift".to_string());
    }

    // Add the key itself
    let key_str = match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.is_empty() {
                c.to_string()
            } else {
                c.to_uppercase().to_string()
            }
        }
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Null => "Null".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::ScrollLock => "ScrollLock".to_string(),
        KeyCode::NumLock => "NumLock".to_string(),
        KeyCode::PrintScreen => "PrintScreen".to_string(),
        KeyCode::Pause => "Pause".to_string(),
        KeyCode::Media(_) => "Media".to_string(),
        KeyCode::Modifier(_) => "Modifier".to_string(),
        KeyCode::Menu => "Menu".to_string(),
        KeyCode::KeypadBegin => "KeypadBegin".to_string(),
    };

    parts.push(key_str);

    parts.join("+")
}

#[derive(Debug, Clone)]
pub struct HelpOverlay {
    visible: bool,
    context: KeyContext,
    theme: Theme,
}

impl HelpOverlay {
    /// Create a new help overlay
    pub fn new(theme: Theme) -> Self {
        Self {
            visible: false,
            context: KeyContext::Global,
            theme,
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

    fn get_display_keybinds(&self) -> Vec<(String, String)> {
        let registry = KeybindRegistry::global();
        let keybinds = registry.get_keybinds_for_context(self.context);

        let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut entries: Vec<(String, String)> = keybinds
            .iter()
            .map(|kb| (format_key_event(&kb.key), kb.description.to_string()))
            .filter(|(key, _)| seen_keys.insert(key.clone()))
            .collect();

        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }
}

/// Render the help overlay
impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.visible || area.is_empty() {
            return;
        }

        let width = (area.width * 7 / 10).clamp(70, 100);
        let height = (area.height * 3 / 5).clamp(15, 30);

        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;

        let help_area = Rect::new(x, y, width, height);

        for row in area.y..area.y + area.height {
            for col in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut(Position::new(col, row)) {
                    cell.set_bg(self.theme.colors.background);
                }
            }
        }

        let block = Block::default()
            .title(" Help (Press F1 to close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.primary))
            .title_style(Style::default().fg(self.theme.colors.foreground))
            .padding(Padding::new(1, 1, 1, 1));

        block.render(help_area, buf);

        let content_area = help_area.inner(Margin::new(1, 1));
        let keybinds = self.get_display_keybinds();

        let key_width = 12u16;
        let col_width = content_area.width / 2;
        let sep_x = content_area.x + col_width - 1;
        let mid_point = (keybinds.len() / 2).min(content_area.height as usize);

        for row in content_area.y..content_area.y + content_area.height.saturating_sub(1) {
            if let Some(cell) = buf.cell_mut(Position::new(sep_x, row)) {
                cell.set_char('│');
                cell.set_fg(self.theme.colors.border);
            }
        }

        let mut render_entry = |i: usize, key: &str, desc: &str, start_x: u16, max_x: u16| {
            let row_offset = i % mid_point;
            let y = content_area.y + row_offset as u16;
            if y >= content_area.y + content_area.height {
                return;
            }

            let key_str = if key.len() > (key_width - 1) as usize {
                format!("{:.1$}", key, (key_width - 1) as usize)
            } else {
                key.to_string()
            };

            buf.set_string(
                start_x,
                y,
                key_str,
                Style::default().fg(self.theme.colors.warning),
            );

            let desc_x = start_x + key_width;
            if desc_x < max_x {
                let avail = (max_x - desc_x) as usize;
                let desc_str = if desc.len() > avail && avail > 3 {
                    format!("{}…", &desc[..avail - 1])
                } else {
                    desc.to_string()
                };
                buf.set_string(
                    desc_x,
                    y,
                    desc_str,
                    Style::default().fg(self.theme.colors.foreground),
                );
            }
        };

        for (i, (key, desc)) in keybinds.iter().enumerate() {
            if i >= mid_point * 2 {
                break;
            }

            let is_right = i >= mid_point;
            let start_x = if is_right { sep_x + 2 } else { content_area.x };
            let end_x = if is_right {
                content_area.x + content_area.width
            } else {
                sep_x
            };

            render_entry(i, key, desc, start_x, end_x);
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
        let mut help = HelpOverlay::new(Theme::dark());
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
        let mut help = HelpOverlay::new(Theme::dark());
        assert_eq!(help.context, KeyContext::Global);

        help.set_context(KeyContext::PatientList);
        assert_eq!(help.context, KeyContext::PatientList);
    }

    #[test]
    fn test_get_display_keybinds() {
        let help = HelpOverlay::new(Theme::dark());
        let keybinds = help.get_display_keybinds();
        assert!(!keybinds.is_empty());
    }
}
