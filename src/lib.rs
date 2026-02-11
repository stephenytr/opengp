//! OpenGP - Open Source General Practice Management Software
//!
//! This library provides the core functionality for OpenGP, a terminal-based
//! general practice management system designed for Australian healthcare providers.

// Top-level modules
pub mod app;
pub mod config;
pub mod error;

// Layer modules
pub mod ui;
pub mod components;
pub mod domain;
pub mod infrastructure;
pub mod integrations;

// Re-exports for convenience
pub use app::App;
pub use config::Config;
pub use error::Error;
