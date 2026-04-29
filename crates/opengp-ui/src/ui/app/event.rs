use chrono::NaiveDate;
use crossterm::event::Event as CrosstermEvent;
use opengp_domain::domain::api::LoginResponse;
use opengp_domain::domain::appointment::CalendarDayView;
use opengp_domain::domain::billing::{Invoice, MedicareClaim};
use uuid::Uuid;

use crate::ui::app::ClinicalWorkspaceLoadResult;
use crate::ui::components::SubtabKind;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};

pub type BookedSlot = chrono::NaiveTime;

#[derive(Debug, Clone)]
pub enum ClinicalSaveOutcome {
    Saved,
}

#[derive(Debug, Clone)]
pub enum BillingSaveOutcome {
    Saved,
}

#[derive(Debug, Clone)]
pub struct BillingWorkspaceData {
    pub invoices: Vec<Invoice>,
    pub claims: Vec<MedicareClaim>,
}

#[derive(Debug)]
pub enum AppEvent {
    // Terminal passthrough
    Term(CrosstermEvent),
    // Task results (replacing AppCommand result-back variants)
    AppointmentSaved(Result<(), String>),
    AppointmentsRefreshed(Result<CalendarDayView, String>),
    AppointmentStatusUpdated(Result<(Uuid, NaiveDate), String>),
    AppointmentRescheduled(Result<(Uuid, NaiveDate), String>),
    AppointmentCancelled(Result<(), String>),
    PractitionersLoaded(Result<Vec<PractitionerViewItem>, String>),
    AvailableSlotsLoaded(Result<Vec<BookedSlot>, String>),
    ClinicalDataSaved(Result<ClinicalSaveOutcome, String>),
    BillingDataSaved(Result<BillingSaveOutcome, String>),
    PatientWorkspaceDataLoaded {
        patient_id: Uuid,
        subtab: SubtabKind,
        result: Result<ClinicalWorkspaceLoadResult, String>,
    },
    BillingDataLoaded(Result<BillingWorkspaceData, String>),
    PatientListLoaded(Result<Vec<PatientListItem>, String>),
    LoginResult(Result<LoginResponse, String>),
}

impl From<CrosstermEvent> for AppEvent {
    fn from(e: CrosstermEvent) -> Self {
        Self::Term(e)
    }
}
