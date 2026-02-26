//! Assertion helper functions for comparing domain entities in tests
//!
//! These helpers provide field-by-field comparisons with clear error messages
//! when assertions fail. Each function compares all important fields of the entity.

use chrono::DateTime;
use opengp::domain::appointment::Appointment;
use opengp::domain::audit::{AuditAction, AuditEntry};
use opengp::domain::clinical::Consultation;
use opengp::domain::immunisation::Immunisation;
use opengp::domain::patient::Patient;
use opengp::domain::prescription::Prescription;

/// Assert that two Patient entities are equal
///
/// Compares all important fields including IDs, names, dates, contact info,
/// and status flags. Provides field-by-field error messages on failure.
///
/// # Example
/// ```ignore
/// let patient1 = Patient::new(data)?;
/// let patient2 = Patient::new(data)?;
/// assert_patient_eq(&patient1, &patient2);
/// ```
pub fn assert_patient_eq(actual: &Patient, expected: &Patient) {
    assert_eq!(actual.id, expected.id, "Patient ID mismatch");
    assert_eq!(actual.ihi, expected.ihi, "Patient IHI mismatch");
    assert_eq!(
        actual.medicare_number, expected.medicare_number,
        "Patient Medicare number mismatch"
    );
    assert_eq!(
        actual.medicare_irn, expected.medicare_irn,
        "Patient Medicare IRN mismatch"
    );
    assert_eq!(
        actual.medicare_expiry, expected.medicare_expiry,
        "Patient Medicare expiry mismatch"
    );
    assert_eq!(actual.title, expected.title, "Patient title mismatch");
    assert_eq!(
        actual.first_name, expected.first_name,
        "Patient first name mismatch"
    );
    assert_eq!(
        actual.middle_name, expected.middle_name,
        "Patient middle name mismatch"
    );
    assert_eq!(
        actual.last_name, expected.last_name,
        "Patient last name mismatch"
    );
    assert_eq!(
        actual.preferred_name, expected.preferred_name,
        "Patient preferred name mismatch"
    );
    assert_eq!(
        actual.date_of_birth, expected.date_of_birth,
        "Patient date of birth mismatch"
    );
    assert_eq!(actual.gender, expected.gender, "Patient gender mismatch");
    assert_eq!(
        actual.address.line1, expected.address.line1,
        "Patient address line1 mismatch"
    );
    assert_eq!(
        actual.address.line2, expected.address.line2,
        "Patient address line2 mismatch"
    );
    assert_eq!(
        actual.address.suburb, expected.address.suburb,
        "Patient address suburb mismatch"
    );
    assert_eq!(
        actual.address.state, expected.address.state,
        "Patient address state mismatch"
    );
    assert_eq!(
        actual.address.postcode, expected.address.postcode,
        "Patient address postcode mismatch"
    );
    assert_eq!(
        actual.address.country, expected.address.country,
        "Patient address country mismatch"
    );
    assert_eq!(
        actual.phone_home, expected.phone_home,
        "Patient home phone mismatch"
    );
    assert_eq!(
        actual.phone_mobile, expected.phone_mobile,
        "Patient mobile phone mismatch"
    );
    assert_eq!(actual.email, expected.email, "Patient email mismatch");
    assert_emergency_contact_eq(&actual.emergency_contact, &expected.emergency_contact);
    assert_concession_type_eq(&actual.concession_type, &expected.concession_type);
    assert_eq!(
        actual.concession_number, expected.concession_number,
        "Patient concession number mismatch"
    );
    assert_eq!(
        actual.preferred_language, expected.preferred_language,
        "Patient preferred language mismatch"
    );
    assert_eq!(
        actual.interpreter_required, expected.interpreter_required,
        "Patient interpreter required mismatch"
    );
    assert_atsi_status_eq(
        &actual.aboriginal_torres_strait_islander,
        &expected.aboriginal_torres_strait_islander,
    );
    assert_eq!(
        actual.is_active, expected.is_active,
        "Patient is_active mismatch"
    );
    assert_eq!(
        actual.is_deceased, expected.is_deceased,
        "Patient is_deceased mismatch"
    );
    assert_eq!(
        actual.deceased_date, expected.deceased_date,
        "Patient deceased_date mismatch"
    );
    assert_datetime_eq(
        actual.created_at,
        expected.created_at,
        "Patient created_at mismatch",
    );
    assert_datetime_eq(
        actual.updated_at,
        expected.updated_at,
        "Patient updated_at mismatch",
    );
}

