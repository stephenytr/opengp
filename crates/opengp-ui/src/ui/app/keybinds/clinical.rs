use crate::ui::app::App;
use crate::ui::keybinds::Action;
use crossterm::event::KeyEvent;

impl App {
    pub(crate) fn handle_clinical_keys(&mut self, _key: KeyEvent) -> Action {
        Action::Unknown
    }

    fn handle_consultation_modal_action(
        &mut self,
        _action: crate::ui::components::clinical::ConsultationDetailModalAction,
    ) {
    }

    fn handle_allergy_modal_action(
        &mut self,
        _action: crate::ui::components::clinical::AllergyDetailModalAction,
    ) {
    }

    fn handle_medical_history_modal_action(
        &mut self,
        _action: crate::ui::components::clinical::MedicalHistoryDetailModalAction,
    ) {
    }

    fn handle_vitals_modal_action(
        &mut self,
        _action: crate::ui::components::clinical::VitalsDetailModalAction,
    ) {
    }

    fn handle_family_history_modal_action(
        &mut self,
        _action: crate::ui::components::clinical::FamilyHistoryDetailModalAction,
    ) {
    }
}
