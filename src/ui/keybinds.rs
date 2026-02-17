//! Centralized keybind registry for the OpenGP application
//!
//! This module provides a single source of truth for all keyboard bindings across
//! the application. It defines keybind contexts, keybind definitions, and helper
//! functions for generating help text.
//!
//! # Design Goals
//!
//! - **Single Source of Truth**: All keybinds defined in one place
//! - **Type Safety**: Context-aware keybind resolution
//! - **Help Text Generation**: Automatic help text for any context
//! - **Conflict Resolution**: Documented handling of context-specific bindings
//!
//! # Known Conflicts (Resolved)
//!
//! - **`n` key conflict**: In appointment calendar day view, `n` now always creates
//!   new appointments. Use `x` for "No Show" status (resolved as of audit 2026-02-13)
//! - **`Enter` in patient list**: Does nothing currently - reserved for future
//!   patient detail view implementation
//!
//! # Usage Example
//!
//! ```rust
//! use opengp::ui::keybinds::{KeybindRegistry, KeybindContext};
//!
//! let help_text = KeybindRegistry::get_help_text(KeybindContext::PatientList);
//! // Returns: "j/k/↑↓: Nav  g/G: First/Last  n: New  /: Search  Esc: Clear"
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Represents the different contexts where keybinds apply
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeybindContext {
    /// Global keybinds that work everywhere
    Global,

    /// Patient list screen (browsing patients)
    PatientList,

    /// Patient list in search mode
    PatientListSearch,

    /// Patient form (create/edit)
    PatientForm,

    /// Appointment list screen (simple list view)
    AppointmentList,

    /// Appointment calendar - month view focus
    CalendarMonthView,

    /// Appointment calendar - day/week view focus (normal mode)
    CalendarDayView,

    /// Appointment calendar - week view focus
    CalendarWeekView,

    /// Appointment calendar - day view in multi-select mode
    CalendarMultiSelect,

    /// Appointment detail modal
    CalendarDetailModal,

    /// Appointment reschedule modal
    CalendarRescheduleModal,

    /// Appointment search modal
    CalendarSearchModal,

    /// Calendar status filter menu
    CalendarFilterMenu,

    /// Calendar practitioner filter menu
    CalendarPractitionerMenu,

    /// Calendar audit history modal
    CalendarAuditModal,

    /// Calendar confirmation dialog
    CalendarConfirmation,

    /// Calendar error modal
    CalendarErrorModal,

    /// Calendar batch operations menu
    CalendarBatchMenu,

    /// Appointment form (create/edit)
    AppointmentForm,

    /// Appointment form patient field focused
    AppointmentFormPatient,

    /// Tab navigation (shared across all screens)
    Tabs,
}

/// Represents a single keybind with its action and description
#[derive(Debug, Clone)]
pub struct Keybind {
    /// The key code (e.g., KeyCode::Char('n'), KeyCode::Enter)
    pub key: KeyCode,

    /// Required modifiers (e.g., KeyModifiers::CONTROL)
    pub modifiers: KeyModifiers,

    /// Brief action name (e.g., "New Patient", "Navigate Up")
    pub action: &'static str,

    /// Human-readable description for help text
    pub description: &'static str,

    /// Whether this keybind is currently implemented
    pub implemented: bool,
}

impl Keybind {
    /// Create a new keybind without modifiers
    pub fn new(key: KeyCode, action: &'static str, description: &'static str) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::empty(),
            action,
            description,
            implemented: true,
        }
    }

    /// Create a new keybind with modifiers
    pub fn with_modifiers(
        key: KeyCode,
        modifiers: KeyModifiers,
        action: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            key,
            modifiers,
            action,
            description,
            implemented: true,
        }
    }

    /// Create a keybind that is not yet implemented
    pub fn unimplemented(key: KeyCode, action: &'static str, description: &'static str) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::empty(),
            action,
            description,
            implemented: false,
        }
    }
}

/// Centralized registry for all application keybinds
pub struct KeybindRegistry;

