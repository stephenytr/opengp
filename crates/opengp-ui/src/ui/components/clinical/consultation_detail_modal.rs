//! Consultation Detail Modal Component
//!
//! Read-only modal displaying consultation details with actions to close, edit, sign, or stop timer.

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Widget};

use crate::ui::theme::Theme;
use opengp_domain::domain::clinical::Consultation;

/// Actions returned by the consultation detail modal's key handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsultationDetailModalAction {
    /// Close the modal
    Close,
    /// Edit the consultation
    Edit,
    /// Sign the consultation
    Sign,
    /// Stop the running timer
    StopTimer,
}

/// Consultation detail modal widget.
///
/// Displays read-only consultation information with options to edit, sign, or stop timer.
/// Follows the modal pattern: centered, with clear background, Escape to close.
#[derive(Clone)]
pub struct ConsultationDetailModal {
    /// The consultation data to display
    consultation: Consultation,
    /// Patient name (pre-resolved, not stored in consultation)
    patient_name: String,
    /// Practitioner name (pre-resolved, not stored in consultation)
    practitioner_name: String,
    /// Theme for styling
    theme: Theme,
    /// Which button is focused (raw index)
    focused_button: usize,
}

impl ConsultationDetailModal {
    /// Create a new consultation detail modal.
    pub fn new(
        consultation: Consultation,
        patient_name: String,
        practitioner_name: String,
        theme: Theme,
    ) -> Self {
        Self {
            consultation,
            patient_name,
            practitioner_name,
            theme,
            focused_button: 0,
        }
    }

