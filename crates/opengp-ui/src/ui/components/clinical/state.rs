use crate::ui::components::clinical::{
    AllergyState, ConsultationState, FamilyHistoryState, MedicalHistoryState, SocialHistoryState,
    VitalsState,
};
use crate::ui::theme::Theme;
use opengp_config::{
    healthcare::HealthcareConfig, AllergyConfig, ClinicalConfig, SocialHistoryConfig,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClinicalSubDomain {
    Consultations,
    Allergies,
    Vitals,
    MedicalHistory,
    FamilyHistory,
    SocialHistory,
}

#[derive(Debug, Clone, Default, PartialEq)]
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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ClinicalFormView {
    #[default]
    None,
    AllergyForm,
    ConsultationForm,
    MedicalHistoryForm,
    VitalSignsForm,
    FamilyHistoryForm,
    SocialHistoryForm,
}

#[derive(Clone)]
pub struct ClinicalState {
    pub view: ClinicalView,
    pub form_view: ClinicalFormView,
    pub selected_patient_id: Option<Uuid>,
    pub active_appointment_id: Option<Uuid>,
    pub page: usize,
    pub page_size: usize,
    pub error: Option<String>,
    // Sub-structs encapsulating domain-specific state
    pub consultations: ConsultationState,
    pub allergies: AllergyState,
    pub vitals: VitalsState,
    pub medical_history: MedicalHistoryState,
    pub family_history: FamilyHistoryState,
    pub social_history: SocialHistoryState,
}

impl ClinicalState {
    pub fn new(
        theme: Theme,
        healthcare_config: HealthcareConfig,
        allergy_config: AllergyConfig,
        clinical_config: ClinicalConfig,
        social_history_config: SocialHistoryConfig,
    ) -> Self {
        Self::with_theme(
            theme,
            healthcare_config,
            allergy_config,
            clinical_config,
            social_history_config,
        )
    }

    pub fn with_theme(
        theme: Theme,
        healthcare_config: HealthcareConfig,
        allergy_config: AllergyConfig,
        clinical_config: ClinicalConfig,
        social_history_config: SocialHistoryConfig,
    ) -> Self {
        Self {
            view: ClinicalView::PatientSummary,
            form_view: ClinicalFormView::None,
            selected_patient_id: None,
            active_appointment_id: None,
            page: 0,
            page_size: 20,
            error: None,
            consultations: ConsultationState::new(theme.clone(), clinical_config.clone()),
            allergies: AllergyState::new(theme.clone(), allergy_config),
            vitals: VitalsState::new(theme.clone(), healthcare_config),
            medical_history: MedicalHistoryState::new(theme.clone(), clinical_config),
            family_history: FamilyHistoryState::new(theme.clone()),
            social_history: SocialHistoryState::new(theme, social_history_config),
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
        self.consultations.open_consultation_form();
        self.form_view = ClinicalFormView::ConsultationForm;
    }

    pub fn close_consultation_form(&mut self) {
        self.consultations.close_consultation_form();
        self.form_view = ClinicalFormView::None;
    }

    pub fn open_allergy_detail(
        &mut self,
        allergy: opengp_domain::domain::clinical::Allergy,
        _theme: &Theme,
    ) {
        self.allergies.open_allergy_detail(allergy);
    }

    pub fn close_allergy_detail(&mut self) {
        self.allergies.close_allergy_detail();
    }

    pub fn open_medical_history_detail(
        &mut self,
        medical_history: opengp_domain::domain::clinical::MedicalHistory,
        _theme: &Theme,
    ) {
        self.medical_history
            .open_medical_history_detail(medical_history);
    }

    pub fn close_medical_history_detail(&mut self) {
        self.medical_history.close_medical_history_detail();
    }

    pub fn open_vitals_detail(
        &mut self,
        vitals: opengp_domain::domain::clinical::VitalSigns,
        _theme: &Theme,
    ) {
        self.vitals
            .open_vitals_detail(vitals, self.vitals.theme.clone());
    }

    pub fn close_vitals_detail(&mut self) {
        self.vitals.close_vitals_detail();
    }

    pub fn open_family_history_detail(
        &mut self,
        family_history: opengp_domain::domain::clinical::FamilyHistory,
        _theme: &Theme,
    ) {
        self.family_history
            .open_family_history_detail(family_history);
    }

    pub fn close_family_history_detail(&mut self) {
        self.family_history.close_family_history_detail();
    }

    pub fn open_allergy_form(&mut self) {
        self.allergies.open_allergy_form();
        self.form_view = ClinicalFormView::AllergyForm;
    }

    pub fn open_medical_history_form(&mut self) {
        self.medical_history.open_medical_history_form();
        self.form_view = ClinicalFormView::MedicalHistoryForm;
    }

    pub fn open_vitals_form(&mut self) {
        self.vitals.open_vitals_form(self.vitals.theme.clone());
        self.form_view = ClinicalFormView::VitalSignsForm;
    }

    pub fn open_family_history_form(&mut self) {
        self.family_history.open_family_history_form();
        self.form_view = ClinicalFormView::FamilyHistoryForm;
    }

    pub fn open_social_history_editing(&mut self) {
        self.social_history.open_social_history_editing();
    }

    pub fn close_social_history_editing(&mut self) {
        self.social_history.close_social_history_editing();
    }

    pub fn open_social_history_form(&mut self) {
        self.social_history.open_social_history_form();
        self.form_view = ClinicalFormView::SocialHistoryForm;
    }

    pub fn close_social_history_form(&mut self) {
        self.social_history.close_social_history_form();
        self.form_view = ClinicalFormView::None;
    }

    pub fn close_form(&mut self) {
        self.form_view = ClinicalFormView::None;
        self.allergies.close_allergy_form();
        self.consultations.close_consultation_form();
        self.medical_history.close_medical_history_form();
        self.vitals.close_vitals_form();
        self.family_history.close_family_history_form();
        self.social_history.close_social_history_form();
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
        self.consultations.consultation_list.move_first();
        self.allergies.allergy_list.move_first();
        self.medical_history.medical_history_list.selected_index = 0;
        self.vitals.vitals_list.move_first();
        self.family_history.family_history_list.move_first();
    }

    pub fn set_patient(&mut self, patient_id: Uuid) {
        self.selected_patient_id = Some(patient_id);
    }

    pub fn set_active_appointment(&mut self, id: Uuid) {
        self.active_appointment_id = Some(id);
    }

    pub fn clear_active_appointment(&mut self) {
        self.active_appointment_id = None;
        self.consultations.clear_active_timer();
    }

    pub fn set_active_timer_started_at(&mut self, at: chrono::DateTime<chrono::Utc>) {
        self.consultations.active_timer_started_at = Some(at);
    }

    pub fn clear_patient(&mut self) {
        self.selected_patient_id = None;
        self.active_appointment_id = None;
        self.close_form();
        self.consultations.clear();
        self.allergies.clear();
        self.medical_history.clear();
        self.vitals.clear();
        self.social_history.clear();
        self.family_history.clear();
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.consultations.loading = loading;
        self.allergies.loading = loading;
        self.vitals.loading = loading;
        self.medical_history.loading = loading;
        self.family_history.loading = loading;
        self.social_history.loading = loading;
    }

    pub fn set_sub_loading(&mut self, domain: ClinicalSubDomain, loading: bool) {
        match domain {
            ClinicalSubDomain::Consultations => self.consultations.loading = loading,
            ClinicalSubDomain::Allergies => self.allergies.loading = loading,
            ClinicalSubDomain::Vitals => self.vitals.loading = loading,
            ClinicalSubDomain::MedicalHistory => self.medical_history.loading = loading,
            ClinicalSubDomain::FamilyHistory => self.family_history.loading = loading,
            ClinicalSubDomain::SocialHistory => self.social_history.loading = loading,
        }
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
            ClinicalView::Consultations => self.consultations.consultation_list.next(),
            ClinicalView::Allergies => self.allergies.allergy_list.next(),
            ClinicalView::MedicalHistory => {
                self.medical_history.medical_history_list.selected_index =
                    (self.medical_history.medical_history_list.selected_index + 1).min(
                        self.medical_history
                            .medical_history_list
                            .conditions
                            .len()
                            .saturating_sub(1),
                    )
            }
            ClinicalView::VitalSigns => self.vitals.vitals_list.next(),
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => self.family_history.family_history_list.next(),
        }
    }

    pub fn prev_item(&mut self) {
        match self.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => self.consultations.consultation_list.prev(),
            ClinicalView::Allergies => self.allergies.allergy_list.prev(),
            ClinicalView::MedicalHistory => {
                self.medical_history.medical_history_list.selected_index = self
                    .medical_history
                    .medical_history_list
                    .selected_index
                    .saturating_sub(1)
            }
            ClinicalView::VitalSigns => self.vitals.vitals_list.prev(),
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => self.family_history.family_history_list.prev(),
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

    pub fn has_open_detail_modal(&self) -> bool {
        self.allergies.allergy_detail_modal.is_some()
            || self.medical_history.medical_history_detail_modal.is_some()
            || self.vitals.vitals_detail_modal.is_some()
            || self.family_history.family_history_detail_modal.is_some()
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        match self.view {
            ClinicalView::PatientSummary => {}
            ClinicalView::Consultations => self
                .consultations
                .consultation_list
                .adjust_scroll(visible_rows),
            ClinicalView::Allergies => self.allergies.allergy_list.adjust_scroll(visible_rows),
            ClinicalView::MedicalHistory => self
                .medical_history
                .medical_history_list
                .adjust_scroll(visible_rows),
            ClinicalView::VitalSigns => self.vitals.vitals_list.adjust_scroll(visible_rows),
            ClinicalView::SocialHistory => {}
            ClinicalView::FamilyHistory => self
                .family_history
                .family_history_list
                .adjust_scroll(visible_rows),
        }
    }

    pub fn selected_index(&self) -> usize {
        match self.view {
            ClinicalView::PatientSummary => 0,
            ClinicalView::Consultations => self.consultations.consultation_list.selected_index(),
            ClinicalView::Allergies => self.allergies.allergy_list.selected_index,
            ClinicalView::MedicalHistory => {
                self.medical_history.medical_history_list.selected_index
            }
            ClinicalView::VitalSigns => self.vitals.vitals_list.selected_index,
            ClinicalView::SocialHistory => 0,
            ClinicalView::FamilyHistory => self.family_history.family_history_list.selected_index,
        }
    }

    pub fn scroll_offset(&self) -> usize {
        match self.view {
            ClinicalView::PatientSummary => 0,
            ClinicalView::Consultations => self.consultations.consultation_list.scroll_offset(),
            ClinicalView::Allergies => self.allergies.allergy_list.scroll_offset,
            ClinicalView::MedicalHistory => self.medical_history.medical_history_list.scroll_offset,
            ClinicalView::VitalSigns => self.vitals.vitals_list.scroll_offset,
            ClinicalView::SocialHistory => 0,
            ClinicalView::FamilyHistory => self.family_history.family_history_list.scroll_offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper to create a test state
    fn test_state() -> ClinicalState {
        ClinicalState::with_theme(
            Theme::dark(),
            HealthcareConfig::default(),
            AllergyConfig::default(),
            ClinicalConfig::default(),
            SocialHistoryConfig::default(),
        )
    }

    // Test 1: View cycling forward
    #[test]
    fn test_cycle_view_forward() {
        let mut state = test_state();
        assert!(matches!(state.view, ClinicalView::PatientSummary));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::Consultations));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::Allergies));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::MedicalHistory));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::VitalSigns));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::SocialHistory));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::FamilyHistory));

        state.cycle_view();
        assert!(matches!(state.view, ClinicalView::PatientSummary)); // Wraps around
    }

    // Test 2: View cycling reverse
    #[test]
    fn test_cycle_view_reverse() {
        let mut state = test_state();
        assert!(matches!(state.view, ClinicalView::PatientSummary));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::FamilyHistory));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::SocialHistory));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::VitalSigns));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::MedicalHistory));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::Allergies));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::Consultations));

        state.cycle_view_reverse();
        assert!(matches!(state.view, ClinicalView::PatientSummary)); // Wraps around
    }

    // Test 3: Allergy form open/close lifecycle
    #[test]
    fn test_allergy_form_lifecycle() {
        let mut state = test_state();

        // Initially no form open
        assert!(!state.is_form_open());
        assert_eq!(state.form_view, ClinicalFormView::None);
        assert!(state.allergies.allergy_form.is_none());

        // Open allergy form
        state.open_allergy_form();
        assert!(state.is_form_open());
        assert!(matches!(state.form_view, ClinicalFormView::AllergyForm));
        assert!(state.allergies.allergy_form.is_some());

        // Close form
        state.close_form();
        assert!(!state.is_form_open());
        assert!(matches!(state.form_view, ClinicalFormView::None));
        assert!(state.allergies.allergy_form.is_none());
    }

    // Test 4: Consultation form lifecycle
    #[test]
    fn test_consultation_form_lifecycle() {
        let mut state = test_state();

        state.open_consultation_form();
        assert!(state.is_form_open());
        assert!(matches!(
            state.form_view,
            ClinicalFormView::ConsultationForm
        ));
        assert!(state.consultations.consultation_form.is_some());

        state.close_consultation_form();
        assert!(!state.is_form_open());
        assert!(matches!(state.form_view, ClinicalFormView::None));
        assert!(state.consultations.consultation_form.is_none());
    }

    // Test 5: Medical history form lifecycle
    #[test]
    fn test_medical_history_form_lifecycle() {
        let mut state = test_state();

        state.open_medical_history_form();
        assert!(state.is_form_open());
        assert!(matches!(
            state.form_view,
            ClinicalFormView::MedicalHistoryForm
        ));
        assert!(state.medical_history.medical_history_form.is_some());

        state.close_form();
        assert!(!state.is_form_open());
        assert!(state.medical_history.medical_history_form.is_none());
    }

    // Test 6: Set and clear patient
    #[test]
    fn test_set_and_clear_patient() {
        let mut state = test_state();
        let patient_id = Uuid::new_v4();

        // Initially no patient
        assert!(!state.has_patient());
        assert_eq!(state.selected_patient_id, None);

        // Set patient
        state.set_patient(patient_id);
        assert!(state.has_patient());
        assert_eq!(state.selected_patient_id, Some(patient_id));

        // Open a form to ensure close_form is tested
        state.open_allergy_form();
        assert!(state.is_form_open());

        // Clear patient - should clear all data and close forms
        state.clear_patient();
        assert!(!state.has_patient());
        assert_eq!(state.selected_patient_id, None);
        assert_eq!(state.consultations.consultations.len(), 0);
        assert_eq!(state.allergies.allergies.len(), 0);
        assert_eq!(state.medical_history.medical_history.len(), 0);
        assert_eq!(state.vitals.vital_signs.len(), 0);
        assert_eq!(state.family_history.family_history.len(), 0);
        assert!(!state.is_form_open());
    }

    // Test 7: Page size calculation from height
    #[test]
    fn test_set_page_size() {
        let mut state = test_state();

        // Height 20 → page_size 14
        state.set_page_size(20);
        assert_eq!(state.page_size, 14);

        // Height 6 → page_size 5 (minimum enforced)
        state.set_page_size(6);
        assert_eq!(state.page_size, 5);

        // Height 0 → page_size 5 (minimum enforced)
        state.set_page_size(0);
        assert_eq!(state.page_size, 5);

        // Height 11 → page_size 5 (minimum)
        state.set_page_size(11);
        assert_eq!(state.page_size, 5);

        // Height 30 → page_size 24
        state.set_page_size(30);
        assert_eq!(state.page_size, 24);
    }

    // Test 8: Total pages calculation
    #[test]
    fn test_total_pages() {
        let mut state = test_state();
        state.page_size = 20;

        // 0 items → 1 page
        assert_eq!(state.total_pages(0), 1);

        // 20 items with page_size 20 → 1 page
        assert_eq!(state.total_pages(20), 1);

        // 21 items → 2 pages
        assert_eq!(state.total_pages(21), 2);

        // 40 items → 2 pages
        assert_eq!(state.total_pages(40), 2);

        // 41 items → 3 pages
        assert_eq!(state.total_pages(41), 3);
    }

    // Test 9: Page offset calculation
    #[test]
    fn test_page_offset() {
        let mut state = test_state();
        state.page_size = 20;

        // Page 0 → offset 0
        state.page = 0;
        assert_eq!(state.page_offset(), 0);

        // Page 1 → offset 20
        state.page = 1;
        assert_eq!(state.page_offset(), 20);

        // Page 2 → offset 40
        state.page = 2;
        assert_eq!(state.page_offset(), 40);
    }

    // Test 10: Form view predicates
    #[test]
    fn test_form_predicates() {
        let mut state = test_state();

        // Initially form closed
        assert!(!state.is_form_open());
        assert!(!state.is_form_view());

        // Open a form
        state.open_allergy_form();
        assert!(state.is_form_open());
        assert!(state.is_form_view());

        // Close form
        state.close_form();
        assert!(!state.is_form_open());
        assert!(!state.is_form_view());
    }

    // Test 11: List view predicates
    #[test]
    fn test_list_view_predicates() {
        let mut state = test_state();

        // PatientSummary is a list view
        state.view = ClinicalView::PatientSummary;
        assert!(state.is_list_view());

        // Consultations is a list view
        state.view = ClinicalView::Consultations;
        assert!(state.is_list_view());

        // Allergies is a list view
        state.view = ClinicalView::Allergies;
        assert!(state.is_list_view());

        // SocialHistory is NOT a list view
        state.view = ClinicalView::SocialHistory;
        assert!(!state.is_list_view());

        // FamilyHistory is a list view
        state.view = ClinicalView::FamilyHistory;
        assert!(state.is_list_view());
    }

    // Test 12: Next/prev item delegation
    #[test]
    fn test_next_prev_delegation() {
        let mut state = test_state();
        state.view = ClinicalView::PatientSummary;

        // PatientSummary has no-op next/prev
        state.next_item();
        state.prev_item();
        // Should not panic

        // Consultations delegates to consultation_list
        state.view = ClinicalView::Consultations;
        state.next_item(); // Should not panic
        state.prev_item(); // Should not panic

        // SocialHistory has no-op next/prev
        state.view = ClinicalView::SocialHistory;
        state.next_item();
        state.prev_item();
        // Should not panic
    }

    // Test 13: Selected index delegation
    #[test]
    fn test_selected_index_delegation() {
        let mut state = test_state();

        // PatientSummary → 0
        state.view = ClinicalView::PatientSummary;
        assert_eq!(state.selected_index(), 0);

        // Consultations delegates to consultation_list
        state.view = ClinicalView::Consultations;
        let idx = state.selected_index();
        assert_eq!(idx, state.consultations.consultation_list.selected_index());

        // SocialHistory → 0
        state.view = ClinicalView::SocialHistory;
        assert_eq!(state.selected_index(), 0);

        // FamilyHistory delegates to family_history_list
        state.view = ClinicalView::FamilyHistory;
        assert_eq!(
            state.selected_index(),
            state.family_history.family_history_list.selected_index
        );
    }

    // Test 14: Scroll offset delegation
    #[test]
    fn test_scroll_offset_delegation() {
        let mut state = test_state();

        // PatientSummary → 0
        state.view = ClinicalView::PatientSummary;
        assert_eq!(state.scroll_offset(), 0);

        // Consultations delegates to consultation_list
        state.view = ClinicalView::Consultations;
        assert_eq!(
            state.scroll_offset(),
            state.consultations.consultation_list.scroll_offset()
        );

        // Allergies delegates to allergy_list
        state.view = ClinicalView::Allergies;
        assert_eq!(
            state.scroll_offset(),
            state.allergies.allergy_list.scroll_offset
        );

        // SocialHistory → 0
        state.view = ClinicalView::SocialHistory;
        assert_eq!(state.scroll_offset(), 0);
    }

    // Test 15: VitalSigns form lifecycle
    #[test]
    fn test_vitals_form_lifecycle() {
        let mut state = test_state();

        state.open_vitals_form();
        assert!(state.is_form_open());
        assert!(matches!(state.form_view, ClinicalFormView::VitalSignsForm));
        assert!(state.vitals.vitals_form.is_some());

        state.close_form();
        assert!(!state.is_form_open());
        assert!(state.vitals.vitals_form.is_none());
    }

    // Test 16: FamilyHistory form lifecycle
    #[test]
    fn test_family_history_form_lifecycle() {
        let mut state = test_state();

        state.open_family_history_form();
        assert!(state.is_form_open());
        assert!(matches!(
            state.form_view,
            ClinicalFormView::FamilyHistoryForm
        ));
        assert!(state.family_history.family_history_form.is_some());

        state.close_form();
        assert!(!state.is_form_open());
        assert!(state.family_history.family_history_form.is_none());
    }

    #[test]
    fn test_has_open_detail_modal() {
        let state = test_state();
        assert!(!state.has_open_detail_modal());
    }

    #[test]
    fn test_set_sub_loading_scoped() {
        use super::ClinicalSubDomain;
        let mut state = test_state();

        // Initially all loading should be false
        assert!(!state.consultations.loading);
        assert!(!state.allergies.loading);
        assert!(!state.vitals.loading);

        // Setting one domain's loading should NOT affect others
        state.set_sub_loading(ClinicalSubDomain::Allergies, true);
        assert!(state.allergies.loading);
        assert!(!state.consultations.loading);
        assert!(!state.vitals.loading);
        assert!(!state.medical_history.loading);
        assert!(!state.family_history.loading);
        assert!(!state.social_history.loading);

        // Setting another domain should not affect Allergies
        state.set_sub_loading(ClinicalSubDomain::Vitals, true);
        assert!(state.allergies.loading); // Still true
        assert!(state.vitals.loading); // Now true
        assert!(!state.consultations.loading);
    }
}
