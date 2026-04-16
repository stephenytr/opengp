//! Vital Signs Detail Modal Component
//!
//! Read-only modal displaying vital signs details with actions to close or edit.

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Widget};

use crate::ui::theme::Theme;
use opengp_domain::domain::clinical::VitalSigns;

/// Actions returned by the vital signs detail modal's key handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VitalsDetailModalAction {
    /// Close the modal
    Close,
    /// Edit the vital signs
    Edit,
}

/// Vital signs detail modal widget.
///
/// Displays read-only vital signs information with options to close or edit.
/// Follows the modal pattern: centered, with clear background, Escape to close.
#[derive(Clone)]
pub struct VitalsDetailModal {
    /// The vital signs data to display
    vitals: VitalSigns,
    /// Theme for styling
    theme: Theme,
    /// Which button is focused (raw index)
    focused_button: usize,
}

impl VitalsDetailModal {
    /// Create a new vital signs detail modal.
    pub fn new(vitals: VitalSigns, theme: Theme) -> Self {
        Self {
            vitals,
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

    pub fn vitals_id(&self) -> uuid::Uuid {
        self.vitals.id
    }

    /// Format blood pressure display.
    fn format_blood_pressure(&self) -> String {
        match (self.vitals.systolic_bp, self.vitals.diastolic_bp) {
            (Some(sys), Some(dia)) => format!("{}/{}", sys, dia),
            (Some(sys), None) => format!("{}/—", sys),
            (None, Some(dia)) => format!("—/{}", dia),
            (None, None) => "—/—".to_string(),
        }
    }

    /// Format BMI display (no recalculation, display stored value).
    fn format_bmi(&self) -> String {
        self.vitals
            .bmi
            .map(|b| format!("{:.1}", b))
            .unwrap_or_else(|| "-".to_string())
    }

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalsDetailModalAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Esc => Some(VitalsDetailModalAction::Close),
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
    fn handle_enter_on_focused_button(&self) -> Option<VitalsDetailModalAction> {
        match self.focused_button {
            0 => Some(VitalsDetailModalAction::Close),
            1 => Some(VitalsDetailModalAction::Edit),
            _ => None,
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for VitalsDetailModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let modal_width = (area.width as f32 * 0.6) as u16;
        let modal_width = modal_width.clamp(50, 80);

        let content_lines = 12;
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
            .title(" Vital Signs Detail ")
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

        // Render Measured At
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Measured At:", label_style);
            let measured_str = self.vitals.measured_at.format("%Y-%m-%d %H:%M").to_string();
            buf.set_string(
                value_x,
                y,
                &measured_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Blood Pressure
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Blood Pressure:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.format_blood_pressure(),
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Heart Rate
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Heart Rate:", label_style);
            let hr_str = self
                .vitals
                .heart_rate
                .map(|hr| format!("{} bpm", hr))
                .unwrap_or_else(|| "—".to_string());
            buf.set_string(
                value_x,
                y,
                &hr_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Respiratory Rate
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Respiratory Rate:", label_style);
            let rr_str = self
                .vitals
                .respiratory_rate
                .map(|rr| format!("{} /min", rr))
                .unwrap_or_else(|| "—".to_string());
            buf.set_string(
                value_x,
                y,
                &rr_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Temperature
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Temperature:", label_style);
            let temp_str = self
                .vitals
                .temperature
                .map(|t| format!("{:.1}°C", t))
                .unwrap_or_else(|| "—".to_string());
            buf.set_string(
                value_x,
                y,
                &temp_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Oxygen Saturation
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "O₂ Saturation:", label_style);
            let o2_str = self
                .vitals
                .oxygen_saturation
                .map(|o2| format!("{}%", o2))
                .unwrap_or_else(|| "—".to_string());
            buf.set_string(
                value_x,
                y,
                &o2_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Height
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Height:", label_style);
            let height_str = self
                .vitals
                .height_cm
                .map(|h| format!("{} cm", h))
                .unwrap_or_else(|| "—".to_string());
            buf.set_string(
                value_x,
                y,
                &height_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Weight
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Weight:", label_style);
            let weight_str = self
                .vitals
                .weight_kg
                .map(|w| format!("{:.1} kg", w))
                .unwrap_or_else(|| "—".to_string());
            buf.set_string(
                value_x,
                y,
                &weight_str,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render BMI
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "BMI:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.format_bmi(),
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Notes
        let notes = self.vitals.notes.as_deref().unwrap_or("(none)");
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

    fn make_vitals() -> VitalSigns {
        use chrono::Utc;

        VitalSigns {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            consultation_id: None,
            measured_at: Utc::now(),
            systolic_bp: Some(120),
            diastolic_bp: Some(80),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(37.0),
            oxygen_saturation: Some(98),
            height_cm: Some(175),
            weight_kg: Some(75.5),
            bmi: Some(24.6),
            notes: Some("Normal vitals".to_string()),
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    fn make_modal() -> VitalsDetailModal {
        let vitals = make_vitals();
        VitalsDetailModal::new(vitals, Theme::dark())
    }

    #[test]
    fn test_vitals_detail_modal_esc_returns_close() {
        let mut modal = make_modal();
        let esc_key = KeyEvent::new(crossterm::event::KeyCode::Esc, KeyModifiers::empty());
        let action = modal.handle_key(esc_key);
        assert_eq!(action, Some(VitalsDetailModalAction::Close));
    }

    #[test]
    fn test_vitals_detail_modal_bmi_format() {
        let vitals = make_vitals();
        let modal = VitalsDetailModal::new(vitals, Theme::dark());
        let bmi_str = modal.format_bmi();
        assert_eq!(bmi_str, "24.6");
    }

    #[test]
    fn test_vitals_detail_modal_bmi_none() {
        let mut vitals = make_vitals();
        vitals.bmi = None;
        let modal = VitalsDetailModal::new(vitals, Theme::dark());
        let bmi_str = modal.format_bmi();
        assert_eq!(bmi_str, "-");
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
        assert_eq!(action, Some(VitalsDetailModalAction::Close));
    }

    #[test]
    fn test_enter_on_edit_button() {
        let mut modal = make_modal();
        modal.focused_button = 1;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(VitalsDetailModalAction::Edit));
    }
}
