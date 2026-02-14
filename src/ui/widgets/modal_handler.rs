//! Modal state management for UI components
//!
//! This module provides a centralized pattern for managing modal states in TUI components.
//! Instead of scattered boolean flags, use `ModalState` to track which modal (if any) is active.
//!
//! # Example
//!
//! ```rust
//! use crate::ui::widgets::modal_handler::{ModalState, ModalHandler};
//!
//! struct MyComponent {
//!     modal_state: ModalState,
//! }
//!
//! impl ModalHandler for MyComponent {
//!     fn get_modal_state(&self) -> &ModalState {
//!         &self.modal_state
//!     }
//!
//!     fn get_modal_state_mut(&mut self) -> &mut ModalState {
//!         &mut self.modal_state
//!     }
//! }
//!
//! // Usage
//! fn handle_key_event(component: &mut MyComponent, key: KeyCode) {
//!     match key {
//!         KeyCode::Char('h') => component.show_modal(ModalType::Help),
//!         KeyCode::Char('d') => component.show_modal(ModalType::Detail),
//!         KeyCode::Esc => component.hide_modal(),
//!         _ => {}
//!     }
//! }
//! ```

use std::fmt;

/// Represents the type of modal being displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalType {
    /// No modal is active
    #[default]
    None,
    /// Help/shortcuts modal
    Help,
    /// Detail view modal (e.g., appointment details, patient details)
    Detail,
    /// Search modal for finding items
    Search,
    /// Confirmation modal (e.g., confirm status change, confirm delete)
    Confirmation,
    /// Error modal displaying an error message
    Error,
    /// Reschedule modal for changing appointment times
    Reschedule,
    /// Filter menu modal
    Filter,
    /// Practitioner selection modal
    Practitioner,
    /// Audit history modal
    Audit,
    /// Batch operations modal
    Batch,
}

impl fmt::Display for ModalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModalType::None => write!(f, "None"),
            ModalType::Help => write!(f, "Help"),
            ModalType::Detail => write!(f, "Detail"),
            ModalType::Search => write!(f, "Search"),
            ModalType::Confirmation => write!(f, "Confirmation"),
            ModalType::Error => write!(f, "Error"),
            ModalType::Reschedule => write!(f, "Reschedule"),
            ModalType::Filter => write!(f, "Filter"),
            ModalType::Practitioner => write!(f, "Practitioner"),
            ModalType::Audit => write!(f, "Audit"),
            ModalType::Batch => write!(f, "Batch"),
        }
    }
}

/// Tracks which modal is currently active and associated data
///
/// This enum replaces scattered boolean flags like `showing_detail_modal`,
/// `showing_search_modal`, etc. that were duplicated across components.
///
/// # Example
///
/// ```rust
/// use crate::ui::widgets::modal_handler::ModalState;
///
/// let state = ModalState::none();
/// assert!(state.is_active());
/// assert!(state.active_type().is_none());
///
/// let state = ModalState::active(ModalType::Help);
/// assert!(state.is_active());
/// assert_eq!(state.active_type(), ModalType::Help);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModalState {
    modal_type: ModalType,
}

impl ModalState {
    /// Create a new ModalState with no active modal
    ///
    /// # Example
    ///
    /// ```rust
    /// let state = ModalState::none();
    /// assert!(!state.is_active());
    /// ```
    pub fn none() -> Self {
        Self {
            modal_type: ModalType::None,
        }
    }

    /// Create a new ModalState with the specified modal active
    ///
    /// # Example
    ///
    /// ```rust
    /// let state = ModalState::active(ModalType::Detail);
    /// assert!(state.is_active());
    /// ```
    pub fn active(modal_type: ModalType) -> Self {
        Self { modal_type }
    }

