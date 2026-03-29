use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}
