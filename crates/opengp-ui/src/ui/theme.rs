//! OpenGP Theme System
//!
//! Global theme configuration with color palettes and predefined themes.

use opengp_config::ColorPalette as ThemeThemePalette;
use ratatui::style::Color;

/// Color palette for OpenGP TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPalette {
    /// Primary brand color
    pub primary: Color,
    /// Secondary/accent color
    pub secondary: Color,
    /// Main background color
    pub background: Color,
    /// Main foreground/text color
    pub foreground: Color,
    /// Error color
    pub error: Color,
    /// Success color
    pub success: Color,
    /// Warning color
    pub warning: Color,
    /// Info/informational color
    pub info: Color,
    /// Border color
    pub border: Color,
    /// Selected/focused item background
    pub selected: Color,
    /// Highlighted item background
    pub highlight: Color,
    /// Disabled/inactive color
    pub disabled: Color,
    /// Scrollbar background
    pub scrollbar_bg: Color,
    /// Scrollbar thumb
    pub scrollbar_thumb: Color,
    /// Appointment status: Scheduled
    pub appointment_scheduled: Color,
    /// Appointment status: Confirmed
    pub appointment_confirmed: Color,
    /// Appointment status: Arrived
    pub appointment_arrived: Color,
    /// Appointment status: In Progress
    pub appointment_in_progress: Color,
    /// Appointment status: Completed
    pub appointment_completed: Color,
    /// Appointment status: Cancelled
    pub appointment_cancelled: Color,
    /// Appointment status: Did Not Attend
    pub appointment_dna: Color,
    /// Appointment status: Rescheduled
    pub appointment_rescheduled: Color,
    /// Dark background color (for Color::Black replacements)
    pub background_dark: Color,
    /// Dim text color (for Color::DarkGray replacements)
    pub text_dim: Color,
    /// Secondary text color (for Color::Gray replacements)
    pub text_secondary: Color,
    /// Patient tab colours (8-colour palette for round-robin assignment)
    pub patient_tab_colours: [Color; 8],
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

impl ColorPalette {
    pub fn from_config(config: &ThemeThemePalette) -> Self {
        Self {
            primary: parse_color(&config.primary),
            secondary: parse_color(&config.secondary),
            background: parse_color(&config.background),
            foreground: parse_color(&config.foreground),
            error: parse_color(&config.error),
            success: parse_color(&config.success),
            warning: parse_color(&config.warning),
            info: parse_color(&config.info),
            border: parse_color(&config.border),
            selected: parse_color(&config.selected),
            highlight: parse_color(&config.highlight),
            disabled: parse_color(&config.disabled),
            scrollbar_bg: parse_color(&config.scrollbar_bg),
            scrollbar_thumb: parse_color(&config.scrollbar_thumb),
            appointment_scheduled: parse_color(&config.appointment_scheduled),
            appointment_confirmed: parse_color(&config.appointment_confirmed),
            appointment_arrived: parse_color(&config.appointment_arrived),
            appointment_in_progress: parse_color(&config.appointment_in_progress),
            appointment_completed: parse_color(&config.appointment_completed),
            appointment_cancelled: parse_color(&config.appointment_cancelled),
            appointment_dna: parse_color(&config.appointment_dna),
            appointment_rescheduled: parse_color(&config.appointment_rescheduled),
            background_dark: parse_color(&config.background_dark),
            text_dim: parse_color(&config.text_dim),
            text_secondary: parse_color(&config.text_secondary),
            patient_tab_colours: Self::default_patient_tab_colours(),
        }
    }

    fn default_patient_tab_colours() -> [Color; 8] {
        [
            Color::Rgb(100, 180, 220),
            Color::Rgb(255, 140, 100),
            Color::Rgb(100, 200, 180),
            Color::Rgb(255, 200, 80),
            Color::Rgb(200, 160, 220),
            Color::Rgb(140, 180, 120),
            Color::Rgb(220, 140, 160),
            Color::Rgb(120, 200, 240),
        ]
    }

    pub fn patient_colour(&self, index: usize) -> Color {
        self.patient_tab_colours[index % 8]
    }

