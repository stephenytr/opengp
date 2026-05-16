//! Patient List Component
//!
//! Wraps UnifiedList<PatientListItem> to provide a consistent list interface
//! with search, pagination, and mouse handling.

use crossterm::event::{MouseEvent, KeyEvent};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use uuid::Uuid;

use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};

/// Wrapper around UnifiedList<PatientListItem> providing a consistent API
/// for patient list operations including search, navigation, and mouse handling.
pub struct PatientList {
    inner: UnifiedList<PatientListItem>,
}

#[derive(Debug, Clone)]
pub enum PatientListAction {
    SelectionChanged,
    OpenPatient(Uuid),
    FocusSearch,
    SearchChanged,
    ContextMenu { x: u16, y: u16, patient_id: Uuid },
}

impl PatientList {
    /// Create a new patient list with the given theme
    pub fn new(theme: Theme) -> Self {
        let config = UnifiedListConfig::new("Patients", 1, "No patients found.")
            .with_search("Search patients...");

        let inner = UnifiedList::new(
            Vec::new(),
            columns(),
            theme,
            config,
        );

        Self { inner }
    }

    /// Set the list of patients to display
    pub fn set_patients(&mut self, patients: Vec<PatientListItem>) {
        let previously_selected_id = self.selected_patient_id();
        self.inner.items = patients;
        self.inner.selected_index = 0;
        self.inner.scroll_offset = 0;

        // Try to restore previously selected patient
        if let Some(id) = previously_selected_id {
            if let Some(pos) = self.inner.items.iter().position(|p| p.id == id) {
                self.inner.selected_index = pos;
            }
        }
    }

    /// Get the currently selected patient, if any
    pub fn selected_patient(&self) -> Option<&PatientListItem> {
        self.inner.items.get(self.inner.selected_index)
    }

    /// Get the ID of the currently selected patient, if any
    pub fn selected_patient_id(&self) -> Option<Uuid> {
        self.selected_patient().map(|p| p.id)
    }

    /// Get a patient by ID from the full list
    pub fn get_patient_by_id(&self, id: Uuid) -> Option<&PatientListItem> {
        self.inner.items.iter().find(|patient| patient.id == id)
    }

    /// Check if the list is currently loading
    pub fn is_loading(&self) -> bool {
        self.inner.loading
    }

