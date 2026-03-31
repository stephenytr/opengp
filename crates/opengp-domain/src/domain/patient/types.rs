use serde::{Deserialize, Serialize};
use std::fmt;

use super::error::ValidationError;

/// MedicareNumber newtype wrapping String.
///
/// Lenient deserialization accepts any string (for DB compatibility).
/// Strict validation enforces 10-digit format (for UI entry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MedicareNumber(String);

impl MedicareNumber {
    /// Create a MedicareNumber from any string (lenient, for database).
    pub fn new_lenient(s: String) -> Self {
        Self(s)
    }

    /// Create a MedicareNumber with strict validation (for user input).
    /// Accepts exactly 10 digits.
    pub fn new_strict(s: String) -> Result<Self, ValidationError> {
        let trimmed = s.trim();
        if !trimmed.chars().all(|c| c.is_ascii_digit()) || trimmed.len() != 10 {
            return Err(ValidationError::InvalidMedicareNumber);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Get the inner string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MedicareNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MedicareNumber {
    fn from(s: String) -> Self {
        Self::new_lenient(s)
    }
}

impl AsRef<str> for MedicareNumber {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for MedicareNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MedicareNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new_lenient(s))
    }
}

/// Ihi newtype wrapping String.
///
/// Lenient deserialization accepts any string (for DB compatibility).
/// Strict validation enforces 16-digit Australian IHI format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ihi(String);

impl Ihi {
    /// Create an Ihi from any string (lenient, for database).
    pub fn new_lenient(s: String) -> Self {
        Self(s)
    }

    /// Create an Ihi with strict validation (for user input).
    /// Accepts exactly 16 digits (Australian IHI format).
    pub fn new_strict(s: String) -> Result<Self, ValidationError> {
        let trimmed = s.trim();
        if !trimmed.chars().all(|c| c.is_ascii_digit()) || trimmed.len() != 16 {
            return Err(ValidationError::InvalidIhi);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Get the inner string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ihi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Ihi {
    fn from(s: String) -> Self {
        Self::new_lenient(s)
    }
}

impl AsRef<str> for Ihi {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for Ihi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Ihi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new_lenient(s))
    }
}

/// PhoneNumber newtype wrapping String.
///
/// Lenient deserialization accepts any string (for DB compatibility).
/// Strict validation enforces Australian mobile/landline format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Create a PhoneNumber from any string (lenient, for database).
    pub fn new_lenient(s: String) -> Self {
        Self(s)
    }

    /// Create a PhoneNumber with strict validation (for user input).
    /// Accepts Australian mobile (04XX XXX XXX) or landline formats (0X XXXX XXXX / 0XX XXX XXXX).
    pub fn new_strict(s: String) -> Result<Self, ValidationError> {
        let trimmed = s.trim();
        let digits_only: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

        if !digits_only.starts_with('0') {
            return Err(ValidationError::InvalidPhoneNumber);
        }

        match digits_only.len() {
            10 | 11 => Ok(Self(trimmed.to_string())),
            _ => Err(ValidationError::InvalidPhoneNumber),
        }
    }

