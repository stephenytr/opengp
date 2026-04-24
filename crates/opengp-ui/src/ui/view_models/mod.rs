//! View Models Module
//!
//! Provides UI-specific data structures that decouple domain entities from UI components.

use chrono::{DateTime, NaiveDate, Utc};
use ratatui::style::Color;
use uuid::Uuid;

use opengp_domain::domain::appointment::{Appointment, AppointmentStatus, AppointmentType};
use opengp_domain::domain::patient::{AtsiStatus, ConcessionType, Gender, Patient};

/// Lightweight representation of a patient used in list views and search widgets.
#[derive(Debug, Clone)]
pub struct PatientListItem {
    pub id: Uuid,
    pub full_name: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub medicare_number: Option<String>,
    pub medicare_irn: Option<u8>,
    pub ihi: Option<String>,
    pub phone_mobile: Option<String>,
}

impl From<Patient> for PatientListItem {
    fn from(patient: Patient) -> Self {
        let full_name = match (&patient.middle_name, &patient.preferred_name) {
            (Some(middle), None) => {
                format!("{} {} {}", patient.first_name, middle, patient.last_name)
            }
            (None, Some(preferred)) => format!("{} ({})", preferred, patient.last_name),
            (Some(middle), Some(preferred)) => {
                format!(
                    "{} {} ({}) {}",
                    patient.first_name, middle, preferred, patient.last_name
                )
            }
            (None, None) => format!("{} {}", patient.first_name, patient.last_name),
        };

        Self {
            id: patient.id,
            full_name,
            date_of_birth: patient.date_of_birth,
            gender: patient.gender,
            medicare_number: patient.medicare_number.map(|m| m.to_string()),
            medicare_irn: patient.medicare_irn,
            ihi: patient.ihi.map(|i| i.to_string()),
            phone_mobile: patient.phone_mobile.map(|p| p.to_string()),
        }
    }
}

/// Data backing the patient form UI.
///
/// This struct flattens the domain `Patient` into fields that match the
/// interactive form widgets.
#[derive(Debug, Clone)]
pub struct PatientFormData {
    // Identification
    pub title: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub preferred_name: Option<String>,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,

    // Healthcare identifiers
    pub ihi: Option<String>,
    pub medicare_number: Option<String>,
    pub medicare_irn: Option<u8>,
    pub medicare_expiry: Option<NaiveDate>,

    // Contact details
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub suburb: Option<String>,
    pub state: Option<String>,
    pub postcode: Option<String>,
    pub country: Option<String>,
    pub phone_home: Option<String>,
    pub phone_mobile: Option<String>,
    pub email: Option<String>,

    // Emergency contact
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,

    // Additional info
    pub concession_type: Option<ConcessionType>,
    pub concession_number: Option<String>,
    pub preferred_language: Option<String>,
    pub interpreter_required: bool,
    pub aboriginal_torres_strait_islander: Option<AtsiStatus>,
}

impl From<Patient> for PatientFormData {
    fn from(patient: Patient) -> Self {
        Self {
            title: patient.title,
            first_name: patient.first_name,
            middle_name: patient.middle_name,
            last_name: patient.last_name,
            preferred_name: patient.preferred_name,
            date_of_birth: patient.date_of_birth,
            gender: patient.gender,
            ihi: patient.ihi.map(|i| i.to_string()),
            medicare_number: patient.medicare_number.map(|m| m.to_string()),
            medicare_irn: patient.medicare_irn,
            medicare_expiry: patient.medicare_expiry,
            address_line1: patient.address.line1,
            address_line2: patient.address.line2,
            suburb: patient.address.suburb,
            state: patient.address.state,
            postcode: patient.address.postcode,
            country: Some(patient.address.country),
            phone_home: patient.phone_home.map(|p| p.to_string()),
            phone_mobile: patient.phone_mobile.map(|p| p.to_string()),
            email: patient.email,
            emergency_contact_name: patient.emergency_contact.as_ref().map(|e| e.name.clone()),
            emergency_contact_phone: patient.emergency_contact.as_ref().map(|e| e.phone.clone()),
            emergency_contact_relationship: patient
                .emergency_contact
                .as_ref()
                .map(|e| e.relationship.clone()),
            concession_type: patient.concession_type,
            concession_number: patient.concession_number,
            preferred_language: Some(patient.preferred_language),
            interpreter_required: patient.interpreter_required,
            aboriginal_torres_strait_islander: patient.aboriginal_torres_strait_islander,
        }
    }
}

