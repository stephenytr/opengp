use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use opengp_domain::domain::appointment::{AppointmentStatus, NewAppointmentData};
use crate::ui::app::{PendingClinicalSaveData, PendingBillingSaveData};
use crate::ui::components::SubtabKind;

#[derive(Debug)]
pub enum AppCommand {
    RefreshAppointments(NaiveDate),
    CreateAppointment(NewAppointmentData),
    UpdateAppointment {
        id: Uuid,
        data: NewAppointmentData,
        version: i32,
    },
    AppointmentSaveResult(Result<(), String>),
    UpdateAppointmentStatus {
        id: Uuid,
        status: AppointmentStatus,
    },
    LoadPractitioners,
    LoadAvailableSlots {
        practitioner_id: Uuid,
        date: NaiveDate,
        duration_minutes: u32,
    },
    CancelAppointment {
        id: Uuid,
        reason: String,
    },
    RescheduleAppointment {
        id: Uuid,
        new_start_time: DateTime<Utc>,
        new_duration_minutes: i64,
        user_id: Uuid,
    },
    SaveClinicalData {
        patient_id: Uuid,
        data: PendingClinicalSaveData,
    },
    SaveBillingData {
        patient_id: Uuid,
        data: PendingBillingSaveData,
    },
    LoadPatientWorkspaceData {
        patient_id: Uuid,
        subtab: SubtabKind,
    },
    LoadBillingData {
        patient_id: Uuid,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_command_variants_exist() {
        let patient_id = Uuid::new_v4();

        let _clinical_cmd = AppCommand::SaveClinicalData {
            patient_id,
            data: PendingClinicalSaveData::Consultation {
                patient_id,
                practitioner_id: Uuid::new_v4(),
                appointment_id: None,
                reason: Some("Test".to_string()),
                clinical_notes: None,
            },
        };

        let _billing_cmd = AppCommand::SaveBillingData {
            patient_id,
            data: PendingBillingSaveData::AwaitingMbsSelection {
                consultation_id: Uuid::new_v4(),
                patient_id,
            },
        };

        let _workspace_cmd = AppCommand::LoadPatientWorkspaceData {
            patient_id,
            subtab: SubtabKind::Clinical,
        };
    }

    #[test]
    fn test_subtab_kind_variants() {
        let subtabs = vec![
            SubtabKind::Summary,
            SubtabKind::Demographics,
            SubtabKind::Clinical,
            SubtabKind::Billing,
            SubtabKind::Appointments,
        ];

        for subtab in subtabs {
            assert!(!subtab.display_name().is_empty());
        }

        #[cfg(feature = "pathology")]
        assert!(!SubtabKind::Pathology.display_name().is_empty());
        #[cfg(feature = "prescription")]
        assert!(!SubtabKind::Prescription.display_name().is_empty());
        #[cfg(feature = "referral")]
        assert!(!SubtabKind::Referral.display_name().is_empty());
        #[cfg(feature = "immunisation")]
        assert!(!SubtabKind::Immunisation.display_name().is_empty());
    }
}