    /// Get the inner string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PhoneNumber {
    fn from(s: String) -> Self {
        Self::new_lenient(s)
    }
}

impl AsRef<str> for PhoneNumber {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for PhoneNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PhoneNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new_lenient(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod medicare_number {
        use super::*;

        #[test]
        fn test_lenient_accepts_any_string() {
            let mn = MedicareNumber::new_lenient("anything".to_string());
            assert_eq!(mn.as_str(), "anything");
        }

        #[test]
        fn test_lenient_accepts_empty() {
            let mn = MedicareNumber::new_lenient("".to_string());
            assert_eq!(mn.as_str(), "");
        }

        #[test]
        fn test_strict_rejects_non_digits() {
            let result = MedicareNumber::new_strict("12345ABCDE".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_rejects_wrong_length() {
            let result = MedicareNumber::new_strict("123456789".to_string());
            assert!(result.is_err());

            let result = MedicareNumber::new_strict("12345678901".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_accepts_10_digits() {
            let result = MedicareNumber::new_strict("1234567890".to_string());
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "1234567890");
        }

        #[test]
        fn test_strict_trims_whitespace() {
            let result = MedicareNumber::new_strict("  1234567890  ".to_string());
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "1234567890");
        }

        #[test]
        fn test_display() {
            let mn = MedicareNumber::new_lenient("1234567890".to_string());
            assert_eq!(mn.to_string(), "1234567890");
        }

        #[test]
        fn test_from_string() {
            let mn = MedicareNumber::from("test".to_string());
            assert_eq!(mn.as_str(), "test");
        }

        #[test]
        fn test_as_ref() {
            let mn = MedicareNumber::new_lenient("abc".to_string());
            let s: &str = mn.as_ref();
            assert_eq!(s, "abc");
        }

        #[test]
        fn test_serialize() {
            let mn = MedicareNumber::new_lenient("1234567890".to_string());
            let json = serde_json::to_string(&mn).unwrap();
            assert_eq!(json, "\"1234567890\"");
        }

        #[test]
        fn test_deserialize_lenient() {
            let json = "\"anything\"";
            let mn: MedicareNumber = serde_json::from_str(json).unwrap();
            assert_eq!(mn.as_str(), "anything");
        }

        #[test]
        fn test_clone() {
            let mn1 = MedicareNumber::new_lenient("test".to_string());
            let mn2 = mn1.clone();
            assert_eq!(mn1, mn2);
        }
    }

    mod ihi {
        use super::*;

        #[test]
        fn test_lenient_accepts_any_string() {
            let ihi = Ihi::new_lenient("anything".to_string());
            assert_eq!(ihi.as_str(), "anything");
        }

        #[test]
        fn test_strict_rejects_non_digits() {
            let result = Ihi::new_strict("1234567890ABCDEF".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_rejects_wrong_length() {
            let result = Ihi::new_strict("123456789012345".to_string());
            assert!(result.is_err());

            let result = Ihi::new_strict("12345678901234567".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_accepts_16_digits() {
            let result = Ihi::new_strict("1234567890123456".to_string());
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "1234567890123456");
        }

        #[test]
        fn test_strict_trims_whitespace() {
            let result = Ihi::new_strict("  1234567890123456  ".to_string());
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "1234567890123456");
        }

        #[test]
        fn test_display() {
            let ihi = Ihi::new_lenient("1234567890123456".to_string());
            assert_eq!(ihi.to_string(), "1234567890123456");
        }

        #[test]
        fn test_from_string() {
            let ihi = Ihi::from("test".to_string());
            assert_eq!(ihi.as_str(), "test");
        }

        #[test]
        fn test_serialize() {
            let ihi = Ihi::new_lenient("1234567890123456".to_string());
            let json = serde_json::to_string(&ihi).unwrap();
            assert_eq!(json, "\"1234567890123456\"");
        }

        #[test]
        fn test_deserialize_lenient() {
            let json = "\"anything\"";
            let ihi: Ihi = serde_json::from_str(json).unwrap();
            assert_eq!(ihi.as_str(), "anything");
        }
    }

    mod phone_number {
        use super::*;

        #[test]
        fn test_lenient_accepts_any_string() {
            let pn = PhoneNumber::new_lenient("anything".to_string());
            assert_eq!(pn.as_str(), "anything");
        }

        #[test]
        fn test_strict_rejects_no_leading_zero() {
            let result = PhoneNumber::new_strict("1234567890".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_accepts_mobile_04() {
            let result = PhoneNumber::new_strict("0412345678".to_string());
            assert!(result.is_ok());
        }

        #[test]
        fn test_strict_accepts_landline_2digit_area() {
            let result = PhoneNumber::new_strict("0212345678".to_string());
            assert!(result.is_ok());
        }

        #[test]
        fn test_strict_accepts_landline_3digit_area() {
            let result = PhoneNumber::new_strict("02123456789".to_string());
            assert!(result.is_ok());
        }

        #[test]
        fn test_strict_accepts_formatted_with_spaces() {
            let result = PhoneNumber::new_strict("04 1234 5678".to_string());
            assert!(result.is_ok());
        }

        #[test]
        fn test_strict_accepts_formatted_with_dashes() {
            let result = PhoneNumber::new_strict("04-1234-5678".to_string());
            assert!(result.is_ok());
        }

        #[test]
        fn test_strict_rejects_too_short() {
            let result = PhoneNumber::new_strict("041234567".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_rejects_too_long() {
            let result = PhoneNumber::new_strict("041234567890".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_strict_rejects_letters() {
            let result = PhoneNumber::new_strict("04 ABCD 5678".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_display() {
            let pn = PhoneNumber::new_lenient("0412345678".to_string());
            assert_eq!(pn.to_string(), "0412345678");
        }

        #[test]
        fn test_from_string() {
            let pn = PhoneNumber::from("test".to_string());
            assert_eq!(pn.as_str(), "test");
        }

        #[test]
        fn test_serialize() {
            let pn = PhoneNumber::new_lenient("0412345678".to_string());
            let json = serde_json::to_string(&pn).unwrap();
            assert_eq!(json, "\"0412345678\"");
        }

        #[test]
        fn test_deserialize_lenient() {
            let json = "\"anything\"";
            let pn: PhoneNumber = serde_json::from_str(json).unwrap();
            assert_eq!(pn.as_str(), "anything");
        }
    }
}
