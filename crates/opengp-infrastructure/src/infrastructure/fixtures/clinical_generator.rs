use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use opengp_domain::domain::clinical::{
    Allergy, AllergyType, ConditionStatus, Consultation, FamilyHistory, MedicalHistory, Severity,
    VitalSigns,
};

/// Configuration for generating synthetic clinical data
#[derive(Debug, Clone)]
pub struct ClinicalDataGeneratorConfig {
    /// Number of consultations to generate
    pub consultation_count: usize,
    /// Number of medical history entries to generate
    pub medical_history_count: usize,
    /// Number of allergies to generate
    pub allergy_count: usize,
    /// Number of vital signs records to generate (10-20 per patient)
    pub vitals_count: usize,
    /// Number of family history entries to generate
    pub family_history_count: usize,
    /// Percentage of consultations with clinical notes (0.0-1.0)
    pub notes_percentage: f32,
    /// Percentage of consultations that are signed (0.0-1.0)
    pub signed_percentage: f32,
    /// Percentage of allergies with severe reactions (0.0-1.0)
    pub severe_allergy_percentage: f32,
}

impl Default for ClinicalDataGeneratorConfig {
    fn default() -> Self {
        Self {
            consultation_count: 5,
            medical_history_count: 3,
            allergy_count: 1,
            vitals_count: 15,
            family_history_count: 2,
            notes_percentage: 0.80,
            signed_percentage: 0.70,
            severe_allergy_percentage: 0.10,
        }
    }
}

