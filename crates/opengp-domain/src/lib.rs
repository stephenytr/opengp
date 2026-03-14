// OpenGP Domain Layer
// Pure business logic with zero infrastructure dependencies

pub mod domain;

// Re-export all domain modules
pub use domain::api;
pub use domain::appointment;
pub use domain::audit;
#[cfg(feature = "billing")]
pub use domain::billing;
pub use domain::clinical;
pub use domain::error;
#[cfg(feature = "immunisation")]
pub use domain::immunisation;
pub use domain::macros;
#[cfg(feature = "pathology")]
pub use domain::pathology;
pub use domain::patient;
#[cfg(feature = "prescription")]
pub use domain::prescription;
#[cfg(feature = "referral")]
pub use domain::referral;
pub use domain::user;

// Re-export commonly used types
pub use domain::error::RepositoryError;
