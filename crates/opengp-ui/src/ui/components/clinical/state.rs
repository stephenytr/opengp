use crate::ui::components::clinical::{
    AllergyForm, AllergyList, ConsultationForm, ConsultationList, FamilyHistoryForm,
    FamilyHistoryList, MedicalHistoryForm, MedicalHistoryList, SocialHistoryComponent,
    VitalSignsForm, VitalSignsList,
};
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;
use crate::ui::widgets::SearchableListState;
use opengp_domain::domain::clinical::{
    Allergy, Consultation, FamilyHistory, MedicalHistory, SocialHistory, VitalSigns,
};
use opengp_domain::domain::prescription::Prescription;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub enum ClinicalView {
    #[default]
    PatientSummary,
    Consultations,
    Allergies,
    MedicalHistory,
    VitalSigns,
    SocialHistory,
    FamilyHistory,
}

#[derive(Debug, Clone, Default)]
pub enum ClinicalFormView {
    #[default]
    None,
    AllergyForm,
    ConsultationForm,
    MedicalHistoryForm,
    VitalSignsForm,
    FamilyHistoryForm,
}

#[derive(Clone)]
pub struct ClinicalState {
    pub view: ClinicalView,
    pub form_view: ClinicalFormView,
    pub selected_patient_id: Option<Uuid>,
    pub loading: bool,
    pub error: Option<String>,
    pub consultations: Vec<Consultation>,
    pub allergies: Vec<Allergy>,
    pub medical_history: Vec<MedicalHistory>,
    pub vital_signs: Vec<VitalSigns>,
    pub social_history: Option<SocialHistory>,
    pub social_history_editing: bool,
    pub family_history: Vec<FamilyHistory>,
    pub page: usize,
    pub page_size: usize,
    pub theme: Theme,
    pub allergy_form: Option<AllergyForm>,
    pub consultation_form: Option<ConsultationForm>,
    pub medical_history_form: Option<MedicalHistoryForm>,
    pub vitals_form: Option<VitalSignsForm>,
    pub family_history_form: Option<FamilyHistoryForm>,
    pub social_history_component: Option<SocialHistoryComponent>,
    pub consultation_filter_start: Option<String>,
    pub consultation_filter_end: Option<String>,
    pub consultation_prescriptions: Vec<Prescription>,
    pub consultation_list: ConsultationList,
    pub allergy_list: AllergyList,
    pub medical_history_list: MedicalHistoryList,
    pub vitals_list: VitalSignsList,
    pub family_history_list: FamilyHistoryList,
    pub patient_search: Option<SearchableListState<PatientListItem>>,
}

impl ClinicalState {
    pub fn new(theme: Theme) -> Self {
        Self::with_theme(theme)
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self {
            view: ClinicalView::PatientSummary,
            form_view: ClinicalFormView::None,
            selected_patient_id: None,
            loading: false,
            error: None,
            consultations: Vec::new(),
            allergies: Vec::new(),
            medical_history: Vec::new(),
            vital_signs: Vec::new(),
            social_history: None,
            social_history_editing: false,
            family_history: Vec::new(),
            page: 0,
            page_size: 20,
            theme: theme.clone(),
            allergy_form: None,
            consultation_form: None,
            medical_history_form: None,
            vitals_form: None,
            family_history_form: None,
            social_history_component: None,
            consultation_filter_start: None,
            consultation_filter_end: None,
            consultation_prescriptions: Vec::new(),
            consultation_list: ConsultationList::new(theme.clone()),
            allergy_list: AllergyList::new(theme.clone()),
            medical_history_list: MedicalHistoryList::new(theme.clone()),
            vitals_list: VitalSignsList::new(theme.clone()),
            family_history_list: FamilyHistoryList::new(theme),
            patient_search: None,
        }
    }

    pub fn set_page_size(&mut self, height: u16) {
        self.page_size = height.saturating_sub(6) as usize;
        if self.page_size < 5 {
            self.page_size = 5;
        }
    }

