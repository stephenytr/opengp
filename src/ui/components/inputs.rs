#[derive(Debug, Clone)]
pub struct InputWrapper {
    value: String,
    placeholder: String,
    label: String,
    is_focused: bool,
}

impl InputWrapper {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            label: String::new(),
            is_focused: false,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn init_value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn push_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn pop_char(&mut self) {
        self.value.pop();
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Default for InputWrapper {
    fn default() -> Self {
        Self::new()
    }
}
