pub struct ButtonWrapper {
    label: String,
    is_focused: bool,
    on_click: Option<Box<dyn Fn() -> crate::components::Action + Send>>,
}

impl ButtonWrapper {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            is_focused: false,
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, action: F) -> Self
    where
        F: Fn() -> crate::components::Action + Send + 'static,
    {
        self.on_click = Some(Box::new(action));
        self
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn clicked(&self) -> Option<crate::components::Action> {
        self.on_click.as_ref().map(|f| f())
    }
}

impl Default for ButtonWrapper {
    fn default() -> Self {
        Self::new("Button")
    }
}