/// Assert that two Appointment entities are equal
///
/// Compares all important fields including IDs, times, status, and audit fields.
/// Uses approximate datetime comparison (same second) for timestamps.
///
/// # Example
/// ```ignore
/// let appt1 = Appointment::new(patient_id, practitioner_id, start, duration, type_, user_id);
/// let appt2 = Appointment::new(patient_id, practitioner_id, start, duration, type_, user_id);
/// assert_appointment_eq(&appt1, &appt2);
/// ```
pub fn assert_appointment_eq(actual: &Appointment, expected: &Appointment) {
    assert_eq!(actual.id, expected.id, "Appointment ID mismatch");
    assert_eq!(
        actual.patient_id, expected.patient_id,
        "Appointment patient_id mismatch"
    );
    assert_eq!(
        actual.practitioner_id, expected.practitioner_id,
        "Appointment practitioner_id mismatch"
    );
    assert_datetime_eq(
        actual.start_time,
        expected.start_time,
        "Appointment start_time mismatch",
    );
    assert_datetime_eq(
        actual.end_time,
        expected.end_time,
        "Appointment end_time mismatch",
    );
    assert_eq!(
        actual.appointment_type, expected.appointment_type,
        "Appointment type mismatch"
    );
    assert_eq!(
        actual.status, expected.status,
        "Appointment status mismatch"
    );
    assert_eq!(
        actual.reason, expected.reason,
        "Appointment reason mismatch"
    );
    assert_eq!(actual.notes, expected.notes, "Appointment notes mismatch");
    assert_eq!(
        actual.is_urgent, expected.is_urgent,
        "Appointment is_urgent mismatch"
    );
    assert_eq!(
        actual.reminder_sent, expected.reminder_sent,
        "Appointment reminder_sent mismatch"
    );
    assert_eq!(
        actual.confirmed, expected.confirmed,
        "Appointment confirmed mismatch"
    );
    assert_eq!(
        actual.cancellation_reason, expected.cancellation_reason,
        "Appointment cancellation_reason mismatch"
    );
    assert_datetime_eq(
        actual.created_at,
        expected.created_at,
        "Appointment created_at mismatch",
    );
    assert_datetime_eq(
        actual.updated_at,
        expected.updated_at,
        "Appointment updated_at mismatch",
    );
    assert_eq!(
        actual.created_by, expected.created_by,
        "Appointment created_by mismatch"
    );
    assert_eq!(
        actual.updated_by, expected.updated_by,
        "Appointment updated_by mismatch"
    );
}

/// Assert that two AuditEntry entities are equal
///
/// Compares all fields including entity info, action details, and timestamps.
/// Uses approximate datetime comparison for changed_at timestamp.
///
/// # Example
/// ```ignore
/// let entry1 = AuditEntry::new_created("patient", patient_id, json_str, user_id);
/// let entry2 = AuditEntry::new_created("patient", patient_id, json_str, user_id);
/// assert_audit_entry_eq(&entry1, &entry2);
/// ```
pub fn assert_audit_entry_eq(actual: &AuditEntry, expected: &AuditEntry) {
    assert_eq!(actual.id, expected.id, "AuditEntry ID mismatch");
    assert_eq!(
        actual.entity_type, expected.entity_type,
        "AuditEntry entity_type mismatch"
    );
    assert_eq!(
        actual.entity_id, expected.entity_id,
        "AuditEntry entity_id mismatch"
    );
    assert_audit_action_eq(&actual.action, &expected.action);
    assert_eq!(
        actual.old_value, expected.old_value,
        "AuditEntry old_value mismatch"
    );
    assert_eq!(
        actual.new_value, expected.new_value,
        "AuditEntry new_value mismatch"
    );
    assert_eq!(
        actual.changed_by, expected.changed_by,
        "AuditEntry changed_by mismatch"
    );
    assert_datetime_eq(
        actual.changed_at,
        expected.changed_at,
        "AuditEntry changed_at mismatch",
    );
}

