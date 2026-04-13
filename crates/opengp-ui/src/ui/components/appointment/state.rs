use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

use opengp_config::CalendarConfig;
use opengp_domain::domain::appointment::{AppointmentType, CalendarDayView};
use opengp_domain::domain::user::Practitioner;

use crate::ui::widgets::LoadingState;

use super::calendar::Calendar;
use super::schedule::Schedule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentView {
    Calendar,
    Schedule,
}

#[derive(Debug, Clone)]
pub struct AppointmentState {
    pub current_view: AppointmentView,
    pub calendar: Calendar,
    pub schedule: Schedule,
    pub selected_date: Option<NaiveDate>,
    pub schedule_data: Option<CalendarDayView>,
    pub practitioners: Vec<Practitioner>,
    pub selected_practitioner: Option<Uuid>,
    pub selected_appointment: Option<Uuid>,
    pub loading_state: LoadingState,
    loading: bool,
    pub hidden_columns: Vec<Uuid>,
}

impl AppointmentState {
    pub fn new(theme: crate::ui::theme::Theme, config: CalendarConfig) -> Self {
        let abbreviations = opengp_config::load_appointment_config()
            .map(|appointment_config| {
                let mut map = HashMap::new();
                for apt_type in [
                    AppointmentType::Standard,
                    AppointmentType::Long,
                    AppointmentType::Brief,
                    AppointmentType::NewPatient,
                    AppointmentType::HealthAssessment,
                    AppointmentType::ChronicDiseaseReview,
                    AppointmentType::MentalHealthPlan,
                    AppointmentType::Immunisation,
                    AppointmentType::Procedure,
                    AppointmentType::Telephone,
                    AppointmentType::Telehealth,
                    AppointmentType::HomeVisit,
                    AppointmentType::Emergency,
                ] {
                    if let Some(option) = appointment_config
                        .types
                        .get(Self::appointment_type_config_key(apt_type))
                        .filter(|option| option.enabled)
                    {
                        map.insert(apt_type.to_string(), option.abbreviation.clone());
                    }
                }
                map
            })
            .unwrap_or_default();

        Self {
            current_view: AppointmentView::Schedule,
            calendar: Calendar::new(theme.clone()),
            schedule: Schedule::new(theme, config).with_abbreviations(abbreviations),
            selected_date: Some(chrono::Utc::now().date_naive()),
            schedule_data: None,
            practitioners: Vec::new(),
            selected_practitioner: None,
            selected_appointment: None,
            loading_state: LoadingState::new().message("Loading appointments..."),
            loading: false,
            hidden_columns: Vec::new(),
        }
    }

