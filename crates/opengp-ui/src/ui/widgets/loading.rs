use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::ui::theme::Theme;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Different visual styles for loading spinners.
#[derive(Debug, Clone, Copy, Default)]
pub enum SpinnerStyle {
    /// Dot based spinner.
    #[default]
    Dots,
    /// Line style spinner.
    Line,
    /// Arrow style spinner.
    Arrow,
}

impl SpinnerStyle {
    /// Returns the animation frames used for this style.
    pub fn frames(&self) -> &[&str] {
        match self {
            SpinnerStyle::Dots => SPINNER_FRAMES,
            SpinnerStyle::Line => &["-", "\\", "|", "/"],
            SpinnerStyle::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
        }
    }
}

/// Widget friendly representation of a loading spinner with a message.
#[derive(Debug, Clone)]
pub struct LoadingIndicator {
    message: String,
    frame_index: usize,
    style: SpinnerStyle,
    theme: Theme,
}

impl LoadingIndicator {
    /// Creates a new loading indicator using the given theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            message: "Loading...".to_string(),
            frame_index: 0,
            style: SpinnerStyle::default(),
            theme,
        }
    }

    /// Returns a copy of this indicator with a different message.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Returns a copy of this indicator with a different spinner style.
    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    /// Advances the spinner by one frame.
    pub fn tick(&mut self) {
        let frames = self.style.frames();
        self.frame_index = (self.frame_index + 1) % frames.len();
    }

    /// Returns the current spinner frame as a string slice.
    pub fn current_frame(&self) -> &str {
        self.style.frames()[self.frame_index]
    }
}

impl Widget for LoadingIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let spinner = self.current_frame();
        let spinner_span = Span::styled(
            spinner,
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(ratatui::style::Modifier::BOLD),
        );

        let message_span = Span::styled(
            format!(" {}", self.message),
            Style::default().fg(self.theme.colors.foreground),
        );

        let line = Line::from(vec![spinner_span, message_span]);

        let x = area.x + (area.width.saturating_sub(line.width() as u16)) / 2;
        let y = area.y + area.height / 2;

        buf.set_line(x, y, &line, area.width);
    }
}

/// Simple state holder for a loading spinner that can be turned into a widget.
#[derive(Debug, Clone, Default)]
pub struct LoadingState {
    message: String,
    frame_index: usize,
    style: SpinnerStyle,
}

impl LoadingState {
    /// Creates a new empty loading state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this state with a different message.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Returns a copy of this state with a different spinner style.
    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    /// Advances the stored frame index by one.
    pub fn tick(&mut self) {
        let frames = self.style.frames();
        self.frame_index = (self.frame_index + 1) % frames.len();
    }

    /// Converts this state into a [`LoadingIndicator`] widget.
    pub fn to_indicator(&self, theme: &Theme) -> LoadingIndicator {
        LoadingIndicator {
            message: self.message.clone(),
            frame_index: self.frame_index,
            style: self.style,
            theme: theme.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_indicator_default() {
        let theme = Theme::dark();
        let indicator = LoadingIndicator::new(theme);

        assert_eq!(indicator.message, "Loading...");
        assert_eq!(indicator.frame_index, 0);
    }

    #[test]
    fn test_loading_indicator_message() {
        let theme = Theme::dark();
        let indicator = LoadingIndicator::new(theme).message("Fetching patients...");

        assert_eq!(indicator.message, "Fetching patients...");
    }

    #[test]
    fn test_loading_indicator_tick() {
        let theme = Theme::dark();
        let mut indicator = LoadingIndicator::new(theme);

        assert_eq!(indicator.current_frame(), "⠋");

        indicator.tick();
        assert_eq!(indicator.current_frame(), "⠙");

        for _ in 0..8 {
            indicator.tick();
        }
        assert_eq!(indicator.frame_index, 9);

        indicator.tick();
        assert_eq!(indicator.frame_index, 0);
    }

    #[test]
    fn test_spinner_style_frames() {
        let dots = SpinnerStyle::Dots;
        assert_eq!(dots.frames().len(), 10);

        let line = SpinnerStyle::Line;
        assert_eq!(line.frames().len(), 4);

        let arrow = SpinnerStyle::Arrow;
        assert_eq!(arrow.frames().len(), 8);
    }

    #[test]
    fn test_loading_state() {
        let mut state = LoadingState::new().message("Saving...");
        let theme = Theme::dark();

        assert_eq!(state.message, "Saving...");

        state.tick();
        let indicator = state.to_indicator(&theme);
        assert_eq!(indicator.frame_index, 1);
    }

    #[test]
    fn test_loading_indicator_snapshot_default_dots_spinner() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
        let indicator = LoadingIndicator::new(Theme::dark()).message("Loading allergies...");

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(indicator, rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_loading_indicator_snapshot_line_spinner() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
        let mut indicator = LoadingIndicator::new(Theme::dark())
            .message("Processing...")
            .style(SpinnerStyle::Line);
        indicator.tick();
        indicator.tick();

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(indicator, rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_loading_indicator_snapshot_arrow_spinner() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
        let mut indicator = LoadingIndicator::new(Theme::dark())
            .message("Saving...")
            .style(SpinnerStyle::Arrow);
        indicator.tick();
        indicator.tick();
        indicator.tick();

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(indicator, rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }
}
