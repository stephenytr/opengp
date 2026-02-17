use tui_realm_stdlib::Input;
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Color},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::ui::msg::Msg;
use crate::ui::theme::Theme;

#[derive(MockComponent)]
pub struct RealmInput {
    component: Input,
}

pub struct RealmInputBuilder {
    label: String,
    placeholder: String,
    initial_value: String,
}

impl RealmInputBuilder {
    pub fn new() -> Self {
        Self {
            label: String::new(),
            placeholder: String::new(),
            initial_value: String::new(),
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn init_value(mut self, value: &str) -> Self {
        self.initial_value = value.to_string();
        self
    }

    pub fn build(self) -> RealmInput {
        let theme = Theme::new();
        let mut input = Input::default()
            .foreground(Color::White)
            .background(Color::Reset)
            .borders(
                Borders::default()
                    .color(Color::LightGreen)
                    .modifiers(BorderType::Rounded),
            )
            .title(&self.label, Alignment::Left)
            .value(&self.initial_value);

        if !self.placeholder.is_empty() {
            input = input.placeholder(&self.placeholder, theme.normal);
        }

        RealmInput { component: input }
    }
}

impl Default for RealmInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmInput {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmInputBuilder {
        RealmInputBuilder::new()
    }

    pub fn get_value(&self) -> String {
        self.component.state().unwrap_one().unwrap_string()
    }
}

impl Default for RealmInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                let value = self.get_value();
                return Some(Msg::InputSubmitted(value));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(tuirealm::command::Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(tuirealm::command::Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Delete,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::InputBlur),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::InputBlur),
            _ => return None,
        };

        match cmd_result {
            CmdResult::Changed(State::One(StateValue::String(value))) => {
                Some(Msg::InputChanged(value))
            }
            _ => None,
        }
    }
}
