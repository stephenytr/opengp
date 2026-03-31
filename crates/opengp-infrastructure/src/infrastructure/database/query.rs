//! PostgreSQL query placeholder generation

/// Return the first positional placeholder used in PostgreSQL queries
///
/// All SQL in this crate uses `$1`, `$2` style parameters to
/// match SQLx and Postgres.
pub fn placeholder() -> &'static str {
    "$1"
}

/// Build a comma separated list of positional placeholders
///
/// For example, `placeholders(3)` returns `"$1, $2, $3"`.
pub fn placeholders(count: usize) -> String {
    (1..=count)
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format a single positional placeholder for the given index
///
/// For example, `placeholder_at(2)` returns `"$2"`.
pub fn placeholder_at(index: usize) -> String {
    format!("${}", index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_single() {
        assert_eq!(placeholder(), "$1");
    }

    #[test]
    fn test_placeholders_multiple() {
        assert_eq!(placeholders(3), "$1, $2, $3");
    }

    #[test]
    fn test_placeholders_empty() {
        assert_eq!(placeholders(0), "");
    }

    #[test]
    fn test_placeholders_single() {
        assert_eq!(placeholders(1), "$1");
    }

    #[test]
    fn test_placeholder_at_indexed() {
        assert_eq!(placeholder_at(1), "$1");
        assert_eq!(placeholder_at(2), "$2");
        assert_eq!(placeholder_at(5), "$5");
    }
}
