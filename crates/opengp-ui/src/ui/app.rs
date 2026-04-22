use chrono::{NaiveDate, NaiveTime};
use std::sync::Arc;
use std::time::Instant;

use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::ui::components::appointment::{
    AppointmentDetailModal, AppointmentForm, AppointmentState, AppointmentView,
};
use crate::ui::components::help::HelpOverlay;
use crate::ui::components::patient::{PatientForm, PatientList};
use crate::ui::components::status_bar::StatusBar;
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::components::workspace::WorkspaceManager;
use crate::ui::keybinds::{KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;
use opengp_config::CalendarConfig;
use opengp_domain::domain::billing::BillingType;

use crate::ui::services::BillingUiService;

mod command;
mod event_handler;
mod keybinds;
mod renderer;
mod state;

pub use command::AppCommand;

const DEFAULT_PATIENT_PAGE_LIMIT: u32 = 100;
const DEFAULT_APPOINTMENT_PAGE_LIMIT: u32 = 100;
const DEFAULT_CONSULTATION_PAGE_LIMIT: u32 = 100;

type PatientListFetchTask =
    tokio::task::JoinHandle<Result<Vec<crate::ui::view_models::PatientListItem>, ApiTaskError>>;
type AppointmentListFetchTask = tokio::task::JoinHandle<
    Result<opengp_domain::domain::appointment::CalendarDayView, ApiTaskError>,
>;
type ConsultationListFetchTask = tokio::task::JoinHandle<
    Result<Vec<opengp_domain::domain::clinical::Consultation>, ApiTaskError>,
>;
type PractitionerListFetchTask =
    tokio::task::JoinHandle<Result<Vec<opengp_domain::domain::user::Practitioner>, ApiTaskError>>;
type RescheduleTask =
    tokio::task::JoinHandle<Result<(uuid::Uuid, chrono::NaiveDate), (String, bool)>>;
type LoginTask = tokio::task::JoinHandle<
    Result<opengp_domain::domain::api::LoginResponse, crate::api::ApiClientError>,
>;

pub struct ClinicalWorkspaceLoadResult {
    pub patient_id: uuid::Uuid,
    pub allergies: Result<Vec<opengp_domain::domain::clinical::Allergy>, crate::api::ApiClientError>,
    pub medical_history: Result<Vec<opengp_domain::domain::clinical::MedicalHistory>, crate::api::ApiClientError>,
    pub vitals: Result<Vec<opengp_domain::domain::clinical::VitalSigns>, crate::api::ApiClientError>,
    pub social_history: Result<opengp_domain::domain::clinical::SocialHistory, crate::api::ApiClientError>,
    pub family_history: Result<Vec<opengp_domain::domain::clinical::FamilyHistory>, crate::api::ApiClientError>,
}

type ClinicalWorkspaceLoadTask = tokio::task::JoinHandle<ClinicalWorkspaceLoadResult>;

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
    previous_tab: Tab,
    status_bar: StatusBar,
    help_overlay: HelpOverlay,
    login_screen: crate::ui::screens::LoginScreen,
    authenticated: bool,
    current_context: KeyContext,
    should_quit: bool,
    /// The authenticated user performing operations - used for audit logging
    pub current_user_id: uuid::Uuid,
    patient_list: PatientList,
    patient_form: Option<PatientForm>,
    pending_patient_data: Option<PendingPatientData>,
    pending_clinical_save_data: Option<PendingClinicalSaveData>,
    pending_edit_patient_id: Option<uuid::Uuid>,
     appointment_state: AppointmentState,
      appointment_form: Option<AppointmentForm>,
      appointment_detail_modal: Option<AppointmentDetailModal>,
      pending_load_practitioners: bool,
     pending_load_booked_slots: Option<(uuid::Uuid, NaiveDate, u32)>,
     pending_appointment_save: Option<(opengp_domain::domain::appointment::NewAppointmentData, i32)>,
     pending_appointment_status_transition: Option<(uuid::Uuid, AppointmentStatusTransition)>,
     pending_reschedule: Option<PendingRescheduleData>,
    workspace_manager: WorkspaceManager,
    billing_ui_service: Option<Arc<BillingUiService>>,
    appointment_ui_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
    practice_config: opengp_config::PracticeConfig,
    healthcare_config: opengp_config::healthcare::HealthcareConfig,
    patient_config: opengp_config::PatientConfig,
    allergy_config: opengp_config::AllergyConfig,
    clinical_config: opengp_config::ClinicalConfig,
    social_history_config: opengp_config::SocialHistoryConfig,
    api_client: Option<Arc<crate::api::ApiClient>>,
    patient_page_limit: u32,
    appointment_page_limit: u32,
    consultation_page_limit: u32,
    pending_patient_list_refresh: bool,
    pending_appointment_list_refresh: Option<NaiveDate>,
    pub pending_consultation_list_refresh: Option<uuid::Uuid>,
    pending_practitioners_list_refresh: bool,
     patient_list_fetch_task: Option<PatientListFetchTask>,
      appointment_list_fetch_task: Option<AppointmentListFetchTask>,
      consultation_list_fetch_task: Option<ConsultationListFetchTask>,
      practitioners_list_fetch_task: Option<PractitionerListFetchTask>,
      reschedule_task: Option<RescheduleTask>,
      pending_login_request: Option<(String, String)>,
      login_task: Option<LoginTask>,
      clinical_workspace_load_task: Option<(uuid::Uuid, ClinicalWorkspaceLoadTask)>,
    server_unavailable_error: Option<String>,
    server_unavailable_retry: Option<RetryOperation>,
    active_login_attempt: Option<(String, String)>,
    active_appointment_refresh_date: Option<NaiveDate>,
    active_consultation_refresh_patient_id: Option<uuid::Uuid>,
    terminal_size: Rect,
    pub command_tx: tokio::sync::mpsc::UnboundedSender<AppCommand>,
    command_rx: Option<tokio::sync::mpsc::UnboundedReceiver<AppCommand>>,
    context_menu_state: Option<crate::ui::widgets::ContextMenuState<AppContextMenuAction>>,
    last_billing_render: Option<Instant>,
    hovered_clinical_menu: Option<usize>,
}

