//! Healthcare configuration types for TOML-based healthcare settings
//!
//! Provides Serde-derivable types for loading and managing healthcare configurations
//! from TOML files. Includes vital sign ranges, appointment durations, billing rates,
//! and prescription/referral expiry settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Top-level healthcare configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareConfig {
    /// Vital sign ranges and units
    #[serde(default)]
    pub vital_signs: HashMap<String, VitalSignRange>,
    /// Appointment duration settings (in minutes)
    #[serde(default)]
    pub appointment_durations: HashMap<String, u32>,
    /// Billing configuration
    #[serde(default)]
    pub billing: BillingConfig,
    /// Prescription configuration
    #[serde(default)]
    pub prescriptions: PrescriptionConfig,
    /// Referral configuration
    #[serde(default)]
    pub referrals: ReferralConfig,
    /// Medicare configuration
    #[serde(default)]
    pub medicare: MedicareConfig,
}

impl Default for HealthcareConfig {
    fn default() -> Self {
        Self {
            vital_signs: HashMap::new(),
            appointment_durations: HashMap::new(),
            billing: BillingConfig::default(),
            prescriptions: PrescriptionConfig::default(),
            referrals: ReferralConfig::default(),
            medicare: MedicareConfig::default(),
        }
    }
}

/// Vital sign range with min, max, and unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSignRange {
    pub min: f64,
    pub max: f64,
    pub unit: String,
}

impl Default for VitalSignRange {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            unit: String::new(),
        }
    }
}

/// Billing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    pub gst_rate: f64,
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self { gst_rate: 0.1 }
    }
}

/// Prescription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescriptionConfig {
    pub default_expiry_days: u32,
}

impl Default for PrescriptionConfig {
    fn default() -> Self {
        Self {
            default_expiry_days: 365,
        }
    }
}

/// Referral configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralConfig {
    pub default_expiry_days: u32,
}

impl Default for ReferralConfig {
    fn default() -> Self {
        Self {
            default_expiry_days: 365,
        }
    }
}

/// Medicare configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicareConfig {
    pub number_length: u32,
    pub digits_only: bool,
}

impl Default for MedicareConfig {
    fn default() -> Self {
        Self {
            number_length: 10,
            digits_only: true,
        }
    }
}

impl HealthcareConfig {
    /// Load healthcare configuration from embedded defaults and optional external override
    pub fn load() -> Result<Self, HealthcareConfigError> {
        let defaults = include_str!("healthcare.toml");
        let mut config: HealthcareConfig =
            toml::from_str(defaults).map_err(HealthcareConfigError::Parse)?;

        if let Ok(path) = std::env::var("HEALTHCARE_CONFIG_PATH") {
            let content = std::fs::read_to_string(&path).map_err(HealthcareConfigError::Io)?;
            let overrides: PartialHealthcareConfig =
                toml::from_str(&content).map_err(HealthcareConfigError::Parse)?;
            config.deep_merge(overrides);
        }

        config.validate()?;
        Ok(config)
    }

    /// Deep merge external overrides into the configuration
    fn deep_merge(&mut self, overrides: PartialHealthcareConfig) {
        if let Some(vital_signs) = overrides.vital_signs {
            for (key, value) in vital_signs {
                self.vital_signs.insert(key, value);
            }
        }

        if let Some(appointment_durations) = overrides.appointment_durations {
            for (key, value) in appointment_durations {
                self.appointment_durations.insert(key, value);
            }
        }

        if let Some(billing) = overrides.billing {
            if let Some(gst_rate) = billing.gst_rate {
                self.billing.gst_rate = gst_rate;
            }
        }

        if let Some(prescriptions) = overrides.prescriptions {
            if let Some(default_expiry_days) = prescriptions.default_expiry_days {
                self.prescriptions.default_expiry_days = default_expiry_days;
            }
        }

        if let Some(referrals) = overrides.referrals {
            if let Some(default_expiry_days) = referrals.default_expiry_days {
                self.referrals.default_expiry_days = default_expiry_days;
            }
        }

        if let Some(medicare) = overrides.medicare {
            if let Some(number_length) = medicare.number_length {
                self.medicare.number_length = number_length;
            }
            if let Some(digits_only) = medicare.digits_only {
                self.medicare.digits_only = digits_only;
            }
        }
    }

