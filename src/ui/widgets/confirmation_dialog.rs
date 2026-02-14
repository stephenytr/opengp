//! Simple confirmation dialog widget.
//!
//! This module provides a `ConfirmationDialog` struct that holds the state
//! for a confirmation dialog, including the message and button labels.

/// A simple confirmation dialog state.
///
/// This struct holds the text content and labels for a confirmation dialog.
/// It is a data structure only - rendering is handled by the UI layer.
///
/// # Fields
///
/// * `message` - The main dialog message to display
/// * `confirm_label` - Label text for the confirm/accept button
/// * `cancel_label` - Label text for the cancel/reject button
///
/// # Examples
///
/// ```rust
/// use opengp::ui::widgets::ConfirmationDialog;
///
/// let dialog = ConfirmationDialog::new(
///     "Are you sure you want to delete this patient?".to_string(),
/// );
///
/// assert_eq!(dialog.confirm_label, "Confirm");
/// assert_eq!(dialog.cancel_label, "Cancel");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmationDialog {
    /// The message displayed in the dialog body.
    pub message: String,
    /// Label for the confirm/positive action button.
    pub confirm_label: String,
    /// Label for the cancel/negative action button.
    pub cancel_label: String,
}

impl ConfirmationDialog {
    /// Creates a new `ConfirmationDialog` with default button labels.
    ///
    /// # Arguments
    ///
    /// * `message` - The dialog message to display
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ConfirmationDialog;
    ///
    /// let dialog = ConfirmationDialog::new("Continue?".to_string());
    /// assert_eq!(dialog.message, "Continue?");
    /// assert_eq!(dialog.confirm_label, "Confirm");
    /// assert_eq!(dialog.cancel_label, "Cancel");
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
        }
    }

    /// Creates a new `ConfirmationDialog` with custom button labels.
    ///
    /// # Arguments
    ///
    /// * `message` - The dialog message to display
    /// * `confirm_label` - Custom label for the confirm button
    /// * `cancel_label` - Custom label for the cancel button
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ConfirmationDialog;
    ///
    /// let dialog = ConfirmationDialog::with_labels(
    ///     "Save changes?",
    ///     "Save",
    ///     "Discard",
    /// );
    ///
    /// assert_eq!(dialog.confirm_label, "Save");
    /// assert_eq!(dialog.cancel_label, "Discard");
    /// ```
    pub fn with_labels(
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            confirm_label: confirm_label.into(),
            cancel_label: cancel_label.into(),
        }
    }

    /// Returns `true` if the dialog has a non-empty message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ConfirmationDialog;
    ///
    /// let dialog = ConfirmationDialog::new("Test");
    /// assert!(dialog.has_message());
    ///
    /// let empty = ConfirmationDialog::new("");
    /// assert!(!empty.has_message());
    /// ```
    pub fn has_message(&self) -> bool {
        !self.message.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Constructor Tests ===

    #[test]
    fn test_new_with_message() {
        let dialog = ConfirmationDialog::new("Are you sure?");
        assert_eq!(dialog.message, "Are you sure?");
    }

    #[test]
    fn test_new_default_labels() {
        let dialog = ConfirmationDialog::new("Test message");
        assert_eq!(dialog.confirm_label, "Confirm");
        assert_eq!(dialog.cancel_label, "Cancel");
    }

    #[test]
    fn test_new_with_empty_message() {
        let dialog = ConfirmationDialog::new("");
        assert!(dialog.message.is_empty());
    }

    // === Custom Labels Tests ===

    #[test]
    fn test_with_labels() {
        let dialog = ConfirmationDialog::with_labels("Save?", "Yes", "No");
        assert_eq!(dialog.message, "Save?");
        assert_eq!(dialog.confirm_label, "Yes");
        assert_eq!(dialog.cancel_label, "No");
    }

    #[test]
    fn test_with_labels_empty_labels() {
        let dialog = ConfirmationDialog::with_labels("Message", "", "");
        assert_eq!(dialog.confirm_label, "");
        assert_eq!(dialog.cancel_label, "");
    }

    // === Method Tests ===

    #[test]
    fn test_has_message_true() {
        let dialog = ConfirmationDialog::new("Hello");
        assert!(dialog.has_message());
    }

    #[test]
    fn test_has_message_false() {
        let dialog = ConfirmationDialog::new("");
        assert!(!dialog.has_message());
    }

    #[test]
    fn test_has_message_whitespace_only() {
        // Whitespace is still considered having a message
        let dialog = ConfirmationDialog::new("   ");
        assert!(dialog.has_message());
    }

    // === Clone and Equality Tests ===

    #[test]
    fn test_clone() {
        let original = ConfirmationDialog::new("Original message");
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_partial_eq() {
        let dialog1 = ConfirmationDialog::new("Same");
        let dialog2 = ConfirmationDialog::new("Same");
        let dialog3 = ConfirmationDialog::new("Different");

        assert_eq!(dialog1, dialog2);
        assert_ne!(dialog1, dialog3);
    }

    // === Debug Tests ===

    #[test]
    fn test_debug_format() {
        let dialog = ConfirmationDialog::new("Test");
        let debug_str = format!("{:?}", dialog);

        assert!(debug_str.contains("ConfirmationDialog"));
        assert!(debug_str.contains("Test"));
    }
}
