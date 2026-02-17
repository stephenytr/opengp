//! OpenGP - Open Source General Practice Management Software
//!
//! This library provides the core functionality for OpenGP, a terminal-based
//! general practice management system designed for Australian healthcare providers.

// Top-level modules
pub mod config;
pub mod error;

// Layer modules
pub mod components;
pub mod domain;
pub mod infrastructure;
pub mod integrations;
pub mod ui;


pub use config::Config;
pub use error::Error;
