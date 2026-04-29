use crate::ui::components::shared::PaginatedState;
use opengp_domain::domain::appointment::Appointment;
use uuid::Uuid;

#[derive(Clone)]
pub struct PatientAppointmentState {
    pub patient_id: Uuid,
    pub appointments: Vec<Appointment>,
    pub pagination: PaginatedState,
    pub loading: bool,
    pub error: Option<String>,
    pub selected_index: Option<usize>,
    pub detail_modal: bool,
}

impl PatientAppointmentState {
    pub fn new(patient_id: Uuid) -> Self {
        Self {
            patient_id,
            appointments: Vec::new(),
            pagination: PaginatedState::default(),
            loading: false,
            error: None,
            selected_index: None,
            detail_modal: false,
        }
    }
}