    /// Set the loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.inner.loading = loading;
    }

    /// Check if search mode is active
    pub fn is_searching(&self) -> bool {
        self.inner.searching
    }

    /// Get the current search query
    pub fn search_query(&self) -> &str {
        &self.inner.search_query
    }

    /// Set the search query and apply filtering
    pub fn set_search_query(&mut self, query: String) {
        self.inner.search_query = query;
        self.inner.apply_search_filter();
    }

    /// Reset search mode and clear the query
    pub fn reset_search(&mut self) {
        self.inner.searching = false;
        self.inner.search_query.clear();
        self.inner.apply_search_filter();
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.inner.selected_index > 0 {
            self.inner.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let max = self.inner.items.len().saturating_sub(1);
        if self.inner.selected_index < max {
            self.inner.selected_index += 1;
        }
    }

    /// Move to first item
    pub fn move_first(&mut self) {
        self.inner.selected_index = 0;
    }

    /// Move to last item
    pub fn move_last(&mut self) {
        self.inner.selected_index = self.inner.items.len().saturating_sub(1);
    }

    /// Move by offset
    pub fn move_by(&mut self, offset: isize) {
        let new_index = (self.inner.selected_index as isize + offset).max(0) as usize;
        self.inner.selected_index = new_index.min(self.inner.items.len().saturating_sub(1));
    }

    /// Adjust scroll to keep selection visible
    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.inner.selected_index < self.inner.scroll_offset {
            self.inner.scroll_offset = self.inner.selected_index;
        } else if self.inner.selected_index >= self.inner.scroll_offset + visible_rows {
            self.inner.scroll_offset = self.inner.selected_index.saturating_sub(visible_rows - 1);
        }
    }

    /// Move up and adjust scroll
    pub fn move_up_and_scroll(&mut self, visible_rows: usize) {
        self.move_up();
        self.adjust_scroll(visible_rows);
    }

    /// Move down and adjust scroll
    pub fn move_down_and_scroll(&mut self, visible_rows: usize) {
        self.move_down();
        self.adjust_scroll(visible_rows);
    }

    /// Move to first and adjust scroll
    pub fn move_first_and_scroll(&mut self, _visible_rows: usize) {
        self.move_first();
        self.inner.scroll_offset = 0;
    }

    /// Move to last and adjust scroll
    pub fn move_last_and_scroll(&mut self, visible_rows: usize) {
        self.move_last();
        if visible_rows > 0 {
            self.inner.scroll_offset = self.inner.items.len().saturating_sub(visible_rows);
        }
    }

    /// Move by offset and adjust scroll
    pub fn move_by_and_scroll(&mut self, offset: isize, visible_rows: usize) {
        self.move_by(offset);
        self.adjust_scroll(visible_rows);
    }

    /// Check if there is a selection
    pub fn has_selection(&self) -> bool {
        !self.inner.items.is_empty()
    }

    /// Get the count of filtered items
    pub fn filtered_count(&self) -> usize {
        self.inner.items.len()
    }

    /// Get the total count of items
    pub fn total_count(&self) -> usize {
        self.inner.items.len()
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.inner.scroll_offset
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PatientListAction> {
        self.inner.handle_key(key).and_then(|action| {
            Some(match action {
                UnifiedListAction::Select(_) => PatientListAction::SelectionChanged,
                UnifiedListAction::Open(item) => PatientListAction::OpenPatient(item.id),
                UnifiedListAction::ContextMenu { index, x, y } => {
                    if let Some(patient) = self.inner.items.get(index) {
                        PatientListAction::ContextMenu {
                            x,
                            y,
                            patient_id: patient.id,
                        }
                    } else {
                        PatientListAction::SelectionChanged
                    }
                }
                UnifiedListAction::New
                | UnifiedListAction::Edit(_)
                | UnifiedListAction::Delete(_)
                | UnifiedListAction::ToggleInactive => PatientListAction::SelectionChanged,
            })
        })
    }

    /// Handle a mouse event
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientListAction> {
        self.inner.handle_mouse(mouse, area).and_then(|action| {
            Some(match action {
                UnifiedListAction::Select(_) => PatientListAction::SelectionChanged,
                UnifiedListAction::Open(item) => PatientListAction::OpenPatient(item.id),
                UnifiedListAction::ContextMenu { index, x, y } => {
                    if let Some(patient) = self.inner.items.get(index) {
                        PatientListAction::ContextMenu {
                            x,
                            y,
                            patient_id: patient.id,
                        }
                    } else {
                        PatientListAction::SelectionChanged
                    }
                }
                UnifiedListAction::New
                | UnifiedListAction::Edit(_)
                | UnifiedListAction::Delete(_)
                | UnifiedListAction::ToggleInactive => PatientListAction::SelectionChanged,
            })
        })
    }

    /// Check if the list is focused
    pub fn is_focused(&self) -> bool {
        self.inner.focus.is_focused()
    }
}

impl Widget for PatientList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.inner.render(area, buf);
    }
}

impl HasFocus for PatientList {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.inner.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

/// Define columns for the patient list
fn columns() -> Vec<UnifiedColumnDef<PatientListItem>> {
    vec![
        UnifiedColumnDef::new("Name", 20, |p: &PatientListItem| p.full_name.clone()),
        UnifiedColumnDef::new("DOB", 8, |p: &PatientListItem| {
            p.date_of_birth.format("%d/%m/%y").to_string()
        }),
        UnifiedColumnDef::new("Medicare #", 11, |p: &PatientListItem| {
            p.medicare_number
                .as_deref()
                .unwrap_or("-")
                .to_string()
        }),
        UnifiedColumnDef::new("Phone", 12, |p: &PatientListItem| {
            p.phone_mobile.as_deref().unwrap_or("-").to_string()
        }),
        UnifiedColumnDef::new("Last Visit", 9, |_p: &PatientListItem| "-".to_string()),
    ]
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
    fn test_patient_list_set_patients() {
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

        assert!(list.has_selection());

        list.move_first();
        assert!(list.selected_patient().unwrap().full_name.contains("John"));

        list.move_down();
        assert!(list.selected_patient().unwrap().full_name.contains("Jane"));

        list.move_up();
        assert!(list.selected_patient().unwrap().full_name.contains("John"));
    }
}
