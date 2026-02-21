//! Consultation List Component
//!
//! Displays a list of patient consultations with date, practitioner, reason, and status.

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};
use uuid::Uuid;

use crate::domain::clinical::Consultation;
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingState;

pub struct ConsultationList {
    pub consultations: Vec<Consultation>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
    /// Optional practitioner names map (practitioner_id -> name)
    practitioner_names: std::collections::HashMap<Uuid, String>,
}

impl Clone for ConsultationList {
    fn clone(&self) -> Self {
        Self {
            consultations: self.consultations.clone(),
            selected_index: self.selected_index,
            scroll_offset: self.scroll_offset,
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
            practitioner_names: self.practitioner_names.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConsultationListAction {
    Select(usize),
    Open(Consultation),
    New,
    NextPage,
    PrevPage,
}

impl ConsultationList {
    pub fn new(theme: Theme) -> Self {
        Self {
            consultations: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading consultations..."),
            theme,
            practitioner_names: std::collections::HashMap::new(),
        }
    }

    pub fn with_consultations(consultations: Vec<Consultation>, theme: Theme) -> Self {
        Self {
            consultations,
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading consultations..."),
            theme,
            practitioner_names: std::collections::HashMap::new(),
        }
    }

    pub fn set_practitioner_name(&mut self, practitioner_id: Uuid, name: String) {
        self.practitioner_names.insert(practitioner_id, name);
    }

    pub fn selected(&self) -> Option<&Consultation> {
        self.consultations.get(self.selected_index)
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected().map(|c| c.id)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.consultations.len() {
            self.selected_index = index;
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn next(&mut self) {
        if self.selected_index + 1 < self.consultations.len() {
            self.selected_index += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.consultations.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.consultations.len().saturating_sub(1);
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index.saturating_sub(visible_rows) + 1;
        }
    }

    pub fn move_up_and_scroll(&mut self, visible_rows: usize) {
        self.move_up();
        self.adjust_scroll(visible_rows);
    }

    pub fn move_down_and_scroll(&mut self, visible_rows: usize) {
        self.move_down();
        self.adjust_scroll(visible_rows);
    }

    pub fn has_selection(&self) -> bool {
        !self.consultations.is_empty()
    }

    pub fn count(&self) -> usize {
        self.consultations.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Option<ConsultationListAction> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Some(ConsultationListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Some(ConsultationListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                Some(ConsultationListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                Some(ConsultationListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                let new_index = self.selected_index.saturating_sub(10);
                self.selected_index = new_index;
                Some(ConsultationListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                let new_index =
                    (self.selected_index + 10).min(self.consultations.len().saturating_sub(1));
                self.selected_index = new_index;
                Some(ConsultationListAction::Select(self.selected_index))
            }
            KeyCode::Enter => {
                if let Some(consultation) = self.selected() {
                    Some(ConsultationListAction::Open(consultation.clone()))
                } else {
                    None
                }
            }
            KeyCode::Char('n') => Some(ConsultationListAction::New),
            KeyCode::Char('+') | KeyCode::Char('=') => Some(ConsultationListAction::NextPage),
            KeyCode::Char('-') => Some(ConsultationListAction::PrevPage),
            _ => None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        area: Rect,
    ) -> Option<ConsultationListAction> {
        // Handle mouse wheel for scrolling
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3).max(0);
            }
            return Some(ConsultationListAction::Select(self.selected_index));
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.consultations.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            return Some(ConsultationListAction::Select(self.selected_index));
        }

        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        if mouse.row < area.y + 2 {
            return None;
        }

        let row_index = (mouse.row - area.y - 2) as usize;
        let actual_index = self.scroll_offset + row_index;
        if actual_index < self.consultations.len() {
            self.selected_index = actual_index;
            Some(ConsultationListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

// Column widths for consultation list
const COL_DATE: u16 = 12;
const COL_PRACTITIONER: u16 = 20;
const COL_REASON: u16 = 30;
const COL_STATUS: u16 = 10;

fn format_date(consultation: &Consultation) -> String {
    consultation
        .consultation_date
        .format("%d/%m/%Y")
        .to_string()
}

fn format_practitioner(
    consultation: &Consultation,
    names: &std::collections::HashMap<Uuid, String>,
) -> String {
    names
        .get(&consultation.practitioner_id)
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_reason(consultation: &Consultation) -> String {
    consultation
        .soap_notes
        .assessment
        .as_ref()
        .or(consultation.soap_notes.subjective.as_ref())
        .map(|s| {
            if s.len() > 28 {
                format!("{}...", &s[..28])
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn format_status(consultation: &Consultation) -> String {
    if consultation.is_signed {
        "Signed".to_string()
    } else {
        "Unsigned".to_string()
    }
}

impl Widget for ConsultationList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Consultations ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        if self.loading {
            let mut loading_state = self.loading_state.clone();
            loading_state.tick();
            let indicator = loading_state.to_indicator(self.theme.clone());
            indicator.render(inner, buf);
            return;
        }

        if self.consultations.is_empty() {
            let message = "No consultations found. Press n to add a new consultation.";
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
            return;
        }

        let col_widths = [
            Constraint::Length(COL_DATE),
            Constraint::Length(COL_PRACTITIONER),
            Constraint::Length(COL_REASON),
            Constraint::Length(COL_STATUS),
        ];

        let header = Row::new(vec!["Date", "Practitioner", "Reason", "Status"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = inner.height as usize;
        let max_scroll = self.consultations.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .consultations
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, consultation)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == self.selected_index {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                let status_text = format_status(consultation);
                let _status_style = if consultation.is_signed {
                    Style::default().fg(self.theme.colors.success)
                } else {
                    Style::default().fg(self.theme.colors.warning)
                };

                Row::new(vec![
                    format_date(consultation),
                    format_practitioner(consultation, &self.practitioner_names),
                    format_reason(consultation),
                    status_text,
                ])
                .style(style)
                .height(1)
            })
            .collect();

        let table = Table::new(rows, col_widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .widths(col_widths);

        table.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_consultation_list_empty() {
        let theme = Theme::dark();
        let list = ConsultationList::new(theme);
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn test_consultation_list_navigation() {
        let theme = Theme::dark();
        let mut list = ConsultationList::new(theme);

        let consultations = vec![
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
        ];
        list.consultations = consultations;

        assert!(list.has_selection());

        list.move_first();
        assert_eq!(list.selected_index, 0);

        list.move_down();
        assert_eq!(list.selected_index, 1);

        list.move_up();
        assert_eq!(list.selected_index, 0);
    }
}
