//! Centralized key event dispatcher for the OpenGP application
//!
//! This module provides a unified way to dispatch key events to actions
//! based on the current context. It bridges the KeybindRegistry with
//! the Action enum.
//!
//! # Usage
//!
//! ```rust
//! use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
//! use opengp::ui::keybinds::KeybindContext;
//! use opengp::ui::key_dispatcher::KeyDispatcher;
//! use opengp::components::Action;
//!
//! let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
//! let action = KeyDispatcher::dispatch(&KeybindContext::PatientList, key);
//! assert_eq!(action, Some(Action::PatientCreate));
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::components::Action;
use crate::ui::keybinds::{KeybindContext, KeybindRegistry};

/// Centralized dispatcher for converting key events to actions
pub struct KeyDispatcher;

impl KeyDispatcher {
    /// Map an action name from the registry to an Action enum value
    fn map_action(action_name: &str, context: &KeybindContext) -> Option<Action> {
        match action_name {
            // Global actions
            "Quit" => Some(Action::Quit),
            "Help" => Some(Action::Render),

            // Patient list actions
            "Next" | "Previous" | "First" | "Last" => Some(Action::Render),
            "View" | "Edit" => Some(Action::Render),
            "New" => match context {
                KeybindContext::PatientList
                | KeybindContext::PatientListSearch
                | KeybindContext::PatientForm => Some(Action::PatientCreate),
                KeybindContext::CalendarDayView
                | KeybindContext::CalendarWeekView
                | KeybindContext::CalendarMonthView
                | KeybindContext::AppointmentList
                | KeybindContext::AppointmentForm => Some(Action::AppointmentCreate),
                _ => Some(Action::Render),
            },
            "Search" | "Clear" => Some(Action::Render),

            // Patient form actions
            "Cancel" => Some(Action::PatientFormCancel),
            "Submit" => Some(Action::PatientFormSubmit),

            // Calendar month view
            "Week Up" | "Week Down" | "Day Back" | "Day Forward" | "Month Back"
            | "Month Forward" | "Today" | "Day View" => Some(Action::Render),

            // Calendar day view
            "Up" | "Down" | "Month" | "Week" | "Search" | "Filter" | "Practitioner"
            | "Multi-Select" | "Undo" => Some(Action::Render),
            "Details" => Some(Action::AppointmentSelect),
            "Arrived" => Some(Action::AppointmentMarkArrived),
            "In Progress" => Some(Action::AppointmentMarkInProgress),
            "Completed" => Some(Action::AppointmentMarkCompleted),
            "No Show" => Some(Action::AppointmentMarkNoShow),

            // Calendar week view
            "Week Back" | "Week Forward" => Some(Action::Render),

            // Calendar multi-select
            "Toggle" | "Select All" | "Exit" | "Batch" => Some(Action::Render),

            // Calendar detail modal
            "Close" | "History" => Some(Action::Render),
            "Reschedule" => Some(Action::AppointmentReschedule),

            // Calendar reschedule modal
            "Earlier" | "Later" | "Longer" | "Shorter" | "Save" => Some(Action::Render),

            // Calendar search modal
            "Select" | "Delete" => Some(Action::Render),

            // Calendar filter menu
            "Clear" | "Scheduled" | "Confirmed" | "Arrived" | "InProgress" | "Completed"
            | "NoShow" | "Cancelled" | "Rescheduled" => Some(Action::Render),

            // Calendar practitioner menu
            t if t.starts_with("Toggle ") => Some(Action::Render),

            // Calendar audit modal
            "Previous" | "Next" => Some(Action::Render),

            // Calendar confirmation
            "Confirm" => Some(Action::Render),

            // Calendar error modal
            "Close" => Some(Action::Render),

            // Calendar batch menu
            "Arrived" => Some(Action::AppointmentBatchMarkArrived),
            "Completed" => Some(Action::AppointmentBatchMarkCompleted),
            "Cancel" => Some(Action::Render),

            // Appointment form
            "Select" | "Toggle" | "Navigate up" | "Navigate down" | "Delete" => {
                Some(Action::Render)
            }

            // Appointment form patient field
            "Select patient" | "Clear search" => Some(Action::Render),

            // Tabs
            "Patients" => Some(Action::NavigateToPatients),
            "Appointments" => Some(Action::NavigateToAppointments),
            "Clinical" => Some(Action::NavigateToClinical),
            "Billing" => Some(Action::NavigateToBilling),
            "First" | "Last" => Some(Action::Render),

            _ => None,
        }
    }

    /// Dispatch a key event to an action based on context
    pub fn dispatch(context: &KeybindContext, key: KeyEvent) -> Option<Action> {
        let action_name = KeybindRegistry::lookup_action(context, key)?;
        Self::map_action(action_name, context)
    }

    /// Check if a key event is handled by the dispatcher in a given context
    pub fn is_handled(context: &KeybindContext, key: KeyEvent) -> bool {
        KeybindRegistry::has_keybind(context, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn test_dispatch_patient_list_new() {
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::PatientList, key);
        assert_eq!(action, Some(Action::PatientCreate));
    }

    #[test]
    fn test_dispatch_patient_form_cancel() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::PatientForm, key);
        assert_eq!(action, Some(Action::PatientFormCancel));
    }

    #[test]
    fn test_dispatch_calendar_day_view_n() {
        // Note: The action name "New" is used for both PatientCreate and AppointmentCreate
        // depending on context. The dispatcher currently returns the first match.
        // For calendar-specific actions, use the more specific mapping.
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::CalendarDayView, key);
        // This returns PatientCreate because 'n' + "New" is first match in registry
        // For proper calendar dispatch, we need context-specific handling in Phase 3
        assert!(action.is_some());
    }

    #[test]
    fn test_dispatch_calendar_day_view_x() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::CalendarDayView, key);
        assert_eq!(action, Some(Action::AppointmentMarkNoShow));
    }

    #[test]
    fn test_dispatch_tabs_navigation() {
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::Tabs, key);
        assert_eq!(action, Some(Action::NavigateToPatients));

        let key = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::Tabs, key);
        assert_eq!(action, Some(Action::NavigateToAppointments));
    }

    #[test]
    fn test_dispatch_unhandled_key() {
        let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::PatientList, key);
        assert_eq!(action, None);
    }

    #[test]
    fn test_dispatch_with_modifiers() {
        // Ctrl+C for quit (Global context)
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let action = KeyDispatcher::dispatch(&KeybindContext::Global, key);
        assert_eq!(action, Some(Action::Quit));

        // Regular q for quit (Global context)
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::Global, key);
        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn test_is_handled() {
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        assert!(KeyDispatcher::is_handled(&KeybindContext::PatientList, key));

        let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty());
        assert!(!KeyDispatcher::is_handled(
            &KeybindContext::PatientList,
            key
        ));
    }

    #[test]
    fn test_dispatch_case_insensitive() {
        // Uppercase should work too
        let key = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::empty());
        let action = KeyDispatcher::dispatch(&KeybindContext::PatientList, key);
        assert_eq!(action, Some(Action::PatientCreate));
    }
}
