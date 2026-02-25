use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};
use ratatui_textarea::TextArea;

use crate::ui::theme::Theme;

/// Controls how the textarea calculates its height.
#[derive(Clone, Debug, PartialEq)]
pub enum HeightMode {
    /// Single-line input. Enter and Ctrl+M are blocked and passed to the caller.
    SingleLine,
    /// Fixed height of N lines (plus 2 for the border).
    FixedLines(u16),
    /// Grows with content, clamped between `min` and `max` lines.
    AutoGrow { min: u16, max: u16 },
}

#[derive(Clone)]
pub struct TextareaState {
    pub textarea: TextArea<'static>,
    pub label: &'static str,
    pub focused: bool,
    pub error: Option<String>,
    pub height_mode: HeightMode,
    pub max_length: Option<usize>,
}

impl std::fmt::Debug for TextareaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextareaState")
            .field("label", &self.label)
            .field("focused", &self.focused)
            .field("error", &self.error)
            .field("height_mode", &self.height_mode)
            .field("max_length", &self.max_length)
            .finish()
    }
}

impl TextareaState {
    pub fn new(label: &'static str) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_line_number_style(Style::default().fg(Color::Reset));
        Self {
            textarea,
            label,
            focused: false,
            error: None,
            height_mode: HeightMode::FixedLines(4),
            max_length: None,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let text = value.into();
        let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
        if lines.is_empty() {
            self.textarea = TextArea::default();
        } else {
            self.textarea = TextArea::from(lines);
        }
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn error(mut self, error: Option<String>) -> Self {
        self.error = error;
        self
    }

    pub fn with_height_mode(mut self, mode: HeightMode) -> Self {
        self.height_mode = mode;
        self
    }

    pub fn max_length(mut self, limit: usize) -> Self {
        self.max_length = Some(limit);
        self
    }

    pub fn value(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn is_empty(&self) -> bool {
        let lines = self.textarea.lines();
        lines.is_empty() || (lines.len() == 1 && lines[0].is_empty())
    }

    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    pub fn height(&self) -> u16 {
        match self.height_mode {
            HeightMode::SingleLine => 3,
            HeightMode::FixedLines(n) => n + 2,
            HeightMode::AutoGrow { min, max } => {
                let content_lines = self.textarea.lines().len() as u16;
                content_lines.clamp(min, max) + 2
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Tab | KeyCode::Esc | KeyCode::BackTab => return false,
            KeyCode::Enter => {
                if matches!(self.height_mode, HeightMode::SingleLine) {
                    return false;
                }
            }
            KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if matches!(self.height_mode, HeightMode::SingleLine) {
                    return false;
                }
            }
            KeyCode::Char(_) => {
                if let Some(limit) = self.max_length {
                    let current_len: usize = self
                        .textarea
                        .lines()
                        .iter()
                        .map(|l| l.chars().count())
                        .sum();
                    if current_len >= limit {
                        return false;
                    }
                }
            }
            _ => {}
        }

        self.textarea.input(key);
        true
    }
}

pub struct TextareaWidget<'a> {
    state: &'a TextareaState,
    theme: Theme,
    focused: bool,
}

impl<'a> TextareaWidget<'a> {
    pub fn new(state: &'a TextareaState, theme: Theme) -> Self {
        Self {
            state,
            theme,
            focused: state.focused,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn build_block(&self) -> Block<'static> {
        let border_style = if self.state.error.is_some() {
            Style::default()
                .fg(self.theme.colors.error)
                .bg(Color::Black)
        } else if self.focused {
            Style::default()
                .fg(self.theme.colors.primary)
                .bg(Color::Black)
        } else {
            Style::default()
                .fg(self.theme.colors.border)
                .bg(Color::Black)
        };

        let title_style = if self.focused {
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.colors.foreground)
        };

        Block::default()
            .title(Span::styled(format!(" {} ", self.state.label), title_style))
            .borders(Borders::ALL)
            .border_style(border_style)
    }
}

impl Widget for TextareaWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let (textarea_area, error_area) = if self.state.error.is_some() && area.height > 2 {
            let ta = Rect::new(area.x, area.y, area.width, area.height - 1);
            let err = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
            (ta, Some(err))
        } else {
            (area, None)
        };

        // Clone so we can apply theme styles without mutating the caller's state.
        let mut ta = self.state.textarea.clone();

        ta.set_block(self.build_block());
        ta.set_style(
            Style::default()
                .fg(self.theme.colors.foreground)
                .bg(Color::Black),
        );
        ta.set_line_number_style(Style::default().fg(Color::Reset));

        if self.focused {
            ta.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            ta.set_cursor_style(
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::REVERSED),
            );
        } else {
            ta.set_cursor_line_style(Style::default());
            // Hide cursor when not focused
            ta.set_cursor_style(Style::default().fg(Color::Reset).bg(Color::Reset));
        }

