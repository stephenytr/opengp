//! Patient List Component
//!
//! Displays a searchable list of patients with pagination.

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};
use sublime_fuzzy::best_match;
use uuid::Uuid;

use crate::ui::layout::{
    HEADER_HEIGHT, PATIENT_COL_DOB, PATIENT_COL_LAST_VISIT, PATIENT_COL_MEDICARE, PATIENT_COL_NAME,
    PATIENT_COL_PHONE,
};
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;
use crate::ui::widgets::{LoadingState, ScrollableState};

pub struct PatientList {
    patients: Vec<PatientListItem>,
    filtered: Vec<PatientListItem>,
    search_query: String,
    searching: bool,
    scrollable: ScrollableState,
    loading: bool,
    loading_state: LoadingState,
    theme: Theme,
}

impl Clone for PatientList {
    fn clone(&self) -> Self {
        Self {
            patients: self.patients.clone(),
            filtered: self.filtered.clone(),
            search_query: self.search_query.clone(),
            searching: self.searching,
            scrollable: self.scrollable.clone(),
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
        }
    }
}

impl PatientList {
    pub fn new(theme: Theme) -> Self {
        Self {
            patients: Vec::new(),
            filtered: Vec::new(),
            search_query: String::new(),
            searching: false,
            scrollable: ScrollableState::new(),
            loading: false,
            loading_state: LoadingState::new().message("Loading patients..."),
            theme,
        }
    }

    pub fn set_patients(&mut self, patients: Vec<PatientListItem>) {
        self.patients = patients;
        self.apply_filter();
        self.scrollable = ScrollableState::new();
        self.scrollable.set_item_count(self.filtered.len());
    }

    pub fn patients(&self) -> &[PatientListItem] {
        &self.patients
    }

    pub fn selected_patient(&self) -> Option<&PatientListItem> {
        self.filtered.get(self.scrollable.selected_index())
    }

