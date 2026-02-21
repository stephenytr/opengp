use crate::domain::clinical::{Allergy, AllergyType, Severity};
use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub enum AllergyFormField {
    Allergen,
    AllergyType,
    Severity,
    Reaction,
    OnsetDate,
    Notes,
}

pub struct AllergyForm {
    pub allergen: String,
    pub allergy_type: Option<AllergyType>,
    pub severity: Option<Severity>,
    pub reaction: String,
    pub onset_date: Option<String>,
    pub notes: String,
    pub focused_field: AllergyFormField,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub enum AllergyFormAction {
    Save,
    Cancel,
    NextField,
    PrevField,
}

impl AllergyForm {
    pub fn new() -> Self {
        Self {
            allergen: String::new(),
            allergy_type: None,
            severity: None,
            reaction: String::new(),
            onset_date: None,
            notes: String::new(),
            focused_field: AllergyFormField::Allergen,
            is_valid: false,
        }
    }

    pub fn validate(&mut self) {
        self.is_valid = !self.allergen.trim().is_empty()
            && self.allergy_type.is_some()
            && self.severity.is_some();
    }

    pub fn to_allergy(&self, patient_id: uuid::Uuid, created_by: uuid::Uuid) -> Allergy {
        Allergy {
            id: uuid::Uuid::new_v4(),
            patient_id,
            allergen: self.allergen.clone(),
            allergy_type: self.allergy_type.unwrap_or(AllergyType::Other),
            severity: self.severity.unwrap_or(Severity::Moderate),
            reaction: Some(self.reaction.clone()).filter(|s| !s.is_empty()),
            onset_date: None,
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        }
    }
}
