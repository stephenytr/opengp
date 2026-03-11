use chrono::NaiveDate;
use std::sync::Arc;

use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::ui::components::appointment::{
    AppointmentDetailModal, AppointmentForm, AppointmentState, AppointmentView,
};
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::help::HelpOverlay;
use crate::ui::components::patient::{PatientForm, PatientList, PatientState};
use crate::ui::components::status_bar::StatusBar;
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::keybinds::{KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;
use opengp_config::CalendarConfig;

mod event_handler;
mod keybinds;
mod renderer;
mod state;

const DEFAULT_PATIENT_PAGE_LIMIT: u32 = 100;
const DEFAULT_APPOINTMENT_PAGE_LIMIT: u32 = 100;
const DEFAULT_CONSULTATION_PAGE_LIMIT: u32 = 100;

type PatientListFetchTask =
    tokio::task::JoinHandle<Result<Vec<crate::ui::view_models::PatientListItem>, ApiTaskError>>;
type AppointmentListFetchTask =
    tokio::task::JoinHandle<
        Result<opengp_domain::domain::appointment::CalendarDayView, ApiTaskError>,
    >;
type ConsultationListFetchTask =
    tokio::task::JoinHandle<Result<Vec<opengp_domain::domain::clinical::Consultation>, ApiTaskError>>;
type LoginTask = tokio::task::JoinHandle<
    Result<opengp_domain::domain::api::LoginResponse, crate::api::ApiClientError>,
>;

pub(super) enum ApiTaskError {
    Unauthorized,
    ServerUnavailable(String),
    Message(String),
}

impl ApiTaskError {
    fn from_client_error(error: crate::api::ApiClientError, context: &str) -> Self {
        match error {
            crate::api::ApiClientError::Unauthorized => Self::Unauthorized,
            crate::api::ApiClientError::ServerUnavailable(message) => {
                Self::ServerUnavailable(format!("{}: {}", context, message))
            }
            other => Self::Message(format!("{}: {}", context, other)),
        }
    }

    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub struct App {
    theme: Theme,
    keybinds: &'static KeybindRegistry,
    tab_bar: TabBar,
    status_bar: StatusBar,
    help_overlay: HelpOverlay,
    login_screen: crate::ui::screens::LoginScreen,
    authenticated: bool,
    current_context: KeyContext,
    should_quit: bool,
    /// The authenticated user performing operations - used for audit logging
    pub current_user_id: uuid::Uuid,
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    patient_state: PatientState,
    patient_list: PatientList,
    patient_form: Option<PatientForm>,
    pending_patient_data: Option<PendingPatientData>,
    pending_edit_patient_id: Option<uuid::Uuid>,
    appointment_state: AppointmentState,
    #[allow(dead_code)]
    appointment_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
    #[allow(dead_code)]
    patient_service: Option<Arc<crate::ui::services::PatientUiService>>,
    pending_appointment_date: Option<NaiveDate>,
    pending_load_practitioners: bool,
    pending_load_booked_slots: Option<(uuid::Uuid, NaiveDate, u32)>,
    appointment_form: Option<AppointmentForm>,
    appointment_detail_modal: Option<AppointmentDetailModal>,
    pending_appointment_save: Option<opengp_domain::domain::appointment::NewAppointmentData>,
    pending_appointment_status_transition: Option<(uuid::Uuid, AppointmentStatusTransition)>,
    pending_clinical_patient_id: Option<uuid::Uuid>,
    pending_clinical_save_data: Option<PendingClinicalSaveData>,
    clinical_state: ClinicalState,
    #[allow(dead_code)]
    clinical_service: Option<Arc<crate::ui::services::ClinicalUiService>>,
    api_client: Option<Arc<crate::api::ApiClient>>,
    patient_page_limit: u32,
    appointment_page_limit: u32,
    consultation_page_limit: u32,
    pending_patient_list_refresh: bool,
    pending_appointment_list_refresh: Option<NaiveDate>,
    pending_consultation_list_refresh: Option<uuid::Uuid>,
    patient_list_fetch_task: Option<PatientListFetchTask>,
    appointment_list_fetch_task: Option<AppointmentListFetchTask>,
    consultation_list_fetch_task: Option<ConsultationListFetchTask>,
    pending_login_request: Option<(String, String)>,
    login_task: Option<LoginTask>,
    server_unavailable_error: Option<String>,
    server_unavailable_retry: Option<RetryOperation>,
    active_login_attempt: Option<(String, String)>,
    active_appointment_refresh_date: Option<NaiveDate>,
    active_consultation_refresh_patient_id: Option<uuid::Uuid>,
    terminal_size: Rect,
}

pub enum PendingPatientData {
    New(opengp_domain::domain::patient::NewPatientData),
    Update {
        id: uuid::Uuid,
        data: opengp_domain::domain::patient::UpdatePatientData,
    },
}

pub enum PendingClinicalSaveData {
    Allergy {
        patient_id: uuid::Uuid,
        allergy: opengp_domain::domain::clinical::Allergy,
    },
    MedicalHistory {
        patient_id: uuid::Uuid,
        history: opengp_domain::domain::clinical::MedicalHistory,
    },
    VitalSigns {
        patient_id: uuid::Uuid,
        vitals: opengp_domain::domain::clinical::VitalSigns,
    },
    FamilyHistory {
        patient_id: uuid::Uuid,
        entry: opengp_domain::domain::clinical::FamilyHistory,
    },
    Consultation {
        patient_id: uuid::Uuid,
        practitioner_id: uuid::Uuid,
        appointment_id: Option<uuid::Uuid>,
        reason: Option<String>,
        clinical_notes: Option<String>,
    },
    SocialHistory {
        patient_id: uuid::Uuid,
        history: opengp_domain::domain::clinical::SocialHistory,
    },
}

#[derive(Debug)]
pub enum AppointmentStatusTransition {
    MarkArrived,
    MarkInProgress,
    MarkCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryOperation {
    Login { username: String, password: String },
    RefreshPatients,
    RefreshAppointments { date: NaiveDate },
    RefreshConsultations { patient_id: uuid::Uuid },
}

impl App {
    pub fn new(
        api_client: Option<Arc<crate::api::ApiClient>>,
        appointment_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
        patient_service: Option<Arc<crate::ui::services::PatientUiService>>,
        clinical_service: Option<Arc<crate::ui::services::ClinicalUiService>>,
        calendar_config: CalendarConfig,
    ) -> Self {
        let theme = Theme::dark();
        let mut app = Self {
            theme: theme.clone(),
            keybinds: KeybindRegistry::global(),
            tab_bar: TabBar::new(theme.clone()),
            status_bar: StatusBar::patient_list(theme.clone()),
            help_overlay: HelpOverlay::new(theme.clone()),
            login_screen: crate::ui::screens::LoginScreen::new(theme.clone()),
            authenticated: true,
            current_context: KeyContext::Global,
            should_quit: false,
            current_user_id: uuid::Uuid::nil(),
            title: "OpenGP".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            patient_state: PatientState::new(),
            patient_list: PatientList::new(theme.clone()),
            patient_form: None,
            pending_patient_data: None,
            pending_edit_patient_id: None,
            appointment_state: AppointmentState::new(theme.clone(), calendar_config),
            appointment_service,
            patient_service,
            pending_appointment_date: None,
            pending_load_practitioners: false,
            pending_load_booked_slots: None,
            appointment_form: None,
            appointment_detail_modal: None,
            pending_appointment_save: None,
            pending_appointment_status_transition: None,
            pending_clinical_patient_id: None,
            pending_clinical_save_data: None,
            clinical_state: ClinicalState::with_theme(theme.clone()),
            clinical_service,
            api_client,
            patient_page_limit: DEFAULT_PATIENT_PAGE_LIMIT,
            appointment_page_limit: DEFAULT_APPOINTMENT_PAGE_LIMIT,
            consultation_page_limit: DEFAULT_CONSULTATION_PAGE_LIMIT,
            pending_patient_list_refresh: false,
            pending_appointment_list_refresh: None,
            pending_consultation_list_refresh: None,
            patient_list_fetch_task: None,
            appointment_list_fetch_task: None,
            consultation_list_fetch_task: None,
            pending_login_request: None,
            login_task: None,
            server_unavailable_error: None,
            server_unavailable_retry: None,
            active_login_attempt: None,
            active_appointment_refresh_date: None,
            active_consultation_refresh_patient_id: None,
            terminal_size: Rect::new(0, 0, 80, 24),
        };

        app.refresh_status_bar();
        app.refresh_context();

        app
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn keybinds(&self) -> &KeybindRegistry {
        &self.keybinds
    }

    pub fn current_tab(&self) -> Tab {
        self.tab_bar.selected()
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn set_authenticated(&mut self, authenticated: bool) {
        self.authenticated = authenticated;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    #[cfg(test)]
    pub fn appointment_state(&self) -> &AppointmentState {
        &self.appointment_state
    }

    pub fn practitioners(&self) -> &[opengp_domain::domain::user::Practitioner] {
        &self.appointment_state.practitioners
    }

    pub fn has_appointment_form(&self) -> bool {
        self.appointment_form.is_some()
    }

    pub fn toggle_theme(&mut self) {
        if self.theme.colors.background == Color::Black {
            self.theme = Theme::light();
        } else {
            self.theme = Theme::dark();
        }
    }

    fn refresh_status_bar(&mut self) {
        self.status_bar = match self.tab_bar.selected() {
            Tab::Patient => StatusBar::patient_list(self.theme.clone()),
            Tab::Appointment => StatusBar::schedule(self.theme.clone()),
            Tab::Clinical => StatusBar::clinical(self.theme.clone()),
            Tab::Billing => StatusBar::billing(self.theme.clone()),
        };
    }

    fn refresh_context(&mut self) {
        self.current_context = match self.tab_bar.selected() {
            Tab::Patient => KeyContext::PatientList,
            Tab::Appointment => {
                if self.appointment_form.is_some() || self.appointment_detail_modal.is_some() {
                    KeyContext::Schedule
                } else {
                    match self.appointment_state.current_view {
                        AppointmentView::Calendar => KeyContext::Calendar,
                        AppointmentView::Schedule => KeyContext::Schedule,
                    }
                }
            }
            Tab::Clinical => KeyContext::Clinical,
            Tab::Billing => KeyContext::Billing,
        };
        self.help_overlay.set_context(self.current_context);
    }

    fn calculate_visible_patient_rows(&self) -> usize {
        let available_height = self.terminal_size.height.saturating_sub(2 + 2 + 1);
        available_height.saturating_sub(1) as usize
    }

    pub(crate) fn show_server_unavailable_error(
        &mut self,
        message: impl Into<String>,
        retry_operation: RetryOperation,
    ) {
        self.server_unavailable_error = Some(message.into());
        self.server_unavailable_retry = Some(retry_operation);
    }

    pub(crate) fn clear_server_unavailable_error(&mut self) {
        self.server_unavailable_error = None;
        self.server_unavailable_retry = None;
    }

    pub(crate) fn retry_server_unavailable_operation(&mut self) {
        if let Some(operation) = self.server_unavailable_retry.clone() {
            match operation {
                RetryOperation::Login { username, password } => {
                    self.pending_login_request = Some((username, password));
                }
                RetryOperation::RefreshPatients => {
                    self.request_refresh_patients();
                }
                RetryOperation::RefreshAppointments { date } => {
                    self.request_refresh_appointments(date);
                }
                RetryOperation::RefreshConsultations { patient_id } => {
                    self.request_refresh_consultations(patient_id);
                }
            }
            self.clear_server_unavailable_error();
        }
    }

    #[cfg(test)]
    pub fn server_unavailable_error_message(&self) -> Option<&str> {
        self.server_unavailable_error.as_deref()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(None, None, None, None, CalendarConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new(None, None, None, None, CalendarConfig::default());
        assert_eq!(app.current_tab(), Tab::Patient);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_tab_switching() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        assert_eq!(app.current_tab(), Tab::Appointment);
    }

    #[test]
    fn test_help_toggle() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());

        assert!(!app.help_overlay.is_visible());

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(1),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        assert!(app.help_overlay.is_visible());

        app.handle_key_event(key);

        assert!(!app.help_overlay.is_visible());
    }

    #[test]
    fn test_quit() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);

        assert!(app.should_quit());
    }

    #[test]
    fn test_calendar_keybind_routing() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert_eq!(app.current_tab(), Tab::Appointment);

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        let initial_date = app.appointment_state().calendar.focused_date;
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('l'),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert!(
            app.appointment_state().calendar.focused_date > initial_date,
            "Calendar focused_date should advance after pressing 'l'"
        );
    }

    #[test]
    fn test_calendar_enter_selects_date() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        use crate::ui::components::appointment::AppointmentView;
        assert_eq!(
            app.appointment_state().current_view,
            AppointmentView::Schedule,
            "Pressing Enter in Calendar should switch to Schedule view"
        );
    }

    #[test]
    fn test_schedule_keybind_routing() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        use crate::ui::components::appointment::AppointmentView;
        assert_eq!(
            app.appointment_state().current_view,
            AppointmentView::Schedule
        );

        let initial_slot = app.appointment_state().schedule.selected_time_slot;
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert!(
            app.appointment_state().schedule.selected_time_slot >= initial_slot,
            "Schedule time slot should advance after pressing 'j'"
        );
    }

    #[test]
    fn test_q_does_not_quit_on_appointment() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert_eq!(app.current_tab(), Tab::Appointment);

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert!(
            !app.should_quit(),
            "Bare 'q' should NOT quit the app when on Appointment tab"
        );
    }

    #[test]
    fn test_ctrl_q_always_quits() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);
        assert!(app.should_quit(), "Ctrl+Q should always quit the app");
    }

    #[test]
    fn test_schedule_escape_returns_to_calendar() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        use crate::ui::components::appointment::AppointmentView;
        assert_eq!(
            app.appointment_state().current_view,
            AppointmentView::Schedule
        );

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert_eq!(
            app.appointment_state().current_view,
            AppointmentView::Calendar,
            "Escape in Schedule should return to Calendar view"
        );
    }

    #[test]
    fn test_patient_keybind_regression() {
        let mut app = App::new(None, None, None, None, CalendarConfig::default());
        assert_eq!(app.current_tab(), Tab::Patient);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert!(
            app.should_quit(),
            "Bare 'q' should still quit from Patient tab"
        );
    }
}
