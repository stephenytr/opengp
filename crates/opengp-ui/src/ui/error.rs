//! UI Error Handling
//!
//! Consistent error types and trait for the UI layer.

use std::fmt;

/// Common UI error variants for consistent error handling across the UI layer.
#[derive(Debug, Clone)]
pub enum UiError {
    /// Form validation failures
    ValidationError(String),
    /// Entity not found
    NotFound(String),
    /// Unauthorized access
    PermissionDenied(String),
    /// File or database I/O errors
    IoError(String),
    /// Rendering failures
    RenderError(String),
    /// Catch-all for unknown errors
    Unknown(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            UiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            UiError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            UiError::IoError(msg) => write!(f, "I/O error: {}", msg),
            UiError::RenderError(msg) => write!(f, "Render error: {}", msg),
            UiError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for UiError {}

/// Trait for consistent error handling in UI components.
///
/// Provides a standardized interface for managing component-level errors.
/// Default implementations are provided for backward compatibility with
/// existing components that don't need error tracking.
pub trait UiComponent {
    /// Get the current error, if any.
    ///
    /// Returns a reference to the error if one is set, otherwise None.
    fn error(&self) -> Option<&dyn std::error::Error> {
        None
    }

    /// Set an error state on the component.
    ///
    /// Default implementation does nothing - override to track errors.
    fn set_error(&mut self, _error: UiError) {}

    /// Clear any existing error state.
    ///
    /// Default implementation does nothing - override to clear errors.
    fn clear_error(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_error_display() {
        let err = UiError::ValidationError("Invalid email".to_string());
        assert_eq!(err.to_string(), "Validation error: Invalid email");

        let err = UiError::NotFound("Patient 123".to_string());
        assert_eq!(err.to_string(), "Not found: Patient 123");

        let err = UiError::PermissionDenied("Admin only".to_string());
        assert_eq!(err.to_string(), "Permission denied: Admin only");
    }

    #[test]
    fn test_ui_error_debug() {
        let err = UiError::IoError("Database unavailable".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("IoError"));
        assert!(debug_str.contains("Database unavailable"));
    }

    #[test]
    fn test_ui_error_trait_default_implementations() {
        /// Test struct with default trait implementations
        struct DefaultComponent;

        impl UiComponent for DefaultComponent {}

        let mut component = DefaultComponent;
        assert!(component.error().is_none());
        component.clear_error(); // Should do nothing, not panic
        component.set_error(UiError::Unknown("test".to_string())); // Should do nothing
    }
}