impl PatientFormData {
    /// Returns an empty patient form with sensible defaults.
    pub fn empty() -> Self {
        Self {
            title: None,
            first_name: String::new(),
            middle_name: None,
            last_name: String::new(),
            preferred_name: None,
            // SAFETY: 1990-01-01 is a valid date
            #[allow(clippy::unwrap_used)]
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            gender: Gender::PreferNotToSay,
            ihi: None,
            medicare_number: None,
            medicare_irn: None,
            medicare_expiry: None,
            address_line1: None,
            address_line2: None,
            suburb: None,
            state: None,
            postcode: None,
            country: Some("Australia".to_string()),
            phone_home: None,
            phone_mobile: None,
            email: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            emergency_contact_relationship: None,
            concession_type: None,
            concession_number: None,
            preferred_language: Some("English".to_string()),
            interpreter_required: false,
            aboriginal_torres_strait_islander: None,
        }
    }
}

/// View model used for rendering appointments in the calendar and schedule UI.
#[derive(Debug, Clone)]
pub struct AppointmentViewItem {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub practitioner_id: Uuid,
    pub practitioner_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: AppointmentStatus,
    pub appointment_type: AppointmentType,
    pub reason: Option<String>,
    pub duration_minutes: i64,
    pub is_urgent: bool,
    pub confirmed: bool,
    /// Number of 15-minute slots this appointment spans (for rendering)
    pub slot_span: u8,
    /// Internal notes (optional, for modal display)
    pub notes: Option<String>,
}

impl From<Appointment> for AppointmentViewItem {
    fn from(appointment: Appointment) -> Self {
        let duration_minutes = appointment.duration_minutes();
        let slot_span = ((duration_minutes + 14) / 15) as u8; // Round up to nearest 15-min slot
        Self {
            id: appointment.id,
            patient_id: appointment.patient_id,
            patient_name: String::new(),
            practitioner_id: appointment.practitioner_id,
            practitioner_name: String::new(),
            start_time: appointment.start_time,
            end_time: appointment.end_time,
            status: appointment.status,
            appointment_type: appointment.appointment_type,
            duration_minutes,
            is_urgent: appointment.is_urgent,
            confirmed: appointment.confirmed,
            reason: appointment.reason,
            slot_span,
            notes: appointment.notes,
        }
    }
}

use opengp_domain::domain::appointment::CalendarAppointment;

impl From<CalendarAppointment> for AppointmentViewItem {
    fn from(apt: CalendarAppointment) -> Self {
        let duration_minutes = apt.duration_minutes();
        Self {
            id: apt.id,
            patient_id: apt.patient_id,
            patient_name: apt.patient_name,
            practitioner_id: apt.practitioner_id,
            practitioner_name: String::new(),
            start_time: apt.start_time,
            end_time: apt.end_time,
            status: apt.status,
            appointment_type: apt.appointment_type,
            duration_minutes,
            is_urgent: apt.is_urgent,
            confirmed: false,
            reason: apt.reason,
            slot_span: apt.slot_span,
            notes: apt.notes,
        }
    }
}

/// Minimal practitioner details required by the UI.
#[derive(Debug, Clone)]
pub struct PractitionerViewItem {
    pub id: Uuid,
    pub display_name: String,
    pub colour: Color,
}

impl PractitionerViewItem {
    /// Create a new practitioner view item with a color.
    pub fn new(id: Uuid, display_name: String, colour: Color) -> Self {
        Self {
            id,
            display_name,
            colour,
        }
    }

    /// Assign a color to this practitioner from a palette using round-robin by index.
    pub fn with_colour_index(mut self, palette: &[Color], index: usize) -> Self {
        if !palette.is_empty() {
            self.colour = palette[index % palette.len()];
        }
        self
    }
}

