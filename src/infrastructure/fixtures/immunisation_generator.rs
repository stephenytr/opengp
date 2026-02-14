use chrono::{Duration, NaiveDate, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use crate::domain::immunisation::{
    AdministrationRoute, AnatomicalSite, ConsentType, Immunisation, Vaccine, VaccineType,
};

/// Configuration for immunisation generation
///
/// Controls how many immunisations are generated and their characteristics.
#[derive(Debug, Clone)]
pub struct ImmunisationGeneratorConfig {
    /// Number of immunisations to generate
    pub count: usize,
    /// Percentage of immunisations reported to AIR (0.0-1.0)
    pub air_reported_percentage: f32,
    /// Percentage of immunisations with adverse events (0.0-1.0)
    pub adverse_event_percentage: f32,
    /// Maximum days in the past for immunisation dates
    pub max_days_past: i64,
}

impl Default for ImmunisationGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            air_reported_percentage: 0.80,
            adverse_event_percentage: 0.05,
            max_days_past: 365,
        }
    }
}

/// Generator for realistic immunisation test data
///
/// Creates immunisations with vaccine types, batch numbers, sites, and AIR reporting status.
/// Links to patient IDs.
pub struct ImmunisationGenerator {
    config: ImmunisationGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl ImmunisationGenerator {
    /// Create a new immunisation generator with the given configuration
    pub fn new(config: ImmunisationGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a vector of immunisations
    pub fn generate(&mut self) -> Vec<Immunisation> {
        (0..self.config.count)
            .map(|_| self.generate_immunisation())
            .collect()
    }

    /// Generate a single immunisation with random data
    fn generate_immunisation(&mut self) -> Immunisation {
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let vaccine = self.random_vaccine();
        let vaccination_date = self.random_vaccination_date();
        let dose_number = self.rng.gen_range(1..=4);
        let batch_number = self.random_batch_number();
        let route = self.random_administration_route();
        let site = self.random_anatomical_site(&route);

        let mut immunisation = Immunisation::new(
            patient_id,
            practitioner_id,
            vaccine,
            vaccination_date,
            dose_number,
            batch_number,
            route,
            site,
            created_by,
        );

        // Set total doses
        immunisation.total_doses = Some(self.rng.gen_range(1..=4));

        // Set expiry date
        immunisation.expiry_date = Some(vaccination_date + Duration::days(365));

        // Set manufacturer
        immunisation.manufacturer = Some(self.random_manufacturer());

        // Set dose quantity
        immunisation.dose_quantity = Some(self.rng.gen_range(0.5..1.0));
        immunisation.dose_unit = Some("mL".to_string());

        // Set consent type
        immunisation.consent_type = Some(self.random_consent_type());

        // Set AIR reporting
        if self
            .rng
            .gen_bool(self.config.air_reported_percentage as f64)
        {
            immunisation.mark_air_reported(self.random_transaction_id());
        }

        // Set adverse event
        if self
            .rng
            .gen_bool(self.config.adverse_event_percentage as f64)
        {
            immunisation.adverse_event = true;
            immunisation.adverse_event_details = Some(self.random_adverse_event_detail());
        }

        immunisation
    }

    /// Generate a random vaccine
    fn random_vaccine(&mut self) -> Vaccine {
        let vaccine_types = [
            ("COVID-19", VaccineType::COVID19, "Pfizer"),
            ("Influenza", VaccineType::Influenza, "Fluzone"),
            ("Pneumococcal", VaccineType::Pneumococcal, "Pneumovax"),
            ("Shingles", VaccineType::Shingles, "Shingrix"),
            ("MMR", VaccineType::MMR, "M-M-RvaxPro"),
            ("DTPa", VaccineType::DTPa, "Boostrix"),
            ("Polio", VaccineType::Polio, "Poliovax"),
            ("Hepatitis B", VaccineType::HepB, "Engerix-B"),
            ("Hepatitis A", VaccineType::HepA, "Havrix"),
            ("Hib", VaccineType::Hib, "ActHIB"),
        ];

        let (name, vaccine_type, brand) = vaccine_types
            .choose(&mut self.rng)
            .expect("vaccines not empty");

        Vaccine {
            name: name.to_string(),
            vaccine_type: *vaccine_type,
            brand_name: Some(brand.to_string()),
            snomed_code: None,
            amt_code: None,
        }
    }

    /// Generate a random administration route
    fn random_administration_route(&mut self) -> AdministrationRoute {
        let routes = [
            AdministrationRoute::Intramuscular,
            AdministrationRoute::Subcutaneous,
            AdministrationRoute::Intradermal,
            AdministrationRoute::Oral,
            AdministrationRoute::Intranasal,
        ];

        *routes.choose(&mut self.rng).expect("routes not empty")
    }

    /// Generate a random anatomical site based on route
    fn random_anatomical_site(&mut self, route: &AdministrationRoute) -> AnatomicalSite {
        match route {
            AdministrationRoute::Intramuscular => {
                let sites = [
                    AnatomicalSite::LeftDeltoid,
                    AnatomicalSite::RightDeltoid,
                    AnatomicalSite::LeftThigh,
                    AnatomicalSite::RightThigh,
                ];
                *sites.choose(&mut self.rng).expect("sites not empty")
            }
            AdministrationRoute::Subcutaneous => {
                let sites = [
                    AnatomicalSite::LeftUpperArm,
                    AnatomicalSite::RightUpperArm,
                    AnatomicalSite::LeftThigh,
                    AnatomicalSite::RightThigh,
                ];
                *sites.choose(&mut self.rng).expect("sites not empty")
            }
            AdministrationRoute::Intradermal => {
                let sites = [AnatomicalSite::LeftDeltoid, AnatomicalSite::RightDeltoid];
                *sites.choose(&mut self.rng).expect("sites not empty")
            }
            AdministrationRoute::Oral => AnatomicalSite::Oral,
            AdministrationRoute::Intranasal => AnatomicalSite::Intranasal,
        }
    }

    /// Generate a random batch number
    fn random_batch_number(&mut self) -> String {
        let batch = self.rng.gen_range(100000..999999);
        format!("BATCH{}", batch)
    }

    /// Generate a random manufacturer
    fn random_manufacturer(&mut self) -> String {
        let manufacturers = [
            "Pfizer",
            "Moderna",
            "AstraZeneca",
            "Janssen",
            "Merck",
            "GSK",
            "Sanofi",
            "Novavax",
        ];

        manufacturers
            .choose(&mut self.rng)
            .expect("manufacturers not empty")
            .to_string()
    }

    /// Generate a random consent type
    fn random_consent_type(&mut self) -> ConsentType {
        let types = [
            ConsentType::Written,
            ConsentType::Verbal,
            ConsentType::Implied,
        ];

        *types.choose(&mut self.rng).expect("types not empty")
    }

    /// Generate a random vaccination date
    fn random_vaccination_date(&mut self) -> NaiveDate {
        let days_ago = self.rng.gen_range(0..=self.config.max_days_past);
        (Utc::now() - Duration::days(days_ago)).date_naive()
    }

    /// Generate a random AIR transaction ID
    fn random_transaction_id(&mut self) -> String {
        let id = self.rng.gen_range(1000000000i64..9999999999i64);
        format!("AIR{}", id)
    }

    /// Generate a random adverse event detail
    fn random_adverse_event_detail(&mut self) -> String {
        let details = [
            "Mild fever",
            "Arm soreness",
            "Headache",
            "Fatigue",
            "Mild rash",
            "Swelling at injection site",
        ];

        details
            .choose(&mut self.rng)
            .expect("details not empty")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_immunisations() {
        let config = ImmunisationGeneratorConfig {
            count: 5,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        assert_eq!(immunisations.len(), 5);

        for immunisation in &immunisations {
            assert_ne!(immunisation.patient_id, Uuid::nil());
            assert_ne!(immunisation.practitioner_id, Uuid::nil());
            assert!(!immunisation.vaccine.name.is_empty());
            assert!(!immunisation.batch_number.is_empty());
            assert!(immunisation.dose_number > 0);
        }
    }

    #[test]
    fn test_immunisation_dates_are_valid() {
        let config = ImmunisationGeneratorConfig {
            count: 10,
            max_days_past: 365,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        let now = Utc::now().date_naive();
        for immunisation in &immunisations {
            assert!(immunisation.vaccination_date <= now);
            let days_old = (now - immunisation.vaccination_date).num_days();
            assert!(days_old <= 365);
        }
    }

    #[test]
    fn test_config_air_reported_percentage() {
        let config = ImmunisationGeneratorConfig {
            count: 20,
            air_reported_percentage: 0.80,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        let reported_count = immunisations.iter().filter(|i| i.air_reported).count();

        assert!(reported_count > 10, "Expected mostly AIR reported");
    }

    #[test]
    fn test_config_adverse_event_percentage() {
        let config = ImmunisationGeneratorConfig {
            count: 100,
            adverse_event_percentage: 0.50,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        let adverse_count = immunisations.iter().filter(|i| i.adverse_event).count();

        assert!(adverse_count > 30, "Expected more adverse events");
    }

    #[test]
    fn test_vaccine_types_are_valid() {
        let config = ImmunisationGeneratorConfig {
            count: 20,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        for immunisation in &immunisations {
            assert!(!immunisation.vaccine.name.is_empty());
            assert!(immunisation.vaccine.brand_name.is_some());
        }
    }

    #[test]
    fn test_anatomical_sites_match_routes() {
        let config = ImmunisationGeneratorConfig {
            count: 20,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        for immunisation in &immunisations {
            match immunisation.route {
                AdministrationRoute::Oral => {
                    assert_eq!(immunisation.site, AnatomicalSite::Oral);
                }
                AdministrationRoute::Intranasal => {
                    assert_eq!(immunisation.site, AnatomicalSite::Intranasal);
                }
                _ => {
                    assert!(
                        immunisation.site != AnatomicalSite::Oral
                            && immunisation.site != AnatomicalSite::Intranasal
                    );
                }
            }
        }
    }

    #[test]
    fn test_consent_obtained_is_true() {
        let config = ImmunisationGeneratorConfig {
            count: 10,
            ..Default::default()
        };

        let mut generator = ImmunisationGenerator::new(config);
        let immunisations = generator.generate();

        for immunisation in &immunisations {
            assert!(immunisation.consent_obtained);
            assert!(immunisation.consent_type.is_some());
        }
    }
}
