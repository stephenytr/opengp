pub mod appointment;
pub mod audit;
pub mod clinical;
pub mod patient;
pub mod practitioner;
pub mod user;

pub use appointment::SqlxAppointmentRepository;
pub use audit::SqlxAuditRepository;
pub use clinical::{
    SqlxAllergyRepository, SqlxClinicalRepository, SqlxFamilyHistoryRepository,
    SqlxMedicalHistoryRepository, SqlxSocialHistoryRepository, SqlxVitalSignsRepository,
};
pub use patient::SqlxPatientRepository;
pub use practitioner::SqlxPractitionerRepository;
pub use user::SqlxUserRepository;
