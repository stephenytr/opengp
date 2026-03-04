//! Query compatibility layer for SQLite and PostgreSQL
//!
//! This module provides placeholder generation functions that adapt to the
//! configured database backend. SQLite uses `?` placeholders while PostgreSQL
//! uses `$1, $2, ...` style placeholders.
//!
//! # Feature Flags
//! - `sqlite` (default): Uses `?` placeholders
//! - `postgres`: Uses `$N` placeholders

/// Generate a single placeholder for the current database backend
///
/// # Returns
/// - SQLite: `"?"`
/// - PostgreSQL: `"$1"`
///
/// # Example
/// ```ignore
/// let placeholder = placeholder();
/// // SQLite: "?"
/// // PostgreSQL: "$1"
/// ```
pub fn placeholder() -> &'static str {
    #[cfg(feature = "sqlite")]
    return "?";
    #[cfg(feature = "postgres")]
    return "$1";
}

/// Generate placeholders for multiple parameters
///
/// # Arguments
/// * `count` - Number of placeholders to generate
///
/// # Returns
/// - SQLite: `"?, ?, ?"` (for count=3)
/// - PostgreSQL: `"$1, $2, $3"` (for count=3)
///
/// # Example
/// ```ignore
/// let placeholders = placeholders(3);
/// // SQLite: "?, ?, ?"
/// // PostgreSQL: "$1, $2, $3"
/// ```
pub fn placeholders(count: usize) -> String {
    #[cfg(feature = "sqlite")]
    return vec!["?"; count].join(", ");
    #[cfg(feature = "postgres")]
    return (1..=count)
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ");
}

/// Generate a placeholder for a specific parameter index
///
/// # Arguments
/// * `index` - Parameter index (1-based for PostgreSQL, ignored for SQLite)
///
/// # Returns
/// - SQLite: `"?"` (index is ignored)
/// - PostgreSQL: `"$N"` where N is the index
///
/// # Example
/// ```ignore
/// let p1 = placeholder_at(1);
/// let p2 = placeholder_at(2);
/// // SQLite: "?", "?"
/// // PostgreSQL: "$1", "$2"
/// ```
pub fn placeholder_at(index: usize) -> String {
    #[cfg(feature = "sqlite")]
    {
        let _ = index;
        return "?".to_string();
    }
    #[cfg(feature = "postgres")]
    return format!("${}", index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_single() {
        let p = placeholder();
        #[cfg(feature = "sqlite")]
        assert_eq!(p, "?");
        #[cfg(feature = "postgres")]
        assert_eq!(p, "$1");
    }

    #[test]
    fn test_placeholders_multiple() {
        let p = placeholders(3);
        #[cfg(feature = "sqlite")]
        assert_eq!(p, "?, ?, ?");
        #[cfg(feature = "postgres")]
        assert_eq!(p, "$1, $2, $3");
    }

    #[test]
    fn test_placeholders_empty() {
        let p = placeholders(0);
        assert_eq!(p, "");
    }

    #[test]
    fn test_placeholders_single() {
        let p = placeholders(1);
        #[cfg(feature = "sqlite")]
        assert_eq!(p, "?");
        #[cfg(feature = "postgres")]
        assert_eq!(p, "$1");
    }

    #[test]
    fn test_placeholder_at_indexed() {
        #[cfg(feature = "sqlite")]
        {
            assert_eq!(placeholder_at(1), "?");
            assert_eq!(placeholder_at(2), "?");
            assert_eq!(placeholder_at(5), "?");
        }
        #[cfg(feature = "postgres")]
        {
            assert_eq!(placeholder_at(1), "$1");
            assert_eq!(placeholder_at(2), "$2");
            assert_eq!(placeholder_at(5), "$5");
        }
    }
}
