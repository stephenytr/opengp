//! Shared input handling helpers and traits
//!
//! Provides:
//! - `is_key_press()`: Guards against non-press key events
//! - `HandleEvent` trait: For components that handle keyboard events
//! - `HandleMouse` trait: For components that handle mouse events

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use ratatui::layout::Rect;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Maximum duration between clicks to count as a double click.
pub const DOUBLE_CLICK_THRESHOLD_MS: u64 = 300;

/// Standard number of lines to scroll per mouse wheel event.
pub const SCROLL_LINES: usize = 3;

/// Tracks hover target and current pointer position.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HoverState<T> {
    pub element_id: Option<T>,
    pub position: Option<(u16, u16)>,
}

impl<T> HoverState<T> {
    /// Creates an empty hover state.
    pub fn new() -> Self {
        Self {
            element_id: None,
            position: None,
        }
    }

    /// Sets the currently hovered element and pointer position.
    pub fn set_hovered(&mut self, element_id: T, position: (u16, u16)) {
        self.element_id = Some(element_id);
        self.position = Some(position);
    }

    /// Clears hover tracking.
    pub fn clear_hover(&mut self) {
        self.element_id = None;
        self.position = None;
    }

    /// Returns true when an element is currently hovered.
    pub fn is_hovered(&self) -> bool {
        self.element_id.is_some() && self.position.is_some()
    }
}

/// High-level hover transition emitted by mouse handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverEvent {
    Enter,
    Leave,
    Move,
}

/// Abstraction for time source used by `DoubleClickDetector`.
pub trait Clock {
    /// Returns current instant.
    fn now(&self) -> Instant;
}

/// System time source backed by `Instant::now()`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Tracks click timing/position to detect double clicks.
pub struct DoubleClickDetector {
    clock: Arc<dyn Clock + Send + Sync>,
    last_click_time: Option<Instant>,
    last_click_position: Option<(u16, u16)>,
}

impl std::fmt::Debug for DoubleClickDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubleClickDetector")
            .field("last_click_time", &self.last_click_time)
            .field("last_click_position", &self.last_click_position)
            .finish()
    }
}

impl Clone for DoubleClickDetector {
    fn clone(&self) -> Self {
        Self {
            clock: Arc::clone(&self.clock),
            last_click_time: self.last_click_time,
            last_click_position: self.last_click_position,
        }
    }
}

impl Default for DoubleClickDetector {
    fn default() -> Self {
        Self {
            clock: Arc::new(SystemClock::default()),
            last_click_time: None,
            last_click_position: None,
        }
    }
}

impl DoubleClickDetector {
    /// Creates a detector with an injected clock source.
    pub fn with_clock(clock: Arc<dyn Clock + Send + Sync>) -> Self {
        Self {
            clock,
            last_click_time: None,
            last_click_position: None,
        }
    }

    /// Checks for a double click using the provided clock.
    ///
    /// Double click requires exact position match (column and row) and
    /// interval under `DOUBLE_CLICK_THRESHOLD_MS`.
    pub fn check_double_click(&mut self, mouse: &MouseEvent, clock: &dyn Clock) -> bool {
        let now = clock.now();
        let current_position = (mouse.column, mouse.row);
        let threshold = Duration::from_millis(DOUBLE_CLICK_THRESHOLD_MS);

        let is_double_click = self
            .last_click_time
            .zip(self.last_click_position)
            .is_some_and(|(last_time, last_position)| {
                last_position == current_position
                    && now.saturating_duration_since(last_time) < threshold
            });

        if is_double_click {
            self.last_click_time = None;
            self.last_click_position = None;
            return true;
        }

        self.last_click_time = Some(now);
        self.last_click_position = Some(current_position);
        false
    }

    /// Checks for a double click using the detector's configured clock.
    pub fn check_double_click_now(&mut self, mouse: &MouseEvent) -> bool {
        let clock = Arc::clone(&self.clock);
        self.check_double_click(mouse, clock.as_ref())
    }
}

/// Checks if a key event is a press event (not repeat or release)
///
/// Some terminals send Release events; components should guard against them.
/// Returns true if `key.kind == KeyEventKind::Press`.
pub fn is_key_press(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
}

/// Trait for components that handle keyboard events
///
/// Components implementing this trait can process KeyEvent and produce
/// an optional action as output.
pub trait HandleEvent {
    /// Action type produced by handling an event
    type Action;

    /// Handles a keyboard event and returns an optional action
    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Action>;
}

