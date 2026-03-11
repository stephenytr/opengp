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
