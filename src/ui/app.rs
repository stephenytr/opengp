use std::sync::Arc;
use std::time::Duration;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use sqlx::SqlitePool;
use tuirealm::{Application, EventListenerCfg, Frame, NoUserEvent, PollStrategy};

use crossterm::event::{Event, poll};

use crate::config::Config;
use crate::domain::appointment::{
    AppointmentCalendarQuery, AppointmentRepository, AppointmentService,
};
use crate::domain::audit::{AuditRepository, AuditService};
use crate::domain::clinical::ClinicalService;
use crate::domain::patient::{PatientRepository, PatientService};
use crate::domain::user::{PractitionerRepository, PractitionerService};
use crate::error::Result;
use crate::infrastructure::crypto::EncryptionService;
use crate::infrastructure::database::repositories::{
    SqlxAllergyRepository, SqlxAppointmentRepository, SqlxAuditRepository, SqlxClinicalRepository,
    SqlxFamilyHistoryRepository, SqlxMedicalHistoryRepository, SqlxPatientRepository,
    SqlxPractitionerRepository, SqlxSocialHistoryRepository, SqlxVitalSignsRepository,
};

use super::component_id::Id;
use super::components::{RealmPatientList, RealmPatientForm, RealmTabs};
use super::event_dispatcher::EventDispatcher;
use super::msg::Msg;
use super::tui::Tui;

use crate::domain::patient::Patient;

pub struct App {
    inner: Application<Id, Msg, NoUserEvent>,
    services: Services,
    pub should_quit: bool,
    pub active_screen: Screen,
    pub tabs: RealmTabs,
    patients: Vec<Patient>,
    pub patient_list: RealmPatientList,
    pub show_patient_form: bool,
    patient_form: Option<RealmPatientForm>,
}

pub struct Services {
    pub patient_service: Arc<PatientService>,
    pub appointment_service: Arc<AppointmentService>,
    pub practitioner_service: Arc<PractitionerService>,
    pub clinical_service: Arc<ClinicalService>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Patients,
    Appointments,
    Clinical,
    Billing,
}

impl Screen {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Screen::Patients,
            1 => Screen::Appointments,
            2 => Screen::Clinical,
            _ => Screen::Billing,
        }
    }
}

