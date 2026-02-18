//! Patient Component State
//!
//! State management for patient list and form components.

use crate::domain::patient::Patient;

/// View mode for patient tab
#[derive(Debug, Clone, Default)]
pub enum PatientView {
    /// Showing patient list
    #[default]
    List,
    /// Showing new patient form
    NewForm,
    /// Showing edit patient form
    EditForm(Patient),
}

/// Patient component state
#[derive(Debug, Clone, Default)]
pub struct PatientState {
    /// Current view mode
    pub view: PatientView,
    /// Current page (for pagination)
    pub page: usize,
    /// Page size based on terminal height
    pub page_size: usize,
    /// Whether data is loading
    pub loading: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl PatientState {
    /// Create new patient state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page size based on terminal height
    pub fn set_page_size(&mut self, height: u16) {
        // Subtract for header/status bar/ borders
        self.page_size = height.saturating_sub(6) as usize;
        if self.page_size < 5 {
            self.page_size = 5;
        }
    }

    /// Check if showing list
    pub fn is_list_view(&self) -> bool {
        matches!(self.view, PatientView::List)
    }

    /// Check if showing form
    pub fn is_form_view(&self) -> bool {
        matches!(self.view, PatientView::NewForm | PatientView::EditForm(_))
    }

    /// Switch to list view
    pub fn show_list(&mut self) {
        self.view = PatientView::List;
    }

    /// Switch to new form
    pub fn show_new_form(&mut self) {
        self.view = PatientView::NewForm;
    }

    /// Switch to edit form
    pub fn show_edit_form(&mut self, patient: Patient) {
        self.view = PatientView::EditForm(patient);
    }

    /// Go to next page
    pub fn next_page(&mut self) {
        self.page += 1;
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        self.page = self.page.saturating_sub(1);
    }

    /// Go to specific page
    pub fn go_to_page(&mut self, page: usize) {
        self.page = page;
    }

    /// Calculate total pages
    pub fn total_pages(&self, total_items: usize) -> usize {
        if total_items == 0 {
            return 1;
        }
        (total_items + self.page_size - 1) / self.page_size
    }

    /// Get page offset
    pub fn page_offset(&self) -> usize {
        self.page * self.page_size
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set error
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_state_default() {
        let state = PatientState::new();
        assert!(state.is_list_view());
        assert!(!state.is_form_view());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_patient_state_page_size() {
        let mut state = PatientState::new();
        // height - 6 = page_size (with minimum of 5)
        state.set_page_size(24);
        assert_eq!(state.page_size, 18); // 24 - 6

        state.set_page_size(10);
        assert_eq!(state.page_size, 5); // 10 - 6 = 4, clamped to minimum 5
    }

    #[test]
    fn test_patient_state_pagination() {
        let mut state = PatientState::new();
        state.set_page_size(10);

        assert_eq!(state.total_pages(0), 1);
        // With page_size = 5 (minimum from 10 - 6), total_pages(10) = (10 + 5 - 1) / 5 = 14 / 5 = 2
        assert_eq!(state.total_pages(10), 2);
        assert_eq!(state.total_pages(11), 3);
        assert_eq!(state.total_pages(25), 5);

        assert_eq!(state.page_offset(), 0);

        // With page_size = 5 (minimum from 10 - 6)
        state.next_page();
        assert_eq!(state.page_offset(), 5);

        state.prev_page();
        assert_eq!(state.page_offset(), 0);

        state.go_to_page(5);
        assert_eq!(state.page_offset(), 25);
    }
}
