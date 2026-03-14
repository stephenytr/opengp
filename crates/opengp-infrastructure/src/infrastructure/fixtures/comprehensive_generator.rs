//! Comprehensive patient data generator
//!
//! Generates complete patient profiles including demographics, clinical history,
//! allergies, medical conditions, and consultation records in a single unified interface.

use uuid::Uuid;

use opengp_domain::domain::clinical::{Allergy, Consultation, MedicalHistory};
use opengp_domain::domain::patient::Patient;

use super::clinical_generator::{ClinicalDataGenerator, ClinicalDataGeneratorConfig};
use super::patient_generator::{PatientGenerator, PatientGeneratorConfig};

/// Complete patient profile with all clinical and demographic data
#[derive(Debug, Clone)]
pub struct ComprehensivePatientProfile {
    /// Patient demographics and contact information
    pub patient: Patient,
    /// Medical history entries (conditions, diagnoses)
    pub medical_history: Vec<MedicalHistory>,
    /// Allergies and adverse reactions
    pub allergies: Vec<Allergy>,
    /// Consultation records and clinical notes
    pub consultations: Vec<Consultation>,
}

/// Configuration for comprehensive patient generation
#[derive(Debug, Clone)]
pub struct ComprehensivePatientGeneratorConfig {
    /// Patient demographics configuration
    pub patient_config: PatientGeneratorConfig,
    /// Clinical data configuration
    pub clinical_config: ClinicalDataGeneratorConfig,
    /// Number of patients to generate
    pub patient_count: usize,
    /// Practitioner IDs to use for consultations (if empty, generates random)
    pub practitioner_ids: Vec<Uuid>,
}

impl Default for ComprehensivePatientGeneratorConfig {
    fn default() -> Self {
        Self {
            patient_config: PatientGeneratorConfig {
                count: 1,
                ..Default::default()
            },
            clinical_config: ClinicalDataGeneratorConfig::default(),
            patient_count: 10,
            practitioner_ids: Vec::new(),
        }
    }
}

/// Unified generator for complete patient profiles
///
/// Generates realistic patient data including:
/// - Demographics (name, DOB, contact info, address)
/// - Medical history (conditions, diagnoses)
/// - Allergies and adverse reactions
/// - Consultation records with clinical notes
///
/// # Example
///
/// ```
/// use opengp_infrastructure::infrastructure::fixtures::{
///     ComprehensivePatientGenerator, ComprehensivePatientGeneratorConfig
/// };
///
/// let config = ComprehensivePatientGeneratorConfig {
///     patient_count: 50,
///     ..Default::default()
/// };
///
/// let mut generator = ComprehensivePatientGenerator::new(config);
/// let profiles = generator.generate();
///
/// for profile in profiles {
///     println!("Patient: {} {}", profile.patient.first_name, profile.patient.last_name);
///     println!("  Medical conditions: {}", profile.medical_history.len());
///     println!("  Allergies: {}", profile.allergies.len());
///     println!("  Consultations: {}", profile.consultations.len());
/// }
/// ```
pub struct ComprehensivePatientGenerator {
    config: ComprehensivePatientGeneratorConfig,
}