/// Generator for realistic clinical test data
pub struct ClinicalDataGenerator {
    config: ClinicalDataGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl ClinicalDataGenerator {
    /// Create a new clinical data generator with the given configuration
    pub fn new(config: ClinicalDataGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a collection of consultations for a patient
    pub fn generate_consultations(
        &mut self,
        patient_id: Uuid,
        practitioner_id: Uuid,
    ) -> Vec<Consultation> {
        (0..self.config.consultation_count)
            .map(|_| self.generate_consultation(patient_id, practitioner_id))
            .collect()
    }

    /// Generate a collection of medical history entries for a patient
    pub fn generate_medical_history(&mut self, patient_id: Uuid) -> Vec<MedicalHistory> {
        (0..self.config.medical_history_count)
            .map(|_| self.generate_medical_history_entry(patient_id))
            .collect()
    }

    /// Generate a collection of allergies for a patient
    pub fn generate_allergies(&mut self, patient_id: Uuid) -> Vec<Allergy> {
        (0..self.config.allergy_count)
            .map(|_| self.generate_allergy(patient_id))
            .collect()
    }

    /// Generate a collection of vital signs records for a patient
    pub fn generate_vitals(&mut self, patient_id: Uuid) -> Vec<VitalSigns> {
        (0..self.config.vitals_count)
            .map(|_| self.generate_vital_signs(patient_id))
            .collect()
    }

    /// Generate a collection of family history entries for a patient
    pub fn generate_family_history(&mut self, patient_id: Uuid) -> Vec<FamilyHistory> {
        (0..self.config.family_history_count)
            .map(|_| self.generate_family_history_entry(patient_id))
            .collect()
    }

    fn generate_vital_signs(&mut self, patient_id: Uuid) -> VitalSigns {
        let created_by = Uuid::new_v4();
        let measured_at = Utc::now() - Duration::days(self.rng.gen_range(0..=365));

        let height_cm = Some(self.random_height_cm());
        let weight_kg = Some(self.random_weight_kg());

        let mut vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id,
            consultation_id: None,
            measured_at,
            systolic_bp: Some(self.random_systolic_bp()),
            diastolic_bp: Some(self.random_diastolic_bp()),
            heart_rate: Some(self.random_heart_rate()),
            respiratory_rate: Some(self.random_respiratory_rate()),
            temperature: Some(self.random_temperature()),
            oxygen_saturation: Some(self.random_oxygen_saturation()),
            height_cm,
            weight_kg,
            bmi: None,
            notes: None,
            created_at: Utc::now(),
            created_by,
        };

        vitals.calculate_bmi();
        vitals
    }

    fn generate_family_history_entry(&mut self, patient_id: Uuid) -> FamilyHistory {
        let created_by = Uuid::new_v4();

        FamilyHistory {
            id: Uuid::new_v4(),
            patient_id,
            relative_relationship: self.random_relative_relationship(),
            condition: self.random_family_condition(),
            age_at_diagnosis: Some(self.random_age_at_diagnosis()),
            notes: if self.rng.gen_bool(0.30) {
                Some(self.random_family_history_notes())
            } else {
                None
            },
            created_at: Utc::now() - Duration::days(self.rng.gen_range(30..=3650)),
            created_by,
        }
    }

    fn random_height_cm(&mut self) -> u16 {
        self.rng.gen_range(150..=195)
    }

    fn random_weight_kg(&mut self) -> f32 {
        let base = match self.rng.gen_range(0..4) {
            0 => 55.0,
            1 => 70.0,
            2 => 85.0,
            _ => 100.0,
        };
        base + self.rng.gen_range(-10.0..=10.0)
    }

    fn random_systolic_bp(&mut self) -> u16 {
        self.rng.gen_range(100..=160)
    }

    fn random_diastolic_bp(&mut self) -> u16 {
        self.rng.gen_range(60..=100)
    }

    fn random_heart_rate(&mut self) -> u16 {
        self.rng.gen_range(55..=100)
    }

    fn random_respiratory_rate(&mut self) -> u16 {
        self.rng.gen_range(12..=22)
    }

    fn random_temperature(&mut self) -> f32 {
        let base = 37.0;
        base + self.rng.gen_range(-0.5..=1.0)
    }

    fn random_oxygen_saturation(&mut self) -> u8 {
        self.rng.gen_range(94..=100)
    }

    fn random_relative_relationship(&mut self) -> String {
        let relationships = [
            "Father",
            "Mother",
            "Brother",
            "Sister",
            "Paternal Grandfather",
            "Maternal Grandfather",
            "Paternal Grandmother",
            "Maternal Grandmother",
            "Son",
            "Daughter",
            "Uncle",
            "Aunt",
            "Cousin",
        ];
        relationships
            .choose(&mut self.rng)
            .unwrap_or(&"Father")
            .to_string()
    }

    fn random_family_condition(&mut self) -> String {
        let conditions = [
            "Heart Disease",
            "Type 2 Diabetes",
            "Hypertension",
            "Stroke",
            "Breast Cancer",
            "Colorectal Cancer",
            "Prostate Cancer",
            "Melanoma",
            "Asthma",
            "COPD",
            "Arthritis",
            "Osteoporosis",
            "Alzheimer's Disease",
            "Parkinson's Disease",
            "Depression",
            "Anxiety",
            "Epilepsy",
            "Kidney Disease",
            "Liver Disease",
            "Thyroid Disorder",
        ];
        conditions
            .choose(&mut self.rng)
            .unwrap_or(&"Heart Disease")
            .to_string()
    }

    fn random_age_at_diagnosis(&mut self) -> u8 {
        self.rng.gen_range(30..=80)
    }

    fn random_family_history_notes(&mut self) -> String {
        let notes = [
            "Diagnosed in his 50s",
            "Multiple family members affected",
            "Early onset, before age 50",
            "Late onset, after age 70",
            "Managed with medication",
            "Required surgery",
            "Passed away from complications",
            "Currently under treatment",
        ];
        notes
            .choose(&mut self.rng)
            .unwrap_or(&"Diagnosed in his 50s")
            .to_string()
    }

    fn generate_consultation(&mut self, patient_id: Uuid, practitioner_id: Uuid) -> Consultation {
        let created_by = Uuid::new_v4();
        let mut consultation = Consultation::new(patient_id, practitioner_id, None, created_by);

        consultation.reason = Some(self.random_consultation_reason());

        if self.rng.gen_bool(self.config.notes_percentage as f64) {
            consultation.clinical_notes = Some(self.generate_clinical_notes());
        }

        if self.rng.gen_bool(self.config.signed_percentage as f64) {
            consultation.sign(Uuid::new_v4());
        }

        consultation.consultation_date = Utc::now() - Duration::days(self.rng.gen_range(0..365));
        consultation
    }

    fn generate_medical_history_entry(&mut self, patient_id: Uuid) -> MedicalHistory {
        let created_by = Uuid::new_v4();
        let condition = self.random_condition();
        let diagnosis_date =
            Some((Utc::now() - Duration::days(self.rng.gen_range(30..=3650))).date_naive());

        MedicalHistory {
            id: Uuid::new_v4(),
            patient_id,
            condition,
            diagnosis_date,
            status: self.random_condition_status(),
            severity: Some(self.random_severity()),
            notes: if self.rng.gen_bool(0.40) {
                Some(self.random_condition_notes())
            } else {
                None
            },
            is_active: true,
            created_at: Utc::now() - Duration::days(self.rng.gen_range(30..=3650)),
            updated_at: Utc::now(),
            created_by,
            updated_by: None,
        }
    }

    fn generate_allergy(&mut self, patient_id: Uuid) -> Allergy {
        let created_by = Uuid::new_v4();
        let is_severe = self
            .rng
            .gen_bool(self.config.severe_allergy_percentage as f64);

        Allergy {
            id: Uuid::new_v4(),
            patient_id,
            allergen: self.random_allergen(),
            allergy_type: self.random_allergy_type(),
            severity: if is_severe {
                Severity::Severe
            } else {
                self.random_severity()
            },
            reaction: Some(self.random_reaction()),
            onset_date: Some(
                (Utc::now() - Duration::days(self.rng.gen_range(30..=10950))).date_naive(),
            ),
            notes: if self.rng.gen_bool(0.30) {
                Some(self.random_allergy_notes())
            } else {
                None
            },
            is_active: true,
            created_at: Utc::now() - Duration::days(self.rng.gen_range(30..=10950)),
            updated_at: Utc::now(),
            created_by,
            updated_by: None,
        }
    }

    fn random_consultation_reason(&mut self) -> String {
        let reasons = [
            "General check-up",
            "Chronic disease management",
            "Acute illness",
            "Preventive care",
            "Health assessment",
            "Medication review",
            "Follow-up consultation",
            "Mental health review",
            "Wound care",
            "Weight management",
        ];

        reasons
            .choose(&mut self.rng)
            .unwrap_or(&"reasons[0]")
            .to_string()
    }

    fn generate_clinical_notes(&mut self) -> String {
        let templates = [
            "Patient presenting with routine check-up. Vitals stable. No acute concerns.",
            "Follow-up for chronic condition management. Treatment plan reviewed and adjusted.",
            "Acute presentation with mild symptoms. Supportive care advised.",
            "Health assessment completed. All markers within normal range.",
            "Preventive care consultation. Vaccination status reviewed.",
            "Medication review completed. Current regimen appropriate for condition.",
            "Patient reports good compliance with treatment plan. Continue current therapy.",
            "Mental health assessment. Patient showing improvement with current management.",
            "Wound assessment: healing progressing well, no signs of infection.",
            "Patient education provided regarding condition management and lifestyle modifications.",
        ];

        templates
            .choose(&mut self.rng)
            .unwrap_or(&"templates[0]")
            .to_string()
    }

    fn random_condition(&mut self) -> String {
        let conditions = [
            "Hypertension",
            "Type 2 Diabetes",
            "Asthma",
            "COPD",
            "Hyperlipidemia",
            "Arthritis",
            "Anxiety Disorder",
            "Depression",
            "Obesity",
            "Sleep Apnea",
            "Heart Disease",
            "Chronic Pain",
            "Thyroid Disorder",
            "Gastric Reflux",
            "Irritable Bowel Syndrome",
        ];

        conditions
            .choose(&mut self.rng)
            .unwrap_or(&"conditions[0]")
            .to_string()
    }

    fn random_condition_status(&mut self) -> ConditionStatus {
        let statuses = [
            ConditionStatus::Active,
            ConditionStatus::Chronic,
            ConditionStatus::Resolved,
            ConditionStatus::InRemission,
        ];

        statuses
            .choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| statuses[0])
    }

