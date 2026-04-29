//! Tab Navigation Component
//!
//! Provides tab bar for switching between main application sections.

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

use crate::ui::input::DoubleClickDetector;
use crate::ui::shared::{hover_style, invert_color};
use crate::ui::theme::Theme;

/// Available tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tab {
    /// Schedule/Appointments tab
    #[default]
    Schedule,
    /// Patient search tab
    PatientSearch,
    /// Patient workspace tab (when patient workspace is open)
    PatientWorkspace,
}

impl Tab {
    /// Get the display name for the tab
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Schedule => "Schedule",
            Tab::PatientSearch => "Patient Search",
            Tab::PatientWorkspace => "Patient Workspace",
        }
    }

    /// Get the background color for the tab
    pub fn bg_color(&self, theme: &Theme) -> ratatui::style::Color {
        match self {
            Tab::Schedule => theme.colors.primary,
            Tab::PatientSearch => theme.colors.primary,
            Tab::PatientWorkspace => theme.colors.primary,
        }
    }

    /// Get the shortcut key for the tab
    pub fn shortcut(&self) -> &'static str {
        match self {
            Tab::Schedule => "F2",
            Tab::PatientSearch => "F3",
            Tab::PatientWorkspace => "",
        }
    }

    /// Get all tabs (visible in main tab bar)
    pub fn all() -> [Tab; 2] {
        [Tab::Schedule, Tab::PatientSearch]
    }

    /// Get the index of this tab
    pub fn index(&self) -> usize {
        match self {
            Tab::Schedule => 0,
            Tab::PatientSearch => 1,
            Tab::PatientWorkspace => 2,
        }
    }

    /// Get tab from index
    pub fn from_index(index: usize) -> Option<Tab> {
        match index {
            0 => Some(Tab::Schedule),
            1 => Some(Tab::PatientSearch),
            2 => Some(Tab::PatientWorkspace),
            _ => None,
        }
    }
}

/// Tab bar state
#[derive(Debug, Clone)]
pub struct TabBar {
    /// Currently selected tab
    selected: Tab,
    /// Whether the tab bar is focused
    focused: bool,
    /// Tab labels (with shortcuts)
    tabs: Vec<TabItem>,
    /// Theme for colors
    theme: Theme,
    /// Currently hovered tab index (None if not hovering)
    hovered_tab: Option<usize>,
    /// Double-click detector for tab switching
    double_click_detector: DoubleClickDetector,
    pub focus: FocusFlag,
}

/// Individual tab item
#[derive(Debug, Clone)]
struct TabItem {
    tab: Tab,
    label: String,
    shortcut: String,
}

impl TabBar {
    pub fn new(theme: Theme) -> Self {
        let tabs = Tab::all()
            .iter()
            .map(|&tab| TabItem {
                label: tab.name().to_string(),
                shortcut: tab.shortcut().to_string(),
                tab,
            })
            .collect();

        Self {
            selected: Tab::default(),
            focused: false,
            tabs,
            theme,
            hovered_tab: None,
            double_click_detector: DoubleClickDetector::default(),
            focus: FocusFlag::default(),
        }
    }

    /// Get the currently selected tab
    pub fn selected(&self) -> Tab {
        self.selected
    }

    /// Set the selected tab
    pub fn select(&mut self, tab: Tab) {
        self.selected = tab;
    }

    /// Select tab by index
    pub fn select_index(&mut self, index: usize) {
        if let Some(tab) = Tab::from_index(index) {
            self.selected = tab;
        }
    }

    /// Move to the next tab
    #[allow(clippy::unwrap_used)]
    pub fn next(&mut self) {
        let current_index = self.selected.index();
        let tab_count = Tab::all().len();
        let next_index = (current_index + 1) % tab_count;
        self.selected = Tab::from_index(next_index).unwrap();
    }

    /// Move to the previous tab
    #[allow(clippy::unwrap_used)]
    pub fn prev(&mut self) {
        let current_index = self.selected.index();
        let tab_count = Tab::all().len();
        let prev_index = if current_index == 0 {
            tab_count - 1
        } else {
            current_index - 1
        };
        self.selected = Tab::from_index(prev_index).unwrap();
    }

