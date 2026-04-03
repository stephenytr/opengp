// OpenGP Infrastructure Layer
// Database, authentication, encryption implementations

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod infrastructure;

// Re-export all infrastructure modules
pub use infrastructure::audit;
pub use infrastructure::auth;
pub use infrastructure::crypto;
pub use infrastructure::database;
pub use infrastructure::fixtures;
pub use infrastructure::mbs;
