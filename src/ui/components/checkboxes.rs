pub struct CheckBoxWrapper {
    label: String,
    checked: bool,
    is_focused: bool,
}

impl CheckBoxWrapper {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            checked: false,
            is_focused: false,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }
}

impl Default for CheckBoxWrapper {
    fn default() -> Self {
        Self::new("Checkbox")
    }
}
