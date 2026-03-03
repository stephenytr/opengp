use chrono::{Duration, NaiveDate, Utc};
use fake::faker::name::en::*;
use fake::Fake;
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use opengp_domain::domain::patient::{Address, Gender, Patient};

#[derive(Debug, Clone)]
pub struct PatientGeneratorConfig {
    pub count: usize,
    pub min_age: u32,
    pub max_age: u32,
    pub include_children: bool,
    pub include_seniors: bool,
    pub medicare_percentage: f32,
    pub ihi_percentage: f32,
    pub mobile_percentage: f32,
    pub email_percentage: f32,
}

impl Default for PatientGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            min_age: 0,
            max_age: 100,
            include_children: true,
            include_seniors: true,
            medicare_percentage: 0.95,
            ihi_percentage: 0.90,
            mobile_percentage: 0.85,
            email_percentage: 0.70,
        }
    }
}

pub struct PatientGenerator {
    config: PatientGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl PatientGenerator {
    pub fn new(config: PatientGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    pub fn generate(&mut self) -> Vec<Patient> {
        (0..self.config.count)
            .map(|_| self.generate_patient())
            .collect()
    }

    fn generate_patient(&mut self) -> Patient {
        let gender = self.random_gender();
        let first_name = self.random_first_name(&gender);
        let last_name = self.random_last_name();
        let date_of_birth = self.random_date_of_birth();
        let title = self.random_title(&gender);

        let has_preferred = self.rng.gen_bool(0.15);
        let preferred_name = if has_preferred {
            Some(self.random_preferred_name(&first_name, &gender))
        } else {
            None
        };

        let medicare = if self.rng.gen_bool(self.config.medicare_percentage as f64) {
            Some((
                self.generate_medicare_number(),
                self.rng.gen_range(1..=4),
                Some(self.random_medicare_expiry()),
            ))
        } else {
            None
        };

        let ihi = if self.rng.gen_bool(self.config.ihi_percentage as f64) {
            Some(self.generate_ihi())
        } else {
            None
        };

        let phone_home = if self.rng.gen_bool(0.60) {
            Some(self.generate_landline())
        } else {
            None
        };

        let phone_mobile = if self.rng.gen_bool(self.config.mobile_percentage as f64) {
            Some(self.generate_mobile())
        } else {
            None
        };

        let email = if self.rng.gen_bool(self.config.email_percentage as f64) {
            Some(self.generate_email(&first_name, &last_name))
        } else {
            None
        };

        let has_middle = self.rng.gen_bool(0.60);
        let middle_name = if has_middle {
            Some(self.random_middle_name(&gender))
        } else {
            None
        };

        Patient {
            id: Uuid::new_v4(),
            ihi,
            medicare_number: medicare.as_ref().map(|(num, _, _)| num.clone()),
            medicare_irn: medicare.as_ref().map(|(_, irn, _)| *irn),
            medicare_expiry: medicare.and_then(|(_, _, exp)| exp),
            title: Some(title),
            first_name,
            middle_name,
            last_name,
            preferred_name,
            date_of_birth,
            gender,
            address: self.generate_address(),
            phone_home,
            phone_mobile,
            email,
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: "English".to_string(),
            interpreter_required: false,
            aboriginal_torres_strait_islander: None,
            is_active: true,
            is_deceased: false,
            deceased_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn random_gender(&mut self) -> Gender {
        let choice = self.rng.gen_range(0..100);
        if choice < 48 {
            Gender::Male
        } else if choice < 96 {
            Gender::Female
        } else {
            Gender::Other
        }
    }

    fn random_first_name(&mut self, gender: &Gender) -> String {
        match gender {
            Gender::Male => FirstName().fake_with_rng(&mut self.rng),
            Gender::Female => FirstName().fake_with_rng(&mut self.rng),
            Gender::Other | Gender::PreferNotToSay => FirstName().fake_with_rng(&mut self.rng),
        }
    }

    fn random_last_name(&mut self) -> String {
        LastName().fake_with_rng(&mut self.rng)
    }

    fn random_middle_name(&mut self, _gender: &Gender) -> String {
        FirstName().fake_with_rng(&mut self.rng)
    }

    fn random_preferred_name(&mut self, _first_name: &str, _gender: &Gender) -> String {
        FirstName().fake_with_rng(&mut self.rng)
    }

    fn random_title(&mut self, gender: &Gender) -> String {
        match gender {
            Gender::Male => {
                let titles = ["Mr", "Dr"];
                titles.choose(&mut self.rng).unwrap().to_string()
            }
            Gender::Female => {
                let titles = ["Ms", "Mrs", "Miss", "Dr"];
                titles.choose(&mut self.rng).unwrap().to_string()
            }
            Gender::Other | Gender::PreferNotToSay => "Mx".to_string(),
        }
    }

    fn random_date_of_birth(&mut self) -> NaiveDate {
        let mut min_age = self.config.min_age;
        let mut max_age = self.config.max_age;

        if !self.config.include_children {
            min_age = min_age.max(18);
        }

        if !self.config.include_seniors {
            max_age = max_age.min(64);
        }

        let age = self.rng.gen_range(min_age..=max_age);
        let days_old = age as i64 * 365 + self.rng.gen_range(0..365);

        (Utc::now() - Duration::days(days_old)).date_naive()
    }

    fn random_medicare_expiry(&mut self) -> NaiveDate {
        let months = self.rng.gen_range(1..48);
        (Utc::now() + Duration::days(months * 30)).date_naive()
    }

    fn generate_medicare_number(&mut self) -> String {
        let mut digits = Vec::with_capacity(10);

        for _ in 0..9 {
            digits.push(self.rng.gen_range(0..=9));
        }

        let checksum = self.calculate_medicare_checksum(&digits);
        digits.push(checksum);

        digits.iter().map(|d| d.to_string()).collect::<String>()
    }

    fn calculate_medicare_checksum(&self, digits: &[u8]) -> u8 {
        let weights = [1, 3, 7, 9, 1, 3, 7, 9, 1];
        let sum: u32 = digits
            .iter()
            .zip(weights.iter())
            .map(|(d, w)| *d as u32 * w)
            .sum();

        (sum % 10) as u8
    }

    fn generate_ihi(&mut self) -> String {
        let prefix = "800360816669";
        let suffix: String = (0..4)
            .map(|_| self.rng.gen_range(0..=9).to_string())
            .collect();

        format!("{}{}", prefix, suffix)
    }

    fn generate_landline(&mut self) -> String {
        let area_codes = ["02", "03", "07", "08"];
        let area_code = area_codes.choose(&mut self.rng).unwrap();

        let first = self.rng.gen_range(1000..=9999);
        let second = self.rng.gen_range(1000..=9999);

        format!("{} {} {}", area_code, first, second)
    }

    fn generate_mobile(&mut self) -> String {
        let first = self.rng.gen_range(400..=499);
        let second = self.rng.gen_range(100..=999);
        let third = self.rng.gen_range(100..=999);

        format!("0{} {} {}", first, second, third)
    }

    fn generate_email(&mut self, first_name: &str, last_name: &str) -> String {
        let domains = [
            "gmail.com",
            "outlook.com",
            "hotmail.com",
            "yahoo.com",
            "icloud.com",
            "example.com",
        ];
        let domain = domains.choose(&mut self.rng).unwrap();

        let styles = [
            format!(
                "{}.{}@{}",
                first_name.to_lowercase(),
                last_name.to_lowercase(),
                domain
            ),
            format!(
                "{}{}@{}",
                first_name.to_lowercase(),
                last_name.to_lowercase(),
                domain
            ),
            format!(
                "{}{}@{}",
                first_name.chars().next().unwrap().to_lowercase(),
                last_name.to_lowercase(),
                domain
            ),
            format!(
                "{}.{}{}@{}",
                first_name.to_lowercase(),
                last_name.to_lowercase(),
                self.rng.gen_range(1..99),
                domain
            ),
        ];

        styles.choose(&mut self.rng).unwrap().clone()
    }

    fn generate_address(&mut self) -> Address {
        let street_numbers = self.rng.gen_range(1..=999);
        let street_names = [
            "Smith",
            "George",
            "High",
            "Victoria",
            "King",
            "Queen",
            "Elizabeth",
            "Main",
            "Park",
            "Church",
            "Station",
            "Bridge",
            "Market",
            "Mill",
        ];
        let street_types = [
            "Street", "Road", "Avenue", "Drive", "Lane", "Court", "Place",
        ];

        let suburbs = [
            "Sydney",
            "Melbourne",
            "Brisbane",
            "Perth",
            "Adelaide",
            "Hobart",
            "Darwin",
            "Canberra",
            "Parramatta",
            "Newcastle",
            "Wollongong",
            "Geelong",
            "Townsville",
            "Cairns",
            "Toowoomba",
            "Ballarat",
        ];

        let states = ["NSW", "VIC", "QLD", "WA", "SA", "TAS", "NT", "ACT"];

        let street_name = street_names.choose(&mut self.rng).unwrap();
        let street_type = street_types.choose(&mut self.rng).unwrap();
        let suburb = suburbs.choose(&mut self.rng).unwrap();
        let state = states.choose(&mut self.rng).unwrap();
        let postcode = self.rng.gen_range(1000..=9999);

        Address {
            line1: Some(format!(
                "{} {} {}",
                street_numbers, street_name, street_type
            )),
            line2: None,
            suburb: Some(suburb.to_string()),
            state: Some(state.to_string()),
            postcode: Some(postcode.to_string()),
            country: "Australia".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_patients() {
        let config = PatientGeneratorConfig {
            count: 5,
            ..Default::default()
        };

        let mut generator = PatientGenerator::new(config);
        let patients = generator.generate();

        assert_eq!(patients.len(), 5);

        for patient in &patients {
            assert!(!patient.first_name.is_empty());
            assert!(!patient.last_name.is_empty());
            assert!(patient.is_active);
            assert!(!patient.is_deceased);
        }
    }

    #[test]
    fn test_generate_medicare_number() {
        let config = PatientGeneratorConfig::default();
        let mut generator = PatientGenerator::new(config);

        let medicare = generator.generate_medicare_number();

        assert_eq!(medicare.len(), 10);
        assert!(medicare.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_ihi() {
        let config = PatientGeneratorConfig::default();
        let mut generator = PatientGenerator::new(config);

        let ihi = generator.generate_ihi();

        assert_eq!(ihi.len(), 16);
        assert!(ihi.starts_with("800360816669"));
    }

    #[test]
    fn test_generate_mobile() {
        let config = PatientGeneratorConfig::default();
        let mut generator = PatientGenerator::new(config);

        let mobile = generator.generate_mobile();

        assert!(mobile.starts_with("04"));
        assert!(mobile.len() >= 12);
    }

    #[test]
    fn test_generate_landline() {
        let config = PatientGeneratorConfig::default();
        let mut generator = PatientGenerator::new(config);

        let landline = generator.generate_landline();

        assert!(
            landline.starts_with("02")
                || landline.starts_with("03")
                || landline.starts_with("07")
                || landline.starts_with("08")
        );
    }

    #[test]
    fn test_generate_address() {
        let config = PatientGeneratorConfig::default();
        let mut generator = PatientGenerator::new(config);

        let address = generator.generate_address();

        assert!(address.line1.is_some());
        assert!(address.suburb.is_some());
        assert!(address.state.is_some());
        assert!(address.postcode.is_some());
        assert_eq!(address.country, "Australia");
    }

    #[test]
    fn test_config_age_range() {
        let config = PatientGeneratorConfig {
            count: 10,
            min_age: 18,
            max_age: 65,
            include_children: false,
            include_seniors: false,
            ..Default::default()
        };

        let mut generator = PatientGenerator::new(config);
        let patients = generator.generate();

        let now = Utc::now().date_naive();
        for patient in patients {
            let age = now.years_since(patient.date_of_birth).unwrap_or(0);
            assert!(age >= 18 && age <= 65);
        }
    }

    #[test]
    fn test_gender_distribution() {
        let config = PatientGeneratorConfig {
            count: 100,
            ..Default::default()
        };

        let mut generator = PatientGenerator::new(config);
        let patients = generator.generate();

        let male_count = patients.iter().filter(|p| p.gender == Gender::Male).count();
        let female_count = patients
            .iter()
            .filter(|p| p.gender == Gender::Female)
            .count();

        assert!(male_count > 0);
        assert!(female_count > 0);
    }
}