/// Trait for components that handle mouse events
///
/// Components implementing this trait can process MouseEvent within
/// a specific rendering area and produce an optional action as output,
/// including click, scroll, and hover-driven interactions.
pub trait HandleMouse {
    /// Action type produced by handling a mouse event
    type Action;

    /// Handles a mouse event within the given area and returns an optional action
    fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Self::Action>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseEventKind};

    struct MockClock {
        current: Instant,
    }

    impl Clock for MockClock {
        fn now(&self) -> Instant {
            self.current
        }
    }

    fn left_click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn is_key_press_returns_true_for_press_event() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        assert!(is_key_press(&key));
    }

    #[test]
    fn is_key_press_returns_false_for_release_event() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert!(!is_key_press(&key));
    }

    #[test]
    fn is_key_press_returns_false_for_repeat_event() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Repeat;
        assert!(!is_key_press(&key));
    }

    #[test]
    fn handle_event_trait_can_be_implemented() {
        #[derive(Debug)]
        enum TestAction {
            Pressed,
        }

        struct TestComponent;

        impl HandleEvent for TestComponent {
            type Action = TestAction;

            fn handle_key(&mut self, _key: KeyEvent) -> Option<Self::Action> {
                Some(TestAction::Pressed)
            }
        }

        let mut component = TestComponent;
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action = component.handle_key(key);
        assert!(matches!(action, Some(TestAction::Pressed)));
    }

    #[test]
    fn handle_mouse_trait_can_be_implemented() {
        #[derive(Debug)]
        enum TestAction {
            Clicked,
        }

        struct TestComponent;

        impl HandleMouse for TestComponent {
            type Action = TestAction;

            fn handle_mouse(&mut self, _mouse: MouseEvent, _area: Rect) -> Option<Self::Action> {
                Some(TestAction::Clicked)
            }
        }

        let mut component = TestComponent;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };
        let action = component.handle_mouse(mouse, area);
        assert!(matches!(action, Some(TestAction::Clicked)));
    }

    #[test]
    fn double_click_detector_returns_true_for_rapid_clicks_at_same_position() {
        let mut detector = DoubleClickDetector::default();
        let mouse = left_click(10, 5);
        let start = Instant::now();

        let first_clock = MockClock { current: start };
        let second_clock = MockClock {
            current: start + Duration::from_millis(250),
        };

        assert!(!detector.check_double_click(&mouse, &first_clock));
        assert!(detector.check_double_click(&mouse, &second_clock));
    }

    #[test]
    fn double_click_detector_returns_false_for_clicks_at_different_positions() {
        let mut detector = DoubleClickDetector::default();
        let start = Instant::now();

        let first_clock = MockClock { current: start };
        let second_clock = MockClock {
            current: start + Duration::from_millis(100),
        };

        assert!(!detector.check_double_click(&left_click(10, 5), &first_clock));
        assert!(!detector.check_double_click(&left_click(11, 5), &second_clock));
    }

    #[test]
    fn double_click_detector_returns_false_for_clicks_beyond_threshold() {
        let mut detector = DoubleClickDetector::default();
        let mouse = left_click(10, 5);
        let start = Instant::now();

        let first_clock = MockClock { current: start };
        let second_clock = MockClock {
            current: start + Duration::from_millis(301),
        };

        assert!(!detector.check_double_click(&mouse, &first_clock));
        assert!(!detector.check_double_click(&mouse, &second_clock));
    }

    #[test]
    fn hover_state_new_starts_empty() {
        let state = HoverState::<u32>::new();
        assert!(!state.is_hovered());
        assert_eq!(state.element_id, None);
        assert_eq!(state.position, None);
    }

    #[test]
    fn hover_state_set_hovered_tracks_element_and_position() {
        let mut state = HoverState::<&str>::new();

        state.set_hovered("row-1", (12, 7));

        assert!(state.is_hovered());
        assert_eq!(state.element_id, Some("row-1"));
        assert_eq!(state.position, Some((12, 7)));
    }

    #[test]
    fn hover_state_set_hovered_replaces_previous_target() {
        let mut state = HoverState::<u8>::new();
        state.set_hovered(1, (3, 4));

        state.set_hovered(2, (9, 10));

        assert!(state.is_hovered());
        assert_eq!(state.element_id, Some(2));
        assert_eq!(state.position, Some((9, 10)));
    }

    #[test]
    fn hover_state_clear_hover_resets_state() {
        let mut state = HoverState::<u8>::new();
        state.set_hovered(7, (1, 1));

        state.clear_hover();

        assert!(!state.is_hovered());
        assert_eq!(state.element_id, None);
        assert_eq!(state.position, None);
    }
}
