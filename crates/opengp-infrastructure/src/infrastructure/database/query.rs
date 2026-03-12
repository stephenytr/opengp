//! PostgreSQL query placeholder generation

pub fn placeholder() -> &'static str {
    "$1"
}

pub fn placeholders(count: usize) -> String {
    (1..=count)
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ")
}

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
