//! View Models Module
//!
//! Provides UI-specific data structures that decouple domain entities from UI components.

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::appointment::{Appointment, AppointmentStatus, AppointmentType};
use crate::domain::patient::{AtsiStatus, ConcessionType, Gender, Patient};

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
            medicare_number: patient.medicare_number,
            medicare_irn: patient.medicare_irn,
            ihi: patient.ihi,
            phone_mobile: patient.phone_mobile,
        }
    }
}

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
            ihi: patient.ihi,
            medicare_number: patient.medicare_number,
            medicare_irn: patient.medicare_irn,
            medicare_expiry: patient.medicare_expiry,
            address_line1: patient.address.line1,
            address_line2: patient.address.line2,
            suburb: patient.address.suburb,
            state: patient.address.state,
            postcode: patient.address.postcode,
            country: Some(patient.address.country),
            phone_home: patient.phone_home,
            phone_mobile: patient.phone_mobile,
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
    pub fn empty() -> Self {
        Self {
            title: None,
            first_name: String::new(),
            middle_name: None,
            last_name: String::new(),
            preferred_name: None,
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

use crate::domain::appointment::CalendarAppointment;

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

#[derive(Debug, Clone)]
pub struct PractitionerViewItem {
    pub id: Uuid,
    pub display_name: String,
}

impl From<crate::domain::user::Practitioner> for PractitionerViewItem {
    fn from(practitioner: crate::domain::user::Practitioner) -> Self {
        Self {
            id: practitioner.id,
            display_name: practitioner.display_name(),
        }
    }
}
