//! OpenGP Theme System
//!
//! Global theme configuration with color palettes and predefined themes.

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
    /// Dark background color (for Color::Black replacements)
    pub background_dark: Color,
    /// Dim text color (for Color::DarkGray replacements)
    pub text_dim: Color,
    /// Secondary text color (for Color::Gray replacements)
    pub text_secondary: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

impl ColorPalette {
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
            // Appointment status colors
            appointment_scheduled: Color::Yellow,
            appointment_confirmed: Color::Cyan,
            appointment_arrived: Color::Green,
            appointment_in_progress: Color::LightBlue,
            appointment_completed: Color::Green,
            appointment_cancelled: Color::Red,
            appointment_dna: Color::Red,
            background_dark: Color::Black,
            text_dim: Color::DarkGray,
            text_secondary: Color::Gray,
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
            // Appointment status colors
            appointment_scheduled: Color::Yellow,
            appointment_confirmed: Color::Blue,
            appointment_arrived: Color::Green,
            appointment_in_progress: Color::Cyan,
            appointment_completed: Color::Green,
            appointment_cancelled: Color::Red,
            appointment_dna: Color::Red,
            background_dark: Color::White,
            text_dim: Color::Gray,
            text_secondary: Color::DarkGray,
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
            // Appointment status colors
            appointment_scheduled: Color::Yellow,
            appointment_confirmed: Color::Cyan,
            appointment_arrived: Color::Green,
            appointment_in_progress: Color::LightBlue,
            appointment_completed: Color::LightGreen,
            appointment_cancelled: Color::LightRed,
            appointment_dna: Color::LightRed,
            background_dark: Color::Black,
            text_dim: Color::DarkGray,
            text_secondary: Color::White,
        }
    }
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
}
