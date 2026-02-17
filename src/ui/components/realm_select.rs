use tui_realm_stdlib::Select;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Color},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::ui::msg::Msg;

#[derive(MockComponent)]
pub struct RealmSelect {
    component: Select,
}

pub struct RealmSelectBuilder {
    title: String,
    options: Vec<String>,
    selected_index: Option<usize>,
}

impl RealmSelectBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            options: Vec::new(),
            selected_index: Some(0),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn options(mut self, options: Vec<&str>) -> Self {
        self.options = options.iter().map(|s| s.to_string()).collect();
        if self.options.is_empty() {
            self.selected_index = None;
        }
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    pub fn build(self) -> RealmSelect {
        let mut select = Select::default()
            .foreground(Color::White)
            .background(Color::Reset)
            .borders(
                Borders::default()
                    .color(Color::LightGreen)
                    .modifiers(BorderType::Rounded),
            )
            .title(&self.title, Alignment::Left)
            .choices(&self.options);

        if let Some(index) = self.selected_index {
            if index < self.options.len() {
                select = select.value(index);
            }
        }

        RealmSelect { component: select }
    }
}

impl Default for RealmSelectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmSelect {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmSelectBuilder {
        RealmSelectBuilder::new()
    }

    pub fn get_value(&self) -> Option<String> {
        self.component
            .states
            .choices
            .get(self.component.states.selected)
            .cloned()
    }

    pub fn get_index(&self) -> usize {
        self.component.states.selected
    }
}

impl Default for RealmSelect {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(tuirealm::command::Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(tuirealm::command::Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::SelectClose),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::SelectClose),
            _ => return None,
        };

        match cmd_result {
            CmdResult::Changed(State::One(StateValue::Usize(index))) => {
                let value = self
                    .component
                    .states
                    .choices
                    .get(index)
                    .cloned()
                    .unwrap_or_default();
                Some(Msg::SelectChanged(index, value))
            }
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                let value = self
                    .component
                    .states
                    .choices
                    .get(index)
                    .cloned()
                    .unwrap_or_default();
                Some(Msg::SelectChanged(index, value))
            }
            _ => None,
        }
    }
}
