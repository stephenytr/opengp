use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::ui::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FieldType {
    #[default]
    Text,
    Date,
    Select(Vec<&'static str>),
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct FormFieldState {
    pub label: &'static str,
    pub value: String,
    pub error: Option<String>,
    pub focused: bool,
    pub required: bool,
    pub field_type: FieldType,
}


impl FormFieldState {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            ..Default::default()
        }
    }

    pub fn value(mut self, value: String) -> Self {
        self.value = value;
        self
    }

    pub fn error(mut self, error: Option<String>) -> Self {
        self.error = error;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn field_type(mut self, field_type: FieldType) -> Self {
        self.field_type = field_type;
        self
    }
}

pub struct FormField<'a> {
    state: &'a FormFieldState,
    theme: Theme,
    width: u16,
}

impl<'a> FormField<'a> {
    pub fn new(state: &'a FormFieldState, theme: Theme) -> Self {
        Self {
            state,
            theme,
            width: 30,
        }
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    fn label_text(&self) -> String {
        if self.state.required {
            format!("{} *", self.state.label)
        } else {
            self.state.label.to_string()
        }
    }
}

impl Widget for FormField<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let label_style = if self.state.focused {
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.colors.foreground)
        };

        let label_text = self.label_text();
        buf.set_string(area.x, area.y, &label_text, label_style);

        if self.state.focused {
            buf.set_string(
                area.x + label_text.len() as u16 + 1,
                area.y,
                ">",
                Style::default().fg(self.theme.colors.primary),
            );
        }

        let value_x = area.x + 24;
        let value_style = if self.state.error.is_some() {
            Style::default().fg(self.theme.colors.error)
        } else {
            Style::default().fg(self.theme.colors.foreground)
        };

        let max_value_width = self.width.saturating_sub(26) as usize;
        let display_value = if self.state.value.len() > max_value_width {
            &self.state.value[self.state.value.len() - max_value_width..]
        } else {
            &self.state.value
        };

        buf.set_string(value_x, area.y, display_value, value_style);

        if let Some(ref error_msg) = self.state.error {
            let error_style = Style::default().fg(self.theme.colors.error);
            buf.set_string(value_x, area.y + 1, format!("  {}", error_msg), error_style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_state_builder() {
        let state = FormFieldState::new("First Name")
            .value("John".to_string())
            .required(true)
            .focused(true);

        assert_eq!(state.label, "First Name");
        assert_eq!(state.value, "John");
        assert!(state.required);
        assert!(state.focused);
    }

    #[test]
    fn test_form_field_label_required() {
        let state = FormFieldState::new("Email").required(true);
        let theme = Theme::dark();
        let field = FormField::new(&state, theme);

        assert_eq!(field.label_text(), "Email *");
    }

    #[test]
    fn test_form_field_label_optional() {
        let state = FormFieldState::new("Middle Name");
        let theme = Theme::dark();
        let field = FormField::new(&state, theme);

        assert_eq!(field.label_text(), "Middle Name");
    }
}
