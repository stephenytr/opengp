//! Main application orchestration
//!
//! The App struct coordinates all components, manages application state,
//! and handles the main event loop.

use crate::components::appointment::{AppointmentFormComponent, AppointmentCalendarComponent};
use crate::components::patient::{PatientFormComponent, PatientListComponent};
use crate::components::{Action, Component};
use crate::config::Config;
use crate::domain::appointment::{AppointmentService, AppointmentRepository};
use crate::domain::patient::{PatientService, PatientRepository};
use crate::domain::user::{PractitionerService, PractitionerRepository, RepositoryError, Practitioner};
use crate::error::Result;
use crate::infrastructure::database::repositories::{SqlxAppointmentRepository, SqlxPatientRepository};
use crate::ui::event::EventHandler;
use crate::ui::tui::Tui;
use async_trait::async_trait;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, info};
use uuid::Uuid;

/// Active screen in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Patients,
    Appointments,
    Clinical,
    Billing,
}

impl Screen {
    pub fn as_str(&self) -> &str {
        match self {
            Screen::Patients => "Patients",
            Screen::Appointments => "Appointments",
            Screen::Clinical => "Clinical",
            Screen::Billing => "Billing",
        }
    }

    pub fn all() -> Vec<Screen> {
        vec![
            Screen::Patients,
            Screen::Appointments,
            Screen::Clinical,
            Screen::Billing,
        ]
    }
}

/// Main application struct
///
/// Coordinates all components and manages the application lifecycle.
pub struct App {
    #[allow(dead_code)]
    config: Config,
    db_pool: SqlitePool,
    patient_service: Arc<PatientService>,
    appointment_service: Arc<AppointmentService>,
    practitioner_service: Arc<crate::domain::user::PractitionerService>,
    should_quit: bool,
    active_screen: Screen,
    patient_component: Option<Box<dyn Component>>,
    patient_form_component: Option<Box<dyn Component>>,
    appointment_component: Option<Box<dyn Component>>,
    appointment_form_component: Option<Box<dyn Component>>,
    clinical_component: Option<Box<dyn Component>>,
    billing_component: Option<Box<dyn Component>>,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    showing_form: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(config: Config, db_pool: SqlitePool) -> Result<Self> {
        info!("Initializing application");
        
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        
        let patient_repository: Arc<dyn PatientRepository> =
            Arc::new(SqlxPatientRepository::new(db_pool.clone()));
        let patient_service = Arc::new(PatientService::new(patient_repository));
        
        let appointment_repository: Arc<dyn AppointmentRepository> =
            Arc::new(SqlxAppointmentRepository::new(db_pool.clone()));
        let appointment_service = Arc::new(AppointmentService::new(appointment_repository));
        
        // Create mock practitioner repository for now (Phase 1)
        // In Phase 2, this would use a real SqlxPractitionerRepository
        struct MockPractitionerRepository;

        #[async_trait]
        impl PractitionerRepository for MockPractitionerRepository {
            async fn list_active(&self) -> std::result::Result<Vec<Practitioner>, RepositoryError> {
                Ok(vec![
                    Practitioner {
                        id: Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                            .expect("valid UUID"),
                        user_id: None,
                        first_name: "Sarah".to_string(),
                        middle_name: None,
                        last_name: "Johnson".to_string(),
                        title: "Dr".to_string(),
                        hpi_i: Some("8003610000000000".to_string()),
                        ahpra_registration: Some("MED0001234567".to_string()),
                        prescriber_number: Some("123456".to_string()),
                        provider_number: "123456A".to_string(),
                        speciality: Some("General Practice".to_string()),
                        qualifications: vec!["MBBS".to_string(), "FRACGP".to_string()],
                        phone: Some("02 9876 5432".to_string()),
                        email: Some("s.johnson@clinic.com".to_string()),
                        is_active: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    },
                    Practitioner {
                        id: Uuid::parse_str("b2c3d4e5-f6a7-89a1-b2c3-d4e5f6a789a1")
                            .expect("valid UUID"),
                        user_id: None,
                        first_name: "Michael".to_string(),
                        middle_name: Some("James".to_string()),
                        last_name: "Chen".to_string(),
                        title: "Dr".to_string(),
                        hpi_i: Some("8003610000000001".to_string()),
                        ahpra_registration: Some("MED0001234568".to_string()),
                        prescriber_number: Some("234567".to_string()),
                        provider_number: "234567B".to_string(),
                        speciality: Some("General Practice".to_string()),
                        qualifications: vec!["MBBS".to_string(), "FRACGP".to_string()],
                        phone: Some("02 9876 5433".to_string()),
                        email: Some("m.chen@clinic.com".to_string()),
                        is_active: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    },
                ])
            }
            
            async fn find_by_id(&self, _id: Uuid) -> std::result::Result<Option<Practitioner>, RepositoryError> {
                Ok(None)
            }
        }

        let practitioner_repository: Arc<dyn PractitionerRepository> = Arc::new(MockPractitionerRepository);
        let practitioner_service = Arc::new(PractitionerService::new(practitioner_repository));
        
        Ok(Self {
            config,
            db_pool,
            patient_service,
            appointment_service,
            practitioner_service,
            should_quit: false,
            active_screen: Screen::Patients,
            patient_component: None,
            patient_form_component: None,
            appointment_component: None,
            appointment_form_component: None,
            clinical_component: None,
            billing_component: None,
            action_tx,
            action_rx,
            showing_form: false,
        })
    }

