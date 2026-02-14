//! Status badge widget with color variants.
//!
//! This module provides a `StatusBadge` struct for displaying colored
//! status indicators in the UI.

use ratatui::style::Color;

/// A colored status badge for displaying status indicators.
///
/// This struct holds the text and color for a status badge.
/// It is a data structure only - rendering is handled by the UI layer.
///
/// # Fields
///
/// * `text` - The badge label text
/// * `color` - The badge background/foreground color
///
/// # Examples
///
/// ```rust
/// use opengp::ui::widgets::StatusBadge;
/// use ratatui::style::Color;
///
/// let badge = StatusBadge::new("Active", Color::Green);
/// assert_eq!(badge.text, "Active");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StatusBadge {
    /// The text content of the badge.
    pub text: String,
    /// The color of the badge.
    pub color: Color,
}

impl StatusBadge {
    /// Creates a new `StatusBadge` with the given text and color.
    ///
    /// # Arguments
    ///
    /// * `text` - The badge text content
    /// * `color` - The badge color
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::StatusBadge;
    /// use ratatui::style::Color;
    ///
    /// let badge = StatusBadge::new("Active", Color::Green);
    /// assert_eq!(badge.text, "Active");
    /// ```
    pub fn new(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color,
        }
    }

    /// Creates a success badge (green).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::StatusBadge;
    ///
    /// let badge = StatusBadge::success("Completed");
    /// assert_eq!(badge.text, "Completed");
    /// ```
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::Green,
        }
    }

    /// Creates a warning badge (yellow).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::StatusBadge;
    ///
    /// let badge = StatusBadge::warning("Pending");
    /// assert_eq!(badge.text, "Pending");
    /// ```
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::Yellow,
        }
    }

    /// Creates an error badge (red).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::StatusBadge;
    ///
    /// let badge = StatusBadge::error("Failed");
    /// assert_eq!(badge.text, "Failed");
    /// ```
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::Red,
        }
    }

    /// Creates an info badge (blue).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::StatusBadge;
    ///
    /// let badge = StatusBadge::info("New");
    /// assert_eq!(badge.text, "New");
    /// ```
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::Blue,
        }
    }

    /// Returns `true` if the badge has non-empty text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::StatusBadge;
    /// use ratatui::style::Color;
    ///
    /// let badge = StatusBadge::new("Text", Color::Green);
    /// assert!(badge.has_text());
    ///
    /// let empty = StatusBadge::new("", Color::Green);
    /// assert!(!empty.has_text());
    /// ```
    pub fn has_text(&self) -> bool {
        !self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    // === Constructor Tests ===

    #[test]
    fn test_new() {
        let badge = StatusBadge::new("Active", Color::Green);
        assert_eq!(badge.text, "Active");
        assert_eq!(badge.color, Color::Green);
    }

    #[test]
    fn test_new_with_empty_text() {
        let badge = StatusBadge::new("", Color::Red);
        assert!(badge.text.is_empty());
    }

    // === Factory Method Tests ===

    #[test]
    fn test_success() {
        let badge = StatusBadge::success("Done");
        assert_eq!(badge.text, "Done");
        assert_eq!(badge.color, Color::Green);
    }

    #[test]
    fn test_warning() {
        let badge = StatusBadge::warning("Wait");
        assert_eq!(badge.text, "Wait");
        assert_eq!(badge.color, Color::Yellow);
    }

    #[test]
    fn test_error() {
        let badge = StatusBadge::error("Failed");
        assert_eq!(badge.text, "Failed");
        assert_eq!(badge.color, Color::Red);
    }

    #[test]
    fn test_info() {
        let badge = StatusBadge::info("New");
        assert_eq!(badge.text, "New");
        assert_eq!(badge.color, Color::Blue);
    }

    // === Method Tests ===

    #[test]
    fn test_has_text_true() {
        let badge = StatusBadge::new("Text", Color::Green);
        assert!(badge.has_text());
    }

    #[test]
    fn test_has_text_false() {
        let badge = StatusBadge::new("", Color::Green);
        assert!(!badge.has_text());
    }

    #[test]
    fn test_has_text_whitespace() {
        let badge = StatusBadge::new("   ", Color::Green);
        assert!(badge.has_text());
    }

    // === Clone and Equality Tests ===

    #[test]
    fn test_clone() {
        let original = StatusBadge::new("Test", Color::Blue);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_partial_eq_same() {
        let badge1 = StatusBadge::new("Active", Color::Green);
        let badge2 = StatusBadge::new("Active", Color::Green);
        assert_eq!(badge1, badge2);
    }

    #[test]
    fn test_partial_eq_different_text() {
        let badge1 = StatusBadge::new("Active", Color::Green);
        let badge2 = StatusBadge::new("Inactive", Color::Green);
        assert_ne!(badge1, badge2);
    }

    #[test]
    fn test_partial_eq_different_color() {
        let badge1 = StatusBadge::new("Active", Color::Green);
        let badge2 = StatusBadge::new("Active", Color::Red);
        assert_ne!(badge1, badge2);
    }

    // === Debug Tests ===

    #[test]
    fn test_debug_format() {
        let badge = StatusBadge::new("Test", Color::Blue);
        let debug_str = format!("{:?}", badge);
        assert!(debug_str.contains("StatusBadge"));
        assert!(debug_str.contains("Test"));
    }
}
