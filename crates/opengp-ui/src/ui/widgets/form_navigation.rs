pub trait FormFieldMeta {
    fn label(&self) -> &'static str;
    fn is_required(&self) -> bool;
}

pub trait FormNavigation {
    type FormField: Copy + PartialEq + FormFieldMeta;

    fn fields(&self) -> Vec<Self::FormField>;

    fn validate(&mut self) -> bool;

    fn current_field(&self) -> Self::FormField;

    fn set_current_field(&mut self, field: Self::FormField);

    fn get_error(&self, field: Self::FormField) -> Option<&str>;

    fn set_error(&mut self, field: Self::FormField, error: Option<String>);

    fn field_label(&self, field: Self::FormField) -> &'static str {
        field.label()
    }

    fn field_is_required(&self, field: Self::FormField) -> bool {
        field.is_required()
    }

    fn next_field(&mut self) {
        let fields = self.fields();
        if fields.is_empty() {
            return;
        }

        if let Some(current_idx) = fields
            .iter()
            .position(|field| *field == self.current_field())
        {
            let next_idx = (current_idx + 1) % fields.len();
            self.set_current_field(fields[next_idx]);
        }
    }

    fn prev_field(&mut self) {
        let fields = self.fields();
        if fields.is_empty() {
            return;
        }

        if let Some(current_idx) = fields
            .iter()
            .position(|field| *field == self.current_field())
        {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.set_current_field(fields[prev_idx]);
        }
    }

    fn has_errors(&self) -> bool {
        self.fields().iter().any(|f| self.get_error(*f).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestField {
        A,
        B,
        C,
    }

    impl FormFieldMeta for TestField {
        fn label(&self) -> &'static str {
            match self {
                TestField::A => "Field A",
                TestField::B => "Field B",
                TestField::C => "Field C",
            }
        }

        fn is_required(&self) -> bool {
            match self {
                TestField::A => true,
                TestField::B => false,
                TestField::C => true,
            }
        }
    }

    struct TestForm {
        current: TestField,
        errors: HashMap<TestField, String>,
    }

    impl TestForm {
        fn new() -> Self {
            TestForm {
                current: TestField::A,
                errors: HashMap::new(),
            }
        }
    }

    impl FormNavigation for TestForm {
        type FormField = TestField;

        fn fields(&self) -> Vec<Self::FormField> {
            vec![TestField::A, TestField::B, TestField::C]
        }

        fn validate(&mut self) -> bool {
            true
        }

        fn current_field(&self) -> Self::FormField {
            self.current
        }

        fn set_current_field(&mut self, field: Self::FormField) {
            self.current = field;
        }

        fn get_error(&self, field: Self::FormField) -> Option<&str> {
            self.errors.get(&field).map(|s| s.as_str())
        }

        fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
            match error {
                Some(err) => {
                    self.errors.insert(field, err);
                }
                None => {
                    self.errors.remove(&field);
                }
            }
        }
    }

    #[test]
    fn test_next_field_wraps_around() {
        let mut form = TestForm::new();
        assert_eq!(form.current_field(), TestField::A);

        form.next_field();
        assert_eq!(form.current_field(), TestField::B);

        form.next_field();
        assert_eq!(form.current_field(), TestField::C);

        form.next_field();
        assert_eq!(form.current_field(), TestField::A);
    }

    #[test]
    fn test_prev_field_wraps_around() {
        let mut form = TestForm::new();
        form.set_current_field(TestField::A);

        form.prev_field();
        assert_eq!(form.current_field(), TestField::C);

        form.prev_field();
        assert_eq!(form.current_field(), TestField::B);

        form.prev_field();
        assert_eq!(form.current_field(), TestField::A);
    }

    #[test]
    fn test_has_errors_false_when_no_errors() {
        let form = TestForm::new();
        assert!(!form.has_errors());
    }

    #[test]
    fn test_has_errors_true_when_error_exists() {
        let mut form = TestForm::new();
        form.set_error(TestField::A, Some("Required field".to_string()));
        assert!(form.has_errors());
    }

    #[test]
    fn test_next_field_with_empty_fields_no_panic() {
        struct EmptyForm;

        impl FormNavigation for EmptyForm {
            type FormField = TestField;

            fn fields(&self) -> Vec<Self::FormField> {
                vec![]
            }

            fn validate(&mut self) -> bool {
                true
            }

            fn current_field(&self) -> Self::FormField {
                TestField::A
            }

            fn set_current_field(&mut self, _field: Self::FormField) {}

            fn get_error(&self, _field: Self::FormField) -> Option<&str> {
                None
            }

            fn set_error(&mut self, _field: Self::FormField, _error: Option<String>) {}
        }

        let mut form = EmptyForm;
        form.next_field();
        assert_eq!(form.current_field(), TestField::A);
    }

    #[test]
    fn test_prev_field_with_single_field_stays_same() {
        struct SingleFieldForm;

        impl FormNavigation for SingleFieldForm {
            type FormField = TestField;

            fn fields(&self) -> Vec<Self::FormField> {
                vec![TestField::A]
            }

            fn validate(&mut self) -> bool {
                true
            }

            fn current_field(&self) -> Self::FormField {
                TestField::A
            }

            fn set_current_field(&mut self, _field: Self::FormField) {}

            fn get_error(&self, _field: Self::FormField) -> Option<&str> {
                None
            }

            fn set_error(&mut self, _field: Self::FormField, _error: Option<String>) {}
        }

        let mut form = SingleFieldForm;
        form.prev_field();
        assert_eq!(form.current_field(), TestField::A);
    }

    #[test]
    fn test_field_label_delegation() {
        let form = TestForm::new();
        assert_eq!(form.field_label(TestField::A), "Field A");
        assert_eq!(form.field_label(TestField::B), "Field B");
        assert_eq!(form.field_label(TestField::C), "Field C");
    }

    #[test]
    fn test_field_is_required_delegation() {
        let form = TestForm::new();
        assert!(form.field_is_required(TestField::A));
        assert!(!form.field_is_required(TestField::B));
        assert!(form.field_is_required(TestField::C));
    }
}
