use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Referral {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub referring_practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub referral_type: ReferralType,
    pub specialty: String,
    pub recipient_name: Option<String>,
    pub recipient_address: Option<String>,
    pub recipient_phone: Option<String>,
    pub recipient_fax: Option<String>,
    pub recipient_email: Option<String>,

    pub reason: String,
    pub clinical_notes: Option<String>,
    pub urgency: ReferralUrgency,

    pub referral_date: NaiveDate,
    pub valid_until: Option<NaiveDate>,

    pub status: ReferralStatus,
    pub sent_via: Option<ReferralDeliveryMethod>,
    pub sent_at: Option<DateTime<Utc>>,
    pub appointment_made: bool,
    pub appointment_date: Option<NaiveDate>,
    pub response_received: bool,
    pub response_date: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

impl Referral {
    pub fn new(
        patient_id: Uuid,
        referring_practitioner_id: Uuid,
        referral_type: ReferralType,
        specialty: String,
        reason: String,
        urgency: ReferralUrgency,
        created_by: Uuid,
    ) -> Self {
        let referral_date = Utc::now().date_naive();
        let valid_until = match referral_type {
            ReferralType::Specialist => Some(referral_date + chrono::Duration::days(365)),
            ReferralType::AlliedHealth => Some(referral_date + chrono::Duration::days(365)),
            _ => None,
        };

        Self {
            id: Uuid::new_v4(),
            patient_id,
            referring_practitioner_id,
            consultation_id: None,
            referral_type,
            specialty,
            recipient_name: None,
            recipient_address: None,
            recipient_phone: None,
            recipient_fax: None,
            recipient_email: None,
            reason,
            clinical_notes: None,
            urgency,
            referral_date,
            valid_until,
            status: ReferralStatus::Draft,
            sent_via: None,
            sent_at: None,
            appointment_made: false,
            appointment_date: None,
            response_received: false,
            response_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by,
            updated_by: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.valid_until {
            expiry < Utc::now().date_naive()
        } else {
            false
        }
    }

    pub fn mark_sent(&mut self, method: ReferralDeliveryMethod, user_id: Uuid) {
        self.status = ReferralStatus::Sent;
        self.sent_via = Some(method);
        self.sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferralType {
    Specialist,
    AlliedHealth,
    Hospital,
    EmergencyDepartment,
    MentalHealth,
    Diagnostic,
}

impl std::fmt::Display for ReferralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferralType::Specialist => write!(f, "Specialist"),
            ReferralType::AlliedHealth => write!(f, "Allied Health"),
            ReferralType::Hospital => write!(f, "Hospital"),
            ReferralType::EmergencyDepartment => write!(f, "Emergency Department"),
            ReferralType::MentalHealth => write!(f, "Mental Health"),
            ReferralType::Diagnostic => write!(f, "Diagnostic"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferralUrgency {
    Routine,
    SemiUrgent,
    Urgent,
    Emergency,
}

impl std::fmt::Display for ReferralUrgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferralUrgency::Routine => write!(f, "Routine"),
            ReferralUrgency::SemiUrgent => write!(f, "Semi-Urgent"),
            ReferralUrgency::Urgent => write!(f, "Urgent"),
            ReferralUrgency::Emergency => write!(f, "Emergency"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferralStatus {
    Draft,
    Sent,
    Acknowledged,
    AppointmentBooked,
    Completed,
    Expired,
    Cancelled,
}

impl std::fmt::Display for ReferralStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferralStatus::Draft => write!(f, "Draft"),
            ReferralStatus::Sent => write!(f, "Sent"),
            ReferralStatus::Acknowledged => write!(f, "Acknowledged"),
            ReferralStatus::AppointmentBooked => write!(f, "Appointment Booked"),
            ReferralStatus::Completed => write!(f, "Completed"),
            ReferralStatus::Expired => write!(f, "Expired"),
            ReferralStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferralDeliveryMethod {
    SecureMessaging,
    Fax,
    Email,
    Post,
    HandDelivered,
}

impl std::fmt::Display for ReferralDeliveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferralDeliveryMethod::SecureMessaging => write!(f, "Secure Messaging"),
            ReferralDeliveryMethod::Fax => write!(f, "Fax"),
            ReferralDeliveryMethod::Email => write!(f, "Email"),
            ReferralDeliveryMethod::Post => write!(f, "Post"),
            ReferralDeliveryMethod::HandDelivered => write!(f, "Hand Delivered"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specialist {
    pub id: Uuid,
    pub title: String,
    pub first_name: String,
    pub last_name: String,
    pub specialty: String,
    pub hpi_i: Option<String>,
    pub provider_number: Option<String>,

    pub practice_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub secure_messaging_address: Option<String>,

    pub accepts_new_patients: bool,
    pub preferred_referral_method: Option<ReferralDeliveryMethod>,

    pub notes: Option<String>,

    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Specialist {
    pub fn display_name(&self) -> String {
        format!("{} {} {}", self.title, self.first_name, self.last_name)
    }

    pub fn full_details(&self) -> String {
        format!(
            "{} - {}{}",
            self.display_name(),
            self.specialty,
            self.practice_name
                .as_ref()
                .map(|p| format!(" ({})", p))
                .unwrap_or_default()
        )
    }
}
