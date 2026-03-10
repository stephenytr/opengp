use chrono::NaiveDate;

use crate::ui::app::{
    App, AppointmentStatusTransition, PendingClinicalSaveData, PendingPatientData,
};
use crate::ui::components::appointment::AppointmentState;
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::patient::PatientForm;
use crate::ui::keybinds::KeyContext;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};

impl App {
    /// Load patients into the list
    pub fn load_patients(&mut self, patients: Vec<opengp_domain::domain::patient::Patient>) {
        let list_items: Vec<PatientListItem> =
            patients.into_iter().map(PatientListItem::from).collect();
        self.patient_list.set_patients(list_items);
    }

    /// Take pending patient data (for saving to database)
    pub fn take_pending_patient_data(&mut self) -> Option<PendingPatientData> {
        self.pending_patient_data.take()
    }

    /// Take pending patient ID to load for editing
    pub fn take_pending_edit_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_edit_patient_id.take()
    }

    /// Set pending patient ID to load for editing (from UI event)
    pub fn request_edit_patient(&mut self, patient_id: uuid::Uuid) {
        self.pending_edit_patient_id = Some(patient_id);
    }

    /// Take pending appointment date (for loading practitioners in main loop)
    pub fn take_pending_appointment_date(&mut self) -> Option<NaiveDate> {
        self.pending_appointment_date.take()
    }

    /// Request loading practitioners for appointment form picker
    pub fn request_load_practitioners(&mut self) {
        self.pending_load_practitioners = true;
    }

    /// Take pending load practitioners flag
    pub fn take_pending_load_practitioners(&mut self) -> bool {
        std::mem::take(&mut self.pending_load_practitioners)
    }

    /// Take pending appointment save data (for saving to database in main loop)
    pub fn take_pending_appointment_save(
        &mut self,
    ) -> Option<opengp_domain::domain::appointment::NewAppointmentData> {
        self.pending_appointment_save.take()
    }

    pub fn take_pending_appointment_status_transition(
        &mut self,
    ) -> Option<(uuid::Uuid, AppointmentStatusTransition)> {
        self.pending_appointment_status_transition.take()
    }

    pub fn take_pending_clinical_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_clinical_patient_id.take()
    }

    pub fn take_pending_clinical_save_data(&mut self) -> Option<PendingClinicalSaveData> {
        self.pending_clinical_save_data.take()
    }

    /// Set an error message on the status bar (for use by main loop)
    pub fn set_status_error(&mut self, message: impl Into<String>) {
        self.status_bar.set_error(message);
    }

    /// Get mutable reference to appointment state (for loading practitioners)
    pub fn appointment_state_mut(&mut self) -> &mut AppointmentState {
        &mut self.appointment_state
    }

    /// Set patients in the appointment form picker
    pub fn appointment_form_set_patients(&mut self, patients: Vec<PatientListItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_patients(patients);
        }
    }

    /// Set practitioners in the appointment form picker
    pub fn appointment_form_set_practitioners(&mut self, practitioners: Vec<PractitionerViewItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_practitioners(practitioners);
        }
    }

    pub fn clinical_state_mut(&mut self) -> &mut ClinicalState {
        &mut self.clinical_state
    }

    /// Open patient form for editing (called from main loop after fetching patient)
    pub fn open_patient_form(&mut self, patient: opengp_domain::domain::patient::Patient) {
        self.patient_form = Some(PatientForm::from_patient(patient, self.theme.clone()));
        self.current_context = KeyContext::PatientForm;
    }

    /// Get patients from the patient list
    pub fn patient_list_patients(&self) -> &[PatientListItem] {
        self.patient_list.patients()
    }
}
