//! Consultation Form Component
//!
//! Form for creating and editing patient consultations (SOAP notes).

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::input::to_ratatui_key;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    FieldType, FormFieldMeta, FormNavigation, HeightMode, ScrollableFormState, TextareaState,
    TextareaWidget,
};
use opengp_domain::domain::clinical::Consultation;

const FIELD_REASON: &str = "reason";
const FIELD_CLINICAL_NOTES: &str = "clinical_notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum ConsultationFormField {
    #[strum(to_string = "Reason")]
    Reason,
    #[strum(to_string = "Clinical Notes")]
    ClinicalNotes,
}

impl ConsultationFormField {
    pub fn all() -> Vec<ConsultationFormField> {
        use strum::IntoEnumIterator;
        ConsultationFormField::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            ConsultationFormField::Reason => FIELD_REASON,
            ConsultationFormField::ClinicalNotes => FIELD_CLINICAL_NOTES,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_REASON => Some(ConsultationFormField::Reason),
            FIELD_CLINICAL_NOTES => Some(ConsultationFormField::ClinicalNotes),
            _ => None,
        }
    }

    pub fn is_required(&self) -> bool {
        true
    }

    pub fn is_textarea(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub enum ConsultationFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct ConsultationForm {
    textareas: HashMap<String, TextareaState>,
    focused_field: String,
    field_ids: Vec<String>,
    pub is_valid: bool,
    pub is_edit_mode: bool,
    pub consultation_id: Option<uuid::Uuid>,
    errors: HashMap<String, String>,
    theme: Theme,
    scroll: ScrollableFormState,
}

impl Clone for ConsultationForm {
    fn clone(&self) -> Self {
        Self {
            textareas: self.textareas.clone(),
            focused_field: self.focused_field.clone(),
            field_ids: self.field_ids.clone(),
            is_valid: self.is_valid,
            is_edit_mode: self.is_edit_mode,
            consultation_id: self.consultation_id,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
        }
    }
}

fn reason_textarea() -> TextareaState {
    TextareaState::new("Reason").with_height_mode(HeightMode::AutoGrow { min: 2, max: 4 })
}

fn clinical_textarea() -> TextareaState {
    TextareaState::new("Clinical Notes").with_height_mode(HeightMode::AutoGrow { min: 10, max: 20 })
}

fn textarea_for_field(field: ConsultationFormField, value: Option<String>) -> TextareaState {
    let mut textarea = match field {
        ConsultationFormField::Reason => reason_textarea(),
        ConsultationFormField::ClinicalNotes => clinical_textarea(),
    };

    if let Some(value) = value {
        textarea = textarea.with_value(value);
    }

    textarea
}

impl ConsultationForm {
    pub fn new(theme: Theme) -> Self {
        let field_ids = ConsultationFormField::all()
            .into_iter()
            .map(|field| field.id().to_string())
            .collect::<Vec<_>>();

        let mut textareas = HashMap::new();
        for field in ConsultationFormField::all() {
            textareas.insert(field.id().to_string(), textarea_for_field(field, None));
        }

        Self {
            textareas,
            focused_field: FIELD_REASON.to_string(),
            field_ids,
            is_valid: true,
            is_edit_mode: false,
            consultation_id: None,
            errors: HashMap::new(),
            theme: theme.clone(),
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn from_consultation(theme: Theme, consultation: &Consultation) -> Self {
        let mut form = Self::new(theme);
        form.is_edit_mode = true;
        form.consultation_id = Some(consultation.id);
        form.set_value(
            ConsultationFormField::Reason,
            consultation.reason.clone().unwrap_or_default(),
        );
        form.set_value(
            ConsultationFormField::ClinicalNotes,
            consultation.clinical_notes.clone().unwrap_or_default(),
        );
        form.errors.clear();
        form
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        self.textareas.get_mut(&self.focused_field)
    }

    fn textarea_for(&self, field_id: &str) -> Option<&TextareaState> {
        self.textareas.get(field_id)
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        self.textareas
            .get(field_id)
            .map(|textarea| textarea.value())
            .unwrap_or_default()
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            let label = textarea.label.clone();
            let height_mode = textarea.height_mode.clone();
            let max_length = textarea.max_length;
            let focused = textarea.focused;

            let mut updated = TextareaState::new(label)
                .with_height_mode(height_mode)
                .with_value(value)
                .focused(focused);

            if let Some(limit) = max_length {
                updated = updated.max_length(limit);
            }

            *textarea = updated;
        }
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            textarea.set_error(error.clone());
        }

        match error {
            Some(msg) => {
                self.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.errors.remove(field_id);
            }
        }
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);

        let value = self.get_value_by_id(field_id);
        let error = match field_id {
            FIELD_REASON if value.trim().is_empty() => Some("Reason is required".to_string()),
            FIELD_CLINICAL_NOTES if value.trim().is_empty() => {
                Some("Clinical notes are required".to_string())
            }
            _ => None,
        };

        self.set_error_by_id(field_id, error);
    }

    fn get_field_height(&self, field_id: &str) -> u16 {
        self.textareas
            .get(field_id)
            .map(|textarea| textarea.height())
            .unwrap_or(1)
    }

    fn get_field_position(&self, field_id: &str) -> (u16, u16) {
        let mut y = 0;

        for id in &self.field_ids {
            if id == field_id {
                return (y, self.get_field_height(id));
            }
            y += self.get_field_height(id) + 1;
        }

        (0, 0)
    }

    pub fn focused_field(&self) -> ConsultationFormField {
        ConsultationFormField::from_id(&self.focused_field).unwrap_or(ConsultationFormField::Reason)
    }

    pub fn get_value(&self, field: ConsultationFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: ConsultationFormField, value: String) {
        self.set_value_by_id(field.id(), value);
        self.validate_field(&field);
    }

    fn validate_field(&mut self, field: &ConsultationFormField) {
        self.validate_field_by_id(field.id());
    }

    pub fn error(&self, field: ConsultationFormField) -> Option<&String> {
        self.errors.get(field.id())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConsultationFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Ctrl+S submits the form from any field
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(ConsultationFormAction::Submit);
        }

        if self.focused_field().is_textarea() {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                let consumed = textarea.handle_key(ratatui_key);
                if consumed {
                    let focused_field = self.focused_field.clone();
                    self.validate_field_by_id(&focused_field);
                    return Some(ConsultationFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Some(ConsultationFormAction::Cancel);
                }
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Enter => None,
            KeyCode::Esc => Some(ConsultationFormAction::Cancel),
            _ => None,
        }
    }

    pub fn to_consultation(
        &self,
        patient_id: uuid::Uuid,
        practitioner_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> Consultation {
        Consultation {
            id: self.consultation_id.unwrap_or_else(uuid::Uuid::new_v4),
            patient_id,
            practitioner_id,
            appointment_id: None,
            consultation_date: chrono::Utc::now(),
            reason: Some(self.get_value(ConsultationFormField::Reason)).filter(|s| !s.is_empty()),
            clinical_notes: Some(self.get_value(ConsultationFormField::ClinicalNotes))
                .filter(|s| !s.is_empty()),
            is_signed: false,
            signed_at: None,
            signed_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
            created_by,
            updated_by: None,
        }
    }
}

impl crate::ui::widgets::DynamicFormMeta for ConsultationForm {
    fn label(&self, field_id: &str) -> String {
        ConsultationFormField::from_id(field_id)
            .map(|field| field.label().to_string())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, field_id: &str) -> bool {
        ConsultationFormField::from_id(field_id)
            .map(|field| field.is_required())
            .unwrap_or(false)
    }

    fn field_type(&self, _field_id: &str) -> FieldType {
        FieldType::Text
    }
}

impl crate::ui::widgets::DynamicForm for ConsultationForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        &self.focused_field
    }

    fn set_current_field(&mut self, field_id: &str) {
        if self.field_ids.iter().any(|id| id == field_id) {
            self.focused_field = field_id.to_string();
        }
    }

    fn get_value(&self, field_id: &str) -> String {
        self.get_value_by_id(field_id)
    }

    fn set_value(&mut self, field_id: &str, value: String) {
        self.set_value_by_id(field_id, value);
    }

    fn validate(&mut self) -> bool {
        for field_id in self.field_ids.clone() {
            self.set_error_by_id(&field_id, None);
        }
        self.errors.clear();
        self.is_valid = true;
        self.is_valid
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl FormFieldMeta for ConsultationFormField {
    fn label(&self) -> &'static str {
        ConsultationFormField::label(self)
    }

    fn is_required(&self) -> bool {
        ConsultationFormField::is_required(self)
    }
}

