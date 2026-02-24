use std::collections::HashMap;

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DropdownOption {
    pub value: String,
    pub label: String,
}

impl DropdownOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownState {
    #[default]
    Closed,
    Open,
}

pub struct DropdownWidget {
    pub options: Vec<DropdownOption>,
    pub selected_index: Option<usize>,
    pub focused_index: usize,
    pub state: DropdownState,
    pub label: String,
    pub placeholder: String,
    pub errors: HashMap<String, String>,
    pub theme: Theme,
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
            theme: self.theme.clone(),
        }
    }
}

impl DropdownWidget {
    pub fn new(label: impl Into<String>, options: Vec<DropdownOption>, theme: Theme) -> Self {
        Self {
            options,
            selected_index: None,
            focused_index: 0,
            state: DropdownState::Closed,
            label: label.into(),
            placeholder: "Select...".to_string(),
            errors: HashMap::new(),
            theme,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.selected_index
            .and_then(|i| self.options.get(i))
            .map(|o| o.value.as_str())
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.selected_index
            .and_then(|i| self.options.get(i))
            .map(|o| o.label.as_str())
    }

    pub fn set_value(&mut self, value: &str) {
        self.selected_index = self.options.iter().position(|o| o.value == value);
    }

    pub fn is_open(&self) -> bool {
        self.state == DropdownState::Open
    }

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

    pub fn open(&mut self) {
        self.state = DropdownState::Open;
        self.focused_index = self
            .selected_index
            .unwrap_or(0)
            .min(self.options.len().saturating_sub(1));
    }

    pub fn close(&mut self) {
        self.state = DropdownState::Closed;
    }

    pub fn select_next(&mut self) {
        if !self.options.is_empty() {
            self.focused_index = (self.focused_index + 1) % self.options.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.options.is_empty() {
            self.focused_index = if self.focused_index == 0 {
                self.options.len() - 1
            } else {
                self.focused_index - 1
            };
        }
    }

    pub fn confirm_selection(&mut self) {
        self.selected_index = Some(self.focused_index);
        self.state = DropdownState::Closed;
    }

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
            _ => None,
        }
    }

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
            let option_height = 1;
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

#[derive(Debug, Clone)]
pub enum DropdownAction {
    Opened,
    Closed,
    Selected(Option<usize>),
    FocusChanged,
}

impl Widget for DropdownWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(self.label.as_str())
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(self.theme.colors.border)
                    .bg(Color::Black),
            );

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Fill inner area with black background
        buf.set_style(inner, Style::default().bg(Color::Black));

        let display_text = self.selected_label().unwrap_or(&self.placeholder);
        let text_style = if self.selected_index.is_some() {
            Style::default()
                .fg(self.theme.colors.foreground)
                .bg(Color::Black)
        } else {
            Style::default()
                .fg(self.theme.colors.disabled)
                .bg(Color::Black)
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
                .bg(Color::Black),
        );

        if self.is_open() && !self.options.is_empty() {
            let options_area = Rect::new(
                inner.x,
                inner.y + inner.height,
                inner.width,
                self.options.len() as u16 + 2,
            );

            ratatui::widgets::Clear.render(options_area, buf);

            let options_block = Block::default().borders(Borders::ALL).border_style(
                Style::default()
                    .fg(self.theme.colors.primary)
                    .bg(Color::Black),
            );

            options_block.clone().render(options_area, buf);

            let options_inner = options_block.inner(options_area);
            buf.set_style(options_inner, Style::default().bg(Color::Black));
            for (i, option) in self.options.iter().enumerate() {
                if (options_inner.y + i as u16) < options_inner.y + options_inner.height {
                    let is_selected = Some(i) == self.selected_index;
                    let is_focused = i == self.focused_index;

                    let prefix = if is_selected { "● " } else { "  " };
                    let style = if is_focused {
                        Style::default()
                            .fg(self.theme.colors.primary)
                            .bg(Color::Black)
                            .add_modifier(Modifier::REVERSED)
                    } else if is_selected {
                        Style::default()
                            .fg(self.theme.colors.primary)
                            .bg(Color::Black)
                    } else {
                        Style::default()
                            .fg(self.theme.colors.foreground)
                            .bg(Color::Black)
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
}
