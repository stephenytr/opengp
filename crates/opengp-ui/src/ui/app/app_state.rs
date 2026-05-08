use chrono::NaiveDate;
use ratatui::layout::Rect;
use std::time::Instant;
use uuid::Uuid;

use crate::ui::components::appointment::AppointmentState;
use crate::ui::components::patient::PatientList;
use crate::ui::components::status_bar::StatusBar;
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::components::workspace::WorkspaceManager;

/// AppState contains all mutable UI and session state.
/// It is mutated by event handlers and rendered each frame.
pub struct AppState {
    // Core UI/navigation/session
    pub tab_bar: TabBar,
    pub previous_tab: Tab,
    pub status_bar: StatusBar,
    pub login_screen: crate::ui::screens::LoginScreen,
    pub authenticated: bool,
    pub should_quit: bool,
    pub current_user_id: Uuid,
    pub terminal_size: Rect,

    // Patient UI
    pub patient_list: PatientList,

    // Appointment UI
    pub appointment_state: AppointmentState,

    // Workspace/clinical/billing UI
    pub workspace_manager: WorkspaceManager,

    // Pagination + refresh intent flags
    pub patient_page_limit: u32,
    pub appointment_page_limit: u32,
    pub consultation_page_limit: u32,

    // Login/retry/server-failure state
    pub active_login_attempt: Option<(String, String)>,
    pub active_appointment_refresh_date: Option<NaiveDate>,

    // Context-menu/modals/render auxiliaries
    pub last_billing_render: Option<Instant>,
    pub hovered_clinical_menu: Option<usize>,
}
