use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::dto::NewUserData;
use super::error::UserError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: Option<String>,
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
    pub fn new(data: NewUserData) -> Result<Self, UserError> {
        Self::validate_username(&data.username)?;
        Self::validate_names(&data.first_name, &data.last_name)?;

        if let Some(ref email) = data.email {
            Self::validate_email(email)?;
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            username: data.username,
            password_hash: None,
            email: data.email,
            first_name: data.first_name,
            last_name: data.last_name,
            role: data.role,
            additional_permissions: data.additional_permissions.unwrap_or_default(),
            is_active: true,
            is_locked: false,
            failed_login_attempts: 0,
            last_login: None,
            password_changed_at: now,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_username(username: &str) -> Result<(), UserError> {
        if username.trim().is_empty() {
            return Err(UserError::Validation(
                "Username cannot be empty".to_string(),
            ));
        }
        if username.len() < 3 {
            return Err(UserError::Validation(
                "Username must be at least 3 characters".to_string(),
            ));
        }
        if username.len() > 50 {
            return Err(UserError::Validation(
                "Username cannot exceed 50 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_names(first_name: &str, last_name: &str) -> Result<(), UserError> {
        if first_name.trim().is_empty() {
            return Err(UserError::Validation(
                "First name cannot be empty".to_string(),
            ));
        }
        if last_name.trim().is_empty() {
            return Err(UserError::Validation(
                "Last name cannot be empty".to_string(),
            ));
        }
        if first_name.len() > 100 {
            return Err(UserError::Validation(
                "First name cannot exceed 100 characters".to_string(),
            ));
        }
        if last_name.len() > 100 {
            return Err(UserError::Validation(
                "Last name cannot exceed 100 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), UserError> {
        if email.trim().is_empty() {
            return Err(UserError::Validation("Email cannot be empty".to_string()));
        }
        if !email.contains('@') {
            return Err(UserError::Validation("Invalid email format".to_string()));
        }
        if email.len() > 255 {
            return Err(UserError::Validation(
                "Email cannot exceed 255 characters".to_string(),
            ));
        }
        Ok(())
    }

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
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
