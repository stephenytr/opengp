use crate::domain::clinical::FamilyHistory;
use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub enum FamilyHistoryFormField {
    Relationship,
    Condition,
    AgeAtDiagnosis,
    Notes,
}

pub struct FamilyHistoryForm {
    pub relationship: String,
    pub condition: String,
    pub age_at_diagnosis: Option<u8>,
    pub notes: String,
    pub focused_field: FamilyHistoryFormField,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub enum FamilyHistoryFormAction {
    Save,
    Cancel,
    NextField,
    PrevField,
}

impl FamilyHistoryForm {
    pub fn new() -> Self {
        Self {
            relationship: String::new(),
            condition: String::new(),
            age_at_diagnosis: None,
            notes: String::new(),
            focused_field: FamilyHistoryFormField::Relationship,
            is_valid: false,
        }
    }

    pub fn validate(&mut self) {
        self.is_valid = !self.relationship.trim().is_empty() && !self.condition.trim().is_empty();
    }

    pub fn to_family_history(
        &self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> FamilyHistory {
        FamilyHistory {
            id: uuid::Uuid::new_v4(),
            patient_id,
            relative_relationship: self.relationship.clone(),
            condition: self.condition.clone(),
            age_at_diagnosis: self.age_at_diagnosis,
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}
