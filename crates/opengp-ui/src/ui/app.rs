//! OpenGP Application State
//!
//! Main application state management, rendering, and event handling.

use chrono::NaiveDate;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use std::sync::Arc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::Frame;

use opengp_config::CalendarConfig;
use crate::ui::components::appointment::{
    AppointmentDetailModal, AppointmentDetailModalAction, AppointmentForm, AppointmentFormAction,
    AppointmentFormField, AppointmentState, AppointmentView, CalendarAction, ScheduleAction,
};
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::help::HelpOverlay;
use crate::ui::components::patient::{PatientForm, PatientList, PatientState};
use crate::ui::components::status_bar::{StatusBar, STATUS_BAR_HEIGHT};
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};
use crate::ui::widgets::format_date;

/// Application state
pub struct App {
    /// Theme configuration
    theme: Theme,
    /// Keybind registry (reference to global singleton)
    keybinds: &'static KeybindRegistry,
    /// Tab bar state
    tab_bar: TabBar,
    /// Status bar
    status_bar: StatusBar,
    /// Help overlay
    help_overlay: HelpOverlay,
    /// Current key context
    current_context: KeyContext,
    /// Whether the application should quit
    should_quit: bool,
    /// Application title
    #[allow(dead_code)]
    title: String,
    /// Version info
    #[allow(dead_code)]
    version: String,
    /// Patient component state
    #[allow(dead_code)]
    patient_state: PatientState,
    /// Patient list component
    patient_list: PatientList,
    /// Patient form component
    patient_form: Option<PatientForm>,
    /// Pending patient data to save (new or update)
    pending_patient_data: Option<PendingPatientData>,
    /// Pending patient ID to load for editing
    pending_edit_patient_id: Option<uuid::Uuid>,
    /// Appointment/schedule component state
    appointment_state: AppointmentState,
    #[allow(dead_code)]
    appointment_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
    #[allow(dead_code)]
    patient_service: Option<Arc<crate::ui::services::PatientUiService>>,
    pending_appointment_date: Option<NaiveDate>,
    /// Flag to load practitioners for appointment form picker
    pending_load_practitioners: bool,
    /// Appointment creation form (open when user presses 'n' in Schedule view)
    appointment_form: Option<AppointmentForm>,
    /// Appointment detail modal (open when user selects an appointment)
    appointment_detail_modal: Option<AppointmentDetailModal>,
    /// Pending appointment data to save (set on form Submit)
    pending_appointment_save: Option<opengp_domain::domain::appointment::NewAppointmentData>,
    /// Pending appointment status transition
    pending_appointment_status_transition: Option<(uuid::Uuid, AppointmentStatusTransition)>,
    /// Pending patient ID to load for clinical view
    pending_clinical_patient_id: Option<uuid::Uuid>,
    pending_clinical_save_data: Option<PendingClinicalSaveData>,
    /// Clinical component state
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

/// Pending clinical data to save (new records from forms)
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
}

#[derive(Debug)]
pub enum AppointmentStatusTransition {
    MarkArrived,
    MarkInProgress,
    MarkCompleted,
}

impl App {
    /// Create a new application instance
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
            tab_bar: TabBar::new(),
            status_bar: StatusBar::patient_list(),
            help_overlay: HelpOverlay::new(),
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

    /// Load patients into the list
    pub fn load_patients(&mut self, patients: Vec<opengp_domain::domain::patient::Patient>) {
        let list_items: Vec<PatientListItem> =
            patients.into_iter().map(PatientListItem::from).collect();
        self.patient_list.set_patients(list_items);
    }

    /// Take pending patient data (for saving to database)
    pub fn take_pending_patient_data(&mut self) -> Option<PendingPatientData> {
        self.pending_patient_data.take()
    }

