//! OpenGP Application State
//!
//! Main application state management, rendering, and event handling.

use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::Frame;

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
    title: String,
    /// Version info
    version: String,
    /// Patient component state
    patient_state: PatientState,
    /// Patient list component
    patient_list: PatientList,
    /// Patient form component
    patient_form: Option<PatientForm>,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
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
        };

        app.refresh_status_bar();
        app.refresh_context();

        app
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

        // Look up the action from keybinds
        if let Some(keybind) = self.keybinds.lookup(key, self.current_context) {
            // Handle tab switching
            match keybind.action {
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
                // Patient list actions
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
                Action::Delete => {
                    // TODO: Implement delete confirmation
                }
                // Patient form actions
                Action::Escape => {
                    if self.patient_form.is_some() {
                        self.patient_form = None();
                        self.current_context = KeyContext::PatientList;
                    }
                }
                Action::Save => {
                    // TODO: Handle form save
                }
                _ => {}
            }
            return keybind.action;
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
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        // Open patient for viewing/editing
                        if let Some(patient) = self.patient_list.selected_patient().cloned() {
                            self.patient_form =
                                Some(PatientForm::from_patient(patient, self.theme.clone()));
                            self.current_context = KeyContext::PatientForm;
                        }
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {
                        // TODO: Focus search input
                    }
                }
                return Action::Enter;
            }
        }

        Action::Unknown
    }

    /// Handle a mouse event
    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, area: Rect) {
        // Handle tab bar mouse events
        let tab_bar_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)[0];

        if let Some(tab) = self.tab_bar.handle_mouse(mouse, tab_bar_area) {
            self.update_status_bar();
            self.update_context();
        }
    }

    /// Handle terminal events
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                self.handle_key_event(key);
            }
            Event::Mouse(mouse) => {
                // Get the main content area
                let terminal_height = 24; // This should come from the frame
                let area = Rect::new(0, 0, 80, terminal_height);
                self.handle_mouse_event(mouse, area);
            }
            Event::Resize(_, _) => {
                // Handle resize if needed
            }
            _ => {}
        }
    }

    /// Render the application
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>) {
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
    fn render_content<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
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
                use ratatui::text::Text;
                use ratatui::widgets::{Block, Borders, Paragraph};

                let content = "Appointments\n\nCalendar and schedule view\nh/l to navigate days\nj/k to navigate weeks";

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
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.current_tab(), Tab::Patient);
        assert!(!app.should_quit());
    }

    #[test]
    fn test_tab_switching() {
        let mut app = App::new();

        // Simulate pressing F3 to switch to Appointments tab
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::Key::F3,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_key_event(key);

        assert_eq!(app.current_tab(), Tab::Appointment);
    }

    #[test]
    fn test_help_toggle() {
        let mut app = App::new();

        assert!(!app.help_overlay.is_visible());

        // Simulate pressing F1 to open help
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::Key::F1,
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
        let mut app = App::new();

        // Simulate Ctrl+Q to quit
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::Key::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        app.handle_key_event(key);

        assert!(app.should_quit());
    }
}