    fn appointment_type_config_key(apt_type: AppointmentType) -> &'static str {
        match apt_type {
            AppointmentType::Standard => "standard",
            AppointmentType::Long => "long",
            AppointmentType::Brief => "brief",
            AppointmentType::NewPatient => "new_patient",
            AppointmentType::HealthAssessment => "health_assessment",
            AppointmentType::ChronicDiseaseReview => "chronic_disease_review",
            AppointmentType::MentalHealthPlan => "mental_health_plan",
            AppointmentType::Immunisation => "immunisation",
            AppointmentType::Procedure => "procedure",
            AppointmentType::Telephone => "telephone",
            AppointmentType::Telehealth => "telehealth",
            AppointmentType::HomeVisit => "home_visit",
            AppointmentType::Emergency => "emergency",
        }
    }

    /// Set the selected date
    pub fn set_selected_date(&mut self, date: Option<NaiveDate>) {
        self.selected_date = date;
    }

    /// Get the currently selected date
    pub fn selected_date(&self) -> Option<NaiveDate> {
        self.selected_date
    }

    /// Switch to a different view
    pub fn set_view(&mut self, view: AppointmentView) {
        self.current_view = view;
    }

    /// Check if loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set the selected practitioner
    pub fn set_selected_practitioner(&mut self, practitioner_id: Option<Uuid>) {
        self.selected_practitioner = practitioner_id;
    }

    /// Get the selected practitioner ID
    pub fn selected_practitioner(&self) -> Option<Uuid> {
        self.selected_practitioner
    }

    /// Set the selected appointment
    pub fn set_selected_appointment(&mut self, appointment_id: Option<Uuid>) {
        self.selected_appointment = appointment_id;
    }

    /// Get the selected appointment ID
    pub fn selected_appointment(&self) -> Option<Uuid> {
        self.selected_appointment
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.selected_date = None;
        self.selected_practitioner = None;
        self.selected_appointment = None;
        self.schedule_data = None;
        self.hidden_columns = Vec::new();
    }

    /// Toggle visibility of a practitioner column
    /// If hiding would leave 0 visible columns, this is a no-op (minimum 1 visible enforced)
    pub fn toggle_column(&mut self, practitioner_id: Uuid, total_practitioners: usize) {
        if let Some(pos) = self
            .hidden_columns
            .iter()
            .position(|&id| id == practitioner_id)
        {
            // Column is hidden, show it
            self.hidden_columns.remove(pos);
        } else {
            // Column is visible, hide it (but only if at least 1 will remain visible)
            if self.hidden_columns.len() < total_practitioners - 1 {
                self.hidden_columns.push(practitioner_id);
            }
        }
    }

    /// Check if a practitioner column is hidden
    pub fn is_column_hidden(&self, practitioner_id: Uuid) -> bool {
        self.hidden_columns.contains(&practitioner_id)
    }

    /// Get visible practitioners (excluding hidden columns)
    pub fn visible_practitioners(&self) -> Vec<&Practitioner> {
        self.practitioners
            .iter()
            .filter(|p| !self.is_column_hidden(p.id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use opengp_config::CalendarConfig;

    fn create_test_state() -> AppointmentState {
        AppointmentState::new(Theme::dark(), CalendarConfig::default())
    }

    #[test]
    fn test_appointment_state_construction() {
        let state = create_test_state();

        assert_eq!(state.current_view, AppointmentView::Schedule);
        assert!(state.selected_date.is_some());
        assert!(state.schedule_data.is_none());
        assert!(state.practitioners.is_empty());
        assert!(state.selected_practitioner.is_none());
        assert!(state.selected_appointment.is_none());
        assert!(!state.is_loading());
    }

    #[test]
    fn test_appointment_state_initial_date_is_today() {
        let state = create_test_state();
        let today = chrono::Utc::now().date_naive();

        assert_eq!(state.selected_date(), Some(today));
    }

    #[test]
    fn test_view_switching_to_calendar() {
        let mut state = create_test_state();
        assert_eq!(state.current_view, AppointmentView::Schedule);

        state.set_view(AppointmentView::Calendar);
        assert_eq!(state.current_view, AppointmentView::Calendar);
    }

    #[test]
    fn test_view_switching_to_schedule() {
        let mut state = create_test_state();
        state.set_view(AppointmentView::Calendar);
        assert_eq!(state.current_view, AppointmentView::Calendar);

        state.set_view(AppointmentView::Schedule);
        assert_eq!(state.current_view, AppointmentView::Schedule);
    }

    #[test]
    fn test_date_selection_set_and_get() {
        let mut state = create_test_state();
        let test_date = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

        state.set_selected_date(Some(test_date));
        assert_eq!(state.selected_date(), Some(test_date));
    }

    #[test]
    fn test_date_selection_clear() {
        let mut state = create_test_state();
        assert!(state.selected_date().is_some());

        state.set_selected_date(None);
        assert_eq!(state.selected_date(), None);
    }

    #[test]
    fn test_practitioner_selection_lifecycle() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        assert!(state.selected_practitioner().is_none());

        state.set_selected_practitioner(Some(practitioner_id));
        assert_eq!(state.selected_practitioner(), Some(practitioner_id));

        state.set_selected_practitioner(None);
        assert!(state.selected_practitioner().is_none());
    }

    #[test]
    fn test_appointment_selection_lifecycle() {
        let mut state = create_test_state();
        let appointment_id = Uuid::new_v4();

        assert!(state.selected_appointment().is_none());

        state.set_selected_appointment(Some(appointment_id));
        assert_eq!(state.selected_appointment(), Some(appointment_id));

        state.set_selected_appointment(None);
        assert!(state.selected_appointment().is_none());
    }

    #[test]
    fn test_loading_state_management() {
        let mut state = create_test_state();
        assert!(!state.is_loading());

        state.set_loading(true);
        assert!(state.is_loading());

        state.set_loading(false);
        assert!(!state.is_loading());
    }

    #[test]
    fn test_clear_selections_resets_all_selections() {
        let mut state = create_test_state();
        let test_date = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let practitioner_id = Uuid::new_v4();
        let appointment_id = Uuid::new_v4();

        state.set_selected_date(Some(test_date));
        state.set_selected_practitioner(Some(practitioner_id));
        state.set_selected_appointment(Some(appointment_id));

        assert!(state.selected_date().is_some());
        assert!(state.selected_practitioner().is_some());
        assert!(state.selected_appointment().is_some());

        state.clear_selections();

        assert!(state.selected_date().is_none());
        assert!(state.selected_practitioner().is_none());
        assert!(state.selected_appointment().is_none());
        assert!(state.schedule_data.is_none());
    }

    #[test]
    fn test_toggle_column_hides_then_shows() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        // Initially visible (not hidden)
        assert!(!state.is_column_hidden(practitioner_id));

        // Toggle once → hidden
        state.toggle_column(practitioner_id, 3);
        assert!(state.is_column_hidden(practitioner_id));

        // Toggle again → visible
        state.toggle_column(practitioner_id, 3);
        assert!(!state.is_column_hidden(practitioner_id));
    }

    #[test]
    fn test_toggle_last_visible_column_is_noop() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        // Only 1 practitioner, hiding it should be a no-op
        state.toggle_column(practitioner_id, 1);
        assert!(!state.is_column_hidden(practitioner_id)); // Still visible
    }

    #[test]
    fn test_clear_selections_resets_hidden_columns() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        state.toggle_column(practitioner_id, 3);
        assert!(state.is_column_hidden(practitioner_id));

        state.clear_selections();
        assert!(!state.is_column_hidden(practitioner_id)); // Reset
    }

    #[test]
    fn test_visible_practitioners_excludes_hidden() {
        let mut state = create_test_state();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let p3 = Uuid::new_v4();

        use chrono::{DateTime, Utc};
        use opengp_domain::domain::user::Practitioner;

        let now = Utc::now();
        state.practitioners = vec![
            Practitioner {
                id: p1,
                user_id: None,
                first_name: "Alice".to_string(),
                middle_name: None,
                last_name: "Doctor".to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: "12345".to_string(),
                speciality: None,
                qualifications: vec![],
                phone: None,
                email: None,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Practitioner {
                id: p2,
                user_id: None,
                first_name: "Bob".to_string(),
                middle_name: None,
                last_name: "Doctor".to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: "12346".to_string(),
                speciality: None,
                qualifications: vec![],
                phone: None,
                email: None,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Practitioner {
                id: p3,
                user_id: None,
                first_name: "Carol".to_string(),
                middle_name: None,
                last_name: "Doctor".to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: "12347".to_string(),
                speciality: None,
                qualifications: vec![],
                phone: None,
                email: None,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
        ];

        state.toggle_column(p2, 3);

        let visible: Vec<_> = state.visible_practitioners();
        assert_eq!(visible.len(), 2);
        assert!(!visible.iter().any(|p| p.id == p2));
    }
}
