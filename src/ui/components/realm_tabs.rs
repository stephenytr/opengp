use ratatui::layout::Rect;
use ratatui::style::{Modifier as RatatuiModifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Tabs as RatatuiTabs};
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, Props, State, StateValue,
};

use crate::ui::keybinds::{Keybind, KeybindContext, KeybindRegistry};
use crate::ui::msg::Msg;
use crate::ui::theme::Theme;

#[derive(MockComponent, Clone)]
pub struct RealmTabs {
    component: TabsWidget,
    keybinds: Vec<Keybind>,
}

pub struct RealmTabsBuilder {
    titles: Vec<String>,
    selected: usize,
    keybinds: Option<Vec<Keybind>>,
}

impl RealmTabsBuilder {
    pub fn new() -> Self {
        Self {
            titles: Vec::new(),
            selected: 0,
            keybinds: None,
        }
    }

    pub fn titles(mut self, titles: Vec<&str>) -> Self {
        self.titles = titles.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn with_keybinds(mut self) -> Self {
        self.keybinds = Some(KeybindRegistry::get_keybinds(KeybindContext::Tabs));
        self
    }

    pub fn build(self) -> RealmTabs {
        let theme = Theme::new();
        let tabs = TabsWidget::default()
            .titles(&self.titles)
            .selected(self.selected)
            .normal_style(theme.normal)
            .selected_style(theme.selected)
            .highlight_style(theme.highlight);

        let keybinds = self
            .keybinds
            .unwrap_or_else(|| KeybindRegistry::get_keybinds(KeybindContext::Tabs));

        RealmTabs {
            component: tabs,
            keybinds,
        }
    }
}

impl Default for RealmTabsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmTabs {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmTabsBuilder {
        RealmTabsBuilder::new()
    }

    pub fn selected(&self) -> usize {
        self.component.selected
    }

    pub fn titles(&self) -> &[String] {
        &self.component.titles
    }

    pub fn set_selected(&mut self, index: usize) {
        self.component.selected = index;
    }

    pub fn keybinds(&self) -> &[Keybind] {
        &self.keybinds
    }
}

impl Default for RealmTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmTabs {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key_event) => self.handle_keyboard(key_event),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(_),
                column,
                ..
            }) => self.handle_tab_click(column),
            _ => None,
        }
    }
}

impl RealmTabs {
    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Msg> {
        let key = key_event.code;
        let modifiers = key_event.modifiers;

        tracing::debug!("Tabs received: key={:?}, modifiers={:?}", key, modifiers);

        for kb in &self.keybinds {
            let key_match = self.key_matches(kb.key, key);

            // For BackTab, don't check modifiers since it already implies shift
            // Also on Linux crossterm sends BackTab with modifiers=NONE
            let mod_match = if matches!(kb.key, crossterm::event::KeyCode::BackTab) {
                true
            } else {
                self.modifiers_match(kb.modifiers, modifiers)
            };

            tracing::debug!(
                "  Trying: {:?} {:?} -> key_match={}, mod_match={}",
                kb.key,
                kb.modifiers,
                key_match,
                mod_match
            );

            if key_match && mod_match {
                tracing::debug!("  MATCHED action: {}", kb.action);
                return self.execute_tab_action(kb.action);
            }
        }
        None
    }

    fn key_matches(&self, crossterm_key: crossterm::event::KeyCode, tui_key: Key) -> bool {
        use crossterm::event::KeyCode;
        match (crossterm_key, tui_key) {
            (KeyCode::Char(c1), Key::Char(c2)) => c1 == c2,
            (KeyCode::Enter, Key::Enter) => true,
            (KeyCode::Esc, Key::Esc) => true,
            (KeyCode::Tab, Key::Tab) => true,
            (KeyCode::BackTab, Key::BackTab) => true,
            (KeyCode::Up, Key::Up) => true,
            (KeyCode::Down, Key::Down) => true,
            (KeyCode::Left, Key::Left) => true,
            (KeyCode::Right, Key::Right) => true,
            (KeyCode::Home, Key::Home) => true,
            (KeyCode::End, Key::End) => true,
            (KeyCode::Backspace, Key::Backspace) => true,
            (KeyCode::Delete, Key::Delete) => true,
            _ => false,
        }
    }

    fn modifiers_match(
        &self,
        crossterm_mods: crossterm::event::KeyModifiers,
        tui_mods: KeyModifiers,
    ) -> bool {
        let has_shift =
            |m: crossterm::event::KeyModifiers| m.contains(crossterm::event::KeyModifiers::SHIFT);
        let has_ctrl =
            |m: crossterm::event::KeyModifiers| m.contains(crossterm::event::KeyModifiers::CONTROL);
        let has_alt =
            |m: crossterm::event::KeyModifiers| m.contains(crossterm::event::KeyModifiers::ALT);

        let tui_has_shift = tui_mods.intersects(KeyModifiers::SHIFT);
        let tui_has_ctrl = tui_mods.intersects(KeyModifiers::CONTROL);
        let tui_has_alt = tui_mods.intersects(KeyModifiers::ALT);

        has_shift(crossterm_mods) == tui_has_shift
            && has_ctrl(crossterm_mods) == tui_has_ctrl
            && has_alt(crossterm_mods) == tui_has_alt
    }

