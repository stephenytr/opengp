use super::FieldType;

pub trait DynamicFormMeta {
    fn label(&self, field_id: &str) -> String;
    fn is_required(&self, field_id: &str) -> bool;
    fn field_type(&self, field_id: &str) -> FieldType;
}

pub trait DynamicForm: DynamicFormMeta {
    fn field_ids(&self) -> &[String];

    fn current_field(&self) -> &str;

    fn set_current_field(&mut self, field_id: &str);

    fn get_value(&self, field_id: &str) -> String;

    fn set_value(&mut self, field_id: &str, value: String);

    fn validate(&mut self) -> bool;

    fn get_error(&self, field_id: &str) -> Option<&str>;

    fn set_error(&mut self, field_id: &str, error: Option<String>);

    fn next_field(&mut self) {
        let fields = self.field_ids();
        if fields.is_empty() {
            return;
        }

        if let Some(current_idx) = fields
            .iter()
            .position(|field_id| field_id.as_str() == self.current_field())
        {
            let next_idx = (current_idx + 1) % fields.len();
            let next_field_id = fields[next_idx].clone();
            self.set_current_field(&next_field_id);
        }
    }

    fn prev_field(&mut self) {
        let fields = self.field_ids();
        if fields.is_empty() {
            return;
        }

        if let Some(current_idx) = fields
            .iter()
            .position(|field_id| field_id.as_str() == self.current_field())
        {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            let prev_field_id = fields[prev_idx].clone();
            self.set_current_field(&prev_field_id);
        }
    }
}