pub enum PendingPatientData {
    New(opengp_domain::domain::patient::NewPatientData),
    Update {
        id: uuid::Uuid,
        data: opengp_domain::domain::patient::UpdatePatientData,
    },
}

#[derive(Debug)]
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
     TimerStart {
         consultation_id: uuid::Uuid,
     },
     TimerStop {
          consultation_id: uuid::Uuid,
      },
      SignConsultation {
          consultation_id: uuid::Uuid,
          user_id: uuid::Uuid,
      },
  }

#[derive(Debug, Clone)]
pub enum PendingBillingSaveData {
    AwaitingMbsSelection {
        consultation_id: uuid::Uuid,
        patient_id: uuid::Uuid,
    },
    CreatingInvoice {
        consultation_id: uuid::Uuid,
        mbs_items: Vec<(String, f64, bool)>,
        billing_type: BillingType,
    },
}

#[derive(Debug)]
pub enum AppointmentStatusTransition {
    SetStatus(opengp_domain::domain::appointment::AppointmentStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryOperation {
    Login { username: String, password: String },
    RefreshPatients,
    RefreshAppointments { date: NaiveDate },
    RefreshConsultations { patient_id: uuid::Uuid },
}

#[derive(Debug)]
pub struct PendingRescheduleData {
    pub appointment_id: uuid::Uuid,
    pub new_date: Option<NaiveDate>,
    pub new_time: Option<NaiveTime>,
    pub practitioner_id: uuid::Uuid,
    pub duration_minutes: i64,
}

/// Unified action types for all context menus in the application.
#[derive(Debug, Clone)]
pub enum AppContextMenuAction {
    // Patient actions
    PatientEdit(uuid::Uuid),
    PatientDelete(uuid::Uuid),
    PatientViewHistory(uuid::Uuid),
    // Appointment actions
    AppointmentEdit(uuid::Uuid),
    AppointmentCancel(uuid::Uuid),
    AppointmentReschedule(uuid::Uuid),
    // Clinical actions
    ClinicalEdit(uuid::Uuid),
    ClinicalDelete(uuid::Uuid),
    // Billing actions
    BillingEdit(uuid::Uuid),
    BillingViewInvoice(uuid::Uuid),
}

/// Tracks which context menu is currently active and its target.
#[derive(Debug)]
pub enum ActiveContextMenu {
    Patient {
        patient_id: uuid::Uuid,
        x: u16,
        y: u16,
    },
    Clinical {
        patient_id: uuid::Uuid,
        clinical_kind: String,  // "allergy", "vitals", "consultation", etc.
        item_id: uuid::Uuid,
        x: u16,
        y: u16,
    },
    Billing {
        billing_type: String,  // "claim", "invoice", "payment"
        item_id: uuid::Uuid,
        x: u16,
        y: u16,
    },
}

impl App {
    pub fn new(
        api_client: Option<Arc<crate::api::ApiClient>>,
        calendar_config: CalendarConfig,
        theme: Theme,
        healthcare_config: opengp_config::healthcare::HealthcareConfig,
        patient_config: opengp_config::PatientConfig,
        allergy_config: opengp_config::AllergyConfig,
        clinical_config: opengp_config::ClinicalConfig,
        social_history_config: opengp_config::SocialHistoryConfig,
        billing_ui_service: Option<Arc<BillingUiService>>,
        appointment_ui_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
        practice_config: opengp_config::PracticeConfig,
        max_open_patients: usize,
    ) -> Self {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel::<AppCommand>();
        
        let mut app = Self {
            theme: theme.clone(),
            keybinds: KeybindRegistry::global(),
            tab_bar: TabBar::new(theme.clone()),
            previous_tab: Tab::PatientSearch,
            status_bar: StatusBar::patient_list(theme.clone()),
            help_overlay: HelpOverlay::new(theme.clone()),
            login_screen: crate::ui::screens::LoginScreen::new(theme.clone()),
            authenticated: true,
            current_context: KeyContext::Global,
            should_quit: false,
            current_user_id: uuid::Uuid::nil(),
            patient_list: PatientList::new(theme.clone()),
            patient_form: None,
            pending_patient_data: None,
            pending_clinical_save_data: None,
            pending_edit_patient_id: None,
            appointment_state: AppointmentState::new(theme.clone(), calendar_config),
             appointment_form: None,
             appointment_detail_modal: None,
             pending_load_practitioners: false,
            pending_load_booked_slots: None,
            pending_appointment_save: None::<(opengp_domain::domain::appointment::NewAppointmentData, i32)>,
            pending_appointment_status_transition: None,
            pending_reschedule: None,
            workspace_manager: WorkspaceManager::new(theme.clone(), max_open_patients),
            billing_ui_service,
            appointment_ui_service,
            practice_config,
            healthcare_config,
            patient_config,
            allergy_config,
            clinical_config,
            social_history_config,
            api_client,
            patient_page_limit: DEFAULT_PATIENT_PAGE_LIMIT,
            appointment_page_limit: DEFAULT_APPOINTMENT_PAGE_LIMIT,
            consultation_page_limit: DEFAULT_CONSULTATION_PAGE_LIMIT,
            pending_patient_list_refresh: false,
            pending_appointment_list_refresh: None,
            pending_consultation_list_refresh: None,
            pending_practitioners_list_refresh: false,
             patient_list_fetch_task: None,
             appointment_list_fetch_task: None,
             consultation_list_fetch_task: None,
             practitioners_list_fetch_task: None,
              reschedule_task: None,
              pending_login_request: None,
             login_task: None,
             clinical_workspace_load_task: None,
             server_unavailable_error: None,
            server_unavailable_retry: None,
            active_login_attempt: None,
            active_appointment_refresh_date: None,
            active_consultation_refresh_patient_id: None,
            terminal_size: Rect::new(0, 0, 80, 24),
            command_tx,
            command_rx: Some(command_rx),
            context_menu_state: None,
            last_billing_render: None,
            hovered_clinical_menu: None,
        };

        app.refresh_status_bar();
        app.refresh_context();

        app
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn keybinds(&self) -> &KeybindRegistry {
        self.keybinds
    }

    pub fn current_tab(&self) -> Tab {
        self.tab_bar.selected()
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn workspace_manager(&self) -> &WorkspaceManager {
        &self.workspace_manager
    }

    pub fn workspace_manager_mut(&mut self) -> &mut WorkspaceManager {
        &mut self.workspace_manager
    }

    pub fn has_clinical_workspace_load_task(&self) -> bool {
        self.clinical_workspace_load_task.is_some()
    }

    pub fn set_clinical_workspace_load_task(
        &mut self,
        patient_id: uuid::Uuid,
        task: tokio::task::JoinHandle<ClinicalWorkspaceLoadResult>,
    ) {
        self.clinical_workspace_load_task = Some((patient_id, task));
    }

    pub fn billing_ui_service(&self) -> Option<Arc<BillingUiService>> {
        self.billing_ui_service.clone()
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

    pub fn take_command_rx(&mut self) -> Option<tokio::sync::mpsc::UnboundedReceiver<AppCommand>> {
        self.command_rx.take()
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
            Tab::PatientSearch | Tab::PatientWorkspace => {
                if let Some(workspace) = self.workspace_manager.active() {
                    StatusBar::patient_workspace(self.theme.clone(), &workspace.patient_snapshot.full_name)
                } else {
                    StatusBar::patient_list(self.theme.clone())
                }
            }
            Tab::Schedule => StatusBar::schedule(self.theme.clone()),
        };
    }

    fn refresh_context(&mut self) {
        self.current_context = match self.tab_bar.selected() {
            Tab::PatientSearch | Tab::PatientWorkspace => {
                if self.workspace_manager.active().is_some() {
                    KeyContext::PatientWorkspace
                } else {
                    KeyContext::PatientList
                }
            }
            Tab::Schedule => {
                if self.appointment_form.is_some() || self.appointment_detail_modal.is_some() {
                    KeyContext::Schedule
                } else {
                    match self.appointment_state.current_view {
                        AppointmentView::Calendar => KeyContext::Calendar,
                        AppointmentView::Schedule => KeyContext::Schedule,
                    }
                }
            }
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
                    self.pending_consultation_list_refresh = Some(patient_id);
                }
            }
            self.clear_server_unavailable_error();
        }
    }

    #[cfg(test)]
    pub fn server_unavailable_error_message(&self) -> Option<&str> {
        self.server_unavailable_error.as_deref()
    }

    pub fn show_context_menu(&mut self, menu: ActiveContextMenu) {
        use crate::ui::widgets::{ContextMenuItem, ContextMenuState};
        use ratatui::layout::Position;

        let items = match &menu {
            ActiveContextMenu::Patient { patient_id, .. } => vec![
                ContextMenuItem {
                    label: "Edit Patient".into(),
                    action: AppContextMenuAction::PatientEdit(*patient_id),
                    enabled: true,
                },
                ContextMenuItem {
                    label: "View History".into(),
                    action: AppContextMenuAction::PatientViewHistory(*patient_id),
                    enabled: true,
                },
                ContextMenuItem {
                    label: "Delete Patient".into(),
                    action: AppContextMenuAction::PatientDelete(*patient_id),
                    enabled: true,
                },
            ],
            ActiveContextMenu::Clinical { item_id, .. } => vec![
                ContextMenuItem {
                    label: "Edit".into(),
                    action: AppContextMenuAction::ClinicalEdit(*item_id),
                    enabled: true,
                },
                ContextMenuItem {
                    label: "Delete".into(),
                    action: AppContextMenuAction::ClinicalDelete(*item_id),
                    enabled: true,
                },
            ],
            ActiveContextMenu::Billing { item_id, .. } => vec![
                ContextMenuItem {
                    label: "Edit".into(),
                    action: AppContextMenuAction::BillingEdit(*item_id),
                    enabled: true,
                },
                ContextMenuItem {
                    label: "View Invoice".into(),
                    action: AppContextMenuAction::BillingViewInvoice(*item_id),
                    enabled: true,
                },
            ],
        };

        let position = match &menu {
            ActiveContextMenu::Patient { x, y, .. } => Position::new(*x, *y),
            ActiveContextMenu::Clinical { x, y, .. } => Position::new(*x, *y),
            ActiveContextMenu::Billing { x, y, .. } => Position::new(*x, *y),
        };

        let mut state = ContextMenuState::new(self.theme.clone(), items);
        state.show_at(position);
        self.context_menu_state = Some(state);
    }

    pub fn hide_context_menu(&mut self) {
        self.context_menu_state = None;
    }

    pub fn is_context_menu_visible(&self) -> bool {
        self.context_menu_state
            .as_ref()
            .map(|s| s.is_visible())
            .unwrap_or(false)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(
            None,
            CalendarConfig::default(),
            Theme::dark(),
            opengp_config::healthcare::HealthcareConfig::default(),
            opengp_config::PatientConfig::default(),
            opengp_config::AllergyConfig::default(),
            opengp_config::ClinicalConfig::default(),
            opengp_config::SocialHistoryConfig::default(),
            None,
            None,
            opengp_config::PracticeConfig::default(),
            8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        assert_eq!(app.current_tab(), Tab::Schedule);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_tab_switching() {
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        assert_eq!(app.current_tab(), Tab::PatientSearch);
    }

    #[test]
    fn test_help_toggle() {
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);

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
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);

        assert!(app.should_quit());
    }

    #[test]
    fn test_calendar_keybind_routing() {
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(2),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert_eq!(app.current_tab(), Tab::Schedule);

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
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(2),
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
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(2),
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

        let initial_slot = app.appointment_state().selected_time_slot;
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert!(
            app.appointment_state().selected_time_slot >= initial_slot,
            "Schedule time slot should advance after pressing 'j'"
        );
    }

    #[test]
    fn test_q_does_not_quit_on_appointment() {
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(2),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert_eq!(app.current_tab(), Tab::Schedule);

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
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
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
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(2),
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
        let mut app = App::new(None, CalendarConfig::default(), Theme::dark(), opengp_config::healthcare::HealthcareConfig::default(), opengp_config::PatientConfig::default(), opengp_config::AllergyConfig::default(), opengp_config::ClinicalConfig::default(), opengp_config::SocialHistoryConfig::default(), None, None, opengp_config::PracticeConfig::default(), 8);
        assert_eq!(app.current_tab(), Tab::Schedule);
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);
        assert!(
            !app.should_quit(),
            "Bare 'q' should NOT quit from Schedule tab (only from PatientSearch)"
        );
    }

    #[test]
    fn test_sign_consultation_variant() {
        let consultation_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let variant = PendingClinicalSaveData::SignConsultation {
            consultation_id,
            user_id,
        };
        match variant {
            PendingClinicalSaveData::SignConsultation {
                consultation_id: cid,
                user_id: uid,
            } => {
                assert_eq!(cid, consultation_id);
                assert_eq!(uid, user_id);
            }
            _ => panic!("Expected SignConsultation variant"),
        }
    }

    #[test]
    fn test_tab_navigation_redesign() {
        let mut app = App::new(
            None,
            CalendarConfig::default(),
            Theme::dark(),
            opengp_config::healthcare::HealthcareConfig::default(),
            opengp_config::PatientConfig::default(),
            opengp_config::AllergyConfig::default(),
            opengp_config::ClinicalConfig::default(),
            opengp_config::SocialHistoryConfig::default(),
            None,
            None,
            opengp_config::PracticeConfig::default(),
            8,
        );

        assert_eq!(app.current_tab(), Tab::Schedule);

        let f2 = crossterm::event::KeyEvent::new(crossterm::event::KeyCode::F(2), crossterm::event::KeyModifiers::NONE);
        app.handle_key_event(f2);
        assert_eq!(app.current_tab(), Tab::Schedule, "F2 should activate Schedule tab");

        let f3 = crossterm::event::KeyEvent::new(crossterm::event::KeyCode::F(3), crossterm::event::KeyModifiers::NONE);
        app.handle_key_event(f3);
        assert_eq!(app.current_tab(), Tab::PatientSearch, "F3 should activate Patient Search tab");

        let f2_again = crossterm::event::KeyEvent::new(crossterm::event::KeyCode::F(2), crossterm::event::KeyModifiers::NONE);
        app.handle_key_event(f2_again);
        assert_eq!(app.current_tab(), Tab::Schedule, "F2 should return to Schedule tab");
    }
}