impl ComprehensivePatientGenerator {
    /// Create a new comprehensive patient generator
    pub fn new(config: ComprehensivePatientGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate complete patient profiles
    pub fn generate(&self) -> Vec<ComprehensivePatientProfile> {
        let mut patient_gen = PatientGenerator::new(PatientGeneratorConfig {
            count: self.config.patient_count,
            ..self.config.patient_config.clone()
        });

        let patients = patient_gen.generate();

        patients
            .into_iter()
            .map(|patient| self.generate_profile(patient))
            .collect()
    }

    /// Generate a single comprehensive patient profile
    fn generate_profile(&self, patient: Patient) -> ComprehensivePatientProfile {
        let mut clinical_gen = ClinicalDataGenerator::new(self.config.clinical_config.clone());

        // Generate clinical data
        let practitioner_id = if self.config.practitioner_ids.is_empty() {
            Uuid::new_v4()
        } else {
            self.config.practitioner_ids[0]
        };

        let medical_history = clinical_gen.generate_medical_history(patient.id);
        let allergies = clinical_gen.generate_allergies(patient.id);
        let consultations = clinical_gen.generate_consultations(patient.id, practitioner_id);

        ComprehensivePatientProfile {
            patient,
            medical_history,
            allergies,
            consultations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_comprehensive_profiles() {
        let config = ComprehensivePatientGeneratorConfig {
            patient_count: 5,
            ..Default::default()
        };

        let generator = ComprehensivePatientGenerator::new(config);
        let profiles = generator.generate();

        assert_eq!(profiles.len(), 5);

        for profile in &profiles {
            assert!(!profile.patient.first_name.is_empty());
            assert!(!profile.patient.last_name.is_empty());
            assert!(profile.patient.is_active);
            assert!(!profile.patient.is_deceased);
        }
    }

    #[test]
    fn test_comprehensive_profile_has_clinical_data() {
        let config = ComprehensivePatientGeneratorConfig {
            patient_count: 1,
            clinical_config: ClinicalDataGeneratorConfig {
                consultation_count: 3,
                medical_history_count: 2,
                allergy_count: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let generator = ComprehensivePatientGenerator::new(config);
        let profiles = generator.generate();

        assert_eq!(profiles.len(), 1);
        let profile = &profiles[0];

        assert_eq!(profile.consultations.len(), 3);
        assert_eq!(profile.medical_history.len(), 2);
        assert_eq!(profile.allergies.len(), 1);
    }

    #[test]
    fn test_comprehensive_profile_with_practitioner_pool() {
        let practitioner_ids = vec![
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
        ];

        let config = ComprehensivePatientGeneratorConfig {
            patient_count: 3,
            practitioner_ids: practitioner_ids.clone(),
            ..Default::default()
        };

        let generator = ComprehensivePatientGenerator::new(config);
        let profiles = generator.generate();

        for profile in &profiles {
            for consultation in &profile.consultations {
                assert!(practitioner_ids.contains(&consultation.practitioner_id));
            }
        }
    }

    #[test]
    fn test_comprehensive_profile_consistency() {
        let config = ComprehensivePatientGeneratorConfig {
            patient_count: 10,
            ..Default::default()
        };

        let generator = ComprehensivePatientGenerator::new(config);
        let profiles = generator.generate();

        for profile in &profiles {
            // All clinical data should reference the same patient
            for history in &profile.medical_history {
                assert_eq!(history.patient_id, profile.patient.id);
            }

            for allergy in &profile.allergies {
                assert_eq!(allergy.patient_id, profile.patient.id);
            }

            for consultation in &profile.consultations {
                assert_eq!(consultation.patient_id, profile.patient.id);
            }
        }
    }

    #[test]
    fn test_comprehensive_profile_with_custom_config() {
        let patient_config = PatientGeneratorConfig {
            count: 1,
            min_age: 65,
            max_age: 85,
            include_children: false,
            include_seniors: true,
            medicare_percentage: 1.0,
            ihi_percentage: 0.95,
            mobile_percentage: 0.70,
            email_percentage: 0.60,
            emergency_contact_percentage: 0.80,
            concession_percentage: 0.30,
            atsi_percentage: 0.05,
            interpreter_percentage: 0.05,
            preferred_name_percentage: 0.15,
            middle_name_percentage: 0.60,
            use_australian_names: true,
        };

        let clinical_config = ClinicalDataGeneratorConfig {
            consultation_count: 10,
            medical_history_count: 5,
            allergy_count: 2,
            notes_percentage: 0.90,
            signed_percentage: 0.85,
            severe_allergy_percentage: 0.20,
        };

        let config = ComprehensivePatientGeneratorConfig {
            patient_count: 1,
            patient_config,
            clinical_config,
            practitioner_ids: Vec::new(),
        };

        let generator = ComprehensivePatientGenerator::new(config);
        let profiles = generator.generate();

        assert_eq!(profiles.len(), 1);
        let profile = &profiles[0];

        // Verify senior patient
        let age = chrono::Utc::now()
            .date_naive()
            .years_since(profile.patient.date_of_birth)
            .unwrap_or(0);
        assert!(age >= 65 && age <= 85);

        // Verify clinical data counts
        assert_eq!(profile.consultations.len(), 10);
        assert_eq!(profile.medical_history.len(), 5);
        assert_eq!(profile.allergies.len(), 2);
    }

    #[test]
    fn test_comprehensive_profile_bulk_generation() {
        let config = ComprehensivePatientGeneratorConfig {
            patient_count: 100,
            ..Default::default()
        };

        let generator = ComprehensivePatientGenerator::new(config);
        let profiles = generator.generate();

        assert_eq!(profiles.len(), 100);

        // Verify all profiles are unique
        let mut ids = std::collections::HashSet::new();
        for profile in &profiles {
            assert!(ids.insert(profile.patient.id));
        }
    }
}
