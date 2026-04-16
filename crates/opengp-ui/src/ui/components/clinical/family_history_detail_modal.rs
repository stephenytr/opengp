//! Family History Detail Modal Component
//!
//! Read-only modal displaying family history details with actions to close or edit.

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Widget};

use crate::ui::theme::Theme;
use opengp_domain::domain::clinical::FamilyHistory;

/// Actions returned by the family history detail modal's key handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilyHistoryDetailModalAction {
    /// Close the modal
    Close,
    /// Edit the family history
    Edit,
}

/// Family history detail modal widget.
///
/// Displays read-only family history information with options to close or edit.
/// Follows the modal pattern: centered, with clear background, Escape to close.
#[derive(Clone)]
pub struct FamilyHistoryDetailModal {
    /// The family history data to display
    family_history: FamilyHistory,
    /// Theme for styling
    theme: Theme,
    /// Which button is focused (raw index)
    focused_button: usize,
}

impl FamilyHistoryDetailModal {
    /// Create a new family history detail modal.
    pub fn new(family_history: FamilyHistory, theme: Theme) -> Self {
        Self {
            family_history,
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

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<FamilyHistoryDetailModalAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Esc => Some(FamilyHistoryDetailModalAction::Close),
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
    fn handle_enter_on_focused_button(&self) -> Option<FamilyHistoryDetailModalAction> {
        match self.focused_button {
            0 => Some(FamilyHistoryDetailModalAction::Close),
            1 => Some(FamilyHistoryDetailModalAction::Edit),
            _ => None,
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for FamilyHistoryDetailModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let modal_width = (area.width as f32 * 0.6) as u16;
        let modal_width = modal_width.clamp(50, 80);

        let content_lines = 7;
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
            .title(" Family History Detail ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.primary));

        block.clone().render(modal_area, buf);

        let inner = block.inner(modal_area);
        if inner.is_empty() {
            return;
        }

        let label_width = 20u16;
        let value_x = inner.x + label_width;
        let value_width = inner.width.saturating_sub(label_width + 2);

        let mut y = inner.y + 1;

        // Render Relative Relationship
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Relationship:", label_style);
            let display_rel =
                if self.family_history.relative_relationship.len() > value_width as usize {
                    format!(
                        "{}...",
                        &self.family_history.relative_relationship[..value_width as usize - 3]
                    )
                } else {
                    self.family_history.relative_relationship.clone()
                };
            buf.set_string(
                value_x,
                y,
                &display_rel,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Condition
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Condition:", label_style);
            let display_cond = if self.family_history.condition.len() > value_width as usize {
                format!(
                    "{}...",
                    &self.family_history.condition[..value_width as usize - 3]
                )
            } else {
                self.family_history.condition.clone()
            };
            buf.set_string(
                value_x,
                y,
                &display_cond,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Age at Diagnosis
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Age at Diagnosis:", label_style);
            let age_str = self
                .family_history
                .age_at_diagnosis
                .map(|a| a.to_string())
                .unwrap_or_else(|| "-".to_string());
            buf.set_string(
                value_x,
                y,
                &age_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Notes
        let notes = self.family_history.notes.as_deref().unwrap_or("(none)");
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

    fn make_family_history() -> FamilyHistory {
        use chrono::Utc;

        FamilyHistory {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            relative_relationship: "Father".to_string(),
            condition: "Myocardial Infarction".to_string(),
            age_at_diagnosis: Some(55),
            notes: Some("Died from heart attack".to_string()),
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    fn make_modal() -> FamilyHistoryDetailModal {
        let family_history = make_family_history();
        FamilyHistoryDetailModal::new(family_history, Theme::dark())
    }

    #[test]
    fn test_family_history_detail_modal_esc_returns_close() {
        let mut modal = make_modal();
        let esc_key = KeyEvent::new(crossterm::event::KeyCode::Esc, KeyModifiers::empty());
        let action = modal.handle_key(esc_key);
        assert_eq!(action, Some(FamilyHistoryDetailModalAction::Close));
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
        assert_eq!(action, Some(FamilyHistoryDetailModalAction::Close));
    }

    #[test]
    fn test_enter_on_edit_button() {
        let mut modal = make_modal();
        modal.focused_button = 1;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(FamilyHistoryDetailModalAction::Edit));
    }
}
