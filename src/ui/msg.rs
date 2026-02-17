use crate::domain::appointment::AppointmentStatus;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    AppClose,
    Tick,
    Render,
    // Navigation
    NavigateTo(NavigationTarget),
    NavigateToTab(usize),

    // Patient domain
    PatientsLoaded,
    PatientSelected(Uuid),
    PatientCreate,
    PatientEdit(Uuid),
    PatientFormSubmit(Uuid),
    PatientFormCancel,
    PatientSaved,
    PatientDeleted(Uuid),
    PatientSearch(String),

    // Appointment domain
    AppointmentsLoaded,
    AppointmentSelected(Uuid),
    AppointmentCreate,
    AppointmentFormSubmit(Uuid),
    AppointmentFormCancel,
    AppointmentStatusChanged(Uuid, AppointmentStatus),
    AppointmentMarkArrived(Uuid),
    AppointmentMarkInProgress(Uuid),
    AppointmentMarkCompleted(Uuid),
    AppointmentMarkNoShow(Uuid),
    AppointmentReschedule(Uuid),
    AppointmentBatchMarkArrived(Vec<Uuid>),
    AppointmentBatchMarkCompleted(Vec<Uuid>),

    // Clinical domain
    ClinicalPatientSelected(Uuid),
    ClinicalPatientClear,
    ClinicalSearchPatients(String),
    NavigateToClinicalWithPatient(Uuid),
    ClinicalConsultationCreate(Uuid),
    ClinicalConsultationEdit(Uuid),
    ClinicalConsultationSign(Uuid),
    ClinicalConsultationSave(Uuid),
    ClinicalConsultationCancel,
    ClinicalAllergyAdd(Uuid),
    ClinicalAllergyEdit(Uuid),
    ClinicalAllergyDeactivate(Uuid),
    ClinicalAllergySave,
    ClinicalAllergyCancel,
    ClinicalVitalSignsRecord(Uuid),
    ClinicalVitalSignsSave,
    ClinicalVitalSignsCancel,
    ClinicalMedicalHistoryAdd(Uuid),
    ClinicalMedicalHistoryEdit(Uuid),
    ClinicalMedicalHistorySave,
    ClinicalMedicalHistoryCancel,
    ClinicalFamilyHistoryAdd(Uuid),
    ClinicalFamilyHistoryEdit(Uuid),
    ClinicalFamilyHistoryDelete(Uuid),
    ClinicalFamilyHistorySave,
    ClinicalFamilyHistoryCancel,
    ClinicalSocialHistoryEdit(Uuid),
    ClinicalSocialHistorySave,
    ClinicalSocialHistoryCancel,

    // View mode
    ClinicalShowOverview,
    ClinicalShowConsultations,
    ClinicalShowAllergies,
    ClinicalShowMedicalHistory,
    ClinicalShowFamilyHistory,
    ClinicalShowSocialHistory,

    // Modal
    ShowHelp,
    ShowDetail(Uuid),
    ShowSearch,
    ShowConfirmation(ConfirmationData),
    ShowError(String),
    ShowReschedule(Uuid),
    ShowFilter,
    ShowPractitionerSelect,
    ShowAudit(Uuid),
    ShowBatch,
    HideModal,

    // Calendar
    CalendarPreviousMonth,
    CalendarNextMonth,
    CalendarJumpToToday,
    CalendarSelectDay(u32),
    CalendarToggleStatusFilter(AppointmentStatus),
    CalendarTogglePractitionerFilter(Uuid),

    // Error
    Error(String),
    ErrorClear,

    // Input
    InputChanged(String),
    InputSubmitted(String),
    InputBlur,

    // Select
    SelectChanged(usize, String),
    SelectOpen,
    SelectClose,

    // List
    ListItemSelected(usize, String),
    ListItemActivated(usize, String),
    ListScrollUp,
    ListScrollDown,

    // Button
    ButtonPressed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationTarget {
    Patients,
    PatientForm(Option<Uuid>),
    Appointments,
    AppointmentForm(Option<Uuid>),
    Clinical,
    ClinicalWithPatient(Uuid),
    Billing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmationData {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
}

impl ConfirmationData {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
        }
    }

    pub fn with_labels(
        title: impl Into<String>,
        message: impl Into<String>,
        confirm: impl Into<String>,
        cancel: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: confirm.into(),
            cancel_label: cancel.into(),
        }
    }
}

impl Default for ConfirmationData {
    fn default() -> Self {
        Self::new("Confirm Action", "Are you sure you want to proceed?")
    }
}