impl KeybindRegistry {
    /// Get all keybinds for a specific context
    pub fn get_keybinds(context: KeybindContext) -> Vec<Keybind> {
        match context {
            KeybindContext::Global => Self::global_keybinds(),
            KeybindContext::PatientList => Self::patient_list_keybinds(),
            KeybindContext::PatientListSearch => Self::patient_list_search_keybinds(),
            KeybindContext::PatientForm => Self::patient_form_keybinds(),
            KeybindContext::AppointmentList => Self::appointment_list_keybinds(),
            KeybindContext::CalendarMonthView => Self::calendar_month_view_keybinds(),
            KeybindContext::CalendarDayView => Self::calendar_day_view_keybinds(),
            KeybindContext::CalendarWeekView => Self::calendar_week_view_keybinds(),
            KeybindContext::CalendarMultiSelect => Self::calendar_multi_select_keybinds(),
            KeybindContext::CalendarDetailModal => Self::calendar_detail_modal_keybinds(),
            KeybindContext::CalendarRescheduleModal => Self::calendar_reschedule_modal_keybinds(),
            KeybindContext::CalendarSearchModal => Self::calendar_search_modal_keybinds(),
            KeybindContext::CalendarFilterMenu => Self::calendar_filter_menu_keybinds(),
            KeybindContext::CalendarPractitionerMenu => Self::calendar_practitioner_menu_keybinds(),
            KeybindContext::CalendarAuditModal => Self::calendar_audit_modal_keybinds(),
            KeybindContext::CalendarConfirmation => Self::calendar_confirmation_keybinds(),
            KeybindContext::CalendarErrorModal => Self::calendar_error_modal_keybinds(),
            KeybindContext::CalendarBatchMenu => Self::calendar_batch_menu_keybinds(),
            KeybindContext::AppointmentForm => Self::appointment_form_keybinds(),
            KeybindContext::AppointmentFormPatient => Self::appointment_form_patient_keybinds(),
            KeybindContext::Tabs => Self::tabs_keybinds(),
        }
    }

    /// Generate help text string for a specific context
    ///
    /// Returns a compact, human-readable string suitable for display in title bars
    /// or footers.
    pub fn get_help_text(context: KeybindContext) -> String {
        let keybinds = Self::get_keybinds(context);
        let mut parts: Vec<String> = Vec::new();

        for kb in keybinds.iter().filter(|k| k.implemented) {
            let key_display = Self::format_key(&kb.key, kb.modifiers);
            parts.push(format!("{}: {}", key_display, kb.description));
        }

        parts.join("  ")
    }

    /// Format a key combination for display
    pub fn format_key(key: &KeyCode, modifiers: KeyModifiers) -> String {
        let mut result = String::new();

        // Add modifiers
        if modifiers.contains(KeyModifiers::CONTROL) {
            result.push_str("Ctrl+");
        }
        if modifiers.contains(KeyModifiers::ALT) {
            result.push_str("Alt+");
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            result.push_str("Shift+");
        }

        // Add key
        match key {
            KeyCode::Char(c) => result.push(*c),
            KeyCode::Enter => result.push_str("Enter"),
            KeyCode::Esc => result.push_str("Esc"),
            KeyCode::Tab => result.push_str("Tab"),
            KeyCode::Up => result.push('↑'),
            KeyCode::Down => result.push('↓'),
            KeyCode::Left => result.push('←'),
            KeyCode::Right => result.push('→'),
            KeyCode::Backspace => result.push_str("Backspace"),
            KeyCode::F(n) => result.push_str(&format!("F{}", n)),
            _ => result.push('?'),
        }

        result
    }

    /// Format a keybind for display
    pub fn format_keybind(kb: &Keybind) -> String {
        format!("{}: {}", Self::format_key(&kb.key, kb.modifiers), kb.action)
    }