        ta.render(textarea_area, buf);

        if let (Some(err_area), Some(ref error_msg)) = (error_area, &self.state.error) {
            let error_line = Line::from(vec![
                Span::styled("  ✗ ", Style::default().fg(self.theme.colors.error)),
                Span::styled(
                    error_msg.as_str(),
                    Style::default().fg(self.theme.colors.error),
                ),
            ]);
            buf.set_line(err_area.x, err_area.y, &error_line, err_area.width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textarea_state_default_empty() {
        let state = TextareaState::new("Subjective");
        assert!(state.is_empty());
        assert!(!state.focused);
        assert!(state.error.is_none());
        assert_eq!(state.label, "Subjective");
    }

    #[test]
    fn test_textarea_state_with_value() {
        let state = TextareaState::new("Plan").with_value("Review in 2 weeks");
        assert!(!state.is_empty());
        assert_eq!(state.value(), "Review in 2 weeks");
    }

    #[test]
    fn test_textarea_state_multiline_value() {
        let state = TextareaState::new("Notes").with_value("Line one\nLine two\nLine three");
        assert_eq!(state.value(), "Line one\nLine two\nLine three");
    }

    #[test]
    fn test_textarea_state_clear() {
        let mut state = TextareaState::new("Assessment").with_value("Some text");
        assert!(!state.is_empty());
        state.clear();
        assert!(state.is_empty());
    }

    #[test]
    fn test_textarea_state_focused_builder() {
        let state = TextareaState::new("Objective").focused(true);
        assert!(state.focused);
    }

    #[test]
    fn test_textarea_state_error_builder() {
        let state = TextareaState::new("Plan").error(Some("Required field".to_string()));
        assert_eq!(state.error, Some("Required field".to_string()));
    }

    #[test]
    fn test_textarea_state_set_focused() {
        let mut state = TextareaState::new("Subjective");
        assert!(!state.focused);
        state.set_focused(true);
        assert!(state.focused);
    }

    #[test]
    fn test_textarea_state_set_error() {
        let mut state = TextareaState::new("Plan");
        assert!(state.error.is_none());
        state.set_error(Some("Cannot be empty".to_string()));
        assert_eq!(state.error, Some("Cannot be empty".to_string()));
        state.set_error(None);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_handle_key_tab_not_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Test");
        let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let consumed = state.handle_key(tab_key);
        assert!(!consumed, "Tab should not be consumed by textarea");
    }

    #[test]
    fn test_handle_key_esc_not_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Test");
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let consumed = state.handle_key(esc_key);
        assert!(!consumed, "Esc should not be consumed by textarea");
    }

    #[test]
    fn test_handle_key_char_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Test");
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let consumed = state.handle_key(char_key);
        assert!(consumed, "Char key should be consumed by textarea");
        assert_eq!(state.value(), "a");
    }

    #[test]
    fn test_height_mode_default_is_fixed_lines_4() {
        let state = TextareaState::new("Notes");
        assert_eq!(state.height_mode, HeightMode::FixedLines(4));
        assert_eq!(state.height(), 6);
    }

