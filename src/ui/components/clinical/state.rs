use crate::domain::clinical::{
    Allergy, Consultation, FamilyHistory, MedicalHistory, SocialHistory, VitalSigns,
};
use crate::ui::theme::Theme;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub enum ClinicalView {
    #[default]
    Consultations,
    Allergies,
    MedicalHistory,
    VitalSigns,
    SocialHistory,
    FamilyHistory,
}

#[derive(Debug, Clone, Default)]
pub struct ClinicalState {
    pub view: ClinicalView,
    pub selected_patient_id: Option<Uuid>,
    pub loading: bool,
    pub error: Option<String>,
    pub consultations: Vec<Consultation>,
    pub allergies: Vec<Allergy>,
    pub medical_history: Vec<MedicalHistory>,
    pub vital_signs: Vec<VitalSigns>,
    pub social_history: Option<SocialHistory>,
    pub family_history: Vec<FamilyHistory>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub page: usize,
    pub page_size: usize,
    pub theme: Theme,
}

impl ClinicalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            ..Default::default()
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
        false
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

    pub fn cycle_view(&mut self) {
        self.view = match self.view {
            ClinicalView::Consultations => ClinicalView::Allergies,
            ClinicalView::Allergies => ClinicalView::MedicalHistory,
            ClinicalView::MedicalHistory => ClinicalView::VitalSigns,
            ClinicalView::VitalSigns => ClinicalView::SocialHistory,
            ClinicalView::SocialHistory => ClinicalView::FamilyHistory,
            ClinicalView::FamilyHistory => ClinicalView::Consultations,
        };
    }

    pub fn cycle_view_reverse(&mut self) {
        self.view = match self.view {
            ClinicalView::Consultations => ClinicalView::FamilyHistory,
            ClinicalView::Allergies => ClinicalView::Consultations,
            ClinicalView::MedicalHistory => ClinicalView::Allergies,
            ClinicalView::VitalSigns => ClinicalView::MedicalHistory,
            ClinicalView::SocialHistory => ClinicalView::VitalSigns,
            ClinicalView::FamilyHistory => ClinicalView::SocialHistory,
        };
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
        let max = self.current_list_count();
        if self.selected_index + 1 < max {
            self.selected_index += 1;
        }
    }

    pub fn prev_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn current_list_count(&self) -> usize {
        match self.view {
            ClinicalView::Consultations => self.consultations.len(),
            ClinicalView::Allergies => self.allergies.len(),
            ClinicalView::MedicalHistory => self.medical_history.len(),
            ClinicalView::VitalSigns => self.vital_signs.len(),
            ClinicalView::SocialHistory => self.social_history.is_some() as usize,
            ClinicalView::FamilyHistory => self.family_history.len(),
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
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index.saturating_sub(visible_rows) + 1;
        }
    }
}