/// Assert that two AuditAction enums are equal
///
/// Handles all variants including those with payloads (StatusChanged, Rescheduled, Cancelled).
fn assert_audit_action_eq(actual: &AuditAction, expected: &AuditAction) {
    match (actual, expected) {
        (AuditAction::Created, AuditAction::Created) => {}
        (AuditAction::Updated, AuditAction::Updated) => {}
        (
            AuditAction::StatusChanged {
                from: actual_from,
                to: actual_to,
            },
            AuditAction::StatusChanged {
                from: expected_from,
                to: expected_to,
            },
        ) => {
            assert_eq!(actual_from, expected_from, "StatusChanged 'from' mismatch");
            assert_eq!(actual_to, expected_to, "StatusChanged 'to' mismatch");
        }
        (
            AuditAction::Rescheduled {
                from: actual_from,
                to: actual_to,
            },
            AuditAction::Rescheduled {
                from: expected_from,
                to: expected_to,
            },
        ) => {
            assert_datetime_eq(*actual_from, *expected_from, "Rescheduled 'from' mismatch");
            assert_datetime_eq(*actual_to, *expected_to, "Rescheduled 'to' mismatch");
        }
        (
            AuditAction::Cancelled {
                reason: actual_reason,
            },
            AuditAction::Cancelled {
                reason: expected_reason,
            },
        ) => {
            assert_eq!(actual_reason, expected_reason, "Cancelled reason mismatch");
        }
        _ => panic!("AuditAction mismatch: {:?} != {:?}", actual, expected),
    }
}

/// Assert that two Prescription entities are equal
///
/// Compares all fields including medication details, dosage, PBS status, and audit fields.
/// Uses approximate datetime comparison for timestamps.
///
/// # Example
/// ```ignore
/// let rx1 = Prescription::new(patient_id, practitioner_id, None, med, dosage, qty, repeats, directions, user_id);
/// let rx2 = Prescription::new(patient_id, practitioner_id, None, med, dosage, qty, repeats, directions, user_id);
/// assert_prescription_eq(&rx1, &rx2);
/// ```
pub fn assert_prescription_eq(actual: &Prescription, expected: &Prescription) {
    assert_eq!(actual.id, expected.id, "Prescription ID mismatch");
    assert_eq!(
        actual.patient_id, expected.patient_id,
        "Prescription patient_id mismatch"
    );
    assert_eq!(
        actual.practitioner_id, expected.practitioner_id,
        "Prescription practitioner_id mismatch"
    );
    assert_eq!(
        actual.consultation_id, expected.consultation_id,
        "Prescription consultation_id mismatch"
    );
    assert_eq!(
        actual.medication.generic_name, expected.medication.generic_name,
        "Prescription medication generic_name mismatch"
    );
    assert_eq!(
        actual.medication.brand_name, expected.medication.brand_name,
        "Prescription medication brand_name mismatch"
    );
    assert_eq!(
        actual.medication.strength, expected.medication.strength,
        "Prescription medication strength mismatch"
    );
    assert_eq!(
        actual.medication.form, expected.medication.form,
        "Prescription medication form mismatch"
    );
    assert_eq!(
        actual.medication.amt_code, expected.medication.amt_code,
        "Prescription medication amt_code mismatch"
    );
    assert_eq!(
        actual.dosage, expected.dosage,
        "Prescription dosage mismatch"
    );
    assert_eq!(
        actual.quantity, expected.quantity,
        "Prescription quantity mismatch"
    );
    assert_eq!(
        actual.repeats, expected.repeats,
        "Prescription repeats mismatch"
    );
    assert_eq!(
        actual.authority_required, expected.authority_required,
        "Prescription authority_required mismatch"
    );
    assert_eq!(
        actual.authority_approval_number, expected.authority_approval_number,
        "Prescription authority_approval_number mismatch"
    );
    assert_eq!(
        actual.authority_type, expected.authority_type,
        "Prescription authority_type mismatch"
    );
    assert_eq!(
        actual.pbs_status, expected.pbs_status,
        "Prescription pbs_status mismatch"
    );
    assert_eq!(
        actual.pbs_item_code, expected.pbs_item_code,
        "Prescription pbs_item_code mismatch"
    );
    assert_eq!(
        actual.indication, expected.indication,
        "Prescription indication mismatch"
    );
    assert_eq!(
        actual.directions, expected.directions,
        "Prescription directions mismatch"
    );
    assert_eq!(actual.notes, expected.notes, "Prescription notes mismatch");
    assert_eq!(
        actual.prescription_type, expected.prescription_type,
        "Prescription prescription_type mismatch"
    );
    assert_datetime_eq(
        actual.prescription_date,
        expected.prescription_date,
        "Prescription prescription_date mismatch",
    );
    assert_eq!(
        actual.expiry_date, expected.expiry_date,
        "Prescription expiry_date mismatch"
    );
    assert_eq!(
        actual.is_active, expected.is_active,
        "Prescription is_active mismatch"
    );
    assert_eq!(
        actual.cancelled_at, expected.cancelled_at,
        "Prescription cancelled_at mismatch"
    );
    assert_eq!(
        actual.cancellation_reason, expected.cancellation_reason,
        "Prescription cancellation_reason mismatch"
    );
    assert_datetime_eq(
        actual.created_at,
        expected.created_at,
        "Prescription created_at mismatch",
    );
    assert_eq!(
        actual.created_by, expected.created_by,
        "Prescription created_by mismatch"
    );
}