    /// Get the number of visible buttons.
    /// Always: Close, Edit
    /// If unsigned: Sign
    /// If timer running: Stop Timer
    fn button_count(&self) -> usize {
        let mut count = 2; // Close, Edit

        if !self.consultation.is_signed {
            count += 1; // Sign
        }

        if self.consultation.consultation_started_at.is_some()
            && self.consultation.consultation_ended_at.is_none()
        {
            count += 1; // Stop Timer
        }

        count
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

    pub fn is_signed(&self) -> bool {
        self.consultation.is_signed
    }

    pub fn consultation_id(&self) -> uuid::Uuid {
        self.consultation.id
    }

    /// Format the consultation date.
    fn format_date(&self) -> String {
        self.consultation
            .consultation_date
            .format("%A %d %B %Y at %H:%M")
            .to_string()
    }

    /// Format the status (signed or draft).
    fn format_status(&self) -> String {
        if self.consultation.is_signed {
            if let Some(signed_at) = self.consultation.signed_at {
                format!(
                    "✓ SIGNED {} by {}",
                    signed_at.format("%H:%M on %d/%m/%Y"),
                    self.practitioner_name
                )
            } else {
                "✓ SIGNED".to_string()
            }
        } else {
            "DRAFT".to_string()
        }
    }

    /// Get the status color.
    fn get_status_color(&self) -> ratatui::style::Color {
        if self.consultation.is_signed {
            self.theme.colors.success
        } else {
            self.theme.colors.warning
        }
    }

    /// Calculate duration if both start and end times are present.
    fn format_duration(&self) -> Option<String> {
        match (
            self.consultation.consultation_started_at,
            self.consultation.consultation_ended_at,
        ) {
            (Some(start), Some(end)) => {
                let duration = end.signed_duration_since(start);
                let minutes = duration.num_minutes();
                Some(format!("{} min", minutes))
            }
            _ => None,
        }
    }

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConsultationDetailModalAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Esc => Some(ConsultationDetailModalAction::Close),
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
    fn handle_enter_on_focused_button(&self) -> Option<ConsultationDetailModalAction> {
        let mut button_index = 0;

        // Button 0: Close
        if button_index == self.focused_button {
            return Some(ConsultationDetailModalAction::Close);
        }
        button_index += 1;

        // Button 1: Edit
        if button_index == self.focused_button {
            return Some(ConsultationDetailModalAction::Edit);
        }
        button_index += 1;

        // Button 2: Sign (only if unsigned)
        if !self.consultation.is_signed {
            if button_index == self.focused_button {
                return Some(ConsultationDetailModalAction::Sign);
            }
            button_index += 1;
        }

        // Button 3+: Stop Timer (only if timer running)
        if self.consultation.consultation_started_at.is_some()
            && self.consultation.consultation_ended_at.is_none()
        {
            if button_index == self.focused_button {
                return Some(ConsultationDetailModalAction::StopTimer);
            }
        }

        None
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for ConsultationDetailModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let modal_width = (area.width as f32 * 0.6) as u16;
        let modal_width = modal_width.clamp(50, 80);

        let mut content_lines = 7;
        if self.consultation.consultation_started_at.is_some()
            && self.consultation.consultation_ended_at.is_none()
        {
            content_lines += 1;
        }
        content_lines += 2;

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
            .title(" Consultation Detail ")
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

        // Render Patient Name
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Patient:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.patient_name,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Date
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Date:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.format_date(),
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Practitioner
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Practitioner:", label_style);
            buf.set_string(
                value_x,
                y,
                &self.practitioner_name,
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

        // Render Reason
        let reason = self.consultation.reason.as_deref().unwrap_or("(none)");
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Reason:", label_style);
            let display_reason = if reason.len() > value_width as usize {
                format!("{}...", &reason[..value_width as usize - 3])
            } else {
                reason.to_string()
            };
            buf.set_string(
                value_x,
                y,
                &display_reason,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Render Clinical Notes
        let clinical_notes = self
            .consultation
            .clinical_notes
            .as_deref()
            .unwrap_or("(none)");
        if y < inner.y + inner.height - 3 {
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, "Notes:", label_style);
            y += 1;

            let note_lines: Vec<&str> = clinical_notes.lines().collect();
            for line in note_lines.iter().take(3) {
                if y >= inner.y + inner.height - 3 {
                    break;
                }
                let display_line = if line.len() > value_width as usize {
                    format!("{}...", &line[..value_width as usize - 3])
                } else {
                    line.to_string()
                };
                buf.set_string(
                    value_x,
                    y,
                    &display_line,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }
        }

        // Render Duration
        if let Some(duration_str) = self.format_duration() {
            if y < inner.y + inner.height - 3 {
                let label_style = Style::default()
                    .fg(self.theme.colors.foreground)
                    .add_modifier(Modifier::BOLD);
                buf.set_string(inner.x + 1, y, "Duration:", label_style);
                buf.set_string(
                    value_x,
                    y,
                    &duration_str,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }
        }

        // Render Timer indicator
        if self.consultation.consultation_started_at.is_some()
            && self.consultation.consultation_ended_at.is_none()
        {
            if y < inner.y + inner.height - 3 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "⏱ Timer running",
                    Style::default()
                        .fg(self.theme.colors.warning)
                        .add_modifier(Modifier::BOLD),
                );
                y += 1;
            }
        }

        // Render buttons at the bottom
        y += 1;

        let mut buttons: Vec<(&str, bool)> = vec![
            ("[C]lose", self.focused_button == 0),
            ("[E]dit", self.focused_button == 1),
        ];

        let mut button_idx = 2;

        if !self.consultation.is_signed {
            buttons.push(("[S]ign", self.focused_button == button_idx));
            button_idx += 1;
        }

        if self.consultation.consultation_started_at.is_some()
            && self.consultation.consultation_ended_at.is_none()
        {
            buttons.push(("[T]imer", self.focused_button == button_idx));
        }

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

    fn make_consultation(is_signed: bool) -> Consultation {
        let mut consultation =
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4());
        if is_signed {
            consultation.sign(Uuid::new_v4());
        }
        consultation
    }

    fn make_modal(is_signed: bool) -> ConsultationDetailModal {
        let consultation = make_consultation(is_signed);
        ConsultationDetailModal::new(
            consultation,
            "Jane Doe".to_string(),
            "Dr. Smith".to_string(),
            Theme::dark(),
        )
    }

    #[test]
    fn test_consultation_detail_modal_signed_hides_sign_button() {
        let modal = make_modal(true);
        assert_eq!(modal.button_count(), 2);
    }

    #[test]
    fn test_consultation_detail_modal_unsigned_shows_sign_button() {
        let modal = make_modal(false);
        assert_eq!(modal.button_count(), 3);
    }

    #[test]
    fn test_consultation_detail_modal_esc_returns_close() {
        let mut modal = make_modal(false);
        let esc_key = KeyEvent::new(crossterm::event::KeyCode::Esc, KeyModifiers::empty());
        let action = modal.handle_key(esc_key);
        assert_eq!(action, Some(ConsultationDetailModalAction::Close));
    }

    #[test]
    fn test_button_navigation() {
        let mut modal = make_modal(false);
        assert_eq!(modal.focused_button, 0);

        modal.next_button();
        assert_eq!(modal.focused_button, 1);

        modal.next_button();
        assert_eq!(modal.focused_button, 2);

        modal.next_button();
        assert_eq!(modal.focused_button, 0);

        modal.prev_button();
        assert_eq!(modal.focused_button, 2);
    }

    #[test]
    fn test_enter_on_close_button() {
        let mut modal = make_modal(false);
        modal.focused_button = 0;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(ConsultationDetailModalAction::Close));
    }

    #[test]
    fn test_enter_on_edit_button() {
        let mut modal = make_modal(false);
        modal.focused_button = 1;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(ConsultationDetailModalAction::Edit));
    }

    #[test]
    fn test_enter_on_sign_button() {
        let mut modal = make_modal(false);
        modal.focused_button = 2;
        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(enter_key);
        assert_eq!(action, Some(ConsultationDetailModalAction::Sign));
    }

    #[test]
    fn test_timer_running_adds_button() {
        let mut consultation = make_consultation(false);
        consultation.start_timer();
        let modal = ConsultationDetailModal::new(
            consultation,
            "Jane Doe".to_string(),
            "Dr. Smith".to_string(),
            Theme::dark(),
        );
        assert_eq!(modal.button_count(), 4);
    }

    #[test]
    fn test_format_status_signed() {
        let modal = make_modal(true);
        let status = modal.format_status();
        assert!(status.contains("SIGNED"));
        assert!(status.contains("Dr. Smith"));
    }

    #[test]
    fn test_format_status_unsigned() {
        let modal = make_modal(false);
        let status = modal.format_status();
        assert_eq!(status, "DRAFT");
    }
}