    /// Look up a keybind by key event in a specific context
    pub fn lookup_action(context: &KeybindContext, key: KeyEvent) -> Option<&'static str> {
        Self::lookup_keybind(context, key).map(|(action, _, _)| action)
    }

    /// Look up full keybind details: (action, description, implemented)
    pub fn lookup_keybind(
        context: &KeybindContext,
        key: KeyEvent,
    ) -> Option<(&'static str, &'static str, bool)> {
        let keybinds = Self::get_keybinds(context.clone());

        for kb in &keybinds {
            if kb.key == key.code && kb.modifiers == key.modifiers {
                return Some((kb.action, kb.description, kb.implemented));
            }
        }

        if let KeyCode::Char(c) = key.code {
            let lower_c = c.to_ascii_lowercase();
            let upper_c = c.to_ascii_uppercase();

            if !key.modifiers.contains(KeyModifiers::SHIFT) {
                for kb in &keybinds {
                    if let KeyCode::Char(kb_c) = kb.key {
                        let kb_lower = kb_c.to_ascii_lowercase();
                        let kb_upper = kb_c.to_ascii_uppercase();

                        if (kb_lower == lower_c && kb_lower == kb_c)
                            || (kb_upper == upper_c && kb_upper == kb_c)
                        {
                            if kb.modifiers.is_empty() || kb.modifiers == KeyModifiers::SHIFT {
                                return Some((kb.action, kb.description, kb.implemented));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if a key event matches any registered keybind in a context
    pub fn has_keybind(context: &KeybindContext, key: KeyEvent) -> bool {
        Self::lookup_action(context, key).is_some()
    }

    /// Get all implemented action names for a context
    pub fn get_implemented_actions(context: &KeybindContext) -> Vec<&'static str> {
        let keybinds = Self::get_keybinds(context.clone());
        keybinds
            .iter()
            .filter(|kb| kb.implemented)
            .map(|kb| kb.action)
            .collect()
    }

    // --- Context-specific keybind definitions ---

    fn global_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::with_modifiers(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                "Quit",
                "Quit application",
            ),
            Keybind::new(KeyCode::Char('q'), "Quit", "Quit application"),
            Keybind::new(KeyCode::Char('?'), "Help", "Show help"),
        ]
    }

    fn patient_list_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char('j'), "Next", "Next patient"),
            Keybind::new(KeyCode::Down, "Next", "Next patient"),
            Keybind::new(KeyCode::Char('k'), "Previous", "Previous patient"),
            Keybind::new(KeyCode::Up, "Previous", "Previous patient"),
            Keybind::new(KeyCode::Char('g'), "First", "First patient"),
            Keybind::new(KeyCode::Char('G'), "Last", "Last patient"),
            Keybind::unimplemented(KeyCode::Enter, "View", "View details (not implemented)"),
            Keybind::new(KeyCode::Char('e'), "Edit", "Edit patient"),
            Keybind::new(KeyCode::Char('n'), "New", "New patient"),
            Keybind::new(KeyCode::Char('/'), "Search", "Enter search"),
            Keybind::new(KeyCode::Esc, "Clear", "Clear search"),
        ]
    }

    fn patient_list_search_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Up, "Previous", "Previous result"),
            Keybind::new(KeyCode::Down, "Next", "Next result"),
            Keybind::new(KeyCode::Enter, "Exit", "Exit search"),
            Keybind::new(KeyCode::Esc, "Exit", "Exit search"),
            Keybind::new(KeyCode::Backspace, "Delete", "Delete character"),
        ]
    }

    fn patient_form_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Cancel", "Cancel form"),
            Keybind::new(KeyCode::F(10), "Submit", "Submit form"),
            Keybind::with_modifiers(
                KeyCode::Char('s'),
                KeyModifiers::CONTROL,
                "Submit",
                "Submit form (Ctrl+S)",
            ),
            Keybind::new(KeyCode::Tab, "Next", "Next field"),
            Keybind::with_modifiers(
                KeyCode::Tab,
                KeyModifiers::SHIFT,
                "Previous",
                "Previous field",
            ),
            Keybind::new(KeyCode::Up, "Up", "Previous field/cycle"),
            Keybind::new(KeyCode::Down, "Down", "Next field/cycle"),
            Keybind::new(KeyCode::Backspace, "Delete", "Delete character"),
        ]
    }

    fn appointment_list_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char('j'), "Next", "Next appointment"),
            Keybind::new(KeyCode::Down, "Next", "Next appointment"),
            Keybind::new(KeyCode::Char('k'), "Previous", "Previous appointment"),
            Keybind::new(KeyCode::Up, "Previous", "Previous appointment"),
            Keybind::new(KeyCode::Char('g'), "First", "First appointment"),
            Keybind::new(KeyCode::Char('G'), "Last", "Last appointment"),
            Keybind::new(KeyCode::Char('n'), "New", "New appointment"),
        ]
    }

    fn calendar_month_view_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Up, "Week Up", "Previous week"),
            Keybind::new(KeyCode::Down, "Week Down", "Next week"),
            Keybind::new(KeyCode::Left, "Day Back", "Previous day"),
            Keybind::new(KeyCode::Right, "Day Forward", "Next day"),
            Keybind::new(KeyCode::Char('h'), "Month Back", "Previous month"),
            Keybind::new(KeyCode::Char('l'), "Month Forward", "Next month"),
            Keybind::new(KeyCode::Char('t'), "Today", "Jump to today"),
            Keybind::new(KeyCode::Enter, "Day View", "Switch to day view"),
            Keybind::new(KeyCode::Tab, "Day View", "Switch to day view"),
            Keybind::new(KeyCode::Char('n'), "New", "New appointment"),
        ]
    }

    fn calendar_day_view_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char('k'), "Up", "Previous slot"),
            Keybind::new(KeyCode::Up, "Up", "Previous slot"),
            Keybind::new(KeyCode::Char('j'), "Down", "Next slot"),
            Keybind::new(KeyCode::Down, "Down", "Next slot"),
            Keybind::new(KeyCode::Tab, "Month", "Switch to month"),
            Keybind::new(KeyCode::Esc, "Month", "Switch to month"),
            Keybind::new(KeyCode::Enter, "Details", "Open details"),
            Keybind::new(KeyCode::Char('a'), "Arrived", "Mark Arrived"),
            Keybind::new(KeyCode::Char('i'), "In Progress", "Mark In Progress"),
            Keybind::new(KeyCode::Char('c'), "Completed", "Mark Completed"),
            Keybind::new(KeyCode::Char('x'), "No Show", "Mark No Show"),
            Keybind::new(KeyCode::Char('n'), "New", "New appointment"),
            Keybind::new(KeyCode::Char('v'), "Week", "Toggle Week view"),
            Keybind::new(KeyCode::Char('/'), "Search", "Open search"),
            Keybind::new(KeyCode::Char('f'), "Filter", "Open filter"),
            Keybind::new(KeyCode::Char('p'), "Practitioner", "Practitioner filter"),
            Keybind::new(KeyCode::Char('m'), "Multi-Select", "Toggle multi-select"),
            Keybind::with_modifiers(
                KeyCode::Char('z'),
                KeyModifiers::CONTROL,
                "Undo",
                "Undo status change",
            ),
        ]
    }

    fn calendar_week_view_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char('j'), "Down", "Next slot"),
            Keybind::new(KeyCode::Down, "Down", "Next slot"),
            Keybind::new(KeyCode::Char('k'), "Up", "Previous slot"),
            Keybind::new(KeyCode::Up, "Up", "Previous slot"),
            Keybind::new(KeyCode::Char('v'), "Day", "Toggle Day view"),
            Keybind::with_modifiers(
                KeyCode::Left,
                KeyModifiers::SHIFT,
                "Week Back",
                "Previous week",
            ),
            Keybind::with_modifiers(
                KeyCode::Right,
                KeyModifiers::SHIFT,
                "Week Forward",
                "Next week",
            ),
            Keybind::new(KeyCode::Char('n'), "New", "New appointment"),
            Keybind::new(KeyCode::Tab, "Month", "Switch to month"),
            Keybind::new(KeyCode::Esc, "Month", "Switch to month"),
        ]
    }

    fn calendar_multi_select_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char(' '), "Toggle", "Toggle selection"),
            Keybind::with_modifiers(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL,
                "Select All",
                "Select all",
            ),
            Keybind::new(KeyCode::Esc, "Exit", "Exit multi-select"),
            Keybind::new(KeyCode::Char('m'), "Exit", "Toggle mode"),
            Keybind::new(KeyCode::Char('b'), "Batch", "Batch operations"),
        ]
    }

    fn calendar_detail_modal_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Close", "Close modal"),
            Keybind::new(KeyCode::Char('A'), "Arrived", "Mark Arrived"),
            Keybind::new(KeyCode::Char('a'), "Arrived", "Mark Arrived"),
            Keybind::new(KeyCode::Char('I'), "In Progress", "Mark In Progress"),
            Keybind::new(KeyCode::Char('i'), "In Progress", "Mark In Progress"),
            Keybind::new(KeyCode::Char('C'), "Completed", "Mark Completed"),
            Keybind::new(KeyCode::Char('c'), "Completed", "Mark Completed"),
            Keybind::new(KeyCode::Char('X'), "No Show", "Mark No Show"),
            Keybind::new(KeyCode::Char('x'), "No Show", "Mark No Show"),
            Keybind::new(KeyCode::Char('H'), "History", "View history"),
            Keybind::new(KeyCode::Char('h'), "History", "View history"),
            Keybind::new(KeyCode::Char('R'), "Reschedule", "Reschedule"),
            Keybind::new(KeyCode::Char('r'), "Reschedule", "Reschedule"),
        ]
    }

    fn calendar_reschedule_modal_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Cancel", "Cancel"),
            Keybind::new(KeyCode::Up, "Earlier", "Move time earlier"),
            Keybind::new(KeyCode::Down, "Later", "Move time later"),
            Keybind::new(KeyCode::Char('+'), "Longer", "Increase duration"),
            Keybind::new(KeyCode::Char('-'), "Shorter", "Decrease duration"),
            Keybind::new(KeyCode::Enter, "Save", "Save"),
        ]
    }

    fn calendar_search_modal_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Close", "Close"),
            Keybind::new(KeyCode::Up, "Previous", "Previous result"),
            Keybind::new(KeyCode::Down, "Next", "Next result"),
            Keybind::new(KeyCode::Enter, "Select", "Select result"),
            Keybind::new(KeyCode::Backspace, "Delete", "Delete character"),
        ]
    }

    fn calendar_filter_menu_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Close", "Close"),
            Keybind::new(KeyCode::Char('0'), "Clear", "Clear all"),
            Keybind::new(KeyCode::Char('1'), "Scheduled", "Toggle Scheduled"),
            Keybind::new(KeyCode::Char('2'), "Confirmed", "Toggle Confirmed"),
            Keybind::new(KeyCode::Char('3'), "Arrived", "Toggle Arrived"),
            Keybind::new(KeyCode::Char('4'), "InProgress", "Toggle InProgress"),
            Keybind::new(KeyCode::Char('5'), "Completed", "Toggle Completed"),
            Keybind::new(KeyCode::Char('6'), "NoShow", "Toggle NoShow"),
            Keybind::new(KeyCode::Char('7'), "Cancelled", "Toggle Cancelled"),
            Keybind::new(KeyCode::Char('8'), "Rescheduled", "Toggle Rescheduled"),
        ]
    }

    fn calendar_practitioner_menu_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Close", "Close"),
            Keybind::new(KeyCode::Char('0'), "Clear", "Clear all"),
            Keybind::new(KeyCode::Char('1'), "Toggle 1", "Toggle practitioner 1"),
            Keybind::new(KeyCode::Char('2'), "Toggle 2", "Toggle practitioner 2"),
            Keybind::new(KeyCode::Char('3'), "Toggle 3", "Toggle practitioner 3"),
            Keybind::new(KeyCode::Char('4'), "Toggle 4", "Toggle practitioner 4"),
            Keybind::new(KeyCode::Char('5'), "Toggle 5", "Toggle practitioner 5"),
            Keybind::new(KeyCode::Char('6'), "Toggle 6", "Toggle practitioner 6"),
            Keybind::new(KeyCode::Char('7'), "Toggle 7", "Toggle practitioner 7"),
            Keybind::new(KeyCode::Char('8'), "Toggle 8", "Toggle practitioner 8"),
            Keybind::new(KeyCode::Char('9'), "Toggle 9", "Toggle practitioner 9"),
        ]
    }

    fn calendar_audit_modal_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Close", "Close"),
            Keybind::new(KeyCode::Up, "Previous", "Previous entry"),
            Keybind::new(KeyCode::Down, "Next", "Next entry"),
        ]
    }

    fn calendar_confirmation_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char('Y'), "Confirm", "Confirm"),
            Keybind::new(KeyCode::Char('y'), "Confirm", "Confirm"),
            Keybind::new(KeyCode::Char('N'), "Cancel", "Cancel"),
            Keybind::new(KeyCode::Char('n'), "Cancel", "Cancel"),
            Keybind::new(KeyCode::Esc, "Cancel", "Cancel"),
        ]
    }

    fn calendar_error_modal_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Close", "Close"),
            Keybind::new(KeyCode::Enter, "Close", "Close"),
        ]
    }

    fn calendar_batch_menu_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Cancel", "Cancel"),
            Keybind::new(KeyCode::Char('1'), "Arrived", "Mark all Arrived"),
            Keybind::new(KeyCode::Char('2'), "Completed", "Mark all Completed"),
            Keybind::unimplemented(KeyCode::Char('3'), "Cancel", "Cancel all (not implemented)"),
        ]
    }

    fn appointment_form_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Esc, "Cancel", "Cancel/Close"),
            Keybind::with_modifiers(
                KeyCode::Char('s'),
                KeyModifiers::CONTROL,
                "Submit",
                "Submit",
            ),
            Keybind::new(KeyCode::Tab, "Next", "Next field"),
            Keybind::with_modifiers(
                KeyCode::Tab,
                KeyModifiers::SHIFT,
                "Previous",
                "Previous field",
            ),
            Keybind::new(KeyCode::Enter, "Select", "Select dropdown"),
            Keybind::new(KeyCode::Char(' '), "Toggle", "Toggle dropdown"),
            Keybind::new(KeyCode::Up, "Previous", "Navigate up"),
            Keybind::new(KeyCode::Down, "Next", "Navigate down"),
            Keybind::new(KeyCode::Backspace, "Delete", "Delete character"),
        ]
    }

    fn appointment_form_patient_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Up, "Previous", "Previous result"),
            Keybind::new(KeyCode::Down, "Next", "Next result"),
            Keybind::new(KeyCode::Enter, "Select", "Select patient"),
            Keybind::new(KeyCode::Esc, "Clear", "Clear search"),
            Keybind::with_modifiers(
                KeyCode::Char('u'),
                KeyModifiers::CONTROL,
                "Clear",
                "Clear search",
            ),
            Keybind::with_modifiers(
                KeyCode::Char('s'),
                KeyModifiers::CONTROL,
                "Submit",
                "Submit form",
            ),
        ]
    }

    fn tabs_keybinds() -> Vec<Keybind> {
        vec![
            Keybind::new(KeyCode::Char('q'), "Quit", "Quit application"),
            Keybind::new(KeyCode::Char('1'), "Patients", "Go to Patients"),
            Keybind::new(KeyCode::Char('2'), "Appointments", "Go to Appointments"),
            Keybind::new(KeyCode::Char('3'), "Clinical", "Go to Clinical"),
            Keybind::new(KeyCode::Char('4'), "Billing", "Go to Billing"),
            Keybind::new(KeyCode::Right, "Next", "Next tab"),
            Keybind::new(KeyCode::Left, "Previous", "Previous tab"),
            Keybind::new(KeyCode::Tab, "Next", "Next tab"),
            Keybind::new(KeyCode::BackTab, "Previous", "Previous tab"),
            Keybind::new(KeyCode::Home, "First", "First tab"),
            Keybind::new(KeyCode::End, "Last", "Last tab"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_key_simple() {
        let result = KeybindRegistry::format_key(&KeyCode::Char('n'), KeyModifiers::empty());
        assert_eq!(result, "n");
    }

    #[test]
    fn test_format_key_with_control() {
        let result = KeybindRegistry::format_key(&KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(result, "Ctrl+c");
    }

    #[test]
    fn test_format_key_with_shift() {
        let result = KeybindRegistry::format_key(&KeyCode::Left, KeyModifiers::SHIFT);
        assert_eq!(result, "Shift+←");
    }

    #[test]
    fn test_format_key_special() {
        assert_eq!(
            KeybindRegistry::format_key(&KeyCode::Enter, KeyModifiers::empty()),
            "Enter"
        );
        assert_eq!(
            KeybindRegistry::format_key(&KeyCode::Esc, KeyModifiers::empty()),
            "Esc"
        );
        assert_eq!(
            KeybindRegistry::format_key(&KeyCode::Tab, KeyModifiers::empty()),
            "Tab"
        );
    }

    #[test]
    fn test_get_keybinds_global() {
        let keybinds = KeybindRegistry::get_keybinds(KeybindContext::Global);
        assert_eq!(keybinds.len(), 3);
        assert!(keybinds
            .iter()
            .any(|kb| matches!(kb.key, KeyCode::Char('q'))));
    }

    #[test]
    fn test_get_keybinds_patient_list() {
        let keybinds = KeybindRegistry::get_keybinds(KeybindContext::PatientList);
        assert_eq!(keybinds.len(), 11);
    }

    #[test]
    fn test_get_help_text_patient_list() {
        let help = KeybindRegistry::get_help_text(KeybindContext::PatientList);
        assert!(help.contains("j: Next patient"));
        assert!(help.contains("n: New patient"));
        // Unimplemented keybinds should not appear
        assert!(!help.contains("not implemented"));
    }

    #[test]
    fn test_n_key_no_conflict_in_day_view() {
        // In day view, 'n' should ONLY be for "New appointment"
        // 'x' is used for "No Show"
        let keybinds = KeybindRegistry::get_keybinds(KeybindContext::CalendarDayView);

        let n_keybinds: Vec<&Keybind> = keybinds
            .iter()
            .filter(|kb| matches!(kb.key, KeyCode::Char('n')))
            .collect();

        assert_eq!(
            n_keybinds.len(),
            1,
            "Only one 'n' keybind should exist in day view"
        );
        assert_eq!(n_keybinds[0].action, "New");

        // Verify 'x' is used for No Show
        let x_keybinds: Vec<&Keybind> = keybinds
            .iter()
            .filter(|kb| matches!(kb.key, KeyCode::Char('x')))
            .collect();

        assert_eq!(x_keybinds.len(), 1);
        assert_eq!(x_keybinds[0].action, "No Show");
    }

    #[test]
    fn test_enter_in_patient_list_is_unimplemented() {
        let keybinds = KeybindRegistry::get_keybinds(KeybindContext::PatientList);

        let enter_keybind = keybinds
            .iter()
            .find(|kb| matches!(kb.key, KeyCode::Enter))
            .expect("Enter keybind should exist in patient list");

        assert!(
            !enter_keybind.implemented,
            "Enter in patient list should be marked as unimplemented"
        );
    }

    #[test]
    fn test_lookup_action_exact_match() {
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        let action = KeybindRegistry::lookup_action(&KeybindContext::PatientList, key);
        assert_eq!(action, Some("New"));
    }

    #[test]
    fn test_lookup_action_case_insensitive() {
        // Lowercase should match lowercase
        let key_lower = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        assert_eq!(
            KeybindRegistry::lookup_action(&KeybindContext::PatientList, key_lower),
            Some("New")
        );

        // Uppercase should also match (case-insensitive for letters)
        let key_upper = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::empty());
        assert_eq!(
            KeybindRegistry::lookup_action(&KeybindContext::PatientList, key_upper),
            Some("New")
        );
    }

    #[test]
    fn test_lookup_action_with_modifiers() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let action = KeybindRegistry::lookup_action(&KeybindContext::Global, key);
        assert_eq!(action, Some("Quit"));
    }

    #[test]
    fn test_lookup_action_not_found() {
        let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty());
        let action = KeybindRegistry::lookup_action(&KeybindContext::PatientList, key);
        assert_eq!(action, None);
    }

    #[test]
    fn test_lookup_keybind_returns_details() {
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        let result = KeybindRegistry::lookup_keybind(&KeybindContext::PatientList, key);

        assert!(result.is_some());
        let (action, description, implemented) = result.unwrap();
        assert_eq!(action, "New");
        assert_eq!(description, "New patient");
        assert!(implemented);
    }

    #[test]
    fn test_lookup_keybind_unimplemented() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let result = KeybindRegistry::lookup_keybind(&KeybindContext::PatientList, key);

        assert!(result.is_some());
        let (action, description, implemented) = result.unwrap();
        assert_eq!(action, "View");
        assert!(!implemented, "Enter should be marked as unimplemented");
    }

    #[test]
    fn test_has_keybind() {
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
        assert!(KeybindRegistry::has_keybind(
            &KeybindContext::PatientList,
            key
        ));

        let unknown_key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty());
        assert!(!KeybindRegistry::has_keybind(
            &KeybindContext::PatientList,
            unknown_key
        ));
    }

    #[test]
    fn test_get_implemented_actions() {
        let actions = KeybindRegistry::get_implemented_actions(&KeybindContext::PatientList);

        assert!(actions.contains(&"New"));
        assert!(actions.contains(&"Edit"));
        assert!(!actions.contains(&"View"), "View is unimplemented");
    }
}
