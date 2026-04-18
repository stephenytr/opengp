//! Allergy Detail Modal Component
//!
//! Read-only modal displaying allergy details with actions to close or edit.

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Widget};

use crate::ui::theme::Theme;
use opengp_domain::domain::clinical::Allergy;

/// Actions returned by the allergy detail modal's key handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllergyDetailModalAction {
    /// Close the modal
    Close,
    /// Edit the allergy
    Edit,
}

/// Allergy detail modal widget.
///
/// Displays read-only allergy information with options to close or edit.
/// Follows the modal pattern: centered, with clear background, Escape to close.
#[derive(Clone)]
pub struct AllergyDetailModal {
    /// The allergy data to display
    allergy: Allergy,
    /// Theme for styling
    theme: Theme,
    /// Which button is focused (raw index)
    focused_button: usize,
}

impl AllergyDetailModal {
    /// Create a new allergy detail modal.
    pub fn new(allergy: Allergy, theme: Theme) -> Self {
        Self {
            allergy,
            theme,
            focused_button: 0,
        }
    }

    /// Get the number of visible buttons.
    /// Always: Close, Edit
    fn button_count(&self) -> usize {
        2
    }

    /// Move focus to the next button.
    pub fn next_button(&mut self) {
        let count = self.button_count();
        self.focused_button = (self.focused_button + 1) % count;
    }

    /// Move focus to the previous button.
    pub fn prev_button(&mut self) {
        let count = self.button_count();
        self.focused_button = if self.focused_button == 0 {
            count - 1
        } else {
            self.focused_button - 1
        };
    }

    pub fn allergy_id(&self) -> uuid::Uuid {
        self.allergy.id
    }

    /// Get the status display text.
    fn format_status(&self) -> String {
        if self.allergy.is_active {
            "Active".to_string()
        } else {
            "Inactive".to_string()
        }
    }

