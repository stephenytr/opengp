use std::sync::Arc;

use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::Frame;
use tracing::info;

use crate::components::{Action, Component};
use crate::domain::clinical::ClinicalService;
use crate::domain::patient::PatientService;
use crate::error::Result;

use super::state::{
    AllergyFormState, ClinicalView, FamilyHistoryFormState, MedicalHistoryFormState,
    ModalType, PatientSearchState, SocialHistoryFormState, VitalSignsFormState,
};
use super::patient_selector::render_patient_selector;
use super::patient_overview::render_patient_overview;
use super::consultation_list::render_consultation_list;
use super::consultation_form::render_consultation_form;
use super::allergy_list::render_allergy_list;
use super::allergy_form::render_allergy_form;
use super::vital_signs_form::render_vital_signs_form;
use super::medical_history_list::render_medical_history_list;
use super::medical_history_form::render_medical_history_form;
use super::family_history_list::render_family_history_list;
use super::family_history_form::render_family_history_form;
use super::social_history_form::render_social_history_form;
use super::renderers::render_modal;

pub struct ClinicalComponent {
    pub clinical_service: Arc<ClinicalService>,
    pub patient_service: Arc<PatientService>,
    pub current_patient: Option<crate::domain::patient::Patient>,
    pub current_view: ClinicalView,
    pub consultations: Vec<crate::domain::clinical::Consultation>,
    pub allergies: Vec<crate::domain::clinical::Allergy>,
    pub medical_history: Vec<crate::domain::clinical::MedicalHistory>,
    pub family_history: Vec<crate::domain::clinical::FamilyHistory>,
    pub social_history: Option<crate::domain::clinical::SocialHistory>,
    pub latest_vitals: Option<crate::domain::clinical::VitalSigns>,
    pub vital_signs_form: VitalSignsFormState,
    pub allergy_form: AllergyFormState,
    pub medical_history_form: MedicalHistoryFormState,
    pub family_history_form: FamilyHistoryFormState,
    pub social_history_form: SocialHistoryFormState,
    pub patient_search: PatientSearchState,
    pub modal_type: ModalType,
    pub error_message: Option<String>,
    pub showing_help: bool,
}

impl ClinicalComponent {
    pub fn new(clinical_service: Arc<ClinicalService>, patient_service: Arc<PatientService>) -> Self {
        Self {
            clinical_service,
            patient_service,
            current_patient: None,
            current_view: ClinicalView::PatientSelector,
            consultations: Vec::new(),
            allergies: Vec::new(),
            medical_history: Vec::new(),
            family_history: Vec::new(),
            social_history: None,
            latest_vitals: None,
            vital_signs_form: VitalSignsFormState::default(),
            allergy_form: AllergyFormState::default(),
            medical_history_form: MedicalHistoryFormState::default(),
            family_history_form: FamilyHistoryFormState::default(),
            social_history_form: SocialHistoryFormState::default(),
            patient_search: PatientSearchState::new(),
            modal_type: ModalType::None,
            error_message: None,
            showing_help: false,
        }
    }

    async fn load_patient_data(&mut self, patient_id: uuid::Uuid) {
        if let Ok(consultations) = self.clinical_service
            .list_patient_consultations(patient_id)
            .await
        {
            self.consultations = consultations;
        }

        if let Ok(allergies) = self.clinical_service
            .list_patient_allergies(patient_id, true)
            .await
        {
            self.allergies = allergies;
        }

        if let Ok(medical_history) = self.clinical_service
            .list_medical_history(patient_id, true)
            .await
        {
            self.medical_history = medical_history;
        }

        if let Ok(family_history) = self.clinical_service
            .list_family_history(patient_id)
            .await
        {
            self.family_history = family_history;
        }

        if let Ok(social_history) = self.clinical_service
            .get_social_history(patient_id)
            .await
        {
            self.social_history = social_history;
        }

        if let Ok(latest_vitals) = self.clinical_service
            .get_latest_vital_signs(patient_id)
            .await
        {
            self.latest_vitals = latest_vitals;
        }
    }

    fn clear_patient_data(&mut self) {
        self.consultations.clear();
        self.allergies.clear();
        self.medical_history.clear();
        self.family_history.clear();
        self.social_history = None;
        self.latest_vitals = None;
        self.vital_signs_form = VitalSignsFormState::default();
        self.allergy_form = AllergyFormState::default();
        self.medical_history_form = MedicalHistoryFormState::default();
        self.family_history_form = FamilyHistoryFormState::default();
        self.social_history_form = SocialHistoryFormState::default();
    }
}

