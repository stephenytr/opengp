//! OpenGP Application State
//!
//! Main application state management, rendering, and event handling.

use chrono::NaiveDate;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use std::sync::Arc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::Frame;

use crate::ui::components::appointment::{
    AppointmentState, AppointmentView, CalendarAction, ScheduleAction,
};
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::help::HelpOverlay;
use crate::ui::components::patient::{PatientForm, PatientList, PatientState};
use crate::ui::components::status_bar::{StatusBar, STATUS_BAR_HEIGHT};
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;

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
    appointment_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
    patient_service: Option<Arc<crate::ui::services::PatientUiService>>,
    pending_appointment_date: Option<NaiveDate>,
    /// Clinical component state
    clinical_state: ClinicalState,
    clinical_service: Option<Arc<crate::ui::services::ClinicalUiService>>,
    terminal_size: Rect,
}

pub enum PendingPatientData {
    New(crate::domain::patient::NewPatientData),
    Update {
        id: uuid::Uuid,
        data: crate::domain::patient::UpdatePatientData,
    },
}

impl App {
    /// Create a new application instance
    pub fn new(
        appointment_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
        patient_service: Option<Arc<crate::ui::services::PatientUiService>>,
        clinical_service: Option<Arc<crate::ui::services::ClinicalUiService>>,
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
            appointment_state: AppointmentState::new(theme.clone()),
            appointment_service,
            patient_service,
            pending_appointment_date: None,
            clinical_state: ClinicalState::with_theme(theme.clone()),
            clinical_service,
            terminal_size: Rect::new(0, 0, 80, 24),
        };

        app.refresh_status_bar();
        app.refresh_context();

        app
    }

    /// Load patients into the list
    pub fn load_patients(&mut self, patients: Vec<crate::domain::patient::Patient>) {
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

    /// Get mutable reference to appointment state (for loading practitioners)
    pub fn appointment_state_mut(&mut self) -> &mut AppointmentState {
        &mut self.appointment_state
    }

    /// Open patient form for editing (called from main loop after fetching patient)
    pub fn open_patient_form(&mut self, patient: crate::domain::patient::Patient) {
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
            Tab::Appointment => match self.appointment_state.current_view {
                AppointmentView::Calendar => KeyContext::Calendar,
                AppointmentView::Schedule => KeyContext::Schedule,
            },
            Tab::Clinical => KeyContext::Clinical,
            Tab::Billing => KeyContext::Billing,
        };
        self.help_overlay.set_context(self.current_context);
    }

    fn calculate_visible_patient_rows(&self) -> usize {
        15_usize.saturating_sub(5)
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
                    // Sync patient selection from Patient tab to Clinical tab
                    if let Some(patient_id) = self.patient_list.selected_patient_id() {
                        self.clinical_state.set_patient(patient_id);
                    }
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
                    self.should_quit = true;
                }
                Action::New => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        self.patient_form = Some(PatientForm::new(self.theme.clone()));
                        self.current_context = KeyContext::PatientForm;
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
                    // Also return to Calendar view from Schedule view
                    if self.tab_bar.selected() == Tab::Appointment
                        && self.appointment_state.current_view == AppointmentView::Schedule
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
        use crate::ui::components::clinical::ClinicalView;
        use crossterm::event::KeyCode;

        // Handle view switching with arrow keys
        if key.code == KeyCode::Right {
            self.clinical_state.cycle_view();
            return Action::Enter;
        }
        if key.code == KeyCode::Left {
            self.clinical_state.cycle_view_reverse();
            return Action::Enter;
        }

        // Handle navigation keys
        let action = self
            .keybinds
            .lookup(key, self.current_context)
            .map(|kb| kb.action.clone());

        if let Some(action) = action {
            match action {
                Action::NavigateDown => {
                    let visible_rows = self.calculate_visible_clinical_rows();
                    self.clinical_state.next_item();
                    self.clinical_state.adjust_scroll(visible_rows);
                    return Action::Enter;
                }
                Action::NavigateUp => {
                    let visible_rows = self.calculate_visible_clinical_rows();
                    self.clinical_state.prev_item();
                    self.clinical_state.adjust_scroll(visible_rows);
                    return Action::Enter;
                }
                Action::New => {
                    // Handle new entry based on current view
                    return Action::Enter;
                }
                Action::Edit => {
                    // Handle edit based on current view
                    return Action::Enter;
                }
                Action::Delete => {
                    // Handle delete based on current view
                    return Action::Enter;
                }
                _ => {}
            }
        }

        Action::Unknown
    }

    fn calculate_visible_clinical_rows(&self) -> usize {
        15_usize.saturating_sub(5)
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
                    self.request_edit_patient(id);
                }
                crate::ui::components::patient::PatientListAction::FocusSearch => {}
                crate::ui::components::patient::PatientListAction::SearchChanged => {}
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
                        self.appointment_state.selected_appointment = Some(id);
                    }
                    ScheduleAction::NavigateTimeSlot(_delta) => {}
                    ScheduleAction::NavigatePractitioner(_delta) => {}
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
                        }
                    }
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

                if let Some(ref data) = self.appointment_state.schedule_data {
                    schedule.load_schedule(data.clone());
                }

                if !self.appointment_state.practitioners.is_empty()
                    && self.appointment_state.schedule_data.is_none()
                {
                    use crate::domain::appointment::{CalendarDayView, PractitionerSchedule};

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
        use crate::ui::components::clinical::{
            AllergyList, ConsultationList, FamilyHistoryList, MedicalHistoryList,
            SocialHistoryComponent, VitalSignsList,
        };

        // Check if we have a patient selected
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

        let theme = self.theme.clone();

        match self.clinical_state.view {
            crate::ui::components::clinical::ClinicalView::Consultations => {
                let mut list = ConsultationList::new(theme);
                list.consultations = self.clinical_state.consultations.clone();
                list.loading = self.clinical_state.loading;
                frame.render_widget(list, area);
            }
            crate::ui::components::clinical::ClinicalView::Allergies => {
                let mut list = AllergyList::new(theme);
                list.allergies = self.clinical_state.allergies.clone();
                list.loading = self.clinical_state.loading;
                frame.render_widget(list, area);
            }
            crate::ui::components::clinical::ClinicalView::MedicalHistory => {
                let mut list = MedicalHistoryList::new(theme);
                list.conditions = self.clinical_state.medical_history.clone();
                list.loading = self.clinical_state.loading;
                frame.render_widget(list, area);
            }
            crate::ui::components::clinical::ClinicalView::VitalSigns => {
                let mut list = VitalSignsList::new(theme);
                list.vitals = self.clinical_state.vital_signs.clone();
                list.loading = self.clinical_state.loading;
                frame.render_widget(list, area);
            }
            crate::ui::components::clinical::ClinicalView::SocialHistory => {
                let mut component = SocialHistoryComponent::new(theme);
                component.loading = self.clinical_state.loading;
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
                frame.render_widget(component, area);
            }
            crate::ui::components::clinical::ClinicalView::FamilyHistory => {
                let mut list = FamilyHistoryList::new(theme);
                list.entries = self.clinical_state.family_history.clone();
                list.loading = self.clinical_state.loading;
                frame.render_widget(list, area);
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
        Self::new(None, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new(None, None, None);
        assert_eq!(app.current_tab(), Tab::Patient);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_tab_switching() {
        let mut app = App::new(None, None, None);

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
        let mut app = App::new(None, None, None);

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
        let mut app = App::new(None, None, None);

        // Simulate Ctrl+Q to quit
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);

        assert!(app.should_quit());
    }
}
