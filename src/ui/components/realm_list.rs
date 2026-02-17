use tui_realm_stdlib::List;
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::ui::msg::Msg;

#[derive(MockComponent)]
pub struct RealmList {
    component: List,
}

pub struct RealmListBuilder {
    title: String,
    items: Vec<String>,
    selected_index: Option<usize>,
    scrollable: bool,
}

impl RealmListBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            items: Vec::new(),
            selected_index: Some(0),
            scrollable: true,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn items(mut self, items: Vec<&str>) -> Self {
        self.items = items.iter().map(|s| s.to_string()).collect();
        if self.items.is_empty() {
            self.selected_index = None;
        }
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn build(self) -> RealmList {
        let mut table_builder = TableBuilder::default();
        for item in &self.items {
            table_builder.add_col(TextSpan::from(item.as_str()));
        }
        let table = table_builder.build();

        let mut list = List::default()
            .foreground(Color::White)
            .background(Color::Reset)
            .borders(
                Borders::default()
                    .color(Color::LightGreen)
                    .modifiers(BorderType::Rounded),
            )
            .title(&self.title, Alignment::Left)
            .rows(table)
            .scroll(self.scrollable);

        if let Some(index) = self.selected_index {
            if index < self.items.len() {
                list = list.selected_line(index);
            }
        }

        RealmList { component: list }
    }
}

impl Default for RealmListBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmList {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmListBuilder {
        RealmListBuilder::new()
    }

    pub fn get_index(&self) -> Option<usize> {
        match self.component.state() {
            State::One(StateValue::Usize(index)) => Some(index),
            _ => None,
        }
    }
}

impl Default for RealmList {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(index) = self.get_index() {
                    return Some(Msg::ListItemSelected(index, format!("{}", index)));
                }
                return None;
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(index) = self.get_index() {
                    return Some(Msg::ListItemActivated(index, format!("{}", index)));
                }
                return None;
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(tuirealm::command::Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(tuirealm::command::Position::Begin)),
            _ => return None,
        };

        match cmd_result {
            CmdResult::Changed(State::One(StateValue::Usize(_index))) => Some(Msg::ListScrollDown),
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(Msg::ListItemSelected(index, format!("{}", index)))
            }
            _ => None,
        }
    }
}
