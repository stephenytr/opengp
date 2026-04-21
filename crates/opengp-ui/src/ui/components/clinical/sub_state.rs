//! Shared trait for clinical sub-domain state management.

/// Common behavior expected from clinical sub-domain state structs.
pub trait ClinicalSubState {
    fn set_loading(&mut self, loading: bool);
    fn set_error(&mut self, error: Option<String>);
    fn clear_error(&mut self);
    fn clear(&mut self);
    fn is_form_open(&self) -> bool;
    fn next_item(&mut self);
    fn prev_item(&mut self);
}
