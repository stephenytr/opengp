use crossterm::event::{Event, KeyEvent};
use rat_event::ct_event;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginFocus {
    Username,
    Password,
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginAction {
    Submit { username: String, password: String },
}

#[derive(Debug, Clone)]
pub struct LoginScreen {
    theme: Theme,
    username: String,
    password: String,
    focus: LoginFocus,
    loading: bool,
    error: Option<String>,
}

impl LoginScreen {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            username: String::new(),
            password: String::new(),
            focus: LoginFocus::Username,
            loading: false,
            error: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<LoginAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.loading {
            return None;
        }

        let event = Event::Key(key);

        match &event {
            ct_event!(keycode press Tab) => {
                self.focus = match self.focus {
                    LoginFocus::Username => LoginFocus::Password,
                    LoginFocus::Password => LoginFocus::Submit,
                    LoginFocus::Submit => LoginFocus::Username,
                };
                None
            }
            ct_event!(keycode press Down) => {
                self.focus = match self.focus {
                    LoginFocus::Username => LoginFocus::Password,
                    LoginFocus::Password => LoginFocus::Submit,
                    LoginFocus::Submit => LoginFocus::Username,
                };
                None
            }
            ct_event!(keycode press BackTab) => {
                self.focus = match self.focus {
                    LoginFocus::Username => LoginFocus::Submit,
                    LoginFocus::Password => LoginFocus::Username,
                    LoginFocus::Submit => LoginFocus::Password,
                };
                None
            }
            ct_event!(keycode press Up) => {
                self.focus = match self.focus {
                    LoginFocus::Username => LoginFocus::Submit,
                    LoginFocus::Password => LoginFocus::Username,
                    LoginFocus::Submit => LoginFocus::Password,
                };
                None
            }
            ct_event!(keycode press Backspace) => {
                match self.focus {
                    LoginFocus::Username => {
                        self.username.pop();
                    }
                    LoginFocus::Password => {
                        self.password.pop();
                    }
                    LoginFocus::Submit => {}
                }
                None
            }
            ct_event!(keycode press Enter) => {
                if self.focus != LoginFocus::Submit {
                    self.focus = match self.focus {
                        LoginFocus::Username => LoginFocus::Password,
                        LoginFocus::Password => LoginFocus::Submit,
                        LoginFocus::Submit => LoginFocus::Submit,
                    };
                    return None;
                }

                if self.username.trim().is_empty() || self.password.is_empty() {
                    self.error = Some("Username and password are required".to_string());
                    return None;
                }

                self.error = None;
                self.loading = true;

                Some(LoginAction::Submit {
                    username: self.username.clone(),
                    password: self.password.clone(),
                })
            }
            _ => {
                if let crossterm::event::KeyCode::Char(c) = key.code {
                    match self.focus {
                        LoginFocus::Username => self.username.push(c),
                        LoginFocus::Password => self.password.push(c),
                        LoginFocus::Submit => {}
                    }
                }
                None
            }
        }
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
        self.loading = false;
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    #[cfg(test)]
    pub fn password_mask(&self) -> String {
        "•".repeat(self.password.chars().count())
    }
}

impl Widget for LoginScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let popup_area = Rect {
            x: area.x + area.width.saturating_sub(70) / 2,
            y: area.y + area.height.saturating_sub(12) / 2,
            width: area.width.min(70),
            height: area.height.min(12),
        };

        Clear.render(popup_area, buf);

        let container = Block::default()
            .title(" OpenGP Login ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));
        let content = container.inner(popup_area);
        container.render(popup_area, buf);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(content);

        let title = Paragraph::new("Authenticate to continue")
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.colors.foreground));
        title.render(rows[0], buf);

        let username_style = if self.focus == LoginFocus::Username {
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.colors.foreground)
        };

        let username = Paragraph::new(Line::from(vec![
            Span::styled("Username: ", username_style),
            Span::raw(self.username),
        ]));
        username.render(rows[1], buf);

        let password_style = if self.focus == LoginFocus::Password {
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.colors.foreground)
        };

        let password = Paragraph::new(Line::from(vec![
            Span::styled("Password: ", password_style),
            Span::raw("•".repeat(self.password.chars().count())),
        ]));
        password.render(rows[2], buf);

        let button_style = if self.focus == LoginFocus::Submit {
            Style::default()
                .fg(self.theme.colors.background)
                .bg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.colors.foreground)
        };
        let button_label = if self.loading {
            "[ Logging in... ]"
        } else {
            "[ Login ]"
        };
        let button = Paragraph::new(button_label)
            .alignment(Alignment::Center)
            .style(button_style);
        button.render(rows[4], buf);

        if let Some(error) = self.error {
            let error_paragraph = Paragraph::new(Text::from(error))
                .style(Style::default().fg(self.theme.colors.error))
                .alignment(Alignment::Center);
            error_paragraph.render(rows[5], buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn masks_password_characters() {
        let mut login = LoginScreen::new(Theme::dark());

        let _ = login.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        let _ = login.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        let _ = login.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        let _ = login.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));

        assert_eq!(login.password_mask(), "•••");
    }

    #[test]
    fn submit_requires_credentials() {
        let mut login = LoginScreen::new(Theme::dark());

        let _ = login.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        let _ = login.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        let result = login.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(result.is_none());
        assert_eq!(
            login.error.as_deref(),
            Some("Username and password are required")
        );
    }

    #[test]
    fn tab_cycles_username_password_submit() {
        let mut login = LoginScreen::new(Theme::dark());

        assert_eq!(login.focus, LoginFocus::Username);

        let _ = login.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(login.focus, LoginFocus::Password);

        let _ = login.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(login.focus, LoginFocus::Submit);

        let _ = login.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(login.focus, LoginFocus::Username);
    }

    #[test]
    fn backtab_cycles_backward() {
        let mut login = LoginScreen::new(Theme::dark());

        assert_eq!(login.focus, LoginFocus::Username);

        let _ = login.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(login.focus, LoginFocus::Submit);

        let _ = login.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(login.focus, LoginFocus::Password);
    }
}
