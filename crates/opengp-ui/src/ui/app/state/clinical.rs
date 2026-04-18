use crate::ui::app::App;

impl App {
    pub fn request_refresh_consultations(&mut self, _patient_id: uuid::Uuid) {}

    pub fn take_pending_clinical_patient_id(&mut self) -> Option<uuid::Uuid> {
        if !self.authenticated {
            return None;
        }
        todo!("Moved to workspace subtab in Task 28")
    }

    pub fn take_pending_clinical_save_data(&mut self) -> Option<crate::ui::app::PendingClinicalSaveData> {
        if !self.authenticated {
            return None;
        }
        todo!("Moved to workspace subtab in Task 28")
    }

    pub fn clinical_state_mut(&mut self) -> &mut crate::ui::components::clinical::ClinicalState {
        todo!("Moved to workspace subtab in Task 28")
    }
}
