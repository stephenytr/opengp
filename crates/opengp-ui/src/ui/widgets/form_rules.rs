use opengp_config::forms::{FormRule, FormRuleType};

#[derive(Debug, Clone, Default)]
pub struct FormRuleEngine {
    rules: Vec<FormRule>,
}

impl FormRuleEngine {
    pub fn new(rules: Vec<FormRule>) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, get_value: impl Fn(&str) -> String) -> Vec<String> {
        let mut errors = Vec::new();

        for rule in &self.rules {
            let passed = match rule.rule_type {
                FormRuleType::AnyNotEmpty => self.any_not_empty(&rule.fields, &get_value),
            };

            if !passed {
                errors.push(rule.message.clone());
            }
        }

        errors
    }

    pub fn any_not_empty(&self, fields: &[String], get_value: &impl Fn(&str) -> String) -> bool {
        fields
            .iter()
            .any(|field| !get_value(field).trim().is_empty())
    }

    pub fn all_filled(&self, fields: &[String], get_value: &impl Fn(&str) -> String) -> bool {
        !fields.is_empty()
            && fields
                .iter()
                .all(|field| !get_value(field).trim().is_empty())
    }

    pub fn at_least_n(
        &self,
        n: usize,
        fields: &[String],
        get_value: &impl Fn(&str) -> String,
    ) -> bool {
        if n == 0 {
            return true;
        }

        fields
            .iter()
            .filter(|field| !get_value(field).trim().is_empty())
            .count()
            >= n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str) -> String {
        name.to_string()
    }

    #[test]
    fn form_rules_any_not_empty_returns_message_when_all_empty() {
        let engine = FormRuleEngine::new(vec![FormRule {
            rule_type: FormRuleType::AnyNotEmpty,
            fields: vec![field("a"), field("b")],
            message: "At least one field is required".to_string(),
        }]);

        let errors = engine.evaluate(|_| String::new());

        assert_eq!(errors, vec!["At least one field is required"]);
    }

    #[test]
    fn form_rules_any_not_empty_passes_when_one_field_has_value() {
        let engine = FormRuleEngine::new(vec![FormRule {
            rule_type: FormRuleType::AnyNotEmpty,
            fields: vec![field("a"), field("b")],
            message: "At least one field is required".to_string(),
        }]);

        let errors = engine.evaluate(|name| {
            if name == "b" {
                "42".to_string()
            } else {
                String::new()
            }
        });

        assert!(errors.is_empty());
    }

    #[test]
    fn form_rules_all_filled_builtin_requires_all_non_empty_values() {
        let engine = FormRuleEngine::default();
        let fields = vec![field("a"), field("b")];

        assert!(engine.all_filled(&fields, &|_| "value".to_string()));
        assert!(!engine.all_filled(&fields, &|name| {
            if name == "a" {
                "value".to_string()
            } else {
                String::new()
            }
        }));
    }

    #[test]
    fn form_rules_at_least_n_builtin_counts_non_empty_values() {
        let engine = FormRuleEngine::default();
        let fields = vec![field("a"), field("b"), field("c")];

        assert!(engine.at_least_n(2, &fields, &|name| {
            if name == "a" || name == "c" {
                "value".to_string()
            } else {
                String::new()
            }
        }));

        assert!(!engine.at_least_n(3, &fields, &|name| {
            if name == "a" || name == "c" {
                "value".to_string()
            } else {
                String::new()
            }
        }));
    }

    #[test]
    fn form_rules_any_not_empty_treats_whitespace_as_empty() {
        let engine = FormRuleEngine::default();
        let fields = vec![field("a")];

        assert!(!engine.any_not_empty(&fields, &|_| "   ".to_string()));
    }

    #[test]
    fn form_rules_builtins_handle_empty_field_lists_gracefully() {
        let engine = FormRuleEngine::default();
        let fields = Vec::new();

        assert!(!engine.any_not_empty(&fields, &|_| "x".to_string()));
        assert!(!engine.all_filled(&fields, &|_| "x".to_string()));
        assert!(!engine.at_least_n(1, &fields, &|_| "x".to_string()));
        assert!(engine.at_least_n(0, &fields, &|_| String::new()));
    }
}