    #[test]
    fn test_height_mode_single_line() {
        let state = TextareaState::new("Reason").with_height_mode(HeightMode::SingleLine);
        assert_eq!(state.height_mode, HeightMode::SingleLine);
        assert_eq!(state.height(), 3);
    }

    #[test]
    fn test_height_mode_fixed_lines() {
        let state = TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(6));
        assert_eq!(state.height(), 8);
    }

    #[test]
    fn test_height_mode_auto_grow_empty_uses_min() {
        let state =
            TextareaState::new("Notes").with_height_mode(HeightMode::AutoGrow { min: 2, max: 8 });
        assert_eq!(state.height(), 4);
    }

    #[test]
    fn test_height_mode_auto_grow_with_content() {
        let state = TextareaState::new("Notes")
            .with_height_mode(HeightMode::AutoGrow { min: 2, max: 8 })
            .with_value("Line one\nLine two\nLine three\nLine four");
        assert_eq!(state.height(), 6);
    }

    #[test]
    fn test_height_mode_auto_grow_capped_at_max() {
        let state = TextareaState::new("Notes")
            .with_height_mode(HeightMode::AutoGrow { min: 2, max: 4 })
            .with_value("a\nb\nc\nd\ne\nf\ng\nh");
        assert_eq!(state.height(), 6);
    }

    #[test]
    fn test_single_line_enter_not_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Reason").with_height_mode(HeightMode::SingleLine);
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let consumed = state.handle_key(enter_key);
        assert!(!consumed, "Enter should not be consumed in SingleLine mode");
    }

    #[test]
    fn test_single_line_ctrl_m_not_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Reason").with_height_mode(HeightMode::SingleLine);
        let ctrl_m = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL);
        let consumed = state.handle_key(ctrl_m);
        assert!(
            !consumed,
            "Ctrl+M should not be consumed in SingleLine mode"
        );
    }

    #[test]
    fn test_fixed_lines_enter_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(4));
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let consumed = state.handle_key(enter_key);
        assert!(consumed, "Enter should be consumed in FixedLines mode");
    }

    #[test]
    fn test_auto_grow_enter_consumed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state =
            TextareaState::new("Notes").with_height_mode(HeightMode::AutoGrow { min: 2, max: 8 });
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let consumed = state.handle_key(enter_key);
        assert!(consumed, "Enter should be consumed in AutoGrow mode");
    }

    #[test]
    fn test_with_height_mode_builder() {
        let state =
            TextareaState::new("Test").with_height_mode(HeightMode::AutoGrow { min: 3, max: 10 });
        assert_eq!(state.height_mode, HeightMode::AutoGrow { min: 3, max: 10 });
    }

    #[test]
    fn test_max_length_builder_sets_field() {
        let state = TextareaState::new("Notes").max_length(10);
        assert_eq!(state.max_length, Some(10));
    }

    #[test]
    fn test_max_length_allows_chars_under_limit() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Notes").max_length(3);
        let consumed_a = state.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        let consumed_b = state.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        assert!(consumed_a, "First char should be consumed when under limit");
        assert!(
            consumed_b,
            "Second char should be consumed when under limit"
        );
        assert_eq!(state.value(), "ab");
    }

    #[test]
    fn test_max_length_blocks_char_at_limit() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Notes").max_length(3);
        state.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        state.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        state.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        assert_eq!(state.value(), "abc");
        let consumed = state.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        assert!(!consumed, "Char at limit should NOT be consumed");
        assert_eq!(state.value(), "abc", "Value must not change when at limit");
    }

    #[test]
    fn test_max_length_none_has_no_restriction() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Notes");
        for ch in ['a', 'b', 'c', 'd', 'e'] {
            let consumed = state.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
            assert!(consumed, "Char should be consumed when no max_length set");
        }
        assert_eq!(state.value(), "abcde");
    }

    #[test]
    fn test_max_length_non_char_keys_not_blocked() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut state = TextareaState::new("Notes").max_length(2);
        state.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        state.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        let consumed = state.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert!(consumed, "Backspace should still be consumed at limit");
        assert_eq!(state.value(), "a", "Backspace should delete a character");
    }
}
