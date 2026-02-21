use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use ratatui_interact::components::InputState;

use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct SearchInputState {
    pub input: InputState,
    pub placeholder: &'static str,
    pub focused: bool,
}

impl Default for SearchInputState {
    fn default() -> Self {
        Self {
            input: InputState::new(""),
            placeholder: "type to search...",
            focused: false,
        }
    }
}

impl SearchInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn query(mut self, query: &str) -> Self {
        self.input = InputState::new(query);
        self
    }

    pub fn placeholder(mut self, placeholder: &'static str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn clear(&mut self) {
        self.input.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input.insert_char(c);
    }

    pub fn pop(&mut self) {
        self.input.delete_char_backward();
    }

    pub fn value(&self) -> &str {
        self.input.text()
    }
}

pub struct SearchInput<'a> {
    state: &'a SearchInputState,
    theme: Theme,
    prompt: char,
}

impl<'a> SearchInput<'a> {
    pub fn new(state: &'a SearchInputState, theme: Theme) -> Self {
        Self {
            state,
            theme,
            prompt: '/',
        }
    }

    pub fn prompt(mut self, prompt: char) -> Self {
        self.prompt = prompt;
        self
    }
}

impl Widget for SearchInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let prompt_span = Span::styled(
            format!("{} ", self.prompt),
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(ratatui::style::Modifier::BOLD),
        );

        let value = self.state.input.text();
        let content_spans = if value.is_empty() {
            if self.state.focused {
                vec![
                    prompt_span,
                    Span::styled(
                        self.state.placeholder,
                        Style::default().fg(self.theme.colors.disabled),
                    ),
                    Span::styled("_", Style::default().fg(self.theme.colors.foreground)),
                ]
            } else {
                vec![prompt_span]
            }
        } else {
            vec![
                prompt_span,
                Span::styled(value, Style::default().fg(self.theme.colors.foreground)),
                if self.state.focused {
                    Span::styled("_", Style::default().fg(self.theme.colors.foreground))
                } else {
                    Span::raw("")
                },
            ]
        };

        let line = Line::from(content_spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_input_state_default() {
        let state = SearchInputState::new();
        assert!(state.is_empty());
        assert!(!state.focused);
    }

    #[test]
    fn test_search_input_state_modification() {
        let mut state = SearchInputState::new().query("test").focused(true);

        assert_eq!(state.value(), "test");
        assert!(state.focused);

        state.push_char('s');
        assert_eq!(state.value(), "tests");

        state.pop();
        assert_eq!(state.value(), "test");

        state.clear();
        assert!(state.is_empty());
    }

    #[test]
    fn test_search_input_state_builder() {
        let state = SearchInputState::new()
            .placeholder("search patients...")
            .focused(true);

        assert_eq!(state.placeholder, "search patients...");
        assert!(state.focused);
    }
}