    /// Take pending patient ID to load for editing
    pub fn take_pending_edit_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_edit_patient_id.take()
    }

    /// Set pending patient ID to load for editing (from UI event)
    pub fn request_edit_patient(&mut self, patient_id: uuid::Uuid) {
        self.pending_edit_patient_id = Some(patient_id);
    }

    /// Take pending appointment date (for loading practitioners in main loop)
    pub fn take_pending_appointment_date(&mut self) -> Option<NaiveDate> {
        self.pending_appointment_date.take()
    }

    /// Request loading practitioners for appointment form picker
    pub fn request_load_practitioners(&mut self) {
        self.pending_load_practitioners = true;
    }

    /// Take pending load practitioners flag
    pub fn take_pending_load_practitioners(&mut self) -> bool {
        std::mem::take(&mut self.pending_load_practitioners)
    }

    /// Take pending appointment save data (for saving to database in main loop)
    pub fn take_pending_appointment_save(
        &mut self,
    ) -> Option<opengp_domain::domain::appointment::NewAppointmentData> {
        self.pending_appointment_save.take()
    }

    pub fn take_pending_appointment_status_transition(
        &mut self,
    ) -> Option<(uuid::Uuid, AppointmentStatusTransition)> {
        self.pending_appointment_status_transition.take()
    }

    pub fn take_pending_clinical_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_clinical_patient_id.take()
    }

    pub fn take_pending_clinical_save_data(&mut self) -> Option<PendingClinicalSaveData> {
        self.pending_clinical_save_data.take()
    }

    /// Get mutable reference to appointment state (for loading practitioners)
    pub fn appointment_state_mut(&mut self) -> &mut AppointmentState {
        &mut self.appointment_state
    }

    /// Set patients in the appointment form picker
    pub fn appointment_form_set_patients(&mut self, patients: Vec<PatientListItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_patients(patients);
        }
    }

    /// Set practitioners in the appointment form picker
    pub fn appointment_form_set_practitioners(&mut self, practitioners: Vec<PractitionerViewItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_practitioners(practitioners);
        }
    }

    pub fn clinical_state_mut(&mut self) -> &mut ClinicalState {
        &mut self.clinical_state
    }

    /// Open patient form for editing (called from main loop after fetching patient)
    pub fn open_patient_form(&mut self, patient: opengp_domain::domain::patient::Patient) {
        self.patient_form = Some(PatientForm::from_patient(patient, self.theme.clone()));
        self.current_context = KeyContext::PatientForm;
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the keybind registry
    pub fn keybinds(&self) -> &KeybindRegistry {
        &self.keybinds
    }

    /// Get the current tab
    pub fn current_tab(&self) -> Tab {
        self.tab_bar.selected()
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Set the quit flag
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get appointment state for testing
    #[cfg(test)]
    pub fn appointment_state(&self) -> &AppointmentState {
        &self.appointment_state
    }

    /// Get practitioners from appointment state
    pub fn practitioners(&self) -> &[opengp_domain::domain::user::Practitioner] {
        &self.appointment_state.practitioners
    }

    /// Check if appointment form is open
    pub fn has_appointment_form(&self) -> bool {
        self.appointment_form.is_some()
    }

    /// Get patients from the patient list
    pub fn patient_list_patients(&self) -> &[PatientListItem] {
        self.patient_list.patients()
    }

    /// Toggle the theme between dark and light
    pub fn toggle_theme(&mut self) {
        if self.theme.colors.background == Color::Black {
            self.theme = Theme::light();
        } else {
            self.theme = Theme::dark();
        }
    }

    fn refresh_status_bar(&mut self) {
        self.status_bar = match self.tab_bar.selected() {
            Tab::Patient => StatusBar::patient_list(),
            Tab::Appointment => StatusBar::schedule(),
            Tab::Clinical => StatusBar::clinical(),
            Tab::Billing => StatusBar::billing(),
        };
    }

    fn refresh_context(&mut self) {
        use crate::ui::components::appointment::AppointmentView;
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
        // Calculate based on terminal size: total height - tab bar (2) - header (2) - status bar (1)
        let available_height = self.terminal_size.height.saturating_sub(2 + 2 + 1);
        // Each patient row takes 1 line, so available_height = visible rows
        available_height.saturating_sub(1) as usize // -1 for safety margin
    }

    /// Handle a key event
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        // Check help overlay first
        if self.help_overlay.is_visible() {
            if key.code == crossterm::event::KeyCode::Esc
                || key.code == crossterm::event::KeyCode::F(1)
            {
                self.help_overlay.hide();
                return Action::Escape;
            }
            return Action::Unknown;
        }

        // Handle patient list search mode - route ALL keys to patient list, bypass global keybinds
        if self.tab_bar.selected() == Tab::Patient
            && self.patient_form.is_none()
            && self.patient_list.is_searching()
        {
            if let Some(action) = self.patient_list.handle_key(key) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(_id) => {}
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
                return Action::Enter;
            }
        }

        if self.tab_bar.selected() == Tab::Clinical && self.clinical_state.is_form_open() {
            return self.handle_clinical_keys(key);
        }

        if self.appointment_form.is_some() {
            return self.handle_appointment_form_keys(key);
        }

        if self.appointment_detail_modal.is_some() {
            return self.handle_appointment_detail_modal_keys(key);
        }

        // Handle patient form keys when form is open - route keys to form
        if self.patient_form.is_some() {
            if let Some(ref mut form) = self.patient_form {
                if let Some(action) = form.handle_key(key) {
                    match action {
                        crate::ui::components::patient::PatientFormAction::FocusChanged => {}
                        crate::ui::components::patient::PatientFormAction::ValueChanged => {}
                        crate::ui::components::patient::PatientFormAction::Submit => {
                            if let Some(ref mut form) = self.patient_form {
                                if !form.has_errors() {
                                    if form.is_edit_mode() {
                                        if let Some((id, data)) = form.to_update_patient_data() {
                                            self.pending_patient_data =
                                                Some(PendingPatientData::Update { id, data });
                                        }
                                    } else if let Some(data) = form.to_new_patient_data() {
                                        self.pending_patient_data =
                                            Some(PendingPatientData::New(data));
                                    }
                                    self.patient_form = None;
                                    self.current_context = KeyContext::PatientList;
                                }
                            }
                        }
                        crate::ui::components::patient::PatientFormAction::Cancel => {
                            self.patient_form = None;
                            self.current_context = KeyContext::PatientList;
                        }
                        crate::ui::components::patient::PatientFormAction::SaveComplete => {}
                    }
                    return Action::Enter;
                }
            }
        }

        // Look up the action from keybinds - clone to avoid borrow issues
        let action = self
            .keybinds
            .lookup(key, self.current_context)
            .map(|kb| kb.action.clone());

        if let Some(action) = action {
            // Handle actions that need mutable self
            match action {
                Action::SwitchToPatient => {
                    self.tab_bar.select(Tab::Patient);
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::SwitchToAppointments => {
                    self.tab_bar.select(Tab::Appointment);
                    let today = chrono::Utc::now().date_naive();
                    self.appointment_state.selected_date = Some(today);
                    self.pending_appointment_date = Some(today);
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::SwitchToClinical => {
                    self.tab_bar.select(Tab::Clinical);
                    if let Some(patient_id) = self.patient_list.selected_patient_id() {
                        self.clinical_state.set_patient(patient_id);
                    }
                    self.clinical_state.show_patient_summary();
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::SwitchToBilling => {
                    self.tab_bar.select(Tab::Billing);
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::OpenHelp => {
                    self.help_overlay.toggle();
                }
                Action::Quit => {
                    // Ctrl+Q always quits. Bare 'q' only quits from Patient tab.
                    // On Appointment/Clinical/Billing tabs, 'q' may be used for navigation.
                    let is_ctrl_q = key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL);
                    if is_ctrl_q || self.tab_bar.selected() == Tab::Patient {
                        self.should_quit = true;
                    }
                }
                Action::New => {
                    use crate::ui::components::clinical::ClinicalView;
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        self.patient_form = Some(PatientForm::new(self.theme.clone()));
                        self.current_context = KeyContext::PatientForm;
                    }
                    if self.tab_bar.selected() == Tab::Clinical
                        && !self.clinical_state.is_form_open()
                    {
                        match self.clinical_state.view {
                            ClinicalView::Allergies => {
                                self.clinical_state.open_allergy_form();
                                self.current_context = KeyContext::ClinicalForm;
                            }
                            ClinicalView::MedicalHistory => {
                                self.clinical_state.open_medical_history_form();
                                self.current_context = KeyContext::ClinicalForm;
                            }
                            ClinicalView::VitalSigns => {
                                self.clinical_state.open_vitals_form();
                                self.current_context = KeyContext::ClinicalForm;
                            }
                            ClinicalView::FamilyHistory => {
                                self.clinical_state.open_family_history_form();
                                self.current_context = KeyContext::ClinicalForm;
                            }
                            ClinicalView::Consultations => {
                                self.clinical_state.open_consultation_form();
                                self.current_context = KeyContext::ClinicalForm;
                            }
                            ClinicalView::SocialHistory => {
                                self.clinical_state.social_history_editing = true;
                            }
                            ClinicalView::PatientSummary => {
                                // Navigate to Consultations to create a new consultation
                                self.clinical_state.view = ClinicalView::Consultations;
                                self.clinical_state.open_consultation_form();
                                self.current_context = KeyContext::ClinicalForm;
                            }
                        }
                    }
                }
                Action::Edit => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        if let Some(patient_id) = self.patient_list.selected_patient_id() {
                            // Set pending edit ID - main loop will fetch full Patient and create form
                            self.request_edit_patient(patient_id);
                        }
                    }
                }
                Action::Delete => {}
                Action::Escape => {
                    if self.patient_form.is_some() {
                        self.patient_form = None;
                        self.current_context = KeyContext::PatientList;
                    }
                    if self.appointment_form.is_some() {
                        self.appointment_form = None;
                    }
                    // Also return to Calendar view from Schedule view
                    if self.tab_bar.selected() == Tab::Appointment
                        && self.appointment_state.current_view == AppointmentView::Schedule
                        && self.appointment_form.is_none()
                    {
                        self.appointment_state.current_view = AppointmentView::Calendar;
                        self.appointment_state.calendar.focused = true;
                        self.appointment_state.schedule.focused = false;
                        self.refresh_context();
                    }
                }
                Action::Save => {}
                Action::NavigateDown => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        let visible_rows = self.calculate_visible_patient_rows();
                        self.patient_list.move_down_and_scroll(visible_rows);
                    }
                }
                Action::NavigateUp => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        let visible_rows = self.calculate_visible_patient_rows();
                        self.patient_list.move_up_and_scroll(visible_rows);
                    }
                }
                // Appointment calendar actions - delegate to handle_appointment_keys
                Action::PrevDay
                | Action::NextDay
                | Action::Today
                | Action::PrevMonth
                | Action::NextMonth
                | Action::SelectDate => {
                    if self.tab_bar.selected() == Tab::Appointment {
                        return self.handle_appointment_keys(key);
                    }
                }
                // Appointment schedule actions - delegate to handle_appointment_keys
                Action::PrevPractitioner
                | Action::NextPractitioner
                | Action::PrevTimeSlot
                | Action::NextTimeSlot
                | Action::ScrollViewportUp
                | Action::ScrollViewportDown => {
                    if self.tab_bar.selected() == Tab::Appointment {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::Enter => {
                    if self.tab_bar.selected() == Tab::Patient {
                        return self.handle_patient_keys(key);
                    }
                    if self.tab_bar.selected() == Tab::Appointment {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::NewAppointment => {
                    if self.tab_bar.selected() == Tab::Appointment
                        && self.appointment_form.is_none()
                    {
                        self.appointment_form = Some(AppointmentForm::new(self.theme.clone()));
                        self.request_load_practitioners();
                    }
                }
                // Clinical view switching (number keys 1-7)
                Action::SwitchToPatientSummary
                | Action::SwitchToConsultations
                | Action::SwitchToAllergies
                | Action::SwitchToMedicalHistory
                | Action::SwitchToVitalSigns
                | Action::SwitchToSocialHistory
                | Action::SwitchToFamilyHistory => {
                    if self.tab_bar.selected() == Tab::Clinical {
                        return self.handle_clinical_keys(key);
                    }
                }
                // Clinical quick actions (letter keys)
                Action::ViewAllergies
                | Action::ViewConditions
                | Action::ViewVitals
                | Action::ViewObservations
                | Action::ViewFamilyHistory
                | Action::ViewSocialHistory => {
                    if self.tab_bar.selected() == Tab::Clinical {
                        return self.handle_clinical_keys(key);
                    }
                }
                _ => {}
            }
            return action;
        }

        // Handle tab bar navigation
        if let Some(_tab) = self.tab_bar.handle_key(key) {
            self.refresh_status_bar();
            self.refresh_context();
            return Action::Enter;
        }

        // Handle patient list navigation when in list view
        if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
            return self.handle_patient_keys(key);
        }

        // Handle calendar navigation when in appointment view (calendar mode)
        if self.tab_bar.selected() == Tab::Appointment {
            return self.handle_appointment_keys(key);
        }

        // Handle clinical tab navigation
        if self.tab_bar.selected() == Tab::Clinical {
            return self.handle_clinical_keys(key);
        }

        Action::Unknown
    }

    /// Handle key events for the Clinical tab
    fn handle_clinical_keys(&mut self, key: KeyEvent) -> Action {
        use crate::ui::components::clinical::{
            AllergyFormAction, ClinicalFormView, ClinicalView, ConsultationFormAction,
            FamilyHistoryFormAction, MedicalHistoryFormAction, VitalSignsFormAction,
        };
        use crate::ui::keybinds::{Action as KeyAction, KeyContext, KeybindRegistry};
        use crate::ui::widgets::SearchableListState;
        use crossterm::event::KeyCode;

        if self.clinical_state.is_form_open() {
            match self.clinical_state.form_view.clone() {
                ClinicalFormView::AllergyForm => {
                    if let Some(ref mut form) = self.clinical_state.allergy_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                AllergyFormAction::FocusChanged
                                | AllergyFormAction::ValueChanged => {}
                                AllergyFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = uuid::Uuid::nil();
                                            let allergy =
                                                form.to_allergy(patient_id, system_user_id);
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::Allergy {
                                                    patient_id,
                                                    allergy,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                AllergyFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::MedicalHistoryForm => {
                    if let Some(ref mut form) = self.clinical_state.medical_history_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                MedicalHistoryFormAction::FocusChanged
                                | MedicalHistoryFormAction::ValueChanged => {}
                                MedicalHistoryFormAction::Submit => {
                                    if let Some(patient_id) =
                                        self.clinical_state.selected_patient_id
                                    {
                                        let system_user_id = uuid::Uuid::nil();
                                        if let Some(history) =
                                            form.to_medical_history(patient_id, system_user_id)
                                        {
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::MedicalHistory {
                                                    patient_id,
                                                    history,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                MedicalHistoryFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::VitalSignsForm => {
                    if let Some(ref mut form) = self.clinical_state.vitals_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                VitalSignsFormAction::FocusChanged
                                | VitalSignsFormAction::ValueChanged => {}
                                VitalSignsFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = uuid::Uuid::nil();
                                            let vitals =
                                                form.to_vital_signs(patient_id, system_user_id);
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::VitalSigns {
                                                    patient_id,
                                                    vitals,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                VitalSignsFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::FamilyHistoryForm => {
                    if let Some(ref mut form) = self.clinical_state.family_history_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                FamilyHistoryFormAction::FocusChanged
                                | FamilyHistoryFormAction::ValueChanged => {}
                                FamilyHistoryFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = uuid::Uuid::nil();
                                            let entry =
                                                form.to_family_history(patient_id, system_user_id);
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::FamilyHistory {
                                                    patient_id,
                                                    entry,
                                                });
                                            self.clinical_state.close_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar
                                                .set_error("Please fill in required fields");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                FamilyHistoryFormAction::Cancel => {
                                    self.clinical_state.close_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::ConsultationForm => {
                    if let Some(ref mut form) = self.clinical_state.consultation_form {
                        if let Some(action) = form.handle_key(key) {
                            match action {
                                ConsultationFormAction::FocusChanged
                                | ConsultationFormAction::ValueChanged => {}
                                ConsultationFormAction::Submit => {
                                    let form_valid = form.validate();
                                    if form_valid {
                                        if let Some(patient_id) =
                                            self.clinical_state.selected_patient_id
                                        {
                                            let system_user_id = uuid::Uuid::nil();
                                            let practitioner_id = uuid::Uuid::nil();
                                            let consultation = form.to_consultation(
                                                patient_id,
                                                practitioner_id,
                                                system_user_id,
                                            );
                                            self.pending_clinical_save_data =
                                                Some(PendingClinicalSaveData::Consultation {
                                                    patient_id,
                                                    practitioner_id,
                                                    appointment_id: consultation.appointment_id,
                                                    reason: consultation.reason,
                                                    clinical_notes: consultation.clinical_notes,
                                                });
                                            self.clinical_state.close_consultation_form();
                                            self.current_context = KeyContext::Clinical;
                                            self.status_bar.clear_error();
                                        } else {
                                            self.status_bar.set_error("No patient selected");
                                        }
                                    } else {
                                        self.status_bar.set_error("Please fill in required fields");
                                    }
                                }
                                ConsultationFormAction::Cancel => {
                                    self.clinical_state.close_consultation_form();
                                    self.current_context = KeyContext::Clinical;
                                    self.status_bar.clear_error();
                                }
                            }
                            return Action::Enter;
                        }
                    }
                }
                ClinicalFormView::None => {}
            }
            return Action::Unknown;
        }

        // Handle patient search popup if open
        if let Some(ref mut search) = self.clinical_state.patient_search {
            if search.is_open() {
                use crate::ui::widgets::{SearchableList, SearchableListAction};
                let mut picker = SearchableList::new(search, &self.theme, "Select Patient", true);
                let action = picker.handle_key(key);
                match action {
                    SearchableListAction::Selected(id, _name) => {
                        self.clinical_state.set_patient(id);
                        self.clinical_state.patient_search = None;
                        return Action::Enter;
                    }
                    SearchableListAction::Cancelled => {
                        self.clinical_state.patient_search = None;
                        return Action::Enter;
                    }
                    SearchableListAction::None => {
                        return Action::Enter;
                    }
                }
            }
        }

        // First check keybinds registry for clinical-specific actions
        let registry = KeybindRegistry::global();
        if let Some(keybind) = registry.lookup(key, KeyContext::Clinical) {
            match keybind.action {
                // Number keys 1-7 to jump to specific views
                KeyAction::SwitchToPatientSummary => {
                    self.clinical_state.show_patient_summary();
                    return Action::Enter;
                }
                KeyAction::SwitchToConsultations => {
                    self.clinical_state.show_consultations();
                    return Action::Enter;
                }
                KeyAction::SwitchToAllergies => {
                    self.clinical_state.show_allergies();
                    return Action::Enter;
                }
                KeyAction::SwitchToMedicalHistory => {
                    self.clinical_state.show_medical_history();
                    return Action::Enter;
                }
                KeyAction::SwitchToVitalSigns => {
                    self.clinical_state.show_vital_signs();
                    return Action::Enter;
                }
                KeyAction::SwitchToSocialHistory => {
                    self.clinical_state.show_social_history();
                    return Action::Enter;
                }
                KeyAction::SwitchToFamilyHistory => {
                    self.clinical_state.show_family_history();
                    return Action::Enter;
                }
                // Quick action keys
                KeyAction::ViewAllergies => {
                    self.clinical_state.show_allergies();
                    return Action::Enter;
                }
                KeyAction::ViewConditions => {
                    self.clinical_state.show_medical_history();
                    return Action::Enter;
                }
                KeyAction::ViewVitals => {
                    self.clinical_state.show_vital_signs();
                    return Action::Enter;
                }
                KeyAction::ViewObservations => {
                    self.clinical_state.show_consultations();
                    return Action::Enter;
                }
                KeyAction::ViewFamilyHistory => {
                    self.clinical_state.show_family_history();
                    return Action::Enter;
                }
                KeyAction::ViewSocialHistory => {
                    self.clinical_state.show_social_history();
                    return Action::Enter;
                }
                KeyAction::Search => {
                    if self.clinical_state.patient_search.is_none()
                        && !self.clinical_state.is_form_open()
                    {
                        let patients: Vec<_> = self
                            .patient_list
                            .patients()
                            .iter()
                            .map(|p| crate::ui::view_models::PatientListItem {
                                id: p.id,
                                full_name: p.full_name.clone(),
                                date_of_birth: p.date_of_birth,
                                gender: p.gender.clone(),
                                medicare_number: p.medicare_number.clone(),
                                medicare_irn: p.medicare_irn,
                                ihi: p.ihi.clone(),
                                phone_mobile: p.phone_mobile.clone(),
                            })
                            .collect();
                        let mut search = SearchableListState::new(patients);
                        search.open();
                        self.clinical_state.patient_search = Some(search);
                    }
                    return Action::Enter;
                }
                _ => {}
            }
        }

        // Handle view switching with arrow keys
        if key.code == KeyCode::Right {
            self.clinical_state.cycle_view();
            return Action::Enter;
        }
        if key.code == KeyCode::Left {
            self.clinical_state.cycle_view_reverse();
            return Action::Enter;
        }

        // Dispatch to active component's handle_key()
        match self.clinical_state.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => {
                if let Some(action) = self.clinical_state.consultation_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::ConsultationListAction::Select(_)
                        | crate::ui::components::clinical::ConsultationListAction::Open(_)
                        | crate::ui::components::clinical::ConsultationListAction::New
                        | crate::ui::components::clinical::ConsultationListAction::NextPage
                        | crate::ui::components::clinical::ConsultationListAction::PrevPage => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::Allergies => {
                if let Some(action) = self.clinical_state.allergy_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::AllergyListAction::New => {
                            self.clinical_state.open_allergy_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::AllergyListAction::Select(_)
                        | crate::ui::components::clinical::AllergyListAction::Open(_)
                        | crate::ui::components::clinical::AllergyListAction::ToggleInactive
                        | crate::ui::components::clinical::AllergyListAction::Delete(_) => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::MedicalHistory => {
                if let Some(action) = self.clinical_state.medical_history_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::MedicalHistoryListAction::New => {
                            self.clinical_state.open_medical_history_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::MedicalHistoryListAction::Select(_)
                        | crate::ui::components::clinical::MedicalHistoryListAction::Open(_)
                        | crate::ui::components::clinical::MedicalHistoryListAction::SetFilter(_)
                        | crate::ui::components::clinical::MedicalHistoryListAction::Delete(_) => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::VitalSigns => {
                if let Some(action) = self.clinical_state.vitals_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::VitalSignsListAction::New => {
                            self.clinical_state.open_vitals_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::VitalSignsListAction::Select(_)
                        | crate::ui::components::clinical::VitalSignsListAction::Open(_)
                        | crate::ui::components::clinical::VitalSignsListAction::NextPage
                        | crate::ui::components::clinical::VitalSignsListAction::PrevPage => {
                            return Action::Enter;
                        }
                    }
                }
            }
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => {
                if let Some(action) = self.clinical_state.family_history_list.handle_key(key) {
                    match action {
                        crate::ui::components::clinical::FamilyHistoryListAction::New => {
                            self.clinical_state.open_family_history_form();
                            self.current_context = KeyContext::ClinicalForm;
                            return Action::Enter;
                        }
                        crate::ui::components::clinical::FamilyHistoryListAction::Select(_)
                        | crate::ui::components::clinical::FamilyHistoryListAction::Open(_)
                        | crate::ui::components::clinical::FamilyHistoryListAction::Delete(_) => {
                            return Action::Enter;
                        }
                    }
                }
            }
        }

        Action::Unknown
    }

    /// Handle key events for the Patient tab
    fn handle_patient_keys(&mut self, key: KeyEvent) -> Action {
        if let Some(action) = self.patient_list.handle_key(key) {
            match action {
                crate::ui::components::patient::PatientListAction::SelectionChanged => {
                    let visible_rows = self.calculate_visible_patient_rows();
                    self.patient_list.adjust_scroll(visible_rows);
                }
                crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                    self.clinical_state.clear_patient();
                    self.clinical_state.set_patient(id);
                    self.clinical_state.show_patient_summary();
                    self.tab_bar.select(Tab::Clinical);
                    self.pending_clinical_patient_id = Some(id);
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                crate::ui::components::patient::PatientListAction::FocusSearch => {
                    self.current_context = KeyContext::Search;
                }
                crate::ui::components::patient::PatientListAction::SearchChanged => {
                    if !self.patient_list.is_searching() {
                        self.current_context = KeyContext::PatientList;
                    }
                }
            }
            return Action::Enter;
        }
        Action::Unknown
    }

    /// Handle key events for the Appointment tab (calendar and schedule)
    fn handle_appointment_keys(&mut self, key: KeyEvent) -> Action {
        use crate::ui::components::appointment::AppointmentView;

        // Handle calendar navigation when in calendar mode
        if self.appointment_state.current_view == AppointmentView::Calendar {
            if let Some(action) = self.appointment_state.calendar.handle_key(key) {
                match action {
                    CalendarAction::SelectDate(date) => {
                        self.appointment_state.selected_date = Some(date);
                        self.appointment_state.current_view = AppointmentView::Schedule;
                        self.appointment_state.schedule.focused = true;
                        self.appointment_state.calendar.focused = false;
                        self.pending_appointment_date = Some(date);
                        self.refresh_context();
                    }
                    CalendarAction::FocusDate(_) => {}
                    CalendarAction::MonthChanged(_) => {}
                    CalendarAction::GoToToday => {}
                }
                return Action::Enter;
            }
        }

        // Handle schedule navigation when in schedule mode
        if self.appointment_state.current_view == AppointmentView::Schedule {
            if let Some(action) = self.appointment_state.schedule.handle_key(key) {
                match action {
                    ScheduleAction::SelectPractitioner(id) => {
                        self.appointment_state.selected_practitioner = Some(id);
                    }
                    ScheduleAction::SelectAppointment(id) => {
                        if let Some(ref schedule_data) = self.appointment_state.schedule_data {
                            for practitioner in &schedule_data.practitioners {
                                if let Some(appointment) =
                                    practitioner.appointments.iter().find(|apt| apt.id == id)
                                {
                                    self.appointment_detail_modal =
                                        Some(AppointmentDetailModal::new(
                                            appointment.clone(),
                                            self.theme.clone(),
                                        ));
                                    self.refresh_context();
                                    break;
                                }
                            }
                        }
                        self.appointment_state.selected_appointment = Some(id);
                    }
                    ScheduleAction::NavigateTimeSlot(_delta) => {}
                    ScheduleAction::NavigatePractitioner(_delta) => {}
                    ScheduleAction::CreateAtSlot {
                        practitioner_id,
                        date,
                        time,
                    } => {
                        self.appointment_form = Some(AppointmentForm::new(self.theme.clone()));
                        if let Some(ref mut form) = self.appointment_form {
                            if let Some(ref schedule_data) = self.appointment_state.schedule_data {
                                if let Some(practitioner) = schedule_data
                                    .practitioners
                                    .iter()
                                    .find(|p| p.practitioner_id == practitioner_id)
                                {
                                    form.set_practitioner(
                                        practitioner_id,
                                        practitioner.practitioner_name.clone(),
                                    );
                                }
                            }
                            form.set_value(AppointmentFormField::Date, format_date(date));
                            form.set_value(AppointmentFormField::StartTime, time);
                        }
                        self.request_load_practitioners();
                    }
                }
                return Action::Enter;
            }
        }

        Action::Unknown
    }

    fn handle_appointment_form_keys(&mut self, key: KeyEvent) -> Action {
        if let Some(ref mut form) = self.appointment_form {
            if let Some(action) = form.handle_key(key) {
                match action {
                    AppointmentFormAction::FocusChanged | AppointmentFormAction::ValueChanged => {}
                    AppointmentFormAction::Submit => {
                        if let Some(ref mut form) = self.appointment_form {
                            if let Some(data) = form.to_new_appointment_data() {
                                self.pending_appointment_save = Some(data);
                                self.appointment_form = None;
                                self.status_bar.clear_error();
                            } else {
                                self.status_bar.set_error(
                                    "Cannot save: select a patient and practitioner from the picker",
                                );
                            }
                        }
                    }
                    AppointmentFormAction::Cancel | AppointmentFormAction::SaveComplete => {
                        self.appointment_form = None;
                        self.status_bar.clear_error();
                    }
                }
                return Action::Enter;
            }
        }
        Action::Unknown
    }

    fn handle_appointment_detail_modal_keys(&mut self, key: KeyEvent) -> Action {
        if let Some(ref mut modal) = self.appointment_detail_modal {
            if let Some(action) = modal.handle_key(key) {
                match action {
                    AppointmentDetailModalAction::Close => {
                        self.appointment_detail_modal = None;
                    }
                    AppointmentDetailModalAction::ViewClinicalNotes => {
                        let patient_id = modal.patient_id();
                        self.appointment_detail_modal = None;
                        self.clinical_state.clear_patient();
                        self.clinical_state.set_patient(patient_id);
                        self.clinical_state.show_patient_summary();
                        self.tab_bar.select(Tab::Clinical);
                        self.pending_clinical_patient_id = Some(patient_id);
                        self.refresh_status_bar();
                        self.refresh_context();
                    }
                    AppointmentDetailModalAction::MarkArrived => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkArrived));
                    }
                    AppointmentDetailModalAction::MarkInProgress => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkInProgress));
                    }
                    AppointmentDetailModalAction::MarkCompleted => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkCompleted));
                    }
                }
                return Action::Enter;
            }
        }
        Action::Unknown
    }

    /// Handle a mouse event
    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, area: Rect) {
        let tab_bar_area = self.tab_bar.area(area);
        if self.tab_bar.handle_mouse(mouse, tab_bar_area).is_some() {
            self.refresh_status_bar();
            self.refresh_context();
            return;
        }

        if let Some(ref mut form) = self.patient_form {
            if let Some(action) = form.handle_mouse(mouse, area) {
                match action {
                    crate::ui::components::patient::PatientFormAction::FocusChanged => {}
                    crate::ui::components::patient::PatientFormAction::ValueChanged => {}
                    crate::ui::components::patient::PatientFormAction::Submit => {}
                    crate::ui::components::patient::PatientFormAction::Cancel => {}
                    crate::ui::components::patient::PatientFormAction::SaveComplete => {}
                }
                return;
            }
        }

        if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
            let content_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
            );
            if let Some(action) = self.patient_list.handle_mouse(mouse, content_area) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        self.request_edit_patient(id);
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
            }
        }

        // Handle appointment view mouse events - layout aware
        if self.tab_bar.selected() == Tab::Appointment {
            use crate::ui::components::appointment::schedule::ScheduleAction;
            use crate::ui::components::appointment::AppointmentView;

            let appointment_content_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
            );

            match self.appointment_state.current_view {
                AppointmentView::Calendar => {
                    // Content area (below tab bar) goes to calendar
                    self.appointment_state.calendar.focused = true;
                    self.appointment_state.schedule.focused = false;
                    if let Some(action) = self
                        .appointment_state
                        .calendar
                        .handle_mouse(mouse, appointment_content_area)
                    {
                        match action {
                            CalendarAction::SelectDate(date) => {
                                self.appointment_state.selected_date = Some(date);
                                self.appointment_state.current_view = AppointmentView::Schedule;
                                self.pending_appointment_date = Some(date);
                                self.refresh_context();
                            }
                            CalendarAction::FocusDate(_) => {}
                            CalendarAction::MonthChanged(_) => {}
                            CalendarAction::GoToToday => {}
                        }
                    }
                }
                AppointmentView::Schedule => {
                    // Replicate the split layout from render_content
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                        .split(appointment_content_area);

                    // Route to calendar (left pane) - only handle clicks, not scroll (scroll is for schedule only)
                    use crossterm::event::MouseEventKind;
                    if let MouseEventKind::Up(_) | MouseEventKind::Down(_) = mouse.kind {
                        if let Some(action) = self
                            .appointment_state
                            .calendar
                            .handle_mouse(mouse, chunks[0])
                        {
                            self.appointment_state.calendar.focused = true;
                            self.appointment_state.schedule.focused = false;
                            match action {
                                CalendarAction::SelectDate(date) => {
                                    self.appointment_state.selected_date = Some(date);
                                    self.pending_appointment_date = Some(date);
                                    // Stay in Schedule view - user clicked a different date
                                }
                                CalendarAction::FocusDate(_) => {}
                                CalendarAction::MonthChanged(_) => {}
                                CalendarAction::GoToToday => {}
                            }
                        }
                    }

                    // Route to schedule (right pane) with correct sub-area
                    if let Some(action) = self
                        .appointment_state
                        .schedule
                        .handle_mouse(mouse, chunks[1])
                    {
                        self.appointment_state.schedule.focused = true;
                        self.appointment_state.calendar.focused = false;
                        match action {
                            ScheduleAction::SelectPractitioner(id) => {
                                self.appointment_state.selected_practitioner = Some(id);
                            }
                            ScheduleAction::SelectAppointment(id) => {
                                self.appointment_state.selected_appointment = Some(id);
                            }
                            ScheduleAction::NavigateTimeSlot(_) => {}
                            ScheduleAction::NavigatePractitioner(_) => {}
                            ScheduleAction::CreateAtSlot { .. } => {}
                        }
                    }
                }
            }
        }

        // Handle clinical tab mouse events
        if self.tab_bar.selected() == Tab::Clinical && self.clinical_state.is_form_open() == false {
            use crate::ui::components::clinical::ClinicalView;
            let clinical_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
            );
            // Route mouse events to clinical components based on current view
            match self.clinical_state.view {
                ClinicalView::Consultations => {
                    let _ = self
                        .clinical_state
                        .consultation_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::Allergies => {
                    let _ = self
                        .clinical_state
                        .allergy_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::MedicalHistory => {
                    let _ = self
                        .clinical_state
                        .medical_history_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::VitalSigns => {
                    let _ = self
                        .clinical_state
                        .vitals_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::FamilyHistory => {
                    let _ = self
                        .clinical_state
                        .family_history_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::PatientSummary | ClinicalView::SocialHistory => {
                    // Patient summary and social history don't have mouse handling
                }
            }
        }
    }

    /// Handle terminal events
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                self.handle_key_event(key);
            }
            Event::Mouse(mouse) => {
                self.handle_mouse_event(mouse, self.terminal_size);
            }
            Event::Resize(w, h) => {
                self.terminal_size = Rect::new(0, 0, w, h);
            }
            _ => {}
        }
    }

    /// Render the application
    pub fn render(&mut self, frame: &mut Frame) {
        let terminal = frame.area();

        // If help overlay is visible, only render it
        if self.help_overlay.is_visible() {
            frame.render_widget(self.help_overlay.clone(), terminal);
            return;
        }

        // Calculate layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),                 // Tab bar
                Constraint::Min(0),                    // Content area
                Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
            ])
            .split(terminal);

        let tab_bar_area = main_layout[0];
        let content_area = main_layout[1];
        let status_bar_area = main_layout[2];

        // Render tab bar
        frame.render_widget(self.tab_bar.clone(), tab_bar_area);

        // Render content area (placeholder for now)
        self.render_content(frame, content_area);

        // Render status bar
        frame.render_widget(self.status_bar.clone(), status_bar_area);

        // Render patient search overlay on top of everything (including schedule)
        if self.patient_list.is_searching() {
            use ratatui::prelude::{Stylize, Widget};
            use ratatui::text::Line;
            use ratatui::widgets::Clear;

            let query = self.patient_list.search_query();
            let search_text = if query.is_empty() {
                Line::from(vec!["/".bold()])
            } else {
                Line::from(vec![format!("/{}", query).into()])
            };
            let overlay_area = Rect::new(
                content_area.x + 1,
                content_area.y + 1,
                content_area.width.saturating_sub(2),
                1,
            );
            frame.render_widget(Clear, overlay_area);
            search_text.render(overlay_area, frame.buffer_mut());
        }
    }

    /// Render the content area based on current tab
    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        let tab = self.tab_bar.selected();

        match tab {
            Tab::Patient => self.render_patient_tab(frame, area),
            Tab::Appointment => self.render_appointment_tab(frame, area),
            Tab::Clinical => self.render_clinical_tab(frame, area),
            Tab::Billing => self.render_billing_tab(frame, area),
        }
    }

    /// Render the Patient tab content
    fn render_patient_tab(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref mut form) = self.patient_form {
            frame.render_widget(form.clone(), area);
        } else {
            frame.render_widget(self.patient_list.clone(), area);
        }
    }

    /// Render the Appointment tab content
    fn render_appointment_tab(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::appointment::AppointmentView;
        use ratatui::widgets::Clear;

        if let Some(ref form) = self.appointment_form {
            frame.render_widget(form.clone(), area);
            return;
        }

        if let Some(ref modal) = self.appointment_detail_modal {
            frame.render_widget(Clear, area);
            frame.render_widget(modal.clone(), area);
            return;
        }

        match self.appointment_state.current_view {
            AppointmentView::Calendar => {
                frame.render_widget(self.appointment_state.calendar.clone(), area);
            }
            AppointmentView::Schedule => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                    .split(area);

                frame.render_widget(self.appointment_state.calendar.clone(), chunks[0]);

                let schedule = &mut self.appointment_state.schedule;

                let schedule_inner_height = chunks[1].height.saturating_sub(2);
                schedule.set_inner_height(schedule_inner_height);

                if let Some(ref data) = self.appointment_state.schedule_data {
                    schedule.load_schedule(data.clone());
                }

                if !self.appointment_state.practitioners.is_empty()
                    && self.appointment_state.schedule_data.is_none()
                {
                    use opengp_domain::domain::appointment::{CalendarDayView, PractitionerSchedule};

                    let date = self
                        .appointment_state
                        .selected_date
                        .unwrap_or_else(|| chrono::Utc::now().date_naive());

                    let schedules: Vec<PractitionerSchedule> = self
                        .appointment_state
                        .practitioners
                        .iter()
                        .map(|p| PractitionerSchedule {
                            practitioner_id: p.id,
                            practitioner_name: p.display_name(),
                            appointments: Vec::new(),
                        })
                        .collect();

                    let day_view = CalendarDayView {
                        date,
                        practitioners: schedules,
                    };

                    schedule.load_schedule(day_view);
                }

                frame.render_widget(schedule.clone(), chunks[1]);
            }
        }
    }

    /// Render the Clinical tab content
    fn render_clinical_tab(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::clinical::{ClinicalFormView, SocialHistoryComponent};

        if self.clinical_state.is_form_open() {
            match self.clinical_state.form_view.clone() {
                ClinicalFormView::AllergyForm => {
                    if let Some(ref form) = self.clinical_state.allergy_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::MedicalHistoryForm => {
                    if let Some(ref form) = self.clinical_state.medical_history_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::VitalSignsForm => {
                    if let Some(ref form) = self.clinical_state.vitals_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::FamilyHistoryForm => {
                    if let Some(ref form) = self.clinical_state.family_history_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::ConsultationForm => {
                    if let Some(ref form) = self.clinical_state.consultation_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::None => {}
            }
            return;
        }

        if !self.clinical_state.has_patient() {
            // Show message to select a patient first
            use ratatui::text::Text;
            use ratatui::widgets::{Block, Borders, Paragraph};

            let content = "No Patient Selected\n\nPlease select a patient from the Patient tab\nto view their clinical records.";

            let paragraph = Paragraph::new(Text::from(content))
                .block(
                    Block::default()
                        .title(format!(" {} ", self.tab_bar.selected().name()))
                        .borders(Borders::ALL)
                        .border_style(
                            ratatui::style::Style::default().fg(self.theme.colors.border),
                        ),
                )
                .style(ratatui::style::Style::default().fg(self.theme.colors.foreground))
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(paragraph, area);
            return;
        }

        // Sync data to persistent component instances before rendering
        self.clinical_state.consultation_list.consultations =
            self.clinical_state.consultations.clone();
        self.clinical_state.consultation_list.loading = self.clinical_state.loading;

        self.clinical_state.allergy_list.allergies = self.clinical_state.allergies.clone();
        self.clinical_state.allergy_list.loading = self.clinical_state.loading;

        self.clinical_state.medical_history_list.conditions =
            self.clinical_state.medical_history.clone();
        self.clinical_state.medical_history_list.loading = self.clinical_state.loading;

        self.clinical_state.vitals_list.vitals = self.clinical_state.vital_signs.clone();
        self.clinical_state.vitals_list.loading = self.clinical_state.loading;

        self.clinical_state.family_history_list.entries =
            self.clinical_state.family_history.clone();
        self.clinical_state.family_history_list.loading = self.clinical_state.loading;

        match self.clinical_state.view {
            crate::ui::components::clinical::ClinicalView::PatientSummary => {
                use crate::ui::components::clinical::PatientSummaryComponent;

                // Get patient data from patient list
                let patient_item = self.patient_list.selected_patient();

                let mut component = PatientSummaryComponent::new(self.theme.clone());

                component.patient = patient_item.cloned();

                component.allergies = self.clinical_state.allergies.clone();
                component.conditions = self.clinical_state.medical_history.clone();
                component.consultations = self.clinical_state.consultations.clone();
                component.vitals = self.clinical_state.vital_signs.last().cloned();

                frame.render_widget(component, area);
            }
            crate::ui::components::clinical::ClinicalView::Consultations => {
                frame.render_widget(self.clinical_state.consultation_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::Allergies => {
                frame.render_widget(self.clinical_state.allergy_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::MedicalHistory => {
                frame.render_widget(self.clinical_state.medical_history_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::VitalSigns => {
                frame.render_widget(self.clinical_state.vitals_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::SocialHistory => {
                let mut component = SocialHistoryComponent::new(self.theme.clone());
                component.loading = self.clinical_state.loading;
                component.is_editing = self.clinical_state.social_history_editing;
                // Convert domain SocialHistory to UI SocialHistoryData
                if let Some(ref sh) = self.clinical_state.social_history {
                    component.social_history = Some(
                        crate::ui::components::clinical::social_history::SocialHistoryData {
                            smoking_status: sh.smoking_status,
                            cigarettes_per_day: sh.cigarettes_per_day,
                            smoking_quit_date: sh.smoking_quit_date,
                            alcohol_status: sh.alcohol_status,
                            standard_drinks_per_week: sh.standard_drinks_per_week,
                            exercise_frequency: sh.exercise_frequency,
                            occupation: sh.occupation.clone(),
                            living_situation: sh.living_situation.clone(),
                            support_network: sh.support_network.clone(),
                            notes: sh.notes.clone(),
                        },
                    );
                }
                if component.is_editing && component.social_history.is_some() {
                    component.start_editing();
                }
                frame.render_widget(component, area);
            }
            crate::ui::components::clinical::ClinicalView::FamilyHistory => {
                frame.render_widget(self.clinical_state.family_history_list.clone(), area);
            }
        }

        if let Some(ref mut search) = self.clinical_state.patient_search {
            if search.is_open() {
                use crate::ui::widgets::SearchableList;
                let picker = SearchableList::new(search, &self.theme, "Select Patient", true);
                frame.render_widget(picker, area);
            }
        }
    }

    /// Render the Billing tab content
    fn render_billing_tab(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::text::Text;
        use ratatui::widgets::{Block, Borders, Paragraph};

        let content = "Billing\n\nInvoicing and payments\nMedicare claims";

        let paragraph = Paragraph::new(Text::from(content))
            .block(
                Block::default()
                    .title(format!(" {} ", self.tab_bar.selected().name()))
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(self.theme.colors.border)),
            )
            .style(ratatui::style::Style::default().fg(self.theme.colors.foreground))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
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

        // Simulate pressing F3 to switch to Appointments tab
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

        // Simulate pressing F1 to open help
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(1),
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        assert!(app.help_overlay.is_visible());

        // Press F1 again to close
        app.handle_key_event(key);

        assert!(!app.help_overlay.is_visible());
    }

    #[test]
    fn test_quit() {
        let mut app = App::new(None, None, None, CalendarConfig::default());

        // Simulate Ctrl+Q to quit
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

        // Switch to Calendar view (default is now Schedule)
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
