use uuid::Uuid;

use opengp_domain::domain::appointment::Appointment;
use opengp_domain::domain::clinical::{
    Allergy, Consultation, FamilyHistory, MedicalHistory, VitalSigns,
};
use opengp_domain::domain::patient::Patient;

use super::appointment_history_generator::{
    AppointmentHistoryGenerator, AppointmentHistoryGeneratorConfig,
};
use super::billing_generator::{BillingData, BillingGenerator, BillingGeneratorConfig};
use super::clinical_generator::{ClinicalDataGenerator, ClinicalDataGeneratorConfig};
use super::patient_generator::{PatientGenerator, PatientGeneratorConfig};

pub struct ComprehensiveFixtureGeneratorConfig {
    pub patient_count: usize,
    pub patient_config: PatientGeneratorConfig,
    pub appointment_history_config: AppointmentHistoryGeneratorConfig,
    pub billing_config: BillingGeneratorConfig,
    pub clinical_config: ClinicalDataGeneratorConfig,
    pub practitioner_ids: Vec<Uuid>,
}

impl Default for ComprehensiveFixtureGeneratorConfig {
    fn default() -> Self {
        Self {
            patient_count: 10,
            patient_config: PatientGeneratorConfig::default(),
            appointment_history_config: AppointmentHistoryGeneratorConfig::default(),
            billing_config: BillingGeneratorConfig::default(),
            clinical_config: ClinicalDataGeneratorConfig::default(),
            practitioner_ids: vec![Uuid::new_v4()],
        }
    }
}

pub struct ComprehensiveFixtureProfile {
    pub patient: Patient,
    pub appointments: Vec<Appointment>,
    pub billing: BillingData,
    pub medical_history: Vec<MedicalHistory>,
    pub allergies: Vec<Allergy>,
    pub consultations: Vec<Consultation>,
    pub vitals: Vec<VitalSigns>,
    pub family_history: Vec<FamilyHistory>,
}

pub struct ComprehensiveFixtureGenerator {
    config: ComprehensiveFixtureGeneratorConfig,
}

impl ComprehensiveFixtureGenerator {
    pub fn new(config: ComprehensiveFixtureGeneratorConfig) -> Self {
        Self { config }
    }

    pub fn generate(&mut self) -> Vec<ComprehensiveFixtureProfile> {
        let mut patient_config = self.config.patient_config.clone();
        patient_config.count = self.config.patient_count;

        let mut patient_generator = PatientGenerator::new(patient_config);
        patient_generator
            .generate()
            .into_iter()
            .map(|patient| self.generate_single(patient))
            .collect()
    }

    pub fn generate_single(&mut self, patient: Patient) -> ComprehensiveFixtureProfile {
        let mut appointment_generator =
            AppointmentHistoryGenerator::new(self.config.appointment_history_config.clone());
        let appointments = appointment_generator
            .generate_for_patient(patient.id, self.config.practitioner_ids.clone());

        let practitioner_id = appointments
            .first()
            .map(|appointment| appointment.practitioner_id)
            .or_else(|| self.config.practitioner_ids.first().copied())
            .unwrap_or_else(Uuid::new_v4);

        let mut clinical_generator =
            ClinicalDataGenerator::new(self.config.clinical_config.clone());
        let medical_history = clinical_generator.generate_medical_history(patient.id);
        let allergies = clinical_generator.generate_allergies(patient.id);
        let consultations = clinical_generator.generate_consultations(patient.id, practitioner_id);
        let vitals = clinical_generator.generate_vitals(patient.id);
        let family_history = clinical_generator.generate_family_history(patient.id);
        let consultation_ids = consultations
            .iter()
            .map(|consultation| consultation.id)
            .collect();

        let mut billing_generator = BillingGenerator::new(self.billing_config());
        let billing =
            billing_generator.generate_for_patient(patient.id, practitioner_id, consultation_ids);

        ComprehensiveFixtureProfile {
            patient,
            appointments,
            billing,
            medical_history,
            allergies,
            consultations,
            vitals,
            family_history,
        }
    }

