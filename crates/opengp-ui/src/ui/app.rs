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

pub struct App {
    theme: Theme,
    keybinds: &'static KeybindRegistry,
    tab_bar: TabBar,
    status_bar: StatusBar,
    help_overlay: HelpOverlay,
    current_context: KeyContext,
    should_quit: bool,
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

impl App {
    pub fn new(
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
            current_context: KeyContext::Global,
            should_quit: false,
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
}

impl Default for App {
    fn default() -> Self {
        Self::new(None, None, None, CalendarConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new(None, None, None, CalendarConfig::default());
        assert_eq!(app.current_tab(), Tab::Patient);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_tab_switching() {
        let mut app = App::new(None, None, None, CalendarConfig::default());
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(3),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        assert_eq!(app.current_tab(), Tab::Appointment);
    }

    #[test]
    fn test_help_toggle() {
        let mut app = App::new(None, None, None, CalendarConfig::default());

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
        let mut app = App::new(None, None, None, CalendarConfig::default());

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);

        assert!(app.should_quit());
    }

    #[test]
    fn test_calendar_keybind_routing() {
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
        let mut app = App::new(None, None, None, CalendarConfig::default());
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
