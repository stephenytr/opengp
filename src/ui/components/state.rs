use std::fmt;

/// Represents the generic state of a component
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

/// Common styling configuration for interactive components
#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub active_color: ratatui::style::Color,
    pub inactive_color: ratatui::style::Color,
    pub error_color: ratatui::style::Color,
    pub focus_style: ratatui::style::Style,
    pub normal_style: ratatui::style::Style,
}

impl Default for StyleConfig {
    fn default() -> Self {
        use ratatui::style::{Color, Style};
        Self {
            active_color: Color::Yellow,
            inactive_color: Color::Gray,
            error_color: Color::Red,
            focus_style: Style::default().fg(Color::Yellow),
            normal_style: Style::default().fg(Color::Gray),
        }
    }
}