    pub fn is_list_view(&self) -> bool {
        !matches!(self.view, ClinicalView::SocialHistory)
    }

    pub fn is_form_view(&self) -> bool {
        self.is_form_open()
    }

    pub fn is_form_open(&self) -> bool {
        !matches!(self.form_view, ClinicalFormView::None)
    }

    pub fn current_form_view(&self) -> &ClinicalFormView {
        &self.form_view
    }

    pub fn current_form_view_mut(&mut self) -> &mut ClinicalFormView {
        &mut self.form_view
    }

    pub fn open_consultation_form(&mut self) {
        self.consultation_form = Some(ConsultationForm::new(self.theme.clone()));
        self.form_view = ClinicalFormView::ConsultationForm;
    }

    pub fn close_consultation_form(&mut self) {
        self.consultation_form = None;
        self.form_view = ClinicalFormView::None;
    }

    pub fn open_allergy_form(&mut self) {
        self.allergy_form = Some(AllergyForm::new(self.theme.clone()));
        self.form_view = ClinicalFormView::AllergyForm;
    }

    pub fn open_medical_history_form(&mut self) {
        self.medical_history_form = Some(MedicalHistoryForm::new(self.theme.clone()));
        self.form_view = ClinicalFormView::MedicalHistoryForm;
    }

    pub fn open_vitals_form(&mut self) {
        self.vitals_form = Some(VitalSignsForm::new(self.theme.clone()));
        self.form_view = ClinicalFormView::VitalSignsForm;
    }

    pub fn open_family_history_form(&mut self) {
        self.family_history_form = Some(FamilyHistoryForm::new(self.theme.clone()));
        self.form_view = ClinicalFormView::FamilyHistoryForm;
    }

    pub fn open_social_history_editing(&mut self) {
        self.social_history_component = Some(SocialHistoryComponent::new(self.theme.clone()));
        self.social_history_editing = true;
    }

    pub fn close_social_history_editing(&mut self) {
        self.social_history_component = None;
        self.social_history_editing = false;
    }

    pub fn close_form(&mut self) {
        self.form_view = ClinicalFormView::None;
        self.allergy_form = None;
        self.consultation_form = None;
        self.medical_history_form = None;
        self.vitals_form = None;
        self.family_history_form = None;
        self.social_history_component = None;
    }

    pub fn show_consultations(&mut self) {
        self.view = ClinicalView::Consultations;
    }

    pub fn show_allergies(&mut self) {
        self.view = ClinicalView::Allergies;
    }

    pub fn show_medical_history(&mut self) {
        self.view = ClinicalView::MedicalHistory;
    }

    pub fn show_vital_signs(&mut self) {
        self.view = ClinicalView::VitalSigns;
    }

    pub fn show_social_history(&mut self) {
        self.view = ClinicalView::SocialHistory;
    }

    pub fn show_family_history(&mut self) {
        self.view = ClinicalView::FamilyHistory;
    }

    pub fn show_patient_summary(&mut self) {
        self.view = ClinicalView::PatientSummary;
    }

    pub fn cycle_view(&mut self) {
        self.view = match self.view {
            ClinicalView::PatientSummary => ClinicalView::Consultations,
            ClinicalView::Consultations => ClinicalView::Allergies,
            ClinicalView::Allergies => ClinicalView::MedicalHistory,
            ClinicalView::MedicalHistory => ClinicalView::VitalSigns,
            ClinicalView::VitalSigns => ClinicalView::SocialHistory,
            ClinicalView::SocialHistory => ClinicalView::FamilyHistory,
            ClinicalView::FamilyHistory => ClinicalView::PatientSummary,
        };
        self.reset_component_selection();
    }

