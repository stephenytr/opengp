use crate::ui::components::traits::InteractiveComponent;
use crossterm::event::{KeyEvent, MouseEvent};

/// Manages focus between multiple interactive components
pub struct FocusGroup {
    components: Vec<Box<dyn InteractiveComponent>>,
    focused_index: Option<usize>,
    cycle: bool,
}

impl FocusGroup {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            focused_index: None,
            cycle: true,
        }
    }

    pub fn cycle(mut self, cycle: bool) -> Self {
        self.cycle = cycle;
        self
    }

    pub fn add(&mut self, component: Box<dyn InteractiveComponent>) {
        self.components.push(component);
        if self.focused_index.is_none() && !self.components.is_empty() {
            self.focus(0);
        }
    }

    pub fn focus(&mut self, index: usize) {
        if index < self.components.len() {
            if let Some(old) = self.focused_index {
                if let Some(comp) = self.components.get_mut(old) {
                    comp.set_focus(false);
                }
            }

            self.focused_index = Some(index);
            if let Some(comp) = self.components.get_mut(index) {
                comp.set_focus(true);
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(current) = self.focused_index {
            let next = current + 1;
            if next < self.components.len() {
                self.focus(next);
            } else if self.cycle {
                self.focus(0);
            }
        } else if !self.components.is_empty() {
            self.focus(0);
        }
    }

    pub fn previous(&mut self) {
        if let Some(current) = self.focused_index {
            if current > 0 {
                self.focus(current - 1);
            } else if self.cycle {
                self.focus(self.components.len() - 1);
            }
        } else if !self.components.is_empty() {
            self.focus(self.components.len() - 1);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if let Some(index) = self.focused_index {
            if let Some(comp) = self.components.get_mut(index) {
                return comp.on_key(key);
            }
        }
        false
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        if let Some(index) = self.focused_index {
            if let Some(comp) = self.components.get_mut(index) {
                return comp.on_mouse(mouse);
            }
        }
        false
    }

    pub fn components(&self) -> &Vec<Box<dyn InteractiveComponent>> {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut Vec<Box<dyn InteractiveComponent>> {
        &mut self.components
    }
}

impl Default for FocusGroup {
    fn default() -> Self {
        Self::new()
    }
}