    fn billing_config(&self) -> BillingGeneratorConfig {
        BillingGeneratorConfig {
            bulk_billing_percentage: self.config.billing_config.bulk_billing_percentage,
            private_billing_percentage: self.config.billing_config.private_billing_percentage,
            dva_percentage: self.config.billing_config.dva_percentage,
            medicare_claim_percentage: self.config.billing_config.medicare_claim_percentage,
            invoice_paid_percentage: self.config.billing_config.invoice_paid_percentage,
            invoice_overdue_percentage: self.config.billing_config.invoice_overdue_percentage,
            average_items_per_invoice: self.config.billing_config.average_items_per_invoice,
            max_items_per_invoice: self.config.billing_config.max_items_per_invoice,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::Utc;

    use super::*;

    fn test_config(patient_count: usize) -> ComprehensiveFixtureGeneratorConfig {
        ComprehensiveFixtureGeneratorConfig {
            patient_count,
            patient_config: PatientGeneratorConfig {
                count: 1,
                ..Default::default()
            },
            appointment_history_config: AppointmentHistoryGeneratorConfig {
                min_appointments_per_patient: 3,
                max_appointments_per_patient: 3,
                ..Default::default()
            },
            clinical_config: ClinicalDataGeneratorConfig {
                consultation_count: 3,
                medical_history_count: 2,
                allergy_count: 1,
                vitals_count: 15,
                family_history_count: 2,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_produces_requested_count() {
        let mut generator = ComprehensiveFixtureGenerator::new(test_config(5));
        let profiles = generator.generate();

        assert_eq!(profiles.len(), 5);
    }

    #[test]
    fn test_all_patients_unique() {
        let mut generator = ComprehensiveFixtureGenerator::new(test_config(10));
        let profiles = generator.generate();

        let mut ids = HashSet::new();
        for profile in &profiles {
            assert!(ids.insert(profile.patient.id));
        }
    }

    #[test]
    fn test_patient_appointment_correlation() {
        let mut generator = ComprehensiveFixtureGenerator::new(test_config(4));
        let profiles = generator.generate();

        for profile in &profiles {
            for appointment in &profile.appointments {
                assert_eq!(appointment.patient_id, profile.patient.id);
            }
        }
    }

    #[test]
    fn test_patient_billing_correlation() {
        let mut generator = ComprehensiveFixtureGenerator::new(test_config(4));
        let profiles = generator.generate();

        for profile in &profiles {
            let consultation_ids: HashSet<Uuid> = profile
                .consultations
                .iter()
                .map(|consultation| consultation.id)
                .collect();

            for invoice in &profile.billing.invoices {
                assert_eq!(invoice.patient_id, profile.patient.id);

                if let Some(consultation_id) = invoice.consultation_id {
                    assert!(consultation_ids.contains(&consultation_id));
                }
            }

            for claim in &profile.billing.medicare_claims {
                assert_eq!(claim.patient_id, profile.patient.id);

                if let Some(consultation_id) = claim.consultation_id {
                    assert!(consultation_ids.contains(&consultation_id));
                }
            }

            for claim in &profile.billing.dva_claims {
                assert_eq!(claim.patient_id, profile.patient.id);

                if let Some(consultation_id) = claim.consultation_id {
                    assert!(consultation_ids.contains(&consultation_id));
                }
            }

            for payment in &profile.billing.payments {
                assert_eq!(payment.patient_id, profile.patient.id);
            }
        }
    }

    #[test]
    fn test_clinical_data_correlation() {
        let mut generator = ComprehensiveFixtureGenerator::new(test_config(6));
        let profiles = generator.generate();

        for profile in &profiles {
            for medical_history in &profile.medical_history {
                assert_eq!(medical_history.patient_id, profile.patient.id);
            }

            for allergy in &profile.allergies {
                assert_eq!(allergy.patient_id, profile.patient.id);
            }

            for consultation in &profile.consultations {
                assert_eq!(consultation.patient_id, profile.patient.id);
            }

            for vitals in &profile.vitals {
                assert_eq!(vitals.patient_id, profile.patient.id);
            }

            for family_history in &profile.family_history {
                assert_eq!(family_history.patient_id, profile.patient.id);
            }
        }
    }

    #[test]
    fn test_appointments_all_in_past() {
        let mut generator = ComprehensiveFixtureGenerator::new(test_config(5));
        let profiles = generator.generate();
        let now = Utc::now();

        for profile in &profiles {
            for appointment in &profile.appointments {
                assert!(appointment.start_time < now);
            }
        }
    }
}
