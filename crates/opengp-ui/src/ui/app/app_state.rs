use chrono::NaiveDate;
use ratatui::layout::Rect;
use std::time::Instant;
use uuid::Uuid;

use crate::ui::app::{
    AppContextMenuAction, AppointmentStatusTransition, PendingClinicalSaveData, PendingPatientData,
    PendingRescheduleData, RetryOperation,
};
use crate::ui::components::appointment::{
    AppointmentState,
};
use crate::ui::components::patient::PatientList;
use crate::ui::components::status_bar::StatusBar;
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::components::workspace::WorkspaceManager;
use crate::ui::keybinds::KeyContext;
use crate::ui::widgets::ContextMenuState;
use opengp_domain::domain::appointment::NewAppointmentData;

/// AppState contains all mutable UI and session state.
/// It is mutated by event handlers and rendered each frame.
pub struct AppState {
    // Core UI/navigation/session
    pub tab_bar: TabBar,
    pub previous_tab: Tab,
    pub status_bar: StatusBar,
    pub login_screen: crate::ui::screens::LoginScreen,
    pub authenticated: bool,
    pub current_context: KeyContext,
    pub should_quit: bool,
    pub current_user_id: Uuid,
    pub terminal_size: Rect,

    // Patient UI
    pub patient_list: PatientList,
    pub pending_patient_data: Option<PendingPatientData>,
    pub pending_edit_patient_id: Option<Uuid>,

    // Appointment UI
    pub appointment_state: AppointmentState,
    pub pending_load_practitioners: bool,
    pub pending_load_booked_slots: Option<(Uuid, NaiveDate, u32)>,
    pub pending_appointment_save: Option<(NewAppointmentData, i32)>,
    pub pending_appointment_status_transition: Option<(Uuid, AppointmentStatusTransition)>,
    pub pending_reschedule: Option<PendingRescheduleData>,

    // Workspace/clinical/billing UI
    pub workspace_manager: WorkspaceManager,
    pub pending_clinical_save_data: Option<PendingClinicalSaveData>,

    // Pagination + refresh intent flags
    pub patient_page_limit: u32,
    pub appointment_page_limit: u32,
    pub consultation_page_limit: u32,
    pub pending_patient_list_refresh: bool,
    pub pending_appointment_list_refresh: Option<NaiveDate>,
    pub pending_consultation_list_refresh: Option<Uuid>,
    pub pending_practitioners_list_refresh: bool,

    // Login/retry/server-failure state
    pub pending_login_request: Option<(String, String)>,
    pub active_login_attempt: Option<(String, String)>,
    pub server_unavailable_error: Option<String>,
    pub server_unavailable_retry: Option<RetryOperation>,
    pub active_appointment_refresh_date: Option<NaiveDate>,

    // Context-menu/modals/render auxiliaries
    pub context_menu_state: Option<ContextMenuState<AppContextMenuAction>>,
    pub last_billing_render: Option<Instant>,
    pub hovered_clinical_menu: Option<usize>,

    // Channel fields (to be removed after full rat-salsa migration)
    pub command_tx: tokio::sync::mpsc::UnboundedSender<crate::ui::app::AppCommand>,
    pub command_rx: Option<tokio::sync::mpsc::UnboundedReceiver<crate::ui::app::AppCommand>>,
}
