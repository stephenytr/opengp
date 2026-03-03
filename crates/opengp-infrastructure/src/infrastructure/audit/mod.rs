use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: AuditAction,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<Uuid>,
    pub result: AuditResult,
    pub timestamp: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(user_id: Uuid, action: AuditAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            action,
            entity_type: None,
            entity_id: None,
            metadata: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            result: AuditResult::Success,
            timestamp: Utc::now(),
        }
    }

    pub fn with_entity(mut self, entity_type: String, entity_id: Uuid) -> Self {
        self.entity_type = Some(entity_type);
        self.entity_id = Some(entity_id);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_failure(mut self) -> Self {
        self.result = AuditResult::Failure;
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    Login,
    Logout,
    LoginFailed,
    MFAEnabled,
    MFADisabled,
    MFAFailed,

    PatientCreate,
    PatientRead,
    PatientUpdate,
    PatientDelete,
    PatientSearch,
    PatientExport,

    ConsultationCreate,
    ConsultationRead,
    ConsultationUpdate,
    ConsultationSign,
    ConsultationDelete,

    PrescriptionCreate,
    PrescriptionRead,
    PrescriptionCancel,

    AppointmentCreate,
    AppointmentRead,
    AppointmentUpdate,
    AppointmentCancel,

    BillingCreate,
    BillingRead,
    BillingProcess,

    ImmunisationCreate,
    ImmunisationRead,
    AIRSubmit,

    ReferralCreate,
    ReferralRead,
    ReferralSend,

    PathologyOrderCreate,
    PathologyResultsReceived,
    PathologyResultsAcknowledged,

    ReportGenerate,
    DataExport,

    UserCreate,
    UserUpdate,
    UserDelete,

    SystemConfigChange,
    AuditLogAccess,

    BreakGlassAccess,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Login => write!(f, "Login"),
            AuditAction::Logout => write!(f, "Logout"),
            AuditAction::LoginFailed => write!(f, "Login Failed"),
            AuditAction::MFAEnabled => write!(f, "MFA Enabled"),
            AuditAction::MFADisabled => write!(f, "MFA Disabled"),
            AuditAction::MFAFailed => write!(f, "MFA Failed"),
            AuditAction::PatientCreate => write!(f, "Patient Create"),
            AuditAction::PatientRead => write!(f, "Patient Read"),
            AuditAction::PatientUpdate => write!(f, "Patient Update"),
            AuditAction::PatientDelete => write!(f, "Patient Delete"),
            AuditAction::PatientSearch => write!(f, "Patient Search"),
            AuditAction::PatientExport => write!(f, "Patient Export"),
            AuditAction::ConsultationCreate => write!(f, "Consultation Create"),
            AuditAction::ConsultationRead => write!(f, "Consultation Read"),
            AuditAction::ConsultationUpdate => write!(f, "Consultation Update"),
            AuditAction::ConsultationSign => write!(f, "Consultation Sign"),
            AuditAction::ConsultationDelete => write!(f, "Consultation Delete"),
            AuditAction::PrescriptionCreate => write!(f, "Prescription Create"),
            AuditAction::PrescriptionRead => write!(f, "Prescription Read"),
            AuditAction::PrescriptionCancel => write!(f, "Prescription Cancel"),
            AuditAction::AppointmentCreate => write!(f, "Appointment Create"),
            AuditAction::AppointmentRead => write!(f, "Appointment Read"),
            AuditAction::AppointmentUpdate => write!(f, "Appointment Update"),
            AuditAction::AppointmentCancel => write!(f, "Appointment Cancel"),
            AuditAction::BillingCreate => write!(f, "Billing Create"),
            AuditAction::BillingRead => write!(f, "Billing Read"),
            AuditAction::BillingProcess => write!(f, "Billing Process"),
            AuditAction::ImmunisationCreate => write!(f, "Immunisation Create"),
            AuditAction::ImmunisationRead => write!(f, "Immunisation Read"),
            AuditAction::AIRSubmit => write!(f, "AIR Submit"),
            AuditAction::ReferralCreate => write!(f, "Referral Create"),
            AuditAction::ReferralRead => write!(f, "Referral Read"),
            AuditAction::ReferralSend => write!(f, "Referral Send"),
            AuditAction::PathologyOrderCreate => write!(f, "Pathology Order Create"),
            AuditAction::PathologyResultsReceived => write!(f, "Pathology Results Received"),
            AuditAction::PathologyResultsAcknowledged => {
                write!(f, "Pathology Results Acknowledged")
            }
            AuditAction::ReportGenerate => write!(f, "Report Generate"),
            AuditAction::DataExport => write!(f, "Data Export"),
            AuditAction::UserCreate => write!(f, "User Create"),
            AuditAction::UserUpdate => write!(f, "User Update"),
            AuditAction::UserDelete => write!(f, "User Delete"),
            AuditAction::SystemConfigChange => write!(f, "System Config Change"),
            AuditAction::AuditLogAccess => write!(f, "Audit Log Access"),
            AuditAction::BreakGlassAccess => write!(f, "Break Glass Access"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditResult {
    Success,
    Failure,
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Success => write!(f, "Success"),
            AuditResult::Failure => write!(f, "Failure"),
        }
    }
}
