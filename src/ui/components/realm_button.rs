use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::Color,
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, Props, State,
};

use crate::ui::msg::Msg;
use crate::ui::theme::Theme;

#[derive(MockComponent)]
pub struct RealmButton {
    component: ButtonWidget,
}

pub struct RealmButtonBuilder {
    label: String,
}

impl RealmButtonBuilder {
    pub fn new() -> Self {
        Self {
            label: String::new(),
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn build(self) -> RealmButton {
        let theme = Theme::new();
        let button = ButtonWidget::default()
            .foreground(Color::White)
            .background(Color::DarkGray)
            .focused(theme.focus)
            .label(&self.label);

        RealmButton { component: button }
    }
}

impl Default for RealmButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmButton {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmButtonBuilder {
        RealmButtonBuilder::new()
    }

    pub fn label(&self) -> &str {
        &self.component.label
    }
}

impl Default for RealmButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmButton {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                return Some(Msg::ButtonPressed(self.component.label.clone()));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            }) => {
                return Some(Msg::ButtonPressed(self.component.label.clone()));
            }
            _ => None,
        }
    }
}

#[derive(Default)]
struct ButtonWidget {
    props: Props,
    label: String,
}

impl ButtonWidget {
    pub fn foreground(mut self, fg: Color) -> Self {
        self.props.set(Attribute::Foreground, AttrValue::Color(fg));
        self
    }

    pub fn background(mut self, bg: Color) -> Self {
        self.props.set(Attribute::Background, AttrValue::Color(bg));
        self
    }

    pub fn focused(mut self, style: ratatui::style::Style) -> Self {
        self.props
            .set(Attribute::Custom("focused".into()), AttrValue::Style(style));
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }
}

impl MockComponent for ButtonWidget {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::ratatui::layout::Rect) {
        let focus = self
            .props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();

        let foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::White))
            .unwrap_color();

        let background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::DarkGray))
            .unwrap_color();

        let style = if focus {
            tuirealm::ratatui::style::Style::default()
                .fg(foreground)
                .bg(background)
                .add_modifier(tuirealm::ratatui::style::Modifier::BOLD)
        } else {
            tuirealm::ratatui::style::Style::default()
                .fg(foreground)
                .bg(background)
        };

        let button_text = if focus {
            format!("[ {} ]", self.label)
        } else {
            format!("  {}  ", self.label)
        };

        let paragraph = tuirealm::ratatui::widgets::Paragraph::new(button_text)
            .style(style)
            .alignment(tuirealm::ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Submit => CmdResult::Submit(State::None),
            _ => CmdResult::None,
        }
    }
}
