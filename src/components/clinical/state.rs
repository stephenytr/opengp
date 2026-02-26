use uuid::Uuid;

use crate::domain::patient::Patient;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ClinicalView {
    #[default]
    PatientSelector,
    PatientOverview,
    ConsultationList,
    ConsultationEditor(Uuid),
    AllergyList,
    AllergyEditor(Option<Uuid>),
    MedicalHistoryList,
    MedicalHistoryEditor(Option<Uuid>),
    FamilyHistoryList,
    FamilyHistoryEditor(Option<Uuid>),
    SocialHistoryEditor,
    VitalSignsEditor,
}

#[derive(Debug, Clone, Default)]
pub enum ModalType {
    #[default]
    None,
    Help,
    Confirmation(String),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct SOAPEditorState {
    pub subjective: String,
    pub objective: String,
    pub assessment: String,
    pub plan: String,
    pub has_changes: bool,
    pub active_section: SOAPSection,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SOAPSection {
    #[default]
    Subjective,
    Objective,
    Assessment,
    Plan,
}

impl SOAPSection {
    pub fn next(&mut self) {
        *self = match self {
            SOAPSection::Subjective => SOAPSection::Objective,
            SOAPSection::Objective => SOAPSection::Assessment,
            SOAPSection::Assessment => SOAPSection::Plan,
            SOAPSection::Plan => SOAPSection::Subjective,
        };
    }

    pub fn previous(&mut self) {
        *self = match self {
            SOAPSection::Subjective => SOAPSection::Plan,
            SOAPSection::Objective => SOAPSection::Subjective,
            SOAPSection::Assessment => SOAPSection::Objective,
            SOAPSection::Plan => SOAPSection::Assessment,
        };
    }
}

#[derive(Debug, Clone, Default)]
pub struct VitalSignsFormState {
    pub systolic_bp: String,
    pub diastolic_bp: String,
    pub heart_rate: String,
    pub respiratory_rate: String,
    pub temperature: String,
    pub oxygen_saturation: String,
    pub height_cm: String,
    pub weight_kg: String,
    pub notes: String,
    pub has_changes: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AllergyFormState {
    pub patient_id: Option<Uuid>,
    pub allergen: String,
    pub allergy_type: String,
    pub severity: String,
    pub reaction: String,
    pub onset_date: String,
    pub notes: String,
    pub is_active: bool,
    pub has_changes: bool,
    pub editing_id: Option<Uuid>,
    pub active_field: AllergyFormField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AllergyFormField {
    #[default]
    Allergen,
    AllergyType,
    Severity,
    Reaction,
    OnsetDate,
    Notes,
}

impl AllergyFormField {
    pub fn next_field(&self) -> Self {
        match self {
            Self::Allergen => Self::AllergyType,
            Self::AllergyType => Self::Severity,
            Self::Severity => Self::Reaction,
            Self::Reaction => Self::OnsetDate,
            Self::OnsetDate => Self::Notes,
            Self::Notes => Self::Allergen,
        }
    }

    pub fn prev_field(&self) -> Self {
        match self {
            Self::Allergen => Self::Notes,
            Self::AllergyType => Self::Allergen,
            Self::Severity => Self::AllergyType,
            Self::Reaction => Self::Severity,
            Self::OnsetDate => Self::Reaction,
            Self::Notes => Self::OnsetDate,
        }
    }
}

impl AllergyFormState {
    pub fn allergy_type(&self) -> crate::domain::clinical::AllergyType {
        match self.allergy_type.to_lowercase().as_str() {
            "drug" => crate::domain::clinical::AllergyType::Drug,
            "food" => crate::domain::clinical::AllergyType::Food,
            "environmental" => crate::domain::clinical::AllergyType::Environmental,
            _ => crate::domain::clinical::AllergyType::Other,
        }
    }

    pub fn severity(&self) -> crate::domain::clinical::Severity {
        match self.severity.to_lowercase().as_str() {
            "mild" => crate::domain::clinical::Severity::Mild,
            "moderate" => crate::domain::clinical::Severity::Moderate,
            "severe" => crate::domain::clinical::Severity::Severe,
            _ => crate::domain::clinical::Severity::Moderate,
        }
    }

    pub fn parse_onset_date(&self) -> Option<chrono::NaiveDate> {
        if self.onset_date.is_empty() {
            return None;
        }
        chrono::NaiveDate::parse_from_str(&self.onset_date, "%d/%m/%Y").ok()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MedicalHistoryFormState {
    pub condition: String,
    pub diagnosis_date: String,
    pub status: String,
    pub severity: String,
    pub notes: String,
    pub is_active: bool,
    pub has_changes: bool,
    pub editing_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct FamilyHistoryFormState {
    pub relative_relationship: String,
    pub condition: String,
    pub age_at_diagnosis: String,
    pub notes: String,
    pub has_changes: bool,
    pub editing_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct SocialHistoryFormState {
    pub smoking_status: String,
    pub cigarettes_per_day: String,
    pub smoking_quit_date: String,
    pub alcohol_status: String,
    pub standard_drinks_per_week: String,
    pub exercise_frequency: String,
    pub occupation: String,
    pub living_situation: String,
    pub support_network: String,
    pub notes: String,
    pub has_changes: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PatientSearchState {
    pub query: String,
    pub results: Vec<Patient>,
    pub is_open: bool,
    pub selected_index: usize,
}

impl PatientSearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.results.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.results.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn selected_patient(&self) -> Option<&Patient> {
        self.results.get(self.selected_index)
    }
}