#[async_trait]
impl Component for ClinicalComponent {
    async fn init(&mut self) -> Result<()> {
        info!("Initializing clinical component");
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.showing_help {
            self.showing_help = false;
            return Action::Render;
        }

        if !matches!(self.modal_type, ModalType::None) {
            self.modal_type = ModalType::None;
            return Action::Render;
        }

        match self.current_view {
            ClinicalView::PatientSelector => {
                self.handle_patient_selector_input(key)
            }
            ClinicalView::PatientOverview => {
                self.handle_patient_overview_input(key)
            }
            ClinicalView::ConsultationList => {
                self.handle_consultation_list_input(key)
            }
            ClinicalView::ConsultationEditor(_) => {
                self.handle_consultation_editor_input(key)
            }
            ClinicalView::AllergyList => {
                self.handle_allergy_list_input(key)
            }
            ClinicalView::AllergyEditor(_) => {
                self.handle_allergy_editor_input(key)
            }
            ClinicalView::MedicalHistoryList => {
                self.handle_medical_history_list_input(key)
            }
            ClinicalView::MedicalHistoryEditor(_) => {
                self.handle_medical_history_editor_input(key)
            }
            ClinicalView::FamilyHistoryList => {
                self.handle_family_history_list_input(key)
            }
            ClinicalView::FamilyHistoryEditor(_) => {
                self.handle_family_history_editor_input(key)
            }
            ClinicalView::SocialHistoryEditor => {
                self.handle_social_history_editor_input(key)
            }
            ClinicalView::VitalSignsEditor => {
                self.handle_vital_signs_editor_input(key)
            }
        }
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ClinicalPatientSelect(patient_id) => {
                if let Ok(Some(patient)) = self.patient_service.find_patient(patient_id).await {
                    self.current_patient = Some(patient);
                    self.load_patient_data(patient_id).await;
                    self.current_view = ClinicalView::PatientOverview;
                }
                Ok(Some(Action::Render))
            }
            Action::ClinicalPatientClear => {
                self.current_patient = None;
                self.clear_patient_data();
                self.current_view = ClinicalView::PatientSelector;
                Ok(Some(Action::Render))
            }
            Action::ClinicalShowOverview => {
                self.current_view = ClinicalView::PatientOverview;
                Ok(Some(Action::Render))
            }
            Action::ClinicalShowConsultations => {
                self.current_view = ClinicalView::ConsultationList;
                Ok(Some(Action::Render))
            }
            Action::ClinicalShowAllergies => {
                self.current_view = ClinicalView::AllergyList;
                Ok(Some(Action::Render))
            }
            Action::ClinicalShowMedicalHistory => {
                self.current_view = ClinicalView::MedicalHistoryList;
                Ok(Some(Action::Render))
            }
            Action::ClinicalShowFamilyHistory => {
                self.current_view = ClinicalView::FamilyHistoryList;
                Ok(Some(Action::Render))
            }
            Action::ClinicalShowSocialHistory => {
                self.current_view = ClinicalView::SocialHistoryEditor;
                Ok(Some(Action::Render))
            }
            _ => Ok(None),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        match &self.current_view {
            ClinicalView::PatientSelector => {
                render_patient_selector(self, frame, area);
            }
            ClinicalView::PatientOverview => {
                render_patient_overview(self, frame, area);
            }
            ClinicalView::ConsultationList => {
                render_consultation_list(self, frame, area);
            }
            ClinicalView::ConsultationEditor(id) => {
                render_consultation_form(self, frame, area, *id);
            }
            ClinicalView::AllergyList => {
                render_allergy_list(self, frame, area);
            }
            ClinicalView::AllergyEditor(id) => {
                render_allergy_form(self, frame, area, *id);
            }
            ClinicalView::MedicalHistoryList => {
                render_medical_history_list(self, frame, area);
            }
            ClinicalView::MedicalHistoryEditor(id) => {
                render_medical_history_form(self, frame, area, *id);
            }
            ClinicalView::FamilyHistoryList => {
                render_family_history_list(self, frame, area);
            }
            ClinicalView::FamilyHistoryEditor(id) => {
                render_family_history_form(self, frame, area, *id);
            }
            ClinicalView::SocialHistoryEditor => {
                render_social_history_form(self, frame, area);
            }
            ClinicalView::VitalSignsEditor => {
                render_vital_signs_form(self, frame, area);
            }
        }

        if self.showing_help {
            render_modal(frame, area, "Help", &self.get_help_text());
        }
    }
}

