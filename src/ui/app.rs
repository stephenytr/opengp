use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tuirealm::{
    Application, EventListenerCfg, Frame, NoUserEvent, PollStrategy,
};

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
use super::msg::Msg;
use super::tui::Tui;

pub struct App {
    inner: Application<Id, Msg, NoUserEvent>,
    services: Services,
    should_quit: bool,
    active_screen: Screen,
}

pub struct Services {
    pub patient_service: Arc<PatientService>,
    pub appointment_service: Arc<AppointmentService>,
    pub practitioner_service: Arc<PractitionerService>,
    pub clinical_service: Arc<ClinicalService>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Patients,
    Appointments,
    Clinical,
    Billing,
}

impl App {
    pub fn new(_config: Config, db_pool: SqlitePool) -> Result<Self> {
        let services = Self::init_services(db_pool)?;

        let inner = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(20), 3)
                .poll_timeout(Duration::from_millis(10)),
        );

        Ok(Self {
            inner,
            services,
            should_quit: false,
            active_screen: Screen::Patients,
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

            if let Ok(msgs) = self.inner.tick(PollStrategy::Once) {
                for msg in msgs {
                    let mut m = Some(msg);
                    while m.is_some() {
                        m = self.update(m);
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    async fn init_components(&mut self) -> Result<()> {
        Ok(())
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
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("Screen: {:?}", self.active_screen)),
            area,
        );
    }

    pub fn services(&self) -> &Services {
        &self.services
    }
}