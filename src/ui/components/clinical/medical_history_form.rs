use crate::domain::clinical::{ConditionStatus, MedicalHistory, Severity};
use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub enum MedicalHistoryFormField {
    Condition,
    DiagnosisDate,
    Status,
    Severity,
    Notes,
}

pub struct MedicalHistoryForm {
    pub condition: String,
    pub diagnosis_date: Option<String>,
    pub status: Option<ConditionStatus>,
    pub severity: Option<Severity>,
    pub notes: String,
    pub focused_field: MedicalHistoryFormField,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub enum MedicalHistoryFormAction {
    Save,
    Cancel,
    NextField,
    PrevField,
}

impl MedicalHistoryForm {
    pub fn new() -> Self {
        Self {
            condition: String::new(),
            diagnosis_date: None,
            status: None,
            severity: None,
            notes: String::new(),
            focused_field: MedicalHistoryFormField::Condition,
            is_valid: false,
        }
    }

    pub fn validate(&mut self) {
        self.is_valid = !self.condition.trim().is_empty() && self.status.is_some();
    }

    pub fn to_medical_history(
        &self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> MedicalHistory {
        MedicalHistory {
            id: uuid::Uuid::new_v4(),
            patient_id,
            condition: self.condition.clone(),
            diagnosis_date: None,
            status: self.status.unwrap_or(ConditionStatus::Active),
            severity: self.severity,
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        }
    }
}