    fn execute_tab_action(&mut self, action: &str) -> Option<Msg> {
        match action {
            "Quit" => Some(Msg::AppClose),
            "Patients" => {
                self.component.selected = 0;
                Some(Msg::NavigateToTab(0))
            }
            "Appointments" => {
                self.component.selected = 1;
                Some(Msg::NavigateToTab(1))
            }
            "Clinical" => {
                self.component.selected = 2;
                Some(Msg::NavigateToTab(2))
            }
            "Billing" => {
                self.component.selected = 3;
                Some(Msg::NavigateToTab(3))
            }
            "Next" => {
                let current = self.component.selected;
                let next = (current + 1) % self.component.titles.len().max(1);
                self.component.selected = next;
                Some(Msg::NavigateToTab(next))
            }
            "Previous" => {
                let current = self.component.selected;
                let prev = if self.component.titles.is_empty() {
                    0
                } else if current == 0 {
                    self.component.titles.len() - 1
                } else {
                    current - 1
                };
                self.component.selected = prev;
                Some(Msg::NavigateToTab(prev))
            }
            "First" => {
                if !self.component.titles.is_empty() {
                    self.component.selected = 0;
                    Some(Msg::NavigateToTab(0))
                } else {
                    None
                }
            }
            "Last" => {
                if !self.component.titles.is_empty() {
                    self.component.selected = self.component.titles.len() - 1;
                    Some(Msg::NavigateToTab(self.component.selected))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn handle_tab_click(&mut self, column: u16) -> Option<Msg> {
        if self.component.titles.is_empty() {
            return None;
        }

        let area = self.component.last_area;
        let tab_count = self.component.titles.len();

        let available_width = area.width.saturating_sub(2);
        let tab_width = (available_width / tab_count as u16).max(1);

        let tabs_start = area.x + 1;

        for i in 0..tab_count {
            let tab_start = tabs_start + (i as u16 * tab_width);
            let tab_end = tab_start + tab_width;

            if column >= tab_start && column < tab_end && i != self.component.selected {
                self.component.selected = i;
                return Some(Msg::NavigateToTab(i));
            }
        }
        None
    }
}

#[derive(Default, Clone)]
struct TabsWidget {
    props: Props,
    titles: Vec<String>,
    selected: usize,
    normal_style: Style,
    selected_style: Style,
    highlight_style: Style,
    last_area: Rect,
}

impl TabsWidget {
    pub fn titles(mut self, titles: &[String]) -> Self {
        self.titles = titles.to_vec();
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    fn get_focus(&self) -> bool {
        self.props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag()
    }
}

impl MockComponent for TabsWidget {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        self.last_area = area;

        if self.titles.is_empty() {
            return;
        }

        let focus = self.get_focus();

        let tab_titles: Vec<Line> = self
            .titles
            .iter()
            .enumerate()
            .map(|(i, title)| {
                let style = if i == self.selected {
                    self.selected_style
                } else {
                    self.normal_style
                };

                let tab_style = if i == self.selected {
                    if focus {
                        style.add_modifier(RatatuiModifier::REVERSED)
                    } else {
                        style.add_modifier(RatatuiModifier::BOLD)
                    }
                } else if focus {
                    style.patch(self.highlight_style)
                } else {
                    style
                };

                Line::from(Span::styled(title.clone(), tab_style))
            })
            .collect();

        let block = if focus {
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.selected_style.add_modifier(RatatuiModifier::BOLD))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.normal_style)
        };

        let tabs = RatatuiTabs::new(tab_titles)
            .block(block)
            .select(self.selected)
            .style(self.normal_style)
            .highlight_style(self.selected_style);

        frame.render_widget(tabs, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(StateValue::Usize(self.selected))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Right) => {
                let next = (self.selected + 1) % self.titles.len().max(1);
                self.selected = next;
                CmdResult::Changed(State::One(StateValue::Usize(next)))
            }
            Cmd::Move(Direction::Left) => {
                let prev = if self.titles.is_empty() {
                    0
                } else if self.selected == 0 {
                    self.titles.len() - 1
                } else {
                    self.selected - 1
                };
                self.selected = prev;
                CmdResult::Changed(State::One(StateValue::Usize(prev)))
            }
            _ => CmdResult::None,
        }
    }
}
