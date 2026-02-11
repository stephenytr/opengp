pub mod appointment;
pub mod clinical;
pub mod patient;

use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::ui::event::Event;

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Tick,
    Render,
    Quit,

    NavigateToPatients,
    NavigateToAppointments,
    NavigateToClinical,
    NavigateToBilling,
}

#[async_trait]
pub trait Component: Send {
    async fn init(&mut self) -> crate::error::Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Action {
        match event {
            Some(Event::Key(key)) => self.handle_key_events(key),
            Some(Event::Mouse(mouse)) => self.handle_mouse_events(mouse),
            Some(Event::Tick) => Action::Tick,
            Some(Event::Render) => Action::Render,
            _ => Action::None,
        }
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Action {
        Action::None
    }

    fn handle_mouse_events(&mut self, _mouse: MouseEvent) -> Action {
        Action::None
    }

    async fn update(&mut self, _action: Action) -> crate::error::Result<Option<Action>> {
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect);
}
