// OpenGP Domain Layer
// Pure business logic with zero infrastructure dependencies

pub mod domain;

// Re-export all domain modules
pub use domain::appointment;
pub use domain::audit;
pub use domain::billing;
pub use domain::clinical;
pub use domain::error;
pub use domain::immunisation;
pub use domain::macros;
pub use domain::pathology;
pub use domain::patient;
pub use domain::prescription;
pub use domain::referral;
pub use domain::user;

// Re-export commonly used types
pub use domain::error::RepositoryError;
