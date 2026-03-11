// OpenGP UI Layer
// Terminal User Interface using Ratatui

pub mod ui;
pub mod api;

// Re-export all UI modules
pub use ui::app;
pub use api::*;
pub use ui::components;
pub use ui::error;
pub use ui::input;
pub use ui::keybinds;
pub use ui::layout;
pub use ui::services;
pub use ui::theme;
pub use ui::view_models;
pub use ui::widgets;

// Re-export commonly used types
pub use ui::app::App;
pub use ui::components::tabs::Tab;
pub use ui::error::{UiComponent, UiError};
pub use ui::keybinds::{Action, KeybindRegistry};
pub use ui::theme::{ColorPalette, Theme};
pub use ui::widgets::{FieldType, FormField, FormFieldState, LoadingIndicator, SpinnerStyle};
