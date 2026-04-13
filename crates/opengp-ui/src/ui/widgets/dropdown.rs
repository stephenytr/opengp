use std::collections::HashMap;

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::theme::Theme;

/// A single selectable option in a [`DropdownWidget`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DropdownOption {
    /// Machine readable value returned when this option is selected.
    pub value: String,
    /// Human readable label shown in the UI.
    pub label: String,
}

impl DropdownOption {
    /// Creates a new dropdown option from a value and label.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Open or closed state of a [`DropdownWidget`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownState {
    /// The dropdown popup is not visible.
    #[default]
    Closed,
    /// The dropdown popup is visible and can be interacted with.
    Open,
}

/// Interactive dropdown widget for choosing one option from a list.
pub struct DropdownWidget {
    pub options: Vec<DropdownOption>,
    pub selected_index: Option<usize>,
    pub focused_index: usize,
    pub state: DropdownState,
    pub label: String,
    pub placeholder: String,
    pub errors: HashMap<String, String>,
    pub error: Option<String>,
    pub theme: Theme,
    pub focused: bool,
}

impl Clone for DropdownWidget {
    fn clone(&self) -> Self {
        Self {
            options: self.options.clone(),
            selected_index: self.selected_index,
            focused_index: self.focused_index,
            state: self.state,
            label: self.label.clone(),
            placeholder: self.placeholder.clone(),
            errors: self.errors.clone(),
            error: self.error.clone(),
            theme: self.theme.clone(),
            focused: self.focused,
        }
    }
}

impl DropdownWidget {
    /// Creates a new dropdown with the given label, options, and theme.
    pub fn new(label: impl Into<String>, options: Vec<DropdownOption>, theme: Theme) -> Self {
        Self {
            options,
            selected_index: None,
            focused_index: 0,
            state: DropdownState::Closed,
            label: label.into(),
            placeholder: "Select...".to_string(),
            errors: HashMap::new(),
            error: None,
            theme,
            focused: false,
        }
    }

    /// Sets placeholder text shown when no value is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Returns a copy of this widget configured with the focused flag.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets an error message to be displayed on the bottom border line.
    pub fn error(mut self, error: Option<String>) -> Self {
        self.error = error;
        self
    }

