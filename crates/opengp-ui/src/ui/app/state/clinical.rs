use crate::ui::app::{App, PendingClinicalSaveData};
use crate::ui::components::clinical::ClinicalState;

impl App {
    pub fn request_refresh_consultations(&mut self, patient_id: uuid::Uuid) {
        self.pending_consultation_list_refresh = Some(patient_id);
    }

    pub fn take_pending_clinical_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_clinical_patient_id.take()
    }

    pub fn take_pending_clinical_save_data(&mut self) -> Option<PendingClinicalSaveData> {
        if !self.authenticated {
            return None;
        }
        self.pending_clinical_save_data.take()
    }

    pub fn clinical_state_mut(&mut self) -> &mut ClinicalState {
        &mut self.clinical_state
    }
}
