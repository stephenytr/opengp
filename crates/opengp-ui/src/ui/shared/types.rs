use uuid::Uuid;

/// Mode that a form is operating in, either creating a new record or editing an existing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    /// Create a new record, with no existing identifier.
    #[default]
    Create,
    /// Edit an existing record identified by its domain id.
    Edit(Uuid),
}

/// High level actions that a form can emit back to its caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormAction {
    /// Focus moved to a different field.
    FocusChanged,
    /// A field value changed.
    ValueChanged,
    /// The user requested to submit the form.
    Submit,
    /// The user cancelled the form without saving.
    Cancel,
}

/// High level actions that a modal dialog can emit back to its caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalAction {
    /// Focus moved to a different button.
    FocusChanged,
    /// The user confirmed by pressing Enter on the focused button.
    Confirm,
    /// The user dismissed the modal without confirming.
    Dismiss,
}