    /// Dark theme color palette (default)
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Magenta,
            background: Color::Black,
            foreground: Color::White,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            info: Color::Blue,
            border: Color::DarkGray,
            selected: Color::Blue,
            highlight: Color::LightBlue,
            disabled: Color::Gray,
            scrollbar_bg: Color::DarkGray,
            scrollbar_thumb: Color::Gray,
            appointment_scheduled: Color::Yellow,
            appointment_confirmed: Color::Cyan,
            appointment_arrived: Color::Green,
            appointment_in_progress: Color::LightBlue,
            appointment_completed: Color::Green,
            appointment_cancelled: Color::Red,
            appointment_dna: Color::Red,
            appointment_rescheduled: Color::Rgb(180, 100, 20),
            background_dark: Color::Black,
            text_dim: Color::DarkGray,
            text_secondary: Color::Gray,
            patient_tab_colours: Self::default_patient_tab_colours(),
        }
    }

    /// Light theme color palette
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Magenta,
            background: Color::White,
            foreground: Color::Black,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            info: Color::Blue,
            border: Color::Gray,
            selected: Color::LightBlue,
            highlight: Color::Cyan,
            disabled: Color::Gray,
            scrollbar_bg: Color::Gray,
            scrollbar_thumb: Color::DarkGray,
            appointment_scheduled: Color::Yellow,
            appointment_confirmed: Color::Blue,
            appointment_arrived: Color::Green,
            appointment_in_progress: Color::Cyan,
            appointment_completed: Color::Green,
            appointment_cancelled: Color::Red,
            appointment_dna: Color::Red,
            appointment_rescheduled: Color::Rgb(200, 120, 0),
            background_dark: Color::White,
            text_dim: Color::Gray,
            text_secondary: Color::DarkGray,
            patient_tab_colours: Self::default_patient_tab_colours(),
        }
    }

    /// High contrast dark theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            primary: Color::White,
            secondary: Color::Yellow,
            background: Color::Black,
            foreground: Color::White,
            error: Color::LightRed,
            success: Color::LightGreen,
            warning: Color::LightYellow,
            info: Color::LightBlue,
            border: Color::White,
            selected: Color::LightBlue,
            highlight: Color::Yellow,
            disabled: Color::DarkGray,
            scrollbar_bg: Color::DarkGray,
            scrollbar_thumb: Color::White,
            appointment_scheduled: Color::Yellow,
            appointment_confirmed: Color::Cyan,
            appointment_arrived: Color::Green,
            appointment_in_progress: Color::LightBlue,
            appointment_completed: Color::LightGreen,
            appointment_cancelled: Color::LightRed,
            appointment_dna: Color::LightRed,
            appointment_rescheduled: Color::LightYellow,
            background_dark: Color::Black,
            text_dim: Color::DarkGray,
            text_secondary: Color::White,
            patient_tab_colours: Self::default_patient_tab_colours(),
        }
    }
}

fn parse_color(value: &str) -> Color {
    let value = value.trim();

    if let Some(color) = parse_rgb_color(value) {
        return color;
    }

    if let Some(color) = parse_indexed_color(value) {
        return color;
    }

    match value {
        "Reset" => Color::Reset,
        "Black" => Color::Black,
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Cyan" => Color::Cyan,
        "Gray" => Color::Gray,
        "DarkGray" => Color::DarkGray,
        "LightRed" => Color::LightRed,
        "LightGreen" => Color::LightGreen,
        "LightYellow" => Color::LightYellow,
        "LightBlue" => Color::LightBlue,
        "LightMagenta" => Color::LightMagenta,
        "LightCyan" => Color::LightCyan,
        "White" => Color::White,
        _ => Color::Reset,
    }
}

fn parse_rgb_color(value: &str) -> Option<Color> {
    let inner = value.strip_prefix("Rgb(")?.strip_suffix(')')?;
    let mut components = inner.split(',').map(str::trim);

    let r = components.next()?.parse::<u8>().ok()?;
    let g = components.next()?.parse::<u8>().ok()?;
    let b = components.next()?.parse::<u8>().ok()?;

    if components.next().is_some() {
        return None;
    }

    Some(Color::Rgb(r, g, b))
}

