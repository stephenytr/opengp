pub mod appointment;
pub mod audit;
pub mod patient;

pub use appointment::SqlxAppointmentRepository;
pub use audit::SqlxAuditRepository;
pub use patient::SqlxPatientRepository;