    pub fn selected_patient_id(&self) -> Option<Uuid> {
        self.selected_patient().map(|p| p.id)
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn is_searching(&self) -> bool {
        self.searching
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.apply_filter();
        self.scrollable = ScrollableState::new();
        self.scrollable.set_item_count(self.filtered.len());
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered = self.patients.clone();
            return;
        }

        let query = self.search_query.to_lowercase();

        let mut matches: Vec<(usize, i64)> = self
            .patients
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let searchable = format!(
                    "{} {} {}",
                    p.full_name,
                    p.medicare_number.as_deref().unwrap_or(""),
                    p.phone_mobile.as_deref().unwrap_or("")
                );

                best_match(&query, &searchable).map(|result| (i, result.score() as i64))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));

        self.filtered = matches
            .into_iter()
            .map(|(i, _)| self.patients[i].clone())
            .collect();
    }

    pub fn move_up(&mut self) {
        self.scrollable.move_up();
    }

    pub fn move_down(&mut self) {
        self.scrollable.move_down();
    }

    pub fn move_first(&mut self) {
        self.scrollable.move_first();
    }

    pub fn move_last(&mut self) {
        self.scrollable.move_last();
    }

    pub fn move_by(&mut self, offset: isize) {
        self.scrollable.move_by(offset);
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        self.scrollable.adjust_scroll(visible_rows);
    }

    pub fn move_up_and_scroll(&mut self, visible_rows: usize) {
        self.scrollable.move_up_and_scroll(visible_rows);
    }

    pub fn move_down_and_scroll(&mut self, visible_rows: usize) {
        self.scrollable.move_down_and_scroll(visible_rows);
    }

    pub fn move_first_and_scroll(&mut self, visible_rows: usize) {
        self.scrollable.move_first_and_scroll(visible_rows);
    }

    pub fn move_last_and_scroll(&mut self, visible_rows: usize) {
        self.scrollable.move_last_and_scroll(visible_rows);
    }

    pub fn move_by_and_scroll(&mut self, offset: isize, visible_rows: usize) {
        self.scrollable.move_by_and_scroll(offset, visible_rows);
    }

    pub fn has_selection(&self) -> bool {
        !self.filtered.is_empty()
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered.len()
    }

    pub fn total_count(&self) -> usize {
        self.patients.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scrollable.scroll_offset()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PatientListAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        // Ignore non-press key events (e.g., Release events from terminals with keyboard enhancement)
        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Handle search input mode
        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.search_query.clear();
                    self.apply_filter();
                    self.scrollable = ScrollableState::new();
                    self.scrollable.set_item_count(self.filtered.len());
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                    self.scrollable = ScrollableState::new();
                    self.scrollable.set_item_count(self.filtered.len());
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.apply_filter();
                    self.scrollable = ScrollableState::new();
                    self.scrollable.set_item_count(self.filtered.len());
                }
                KeyCode::Enter => {
                    self.searching = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_up();
                    return Some(PatientListAction::SelectionChanged);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_down();
                    return Some(PatientListAction::SelectionChanged);
                }
                _ => {}
            }
            return Some(PatientListAction::SearchChanged);
        }

        // Normal navigation mode
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Some(PatientListAction::SelectionChanged)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Some(PatientListAction::SelectionChanged)
            }
            KeyCode::Home => {
                self.move_first();
                Some(PatientListAction::SelectionChanged)
            }
            KeyCode::End => {
                self.move_last();
                Some(PatientListAction::SelectionChanged)
            }
            KeyCode::PageUp => {
                self.move_by(-10);
                Some(PatientListAction::SelectionChanged)
            }
            KeyCode::PageDown => {
                self.move_by(10);
                Some(PatientListAction::SelectionChanged)
            }
            KeyCode::Enter => {
                if self.has_selection() {
                    // SAFETY: has_selection() confirmed filtered is not empty
                    #[allow(clippy::unwrap_used)]
                    Some(PatientListAction::OpenPatient(
                        self.selected_patient_id().unwrap(),
                    ))
                } else {
                    None
                }
            }
            KeyCode::Char('/') => {
                self.searching = true;
                Some(PatientListAction::FocusSearch)
            }
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientListAction> {
        // Handle mouse wheel for scrolling
        if let MouseEventKind::ScrollUp = mouse.kind {
            for _ in 0..3 {
                self.scrollable.scroll_up();
            }
            return Some(PatientListAction::SelectionChanged);
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.filtered.len().saturating_sub(visible_rows);
            for _ in 0..3 {
                if self.scrollable.scroll_offset() < max_scroll {
                    self.scrollable.scroll_down();
                }
            }
            return Some(PatientListAction::SelectionChanged);
        }

        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        if mouse.row < area.y + HEADER_HEIGHT {
            return None;
        }

        let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
        // Account for scroll offset
        let actual_index = self.scrollable.scroll_offset() + row_index;
        if actual_index < self.filtered.len() {
            // Move selection to the clicked item
            let current_index = self.scrollable.selected_index();
            let offset = actual_index as isize - current_index as isize;
            self.scrollable.move_by(offset);
            Some(PatientListAction::SelectionChanged)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum PatientListAction {
    SelectionChanged,
    OpenPatient(Uuid),
    FocusSearch,
    SearchChanged,
}

fn format_name(patient: &PatientListItem) -> String {
    patient.full_name.clone()
}

fn format_dob(patient: &PatientListItem) -> String {
    patient.date_of_birth.format("%d/%m/%y").to_string()
}

fn format_medicare(patient: &PatientListItem) -> String {
    patient
        .medicare_number
        .as_deref()
        .unwrap_or("-")
        .to_string()
}

fn format_phone(patient: &PatientListItem) -> String {
    patient.phone_mobile.as_deref().unwrap_or("-").to_string()
}

fn format_last_visit(_patient: &PatientListItem) -> String {
    "-".to_string()
}

impl Widget for PatientList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Patients ")
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

        if self.filtered.is_empty() {
            let message = if self.search_query.is_empty() {
                "No patients found. Press n to add a new patient."
            } else {
                "No patients match your search."
            };
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
            Constraint::Length(PATIENT_COL_NAME),
            Constraint::Length(PATIENT_COL_DOB),
            Constraint::Length(PATIENT_COL_MEDICARE),
            Constraint::Length(PATIENT_COL_PHONE),
            Constraint::Length(PATIENT_COL_LAST_VISIT),
        ];

        let header = Row::new(vec!["Name", "DOB", "Medicare #", "Phone", "Last Visit"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = inner.height as usize;
        let max_scroll = self.filtered.len().saturating_sub(visible_rows);
        let scroll_offset = self.scrollable.scroll_offset().min(max_scroll);
        let selected_index = self.scrollable.selected_index();

        let rows: Vec<Row> = self
            .filtered
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, patient)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == selected_index {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                Row::new(vec![
                    format_name(patient),
                    format_dob(patient),
                    format_medicare(patient),
                    format_phone(patient),
                    format_last_visit(patient),
                ])
                .style(style)
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
    use chrono::NaiveDate;
    use opengp_domain::domain::patient::Patient;

    fn create_test_patient(first: &str, last: &str) -> PatientListItem {
        let patient = Patient::new(
            first.to_string(),
            last.to_string(),
            NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            opengp_domain::domain::patient::Gender::Male,
            None,
            Some(opengp_domain::domain::patient::MedicareNumber::new_lenient(
                "1234567890".to_string(),
            )),
            None,
            None,
            None,
            None,
            None,
            opengp_domain::domain::patient::Address::default(),
            None,
            Some(opengp_domain::domain::patient::PhoneNumber::new_lenient(
                "0412345678".to_string(),
            )),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        PatientListItem::from(patient)
    }

    #[test]
    fn test_patient_list_empty() {
        let theme = Theme::dark();
        let list = PatientList::new(theme);
        assert_eq!(list.filtered_count(), 0);
    }

    #[test]
    fn test_patient_list_filter() {
        let theme = Theme::dark();
        let mut list = PatientList::new(theme);

        let patients = vec![
            create_test_patient("John", "Smith"),
            create_test_patient("Jane", "Doe"),
            create_test_patient("Bob", "Smith"),
        ];
        list.set_patients(patients);

        assert_eq!(list.total_count(), 3);
        assert_eq!(list.filtered_count(), 3);

        list.set_search_query("smith".to_string());
        assert_eq!(list.filtered_count(), 2);
    }

    #[test]
    fn test_patient_list_navigation() {
        let theme = Theme::dark();
        let mut list = PatientList::new(theme);

        let patients = vec![
            create_test_patient("John", "Smith"),
            create_test_patient("Jane", "Doe"),
            create_test_patient("Bob", "Jones"),
        ];
        list.set_patients(patients);

        // With 3 patients, filtered is not empty, so has_selection returns true
        assert!(list.has_selection());

        list.move_first();
        assert!(list.selected_patient().unwrap().full_name.contains("John"));

        list.move_down();
        assert!(list.selected_patient().unwrap().full_name.contains("Jane"));

        list.move_up();
        assert!(list.selected_patient().unwrap().full_name.contains("John"));
    }
}
