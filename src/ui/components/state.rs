use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentState {
    Idle,
    Focused,
    Active,
    Disabled,
    Error(String),
}

impl Default for ComponentState {
    fn default() -> Self {
        Self::Idle
    }
}

impl fmt::Display for ComponentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Focused => write!(f, "Focused"),
            Self::Active => write!(f, "Active"),
            Self::Disabled => write!(f, "Disabled"),
            Self::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}
