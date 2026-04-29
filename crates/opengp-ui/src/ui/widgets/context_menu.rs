use crossterm::event::{Event, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use rat_event::ct_event;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Widget};

use crate::theme::Theme;

/// Menu item shown in a [`ContextMenuState`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuItem<A> {
    pub label: String,
    pub action: A,
    pub enabled: bool,
}

/// High-level actions emitted by context menu interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction<A> {
    FocusChanged,
    Selected(A),
    Dismissed,
}

/// Floating context menu state with keyboard and mouse interaction.
#[derive(Debug, Clone)]
pub struct ContextMenuState<A> {
    pub items: Vec<ContextMenuItem<A>>,
    pub selected_index: usize,
    pub visible: bool,
    pub position: Position,
    pub theme: Theme,
    pub focus: FocusFlag,
}

impl<A> ContextMenuState<A> {
    pub fn new(theme: Theme, items: Vec<ContextMenuItem<A>>) -> Self {
        let mut state = Self {
            items,
            selected_index: 0,
            visible: false,
            position: Position::new(0, 0),
            theme,
            focus: FocusFlag::default(),
        };
        state.selected_index = state.first_enabled_index().unwrap_or(0);
        state
    }

    pub fn show_at(&mut self, position: Position) {
        self.position = position;
        self.visible = true;
        self.selected_index = self.first_enabled_index().unwrap_or(0);
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn selected_item(&self) -> Option<&ContextMenuItem<A>> {
        self.items.get(self.selected_index)
    }

    /// Returns the clamped render area for this menu within a viewport.
    pub fn menu_area(&self, viewport: Rect) -> Option<Rect> {
        if viewport.is_empty() {
            return None;
        }

        let desired_width = self
            .items
            .iter()
            .map(|item| item.label.chars().count() as u16)
            .max()
            .unwrap_or(0)
            .saturating_add(4)
            .max(8);
        let desired_height = (self.items.len() as u16).saturating_add(2).max(3);

        let width = desired_width.min(viewport.width);
        let height = desired_height.min(viewport.height);

        if width == 0 || height == 0 {
            return None;
        }

        let max_x = viewport
            .x
            .saturating_add(viewport.width)
            .saturating_sub(width);
        let max_y = viewport
            .y
            .saturating_add(viewport.height)
            .saturating_sub(height);

        let x = self.position.x.clamp(viewport.x, max_x);
        let y = self.position.y.clamp(viewport.y, max_y);

        Some(Rect::new(x, y, width, height))
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ContextMenuAction<A>>
    where
        A: Clone,
    {
        if !self.visible || key.kind != KeyEventKind::Press {
            return None;
        }

        let event = Event::Key(key);
        match &event {
            ct_event!(keycode press Up) => {
                self.move_prev_enabled();
                Some(ContextMenuAction::FocusChanged)
            }
            ct_event!(keycode press Down) => {
                self.move_next_enabled();
                Some(ContextMenuAction::FocusChanged)
            }
            ct_event!(keycode press Enter) => {
                if let Some(item) = self.selected_item() {
                    if item.enabled {
                        let action = item.action.clone();
                        self.visible = false;
                        return Some(ContextMenuAction::Selected(action));
                    }
                }
                None
            }
            ct_event!(keycode press Esc) => {
                self.visible = false;
                Some(ContextMenuAction::Dismissed)
            }
            _ => None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        viewport: Rect,
    ) -> Option<ContextMenuAction<A>> {
        if !self.visible {
            return None;
        }

        let menu_area = self.menu_area(viewport)?;
        let is_left_click = matches!(
            mouse.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left)
        );

        if !is_left_click {
            return None;
        }

        let click_position = Position::new(mouse.column, mouse.row);
        if !menu_area.contains(click_position) {
            self.visible = false;
            return Some(ContextMenuAction::Dismissed);
        }

        None
    }

    pub fn render(&self, viewport: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        let Some(menu_area) = self.menu_area(viewport) else {
            return;
        };

        Clear.render(menu_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border))
            .style(Style::default().bg(self.theme.colors.background));
        let inner = block.inner(menu_area);
        block.render(menu_area, buf);

        if inner.is_empty() {
            return;
        }

        for (i, item) in self.items.iter().take(inner.height as usize).enumerate() {
            let mut style = Style::default().fg(self.theme.colors.foreground);

            if !item.enabled {
                style = style
                    .fg(self.theme.colors.disabled)
                    .add_modifier(Modifier::DIM);
            }

            if i == self.selected_index {
                style = style
                    .bg(self.theme.colors.selected)
                    .add_modifier(Modifier::BOLD);
            }

            let y = inner.y + i as u16;
            let mut label = item.label.clone();
            let max_label_width = inner.width.saturating_sub(1) as usize;
            if label.chars().count() > max_label_width {
                label = label.chars().take(max_label_width).collect();
            }

            let padded = format!("{label:<width$}", width = inner.width as usize);
            buf.set_string(inner.x, y, padded, style);
        }
    }

    fn first_enabled_index(&self) -> Option<usize> {
        self.items.iter().position(|item| item.enabled)
    }

    fn move_next_enabled(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let len = self.items.len();
        for step in 1..=len {
            let idx = (self.selected_index + step) % len;
            if self.items[idx].enabled {
                self.selected_index = idx;
                return;
            }
        }
    }

    fn move_prev_enabled(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let len = self.items.len();
        for step in 1..=len {
            let idx = (self.selected_index + len - step) % len;
            if self.items[idx].enabled {
                self.selected_index = idx;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestAction {
        Edit,
        Delete,
        Archive,
    }

    fn build_items() -> Vec<ContextMenuItem<TestAction>> {
        vec![
            ContextMenuItem {
                label: "Edit".to_string(),
                action: TestAction::Edit,
                enabled: true,
            },
            ContextMenuItem {
                label: "Delete".to_string(),
                action: TestAction::Delete,
                enabled: false,
            },
            ContextMenuItem {
                label: "Archive".to_string(),
                action: TestAction::Archive,
                enabled: true,
            },
        ]
    }

    fn press(code: KeyCode) -> KeyEvent {
        let mut key = KeyEvent::new(code, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        key
    }

    fn click(x: u16, y: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn context_menu_show_hide_and_selected_item_work() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        assert!(!state.is_visible());

        state.show_at(Position::new(4, 5));
        assert!(state.is_visible());
        assert_eq!(
            state.selected_item().map(|i| &i.label),
            Some(&"Edit".to_string())
        );

        state.hide();
        assert!(!state.is_visible());
    }

    #[test]
    fn context_menu_down_up_navigation_skips_disabled_items() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        state.show_at(Position::new(1, 1));

        assert_eq!(
            state.handle_key(press(KeyCode::Down)),
            Some(ContextMenuAction::FocusChanged)
        );
        assert_eq!(state.selected_index, 2);

        assert_eq!(
            state.handle_key(press(KeyCode::Up)),
            Some(ContextMenuAction::FocusChanged)
        );
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn context_menu_enter_selects_and_hides() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        state.show_at(Position::new(1, 1));
        state.selected_index = 2;

        let action = state.handle_key(press(KeyCode::Enter));
        assert_eq!(
            action,
            Some(ContextMenuAction::Selected(TestAction::Archive))
        );
        assert!(!state.is_visible());
    }

    #[test]
    fn context_menu_escape_dismisses_and_hides() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        state.show_at(Position::new(1, 1));

        let action = state.handle_key(press(KeyCode::Esc));
        assert_eq!(action, Some(ContextMenuAction::Dismissed));
        assert!(!state.is_visible());
    }

    #[test]
    fn context_menu_click_outside_closes_menu() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        let viewport = Rect::new(0, 0, 40, 20);
        state.show_at(Position::new(5, 5));

        let action = state.handle_mouse(click(0, 0), viewport);
        assert_eq!(action, Some(ContextMenuAction::Dismissed));
        assert!(!state.is_visible());
    }

    #[test]
    fn context_menu_area_is_clamped_to_viewport() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        let viewport = Rect::new(0, 0, 20, 6);
        state.show_at(Position::new(99, 99));

        let area = match state.menu_area(viewport) {
            Some(area) => area,
            None => panic!("menu should render"),
        };
        assert!(area.x + area.width <= viewport.x + viewport.width);
        assert!(area.y + area.height <= viewport.y + viewport.height);
    }

    #[test]
    fn context_menu_render_draws_items_when_visible() {
        let mut state = ContextMenuState::new(Theme::dark(), build_items());
        let viewport = Rect::new(0, 0, 30, 10);
        state.show_at(Position::new(1, 1));

        let mut buf = Buffer::empty(viewport);
        state.render(viewport, &mut buf);

        let area = match state.menu_area(viewport) {
            Some(area) => area,
            None => panic!("menu should render"),
        };
        let cell = &buf[(area.x + 1, area.y + 1)];
        assert_eq!(cell.symbol(), "E");
    }
}

impl<A: Clone> HasFocus for ContextMenuState<A> {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}