impl App {
    pub fn new(_config: Config, db_pool: SqlitePool) -> Result<Self> {
        let services = Self::init_services(db_pool)?;

        let inner = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(1), 3)
                .poll_timeout(Duration::from_millis(1)),
        );

        Ok(Self {
            inner,
            services,
            should_quit: false,
            active_screen: Screen::Patients,
            tabs: RealmTabs::builder()
                .titles(vec!["Patients", "Appointments", "Clinical", "Billing"])
                .selected(0)
                .build(),
            patients: Vec::new(),
            patient_list: RealmPatientList::builder()
                .with_keybinds()
                .build(),
            show_patient_form: false,
            patient_form: None,
        })
    }

    fn init_services(db_pool: SqlitePool) -> Result<Services> {
        let crypto = Arc::new(EncryptionService::new()?);

        let patient_repository: Arc<dyn PatientRepository> =
            Arc::new(SqlxPatientRepository::new(db_pool.clone(), crypto.clone()));
        let patient_service = Arc::new(PatientService::new(patient_repository));

        let audit_repository: Arc<dyn AuditRepository> =
            Arc::new(SqlxAuditRepository::new(db_pool.clone()));
        let audit_service = Arc::new(AuditService::new(audit_repository));

        let appointment_repo = Arc::new(SqlxAppointmentRepository::new(db_pool.clone()));
        let appointment_repository: Arc<dyn AppointmentRepository> = appointment_repo.clone();
        let appointment_calendar_query: Arc<dyn AppointmentCalendarQuery> = appointment_repo;
        let appointment_service = Arc::new(AppointmentService::new(
            appointment_repository,
            audit_service.clone(),
            appointment_calendar_query,
        ));

        let practitioner_repository: Arc<dyn PractitionerRepository> =
            Arc::new(SqlxPractitionerRepository::new(db_pool.clone()));
        let practitioner_service = Arc::new(PractitionerService::new(practitioner_repository));

        let consultation_repo: Arc<dyn crate::domain::clinical::ConsultationRepository> =
            Arc::new(SqlxClinicalRepository::new(db_pool.clone(), crypto.clone()));
        let allergy_repo: Arc<dyn crate::domain::clinical::AllergyRepository> =
            Arc::new(SqlxAllergyRepository::new(db_pool.clone(), crypto.clone()));
        let medical_history_repo: Arc<dyn crate::domain::clinical::MedicalHistoryRepository> =
            Arc::new(SqlxMedicalHistoryRepository::new(db_pool.clone(), crypto.clone()));
        let vital_signs_repo: Arc<dyn crate::domain::clinical::VitalSignsRepository> = Arc::new(
            SqlxVitalSignsRepository::new(db_pool.clone(), crypto.clone()),
        );
        let social_history_repo: Arc<dyn crate::domain::clinical::SocialHistoryRepository> =
            Arc::new(SqlxSocialHistoryRepository::new(db_pool.clone(), crypto.clone()));
        let family_history_repo: Arc<dyn crate::domain::clinical::FamilyHistoryRepository> =
            Arc::new(SqlxFamilyHistoryRepository::new(db_pool.clone(), crypto.clone()));

        let clinical_service = Arc::new(ClinicalService::new(
            consultation_repo,
            allergy_repo,
            medical_history_repo,
            vital_signs_repo,
            social_history_repo,
            family_history_repo,
            patient_service.clone(),
            audit_service,
            crypto,
        ));

        Ok(Services {
            patient_service,
            appointment_service,
            practitioner_service,
            clinical_service,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        self.init_components().await?;

        while !self.should_quit {
            tui.draw(|f| self.render(f))?;

            while let Ok(msgs) = self.inner.tick(PollStrategy::UpTo(1)) {
                if msgs.is_empty() {
                    break;
                }
                for msg in msgs {
                    self.handle_msg(msg);
                }
            }

            if self.should_quit {
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    fn handle_msg(&mut self, msg: Msg) {
        let mut m = Some(msg);
        while m.is_some() {
            m = self.update(m);
        }
    }

    pub fn handle_patient_create(&mut self) {
        let form = RealmPatientForm::builder()
            .with_keybinds()
            .build();
        self.inner
            .mount(Id::PatientForm, Box::new(form.clone()), vec![])
            .ok();
        self.show_patient_form = true;
        self.patient_form = Some(form);
        self.update_focus();
    }

    pub fn handle_patient_edit(&mut self, patient_id: uuid::Uuid) {
        if let Some(patient) = self.patients.iter().find(|p| p.id == patient_id) {
            let form = RealmPatientForm::builder()
                .patient(patient.clone())
                .with_keybinds()
                .build();
            self.inner
                .mount(Id::PatientForm, Box::new(form.clone()), vec![])
                .ok();
            self.show_patient_form = true;
            self.patient_form = Some(form);
            self.update_focus();
        }
    }

    pub fn handle_patient_form_cancel(&mut self) {
        self.inner.umount(&Id::PatientForm).ok();
        self.show_patient_form = false;
        self.patient_form = None;
        self.update_focus();
    }

    pub fn handle_patient_form_submit(&mut self) {
        self.handle_patient_form_cancel();
    }

    async fn init_components(&mut self) -> Result<()> {
        // Mount the tabs component
        self.inner
            .mount(Id::Navigation, Box::new(self.tabs.clone()), vec![])
            .ok();

        // Load patients from service
        self.patients = self.services.patient_service.list_active_patients().await.unwrap_or_default();
        self.patient_list.update_patients(self.patients.clone());

        // Mount patient list component
        self.inner
            .mount(Id::PatientList, Box::new(self.patient_list.clone()), vec![])
            .ok();

        // Set focus based on active screen
        self.update_focus();

        Ok(())
    }

    pub fn update_focus(&mut self) {
        match self.active_screen {
            Screen::Patients => {
                if self.show_patient_form {
                    self.inner.active(&Id::PatientForm).ok();
                } else {
                    self.inner.active(&Id::PatientList).ok();
                }
            }
            _ => {
                self.inner.active(&Id::Navigation).ok();
            }
        }
    }

    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        match msg? {
            Msg::AppClose => {
                self.should_quit = true;
                None
            }
            Msg::NavigateTo(target) => {
                self.handle_navigation(target);
                None
            }
            Msg::NavigateToTab(index) => {
                self.active_screen = Screen::from_index(index);
                self.update_focus();
                None
            }
            Msg::PatientCreate => {
                let form = RealmPatientForm::builder()
                    .with_keybinds()
                    .build();
                self.inner
                    .mount(Id::PatientForm, Box::new(form.clone()), vec![])
                    .ok();
                self.show_patient_form = true;
                self.patient_form = Some(form);
                self.update_focus();
                Some(Msg::Render)
            }
            Msg::PatientEdit(patient_id) => {
                if let Some(patient) = self.patients.iter().find(|p| p.id == patient_id) {
                    let form = RealmPatientForm::builder()
                        .patient(patient.clone())
                        .with_keybinds()
                        .build();
                    self.inner
                        .mount(Id::PatientForm, Box::new(form.clone()), vec![])
                        .ok();
                    self.show_patient_form = true;
                    self.patient_form = Some(form);
                    self.update_focus();
                }
                Some(Msg::Render)
            }
            Msg::PatientFormCancel => {
                self.inner.umount(&Id::PatientForm).ok();
                self.show_patient_form = false;
                self.patient_form = None;
                self.update_focus();
                Some(Msg::Render)
            }
            Msg::PatientSelected(_) => {
                Some(Msg::Render)
            }
            _ => None,
        }
    }

    fn handle_navigation(&mut self, target: super::msg::NavigationTarget) {
        match target {
            super::msg::NavigationTarget::Patients => {
                self.active_screen = Screen::Patients;
            }
            super::msg::NavigationTarget::Appointments => {
                self.active_screen = Screen::Appointments;
            }
            super::msg::NavigationTarget::Clinical => {
                self.active_screen = Screen::Clinical;
            }
            super::msg::NavigationTarget::Billing => {
                self.active_screen = Screen::Billing;
            }
            super::msg::NavigationTarget::PatientForm(_) => {}
            super::msg::NavigationTarget::AppointmentForm(_) => {}
            super::msg::NavigationTarget::ClinicalWithPatient(_) => {
                self.active_screen = Screen::Clinical;
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        self.inner.view(&Id::Navigation, frame, chunks[0]);

        match self.active_screen {
            Screen::Patients => self.render_patients(frame, chunks[1]),
            Screen::Appointments => self.render_appointments(frame, chunks[1]),
            Screen::Clinical => self.render_clinical(frame, chunks[1]),
            Screen::Billing => self.render_billing(frame, chunks[1]),
        }
    }

    fn render_patients(&mut self, frame: &mut Frame, area: Rect) {
        if self.show_patient_form {
            // Render form in modal-like overlay
            let overlay_area = Rect {
                x: area.x + area.width / 4,
                y: area.y + area.height / 6,
                width: area.width / 2,
                height: area.height * 2 / 3,
            };
            self.inner.view(&Id::PatientForm, frame, overlay_area);
        } else {
            // Render patient list via tui-realm
            self.inner.view(&Id::PatientList, frame, area);
        }
    }

    fn render_appointments(&self, frame: &mut Frame, area: Rect) {
        let content = ratatui::widgets::Paragraph::new(
            r#"
[<]  February 2026  [>]  [Today: t]  [Calendar View: v]

Sun  | Mon  | Tue  | Wed  | Thu  | Fri  | Sat
-----|------|------|------|------|------|------
 1   |  2   |  3   |  4   |  5   |  6   |  7
     |      | 3ap  |      |      | 2ap  |      
-----|------|------|------|------|------|------
 8   |  9   | 10   | 11   | 12   | 13   | 14
     |      |      |      |      |      |      
-----|------|------|------|------|------|------
15   | 16   | 17   | 18   | 19   | 20   | 21
     |      | Today|      |      |      |      
-----|------|------|------|------|------|------
22   | 23   | 24   | 25   | 26   | 27   | 28
     |      |      |      |      |      |      

[n: New Appointment]  [Enter: View]  [f: Filter]
"#
            .trim(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Appointments ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

        frame.render_widget(content, area);
    }

    fn render_clinical(&self, frame: &mut Frame, area: Rect) {
        let content = ratatui::widgets::Paragraph::new(
            r#"
Patient: [Select Patient... ]

Tab: [Overview] [Consultations] [Allergies]

Patient Overview
----------------------------------------
Name:     John Smith
DOB:      15 March 1980 (45 years)
Gender:   Male
Phone:    0412 345 678
Address:  123 Main St, Sydney NSW 2000

----------------------------------------
Allergies: Penicillin (Severe)
Medications: Metformin 500mg, Amlodipine 5mg

[c: New Consultation]  [a: Add Allergy]  [v: Vitals]
"#
            .trim(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Clinical ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

        frame.render_widget(content, area);
    }

    fn render_billing(&self, frame: &mut Frame, area: Rect) {
        let content = ratatui::widgets::Paragraph::new(
            r#"
[Search: /]  [New Invoice: n]

ID    | Patient       | Date    | Amount | Status
------|---------------|---------|--------|--------
INV001| John Smith    | 17/02/26| $150.00| Paid
INV002| Jane Doe      | 16/02/26| $85.00 | Pending
INV003| Bob Johnson   | 15/02/26| $220.00| Paid
INV004| Mary Williams | 14/02/26| $95.00 | Overdue

Summary: Total: $550.00  Paid: $370.00  Pending: $180.00

[Enter: View Invoice]  [p: Process Payment]  [x: Export]
"#
            .trim(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Billing ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

        frame.render_widget(content, area);
    }

    pub fn services(&self) -> &Services {
        &self.services
    }
}