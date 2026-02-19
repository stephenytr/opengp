//! Patient List Component
//!
//! Displays a searchable list of patients with pagination.

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};
use sublime_fuzzy::best_match;
use uuid::Uuid;

use crate::domain::patient::Patient;
use crate::ui::theme::Theme;

const COL_NAME: u16 = 25;
const COL_DOB: u16 = 10;
const COL_MEDICARE: u16 = 15;
const COL_PHONE: u16 = 15;
const COL_LAST_VISIT: u16 = 12;

pub struct PatientList {
    patients: Vec<Patient>,
    filtered: Vec<Patient>,
    search_query: String,
    searching: bool,
    selected_index: usize,
    scroll_offset: usize,
    loading: bool,
    theme: Theme,
}

impl Clone for PatientList {
    fn clone(&self) -> Self {
        Self {
            patients: self.patients.clone(),
            filtered: self.filtered.clone(),
            search_query: self.search_query.clone(),
            searching: self.searching,
            selected_index: self.selected_index,
            scroll_offset: self.scroll_offset,
            loading: self.loading,
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
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            theme,
        }
    }

    pub fn set_patients(&mut self, patients: Vec<Patient>) {
        self.patients = patients;
        self.apply_filter();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn selected_patient(&self) -> Option<&Patient> {
        self.filtered.get(self.selected_index)
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
        self.selected_index = 0;
        self.scroll_offset = 0;
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
                    "{} {} {} {}",
                    p.last_name,
                    p.first_name,
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
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.filtered.len().saturating_sub(1);
    }

    pub fn move_by(&mut self, offset: isize) {
        let new_index = (self.selected_index as isize + offset)
            .clamp(0, self.filtered.len().saturating_sub(1) as isize);
        self.selected_index = new_index as usize;
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

    pub fn move_first_and_scroll(&mut self, visible_rows: usize) {
        self.move_first();
        self.adjust_scroll(visible_rows);
    }

    pub fn move_last_and_scroll(&mut self, visible_rows: usize) {
        self.move_last();
        self.adjust_scroll(visible_rows);
    }

    pub fn move_by_and_scroll(&mut self, offset: isize, visible_rows: usize) {
        self.move_by(offset);
        self.adjust_scroll(visible_rows);
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
        self.scroll_offset
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PatientListAction> {
        use crossterm::event::KeyCode;

        // Handle search input mode
        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.search_query.clear();
                    self.apply_filter();
                    self.selected_index = 0;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                    self.selected_index = 0;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.apply_filter();
                    self.selected_index = 0;
                }
                KeyCode::Enter => {
                    self.searching = false;
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
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3).max(0);
            }
            return Some(PatientListAction::SelectionChanged);
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.filtered.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            return Some(PatientListAction::SelectionChanged);
        }

        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        let header_height = 2;
        if mouse.row < area.y + header_height {
            return None;
        }

        let row_index = (mouse.row - area.y - header_height) as usize;
        // Account for scroll offset
        let actual_index = self.scroll_offset + row_index;
        if actual_index < self.filtered.len() {
            self.selected_index = actual_index;
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

fn format_name(patient: &Patient) -> String {
    match &patient.preferred_name {
        Some(preferred) if !preferred.is_empty() => format!("{}, {}", patient.last_name, preferred),
        _ => format!("{}, {}", patient.last_name, patient.first_name),
    }
}

fn format_dob(patient: &Patient) -> String {
    format!(
        "{} ({})",
        patient.date_of_birth.format("%d/%m/%y"),
        patient.age()
    )
}

fn format_medicare(patient: &Patient) -> String {
    patient
        .medicare_number
        .clone()
        .unwrap_or_else(|| "-".to_string())
}

fn format_phone(patient: &Patient) -> String {
    patient
        .phone_mobile
        .clone()
        .or_else(|| patient.phone_home.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn format_last_visit(_patient: &Patient) -> String {
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

        // Reserve top line for search input when searching
        let content_area = if self.searching && inner.height > 1 {
            Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1)
        } else {
            inner
        };

        if self.loading {
            let text = Line::from(vec![Span::raw("Loading patients...")]);
            let x = inner.x + (inner.width.saturating_sub(17)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
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
            Constraint::Length(COL_NAME),
            Constraint::Length(COL_DOB),
            Constraint::Length(COL_MEDICARE),
            Constraint::Length(COL_PHONE),
            Constraint::Length(COL_LAST_VISIT),
        ];

        let header = Row::new(vec!["Name", "DOB", "Medicare #", "Phone", "Last Visit"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = content_area.height as usize;
        let max_scroll = self.filtered.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .filtered
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, patient)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == self.selected_index {
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

        table.render(content_area, buf);

        // Render search input AFTER table so it's visible
        if self.searching {
            let search_text = if self.search_query.is_empty() {
                Span::styled("/", Style::default().fg(self.theme.colors.primary).bold())
            } else {
                Span::from(format!("/{}", self.search_query))
            };
            let search_line = Line::from(vec![
                search_text,
                Span::styled(" _", Style::default().fg(self.theme.colors.disabled)),
            ]);
            buf.set_line(inner.x, inner.y, &search_line, inner.width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_patient(first: &str, last: &str) -> Patient {
        Patient::new(
            first.to_string(),
            last.to_string(),
            NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            crate::domain::patient::Gender::Male,
            None,
            Some("1234567890".to_string()),
            None,
            None,
            None,
            None,
            None,
            crate::domain::patient::Address::default(),
            None,
            Some("0412345678".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_patient_list_empty() {
        let theme = Theme::dark();
        let mut list = PatientList::new(theme);
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
        assert_eq!(list.selected_patient().unwrap().first_name, "John");

        list.move_down();
        assert_eq!(list.selected_patient().unwrap().first_name, "Jane");

        list.move_up();
        assert_eq!(list.selected_patient().unwrap().first_name, "John");
    }
}
