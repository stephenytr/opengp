//! Reusable Widgets Module
//!
//! Provides reusable UI widgets for the OpenGP TUI.
//! These widgets follow consistent styling via the Theme system.

mod calendar;
mod clinical_table_list;
mod date_picker_popup;
mod date_utils;
mod dropdown;
mod dynamic_form;
mod form_field;
mod form_field_macro;
mod form_navigation;
mod form_rules;
mod form_state;
mod list_nav;
mod loading;
mod modal_state;
mod navigation;
mod scrollable;
mod scrollable_form;
mod searchable_list;
mod textarea;
mod time_picker_popup;
mod validation;

pub use calendar::{AppointmentStyler, CalendarAction, CalendarMode, CalendarWidget};
pub use clinical_table_list::{ClinicalTableList, ColumnDef, ListAction};
pub use date_picker_popup::{DatePickerAction, DatePickerPopup};
pub use date_utils::{format_date, format_user_input, is_valid_date, parse_date};
pub use dropdown::{DropdownAction, DropdownOption, DropdownState, DropdownWidget};
pub use dynamic_form::{DynamicForm, DynamicFormMeta};
pub use form_field::{FieldType, FormField as FormFieldWidget, FormFieldState};
pub(crate) use form_field_macro::impl_form_field_wrapper;
pub use form_navigation::{FormFieldMeta, FormNavigation};
pub use form_rules::FormRuleEngine;
pub use form_state::{FormField, FormState};
pub use list_nav::{list_handle_key, list_handle_mouse, ListNavAction};
pub use loading::{LoadingIndicator, LoadingState, SpinnerStyle};
pub use modal_state::{ModalButton, ModalState};
pub use navigation::{migration_guide, scroll_management};
pub use scrollable::ScrollableState;
pub use scrollable_form::ScrollableFormState;
pub use searchable_list::{Searchable, SearchableList, SearchableListAction, SearchableListState};
pub use textarea::{HeightMode, TextareaState, TextareaWidget};
pub use time_picker_popup::{TimePickerAction, TimePickerPopup};
pub use validation::FormValidator;
