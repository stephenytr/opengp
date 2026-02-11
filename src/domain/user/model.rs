use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,

    pub first_name: String,
    pub last_name: String,

    pub role: Role,
    pub additional_permissions: Vec<Permission>,

    pub is_active: bool,
    pub is_locked: bool,
    pub failed_login_attempts: u8,
    pub last_login: Option<DateTime<Utc>>,
    pub password_changed_at: DateTime<Utc>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn has_permission(&self, permission: Permission) -> bool {
        if self.role.permissions().contains(&permission) {
            return true;
        }
        self.additional_permissions.contains(&permission)
    }

    pub fn can_access_patient(&self, _patient_id: Uuid) -> bool {
        self.has_permission(Permission::PatientRead)
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Doctor,
    Nurse,
    Receptionist,
    Billing,
}

impl Role {
    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;

        match self {
            Role::Admin => &[
                PatientRead,
                PatientCreate,
                PatientUpdate,
                PatientDelete,
                PatientSearch,
                PatientExport,
                ClinicalRead,
                ClinicalWrite,
                ClinicalSign,
                ClinicalDelete,
                PrescriptionCreate,
                PrescriptionCancel,
                PrescriptionAuthority,
                BillingRead,
                BillingCreate,
                BillingProcess,
                UserManage,
                SystemConfig,
                AuditView,
            ],
            Role::Doctor => &[
                PatientRead,
                PatientCreate,
                PatientUpdate,
                PatientSearch,
                ClinicalRead,
                ClinicalWrite,
                ClinicalSign,
                PrescriptionCreate,
                PrescriptionCancel,
                PrescriptionAuthority,
                BillingRead,
            ],
            Role::Nurse => &[
                PatientRead,
                PatientUpdate,
                PatientSearch,
                ClinicalRead,
                ClinicalWrite,
                PrescriptionCreate,
            ],
            Role::Receptionist => &[
                PatientRead,
                PatientCreate,
                PatientUpdate,
                PatientSearch,
                BillingRead,
                BillingCreate,
            ],
            Role::Billing => &[PatientRead, BillingRead, BillingCreate, BillingProcess],
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Doctor => write!(f, "Doctor"),
            Role::Nurse => write!(f, "Nurse"),
            Role::Receptionist => write!(f, "Receptionist"),
            Role::Billing => write!(f, "Billing"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    PatientRead,
    PatientCreate,
    PatientUpdate,
    PatientDelete,
    PatientSearch,
    PatientExport,

    ClinicalRead,
    ClinicalWrite,
    ClinicalSign,
    ClinicalDelete,

    PrescriptionCreate,
    PrescriptionCancel,
    PrescriptionAuthority,

    BillingRead,
    BillingCreate,
    BillingProcess,

    UserManage,
    SystemConfig,
    AuditView,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::PatientRead => write!(f, "Patient Read"),
            Permission::PatientCreate => write!(f, "Patient Create"),
            Permission::PatientUpdate => write!(f, "Patient Update"),
            Permission::PatientDelete => write!(f, "Patient Delete"),
            Permission::PatientSearch => write!(f, "Patient Search"),
            Permission::PatientExport => write!(f, "Patient Export"),
            Permission::ClinicalRead => write!(f, "Clinical Read"),
            Permission::ClinicalWrite => write!(f, "Clinical Write"),
            Permission::ClinicalSign => write!(f, "Clinical Sign"),
            Permission::ClinicalDelete => write!(f, "Clinical Delete"),
            Permission::PrescriptionCreate => write!(f, "Prescription Create"),
            Permission::PrescriptionCancel => write!(f, "Prescription Cancel"),
            Permission::PrescriptionAuthority => write!(f, "Prescription Authority"),
            Permission::BillingRead => write!(f, "Billing Read"),
            Permission::BillingCreate => write!(f, "Billing Create"),
            Permission::BillingProcess => write!(f, "Billing Process"),
            Permission::UserManage => write!(f, "User Management"),
            Permission::SystemConfig => write!(f, "System Configuration"),
            Permission::AuditView => write!(f, "Audit View"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Practitioner {
    pub id: Uuid,
    pub user_id: Option<Uuid>,

    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub title: String,

    pub hpi_i: Option<String>,
    pub ahpra_registration: Option<String>,
    pub prescriber_number: Option<String>,
    pub provider_number: String,

    pub speciality: Option<String>,
    pub qualifications: Vec<String>,

    pub phone: Option<String>,
    pub email: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Practitioner {
    pub fn full_name(&self) -> String {
        if let Some(ref middle) = self.middle_name {
            format!("{} {} {}", self.first_name, middle, self.last_name)
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} {}", self.title, self.last_name)
    }
}