fn parse_indexed_color(value: &str) -> Option<Color> {
    let inner = value.strip_prefix("Indexed(")?.strip_suffix(')')?;
    let idx = inner.trim().parse::<u8>().ok()?;
    Some(Color::Indexed(idx))
}

/// Font configuration for the TUI
#[derive(Debug, Clone, Copy, Default)]
pub struct FontConfig {
    /// Whether to use bold text
    pub use_bold: bool,
    /// Whether to use italic text
    pub use_italic: bool,
}

/// Spacing configuration
#[derive(Debug, Clone, Copy)]
pub struct SpacingConfig {
    /// Base unit for spacing
    pub base: u16,
    /// Compact spacing
    pub compact: u16,
    /// Comfortable spacing
    pub comfortable: u16,
    /// Spacious spacing
    pub spacious: u16,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            base: 1,
            compact: 0,
            comfortable: 1,
            spacious: 2,
        }
    }
}

/// Global theme for OpenGP TUI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Color palette
    pub colors: ColorPalette,
    /// Font configuration
    pub fonts: FontConfig,
    /// Spacing configuration
    pub spacing: SpacingConfig,
    /// Whether to show scrollbars
    pub show_scrollbars: bool,
    /// Whether to use mouse support
    pub mouse_support: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create a new theme with the given palette
    pub fn new(colors: ColorPalette) -> Self {
        Self {
            colors,
            fonts: FontConfig::default(),
            spacing: SpacingConfig::default(),
            show_scrollbars: true,
            mouse_support: true,
        }
    }

    /// Create the default dark theme
    pub fn dark() -> Self {
        Self {
            colors: ColorPalette::dark(),
            fonts: FontConfig::default(),
            spacing: SpacingConfig::default(),
            show_scrollbars: true,
            mouse_support: true,
        }
    }

    /// Create the light theme
    pub fn light() -> Self {
        Self {
            colors: ColorPalette::light(),
            fonts: FontConfig::default(),
            spacing: SpacingConfig::default(),
            show_scrollbars: true,
            mouse_support: true,
        }
    }

    /// Create the high contrast theme
    pub fn high_contrast() -> Self {
        Self {
            colors: ColorPalette::high_contrast(),
            fonts: FontConfig {
                use_bold: true,
                use_italic: false,
            },
            spacing: SpacingConfig::default(),
            show_scrollbars: true,
            mouse_support: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_palette_default() {
        let palette = ColorPalette::dark();
        assert_eq!(palette.background, Color::Black);
        assert_eq!(palette.foreground, Color::White);
    }

    #[test]
    fn test_light_palette() {
        let palette = ColorPalette::light();
        assert_eq!(palette.background, Color::White);
        assert_eq!(palette.foreground, Color::Black);
    }

    #[test]
    fn test_theme_default_is_dark() {
        let theme = Theme::default();
        assert_eq!(theme.colors.background, Color::Black);
    }

    #[test]
    fn test_theme_presets() {
        let dark = Theme::dark();
        let light = Theme::light();
        let hc = Theme::high_contrast();

        assert_eq!(dark.colors.background, Color::Black);
        assert_eq!(light.colors.background, Color::White);
        assert_eq!(hc.colors.background, Color::Black);
    }

    #[test]
    fn test_patient_colour_round_robin() {
        let palette = ColorPalette::dark();

        // Test first colour
        let colour_0 = palette.patient_colour(0);
        assert_eq!(colour_0, palette.patient_tab_colours[0]);

        // Test all 8 colours
        for i in 0..8 {
            assert_eq!(palette.patient_colour(i), palette.patient_tab_colours[i]);
        }

        // Test round-robin wrapping: index 8 should equal index 0
        assert_eq!(palette.patient_colour(8), palette.patient_colour(0));

        // Test larger indices wrap correctly
        assert_eq!(palette.patient_colour(16), palette.patient_colour(0));
        assert_eq!(palette.patient_colour(17), palette.patient_colour(1));
    }
}
