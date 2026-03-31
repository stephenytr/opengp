use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

#[cfg(test)]
use chrono::Utc;

use opengp_domain::domain::prescription::{
    AuthorityType, Medication, MedicationForm, PBSStatus, Prescription, PrescriptionType,
};

/// Configuration for prescription generation
///
/// Controls how many prescriptions are generated and their characteristics.
#[derive(Debug, Clone)]
pub struct PrescriptionGeneratorConfig {
    /// Number of prescriptions to generate
    pub count: usize,
    /// Percentage of prescriptions requiring authority (0.0-1.0)
    pub authority_required_percentage: f32,
    /// Percentage of prescriptions with PBS status (0.0-1.0)
    pub pbs_percentage: f32,
    /// Percentage of prescriptions with indication (0.0-1.0)
    pub indication_percentage: f32,
}

impl Default for PrescriptionGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            authority_required_percentage: 0.20,
            pbs_percentage: 0.60,
            indication_percentage: 0.70,
        }
    }
}

/// Generator for realistic prescription test data
///
/// Creates prescriptions with realistic medications, dosages, quantities, and repeats.
/// Links to patient and practitioner IDs.
pub struct PrescriptionGenerator {
    config: PrescriptionGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl PrescriptionGenerator {
    /// Create a new prescription generator with the given configuration
    pub fn new(config: PrescriptionGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a vector of prescriptions
    pub fn generate(&mut self) -> Vec<Prescription> {
        (0..self.config.count)
            .map(|_| self.generate_prescription())
            .collect()
    }

    /// Generate a single prescription with random data
    fn generate_prescription(&mut self) -> Prescription {
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let medication = self.random_medication();
        let dosage = self.random_dosage(&medication);
        let quantity = self.rng.gen_range(10..100);
        let repeats = self.rng.gen_range(0..5);
        let directions = self.random_directions();

        let mut prescription = Prescription::new(
            patient_id,
            practitioner_id,
            None,
            medication,
            dosage,
            quantity,
            repeats,
            directions,
            created_by,
        );

        // Set authority if needed
        if self
            .rng
            .gen_bool(self.config.authority_required_percentage as f64)
        {
            prescription.authority_required = true;
            prescription.authority_type = Some(self.random_authority_type());
            prescription.authority_approval_number = Some(self.random_approval_number());
        }

        // Set PBS status
        if self.rng.gen_bool(self.config.pbs_percentage as f64) {
            prescription.pbs_status = self.random_pbs_status();
            prescription.pbs_item_code = Some(self.random_pbs_item_code());
        }

        // Set indication
        if self.rng.gen_bool(self.config.indication_percentage as f64) {
            prescription.indication = Some(self.random_indication());
        }

        // Set prescription type
        prescription.prescription_type = self.random_prescription_type();

        prescription
    }

    /// Generate a random medication
    fn random_medication(&mut self) -> Medication {
        let medications = [
            ("Paracetamol", "Panadol", "500mg"),
            ("Ibuprofen", "Nurofen", "200mg"),
            ("Amoxicillin", "Amoxil", "500mg"),
            ("Metformin", "Diabex", "500mg"),
            ("Lisinopril", "Carace", "10mg"),
            ("Atorvastatin", "Lipitor", "20mg"),
            ("Omeprazole", "Losec", "20mg"),
            ("Sertraline", "Zoloft", "50mg"),
            ("Fluoxetine", "Prozac", "20mg"),
            ("Cetirizine", "Piriteze", "10mg"),
        ];

        let (generic, brand, strength) = medications
            .choose(&mut self.rng)
            .expect("medications not empty");

        Medication {
            generic_name: generic.to_string(),
            brand_name: Some(brand.to_string()),
            strength: strength.to_string(),
            form: self.random_medication_form(),
            amt_code: None,
        }
    }

    /// Generate a random medication form
    fn random_medication_form(&mut self) -> MedicationForm {
        let forms = [
            MedicationForm::Tablet,
            MedicationForm::Capsule,
            MedicationForm::Liquid,
            MedicationForm::Syrup,
            MedicationForm::Cream,
            MedicationForm::Ointment,
            MedicationForm::Inhaler,
            MedicationForm::Injection,
        ];

        *forms.choose(&mut self.rng).unwrap_or(&forms[0])
    }

    /// Generate a random dosage based on medication form
    fn random_dosage(&mut self, medication: &Medication) -> String {
        match medication.form {
            MedicationForm::Tablet | MedicationForm::Capsule => {
                let quantity = self.rng.gen_range(1..4);
                format!("{} tablet(s) twice daily", quantity)
            }
            MedicationForm::Liquid | MedicationForm::Syrup => {
                let ml = self.rng.gen_range(5..20);
                format!("{}ml three times daily", ml)
            }
            MedicationForm::Cream | MedicationForm::Ointment => {
                "Apply to affected area twice daily".to_string()
            }
            MedicationForm::Inhaler => "2 puffs twice daily".to_string(),
            MedicationForm::Injection => "1 injection weekly".to_string(),
            _ => "As directed".to_string(),
        }
    }

    /// Generate random directions
    fn random_directions(&mut self) -> String {
        let directions = [
            "Take with food",
            "Take on empty stomach",
            "Take with water",
            "Do not take with dairy",
            "Avoid alcohol",
            "May cause drowsiness",
            "Take at bedtime",
            "Take in the morning",
        ];

        directions
            .choose(&mut self.rng)
            .unwrap_or(&directions[0])
            .to_string()
    }

    /// Generate a random PBS status
    fn random_pbs_status(&mut self) -> PBSStatus {
        let statuses = [
            PBSStatus::GeneralSchedule,
            PBSStatus::RestrictedBenefit,
            PBSStatus::AuthorityRequired,
            PBSStatus::RPBS,
        ];

        *statuses.choose(&mut self.rng).unwrap_or(&statuses[0])
    }

    /// Generate a random PBS item code
    fn random_pbs_item_code(&mut self) -> String {
        let code = self.rng.gen_range(10000..99999);
        format!("{}", code)
    }

    /// Generate a random authority type
    fn random_authority_type(&mut self) -> AuthorityType {
        let types = [
            AuthorityType::Streamlined,
            AuthorityType::Complex,
            AuthorityType::Telephone,
            AuthorityType::Written,
        ];

        *types.choose(&mut self.rng).unwrap_or(&types[0])
    }

    /// Generate a random approval number
    fn random_approval_number(&mut self) -> String {
        let num = self.rng.gen_range(100000..999999);
        format!("AUTH{}", num)
    }

    /// Generate a random indication
    fn random_indication(&mut self) -> String {
        let indications = [
            "Hypertension",
            "Type 2 Diabetes",
            "High cholesterol",
            "Anxiety",
            "Depression",
            "Infection",
            "Pain relief",
            "Acid reflux",
            "Allergies",
            "Asthma",
        ];

        indications
            .choose(&mut self.rng)
            .unwrap_or(&indications[0])
            .to_string()
    }

    /// Generate a random prescription type
    fn random_prescription_type(&mut self) -> PrescriptionType {
        let types = [
            PrescriptionType::Paper,
            PrescriptionType::Electronic,
            PrescriptionType::Verbal,
        ];

        *types.choose(&mut self.rng).unwrap_or(&types[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_prescriptions() {
        let config = PrescriptionGeneratorConfig {
            count: 5,
            ..Default::default()
        };

        let mut generator = PrescriptionGenerator::new(config);
        let prescriptions = generator.generate();

        assert_eq!(prescriptions.len(), 5);

        for prescription in &prescriptions {
            assert_ne!(prescription.patient_id, Uuid::nil());
            assert_ne!(prescription.practitioner_id, Uuid::nil());
            assert!(!prescription.medication.generic_name.is_empty());
            assert!(!prescription.dosage.is_empty());
            assert!(prescription.quantity > 0);
            assert!(prescription.is_active);
        }
    }

    #[test]
    fn test_prescription_has_valid_expiry() {
        let config = PrescriptionGeneratorConfig {
            count: 10,
            ..Default::default()
        };

        let mut generator = PrescriptionGenerator::new(config);
        let prescriptions = generator.generate();

        for prescription in &prescriptions {
            if let Some(expiry) = prescription.expiry_date {
                assert!(expiry > Utc::now().date_naive());
            }
        }
    }

    #[test]
    fn test_config_authority_percentage() {
        let config = PrescriptionGeneratorConfig {
            count: 20,
            authority_required_percentage: 0.80,
            ..Default::default()
        };

        let mut generator = PrescriptionGenerator::new(config);
        let prescriptions = generator.generate();

        let authority_count = prescriptions
            .iter()
            .filter(|p| p.authority_required)
            .count();

        assert!(authority_count > 10, "Expected mostly authority required");
    }

    #[test]
    fn test_config_pbs_percentage() {
        let config = PrescriptionGeneratorConfig {
            count: 20,
            pbs_percentage: 0.80,
            ..Default::default()
        };

        let mut generator = PrescriptionGenerator::new(config);
        let prescriptions = generator.generate();

        let pbs_count = prescriptions
            .iter()
            .filter(|p| p.pbs_item_code.is_some())
            .count();

        assert!(pbs_count > 10, "Expected mostly PBS prescriptions");
    }

    #[test]
    fn test_medication_forms_are_valid() {
        let config = PrescriptionGeneratorConfig {
            count: 20,
            ..Default::default()
        };

        let mut generator = PrescriptionGenerator::new(config);
        let prescriptions = generator.generate();

        for prescription in &prescriptions {
            assert!(!prescription.medication.generic_name.is_empty());
            assert!(!prescription.medication.strength.is_empty());
        }
    }

    #[test]
    fn test_prescription_types_are_valid() {
        let config = PrescriptionGeneratorConfig {
            count: 20,
            ..Default::default()
        };

        let mut generator = PrescriptionGenerator::new(config);
        let prescriptions = generator.generate();

        for prescription in &prescriptions {
            match prescription.prescription_type {
                PrescriptionType::Paper
                | PrescriptionType::Electronic
                | PrescriptionType::Verbal => {}
                _ => panic!("Invalid prescription type"),
            }
        }
    }
}
