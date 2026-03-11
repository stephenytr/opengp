use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::dto::NewUserData;
use super::error::ServiceError;

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
    pub fn new(data: NewUserData) -> Result<Self, ServiceError> {
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

    fn validate_username(username: &str) -> Result<(), ServiceError> {
        if username.trim().is_empty() {
            return Err(ServiceError::Validation(
                "Username cannot be empty".to_string(),
            ));
        }
        if username.len() < 3 {
            return Err(ServiceError::Validation(
                "Username must be at least 3 characters".to_string(),
            ));
        }
        if username.len() > 50 {
            return Err(ServiceError::Validation(
                "Username cannot exceed 50 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_names(first_name: &str, last_name: &str) -> Result<(), ServiceError> {
        if first_name.trim().is_empty() {
            return Err(ServiceError::Validation(
                "First name cannot be empty".to_string(),
            ));
        }
        if last_name.trim().is_empty() {
            return Err(ServiceError::Validation(
                "Last name cannot be empty".to_string(),
            ));
        }
        if first_name.len() > 100 {
            return Err(ServiceError::Validation(
                "First name cannot exceed 100 characters".to_string(),
            ));
        }
        if last_name.len() > 100 {
            return Err(ServiceError::Validation(
                "Last name cannot exceed 100 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), ServiceError> {
        if email.trim().is_empty() {
            return Err(ServiceError::Validation(
                "Email cannot be empty".to_string(),
            ));
        }
        if !email.contains('@') {
            return Err(ServiceError::Validation("Invalid email format".to_string()));
        }
        if email.len() > 255 {
            return Err(ServiceError::Validation(
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

/// Working hours for a practitioner on a specific day of the week
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    pub id: Uuid,
    pub practitioner_id: Uuid,

    /// Day of week (0 = Monday, 6 = Sunday)
    pub day_of_week: u8,

    /// Start time of working hours
    pub start_time: NaiveTime,

    /// End time of working hours
    pub end_time: NaiveTime,

    /// Is this working hours entry active?
    pub is_active: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkingHours {
    /// Create a new working hours entry with validation
    ///
    /// # Arguments
    /// * `practitioner_id` - UUID of the practitioner
    /// * `day_of_week` - Day of week (0-6, Monday-Sunday)
    /// * `start_time` - Start time of working hours
    /// * `end_time` - End time of working hours
    ///
    /// # Returns
    /// * `Ok(WorkingHours)` - Valid working hours entry
    /// * `Err(ServiceError::Validation)` - If validation fails
    pub fn new(
        practitioner_id: Uuid,
        day_of_week: u8,
        start_time: NaiveTime,
        end_time: NaiveTime,
    ) -> Result<Self, ServiceError> {
        Self::validate_day_of_week(day_of_week)?;
        Self::validate_times(start_time, end_time)?;

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            practitioner_id,
            day_of_week,
            start_time,
            end_time,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Validate day of week (0-6)
    fn validate_day_of_week(day_of_week: u8) -> Result<(), ServiceError> {
        if day_of_week > 6 {
            return Err(ServiceError::Validation(
                "Day of week must be between 0 (Monday) and 6 (Sunday)".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate start and end times
    ///
    /// Ensures:
    /// - end_time > start_time
    /// - both times are within reasonable bounds (5:00 AM - 11:00 PM)
    fn validate_times(start_time: NaiveTime, end_time: NaiveTime) -> Result<(), ServiceError> {
        if end_time <= start_time {
            return Err(ServiceError::Validation(
                "End time must be after start time".to_string(),
            ));
        }

        let min_hour = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        let max_hour = NaiveTime::from_hms_opt(23, 0, 0).unwrap();

        if start_time < min_hour {
            return Err(ServiceError::Validation(
                "Start time must be at or after 5:00 AM".to_string(),
            ));
        }

        if end_time > max_hour {
            return Err(ServiceError::Validation(
                "End time must be at or before 11:00 PM".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the day of week as a string
    pub fn day_of_week_str(&self) -> &'static str {
        match self.day_of_week {
            0 => "Monday",
            1 => "Tuesday",
            2 => "Wednesday",
            3 => "Thursday",
            4 => "Friday",
            5 => "Saturday",
            6 => "Sunday",
            _ => "Invalid",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_hours_valid_creation() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, start_time, end_time);
        assert!(result.is_ok());

        let wh = result.unwrap();
        assert_eq!(wh.practitioner_id, practitioner_id);
        assert_eq!(wh.day_of_week, 0);
        assert_eq!(wh.start_time, start_time);
        assert_eq!(wh.end_time, end_time);
        assert!(wh.is_active);
    }

    #[test]
    fn test_working_hours_end_before_start_fails() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, start_time, end_time);
        assert!(result.is_err());

        match result {
            Err(ServiceError::Validation(msg)) => {
                assert!(msg.contains("End time must be after start time"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_working_hours_same_time_fails() {
        let practitioner_id = Uuid::new_v4();
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, time, time);
        assert!(result.is_err());

        match result {
            Err(ServiceError::Validation(msg)) => {
                assert!(msg.contains("End time must be after start time"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_working_hours_start_before_5am_fails() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(4, 59, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, start_time, end_time);
        assert!(result.is_err());

        match result {
            Err(ServiceError::Validation(msg)) => {
                assert!(msg.contains("Start time must be at or after 5:00 AM"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_working_hours_end_after_11pm_fails() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(23, 1, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, start_time, end_time);
        assert!(result.is_err());

        match result {
            Err(ServiceError::Validation(msg)) => {
                assert!(msg.contains("End time must be at or before 11:00 PM"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_working_hours_end_at_11pm_succeeds() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, start_time, end_time);
        assert!(result.is_ok());
    }

    #[test]
    fn test_working_hours_start_at_5am_succeeds() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 0, start_time, end_time);
        assert!(result.is_ok());
    }

    #[test]
    fn test_working_hours_invalid_day_of_week_fails() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

        let result = WorkingHours::new(practitioner_id, 7, start_time, end_time);
        assert!(result.is_err());

        match result {
            Err(ServiceError::Validation(msg)) => {
                assert!(msg.contains("Day of week must be between 0 (Monday) and 6 (Sunday)"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_working_hours_all_days_of_week_valid() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

        for day in 0..=6 {
            let result = WorkingHours::new(practitioner_id, day, start_time, end_time);
            assert!(result.is_ok(), "Day {} should be valid", day);
        }
    }

    #[test]
    fn test_working_hours_day_of_week_str() {
        let practitioner_id = Uuid::new_v4();
        let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

        let wh = WorkingHours::new(practitioner_id, 0, start_time, end_time).unwrap();
        assert_eq!(wh.day_of_week_str(), "Monday");

        let wh = WorkingHours::new(practitioner_id, 4, start_time, end_time).unwrap();
        assert_eq!(wh.day_of_week_str(), "Friday");

        let wh = WorkingHours::new(practitioner_id, 6, start_time, end_time).unwrap();
        assert_eq!(wh.day_of_week_str(), "Sunday");
    }
}
