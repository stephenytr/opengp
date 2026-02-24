//! Reusable Widgets Module
//!
//! Provides reusable UI widgets for the OpenGP TUI.
//! These widgets follow consistent styling via the Theme system.

mod dropdown;
mod form_field;
mod loading;
mod search_input;
mod searchable_list;

pub use dropdown::{DropdownAction, DropdownOption, DropdownState, DropdownWidget};
pub use form_field::{FieldType, FormField, FormFieldState};
pub use loading::{LoadingIndicator, LoadingState, SpinnerStyle};
pub use search_input::{SearchInput, SearchInputState};
pub use searchable_list::{Searchable, SearchableList, SearchableListAction, SearchableListState};