    /// Run the application main event loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting application main loop");
        
        let mut tui = Tui::new()?;
        tui.enter()?;
        
        let event_handler = EventHandler::new();
        self.init_components().await?;
        
        loop {
            tui.draw(|f| self.render(f))?;
            
            let event = event_handler.next()?;
            debug!("Received event: {:?}", event);
            
            let action = self.handle_global_events(&event);
            if action != Action::None {
                self.action_tx.send(action)?;
            }
            
            let component_action = if self.showing_form {
                if let Some(form) = self.patient_form_component.as_mut() {
                    form.handle_events(Some(event))
                } else if let Some(form) = self.appointment_form_component.as_mut() {
                    form.handle_events(Some(event))
                } else {
                    Action::None
                }
            } else {
                self.get_active_component_mut()
                    .map(|c| c.handle_events(Some(event)))
                    .unwrap_or(Action::None)
            };
            
            if component_action != Action::None {
                self.action_tx.send(component_action)?;
            }
            
            while let Ok(action) = self.action_rx.try_recv() {
                self.update(action).await?;
            }
            
            if self.should_quit {
                break;
            }
        }
        
        tui.exit()?;
        info!("Application shutdown complete");
        
        Ok(())
    }

    /// Initialize all components
    async fn init_components(&mut self) -> Result<()> {
        info!("Initializing components");
        
        let mut patient_list = PatientListComponent::new(self.patient_service.clone());
        patient_list.init().await?;
        self.patient_component = Some(Box::new(patient_list));
        
        let mut appointment_calendar = AppointmentCalendarComponent::new(
            self.appointment_service.clone(),
            self.practitioner_service.clone(),
            self.patient_service.clone(),
        );
        appointment_calendar.init().await?;
        self.appointment_component = Some(Box::new(appointment_calendar));
        
        Ok(())
    }
    
    /// Get reference to patient service
    pub fn patient_service(&self) -> Arc<PatientService> {
        self.patient_service.clone()
    }

    /// Handle global key events (navigation, quit)
    fn handle_global_events(&self, event: &crate::ui::event::Event) -> Action {
        use crate::ui::event::Event;
        
        match event {
            Event::Key(key) => {
                if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
                    || key.code == KeyCode::Char('q')
                {
                    return Action::Quit;
                }
                
                if self.showing_form {
                    return Action::None;
                }
                
                match key.code {
                    KeyCode::Char('1') => Action::NavigateToPatients,
                    KeyCode::Char('2') => Action::NavigateToAppointments,
                    KeyCode::Char('3') => Action::NavigateToClinical,
                    KeyCode::Char('4') => Action::NavigateToBilling,
                    _ => Action::None,
                }
            }
            _ => Action::None,
        }
    }

    /// Navigate to next screen
    fn next_screen(&self) -> Action {
        match self.active_screen {
            Screen::Patients => Action::NavigateToAppointments,
            Screen::Appointments => Action::NavigateToClinical,
            Screen::Clinical => Action::NavigateToBilling,
            Screen::Billing => Action::NavigateToPatients,
        }
    }

    /// Navigate to previous screen
    fn prev_screen(&self) -> Action {
        match self.active_screen {
            Screen::Patients => Action::NavigateToBilling,
            Screen::Appointments => Action::NavigateToPatients,
            Screen::Clinical => Action::NavigateToAppointments,
            Screen::Billing => Action::NavigateToClinical,
        }
    }

    /// Update application state based on action
    async fn update(&mut self, action: Action) -> Result<()> {
        debug!("Processing action: {:?}", action);
        
        match action {
            Action::Quit => {
                info!("Quit action received");
                self.should_quit = true;
            }
            Action::NavigateToPatients => {
                info!("Navigating to Patients");
                self.active_screen = Screen::Patients;
                self.showing_form = false;
            }
            Action::NavigateToAppointments => {
                info!("Navigating to Appointments");
                self.active_screen = Screen::Appointments;
                self.showing_form = false;
            }
            Action::NavigateToClinical => {
                info!("Navigating to Clinical");
                self.active_screen = Screen::Clinical;
                self.showing_form = false;
            }
            Action::NavigateToBilling => {
                info!("Navigating to Billing");
                self.active_screen = Screen::Billing;
                self.showing_form = false;
            }
            Action::PatientCreate => {
                info!("Opening patient creation form");
                let mut form = PatientFormComponent::new(self.patient_service.clone());
                form.init().await?;
                self.patient_form_component = Some(Box::new(form));
                self.showing_form = true;
            }
            Action::AppointmentCreate => {
                info!("Opening appointment creation form");
                let mut form = AppointmentFormComponent::new(
                    self.appointment_service.clone(),
                    self.patient_service.clone(),
                );
                form.init().await?;
                self.appointment_form_component = Some(Box::new(form));
                self.showing_form = true;
            }
            Action::PatientFormCancel => {
                info!("Patient form cancelled");
                self.showing_form = false;
                self.patient_form_component = None;
            }
            Action::AppointmentFormCancel => {
                info!("Appointment form cancelled");
                self.showing_form = false;
                self.appointment_form_component = None;
            }
            _ => {
                let component = if self.showing_form {
                    if let Some(form) = self.patient_form_component.as_mut() {
                        Some(form)
                    } else {
                        self.appointment_form_component.as_mut()
                    }
                } else {
                    self.get_active_component_mut()
                };
                
                if let Some(comp) = component {
                    if let Some(new_action) = comp.update(action).await? {
                        if new_action == Action::PatientFormSubmit {
                            info!("Patient form submitted successfully");
                            self.showing_form = false;
                            self.patient_form_component = None;
                            if let Some(list_component) = &mut self.patient_component {
                                list_component.init().await?;
                            }
                        } else if new_action == Action::AppointmentFormSubmit {
                            info!("Appointment form submitted successfully");
                            self.showing_form = false;
                            self.appointment_form_component = None;
                            if let Some(list_component) = &mut self.appointment_component {
                                list_component.init().await?;
                            }
                        } else {
                            self.action_tx.send(new_action)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Render the application UI
    fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(size);
        
        self.render_header(frame, chunks[0]);
        self.render_content(frame, chunks[1]);
    }

    /// Render header with navigation tabs
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let screens = Screen::all();
        let titles: Vec<&str> = screens.iter().map(|s| s.as_str()).collect();
        
        let selected = match self.active_screen {
            Screen::Patients => 0,
            Screen::Appointments => 1,
            Screen::Clinical => 2,
            Screen::Billing => 3,
        };
        
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("OpenGP"))
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        
        frame.render_widget(tabs, area);
    }

    /// Render active component content
    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        if self.showing_form {
            if let Some(form) = &mut self.patient_form_component {
                form.render(frame, area);
            } else if let Some(form) = &mut self.appointment_form_component {
                form.render(frame, area);
            }
        } else if let Some(component) = self.get_active_component_mut() {
            component.render(frame, area);
        } else {
            self.render_placeholder(frame, area);
        }
    }

    /// Render placeholder when component not implemented
    fn render_placeholder(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "{} Screen\n\n\
            Component not yet implemented\n\n\
            Controls:\n\
              1-4: Switch screens\n\
              q or Ctrl+C: Quit",
            self.active_screen.as_str()
        );
        
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", self.active_screen.as_str())),
            )
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(paragraph, area);
    }

    /// Get mutable reference to active component
    fn get_active_component_mut(&mut self) -> Option<&mut Box<dyn Component>> {
        match self.active_screen {
            Screen::Patients => self.patient_component.as_mut(),
            Screen::Appointments => self.appointment_component.as_mut(),
            Screen::Clinical => self.clinical_component.as_mut(),
            Screen::Billing => self.billing_component.as_mut(),
        }
    }

    /// Signal the application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get reference to database pool
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }
}