    /// Validate healthcare configuration
    fn validate(&self) -> Result<(), HealthcareConfigError> {
        // Validate vital sign ranges
        for (name, range) in &self.vital_signs {
            if range.min > range.max {
                return Err(HealthcareConfigError::Validation(format!(
                    "vital sign '{}' has min ({}) > max ({})",
                    name, range.min, range.max
                )));
            }
            if range.unit.trim().is_empty() {
                return Err(HealthcareConfigError::Validation(format!(
                    "vital sign '{}' has empty unit",
                    name
                )));
            }
        }

        // Validate appointment durations
        for (name, duration) in &self.appointment_durations {
            if *duration == 0 {
                return Err(HealthcareConfigError::Validation(format!(
                    "appointment duration '{}' must be > 0",
                    name
                )));
            }
        }

        // Validate billing
        if self.billing.gst_rate < 0.0 || self.billing.gst_rate > 1.0 {
            return Err(HealthcareConfigError::Validation(
                "gst_rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate prescriptions
        if self.prescriptions.default_expiry_days == 0 {
            return Err(HealthcareConfigError::Validation(
                "prescription default_expiry_days must be > 0".to_string(),
            ));
        }

        // Validate referrals
        if self.referrals.default_expiry_days == 0 {
            return Err(HealthcareConfigError::Validation(
                "referral default_expiry_days must be > 0".to_string(),
            ));
        }

        // Validate medicare
        if self.medicare.number_length == 0 {
            return Err(HealthcareConfigError::Validation(
                "medicare number_length must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get a specific vital sign range by name
    pub fn get_vital_sign(&self, name: &str) -> Option<&VitalSignRange> {
        self.vital_signs.get(name)
    }

    /// Get a specific appointment duration by name
    pub fn get_appointment_duration(&self, name: &str) -> Option<u32> {
        self.appointment_durations.get(name).copied()
    }
}

/// Partial healthcare configuration for TOML deserialization with optional fields
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialHealthcareConfig {
    #[serde(default)]
    pub vital_signs: Option<HashMap<String, VitalSignRange>>,
    #[serde(default)]
    pub appointment_durations: Option<HashMap<String, u32>>,
    #[serde(default)]
    pub billing: Option<PartialBillingConfig>,
    #[serde(default)]
    pub prescriptions: Option<PartialPrescriptionConfig>,
    #[serde(default)]
    pub referrals: Option<PartialReferralConfig>,
    #[serde(default)]
    pub medicare: Option<PartialMedicareConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialBillingConfig {
    pub gst_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialPrescriptionConfig {
    pub default_expiry_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialReferralConfig {
    pub default_expiry_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialMedicareConfig {
    pub number_length: Option<u32>,
    pub digits_only: Option<bool>,
}

#[derive(Debug)]
pub enum HealthcareConfigError {
    Parse(toml::de::Error),
    Validation(String),
    Io(std::io::Error),
}

impl fmt::Display for HealthcareConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthcareConfigError::Parse(e) => write!(f, "TOML parse error: {}", e),
            HealthcareConfigError::Validation(msg) => write!(f, "Validation error: {}", msg),
            HealthcareConfigError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for HealthcareConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn write_temp_healthcare_override(content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!(
            "healthcare_override_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, content).expect("should write temp file");
        path
    }

    #[test]
    fn test_healthcare_config_default() {
        let config = HealthcareConfig::default();
        assert!(config.vital_signs.is_empty());
        assert!(config.appointment_durations.is_empty());
        assert_eq!(config.billing.gst_rate, 0.1);
        assert_eq!(config.prescriptions.default_expiry_days, 365);
        assert_eq!(config.referrals.default_expiry_days, 365);
        assert_eq!(config.medicare.number_length, 10);
        assert!(config.medicare.digits_only);
    }

    #[test]
    fn test_vital_sign_range_default() {
        let range = VitalSignRange::default();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.max, 100.0);
        assert_eq!(range.unit, "");
    }

    #[test]
    fn test_load_embedded_defaults() {
        temp_env::with_vars([("HEALTHCARE_CONFIG_PATH", None::<&str>)], || {
            let config = HealthcareConfig::load().expect("embedded healthcare config should load");

            // Verify vital signs exist and have correct values
            assert!(config.vital_signs.contains_key("systolic_bp"));
            assert!(config.vital_signs.contains_key("diastolic_bp"));
            assert!(config.vital_signs.contains_key("heart_rate"));
            assert!(config.vital_signs.contains_key("respiratory_rate"));
            assert!(config.vital_signs.contains_key("temperature"));
            assert!(config.vital_signs.contains_key("o2_saturation"));
            assert!(config.vital_signs.contains_key("height"));
            assert!(config.vital_signs.contains_key("weight"));

            // Verify specific vital sign ranges
            let systolic = config.get_vital_sign("systolic_bp").unwrap();
            assert_eq!(systolic.min, 50.0);
            assert_eq!(systolic.max, 300.0);
            assert_eq!(systolic.unit, "mmHg");

            let diastolic = config.get_vital_sign("diastolic_bp").unwrap();
            assert_eq!(diastolic.min, 20.0);
            assert_eq!(diastolic.max, 200.0);
            assert_eq!(diastolic.unit, "mmHg");

            let heart_rate = config.get_vital_sign("heart_rate").unwrap();
            assert_eq!(heart_rate.min, 20.0);
            assert_eq!(heart_rate.max, 300.0);
            assert_eq!(heart_rate.unit, "bpm");

            let respiratory_rate = config.get_vital_sign("respiratory_rate").unwrap();
            assert_eq!(respiratory_rate.min, 4.0);
            assert_eq!(respiratory_rate.max, 60.0);
            assert_eq!(respiratory_rate.unit, "breaths/min");

            let temperature = config.get_vital_sign("temperature").unwrap();
            assert_eq!(temperature.min, 30.0);
            assert_eq!(temperature.max, 45.0);
            assert_eq!(temperature.unit, "°C");

            let o2_saturation = config.get_vital_sign("o2_saturation").unwrap();
            assert_eq!(o2_saturation.min, 50.0);
            assert_eq!(o2_saturation.max, 100.0);
            assert_eq!(o2_saturation.unit, "%");

            let height = config.get_vital_sign("height").unwrap();
            assert_eq!(height.min, 30.0);
            assert_eq!(height.max, 300.0);
            assert_eq!(height.unit, "cm");

            let weight = config.get_vital_sign("weight").unwrap();
            assert_eq!(weight.min, 0.5);
            assert_eq!(weight.max, 700.0);
            assert_eq!(weight.unit, "kg");

            // Verify appointment durations
            assert_eq!(config.get_appointment_duration("standard"), Some(15));
            assert_eq!(config.get_appointment_duration("long"), Some(30));
            assert_eq!(config.get_appointment_duration("brief"), Some(10));
            assert_eq!(config.get_appointment_duration("new_patient"), Some(45));

            // Verify billing
            assert_eq!(config.billing.gst_rate, 0.1);

            // Verify prescriptions
            assert_eq!(config.prescriptions.default_expiry_days, 365);

            // Verify referrals
            assert_eq!(config.referrals.default_expiry_days, 365);

            // Verify medicare
            assert_eq!(config.medicare.number_length, 10);
            assert!(config.medicare.digits_only);
        });
    }

    #[test]
    fn test_load_external_override_deep_merge() {
        temp_env::with_vars([("HEALTHCARE_CONFIG_PATH", None::<&str>)], || {
            let override_toml = r#"
[vital_signs.systolic_bp]
min = 60
max = 280
unit = "mmHg"

[appointment_durations]
standard = 20
custom = 25

[billing]
gst_rate = 0.15

[prescriptions]
default_expiry_days = 180

[referrals]
default_expiry_days = 200

[medicare]
number_length = 11
digits_only = false
"#;

            let path = write_temp_healthcare_override(override_toml);

            temp_env::with_vars(
                [(
                    "HEALTHCARE_CONFIG_PATH",
                    Some(path.to_string_lossy().as_ref()),
                )],
                || {
                    let config = HealthcareConfig::load()
                        .expect("healthcare config with override should load");

                    // Verify systolic_bp was overridden
                    let systolic = config.get_vital_sign("systolic_bp").unwrap();
                    assert_eq!(systolic.min, 60.0);
                    assert_eq!(systolic.max, 280.0);

                    // Verify other vital signs remain from defaults
                    let diastolic = config.get_vital_sign("diastolic_bp").unwrap();
                    assert_eq!(diastolic.min, 20.0);
                    assert_eq!(diastolic.max, 200.0);

                    // Verify appointment duration override
                    assert_eq!(config.get_appointment_duration("standard"), Some(20));
                    // Verify new appointment duration added
                    assert_eq!(config.get_appointment_duration("custom"), Some(25));
                    // Verify other durations remain
                    assert_eq!(config.get_appointment_duration("long"), Some(30));

                    // Verify billing override
                    assert_eq!(config.billing.gst_rate, 0.15);

                    // Verify prescriptions override
                    assert_eq!(config.prescriptions.default_expiry_days, 180);

                    // Verify referrals override
                    assert_eq!(config.referrals.default_expiry_days, 200);

                    // Verify medicare override
                    assert_eq!(config.medicare.number_length, 11);
                    assert!(!config.medicare.digits_only);
                },
            );

            let _ = fs::remove_file(path);
        });
    }

    #[test]
    fn test_validation_vital_sign_min_greater_than_max() {
        let toml_str = r#"
[vital_signs.invalid]
min = 100
max = 50
unit = "test"
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("min (100) > max (50)"));
    }

    #[test]
    fn test_validation_vital_sign_empty_unit() {
        let toml_str = r#"
[vital_signs.invalid]
min = 0
max = 100
unit = ""
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty unit"));
    }

    #[test]
    fn test_validation_appointment_duration_zero() {
        let toml_str = r#"
[appointment_durations]
invalid = 0
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be > 0"));
    }

    #[test]
    fn test_validation_gst_rate_out_of_range() {
        let toml_str = r#"
[billing]
gst_rate = 1.5
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("gst_rate must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_validation_prescription_expiry_zero() {
        let toml_str = r#"
[prescriptions]
default_expiry_days = 0
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("prescription default_expiry_days must be > 0"));
    }

    #[test]
    fn test_validation_referral_expiry_zero() {
        let toml_str = r#"
[referrals]
default_expiry_days = 0
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("referral default_expiry_days must be > 0"));
    }

    #[test]
    fn test_validation_medicare_number_length_zero() {
        let toml_str = r#"
[medicare]
number_length = 0
digits_only = true
"#;

        let config: Result<HealthcareConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());

        let config = config.unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("medicare number_length must be > 0"));
    }

    #[test]
    fn test_get_vital_sign_exists() {
        let mut config = HealthcareConfig::default();
        config.vital_signs.insert(
            "test".to_string(),
            VitalSignRange {
                min: 10.0,
                max: 20.0,
                unit: "units".to_string(),
            },
        );

        let range = config.get_vital_sign("test");
        assert!(range.is_some());
        assert_eq!(range.unwrap().min, 10.0);
    }

    #[test]
    fn test_get_vital_sign_not_exists() {
        let config = HealthcareConfig::default();
        let range = config.get_vital_sign("nonexistent");
        assert!(range.is_none());
    }

    #[test]
    fn test_get_appointment_duration_exists() {
        let mut config = HealthcareConfig::default();
        config.appointment_durations.insert("test".to_string(), 25);

        let duration = config.get_appointment_duration("test");
        assert_eq!(duration, Some(25));
    }

    #[test]
    fn test_get_appointment_duration_not_exists() {
        let config = HealthcareConfig::default();
        let duration = config.get_appointment_duration("nonexistent");
        assert!(duration.is_none());
    }
}