    /// Sets an error message to be displayed on the bottom border line.
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// Returns the value of the currently selected option, if any.
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_index
            .and_then(|i| self.options.get(i))
            .map(|o| o.value.as_str())
    }

    /// Returns the label of the currently selected option, if any.
    pub fn selected_label(&self) -> Option<&str> {
        self.selected_index
            .and_then(|i| self.options.get(i))
            .map(|o| o.label.as_str())
    }

    /// Selects the option that matches the given value.
    pub fn set_value(&mut self, value: &str) {
        self.selected_index = self.options.iter().position(|o| o.value == value);
    }

    /// Returns true if the dropdown popup is open.
    pub fn is_open(&self) -> bool {
        self.state == DropdownState::Open
    }

    /// Toggles between open and closed states.
    pub fn toggle(&mut self) {
        self.state = match self.state {
            DropdownState::Closed => DropdownState::Open,
            DropdownState::Open => DropdownState::Closed,
        };
        if self.is_open() {
            self.focused_index = self
                .selected_index
                .unwrap_or(0)
                .min(self.options.len().saturating_sub(1));
        }
    }

    /// Opens the dropdown popup and focuses the current selection.
    pub fn open(&mut self) {
        self.state = DropdownState::Open;
        self.focused_index = self
            .selected_index
            .unwrap_or(0)
            .min(self.options.len().saturating_sub(1));
    }

    /// Closes the dropdown popup.
    pub fn close(&mut self) {
        self.state = DropdownState::Closed;
    }

    /// Moves the focused option to the next item in the list.
    pub fn select_next(&mut self) {
        if !self.options.is_empty() {
            self.focused_index = (self.focused_index + 1) % self.options.len();
        }
    }

    /// Moves the focused option to the previous item in the list.
    pub fn select_prev(&mut self) {
        if !self.options.is_empty() {
            self.focused_index = if self.focused_index == 0 {
                self.options.len() - 1
            } else {
                self.focused_index - 1
            };
        }
    }

    /// Confirms the currently focused option as the selected value.
    pub fn confirm_selection(&mut self) {
        self.selected_index = Some(self.focused_index);
        self.state = DropdownState::Closed;
    }

    /// Handles keyboard input for opening, closing, and navigating options.
    ///
    /// Returns a [`DropdownAction`] that the caller can use to react to
    /// changes or `None` when the key is ignored.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<DropdownAction> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Enter => {
                if self.is_open() {
                    self.confirm_selection();
                    Some(DropdownAction::Selected(self.selected_index))
                } else {
                    self.open();
                    Some(DropdownAction::Opened)
                }
            }
            KeyCode::Esc => {
                if self.is_open() {
                    self.close();
                    Some(DropdownAction::Closed)
                } else {
                    None
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.is_open() {
                    self.select_prev();
                    Some(DropdownAction::FocusChanged)
                } else {
                    None
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.is_open() {
                    self.select_next();
                    Some(DropdownAction::FocusChanged)
                } else {
                    self.open();
                    Some(DropdownAction::Opened)
                }
            }
            KeyCode::Tab => {
                if self.is_open() {
                    // Close dropdown and let parent handle Tab for field navigation
                    self.close();
                    Some(DropdownAction::Closed)
                } else {
                    None
                }
            }
            KeyCode::BackTab => {
                if self.is_open() {
                    self.close();
                    Some(DropdownAction::Closed)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handles mouse clicks inside or outside the dropdown area.
    ///
    /// Returns a [`DropdownAction`] when a click opens, closes, or selects
    /// a value, or `None` when the event is ignored.
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<DropdownAction> {
        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
            return None;
        }

        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        if !inner.contains((mouse.column, mouse.row).into()) {
            if self.is_open() {
                self.close();
                return Some(DropdownAction::Closed);
            }
            return None;
        }

        if self.is_open() {
            let _option_height = 1;
            let header_height = 1;
            let options_area_start = inner.y + header_height;
            let mouse_y = mouse.row;

            if mouse_y >= options_area_start {
                let relative_y = (mouse_y - options_area_start) as usize;
                if relative_y < self.options.len() {
                    self.focused_index = relative_y;
                    self.confirm_selection();
                    return Some(DropdownAction::Selected(self.selected_index));
                }
            }
        } else {
            self.open();
            return Some(DropdownAction::Opened);
        }

        None
    }
}

/// High level events produced by a [`DropdownWidget`].
#[derive(Debug, Clone)]
pub enum DropdownAction {
    /// The dropdown popup was opened.
    Opened,
    /// The dropdown popup was closed.
    Closed,
    /// A selection was confirmed, returning the selected index if present.
    Selected(Option<usize>),
    /// The focused option changed while the dropdown was open.
    FocusChanged,
}

impl Widget for DropdownWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let border_style = if self.focused {
            Style::default()
                .fg(self.theme.colors.primary)
                .bg(self.theme.colors.background_dark)
        } else {
            Style::default()
                .fg(self.theme.colors.border)
                .bg(self.theme.colors.background_dark)
        };

        let block = Block::default()
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_style(border_style);

        block.clone().render(area, buf);

        // Render error on bottom border line if present
        if self.error.is_some() && area.height >= 3 {
            let error_y = area.y + area.height - 1;
            let error_msg = self.error.as_ref().unwrap();
            let error_text = format!("  ✗ {}", error_msg);
            let error_style = Style::default()
                .fg(self.theme.colors.error)
                .bg(self.theme.colors.background_dark);
            buf.set_string(area.x, error_y, &error_text, error_style);
        }

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Fill inner area with black background
        buf.set_style(
            inner,
            Style::default().bg(self.theme.colors.background_dark),
        );

        let display_text = self.selected_label().unwrap_or(&self.placeholder);
        let text_style = if self.selected_index.is_some() {
            Style::default()
                .fg(self.theme.colors.foreground)
                .bg(self.theme.colors.background_dark)
        } else {
            Style::default()
                .fg(self.theme.colors.disabled)
                .bg(self.theme.colors.background_dark)
        };

        let max_width = inner.width.saturating_sub(2) as usize;
        let display = if display_text.len() > max_width {
            &display_text[display_text.len() - max_width..]
        } else {
            display_text
        };

        buf.set_string(inner.x + 1, inner.y, display, text_style);

        let arrow = if self.is_open() { "▼" } else { "▶" };
        let arrow_x = inner.x + inner.width - 2;
        buf.set_string(
            arrow_x,
            inner.y,
            arrow,
            Style::default()
                .fg(self.theme.colors.primary)
                .bg(self.theme.colors.background_dark),
        );

        // Guard against empty options: treat as closed state
        if self.is_open() && self.options.is_empty() {
            return;
        }

        if self.is_open() {
            let popup_height = self.options.len() as u16 + 2;

            // Try to position popup below the field
            let mut options_area =
                Rect::new(inner.x, inner.y + inner.height, inner.width, popup_height);

            // Check if popup would extend beyond terminal bottom
            if options_area.bottom() > buf.area.bottom() {
                // Try repositioning above the field
                let above_y = inner.y.saturating_sub(popup_height);
                options_area = Rect::new(inner.x, above_y, inner.width, popup_height);

                // If still out of bounds (field at top), skip rendering popup this frame
                if options_area.bottom() > buf.area.bottom()
                    || options_area.right() > buf.area.right()
                {
                    return;
                }
            }

            // Final bounds check before rendering
            if options_area.right() > buf.area.right() {
                return;
            }

            ratatui::widgets::Clear.render(options_area, buf);

            let options_block = Block::default().borders(Borders::ALL).border_style(
                Style::default()
                    .fg(self.theme.colors.primary)
                    .bg(self.theme.colors.background_dark),
            );

            options_block.clone().render(options_area, buf);

            let options_inner = options_block.inner(options_area);
            buf.set_style(
                options_inner,
                Style::default().bg(self.theme.colors.background_dark),
            );
            for (i, option) in self.options.iter().enumerate() {
                if (options_inner.y + i as u16) < options_inner.y + options_inner.height {
                    let is_selected = Some(i) == self.selected_index;
                    let is_focused = i == self.focused_index;

                    let prefix = if is_selected { "● " } else { "  " };
                    let style = if is_focused {
                        Style::default()
                            .fg(self.theme.colors.primary)
                            .bg(self.theme.colors.background_dark)
                            .add_modifier(Modifier::REVERSED)
                    } else if is_selected {
                        Style::default()
                            .fg(self.theme.colors.primary)
                            .bg(self.theme.colors.background_dark)
                    } else {
                        Style::default()
                            .fg(self.theme.colors.foreground)
                            .bg(self.theme.colors.background_dark)
                    };

                    let max_opt_width = options_inner.width.saturating_sub(4) as usize;
                    let label = if option.label.len() > max_opt_width {
                        &option.label[option.label.len() - max_opt_width..]
                    } else {
                        &option.label
                    };

                    buf.set_string(
                        options_inner.x + 1,
                        options_inner.y + i as u16,
                        prefix,
                        style,
                    );
                    buf.set_string(
                        options_inner.x + 3,
                        options_inner.y + i as u16,
                        label,
                        style,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_options() {
        let options = vec![
            DropdownOption::new("male", "Male"),
            DropdownOption::new("female", "Female"),
            DropdownOption::new("other", "Other"),
        ];

        let theme = Theme::dark();
        let dropdown = DropdownWidget::new("Gender", options, theme);

        assert_eq!(dropdown.options.len(), 3);
        assert_eq!(dropdown.selected_index, None);
    }

    #[test]
    fn test_dropdown_select() {
        let options = vec![
            DropdownOption::new("male", "Male"),
            DropdownOption::new("female", "Female"),
        ];

        let theme = Theme::dark();
        let mut dropdown = DropdownWidget::new("Gender", options, theme);

        dropdown.set_value("female");
        assert_eq!(dropdown.selected_index, Some(1));
        assert_eq!(dropdown.selected_value(), Some("female"));
        assert_eq!(dropdown.selected_label(), Some("Female"));
    }

    #[test]
    fn test_dropdown_navigation() {
        let options = vec![
            DropdownOption::new("a", "Option A"),
            DropdownOption::new("b", "Option B"),
            DropdownOption::new("c", "Option C"),
        ];

        let theme = Theme::dark();
        let mut dropdown = DropdownWidget::new("Test", options, theme);

        dropdown.open();
        assert!(dropdown.is_open());

        dropdown.select_next();
        assert_eq!(dropdown.focused_index, 1);

        dropdown.select_prev();
        assert_eq!(dropdown.focused_index, 0);

        dropdown.confirm_selection();
        assert_eq!(dropdown.selected_index, Some(0));
        assert!(!dropdown.is_open());
    }

    #[test]
    fn dropdown_render_no_panic_when_near_bottom() {
        let options = vec![
            DropdownOption::new("opt1", "Option 1"),
            DropdownOption::new("opt2", "Option 2"),
            DropdownOption::new("opt3", "Option 3"),
        ];

        let theme = Theme::dark();
        let mut dropdown = DropdownWidget::new("Test", options, theme);
        dropdown.open();

        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);

        let field_area = Rect::new(2, 8, 36, 1);
        dropdown.render(field_area, &mut buf);
    }

    #[test]
    fn dropdown_render_no_panic_with_empty_options() {
        let theme = Theme::dark();
        let mut dropdown = DropdownWidget::new("Test", vec![], theme);
        dropdown.open();

        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);

        let field_area = Rect::new(2, 5, 36, 1);
        dropdown.render(field_area, &mut buf);
    }

    #[test]
    fn dropdown_render_with_error_no_panic() {
        let options = vec![DropdownOption::new("opt1", "Option 1")];
        let theme = Theme::dark();
        let mut dropdown = DropdownWidget::new("Test", options, theme);
        dropdown.set_error(Some("This is an error".to_string()));

        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);

        let field_area = Rect::new(2, 5, 36, 3);
        dropdown.render(field_area, &mut buf);
    }

    #[test]
    fn dropdown_set_error() {
        let theme = Theme::dark();
        let mut dropdown = DropdownWidget::new("Test", vec![], theme);

        assert_eq!(dropdown.error, None);

        dropdown.set_error(Some("Error message".to_string()));
        assert_eq!(dropdown.error, Some("Error message".to_string()));

        dropdown.set_error(None);
        assert_eq!(dropdown.error, None);
    }

    #[test]
    fn dropdown_builder_error() {
        let theme = Theme::dark();
        let dropdown =
            DropdownWidget::new("Test", vec![], theme).error(Some("Builder error".to_string()));

        assert_eq!(dropdown.error, Some("Builder error".to_string()));
    }
}