impl FormNavigation for ConsultationForm {
    type FormField = ConsultationFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        self.set_error_by_id(field.id(), error);
    }

    fn validate(&mut self) -> bool {
        <Self as crate::ui::widgets::DynamicForm>::validate(self)
    }

    fn current_field(&self) -> Self::FormField {
        ConsultationFormField::from_id(&self.focused_field).unwrap_or(ConsultationFormField::Reason)
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.field_ids
            .iter()
            .filter_map(|field_id| ConsultationFormField::from_id(field_id))
            .collect()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field.id().to_string();
    }
}

impl Widget for ConsultationForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let title = if self.is_edit_mode {
            " Edit Consultation "
        } else {
            " New Consultation "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let fields = self.field_ids.clone();

        let mut total_height: u16 = 0;
        for field_id in &fields {
            total_height += self.get_field_height(field_id) + 1;
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        let (focused_y, focused_height) = self.get_field_position(&self.focused_field);
        self.scroll
            .scroll_to_field(focused_y, focused_height, inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        for field_id in fields {
            let field_height = self.get_field_height(&field_id) as i32;

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height + 1;
                continue;
            }

            let is_focused = field_id == self.focused_field;

            if let Some(textarea) = self.textarea_for(&field_id) {
                let textarea_height = textarea.height();
                if y >= inner.y as i32 && y < max_y {
                    let field_area =
                        Rect::new(inner.x + 1, y as u16, inner.width - 2, textarea_height);
                    TextareaWidget::new(textarea, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                y += textarea_height as i32 + 1;
            }
        }

        self.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consultation_form_creation() {
        let theme = Theme::dark();
        let form = ConsultationForm::new(theme);

        assert_eq!(form.focused_field(), ConsultationFormField::Reason);
        assert!(form.is_valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_consultation_form_validation() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        let valid = form.validate();
        assert!(valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_consultation_form_field_navigation() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        assert_eq!(form.focused_field(), ConsultationFormField::Reason);
        form.next_field();
        assert_eq!(form.focused_field(), ConsultationFormField::ClinicalNotes);
        form.next_field();
        assert_eq!(form.focused_field(), ConsultationFormField::Reason);
        form.prev_field();
        assert_eq!(form.focused_field(), ConsultationFormField::ClinicalNotes);
    }

    #[test]
    fn test_consultation_form_all_fields_ordered() {
        let fields = ConsultationFormField::all();
        assert_eq!(fields[0], ConsultationFormField::Reason);
        assert_eq!(fields[1], ConsultationFormField::ClinicalNotes);
    }

    #[test]
    fn test_clinical_notes_is_textarea() {
        assert!(ConsultationFormField::Reason.is_textarea());
        assert!(ConsultationFormField::ClinicalNotes.is_textarea());
    }

    #[test]
    fn test_clinical_notes_field_get_set_value() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        form.set_value(
            ConsultationFormField::ClinicalNotes,
            "Patient reports headache".to_string(),
        );
        assert_eq!(
            form.get_value(ConsultationFormField::ClinicalNotes),
            "Patient reports headache"
        );
    }

    #[test]
    fn test_consultation_form_ctrl_s_submits() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = form.handle_key(key);
        assert!(matches!(action, Some(ConsultationFormAction::Submit)));
    }
}
