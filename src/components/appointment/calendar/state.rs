//! State management for the AppointmentCalendarComponent
//!
//! This module contains state structs that organize the 73 fields from the monolithic
//! AppointmentCalendarComponent into logical groups:
//! - `CalendarState`: Calendar navigation and view state
//! - `FilterState`: Status and practitioner filter state
//! - `HistoryState`: Undo/redo and multi-select state

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use ratatui::widgets::TableState;
use std::collections::HashSet;
use uuid::Uuid;

use crate::domain::appointment::{AppointmentStatus, CalendarAppointment};
use crate::domain::patient::Patient;
use crate::domain::user::Practitioner;

/// Calendar navigation and view state
///
/// Contains all fields related to:
/// - Current date and month navigation
/// - View mode (day/week)
/// - Focus area (month view / day view)
/// - Time slot selection
/// - Loaded data (practitioners, appointments)
#[derive(Debug, Clone)]
pub struct CalendarState {
    /// Today's date
    pub current_date: NaiveDate,
    /// First day of the current displayed month
    pub current_month_start: NaiveDate,
    /// Currently selected day in the month view (1-31)
    pub selected_month_day: u32,
    /// First day of the current week (Monday)
    pub week_start_date: NaiveDate,
    /// Current view mode: day or week
    pub view_mode: ViewMode,
    /// Current focus area: month view or day view
    pub focus_area: FocusArea,
    /// State for time slot table navigation
    pub time_slot_state: TableState,
    /// Loaded practitioners
    pub practitioners: Vec<Practitioner>,
    /// Loaded appointments for current view
    pub appointments: Vec<CalendarAppointment>,
    /// Currently selected practitioner column index (0-based)
    pub selected_practitioner_column: usize,
}

impl CalendarState {
    /// Create a new CalendarState with default values
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        let today = chrono::Local::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .expect("first day of month is always valid");

        // Calculate week start date (Monday of current week)
        let weekday = today.weekday();
        let days_from_monday = weekday.num_days_from_monday();
        let week_start = today - chrono::Duration::days(days_from_monday as i64);

        Self {
            current_date: today,
            current_month_start: month_start,
            selected_month_day: today.day(),
            week_start_date: week_start,
            view_mode: ViewMode::Day,
            focus_area: FocusArea::MonthView,
            time_slot_state: table_state,
            practitioners: Vec::new(),
            appointments: Vec::new(),
            selected_practitioner_column: 0,
        }
    }
}

impl Default for CalendarState {
    fn default() -> Self {
        Self::new()
    }
}

/// View mode for the day schedule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Single day view
    #[default]
    Day,
    /// Week view (7 days)
    Week,
}

/// Focus area in the calendar component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusArea {
    /// Month calendar view
    #[default]
    MonthView,
    /// Day/Week schedule view
    DayView,
}

/// Filter state for appointments
///
/// Contains fields related to:
/// - Status filters (confirmed, booked, cancelled, etc.)
/// - Practitioner filters
#[derive(Debug, Clone)]
pub struct FilterState {
    /// Active status filters (currently applied)
    pub active_status_filters: HashSet<AppointmentStatus>,
    /// Whether the filter menu is currently displayed
    pub showing_filter_menu: bool,
    /// Active practitioner filters (currently applied)
    pub active_practitioner_filters: HashSet<Uuid>,
    /// Whether the practitioner filter menu is displayed
    pub showing_practitioner_menu: bool,
}

impl FilterState {
    /// Create a new FilterState with empty filters
    pub fn new() -> Self {
        Self {
            active_status_filters: HashSet::new(),
            showing_filter_menu: false,
            active_practitioner_filters: HashSet::new(),
            showing_practitioner_menu: false,
        }
    }

    /// Toggle a status filter on/off
    pub fn toggle_status_filter(&mut self, status: AppointmentStatus) {
        if self.active_status_filters.contains(&status) {
            self.active_status_filters.remove(&status);
        } else {
            self.active_status_filters.insert(status);
        }
    }

    /// Toggle a practitioner filter on/off
    pub fn toggle_practitioner_filter(&mut self, practitioner_id: Uuid) {
        if self.active_practitioner_filters.contains(&practitioner_id) {
            self.active_practitioner_filters.remove(&practitioner_id);
        } else {
            self.active_practitioner_filters.insert(practitioner_id);
        }
    }
}

impl Default for FilterState {
    fn default() -> Self {
        Self::new()
    }
}