/// Assert that two Immunisation entities are equal
///
/// Compares all fields including vaccine details, administration info, AIR reporting status,
/// and audit fields. Uses approximate datetime comparison for timestamps.
///
/// # Example
/// ```ignore
/// let imm1 = Immunisation::new(patient_id, practitioner_id, vaccine, date, dose, batch, route, site, user_id);
/// let imm2 = Immunisation::new(patient_id, practitioner_id, vaccine, date, dose, batch, route, site, user_id);
/// assert_immunisation_eq(&imm1, &imm2);
/// ```
pub fn assert_immunisation_eq(actual: &Immunisation, expected: &Immunisation) {
    assert_eq!(actual.id, expected.id, "Immunisation ID mismatch");
    assert_eq!(
        actual.patient_id, expected.patient_id,
        "Immunisation patient_id mismatch"
    );
    assert_eq!(
        actual.practitioner_id, expected.practitioner_id,
        "Immunisation practitioner_id mismatch"
    );
    assert_eq!(
        actual.consultation_id, expected.consultation_id,
        "Immunisation consultation_id mismatch"
    );
    assert_eq!(
        actual.vaccine.name, expected.vaccine.name,
        "Immunisation vaccine name mismatch"
    );
    assert_eq!(
        actual.vaccine.vaccine_type, expected.vaccine.vaccine_type,
        "Immunisation vaccine type mismatch"
    );
    assert_eq!(
        actual.vaccine.brand_name, expected.vaccine.brand_name,
        "Immunisation vaccine brand_name mismatch"
    );
    assert_eq!(
        actual.vaccine.snomed_code, expected.vaccine.snomed_code,
        "Immunisation vaccine snomed_code mismatch"
    );
    assert_eq!(
        actual.vaccine.amt_code, expected.vaccine.amt_code,
        "Immunisation vaccine amt_code mismatch"
    );
    assert_eq!(
        actual.vaccination_date, expected.vaccination_date,
        "Immunisation vaccination_date mismatch"
    );
    assert_eq!(
        actual.dose_number, expected.dose_number,
        "Immunisation dose_number mismatch"
    );
    assert_eq!(
        actual.total_doses, expected.total_doses,
        "Immunisation total_doses mismatch"
    );
    assert_eq!(
        actual.batch_number, expected.batch_number,
        "Immunisation batch_number mismatch"
    );
    assert_eq!(
        actual.expiry_date, expected.expiry_date,
        "Immunisation expiry_date mismatch"
    );
    assert_eq!(
        actual.manufacturer, expected.manufacturer,
        "Immunisation manufacturer mismatch"
    );
    assert_eq!(actual.route, expected.route, "Immunisation route mismatch");
    assert_eq!(actual.site, expected.site, "Immunisation site mismatch");
    assert_eq!(
        actual.dose_quantity, expected.dose_quantity,
        "Immunisation dose_quantity mismatch"
    );
    assert_eq!(
        actual.dose_unit, expected.dose_unit,
        "Immunisation dose_unit mismatch"
    );
    assert_eq!(
        actual.consent_obtained, expected.consent_obtained,
        "Immunisation consent_obtained mismatch"
    );
    assert_eq!(
        actual.consent_type, expected.consent_type,
        "Immunisation consent_type mismatch"
    );
    assert_eq!(
        actual.air_reported, expected.air_reported,
        "Immunisation air_reported mismatch"
    );
    assert_eq!(
        actual.air_report_date, expected.air_report_date,
        "Immunisation air_report_date mismatch"
    );
    assert_eq!(
        actual.air_transaction_id, expected.air_transaction_id,
        "Immunisation air_transaction_id mismatch"
    );
    assert_eq!(
        actual.adverse_event, expected.adverse_event,
        "Immunisation adverse_event mismatch"
    );
    assert_eq!(
        actual.adverse_event_details, expected.adverse_event_details,
        "Immunisation adverse_event_details mismatch"
    );
    assert_eq!(actual.notes, expected.notes, "Immunisation notes mismatch");
    assert_datetime_eq(
        actual.created_at,
        expected.created_at,
        "Immunisation created_at mismatch",
    );
    assert_eq!(
        actual.created_by, expected.created_by,
        "Immunisation created_by mismatch"
    );
}