    pub fn cycle_view_reverse(&mut self) {
        self.view = match self.view {
            ClinicalView::PatientSummary => ClinicalView::FamilyHistory,
            ClinicalView::Consultations => ClinicalView::PatientSummary,
            ClinicalView::Allergies => ClinicalView::Consultations,
            ClinicalView::MedicalHistory => ClinicalView::Allergies,
            ClinicalView::VitalSigns => ClinicalView::MedicalHistory,
            ClinicalView::SocialHistory => ClinicalView::VitalSigns,
            ClinicalView::FamilyHistory => ClinicalView::SocialHistory,
        };
        self.reset_component_selection();
    }

    fn reset_component_selection(&mut self) {
        self.consultation_list.move_first();
        self.allergy_list.move_first();
        self.medical_history_list.move_first();
        self.vitals_list.move_first();
        self.family_history_list.move_first();
    }

    pub fn set_patient(&mut self, patient_id: Uuid) {
        self.selected_patient_id = Some(patient_id);
    }

    pub fn clear_patient(&mut self) {
        self.selected_patient_id = None;
        self.consultations.clear();
        self.allergies.clear();
        self.medical_history.clear();
        self.vital_signs.clear();
        self.social_history = None;
        self.family_history.clear();
        self.social_history_component = None;
        self.close_form();

        self.consultation_list.consultations.clear();
        self.consultation_list.move_first();
        self.allergy_list.allergies.clear();
        self.allergy_list.move_first();
        self.medical_history_list.conditions.clear();
        self.medical_history_list.move_first();
        self.vitals_list.vitals.clear();
        self.vitals_list.move_first();
        self.family_history_list.entries.clear();
        self.family_history_list.move_first();
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn next_item(&mut self) {
        match self.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => self.consultation_list.next(),
            ClinicalView::Allergies => self.allergy_list.next(),
            ClinicalView::MedicalHistory => self.medical_history_list.next(),
            ClinicalView::VitalSigns => self.vitals_list.next(),
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => self.family_history_list.next(),
        }
    }

    pub fn prev_item(&mut self) {
        match self.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => self.consultation_list.prev(),
            ClinicalView::Allergies => self.allergy_list.prev(),
            ClinicalView::MedicalHistory => self.medical_history_list.prev(),
            ClinicalView::VitalSigns => self.vitals_list.prev(),
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => self.family_history_list.prev(),
        }
    }

    pub fn total_pages(&self, total_items: usize) -> usize {
        if total_items == 0 {
            return 1;
        }
        total_items.div_ceil(self.page_size)
    }

    pub fn page_offset(&self) -> usize {
        self.page * self.page_size
    }

    pub fn has_patient(&self) -> bool {
        self.selected_patient_id.is_some()
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        match self.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => self.consultation_list.adjust_scroll(visible_rows),
            ClinicalView::Allergies => self.allergy_list.adjust_scroll(visible_rows),
            ClinicalView::MedicalHistory => self.medical_history_list.adjust_scroll(visible_rows),
            ClinicalView::VitalSigns => self.vitals_list.adjust_scroll(visible_rows),
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => self.family_history_list.adjust_scroll(visible_rows),
        }
    }

    pub fn selected_index(&self) -> usize {
        match self.view {
            ClinicalView::PatientSummary => 0,
            ClinicalView::Consultations => self.consultation_list.selected_index,
            ClinicalView::Allergies => self.allergy_list.selected_index,
            ClinicalView::MedicalHistory => self.medical_history_list.selected_index,
            ClinicalView::VitalSigns => self.vitals_list.selected_index,
            ClinicalView::SocialHistory => 0,
            ClinicalView::FamilyHistory => self.family_history_list.selected_index,
        }
    }

    pub fn scroll_offset(&self) -> usize {
        match self.view {
            ClinicalView::PatientSummary => 0,
            ClinicalView::Consultations => self.consultation_list.scroll_offset,
            ClinicalView::Allergies => self.allergy_list.scroll_offset,
            ClinicalView::MedicalHistory => self.medical_history_list.scroll_offset,
            ClinicalView::VitalSigns => self.vitals_list.scroll_offset,
            ClinicalView::SocialHistory => 0,
            ClinicalView::FamilyHistory => self.family_history_list.scroll_offset,
        }
    }
}
