use crossterm::event::Event as CrosstermEvent;

/// Minimal AppEvent enum for rat-salsa integration.
/// This enum will be expanded in Task 5 to include all request/result event variants.
/// For now, it only includes the terminal event wrapper to support GlobalState compilation.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Terminal event (keyboard, mouse, resize, etc.)
    Term(CrosstermEvent),
}

impl From<CrosstermEvent> for AppEvent {
    fn from(e: CrosstermEvent) -> Self {
        Self::Term(e)
    }
}
