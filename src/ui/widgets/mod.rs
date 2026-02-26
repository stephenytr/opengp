//! Reusable Widgets Module
//!
//! Provides reusable UI widgets for the OpenGP TUI.
//! These widgets follow consistent styling via the Theme system.

mod calendar;
mod date_utils;
mod dropdown;
mod form_field;
mod loading;
mod scrollable;
mod searchable_list;
mod textarea;

pub use calendar::{AppointmentStyler, CalendarAction, CalendarMode, CalendarWidget};
pub use date_utils::{format_date, format_user_input, is_valid_date, parse_date};
pub use dropdown::{DropdownAction, DropdownOption, DropdownState, DropdownWidget};
pub use form_field::{FieldType, FormField, FormFieldState};
pub use loading::{LoadingIndicator, LoadingState, SpinnerStyle};
pub use scrollable::ScrollableState;
pub use searchable_list::{Searchable, SearchableList, SearchableListAction, SearchableListState};
pub use textarea::{HeightMode, TextareaState, TextareaWidget};
