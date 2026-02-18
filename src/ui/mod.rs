//! OpenGP TUI Module
//!
//! Terminal User Interface for OpenGP using ratatui and ratatui-interact.
//! Provides patient management, appointment scheduling, clinical notes,
//! and billing features in a terminal-based interface.

pub mod components;
pub mod keybinds;
pub mod theme;

pub use components::tabs::Tab;
pub use keybinds::{Action, KeybindRegistry};
pub use theme::{ColorPalette, Theme};