    /// Check if the tab bar is focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set focus state
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Handle key event
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Tab> {
        use crossterm::event::KeyEventKind;
        use crate::ui::keybinds::{KeyContext, KeybindRegistry, Action};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        let registry = KeybindRegistry::global();
        if let Some(keybind) = registry.lookup(key, KeyContext::TabBar) {
            match keybind.action {
                Action::TabBarHome => {
                    self.select_index(0);
                    Some(self.selected)
                }
                Action::TabBarEnd => {
                    self.select_index(3);
                    Some(self.selected)
                }
                Action::TabBarConfirm => Some(self.selected),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Handle mouse event
    /// Returns the clicked tab if any
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Tab> {
        use crossterm::event::MouseButton;

        // Check if mouse is within the tab bar area
        if !area.contains(Position::new(mouse.column, mouse.row)) {
            self.hovered_tab = None;
            return None;
        }

        // Calculate tab width and current tab index
        let tab_width = area.width as usize / self.tabs.len().max(1);
        let tab_index = (mouse.column.saturating_sub(area.x)) as usize / tab_width.max(1);

        match mouse.kind {
            MouseEventKind::Moved => {
                // Track hovered tab for visual feedback
                if tab_index < self.tabs.len() {
                    self.hovered_tab = Some(tab_index);
                }
                None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Check for double-click to switch tabs
                if self.double_click_detector.check_double_click_now(&mouse) {
                    if tab_index < self.tabs.len() {
                        self.selected = self.tabs[tab_index].tab;
                        self.hovered_tab = None;
                        return Some(self.selected);
                    }
                }
                None
            }
            MouseEventKind::Up(MouseButton::Left) => {
                // Single-click also switches tab (existing behavior)
                if tab_index < self.tabs.len() {
                    self.selected = self.tabs[tab_index].tab;
                    return Some(self.selected);
                }
                None
            }
            _ => None,
        }
    }

    /// Get the area occupied by a specific tab
    pub fn get_tab_area(&self, index: usize, total_area: Rect) -> Option<Rect> {
        if index >= self.tabs.len() {
            return None;
        }

        let tab_width = total_area.width as usize / self.tabs.len();
        let x = total_area.x + (index * tab_width) as u16;

        Some(Rect::new(
            x,
            total_area.y,
            tab_width as u16,
            total_area.height,
        ))
    }

    /// Get the tab bar area within a given terminal area
    pub fn area(&self, terminal: Rect) -> Rect {
        Rect::new(terminal.x, terminal.y, terminal.width, 2)
    }
}

/// Render the tab bar
impl Widget for TabBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default());

        block.render(area, buf);

        // Calculate tab width
        let tab_count = self.tabs.len();
        if tab_count == 0 {
            return;
        }

        let _tab_width = (area.width as usize / tab_count).max(1);

        for (i, tab_item) in self.tabs.iter().enumerate() {
            let tab_area = self.get_tab_area(i, area);
            if let Some(rect) = tab_area {
                let is_selected = tab_item.tab == self.selected;
                let is_focused = self.focused && is_selected;
                let is_hovered = self.hovered_tab == Some(i);

                // Build the label with shortcut
                let label = if tab_item.shortcut.is_empty() {
                    format!(" {} ", tab_item.label)
                } else {
                    format!(" {} {} ", tab_item.label, tab_item.shortcut)
                };

                let style = if is_hovered || is_selected {
                    // Both hover and selected states use the tab's color with luminance-based text color
                    let tab_bg_color = tab_item.tab.bg_color(&self.theme);
                    let tab_fg_color = invert_color(tab_bg_color);
                    Style::default()
                        .bg(tab_bg_color)
                        .fg(tab_fg_color)
                        .add_modifier(Modifier::BOLD)
                } else if is_focused {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.background)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                buf.set_string(rect.x, rect.y, label, style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_selection() {
        let mut tab_bar = TabBar::new(Theme::dark());
        assert_eq!(tab_bar.selected(), Tab::Schedule);

        tab_bar.select(Tab::PatientSearch);
        assert_eq!(tab_bar.selected(), Tab::PatientSearch);
    }

    #[test]
    fn test_tab_navigation() {
        let mut tab_bar = TabBar::new(Theme::dark());
        assert_eq!(tab_bar.selected(), Tab::Schedule);

        tab_bar.next();
        assert_eq!(tab_bar.selected(), Tab::PatientSearch);

        tab_bar.next();
        assert_eq!(tab_bar.selected(), Tab::Schedule);

        tab_bar.prev();
        assert_eq!(tab_bar.selected(), Tab::PatientSearch);
    }

    #[test]
    fn test_tab_from_index() {
        assert_eq!(Tab::from_index(0), Some(Tab::Schedule));
        assert_eq!(Tab::from_index(1), Some(Tab::PatientSearch));
        assert_eq!(Tab::from_index(2), Some(Tab::PatientWorkspace));
        assert_eq!(Tab::from_index(3), None);
    }

    #[test]
    fn test_tab_names() {
        assert_eq!(Tab::Schedule.name(), "Schedule");
        assert_eq!(Tab::PatientSearch.name(), "Patient Search");
    }
}

impl HasFocus for TabBar {
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
