use ratatui::layout::Rect;
use ratatui::style::{Modifier as RatatuiModifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Tabs as RatatuiTabs};
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, Props, State, StateValue,
};

use crate::ui::msg::Msg;
use crate::ui::theme::Theme;

#[derive(MockComponent, Clone)]
pub struct RealmTabs {
    component: TabsWidget,
}

pub struct RealmTabsBuilder {
    titles: Vec<String>,
    selected: usize,
}

impl RealmTabsBuilder {
    pub fn new() -> Self {
        Self {
            titles: Vec::new(),
            selected: 0,
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

    pub fn build(self) -> RealmTabs {
        let theme = Theme::new();
        let tabs = TabsWidget::default()
            .titles(&self.titles)
            .selected(self.selected)
            .normal_style(theme.normal)
            .selected_style(theme.selected)
            .highlight_style(theme.highlight);

        RealmTabs { component: tabs }
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
}

impl Default for RealmTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmTabs {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                let current = self.component.selected;
                let next = (current + 1) % self.component.titles.len().max(1);
                self.component.selected = next;
                Some(Msg::NavigateToTab(next))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => {
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
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => {
                let current = self.component.selected;
                let next = (current + 1) % self.component.titles.len().max(1);
                self.component.selected = next;
                Some(Msg::NavigateToTab(next))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
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
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => {
                if !self.component.titles.is_empty() {
                    self.component.selected = 0;
                    Some(Msg::NavigateToTab(0))
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => {
                if !self.component.titles.is_empty() {
                    self.component.selected = self.component.titles.len() - 1;
                    Some(Msg::NavigateToTab(self.component.selected))
                } else {
                    None
                }
            }
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

            if column >= tab_start && column < tab_end {
                if i != self.component.selected {
                    self.component.selected = i;
                    return Some(Msg::NavigateToTab(i));
                }
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
