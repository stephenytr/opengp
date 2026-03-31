use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Outgoing referral from the general practice to another provider.
///
/// Links a patient and referring practitioner to the target
/// specialist or service, including reason for referral, urgency and
/// delivery metadata (eg secure messaging, fax).
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
    /// Create a new referral in draft form.
    ///
    /// For specialist and allied health referrals the validity period
    /// is initialised according to common Australian rules
    /// (typically 12 months).
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

    /// Return true when the referral has passed its validity period.
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.valid_until {
            expiry < Utc::now().date_naive()
        } else {
            false
        }
    }

    /// Mark the referral as sent via the specified delivery method.
    pub fn mark_sent(&mut self, method: ReferralDeliveryMethod, user_id: Uuid) {
        self.status = ReferralStatus::Sent;
        self.sent_via = Some(method);
        self.sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }
}

/// Category of referral destination.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ReferralType {
    /// Referral to a medical specialist.
    Specialist,
    /// Referral to an allied health provider.
    AlliedHealth,
    /// Referral to a hospital outpatient or inpatient service.
    Hospital,
    /// Referral to the emergency department.
    EmergencyDepartment,
    /// Referral focused on mental health services.
    MentalHealth,
    /// Referral for diagnostic imaging or pathology.
    Diagnostic,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Display, EnumString,
)]
pub enum ReferralUrgency {
    /// Routine referral.
    Routine,
    /// Referral that should be prioritised but is not an emergency.
    SemiUrgent,
    /// Requires urgent review.
    Urgent,
    /// Immediate emergency department transfer.
    Emergency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ReferralStatus {
    /// Draft referral that has not yet been sent.
    Draft,
    /// Referral has been dispatched to the recipient.
    Sent,
    /// Recipient has acknowledged receipt.
    Acknowledged,
    /// Appointment has been booked with the recipient service.
    AppointmentBooked,
    /// Episode of care is complete.
    Completed,
    /// Referral has passed its validity period.
    Expired,
    /// Referral has been cancelled.
    Cancelled,
}

/// Delivery channel used to send the referral.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ReferralDeliveryMethod {
    /// Sent via secure messaging provider.
    SecureMessaging,
    /// Sent by fax.
    Fax,
    /// Sent by email.
    Email,
    /// Sent via postal mail.
    Post,
    /// Hand delivered or given to the patient.
    HandDelivered,
}

/// Directory entry for a specialist or external clinician.
///
/// Includes HPI‑I and provider number details where available plus
/// preferred referral method.
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
    /// Construct a human‑friendly display name for the specialist.
    pub fn display_name(&self) -> String {
        format!("{} {} {}", self.title, self.first_name, self.last_name)
    }

    /// Full details string including specialty and practice name.
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
