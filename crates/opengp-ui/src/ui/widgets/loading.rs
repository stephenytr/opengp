use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::ui::theme::Theme;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Debug, Clone, Copy, Default)]
pub enum SpinnerStyle {
    #[default]
    Dots,
    Line,
    Arrow,
}

impl SpinnerStyle {
    pub fn frames(&self) -> &[&str] {
        match self {
            SpinnerStyle::Dots => SPINNER_FRAMES,
            SpinnerStyle::Line => &["-", "\\", "|", "/"],
            SpinnerStyle::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadingIndicator {
    message: String,
    frame_index: usize,
    style: SpinnerStyle,
    theme: Theme,
}

impl LoadingIndicator {
    pub fn new(theme: Theme) -> Self {
        Self {
            message: "Loading...".to_string(),
            frame_index: 0,
            style: SpinnerStyle::default(),
            theme,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn tick(&mut self) {
        let frames = self.style.frames();
        self.frame_index = (self.frame_index + 1) % frames.len();
    }

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

#[derive(Debug, Clone, Default)]
pub struct LoadingState {
    message: String,
    frame_index: usize,
    style: SpinnerStyle,
}

impl LoadingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn tick(&mut self) {
        let frames = self.style.frames();
        self.frame_index = (self.frame_index + 1) % frames.len();
    }

    pub fn to_indicator(&self, theme: Theme) -> LoadingIndicator {
        LoadingIndicator {
            message: self.message.clone(),
            frame_index: self.frame_index,
            style: self.style,
            theme,
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
        let indicator = state.to_indicator(theme);
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
