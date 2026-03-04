//! OpenGP TUI Module
//!
//! Terminal User Interface for OpenGP using ratatui and ratatui-interact.
//! Provides patient management, appointment scheduling, clinical notes,
//! and billing features in a terminal-based interface.

pub mod app;
pub mod components;
pub mod error;
pub mod input;
pub mod keybinds;
pub mod layout;
pub mod services;
pub mod theme;
pub mod view_models;
pub mod widgets;

#[cfg(test)]
mod tests;

pub use app::App;
pub use components::tabs::Tab;
pub use error::{UiComponent, UiError};
pub use keybinds::{Action, KeybindRegistry};
pub use theme::{ColorPalette, Theme};
pub use widgets::{FieldType, FormField, FormFieldState, LoadingIndicator, SpinnerStyle};
