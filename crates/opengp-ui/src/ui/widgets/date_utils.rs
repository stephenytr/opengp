//! Date utility functions for Australian date format (dd/mm/yyyy)
//!
//! Provides functions to format and parse dates in the Australian standard
//! format (dd/mm/yyyy). No backward compatibility for yyyy-mm-dd.

use chrono::{Datelike, NaiveDate};

/// Formats a NaiveDate to Australian format "dd/mm/yyyy"
///
/// # Arguments
/// * `date` - The NaiveDate to format
///
/// # Returns
/// A String in "dd/mm/yyyy" format
///
/// # Example
/// ```
/// use chrono::NaiveDate;
/// use opengp_ui::ui::widgets::format_date;
///
/// let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
/// assert_eq!(format_date(date), "15/03/2024");
/// ```
pub fn format_date(date: NaiveDate) -> String {
    format!("{:02}/{:02}/{}", date.day(), date.month(), date.year())
}

/// Parses a date string to NaiveDate
///
/// Supports formats:
/// - Australian format: "dd/mm/yyyy"
/// - Compact format: "ddmmyyyy"
///
/// # Arguments
/// * `s` - The date string to parse
///
/// # Returns
/// Some(NaiveDate) if parsing succeeds, None otherwise
///
/// # Example
/// ```
/// use chrono::NaiveDate;
/// use opengp_ui::ui::widgets::parse_date;
///
/// // Australian format
/// assert_eq!(parse_date("15/03/2024"), Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
///
/// // Compact format
/// assert_eq!(parse_date("15032024"), Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
///
/// // Invalid format
/// assert_eq!(parse_date("invalid"), None);
/// ```
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();

    // Try Australian format first: dd/mm/yyyy
    if let Some(_pos) = trimmed.find('/') {
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() == 3 {
            if let (Ok(day), Ok(month), Ok(year)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                return NaiveDate::from_ymd_opt(year, month, day);
            }
        }
    }

    if trimmed.len() == 8 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        if let (Ok(day), Ok(month), Ok(year)) = (
            trimmed[0..2].parse::<u32>(),
            trimmed[2..4].parse::<u32>(),
            trimmed[4..8].parse::<i32>(),
        ) {
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }

    None
}

pub fn format_user_input(input: &str) -> String {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    match digits.len() {
        0 => String::new(),
        1 => digits.clone(),
        2 => digits.to_string(),
        3 => format!("{}/{}", &digits[0..2], &digits[2..3]),
        4 => format!("{}/{}", &digits[0..2], &digits[2..4]),
        5 => format!("{}/{}/{}", &digits[0..2], &digits[2..4], &digits[4..5]),
        6 => format!("{}/{}/{}", &digits[0..2], &digits[2..4], &digits[4..6]),
        7 => format!("{}/{}/{}", &digits[0..2], &digits[2..4], &digits[4..7]),
        _ => format!("{}/{}/{}", &digits[0..2], &digits[2..4], &digits[4..8]),
    }
}

pub fn is_valid_date(s: &str) -> bool {
    parse_date(s).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_basic() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        assert_eq!(format_date(date), "05/03/2024");
    }

    #[test]
    fn test_format_date_double_digit_month() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        assert_eq!(format_date(date), "25/12/2024");
    }

    #[test]
    fn test_format_date_leading_zeros() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(format_date(date), "01/01/2024");
    }

    #[test]
    fn test_parse_date_au_format() {
        let result = parse_date("15/03/2024");
        assert_eq!(result, Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
    }

    #[test]
    fn test_parse_date_au_format_leading_zeros() {
        let result = parse_date("05/03/2024");
        assert_eq!(result, Some(NaiveDate::from_ymd_opt(2024, 3, 5).unwrap()));
    }

    #[test]
    fn test_parse_date_compact_format() {
        let result = parse_date("15032024");
        assert_eq!(result, Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
    }

    #[test]
    fn test_parse_date_compact_format_leading_zeros() {
        let result = parse_date("05032024");
        assert_eq!(result, Some(NaiveDate::from_ymd_opt(2024, 3, 5).unwrap()));
    }

    #[test]
    fn test_parse_date_invalid() {
        assert_eq!(parse_date("invalid"), None);
    }

    #[test]
    fn test_parse_date_invalid_format() {
        assert_eq!(parse_date("15-03-2024"), None);
    }

    #[test]
    fn test_parse_date_empty() {
        assert_eq!(parse_date(""), None);
    }

    #[test]
    fn test_parse_date_whitespace() {
        let result = parse_date("  15/03/2024  ");
        assert_eq!(result, Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
    }

    #[test]
    fn test_parse_date_invalid_day() {
        assert_eq!(parse_date("32/03/2024"), None);
    }

    #[test]
    fn test_parse_date_invalid_month() {
        assert_eq!(parse_date("15/13/2024"), None);
    }

    #[test]
    fn test_roundtrip() {
        let original = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let formatted = format_date(original);
        let parsed = parse_date(&formatted).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_format_user_input() {
        assert_eq!(format_user_input(""), "");
        assert_eq!(format_user_input("2"), "2");
        assert_eq!(format_user_input("26"), "26");
        assert_eq!(format_user_input("260"), "26/0");
        assert_eq!(format_user_input("2602"), "26/02");
        assert_eq!(format_user_input("26022"), "26/02/2");
        assert_eq!(format_user_input("260220"), "26/02/20");
        assert_eq!(format_user_input("2602202"), "26/02/202");
        assert_eq!(format_user_input("26022026"), "26/02/2026");
    }

    #[test]
    fn test_is_valid_date() {
        assert!(!is_valid_date(""));
        assert!(!is_valid_date("32/03/2026"));
        assert!(is_valid_date("26/02/2026"));
        assert!(is_valid_date("26022026"));
    }
}
