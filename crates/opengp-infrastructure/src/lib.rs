// OpenGP Infrastructure Layer
// Database, authentication, encryption implementations

pub mod infrastructure;

// Re-export all infrastructure modules
pub use infrastructure::audit;
pub use infrastructure::auth;
pub use infrastructure::crypto;
pub use infrastructure::database;
pub use infrastructure::fixtures;
