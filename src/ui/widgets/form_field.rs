//! Form field widget with validation.
//!
//! This module provides a `FormField` struct for displaying form inputs
//! with validation state.

/// A form field with label, value, focus state, and validation error.
///
/// This struct holds the state for a form input field including
/// the label, current value, focus state, and any validation errors.
///
/// # Fields
///
/// * `label` - The field label/description
/// * `value` - The current field value
/// * `is_focused` - Whether the field currently has focus
/// * `error` - Optional validation error message
///
/// # Examples
///
/// ```rust
/// use opengp::ui::widgets::FormField;
///
/// let field = FormField::new("Name", "John");
/// assert_eq!(field.label, "Name");
/// assert_eq!(field.value, "John");
/// assert!(!field.is_focused);
/// assert!(field.error.is_none());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FormField {
    /// The field label displayed to the user.
    pub label: String,
    /// The current value of the field.
    pub value: String,
    /// Whether the field currently has focus.
    pub is_focused: bool,
    /// Validation error message if validation failed.
    pub error: Option<String>,
}

impl FormField {
    /// Creates a new `FormField` with the given label and value.
    ///
    /// # Arguments
    ///
    /// * `label` - The field label
    /// * `value` - The initial field value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let field = FormField::new("Email", "test@example.com");
    /// assert_eq!(field.label, "Email");
    /// assert_eq!(field.value, "test@example.com");
    /// ```
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            is_focused: false,
            error: None,
        }
    }

    /// Creates a new focused `FormField`.
    ///
    /// # Arguments
    ///
    /// * `label` - The field label
    /// * `value` - The initial field value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let field = FormField::new_focused("Search", "");
    /// assert!(field.is_focused);
    /// ```
    pub fn new_focused(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            is_focused: true,
            error: None,
        }
    }

    /// Sets the field value.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let mut field = FormField::new("Name", "John");
    /// field.set_value("Jane");
    /// assert_eq!(field.value, "Jane");
    /// ```
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        // Clear error when value changes
        self.error = None;
    }

    /// Sets the validation error.
    ///
    /// # Arguments
    ///
    /// * `error` - The error message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let mut field = FormField::new("Email", "invalid");
    /// field.set_error("Invalid email format");
    /// assert!(field.error.is_some());
    /// ```
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    /// Clears the validation error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let mut field = FormField::new("Email", "invalid");
    /// field.set_error("Error");
    /// assert!(field.error.is_some());
    ///
    /// field.clear_error();
    /// assert!(field.error.is_none());
    /// ```
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Sets the focus state.
    ///
    /// # Arguments
    ///
    /// * `focused` - Whether the field should have focus
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let mut field = FormField::new("Name", "John");
    /// assert!(!field.is_focused);
    ///
    /// field.set_focus(true);
    /// assert!(field.is_focused);
    /// ```
    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Returns `true` if the field has a validation error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let mut field = FormField::new("Email", "test");
    /// assert!(!field.has_error());
    ///
    /// field.set_error("Invalid");
    /// assert!(field.has_error());
    /// ```
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Returns `true` if the field value is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let field = FormField::new("Name", "");
    /// assert!(field.is_empty());
    ///
    /// let field = FormField::new("Name", "John");
    /// assert!(!field.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Returns the value length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::FormField;
    ///
    /// let field = FormField::new("Name", "John");
    /// assert_eq!(field.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Constructor Tests ===

    #[test]
    fn test_new() {
        let field = FormField::new("Name", "John");
        assert_eq!(field.label, "Name");
        assert_eq!(field.value, "John");
        assert!(!field.is_focused);
        assert!(field.error.is_none());
    }

    #[test]
    fn test_new_focused() {
        let field = FormField::new_focused("Search", "query");
        assert_eq!(field.label, "Search");
        assert_eq!(field.value, "query");
        assert!(field.is_focused);
    }

    #[test]
    fn test_new_with_empty_values() {
        let field = FormField::new("", "");
        assert!(field.label.is_empty());
        assert!(field.value.is_empty());
    }

    // === Value Method Tests ===

    #[test]
    fn test_set_value() {
        let mut field = FormField::new("Name", "John");
        field.set_value("Jane");
        assert_eq!(field.value, "Jane");
    }

    #[test]
    fn test_set_value_clears_error() {
        let mut field = FormField::new("Email", "test");
        field.set_error("Invalid");
        assert!(field.has_error());

        field.set_value("valid@email.com");
        assert!(!field.has_error());
    }

    // === Error Method Tests ===

    #[test]
    fn test_set_error() {
        let mut field = FormField::new("Email", "test");
        field.set_error("Invalid email");
        assert_eq!(field.error, Some("Invalid email".to_string()));
    }

    #[test]
    fn test_clear_error() {
        let mut field = FormField::new("Email", "test");
        field.set_error("Error");
        field.clear_error();
        assert!(field.error.is_none());
    }

    #[test]
    fn test_has_error_true() {
        let mut field = FormField::new("Email", "test");
        field.set_error("Error");
        assert!(field.has_error());
    }

    #[test]
    fn test_has_error_false() {
        let field = FormField::new("Email", "test");
        assert!(!field.has_error());
    }

    // === Focus Method Tests ===

    #[test]
    fn test_set_focus() {
        let mut field = FormField::new("Name", "John");
        assert!(!field.is_focused);

        field.set_focus(true);
        assert!(field.is_focused);

        field.set_focus(false);
        assert!(!field.is_focused);
    }

    // === Value State Tests ===

    #[test]
    fn test_is_empty_true() {
        let field = FormField::new("Name", "");
        assert!(field.is_empty());
    }

    #[test]
    fn test_is_empty_false() {
        let field = FormField::new("Name", "John");
        assert!(!field.is_empty());
    }

    #[test]
    fn test_len() {
        let field = FormField::new("Name", "John");
        assert_eq!(field.len(), 4);
    }

    #[test]
    fn test_len_empty() {
        let field = FormField::new("Name", "");
        assert_eq!(field.len(), 0);
    }

    // === Clone Tests ===

    #[test]
    fn test_clone() {
        let original = FormField::new("Label", "Value");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_clone_preserves_focus() {
        let field = FormField::new_focused("Name", "John");
        let cloned = field.clone();
        assert_eq!(field.is_focused, cloned.is_focused);
    }

    #[test]
    fn test_clone_preserves_error() {
        let mut field = FormField::new("Email", "test");
        field.set_error("Error");
        let cloned = field.clone();
        assert_eq!(field.error, cloned.error);
    }

    // === Equality Tests ===

    #[test]
    fn test_partial_eq_same() {
        let field1 = FormField::new("Name", "John");
        let field2 = FormField::new("Name", "John");
        assert_eq!(field1, field2);
    }

    #[test]
    fn test_partial_eq_different_value() {
        let field1 = FormField::new("Name", "John");
        let field2 = FormField::new("Name", "Jane");
        assert_ne!(field1, field2);
    }

    #[test]
    fn test_partial_eq_different_focus() {
        let field1 = FormField::new("Name", "John");
        let field2 = FormField::new_focused("Name", "John");
        assert_ne!(field1, field2);
    }

    #[test]
    fn test_partial_eq_different_error() {
        let mut field1 = FormField::new("Name", "John");
        let field2 = FormField::new("Name", "John");
        field1.set_error("Error");
        assert_ne!(field1, field2);
    }

    // === Debug Tests ===

    #[test]
    fn test_debug_format() {
        let field = FormField::new("Name", "John");
        let debug_str = format!("{:?}", field);
        assert!(debug_str.contains("FormField"));
        assert!(debug_str.contains("Name"));
        assert!(debug_str.contains("John"));
    }
}
