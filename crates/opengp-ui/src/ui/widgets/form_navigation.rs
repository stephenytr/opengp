pub trait FormNavigation {
    type FormField: Copy + PartialEq;

    fn validate(&mut self) -> bool;

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

    fn current_field(&self) -> Self::FormField;

    fn fields(&self) -> &[Self::FormField];

    fn set_current_field(&mut self, field: Self::FormField);
}