impl ClinicalComponent {
    fn handle_patient_selector_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('/') => {
                self.patient_search.is_open = true;
                Action::Render
            }
            KeyCode::Char(c) if self.patient_search.is_open => {
                self.patient_search.query.push(c);
                Action::Render
            }
            KeyCode::Backspace if self.patient_search.is_open => {
                self.patient_search.query.pop();
                Action::Render
            }
            KeyCode::Esc if self.patient_search.is_open => {
                self.patient_search.is_open = false;
                self.patient_search.query.clear();
                self.patient_search.results.clear();
                Action::Render
            }
            KeyCode::Down if self.patient_search.is_open => {
                self.patient_search.select_next();
                Action::Render
            }
            KeyCode::Up if self.patient_search.is_open => {
                self.patient_search.select_previous();
                Action::Render
            }
            KeyCode::Enter if self.patient_search.is_open => {
                if let Some(patient) = self.patient_search.selected_patient() {
                    return Action::ClinicalPatientSelect(patient.id);
                }
                Action::Render
            }
            KeyCode::Char('?') => {
                self.showing_help = true;
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_patient_overview_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::F(1) => {
                if let Some(ref patient) = self.current_patient {
                    return Action::ClinicalConsultationCreate(patient.id);
                }
                Action::None
            }
            KeyCode::F(2) => {
                if let Some(ref patient) = self.current_patient {
                    return Action::ClinicalVitalSignsRecord(patient.id);
                }
                Action::None
            }
            KeyCode::F(3) => Action::ClinicalShowConsultations,
            KeyCode::F(4) => Action::ClinicalShowAllergies,
            KeyCode::F(5) => Action::ClinicalShowMedicalHistory,
            KeyCode::Char('f') => Action::ClinicalShowFamilyHistory,
            KeyCode::Char('s') => Action::ClinicalShowSocialHistory,
            KeyCode::Esc => Action::ClinicalPatientClear,
            KeyCode::Char('?') => {
                self.showing_help = true;
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_consultation_list_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                Action::None
            }
            KeyCode::Esc => Action::ClinicalShowOverview,
            _ => Action::None,
        }
    }

    fn handle_consultation_editor_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Tab => {
                Action::Render
            }
            KeyCode::Esc => Action::ClinicalConsultationCancel,
            _ => Action::None,
        }
    }

    fn handle_allergy_list_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('a') => {
                if let Some(ref patient) = self.current_patient {
                    return Action::ClinicalAllergyAdd(patient.id);
                }
                Action::None
            }
            KeyCode::Esc => Action::ClinicalShowOverview,
            _ => Action::None,
        }
    }

    fn handle_allergy_editor_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::ClinicalAllergyCancel,
            _ => Action::None,
        }
    }

    fn handle_medical_history_list_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('a') => {
                if let Some(ref patient) = self.current_patient {
                    return Action::ClinicalMedicalHistoryAdd(patient.id);
                }
                Action::None
            }
            KeyCode::Esc => Action::ClinicalShowOverview,
            _ => Action::None,
        }
    }

    fn handle_medical_history_editor_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::ClinicalMedicalHistoryCancel,
            _ => Action::None,
        }
    }

    fn handle_family_history_list_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('a') => {
                if let Some(ref patient) = self.current_patient {
                    return Action::ClinicalFamilyHistoryAdd(patient.id);
                }
                Action::None
            }
            KeyCode::Esc => Action::ClinicalShowOverview,
            _ => Action::None,
        }
    }

    fn handle_family_history_editor_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::ClinicalFamilyHistoryCancel,
            _ => Action::None,
        }
    }

    fn handle_social_history_editor_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::ClinicalShowOverview,
            _ => Action::None,
        }
    }

    fn handle_vital_signs_editor_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::ClinicalVitalSignsCancel,
            _ => Action::None,
        }
    }

    fn get_help_text(&self) -> String {
        match self.current_view {
            ClinicalView::PatientSelector => {
                r#"Clinical Tab - Patient Selector

Keys:
  /       Open patient search
  ↑/↓     Navigate search results
  Enter   Select patient
  Esc     Close search
  ?       Show this help

Press any key to close..."#.to_string()
            }
            ClinicalView::PatientOverview => {
                r#"Clinical Tab - Patient Overview

Keys:
  F1      New Consultation
  F2      Record Vital Signs
  F3      View Consultations
  F4      Manage Allergies
  F5      Manage Medical History
  f       Family History
  s       Social History
  Esc     Clear patient / Go back
  ?       Show this help

Press any key to close..."#.to_string()
            }
            _ => {
                r#"Clinical Tab

Keys:
  Esc     Go back
  ?       Show this help

Press any key to close..."#.to_string()
            }
        }
    }
}
