use crate::ui::app::{App, AppCommand, PendingPatientData};
use crate::ui::components::patient::PatientForm;
use crate::ui::components::SubtabKind;
use crate::ui::components::tabs::Tab;
use crate::ui::components::workspace::WorkspaceError;
use crate::ui::keybinds::KeyContext;
use crate::ui::view_models::PatientListItem;

impl App {
    pub fn request_refresh_patients(&mut self) {
        self.pending_patient_list_refresh = true;
    }

    pub fn load_patients(&mut self, patients: Vec<opengp_domain::domain::patient::Patient>) {
        let list_items: Vec<PatientListItem> =
            patients.into_iter().map(PatientListItem::from).collect();
        self.patient_list.set_patients(list_items);
    }

    pub fn take_pending_patient_data(&mut self) -> Option<PendingPatientData> {
        if !self.authenticated {
            return None;
        }
        self.pending_patient_data.take()
    }

    pub fn take_pending_edit_patient_id(&mut self) -> Option<uuid::Uuid> {
        self.pending_edit_patient_id.take()
    }

    pub fn request_edit_patient(&mut self, patient_id: uuid::Uuid) {
        self.pending_edit_patient_id = Some(patient_id);
    }

    pub fn open_patient_form(&mut self, patient: opengp_domain::domain::patient::Patient) {
        self.patient_form = Some(PatientForm::from_patient(
            patient,
            self.theme.clone(),
            &self.patient_config,
        ));
        self.current_context = KeyContext::PatientForm;
    }

    pub fn patient_list_patients(&self) -> &[PatientListItem] {
        self.patient_list.patients()
    }

    pub fn open_patient_workspace(&mut self, patient: PatientListItem) -> Result<usize, WorkspaceError> {
        let patient_id = patient.id;
        let index = self.workspace_manager.open_patient(patient)?;
        self.workspace_manager.active_index = Some(index);
        self.current_context = KeyContext::PatientWorkspace;
        self.tab_bar.select(Tab::PatientWorkspace);
        self.refresh_status_bar();
        self.refresh_context();

        if !self.workspace_manager.is_subtab_loaded(SubtabKind::Clinical)
            && !self.workspace_manager.is_subtab_loading(SubtabKind::Clinical)
        {
            let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
                patient_id,
                subtab: SubtabKind::Clinical,
            });
        }

        Ok(index)
    }
}
