use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct ModalWrapper {
    is_open: bool,
    title: String,
    message: String,
    confirm_label: String,
    cancel_label: String,
    focused_button: ModalButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalButton {
    Confirm,
    Cancel,
}

impl ModalWrapper {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            is_open: false,
            title: title.to_string(),
            message: message.to_string(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            focused_button: ModalButton::Confirm,
        }
    }

    pub fn with_labels(title: &str, message: &str, confirm: &str, cancel: &str) -> Self {
        Self {
            is_open: false,
            title: title.to_string(),
            message: message.to_string(),
            confirm_label: confirm.to_string(),
            cancel_label: cancel.to_string(),
            focused_button: ModalButton::Confirm,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.focused_button = ModalButton::Confirm;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn toggle_focus(&mut self) {
        self.focused_button = match self.focused_button {
            ModalButton::Confirm => ModalButton::Cancel,
            ModalButton::Cancel => ModalButton::Confirm,
        };
    }

    pub fn confirm_focused(&self) -> bool {
        self.focused_button == ModalButton::Confirm
    }

    pub fn set_confirm_focused(&mut self, focused: bool) {
        self.focused_button = if focused {
            ModalButton::Confirm
        } else {
            ModalButton::Cancel
        };
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn confirm_label(&self) -> &str {
        &self.confirm_label
    }

    pub fn cancel_label(&self) -> &str {
        &self.cancel_label
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ModalAction> {
        if !self.is_open {
            return None;
        }

        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Left | KeyCode::Tab => {
                self.toggle_focus();
                Some(ModalAction::Render)
            }
            KeyCode::Right => {
                self.toggle_focus();
                Some(ModalAction::Render)
            }
            KeyCode::Enter => {
                let action = if self.confirm_focused() {
                    ModalAction::Confirm
                } else {
                    ModalAction::Cancel
                };
                self.close();
                Some(action)
            }
            KeyCode::Esc => {
                self.close();
                Some(ModalAction::Cancel)
            }
            _ => None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        confirm_area: Rect,
        cancel_area: Rect,
    ) -> Option<ModalAction> {
        if !self.is_open {
            return None;
        }

        use crossterm::event::MouseEventKind;

        if mouse.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
            return None;
        }

        let col = mouse.column;
        let row = mouse.row;

        if col >= confirm_area.x
            && col < confirm_area.x + confirm_area.width
            && row >= confirm_area.y
            && row < confirm_area.y + confirm_area.height
        {
            self.close();
            return Some(ModalAction::Confirm);
        }

        if col >= cancel_area.x
            && col < cancel_area.x + cancel_area.width
            && row >= cancel_area.y
            && row < cancel_area.y + cancel_area.height
        {
            self.close();
            return Some(ModalAction::Cancel);
        }

        None
    }
}

impl Default for ModalWrapper {
    fn default() -> Self {
        Self::new("Confirm", "Are you sure?")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalAction {
    Confirm,
    Cancel,
    Render,
}