    fn random_severity(&mut self) -> Severity {
        let severities = [Severity::Mild, Severity::Moderate, Severity::Severe];
        severities
            .choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| severities[0])
    }

    fn random_condition_notes(&mut self) -> String {
        let notes = [
            "Stable on current medication",
            "Patient reports symptom improvement",
            "Requires regular monitoring",
            "Lifestyle modifications recommended",
            "Currently under specialist care",
        ];

        notes
            .choose(&mut self.rng)
            .unwrap_or(&"notes[0]")
            .to_string()
    }

    fn random_allergen(&mut self) -> String {
        let allergens = [
            "Penicillin",
            "Aspirin",
            "Codeine",
            "Latex",
            "Peanuts",
            "Tree nuts",
            "Shellfish",
            "Dairy",
            "Eggs",
            "Soy",
            "Gluten",
            "Sulfonamides",
            "NSAIDs",
            "ACE Inhibitors",
            "Local Anesthetics",
        ];

        allergens
            .choose(&mut self.rng)
            .unwrap_or(&"allergens[0]")
            .to_string()
    }

    fn random_allergy_type(&mut self) -> AllergyType {
        let types = [
            AllergyType::Drug,
            AllergyType::Food,
            AllergyType::Environmental,
            AllergyType::Other,
        ];

        types
            .choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| types[0])
    }

    fn random_reaction(&mut self) -> String {
        let reactions = [
            "Rash and itching",
            "Anaphylaxis",
            "Swelling of lips and tongue",
            "Respiratory distress",
            "Gastrointestinal upset",
            "Hives",
            "Angioedema",
            "Hypotension",
            "Loss of consciousness",
        ];

        reactions
            .choose(&mut self.rng)
            .unwrap_or(&"reactions[0]")
            .to_string()
    }

    fn random_allergy_notes(&mut self) -> String {
        let notes = [
            "Requires EpiPen on hand",
            "Previously hospitalized for reaction",
            "Cross-reactivity with related substances",
            "Mild reaction, manageable with antihistamines",
            "Reaction occurs even with small exposure",
        ];

        notes
            .choose(&mut self.rng)
            .unwrap_or(&"notes[0]")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_consultations() {
        let config = ClinicalDataGeneratorConfig {
            consultation_count: 5,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();

        let consultations = generator.generate_consultations(patient_id, practitioner_id);

        assert_eq!(consultations.len(), 5);

        for consultation in &consultations {
            assert_eq!(consultation.patient_id, patient_id);
            assert_eq!(consultation.practitioner_id, practitioner_id);
            assert!(consultation.reason.is_some());
        }
    }

    #[test]
    fn test_generate_medical_history() {
        let config = ClinicalDataGeneratorConfig {
            medical_history_count: 3,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();

        let histories = generator.generate_medical_history(patient_id);

        assert_eq!(histories.len(), 3);

        for history in &histories {
            assert_eq!(history.patient_id, patient_id);
            assert!(!history.condition.is_empty());
            assert!(history.diagnosis_date.is_some());
            assert!(history.is_active);
        }
    }

    #[test]
    fn test_generate_allergies() {
        let config = ClinicalDataGeneratorConfig {
            allergy_count: 2,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();

        let allergies = generator.generate_allergies(patient_id);

        assert_eq!(allergies.len(), 2);

        for allergy in &allergies {
            assert_eq!(allergy.patient_id, patient_id);
            assert!(!allergy.allergen.is_empty());
            assert!(allergy.reaction.is_some());
            assert!(allergy.is_active);
        }
    }

    #[test]
    fn test_clinical_notes_percentage() {
        let config = ClinicalDataGeneratorConfig {
            consultation_count: 100,
            notes_percentage: 1.0,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();

        let consultations = generator.generate_consultations(patient_id, practitioner_id);

        let with_notes = consultations
            .iter()
            .filter(|c| c.clinical_notes.is_some())
            .count();

        assert_eq!(with_notes, 100);
    }

    #[test]
    fn test_signed_consultation_percentage() {
        let config = ClinicalDataGeneratorConfig {
            consultation_count: 50,
            signed_percentage: 1.0,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();

        let consultations = generator.generate_consultations(patient_id, practitioner_id);

        let signed = consultations.iter().filter(|c| c.is_signed).count();

        assert_eq!(signed, 50);
    }

    #[test]
    fn test_severe_allergy_percentage() {
        let config = ClinicalDataGeneratorConfig {
            allergy_count: 50,
            severe_allergy_percentage: 1.0,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();

        let allergies = generator.generate_allergies(patient_id);

        let severe = allergies
            .iter()
            .filter(|a| a.severity == Severity::Severe)
            .count();

        assert_eq!(severe, 50);
    }

    #[test]
    fn test_allergy_types_distribution() {
        let config = ClinicalDataGeneratorConfig {
            allergy_count: 100,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();

        let allergies = generator.generate_allergies(patient_id);

        for allergy in &allergies {
            match allergy.allergy_type {
                AllergyType::Drug
                | AllergyType::Food
                | AllergyType::Environmental
                | AllergyType::Other => {}
            }
        }
    }

    #[test]
    fn test_condition_severity() {
        let config = ClinicalDataGeneratorConfig {
            medical_history_count: 20,
            ..Default::default()
        };

        let mut generator = ClinicalDataGenerator::new(config);
        let patient_id = Uuid::new_v4();

        let histories = generator.generate_medical_history(patient_id);

        for history in &histories {
            if let Some(severity) = history.severity {
                match severity {
                    Severity::Mild | Severity::Moderate | Severity::Severe => {}
                }
            }
        }
    }
}