    /// Check if any modal is currently active
    ///
    /// # Example
    ///
    /// ```rust
    /// let state = ModalState::none();
    /// assert!(!state.is_active());
    ///
    /// let state = ModalState::active(ModalType::Search);
    /// assert!(state.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.modal_type != ModalType::None
    }

    /// Get the type of the currently active modal
    ///
    /// # Example
    ///
    /// ```rust
    /// let state = ModalState::active(ModalType::Error);
    /// assert_eq!(state.active_type(), ModalType::Error);
    /// ```
    pub fn active_type(&self) -> ModalType {
        self.modal_type
    }

    /// Show a modal of the specified type
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut state = ModalState::none();
    /// state.show(ModalType::Help);
    /// assert!(state.is_active());
    /// assert_eq!(state.active_type(), ModalType::Help);
    /// ```
    pub fn show(&mut self, modal_type: ModalType) {
        self.modal_type = modal_type;
    }

    /// Hide the current modal (set to None)
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut state = ModalState::active(ModalType::Detail);
    /// state.hide();
    /// assert!(!state.is_active());
    /// ```
    pub fn hide(&mut self) {
        self.modal_type = ModalType::None;
    }

    /// Toggle a modal - show if hidden, hide if shown
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut state = ModalState::none();
    /// state.toggle(ModalType::Search);
    /// assert!(state.is_active());
    ///
    /// state.toggle(ModalType::Search);
    /// assert!(!state.is_active());
    /// ```
    pub fn toggle(&mut self, modal_type: ModalType) {
        if self.modal_type == modal_type {
            self.modal_type = ModalType::None;
        } else {
            self.modal_type = modal_type;
        }
    }

    /// Check if a specific modal type is currently active
    ///
    /// # Example
    ///
    /// ```rust
    /// let state = ModalState::active(ModalType::Detail);
    /// assert!(state.is_showing(ModalType::Detail));
    /// assert!(!state.is_showing(ModalType::Search));
    /// ```
    pub fn is_showing(&self, modal_type: ModalType) -> bool {
        self.modal_type == modal_type
    }
}

/// Trait for components that manage modal states
///
/// Implement this trait to get convenient helper methods for managing modal state.
/// The trait provides a default implementation that works with a `ModalState` field.
///
/// # Example
///
/// ```rust
/// use crate::ui::widgets::modal_handler::{ModalHandler, ModalState, ModalType};
///
/// struct MyComponent {
///     modal_state: ModalState,
/// }
///
/// impl ModalHandler for MyComponent {
///     fn get_modal_state(&self) -> &ModalState {
///         &self.modal_state
///     }
///
///     fn get_modal_state_mut(&mut self) -> &mut ModalState {
///         &mut self.modal_state
///     }
/// }
/// ```
pub trait ModalHandler: Sized {
    /// Get immutable reference to the modal state
    fn get_modal_state(&self) -> &ModalState;

    /// Get mutable reference to the modal state
    fn get_modal_state_mut(&mut self) -> &mut ModalState;

    /// Show a modal of the specified type
    ///
    /// # Example
    ///
    /// ```rust
    /// # use crate::ui::widgets::modal_handler::{ModalHandler, ModalType};
    /// # struct Example { modal_state: crate::ui::widgets::modal_handler::ModalState }
    /// # impl ModalHandler for Example {
    /// #     fn get_modal_state(&self) -> &crate::ui::widgets::modal_handler::ModalState { &self.modal_state }
    /// #     fn get_modal_state_mut(&mut self) -> &mut crate::ui::widgets::modal_handler::ModalState { &mut self.modal_state }
    /// # }
    /// let mut component = Example { modal_state: ModalState::none() };
    /// component.show_modal(ModalType::Help);
    /// assert!(component.is_modal_active());
    /// ```
    fn show_modal(&mut self, modal_type: ModalType) {
        self.get_modal_state_mut().show(modal_type);
    }

    /// Hide the current modal
    ///
    /// # Example
    ///
    /// ```rust
    /// # use crate::ui::widgets::modal_handler::{ModalHandler, ModalType};
    /// # struct Example { modal_state: crate::ui::widgets::modal_handler::ModalState }
    /// # impl ModalHandler for Example {
    /// #     fn get_modal_state(&self) -> &crate::ui::widgets::modal_handler::ModalState { &self.modal_state }
    /// #     fn get_modal_state_mut(&mut self) -> &mut crate::ui::widgets::modal_handler::ModalState { &mut self.modal_state }
    /// # }
    /// let mut component = Example { modal_state: ModalState::active(ModalType::Detail) };
    /// component.hide_modal();
    /// assert!(!component.is_modal_active());
    /// ```
    fn hide_modal(&mut self) {
        self.get_modal_state_mut().hide();
    }

    /// Toggle a modal type
    fn toggle_modal(&mut self, modal_type: ModalType) {
        self.get_modal_state_mut().toggle(modal_type);
    }