/// Assert that two Consultation entities are equal
///
/// Compares all fields including SOAP notes, signing status, and audit fields.
/// Uses approximate datetime comparison for timestamps.
///
/// # Example
/// ```ignore
/// let cons1 = Consultation::new(patient_id, practitioner_id, None, user_id);
/// let cons2 = Consultation::new(patient_id, practitioner_id, None, user_id);
/// assert_consultation_eq(&cons1, &cons2);
/// ```
pub fn assert_consultation_eq(actual: &Consultation, expected: &Consultation) {
    assert_eq!(actual.id, expected.id, "Consultation ID mismatch");
    assert_eq!(
        actual.patient_id, expected.patient_id,
        "Consultation patient_id mismatch"
    );
    assert_eq!(
        actual.practitioner_id, expected.practitioner_id,
        "Consultation practitioner_id mismatch"
    );
    assert_eq!(
        actual.appointment_id, expected.appointment_id,
        "Consultation appointment_id mismatch"
    );
    assert_datetime_eq(
        actual.consultation_date,
        expected.consultation_date,
        "Consultation consultation_date mismatch",
    );
    assert_eq!(
        actual.clinical_notes, expected.clinical_notes,
        "Consultation clinical_notes mismatch"
    );
    assert_eq!(
        actual.is_signed, expected.is_signed,
        "Consultation is_signed mismatch"
    );
    assert_eq!(
        actual.signed_at, expected.signed_at,
        "Consultation signed_at mismatch"
    );
    assert_eq!(
        actual.signed_by, expected.signed_by,
        "Consultation signed_by mismatch"
    );
    assert_datetime_eq(
        actual.created_at,
        expected.created_at,
        "Consultation created_at mismatch",
    );
    assert_datetime_eq(
        actual.updated_at,
        expected.updated_at,
        "Consultation updated_at mismatch",
    );
    assert_eq!(
        actual.created_by, expected.created_by,
        "Consultation created_by mismatch"
    );
    assert_eq!(
        actual.updated_by, expected.updated_by,
        "Consultation updated_by mismatch"
    );
}

/// Helper function to compare EmergencyContact options
fn assert_emergency_contact_eq(
    actual: &Option<opengp::domain::patient::EmergencyContact>,
    expected: &Option<opengp::domain::patient::EmergencyContact>,
) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => {
            assert_eq!(a.name, e.name, "EmergencyContact name mismatch");
            assert_eq!(a.phone, e.phone, "EmergencyContact phone mismatch");
            assert_eq!(
                a.relationship, e.relationship,
                "EmergencyContact relationship mismatch"
            );
        }
        _ => panic!("EmergencyContact mismatch: {:?} != {:?}", actual, expected),
    }
}

/// Helper function to compare ConcessionType options
fn assert_concession_type_eq(
    actual: &Option<opengp::domain::patient::ConcessionType>,
    expected: &Option<opengp::domain::patient::ConcessionType>,
) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => {
            assert_eq!(a.to_string(), e.to_string(), "ConcessionType mismatch");
        }
        _ => panic!("ConcessionType mismatch: {:?} != {:?}", actual, expected),
    }
}

/// Helper function to compare AtsiStatus options
fn assert_atsi_status_eq(
    actual: &Option<opengp::domain::patient::AtsiStatus>,
    expected: &Option<opengp::domain::patient::AtsiStatus>,
) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => {
            assert_eq!(a.to_string(), e.to_string(), "AtsiStatus mismatch");
        }
        _ => panic!("AtsiStatus mismatch: {:?} != {:?}", actual, expected),
    }
}

/// Helper function to compare DateTime values with approximate equality
///
/// Compares two DateTime<Utc> values to the same second, allowing for
/// minor differences in microseconds that may occur during test execution.
///
/// # Arguments
/// * `actual` - The actual DateTime value
/// * `expected` - The expected DateTime value
/// * `message` - Error message to display on failure
fn assert_datetime_eq(
    actual: DateTime<chrono::Utc>,
    expected: DateTime<chrono::Utc>,
    message: &str,
) {
    let actual_secs = actual.timestamp();
    let expected_secs = expected.timestamp();
    assert_eq!(actual_secs, expected_secs, "{}", message);
}
