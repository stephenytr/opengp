use std::collections::HashMap;

use opengp_config::forms::ValidationRules;
use regex::Regex;

pub struct FormValidator {
    rules: HashMap<String, ValidationRules>,
    compiled_regex: HashMap<String, Regex>,
}

impl FormValidator {
    pub fn new(rules: &HashMap<String, ValidationRules>) -> Self {
        let mut compiled_regex = HashMap::new();

        for (field_id, field_rules) in rules {
            if let Some(pattern) = &field_rules.regex {
                if let Ok(regex) = Regex::new(pattern) {
                    compiled_regex.insert(field_id.clone(), regex);
                }
            }
        }

        Self {
            rules: rules.clone(),
            compiled_regex,
        }
    }

    pub fn validate(&self, field_id: &str, value: &str) -> Vec<String> {
        let Some(rules) = self.rules.get(field_id) else {
            return Vec::new();
        };

        let mut errors = Vec::new();

        if rules.required && value.trim().is_empty() {
            errors.push("This field is required".to_string());
            return errors;
        }

        if !rules.required && value.trim().is_empty() {
            return errors;
        }

        if let Some(max_length) = rules.max_length {
            if value.len() > max_length {
                errors.push(format!("Maximum {max_length} characters"));
            }
        }

        if let Some(min_length) = rules.min_length {
            if value.len() < min_length {
                errors.push(format!("Minimum {min_length} characters"));
            }
        }

        if rules.email && !value.contains('@') {
            errors.push("Invalid email format".to_string());
        }

        if rules.phone {
            let cleaned: String = value
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == ' ' || *c == '-' || *c == '(' || *c == ')')
                .collect();

            if cleaned.len() < 8 {
                errors.push("Invalid phone number".to_string());
            }
        }

        if let Some(range) = &rules.numeric_range {
            match value.trim().parse::<f64>() {
                Ok(parsed) => {
                    if parsed < range.min || parsed > range.max {
                        errors.push(format!(
                            "Value must be between {} and {}",
                            range.min, range.max
                        ));
                    }
                }
                Err(_) => errors.push("Invalid number".to_string()),
            }
        }

        if rules.regex.is_some() {
            match self.compiled_regex.get(field_id) {
                Some(regex) => {
                    if !regex.is_match(value) {
                        errors.push("Invalid format".to_string());
                    }
                }
                None => errors.push("Invalid format".to_string()),
            }
        }

        if rules.date_format.is_some() && !is_dd_mm_yyyy(value) {
            errors.push("Use dd/mm/yyyy format".to_string());
        }

        errors
    }
}

fn is_dd_mm_yyyy(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }

    bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2] == b'/'
        && bytes[3].is_ascii_digit()
        && bytes[4].is_ascii_digit()
        && bytes[5] == b'/'
        && bytes[6].is_ascii_digit()
        && bytes[7].is_ascii_digit()
        && bytes[8].is_ascii_digit()
        && bytes[9].is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opengp_config::forms::NumericRange;

    fn validator_with_rule(field_id: &str, rule: ValidationRules) -> FormValidator {
        let mut rules = HashMap::new();
        rules.insert(field_id.to_string(), rule);
        FormValidator::new(&rules)
    }

    #[test]
    fn required_rule_fails_on_empty() {
        let validator = validator_with_rule(
            "first_name",
            ValidationRules {
                required: true,
                ..ValidationRules::default()
            },
        );

        assert_eq!(
            validator.validate("first_name", "   "),
            vec!["This field is required".to_string()]
        );
    }

    #[test]
    fn optional_empty_value_skips_other_rules() {
        let validator = validator_with_rule(
            "optional",
            ValidationRules {
                min_length: Some(10),
                email: true,
                ..ValidationRules::default()
            },
        );

        assert!(validator.validate("optional", "").is_empty());
    }

    #[test]
    fn max_and_min_length_rules_apply() {
        let validator = validator_with_rule(
            "name",
            ValidationRules {
                max_length: Some(5),
                min_length: Some(3),
                ..ValidationRules::default()
            },
        );

        assert_eq!(
            validator.validate("name", "ab"),
            vec!["Minimum 3 characters".to_string()]
        );
        assert_eq!(
            validator.validate("name", "abcdef"),
            vec!["Maximum 5 characters".to_string()]
        );
    }

    #[test]
    fn email_rule_matches_existing_behavior() {
        let validator = validator_with_rule(
            "email",
            ValidationRules {
                email: true,
                ..ValidationRules::default()
            },
        );

        assert_eq!(
            validator.validate("email", "invalid-email"),
            vec!["Invalid email format".to_string()]
        );
    }

    #[test]
    fn phone_rule_matches_existing_behavior() {
        let validator = validator_with_rule(
            "phone",
            ValidationRules {
                phone: true,
                ..ValidationRules::default()
            },
        );

        assert_eq!(
            validator.validate("phone", "12345"),
            vec!["Invalid phone number".to_string()]
        );
        assert!(validator.validate("phone", "0412 345 678").is_empty());
    }

    #[test]
    fn numeric_range_rule_validates_parse_and_bounds() {
        let validator = validator_with_rule(
            "age",
            ValidationRules {
                numeric_range: Some(NumericRange {
                    min: 0.0,
                    max: 120.0,
                }),
                ..ValidationRules::default()
            },
        );

        assert_eq!(
            validator.validate("age", "abc"),
            vec!["Invalid number".to_string()]
        );
        assert_eq!(
            validator.validate("age", "121"),
            vec!["Value must be between 0 and 120".to_string()]
        );
        assert!(validator.validate("age", "24").is_empty());
    }

    #[test]
    fn regex_rule_uses_compiled_pattern() {
        let validator = validator_with_rule(
            "postcode",
            ValidationRules {
                regex: Some("^\\d{4}$".to_string()),
                ..ValidationRules::default()
            },
        );

        assert!(validator.compiled_regex.contains_key("postcode"));
        assert_eq!(
            validator.validate("postcode", "12ab"),
            vec!["Invalid format".to_string()]
        );
        assert!(validator.validate("postcode", "3000").is_empty());
    }

    #[test]
    fn date_format_rule_validates_dd_mm_yyyy() {
        let validator = validator_with_rule(
            "dob",
            ValidationRules {
                date_format: Some("dd/mm/yyyy".to_string()),
                ..ValidationRules::default()
            },
        );

        assert_eq!(
            validator.validate("dob", "01011990"),
            vec!["Use dd/mm/yyyy format".to_string()]
        );
        assert!(validator.validate("dob", "01/01/1990").is_empty());
    }
}