    /// Check if any modal is currently active
    ///
    /// # Example
    ///
    /// ```rust
    /// # use crate::ui::widgets::modal_handler::{ModalHandler, ModalType};
    /// # struct Example { modal_state: crate::ui::widgets::modal_handler::ModalState }
    /// # impl ModalHandler for Example {
    /// #     fn get_modal_state(&self) -> &crate::ui::widgets::modal_handler::ModalState { &self.modal_state }
    /// #     fn get_modal_state_mut(&mut self) -> &mut crate::ui::widgets::modal_handler::ModalState { &mut self.modal_state }
    /// # }
    /// let mut component = Example { modal_state: ModalState::none() };
    /// assert!(!component.is_modal_active());
    /// component.show_modal(ModalType::Search);
    /// assert!(component.is_modal_active());
    /// ```
    fn is_modal_active(&self) -> bool {
        self.get_modal_state().is_active()
    }

    /// Get the type of the currently active modal
    fn active_modal_type(&self) -> ModalType {
        self.get_modal_state().active_type()
    }

    /// Check if a specific modal type is showing
    fn is_showing_modal(&self, modal_type: ModalType) -> bool {
        self.get_modal_state().is_showing(modal_type)
    }

    /// Check if the help modal is showing
    fn is_showing_help(&self) -> bool {
        self.is_showing_modal(ModalType::Help)
    }

    /// Check if the detail modal is showing
    fn is_showing_detail(&self) -> bool {
        self.is_showing_modal(ModalType::Detail)
    }

    /// Check if the search modal is showing
    fn is_showing_search(&self) -> bool {
        self.is_showing_modal(ModalType::Search)
    }

    /// Check if the confirmation modal is showing
    fn is_showing_confirmation(&self) -> bool {
        self.is_showing_modal(ModalType::Confirmation)
    }

    /// Check if the error modal is showing
    fn is_showing_error(&self) -> bool {
        self.is_showing_modal(ModalType::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_state_none() {
        let state = ModalState::none();
        assert!(!state.is_active());
        assert_eq!(state.active_type(), ModalType::None);
    }

    #[test]
    fn test_modal_state_active() {
        let state = ModalState::active(ModalType::Detail);
        assert!(state.is_active());
        assert_eq!(state.active_type(), ModalType::Detail);
    }

    #[test]
    fn test_modal_state_show() {
        let mut state = ModalState::none();
        state.show(ModalType::Search);
        assert!(state.is_active());
        assert_eq!(state.active_type(), ModalType::Search);
    }

    #[test]
    fn test_modal_state_hide() {
        let mut state = ModalState::active(ModalType::Error);
        state.hide();
        assert!(!state.is_active());
    }

    #[test]
    fn test_modal_state_toggle() {
        let mut state = ModalState::none();

        // Toggle on
        state.toggle(ModalType::Batch);
        assert!(state.is_showing(ModalType::Batch));

        // Toggle off
        state.toggle(ModalType::Batch);
        assert!(!state.is_showing(ModalType::Batch));

        // Toggle different modal
        state.show(ModalType::Detail);
        state.toggle(ModalType::Search);
        assert!(state.is_showing(ModalType::Search));
    }

    #[test]
    fn test_modal_type_display() {
        assert_eq!(format!("{}", ModalType::None), "None");
        assert_eq!(format!("{}", ModalType::Help), "Help");
        assert_eq!(format!("{}", ModalType::Detail), "Detail");
    }

    #[test]
    fn test_modal_handler_trait() {
        struct TestComponent {
            modal_state: ModalState,
        }

        impl ModalHandler for TestComponent {
            fn get_modal_state(&self) -> &ModalState {
                &self.modal_state
            }

            fn get_modal_state_mut(&mut self) -> &mut ModalState {
                &mut self.modal_state
            }
        }

        let mut component = TestComponent {
            modal_state: ModalState::none(),
        };

        // Test is_modal_active
        assert!(!component.is_modal_active());

        // Test show_modal
        component.show_modal(ModalType::Help);
        assert!(component.is_modal_active());
        assert!(component.is_showing_help());

        // Test is_showing_modal
        assert!(component.is_showing_modal(ModalType::Help));
        assert!(!component.is_showing_modal(ModalType::Search));

        // Test hide_modal
        component.hide_modal();
        assert!(!component.is_modal_active());

        // Test toggle_modal
        component.toggle_modal(ModalType::Search);
        assert!(component.is_showing_search());
        component.toggle_modal(ModalType::Search);
        assert!(!component.is_showing_search());
    }
}
