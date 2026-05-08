mod app_state;
mod error;
mod event;
mod global;

pub use app_state::AppState;
pub use error::AppError;
pub use event::AppEvent;
pub use global::{AppContextMenuAction, DialogContent, GlobalState, RetryOperation};

const DEFAULT_PATIENT_PAGE_LIMIT: u32 = 100;
const DEFAULT_APPOINTMENT_PAGE_LIMIT: u32 = 100;
const DEFAULT_CONSULTATION_PAGE_LIMIT: u32 = 100;

#[derive(Debug)]
pub struct ClinicalWorkspaceLoadResult {
    pub patient_id: uuid::Uuid,
    pub allergies:
        Result<Vec<opengp_domain::domain::clinical::Allergy>, crate::api::ApiClientError>,
    pub medical_history:
        Result<Vec<opengp_domain::domain::clinical::MedicalHistory>, crate::api::ApiClientError>,
    pub vitals:
        Result<Vec<opengp_domain::domain::clinical::VitalSigns>, crate::api::ApiClientError>,
    pub social_history:
        Result<opengp_domain::domain::clinical::SocialHistory, crate::api::ApiClientError>,
    pub family_history:
        Result<Vec<opengp_domain::domain::clinical::FamilyHistory>, crate::api::ApiClientError>,
    pub consultations:
        Result<Vec<opengp_domain::domain::clinical::Consultation>, crate::api::ApiClientError>,
}

pub enum ApiTaskError {
    Unauthorized,
    ServerUnavailable(String),
    Message(String),
}

impl ApiTaskError {
    fn from_client_error(error: crate::api::ApiClientError, context: &str) -> Self {
        match error {
            crate::api::ApiClientError::Unauthorized => Self::Unauthorized,
            crate::api::ApiClientError::ServerUnavailable(message) => {
                Self::ServerUnavailable(format!("{}: {}", context, message))
            }
            other => Self::Message(format!("{}: {}", context, other)),
        }
    }

    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub enum PendingPatientData {
    New(opengp_domain::domain::patient::NewPatientData),
    Update {
        id: uuid::Uuid,
        data: opengp_domain::domain::patient::UpdatePatientData,
    },
}