    /// Get the status color.
    fn get_status_color(&self) -> ratatui::style::Color {
        if self.allergy.is_active {
            self.theme.colors.success
        } else {
            self.theme.colors.disabled
        }
    }

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyDetailModalAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Esc => Some(AllergyDetailModalAction::Close),
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_button();
                } else {
                    self.next_button();
                }
                None
            }
            KeyCode::BackTab => {
                self.prev_button();
                None
            }
            KeyCode::Left | KeyCode::Up => {
                self.prev_button();
                None
            }
            KeyCode::Right | KeyCode::Down => {
                self.next_button();
                None
            }
            KeyCode::Enter => self.handle_enter_on_focused_button(),
            _ => None,
        }
    }

    /// Return the action for the currently focused button when Enter is pressed.
    fn handle_enter_on_focused_button(&self) -> Option<AllergyDetailModalAction> {
        match self.focused_button {
            0 => Some(AllergyDetailModalAction::Close),
            1 => Some(AllergyDetailModalAction::Edit),
            _ => None,
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for AllergyDetailModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let modal_width = (area.width as f32 * 0.6) as u16;
        let modal_width = modal_width.clamp(50, 80);

        let content_lines = 8;
        let modal_height = (content_lines as u16).min(area.height.saturating_sub(4));

        let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
        let y = area.y + (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect::new(x, y, modal_width, modal_height);

        let bg_style = Style::default().bg(self.theme.colors.background);
        Clear.render(modal_area, buf);

        for row in modal_area.top()..modal_area.bottom() {
            for col in modal_area.left()..modal_area.right() {
                if let Some(cell) = buf.cell_mut((col, row)) {
                    cell.set_style(bg_style);
                }
            }
        }

        let block = Block::default()
            .title(" Allergy Detail ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.primary));

        block.clone().render(modal_area, buf);

        let inner = block.inner(modal_area);
        if inner.is_empty() {
            return;
        }

        let label_width = 18u16;
        let value_x = inner.x + label_width;
        let value_width = inner.width.saturating_sub(label_width + 2);

        let mut y = inner.y + 1;

        // Render Allergen
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Allergen:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.allergy.allergen,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Allergy Type
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Type:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.allergy.allergy_type.to_string(),
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Severity
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Severity:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.allergy.severity.to_string(),
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Reaction
        let reaction = self.allergy.reaction.as_deref().unwrap_or("(none)");
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Reaction:", label_style);
            let display_reaction = if reaction.len() > value_width as usize {
                format!("{}...", &reaction[..value_width as usize - 3])
            } else {
                reaction.to_string()
            };
            buf.set_string(
                value_x,
                y,
                &display_reaction,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Onset Date
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Onset Date:", label_style);
            let onset_str = self
                .allergy
                .onset_date
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            buf.set_string(
                value_x,
                y,
                &onset_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Notes
        let notes = self.allergy.notes.as_deref().unwrap_or("(none)");
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Notes:", label_style);
            let display_notes = if notes.len() > value_width as usize {
                format!("{}...", &notes[..value_width as usize - 3])
            } else {
                notes.to_string()
            };
            buf.set_string(
                value_x,
                y,
                &display_notes,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Status
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Status:", label_style);
            let status_color = self.get_status_color();
            buf.set_string(
                value_x,
                y,
                &self.format_status(),
                Style::default().fg(status_color),
            );
            y += 1;
        }

        // Render buttons at the bottom
        y += 1;

        let buttons: Vec<(&str, bool)> = vec![
            ("[C]lose", self.focused_button == 0),
            ("[E]dit", self.focused_button == 1),
        ];

        let button_width = 12u16;
        let spacing = 2u16;
        let total_buttons_width = button_width * buttons.len() as u16
            + spacing * (buttons.len().saturating_sub(1)) as u16;
        let button_start_offset = (inner.width.saturating_sub(total_buttons_width)) / 2;
        let button_start_x = inner.x + button_start_offset;

        let mut current_x = button_start_x;
        for (label, is_focused) in &buttons {
            let style = if *is_focused {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };
            buf.set_string(current_x, y, label, style);
            current_x += button_width + spacing;
        }

        // Help text
        y += 1;
        let help_text = "Tab: Navigate | Enter: Select | Esc: Close";
        let help_x = inner.x + (inner.width.saturating_sub(help_text.len() as u16)) / 2;
        buf.set_string(
            help_x,
            y,
            help_text,
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_allergy() -> Allergy {
        use chrono::Utc;
        use opengp_domain::domain::clinical::AllergyType;
        use opengp_domain::domain::clinical::Severity;

        Allergy {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            allergen: "Penicillin".to_string(),
            allergy_type: AllergyType::Drug,
            severity: Severity::Severe,
            reaction: Some("Anaphylaxis".to_string()),
            onset_date: None,
            notes: Some("Severe reaction observed".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        }
    }

    fn make_modal() -> AllergyDetailModal {
        let allergy = make_allergy();
        AllergyDetailModal::new(allergy, Theme::dark())
    }

    #[test]
    fn test_allergy_detail_modal_esc_returns_close() {
        let mut modal = make_modal();
        let esc_key = KeyEvent::new(crossterm::event::KeyCode::Esc, KeyModifiers::empty());
        let action = modal.handle_key(esc_key);
        assert_eq!(action, Some(AllergyDetailModalAction::Close));
    }

    #[test]
    fn test_button_navigation() {
        let mut modal = make_modal();
        assert_eq!(modal.focused_button, 0);

        modal.next_button();
        assert_eq!(modal.focused_button, 1);

        modal.next_button();
        assert_eq!(modal.focused_button, 0);

        modal.prev_button();
        assert_eq!(modal.focused_button, 1);
    }

    #[test]
    fn test_enter_on_close_button() {
        let mut modal = make_modal();
        modal.focused_button = 0;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(AllergyDetailModalAction::Close));
    }

    #[test]
    fn test_enter_on_edit_button() {
        let mut modal = make_modal();
        modal.focused_button = 1;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(AllergyDetailModalAction::Edit));
    }
}