/// History state for undo/redo and multi-select
///
/// Contains fields related to:
/// - Undo stack for status changes
/// - Multi-select mode and selected appointments
#[derive(Debug, Clone)]
pub struct HistoryState {
    /// Recent status changes for undo (max 5)
    pub recent_status_changes: Vec<(Uuid, AppointmentStatus)>,
    /// Timestamp of last status change (for undo expiration)
    pub undo_timestamp: Option<DateTime<Utc>>,
    /// Whether multi-select mode is active
    pub multi_select_mode: bool,
    /// Currently selected appointments in multi-select mode
    pub selected_appointments: HashSet<Uuid>,
}

impl HistoryState {
    /// Create a new HistoryState with empty history
    pub fn new() -> Self {
        Self {
            recent_status_changes: Vec::new(),
            undo_timestamp: None,
            multi_select_mode: false,
            selected_appointments: HashSet::new(),
        }
    }
}

impl Default for HistoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Data associated with detail modals
#[derive(Debug, Clone, Default)]
pub struct DetailModalData {
    /// Whether the detail modal is currently showing
    pub showing: bool,
    /// The selected appointment ID
    pub appointment_id: Option<Uuid>,
    /// The patient associated with the selected appointment
    pub patient: Option<Patient>,
}

/// Data associated with reschedule modals
#[derive(Debug, Clone, Default)]
pub struct RescheduleModalData {
    /// Whether the reschedule modal is currently showing
    pub showing: bool,
    /// New start time for rescheduling
    pub new_start_time: Option<DateTime<Utc>>,
    /// New duration in minutes
    pub new_duration: i64,
    /// Conflict warning message if any
    pub conflict_warning: Option<String>,
}

impl RescheduleModalData {
    /// Create new reschedule modal data with defaults
    pub fn new() -> Self {
        Self {
            showing: false,
            new_start_time: None,
            new_duration: 15,
            conflict_warning: None,
        }
    }
}

/// Data associated with search modals
#[derive(Debug, Clone, Default)]
pub struct SearchModalData {
    /// Whether the search modal is currently showing
    pub showing: bool,
    /// Current search query string
    pub query: String,
    /// Search results
    pub results: Vec<CalendarAppointment>,
    /// Currently selected index in search results
    pub selected_index: usize,
}

impl SearchModalData {
    /// Create new search modal data
    pub fn new() -> Self {
        Self {
            showing: false,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
        }
    }
}

/// Data associated with confirmation modals
#[derive(Debug, Clone, Default)]
pub struct ConfirmationModalData {
    /// Whether the confirmation modal is currently showing
    pub showing: bool,
    /// Message to display in confirmation
    pub message: String,
    /// Pending status to apply after confirmation
    pub pending_status: Option<AppointmentStatus>,
    /// Pending appointment ID for the status change
    pub pending_appointment_id: Option<Uuid>,
}

impl ConfirmationModalData {
    /// Create new confirmation modal data
    pub fn new() -> Self {
        Self {
            showing: false,
            message: String::new(),
            pending_status: None,
            pending_appointment_id: None,
        }
    }
}

/// Data associated with audit history modals
#[derive(Debug, Clone, Default)]
pub struct AuditModalData {
    /// Whether the audit modal is currently showing
    pub showing: bool,
    /// Audit entries to display
    pub entries: Vec<crate::domain::audit::AuditEntry>,
    /// Currently selected index in audit entries
    pub selected_index: usize,
}

impl AuditModalData {
    /// Create new audit modal data
    pub fn new() -> Self {
        Self {
            showing: false,
            entries: Vec::new(),
            selected_index: 0,
        }
    }
}

/// Data associated with batch operations
#[derive(Debug, Clone)]
pub struct BatchModalData {
    /// Whether the batch menu is displayed
    pub showing_menu: bool,
    /// Whether a batch operation is in progress
    pub operation_in_progress: bool,
    /// Current progress count
    pub progress_current: usize,
    /// Total progress count
    pub progress_total: usize,
    /// Progress message
    pub progress_message: String,
}

impl BatchModalData {
    /// Create new batch modal data
    pub fn new() -> Self {
        Self {
            showing_menu: false,
            operation_in_progress: false,
            progress_current: 0,
            progress_total: 0,
            progress_message: String::new(),
        }
    }
}

impl Default for BatchModalData {
    fn default() -> Self {
        Self::new()
    }
}

/// Error modal data
#[derive(Debug, Clone, Default)]
pub struct ErrorModalData {
    /// Whether the error modal is currently showing
    pub showing: bool,
    /// Error message to display
    pub message: String,
}

impl ErrorModalData {
    /// Create new error modal data
    pub fn new() -> Self {
        Self {
            showing: false,
            message: String::new(),
        }
    }
}