impl From<opengp_domain::domain::user::Practitioner> for PractitionerViewItem {
    fn from(practitioner: opengp_domain::domain::user::Practitioner) -> Self {
        Self {
            id: practitioner.id,
            display_name: practitioner.display_name(),
            colour: Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use opengp_domain::domain::patient::{Address, EmergencyContact};

    // ============================================================================
    // From<Patient> for PatientListItem Tests
    // ============================================================================

    fn create_test_patient(
        first_name: &str,
        last_name: &str,
        middle_name: Option<&str>,
        preferred_name: Option<&str>,
    ) -> Patient {
        Patient::new(
            first_name.to_string(),
            last_name.to_string(),
            NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            Gender::Male,
            None,
            None,
            None,
            None,
            None,
            middle_name.map(|s| s.to_string()),
            preferred_name.map(|s| s.to_string()),
            Address::default(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn patient_list_item_full_name_no_middle_no_preferred() {
        let patient = create_test_patient("John", "Smith", None, None);
        let item: PatientListItem = patient.into();
        assert_eq!(item.full_name, "John Smith");
    }

    #[test]
    fn patient_list_item_full_name_with_middle_no_preferred() {
        let patient = create_test_patient("John", "Smith", Some("Michael"), None);
        let item: PatientListItem = patient.into();
        assert_eq!(item.full_name, "John Michael Smith");
    }

    #[test]
    fn patient_list_item_full_name_no_middle_with_preferred() {
        let patient = create_test_patient("John", "Smith", None, Some("Johnny"));
        let item: PatientListItem = patient.into();
        assert_eq!(item.full_name, "Johnny (Smith)");
    }

    #[test]
    fn patient_list_item_full_name_with_middle_and_preferred() {
        let patient = create_test_patient("John", "Smith", Some("Michael"), Some("Johnny"));
        let item: PatientListItem = patient.into();
        assert_eq!(item.full_name, "John Michael (Johnny) Smith");
    }

    // ============================================================================
    // From<Patient> for PatientFormData Tests
    // ============================================================================

    #[test]
    fn patient_form_data_address_fields_mapped_correctly() {
        let address = Address {
            line1: Some("123 Main St".to_string()),
            line2: Some("Apt 4B".to_string()),
            suburb: Some("Sydney".to_string()),
            state: Some("NSW".to_string()),
            postcode: Some("2000".to_string()),
            country: "Australia".to_string(),
        };

        let patient = Patient::new(
            "John".to_string(),
            "Smith".to_string(),
            NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            Gender::Male,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            address,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let form_data: PatientFormData = patient.into();
        assert_eq!(form_data.address_line1, Some("123 Main St".to_string()));
        assert_eq!(form_data.address_line2, Some("Apt 4B".to_string()));
        assert_eq!(form_data.suburb, Some("Sydney".to_string()));
        assert_eq!(form_data.state, Some("NSW".to_string()));
        assert_eq!(form_data.postcode, Some("2000".to_string()));
        assert_eq!(form_data.country, Some("Australia".to_string()));
    }

    #[test]
    fn patient_form_data_emergency_contact_mapped_correctly() {
        let emergency_contact = EmergencyContact {
            name: "Jane Smith".to_string(),
            phone: "0412345678".to_string(),
            relationship: "Spouse".to_string(),
        };

        let patient = Patient::new(
            "John".to_string(),
            "Smith".to_string(),
            NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            Gender::Male,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Address::default(),
            None,
            None,
            None,
            Some(emergency_contact),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let form_data: PatientFormData = patient.into();
        assert_eq!(
            form_data.emergency_contact_name,
            Some("Jane Smith".to_string())
        );
        assert_eq!(
            form_data.emergency_contact_phone,
            Some("0412345678".to_string())
        );
        assert_eq!(
            form_data.emergency_contact_relationship,
            Some("Spouse".to_string())
        );
    }

    #[test]
    fn patient_form_data_empty_form_defaults() {
        let form_data = PatientFormData::empty();
        assert_eq!(form_data.country, Some("Australia".to_string()));
        assert_eq!(form_data.preferred_language, Some("English".to_string()));
        assert_eq!(form_data.first_name, "");
        assert_eq!(form_data.last_name, "");
        assert_eq!(form_data.interpreter_required, false);
        assert_eq!(form_data.emergency_contact_name, None);
        assert_eq!(form_data.emergency_contact_phone, None);
        assert_eq!(form_data.emergency_contact_relationship, None);
    }

    // ============================================================================
    // From<Appointment> for AppointmentViewItem Tests
    // ============================================================================

    fn create_test_appointment(duration_minutes: i64) -> Appointment {
        let start_time = Utc::now();
        let end_time = start_time + Duration::minutes(duration_minutes);
        Appointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            reason: None,
            notes: None,
            is_urgent: false,
            reminder_sent: false,
            confirmed: false,
            cancellation_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            created_by: None,
            updated_by: None,
        }
    }

    #[test]
    fn appointment_view_item_slot_span_15_minutes() {
        let appointment = create_test_appointment(15);
        let view_item: AppointmentViewItem = appointment.into();
        assert_eq!(view_item.slot_span, 1);
    }

    #[test]
    fn appointment_view_item_slot_span_30_minutes() {
        let appointment = create_test_appointment(30);
        let view_item: AppointmentViewItem = appointment.into();
        assert_eq!(view_item.slot_span, 2);
    }

    #[test]
    fn appointment_view_item_slot_span_45_minutes() {
        let appointment = create_test_appointment(45);
        let view_item: AppointmentViewItem = appointment.into();
        assert_eq!(view_item.slot_span, 3);
    }

    #[test]
    fn appointment_view_item_slot_span_20_minutes_rounds_up() {
        let appointment = create_test_appointment(20);
        let view_item: AppointmentViewItem = appointment.into();
        assert_eq!(view_item.slot_span, 2);
    }

    #[test]
    fn appointment_view_item_patient_name_initialized_empty() {
        let appointment = create_test_appointment(30);
        let view_item: AppointmentViewItem = appointment.into();
        assert_eq!(view_item.patient_name, "");
    }
}
