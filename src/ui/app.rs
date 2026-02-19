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
    AppointmentState, AppointmentView, CalendarAction, Schedule,
};
use crate::ui::components::help::HelpOverlay;
use crate::ui::components::patient::{PatientForm, PatientList, PatientState};
use crate::ui::components::status_bar::{StatusBar, STATUS_BAR_HEIGHT};
use crate::ui::components::tabs::{Tab, TabBar};
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;

/// Application state
pub struct App {
    /// Theme configuration
    theme: Theme,
    /// Keybind registry
    keybinds: KeybindRegistry,
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
    /// Appointment/schedule component state
    appointment_state: AppointmentState,
    /// Appointment UI service for loading practitioners
    appointment_service: Option<Arc<crate::ui::services::AppointmentUiService>>,
    /// Pending appointment date to load (for async loading in main loop)
    pending_appointment_date: Option<NaiveDate>,
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
    ) -> Self {
        let theme = Theme::dark();
        let mut app = Self {
            theme: theme.clone(),
            keybinds: KeybindRegistry::new(),
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
            appointment_state: AppointmentState::new(theme.clone()),
            appointment_service,
            pending_appointment_date: None,
        };

        app.refresh_status_bar();
        app.refresh_context();

        app
    }

    /// Load patients into the list
    pub fn load_patients(&mut self, patients: Vec<crate::domain::patient::Patient>) {
        self.patient_list.set_patients(patients);
    }

    /// Take pending patient data (for saving to database)
    pub fn take_pending_patient_data(&mut self) -> Option<PendingPatientData> {
        self.pending_patient_data.take()
    }

    /// Take pending appointment date (for loading practitioners in main loop)
    pub fn take_pending_appointment_date(&mut self) -> Option<NaiveDate> {
        self.pending_appointment_date.take()
    }

    /// Get mutable reference to appointment state (for loading practitioners)
    pub fn appointment_state_mut(&mut self) -> &mut AppointmentState {
        &mut self.appointment_state
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
        self.current_context = match self.tab_bar.selected() {
            Tab::Patient => KeyContext::PatientList,
            Tab::Appointment => KeyContext::Schedule,
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
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::SwitchToClinical => {
                    self.tab_bar.select(Tab::Clinical);
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
                        if let Some(patient) = self.patient_list.selected_patient().cloned() {
                            self.patient_form =
                                Some(PatientForm::from_patient(patient, self.theme.clone()));
                            self.current_context = KeyContext::PatientForm;
                        }
                    }
                }
                Action::Delete => {}
                Action::Escape => {
                    if self.patient_form.is_some() {
                        self.patient_form = None;
                        self.current_context = KeyContext::PatientList;
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
            if let Some(action) = self.patient_list.handle_key(key) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {
                        let visible_rows = self.calculate_visible_patient_rows();
                        self.patient_list.adjust_scroll(visible_rows);
                    }
                    crate::ui::components::patient::PatientListAction::OpenPatient(_id) => {
                        // Open patient for viewing/editing
                        if let Some(patient) = self.patient_list.selected_patient().cloned() {
                            self.patient_form =
                                Some(PatientForm::from_patient(patient, self.theme.clone()));
                            self.current_context = KeyContext::PatientForm;
                        }
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
                return Action::Enter;
            }
        }

        // Handle calendar navigation when in appointment view (calendar mode)
        if self.tab_bar.selected() == Tab::Appointment
            && self.appointment_state.current_view == AppointmentView::Calendar
        {
            if let Some(action) = self.appointment_state.calendar.handle_key(key) {
                match action {
                    CalendarAction::SelectDate(date) => {
                        self.appointment_state.selected_date = Some(date);
                        self.appointment_state.current_view = AppointmentView::Schedule;
                        self.pending_appointment_date = Some(date);
                    }
                    CalendarAction::FocusDate(_) => {}
                    CalendarAction::MonthChanged(_) => {}
                    CalendarAction::GoToToday => {}
                }
                return Action::Enter;
            }
        }

        if self.tab_bar.selected() == Tab::Appointment
            && self.appointment_state.current_view == AppointmentView::Schedule
        {
            if let Some(keybind) = self.keybinds.lookup(key, KeyContext::Schedule) {
                match keybind.action {
                    Action::NavigateUp | Action::PrevTimeSlot => {
                        tracing::debug!("Schedule: Navigate to previous time slot");
                    }
                    Action::NavigateDown | Action::NextTimeSlot => {
                        tracing::debug!("Schedule: Navigate to next time slot");
                    }
                    Action::NavigateLeft | Action::PrevPractitioner => {
                        tracing::debug!("Schedule: Navigate to previous practitioner");
                    }
                    Action::NavigateRight | Action::NextPractitioner => {
                        tracing::debug!("Schedule: Navigate to next practitioner");
                    }
                    Action::NewAppointment => {
                        tracing::debug!("Schedule: Create new appointment");
                    }
                    _ => {}
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
                    crate::ui::components::patient::PatientListAction::OpenPatient(_id) => {
                        if let Some(patient) = self.patient_list.selected_patient().cloned() {
                            self.patient_form =
                                Some(PatientForm::from_patient(patient, self.theme.clone()));
                            self.current_context = KeyContext::PatientForm;
                        }
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
            }
        }

        // Handle calendar mouse events when in appointment view (calendar mode)
        if self.tab_bar.selected() == Tab::Appointment
            && self.appointment_state.current_view == AppointmentView::Calendar
        {
            if let Some(action) = self.appointment_state.calendar.handle_mouse(mouse, area) {
                match action {
                    CalendarAction::SelectDate(date) => {
                        self.appointment_state.selected_date = Some(date);
                        self.appointment_state.current_view = AppointmentView::Schedule;
                        self.pending_appointment_date = Some(date);
                    }
                    CalendarAction::FocusDate(_) => {}
                    CalendarAction::MonthChanged(_) => {}
                    CalendarAction::GoToToday => {}
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
                let terminal_area = Rect::new(0, 0, 80, 24);
                self.handle_mouse_event(mouse, terminal_area);
            }
            Event::Resize(_, _) => {}
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
            Tab::Patient => {
                // Render patient form if active, otherwise render list
                if let Some(ref mut form) = self.patient_form {
                    frame.render_widget(form.clone(), area);
                } else {
                    frame.render_widget(self.patient_list.clone(), area);
                }
            }
            Tab::Appointment => {
                use crate::ui::components::appointment::AppointmentView;

                match self.appointment_state.current_view {
                    AppointmentView::Calendar => {
                        frame.render_widget(self.appointment_state.calendar.clone(), area);
                    }
                    AppointmentView::Schedule => {
                        // Split view: calendar on left, schedule on right
                        use ratatui::layout::Constraint;

                        let chunks = Layout::default()
                            .direction(ratatui::layout::Direction::Horizontal)
                            .constraints([
                                Constraint::Percentage(25), // Calendar takes 25%
                                Constraint::Percentage(75), // Schedule takes 75%
                            ])
                            .split(area);

                        // Render calendar on left
                        frame.render_widget(self.appointment_state.calendar.clone(), chunks[0]);

                        // Render schedule on right (create with theme)
                        let mut schedule = Schedule::new(self.theme.clone());

                        // If we have schedule data loaded, load it into the schedule
                        if let Some(ref data) = self.appointment_state.schedule_data {
                            schedule.load_schedule(data.clone());
                        }

                        // If we have practitioners but no schedule_data, load them directly
                        if !self.appointment_state.practitioners.is_empty()
                            && self.appointment_state.schedule_data.is_none()
                        {
                            use crate::domain::appointment::{
                                CalendarDayView, PractitionerSchedule,
                            };

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

                        frame.render_widget(schedule, chunks[1]);
                    }
                }
            }
            Tab::Clinical => {
                use ratatui::text::Text;
                use ratatui::widgets::{Block, Borders, Paragraph};

                let content =
                    "Clinical Notes\n\nPatient clinical records\nSOAP notes, allergies, history";

                let paragraph = Paragraph::new(Text::from(content))
                    .block(
                        Block::default()
                            .title(format!(" {} ", tab.name()))
                            .borders(Borders::ALL)
                            .border_style(
                                ratatui::style::Style::default().fg(self.theme.colors.border),
                            ),
                    )
                    .style(ratatui::style::Style::default().fg(self.theme.colors.foreground))
                    .alignment(ratatui::layout::Alignment::Center);

                frame.render_widget(paragraph, area);
            }
            Tab::Billing => {
                use ratatui::text::Text;
                use ratatui::widgets::{Block, Borders, Paragraph};

                let content = "Billing\n\nInvoicing and payments\nMedicare claims";

                let paragraph = Paragraph::new(Text::from(content))
                    .block(
                        Block::default()
                            .title(format!(" {} ", tab.name()))
                            .borders(Borders::ALL)
                            .border_style(
                                ratatui::style::Style::default().fg(self.theme.colors.border),
                            ),
                    )
                    .style(ratatui::style::Style::default().fg(self.theme.colors.foreground))
                    .alignment(ratatui::layout::Alignment::Center);

                frame.render_widget(paragraph, area);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new(None);
        assert_eq!(app.current_tab(), Tab::Patient);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_tab_switching() {
        let mut app = App::new(None);

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
        let mut app = App::new(None);

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
        let mut app = App::new(None);

        // Simulate Ctrl+Q to quit
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);

        assert!(app.should_quit());
    }
}
