use uuid::Uuid;

use crate::domain::patient::Patient;
use crate::ui::components::{InputWrapper, SelectWrapper};

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

#[derive(Debug, Clone)]
pub struct VitalSignsFormState {
    pub systolic_bp: InputWrapper,
    pub diastolic_bp: InputWrapper,
    pub heart_rate: InputWrapper,
    pub respiratory_rate: InputWrapper,
    pub temperature: InputWrapper,
    pub oxygen_saturation: InputWrapper,
    pub height_cm: InputWrapper,
    pub weight_kg: InputWrapper,
    pub notes: InputWrapper,
    pub has_changes: bool,
}

impl Default for VitalSignsFormState {
    fn default() -> Self {
        Self {
            systolic_bp: InputWrapper::new(),
            diastolic_bp: InputWrapper::new(),
            heart_rate: InputWrapper::new(),
            respiratory_rate: InputWrapper::new(),
            temperature: InputWrapper::new(),
            oxygen_saturation: InputWrapper::new(),
            height_cm: InputWrapper::new(),
            weight_kg: InputWrapper::new(),
            notes: InputWrapper::new(),
            has_changes: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllergyFormState {
    pub allergen: InputWrapper,
    pub allergy_type: InputWrapper,
    pub severity: SelectWrapper,
    pub reaction: InputWrapper,
    pub onset_date: InputWrapper,
    pub notes: InputWrapper,
    pub is_active: bool,
    pub has_changes: bool,
    pub editing_id: Option<Uuid>,
}

impl Default for AllergyFormState {
    fn default() -> Self {
        Self {
            allergen: InputWrapper::new(),
            allergy_type: InputWrapper::new(),
            severity: SelectWrapper::new().items(vec![
                "Mild".to_string(),
                "Moderate".to_string(),
                "Severe".to_string(),
                "Life-threatening".to_string(),
            ]),
            reaction: InputWrapper::new(),
            onset_date: InputWrapper::new(),
            notes: InputWrapper::new(),
            is_active: true,
            has_changes: false,
            editing_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MedicalHistoryFormState {
    pub condition: InputWrapper,
    pub diagnosis_date: InputWrapper,
    pub status: SelectWrapper,
    pub severity: SelectWrapper,
    pub notes: InputWrapper,
    pub is_active: bool,
    pub has_changes: bool,
    pub editing_id: Option<Uuid>,
}

impl Default for MedicalHistoryFormState {
    fn default() -> Self {
        Self {
            condition: InputWrapper::new(),
            diagnosis_date: InputWrapper::new(),
            status: SelectWrapper::new().items(vec![
                "Active".to_string(),
                "Resolved".to_string(),
                "Managed".to_string(),
                "Unknown".to_string(),
            ]),
            severity: SelectWrapper::new().items(vec![
                "Mild".to_string(),
                "Moderate".to_string(),
                "Severe".to_string(),
            ]),
            notes: InputWrapper::new(),
            is_active: true,
            has_changes: false,
            editing_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FamilyHistoryFormState {
    pub relative_relationship: SelectWrapper,
    pub condition: InputWrapper,
    pub age_at_diagnosis: InputWrapper,
    pub notes: InputWrapper,
    pub has_changes: bool,
    pub editing_id: Option<Uuid>,
}

impl Default for FamilyHistoryFormState {
    fn default() -> Self {
        Self {
            relative_relationship: SelectWrapper::new().items(vec![
                "Mother".to_string(),
                "Father".to_string(),
                "Sibling".to_string(),
                "Grandparent".to_string(),
                "Aunt/Uncle".to_string(),
                "Other".to_string(),
            ]),
            condition: InputWrapper::new(),
            age_at_diagnosis: InputWrapper::new(),
            notes: InputWrapper::new(),
            has_changes: false,
            editing_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SocialHistoryFormState {
    pub smoking_status: SelectWrapper,
    pub cigarettes_per_day: InputWrapper,
    pub smoking_quit_date: InputWrapper,
    pub alcohol_status: SelectWrapper,
    pub standard_drinks_per_week: InputWrapper,
    pub exercise_frequency: InputWrapper,
    pub occupation: InputWrapper,
    pub living_situation: InputWrapper,
    pub support_network: InputWrapper,
    pub notes: InputWrapper,
    pub has_changes: bool,
}

impl Default for SocialHistoryFormState {
    fn default() -> Self {
        Self {
            smoking_status: SelectWrapper::new().items(vec![
                "Non-smoker".to_string(),
                "Current smoker".to_string(),
                "Former smoker".to_string(),
                "Unknown".to_string(),
            ]),
            cigarettes_per_day: InputWrapper::new(),
            smoking_quit_date: InputWrapper::new(),
            alcohol_status: SelectWrapper::new().items(vec![
                "Non-drinker".to_string(),
                "Occasional".to_string(),
                "Regular".to_string(),
                "Heavy".to_string(),
                "Unknown".to_string(),
            ]),
            standard_drinks_per_week: InputWrapper::new(),
            exercise_frequency: InputWrapper::new(),
            occupation: InputWrapper::new(),
            living_situation: InputWrapper::new(),
            support_network: InputWrapper::new(),
            notes: InputWrapper::new(),
            has_changes: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatientSearchState {
    pub query: InputWrapper,
    pub results: Vec<Patient>,
    pub is_open: bool,
    pub selected_index: usize,
}

impl Default for PatientSearchState {
    fn default() -> Self {
        Self {
            query: InputWrapper::new(),
            results: Vec::new(),
            is_open: false,
            selected_index: 0,
        }
    }
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
