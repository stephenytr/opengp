pub mod forms;
pub mod healthcare;

use crate::forms::FormConfig;
use crate::healthcare::HealthcareConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumOption {
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub ttl_default_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
            min_connections: 2,
            ttl_default_secs: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_scrollbars: bool,
    pub mouse_support: bool,
    pub tick_rate_ms: u64,
    #[serde(default = "default_min_terminal_width")]
    pub min_terminal_width: u16,
    #[serde(default = "default_min_terminal_height")]
    pub min_terminal_height: u16,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_scrollbars: true,
            mouse_support: true,
            tick_rate_ms: 16,
            min_terminal_width: 80,
            min_terminal_height: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    pub min_hour: u8,
    pub max_hour: u8,
    pub viewport_start_hour: u8,
    pub viewport_end_hour: u8,
    pub appointment_type_abbreviations: HashMap<String, String>,
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 8,
            viewport_end_hour: 18,
            appointment_type_abbreviations: Self::default_appointment_type_abbreviations(),
        }
    }
}

impl CalendarConfig {
    pub fn default_appointment_type_abbreviations() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("Standard".to_string(), "STD".to_string());
        map.insert("Long".to_string(), "LNG".to_string());
        map.insert("Brief".to_string(), "BRF".to_string());
        map.insert("NewPatient".to_string(), "NEW".to_string());
        map.insert("HealthAssessment".to_string(), "HLT".to_string());
        map.insert("ChronicDiseaseReview".to_string(), "CHR".to_string());
        map.insert("MentalHealthPlan".to_string(), "MHP".to_string());
        map.insert("Immunisation".to_string(), "IMM".to_string());
        map.insert("Procedure".to_string(), "PRC".to_string());
        map.insert("Telephone".to_string(), "TEL".to_string());
        map.insert("Telehealth".to_string(), "TLH".to_string());
        map.insert("HomeVisit".to_string(), "HOM".to_string());
        map.insert("Emergency".to_string(), "EMG".to_string());
        map
    }

    pub fn get_abbreviation(&self, appointment_type: &str) -> String {
        self.appointment_type_abbreviations
            .get(appointment_type)
            .cloned()
            .unwrap_or_else(|| appointment_type.chars().take(3).collect::<String>())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            database: DatabaseConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    pub base_url: String,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_file: String,
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_file: "logs/opengp.log".to_string(),
            level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub timeout_secs: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self { timeout_secs: 900 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub open_duration_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_duration_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StampedeConfig {
    pub default_ttl_secs: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

impl Default for StampedeConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 5,
            retry_attempts: 3,
            retry_delay_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTtlConfig {
    pub patient_secs: u64,
    pub search_secs: u64,
    pub appointment_secs: u64,
}

impl Default for EntityTtlConfig {
    fn default() -> Self {
        Self {
            patient_secs: 900,
            search_secs: 300,
            appointment_secs: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub default_ttl_secs: u64,
    pub key_prefix: String,
    pub circuit_breaker: CircuitBreakerConfig,
    pub stampede: StampedeConfig,
    pub entity_ttl: EntityTtlConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 3600,
            key_prefix: "opengp".to_string(),
            circuit_breaker: CircuitBreakerConfig::default(),
            stampede: StampedeConfig::default(),
            entity_ttl: EntityTtlConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub api_server: ApiServerConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub calendar: CalendarConfig,
    #[serde(default)]
    pub api_client: ApiClientConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllergyConfig {
    #[serde(default)]
    pub allergy_types: HashMap<String, EnumOption>,
    #[serde(default)]
    pub severities: HashMap<String, EnumOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentTypeOption {
    pub label: String,
    pub abbreviation: String,
    pub duration_minutes: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppointmentConfig {
    #[serde(default)]
    pub types: HashMap<String, AppointmentTypeOption>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClinicalConfig {
    #[serde(default)]
    pub condition_status: HashMap<String, EnumOption>,
    #[serde(default)]
    pub severity: HashMap<String, EnumOption>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialHistoryConfig {
    #[serde(default)]
    pub smoking_status: HashMap<String, EnumOption>,
    #[serde(default)]
    pub alcohol_status: HashMap<String, EnumOption>,
    #[serde(default)]
    pub exercise_frequency: HashMap<String, EnumOption>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatientConfig {
    #[serde(default)]
    pub gender: HashMap<String, EnumOption>,
    #[serde(default)]
    pub concession_type: HashMap<String, EnumOption>,
    #[serde(default)]
    pub atsi_status: HashMap<String, EnumOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub foreground: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub info: String,
    pub border: String,
    pub selected: String,
    pub highlight: String,
    pub disabled: String,
    pub scrollbar_bg: String,
    pub scrollbar_thumb: String,
    pub appointment_scheduled: String,
    pub appointment_confirmed: String,
    pub appointment_arrived: String,
    pub appointment_in_progress: String,
    pub appointment_completed: String,
    pub appointment_cancelled: String,
    pub appointment_dna: String,
    pub appointment_rescheduled: String,
    pub background_dark: String,
    pub text_dim: String,
    pub text_secondary: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub dark: ColorPalette,
    #[serde(default)]
    pub light: ColorPalette,
    #[serde(default)]
    pub high_contrast: ColorPalette,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub provider_number: String,
    pub specialty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PracticeProfile {
    pub name: String,
    pub abn: String,
    pub accreditation_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PracticeContact {
    pub phone: String,
    pub email: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PracticeBanking {
    pub bsb: String,
    pub account_number: String,
    pub account_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PracticeConfig {
    #[serde(default)]
    pub profile: PracticeProfile,
    #[serde(default)]
    pub contact: PracticeContact,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub banking: PracticeBanking,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "Cyan".to_string(),
            secondary: "Magenta".to_string(),
            background: "Black".to_string(),
            foreground: "DarkGray".to_string(),
            error: "Red".to_string(),
            success: "Green".to_string(),
            warning: "Yellow".to_string(),
            info: "Blue".to_string(),
            border: "DarkGray".to_string(),
            selected: "Blue".to_string(),
            highlight: "LightBlue".to_string(),
            disabled: "Gray".to_string(),
            scrollbar_bg: "DarkGray".to_string(),
            scrollbar_thumb: "Gray".to_string(),
            appointment_scheduled: "Yellow".to_string(),
            appointment_confirmed: "Cyan".to_string(),
            appointment_arrived: "Green".to_string(),
            appointment_in_progress: "LightBlue".to_string(),
            appointment_completed: "Green".to_string(),
            appointment_cancelled: "Red".to_string(),
            appointment_dna: "Red".to_string(),
            appointment_rescheduled: "Rgb(180, 100, 20)".to_string(),
            background_dark: "Black".to_string(),
            text_dim: "DarkGray".to_string(),
            text_secondary: "Gray".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAppConfig {
    #[serde(default)]
    pub api_server: Option<PartialApiServerConfig>,
    #[serde(default)]
    pub ui: Option<PartialUiConfig>,
    #[serde(default)]
    pub calendar: Option<PartialCalendarConfig>,
    #[serde(default)]
    pub api_client: Option<PartialApiClientConfig>,
    #[serde(default)]
    pub logging: Option<PartialLoggingConfig>,
    #[serde(default)]
    pub session: Option<PartialSessionConfig>,
    #[serde(default)]
    pub cache: Option<PartialCacheConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialApiServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<PartialDatabaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialDatabaseConfig {
    pub url: Option<String>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout_secs: Option<u64>,
    pub idle_timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialUiConfig {
    pub theme: Option<String>,
    pub show_scrollbars: Option<bool>,
    pub mouse_support: Option<bool>,
    pub tick_rate_ms: Option<u64>,
    pub min_terminal_width: Option<u16>,
    pub min_terminal_height: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialCalendarConfig {
    pub min_hour: Option<u8>,
    pub max_hour: Option<u8>,
    pub viewport_start_hour: Option<u8>,
    pub viewport_end_hour: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialApiClientConfig {
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialLoggingConfig {
    pub log_file: Option<String>,
    pub level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialSessionConfig {
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialCacheConfig {
    pub default_ttl_secs: Option<u64>,
    pub key_prefix: Option<String>,
    pub circuit_breaker: Option<PartialCircuitBreakerConfig>,
    pub stampede: Option<PartialStampedeConfig>,
    pub entity_ttl: Option<PartialEntityTtlConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialCircuitBreakerConfig {
    pub failure_threshold: Option<u32>,
    pub open_duration_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialStampedeConfig {
    pub default_ttl_secs: Option<u64>,
    pub retry_attempts: Option<u32>,
    pub retry_delay_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialEntityTtlConfig {
    pub patient_secs: Option<u64>,
    pub search_secs: Option<u64>,
    pub appointment_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAllergyConfig {
    #[serde(default)]
    pub allergy_types: Option<HashMap<String, EnumOption>>,
    #[serde(default)]
    pub severities: Option<HashMap<String, EnumOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAppointmentConfig {
    #[serde(default)]
    pub types: Option<HashMap<String, PartialAppointmentTypeOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAppointmentTypeOption {
    pub label: Option<String>,
    pub abbreviation: Option<String>,
    pub duration_minutes: Option<u32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialClinicalConfig {
    #[serde(default)]
    pub condition_status: Option<HashMap<String, EnumOption>>,
    #[serde(default)]
    pub severity: Option<HashMap<String, EnumOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialSocialHistoryConfig {
    #[serde(default)]
    pub smoking_status: Option<HashMap<String, EnumOption>>,
    #[serde(default)]
    pub alcohol_status: Option<HashMap<String, EnumOption>>,
    #[serde(default)]
    pub exercise_frequency: Option<HashMap<String, EnumOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialPatientConfig {
    #[serde(default)]
    pub gender: Option<HashMap<String, EnumOption>>,
    #[serde(default)]
    pub concession_type: Option<HashMap<String, EnumOption>>,
    #[serde(default)]
    pub atsi_status: Option<HashMap<String, EnumOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialThemeConfig {
    pub dark: Option<PartialColorPalette>,
    pub light: Option<PartialColorPalette>,
    pub high_contrast: Option<PartialColorPalette>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialColorPalette {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub error: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub info: Option<String>,
    pub border: Option<String>,
    pub selected: Option<String>,
    pub highlight: Option<String>,
    pub disabled: Option<String>,
    pub scrollbar_bg: Option<String>,
    pub scrollbar_thumb: Option<String>,
    pub appointment_scheduled: Option<String>,
    pub appointment_confirmed: Option<String>,
    pub appointment_arrived: Option<String>,
    pub appointment_in_progress: Option<String>,
    pub appointment_completed: Option<String>,
    pub appointment_cancelled: Option<String>,
    pub appointment_dna: Option<String>,
    pub appointment_rescheduled: Option<String>,
    pub background_dark: Option<String>,
    pub text_dim: Option<String>,
    pub text_secondary: Option<String>,
}

impl AppConfig {
    fn deep_merge(&mut self, overrides: PartialAppConfig) {
        if let Some(api_server) = overrides.api_server {
            if let Some(host) = api_server.host {
                self.api_server.host = host;
            }
            if let Some(port) = api_server.port {
                self.api_server.port = port;
            }
            if let Some(database) = api_server.database {
                if let Some(url) = database.url {
                    self.api_server.database.url = url;
                }
                if let Some(max_connections) = database.max_connections {
                    self.api_server.database.max_connections = max_connections;
                }
                if let Some(min_connections) = database.min_connections {
                    self.api_server.database.min_connections = min_connections;
                }
                if let Some(connect_timeout_secs) = database.connect_timeout_secs {
                    self.api_server.database.connect_timeout_secs = connect_timeout_secs;
                }
                if let Some(idle_timeout_secs) = database.idle_timeout_secs {
                    self.api_server.database.idle_timeout_secs = idle_timeout_secs;
                }
            }
        }

        if let Some(ui) = overrides.ui {
            if let Some(theme) = ui.theme {
                self.ui.theme = theme;
            }
            if let Some(show_scrollbars) = ui.show_scrollbars {
                self.ui.show_scrollbars = show_scrollbars;
            }
            if let Some(mouse_support) = ui.mouse_support {
                self.ui.mouse_support = mouse_support;
            }
            if let Some(tick_rate_ms) = ui.tick_rate_ms {
                self.ui.tick_rate_ms = tick_rate_ms;
            }
        }

        if let Some(calendar) = overrides.calendar {
            if let Some(min_hour) = calendar.min_hour {
                self.calendar.min_hour = min_hour;
            }
            if let Some(max_hour) = calendar.max_hour {
                self.calendar.max_hour = max_hour;
            }
            if let Some(viewport_start_hour) = calendar.viewport_start_hour {
                self.calendar.viewport_start_hour = viewport_start_hour;
            }
            if let Some(viewport_end_hour) = calendar.viewport_end_hour {
                self.calendar.viewport_end_hour = viewport_end_hour;
            }
        }

        if let Some(api_client) = overrides.api_client {
            if let Some(base_url) = api_client.base_url {
                self.api_client.base_url = base_url;
            }
        }

        if let Some(logging) = overrides.logging {
            if let Some(log_file) = logging.log_file {
                self.logging.log_file = log_file;
            }
            if let Some(level) = logging.level {
                self.logging.level = level;
            }
        }

        if let Some(session) = overrides.session {
            if let Some(timeout_secs) = session.timeout_secs {
                self.session.timeout_secs = timeout_secs;
            }
        }

        if let Some(cache) = overrides.cache {
            if let Some(default_ttl_secs) = cache.default_ttl_secs {
                self.cache.default_ttl_secs = default_ttl_secs;
            }
            if let Some(key_prefix) = cache.key_prefix {
                self.cache.key_prefix = key_prefix;
            }
            if let Some(circuit_breaker) = cache.circuit_breaker {
                if let Some(failure_threshold) = circuit_breaker.failure_threshold {
                    self.cache.circuit_breaker.failure_threshold = failure_threshold;
                }
                if let Some(open_duration_secs) = circuit_breaker.open_duration_secs {
                    self.cache.circuit_breaker.open_duration_secs = open_duration_secs;
                }
            }
            if let Some(stampede) = cache.stampede {
                if let Some(default_ttl_secs) = stampede.default_ttl_secs {
                    self.cache.stampede.default_ttl_secs = default_ttl_secs;
                }
                if let Some(retry_attempts) = stampede.retry_attempts {
                    self.cache.stampede.retry_attempts = retry_attempts;
                }
                if let Some(retry_delay_ms) = stampede.retry_delay_ms {
                    self.cache.stampede.retry_delay_ms = retry_delay_ms;
                }
            }
            if let Some(entity_ttl) = cache.entity_ttl {
                if let Some(patient_secs) = entity_ttl.patient_secs {
                    self.cache.entity_ttl.patient_secs = patient_secs;
                }
                if let Some(search_secs) = entity_ttl.search_secs {
                    self.cache.entity_ttl.search_secs = search_secs;
                }
                if let Some(appointment_secs) = entity_ttl.appointment_secs {
                    self.cache.entity_ttl.appointment_secs = appointment_secs;
                }
            }
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.api_server.host.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "app.api_server.host cannot be empty".to_string(),
            ));
        }
        if self.api_server.port == 0 {
            return Err(ConfigError::Invalid(
                "app.api_server.port must be > 0".to_string(),
            ));
        }
        if self.api_server.database.url.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "app.api_server.database.url cannot be empty".to_string(),
            ));
        }
        if self.calendar.viewport_start_hour < self.calendar.min_hour {
            return Err(ConfigError::Invalid(
                "app.calendar.viewport_start_hour must be >= app.calendar.min_hour".to_string(),
            ));
        }
        if self.calendar.viewport_end_hour > self.calendar.max_hour {
            return Err(ConfigError::Invalid(
                "app.calendar.viewport_end_hour must be <= app.calendar.max_hour".to_string(),
            ));
        }
        if self.calendar.viewport_start_hour >= self.calendar.viewport_end_hour {
            return Err(ConfigError::Invalid(
                "app.calendar.viewport_start_hour must be < app.calendar.viewport_end_hour"
                    .to_string(),
            ));
        }
        if self.api_client.base_url.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "app.api_client.base_url cannot be empty".to_string(),
            ));
        }
        if self.logging.log_file.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "app.logging.log_file cannot be empty".to_string(),
            ));
        }
        if self.logging.level.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "app.logging.level cannot be empty".to_string(),
            ));
        }
        if self.session.timeout_secs == 0 {
            return Err(ConfigError::Invalid(
                "app.session.timeout_secs must be > 0".to_string(),
            ));
        }
        if self.cache.default_ttl_secs == 0
            || self.cache.circuit_breaker.failure_threshold == 0
            || self.cache.circuit_breaker.open_duration_secs == 0
            || self.cache.stampede.default_ttl_secs == 0
            || self.cache.stampede.retry_attempts == 0
            || self.cache.entity_ttl.patient_secs == 0
            || self.cache.entity_ttl.search_secs == 0
            || self.cache.entity_ttl.appointment_secs == 0
        {
            return Err(ConfigError::Invalid(
                "app.cache values must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

impl AllergyConfig {
    fn deep_merge(&mut self, overrides: PartialAllergyConfig) {
        if let Some(allergy_types) = overrides.allergy_types {
            self.allergy_types.extend(allergy_types);
        }
        if let Some(severities) = overrides.severities {
            self.severities.extend(severities);
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_enabled_options("allergies.allergy_types", &self.allergy_types)?;
        validate_enabled_options("allergies.severities", &self.severities)
    }
}

impl AppointmentConfig {
    fn deep_merge(&mut self, overrides: PartialAppointmentConfig) {
        if let Some(types) = overrides.types {
            for (key, partial) in types {
                if let Some(existing) = self.types.get_mut(&key) {
                    if let Some(label) = partial.label {
                        existing.label = label;
                    }
                    if let Some(abbreviation) = partial.abbreviation {
                        existing.abbreviation = abbreviation;
                    }
                    if let Some(duration_minutes) = partial.duration_minutes {
                        existing.duration_minutes = duration_minutes;
                    }
                    if let Some(enabled) = partial.enabled {
                        existing.enabled = enabled;
                    }
                } else {
                    self.types.insert(
                        key.clone(),
                        AppointmentTypeOption {
                            label: partial.label.unwrap_or_else(|| key.clone()),
                            abbreviation: partial.abbreviation.unwrap_or_else(|| {
                                key.chars().take(3).collect::<String>().to_uppercase()
                            }),
                            duration_minutes: partial.duration_minutes.unwrap_or(15),
                            enabled: partial.enabled.unwrap_or(true),
                        },
                    );
                }
            }
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.types.is_empty() {
            return Err(ConfigError::Invalid(
                "appointments.types cannot be empty".to_string(),
            ));
        }
        if !self.types.values().any(|v| v.enabled) {
            return Err(ConfigError::Invalid(
                "appointments.types must have at least one enabled option".to_string(),
            ));
        }

        for (key, option) in &self.types {
            if option.label.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "appointments.types.{key}.label cannot be empty"
                )));
            }
            if option.abbreviation.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "appointments.types.{key}.abbreviation cannot be empty"
                )));
            }
            if option.duration_minutes == 0 {
                return Err(ConfigError::Invalid(format!(
                    "appointments.types.{key}.duration_minutes must be > 0"
                )));
            }
        }

        Ok(())
    }
}

impl ClinicalConfig {
    fn deep_merge(&mut self, overrides: PartialClinicalConfig) {
        if let Some(condition_status) = overrides.condition_status {
            self.condition_status.extend(condition_status);
        }
        if let Some(severity) = overrides.severity {
            self.severity.extend(severity);
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_enabled_options("clinical.condition_status", &self.condition_status)?;
        validate_enabled_options("clinical.severity", &self.severity)
    }
}

impl SocialHistoryConfig {
    fn deep_merge(&mut self, overrides: PartialSocialHistoryConfig) {
        if let Some(smoking_status) = overrides.smoking_status {
            self.smoking_status.extend(smoking_status);
        }
        if let Some(alcohol_status) = overrides.alcohol_status {
            self.alcohol_status.extend(alcohol_status);
        }
        if let Some(exercise_frequency) = overrides.exercise_frequency {
            self.exercise_frequency.extend(exercise_frequency);
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_enabled_options("social_history.smoking_status", &self.smoking_status)?;
        validate_enabled_options("social_history.alcohol_status", &self.alcohol_status)?;
        validate_enabled_options(
            "social_history.exercise_frequency",
            &self.exercise_frequency,
        )
    }
}

impl PatientConfig {
    fn deep_merge(&mut self, overrides: PartialPatientConfig) {
        if let Some(gender) = overrides.gender {
            self.gender.extend(gender);
        }
        if let Some(concession_type) = overrides.concession_type {
            self.concession_type.extend(concession_type);
        }
        if let Some(atsi_status) = overrides.atsi_status {
            self.atsi_status.extend(atsi_status);
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_enabled_options("patient.gender", &self.gender)?;
        validate_enabled_options("patient.concession_type", &self.concession_type)?;
        validate_enabled_options("patient.atsi_status", &self.atsi_status)
    }
}

impl ThemeConfig {
    fn deep_merge(&mut self, overrides: PartialThemeConfig) {
        if let Some(dark) = overrides.dark {
            merge_palette(&mut self.dark, dark);
        }
        if let Some(light) = overrides.light {
            merge_palette(&mut self.light, light);
        }
        if let Some(high_contrast) = overrides.high_contrast {
            merge_palette(&mut self.high_contrast, high_contrast);
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_palette("theme.dark", &self.dark)?;
        validate_palette("theme.light", &self.light)?;
        validate_palette("theme.high_contrast", &self.high_contrast)
    }
}

fn default_min_terminal_width() -> u16 {
    80
}

fn default_min_terminal_height() -> u16 {
    24
}

fn merge_palette(palette: &mut ColorPalette, partial: PartialColorPalette) {
    if let Some(value) = partial.primary {
        palette.primary = value;
    }
    if let Some(value) = partial.secondary {
        palette.secondary = value;
    }
    if let Some(value) = partial.background {
        palette.background = value;
    }
    if let Some(value) = partial.foreground {
        palette.foreground = value;
    }
    if let Some(value) = partial.error {
        palette.error = value;
    }
    if let Some(value) = partial.success {
        palette.success = value;
    }
    if let Some(value) = partial.warning {
        palette.warning = value;
    }
    if let Some(value) = partial.info {
        palette.info = value;
    }
    if let Some(value) = partial.border {
        palette.border = value;
    }
    if let Some(value) = partial.selected {
        palette.selected = value;
    }
    if let Some(value) = partial.highlight {
        palette.highlight = value;
    }
    if let Some(value) = partial.disabled {
        palette.disabled = value;
    }
    if let Some(value) = partial.scrollbar_bg {
        palette.scrollbar_bg = value;
    }
    if let Some(value) = partial.scrollbar_thumb {
        palette.scrollbar_thumb = value;
    }
    if let Some(value) = partial.appointment_scheduled {
        palette.appointment_scheduled = value;
    }
    if let Some(value) = partial.appointment_confirmed {
        palette.appointment_confirmed = value;
    }
    if let Some(value) = partial.appointment_arrived {
        palette.appointment_arrived = value;
    }
    if let Some(value) = partial.appointment_in_progress {
        palette.appointment_in_progress = value;
    }
    if let Some(value) = partial.appointment_completed {
        palette.appointment_completed = value;
    }
    if let Some(value) = partial.appointment_cancelled {
        palette.appointment_cancelled = value;
    }
    if let Some(value) = partial.appointment_dna {
        palette.appointment_dna = value;
    }
    if let Some(value) = partial.appointment_rescheduled {
        palette.appointment_rescheduled = value;
    }
    if let Some(value) = partial.background_dark {
        palette.background_dark = value;
    }
    if let Some(value) = partial.text_dim {
        palette.text_dim = value;
    }
    if let Some(value) = partial.text_secondary {
        palette.text_secondary = value;
    }
}

fn validate_palette(path: &str, palette: &ColorPalette) -> Result<(), ConfigError> {
    let fields = [
        (&palette.primary, "primary"),
        (&palette.secondary, "secondary"),
        (&palette.background, "background"),
        (&palette.foreground, "foreground"),
        (&palette.error, "error"),
        (&palette.success, "success"),
        (&palette.warning, "warning"),
        (&palette.info, "info"),
        (&palette.border, "border"),
        (&palette.selected, "selected"),
        (&palette.highlight, "highlight"),
        (&palette.disabled, "disabled"),
        (&palette.scrollbar_bg, "scrollbar_bg"),
        (&palette.scrollbar_thumb, "scrollbar_thumb"),
        (&palette.appointment_scheduled, "appointment_scheduled"),
        (&palette.appointment_confirmed, "appointment_confirmed"),
        (&palette.appointment_arrived, "appointment_arrived"),
        (&palette.appointment_in_progress, "appointment_in_progress"),
        (&palette.appointment_completed, "appointment_completed"),
        (&palette.appointment_cancelled, "appointment_cancelled"),
        (&palette.appointment_dna, "appointment_dna"),
        (&palette.appointment_rescheduled, "appointment_rescheduled"),
        (&palette.background_dark, "background_dark"),
        (&palette.text_dim, "text_dim"),
        (&palette.text_secondary, "text_secondary"),
    ];

    for (value, field_name) in fields {
        if value.trim().is_empty() {
            return Err(ConfigError::Invalid(format!(
                "{path}.{field_name} cannot be empty"
            )));
        }
    }

    Ok(())
}

fn validate_enabled_options(
    path: &str,
    options: &HashMap<String, EnumOption>,
) -> Result<(), ConfigError> {
    if options.is_empty() {
        return Err(ConfigError::Invalid(format!("{path} cannot be empty")));
    }

    if !options.values().any(|v| v.enabled) {
        return Err(ConfigError::Invalid(format!(
            "{path} must have at least one enabled option"
        )));
    }

    if options.values().any(|v| v.label.trim().is_empty()) {
        return Err(ConfigError::Invalid(format!(
            "{path} has options with empty labels"
        )));
    }

    Ok(())
}

pub fn load_app_config() -> Result<AppConfig, ConfigError> {
    let defaults = include_str!("app.toml");
    let mut config: AppConfig = toml::from_str(defaults)
        .map_err(|e| ConfigError::Invalid(format!("failed to parse app.toml defaults: {e}")))?;

    if let Ok(path) = std::env::var("APP_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::Invalid(format!("failed to read APP_CONFIG_PATH: {e}")))?;
        let overrides: PartialAppConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!("failed to parse APP_CONFIG_PATH overrides: {e}"))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_allergy_config() -> Result<AllergyConfig, ConfigError> {
    let defaults = include_str!("allergies.toml");
    let mut config: AllergyConfig = toml::from_str(defaults).map_err(|e| {
        ConfigError::Invalid(format!("failed to parse allergies.toml defaults: {e}"))
    })?;

    if let Ok(path) = std::env::var("ALLERGIES_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            ConfigError::Invalid(format!("failed to read ALLERGIES_CONFIG_PATH: {e}"))
        })?;
        let overrides: PartialAllergyConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!(
                "failed to parse ALLERGIES_CONFIG_PATH overrides: {e}"
            ))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_appointment_config() -> Result<AppointmentConfig, ConfigError> {
    let defaults = include_str!("appointments.toml");
    let mut config: AppointmentConfig = toml::from_str(defaults).map_err(|e| {
        ConfigError::Invalid(format!("failed to parse appointments.toml defaults: {e}"))
    })?;

    if let Ok(path) = std::env::var("APPOINTMENTS_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            ConfigError::Invalid(format!("failed to read APPOINTMENTS_CONFIG_PATH: {e}"))
        })?;
        let overrides: PartialAppointmentConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!(
                "failed to parse APPOINTMENTS_CONFIG_PATH overrides: {e}"
            ))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_clinical_config() -> Result<ClinicalConfig, ConfigError> {
    let defaults = include_str!("clinical.toml");
    let mut config: ClinicalConfig = toml::from_str(defaults).map_err(|e| {
        ConfigError::Invalid(format!("failed to parse clinical.toml defaults: {e}"))
    })?;

    if let Ok(path) = std::env::var("CLINICAL_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            ConfigError::Invalid(format!("failed to read CLINICAL_CONFIG_PATH: {e}"))
        })?;
        let overrides: PartialClinicalConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!(
                "failed to parse CLINICAL_CONFIG_PATH overrides: {e}"
            ))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_social_history_config() -> Result<SocialHistoryConfig, ConfigError> {
    let defaults = include_str!("social_history.toml");
    let mut config: SocialHistoryConfig = toml::from_str(defaults).map_err(|e| {
        ConfigError::Invalid(format!("failed to parse social_history.toml defaults: {e}"))
    })?;

    if let Ok(path) = std::env::var("SOCIAL_HISTORY_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            ConfigError::Invalid(format!("failed to read SOCIAL_HISTORY_CONFIG_PATH: {e}"))
        })?;
        let overrides: PartialSocialHistoryConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!(
                "failed to parse SOCIAL_HISTORY_CONFIG_PATH overrides: {e}"
            ))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_patient_config() -> Result<PatientConfig, ConfigError> {
    let defaults = include_str!("patient.toml");
    let mut config: PatientConfig = toml::from_str(defaults)
        .map_err(|e| ConfigError::Invalid(format!("failed to parse patient.toml defaults: {e}")))?;

    if let Ok(path) = std::env::var("PATIENT_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            ConfigError::Invalid(format!("failed to read PATIENT_CONFIG_PATH: {e}"))
        })?;
        let overrides: PartialPatientConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!(
                "failed to parse PATIENT_CONFIG_PATH overrides: {e}"
            ))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_theme_config() -> Result<ThemeConfig, ConfigError> {
    let defaults = include_str!("theme.toml");
    let mut config: ThemeConfig = toml::from_str(defaults)
        .map_err(|e| ConfigError::Invalid(format!("failed to parse theme.toml defaults: {e}")))?;

    if let Ok(path) = std::env::var("THEME_CONFIG_PATH") {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::Invalid(format!("failed to read THEME_CONFIG_PATH: {e}")))?;
        let overrides: PartialThemeConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!("failed to parse THEME_CONFIG_PATH overrides: {e}"))
        })?;
        config.deep_merge(overrides);
    }

    config.validate()?;
    Ok(config)
}

pub fn load_practice_config() -> Result<PracticeConfig, ConfigError> {
    let defaults = include_str!("practice.toml");
    let config: PracticeConfig = toml::from_str(defaults).map_err(|e| {
        ConfigError::Invalid(format!("failed to parse practice.toml defaults: {e}"))
    })?;

    if let Ok(path) = std::env::var("PRACTICE_CONFIG_PATH") {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let overrides: PracticeConfig = toml::from_str(&content).map_err(|e| {
                    ConfigError::Invalid(format!(
                        "failed to parse PRACTICE_CONFIG_PATH overrides: {e}"
                    ))
                })?;
                return Ok(overrides);
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(config),
            Err(err) => {
                return Err(ConfigError::Invalid(format!(
                    "failed to read PRACTICE_CONFIG_PATH: {err}"
                )))
            }
        }
    }

    Ok(config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub allergies: AllergyConfig,
    pub appointments: AppointmentConfig,
    pub clinical: ClinicalConfig,
    pub social_history: SocialHistoryConfig,
    pub patient: PatientConfig,
    pub theme: ThemeConfig,
    pub practice: PracticeConfig,
    pub healthcare: HealthcareConfig,
    pub forms: FormConfig,
    pub encryption_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();

        let mut app_config = load_app_config()?;

        if let Ok(host) = std::env::var("API_HOST") {
            app_config.api_server.host = host;
        }
        if let Ok(port_str) = std::env::var("API_PORT") {
            if let Ok(port) = port_str.parse() {
                app_config.api_server.port = port;
            }
        }
        if let Ok(db_url) = std::env::var("API_DATABASE_URL") {
            app_config.api_server.database.url = db_url;
        }

        Ok(Self {
            app: app_config,
            allergies: load_allergy_config()?,
            appointments: load_appointment_config()?,
            clinical: load_clinical_config()?,
            social_history: load_social_history_config()?,
            patient: load_patient_config()?,
            theme: load_theme_config()?,
            practice: load_practice_config()?,
            healthcare: HealthcareConfig::load()?,
            forms: FormConfig::load()?,
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .map_err(|_| ConfigError::MissingEncryptionKey)?,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            allergies: AllergyConfig::default(),
            appointments: AppointmentConfig::default(),
            clinical: ClinicalConfig::default(),
            social_history: SocialHistoryConfig::default(),
            patient: PatientConfig::default(),
            theme: ThemeConfig::default(),
            practice: PracticeConfig::default(),
            healthcare: HealthcareConfig::default(),
            forms: FormConfig::default(),
            encryption_key: String::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing encryption key - set ENCRYPTION_KEY environment variable")]
    MissingEncryptionKey,

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Healthcare configuration error: {0}")]
    Healthcare(String),

    #[error("Forms configuration error: {0}")]
    Forms(String),
}

impl From<crate::healthcare::HealthcareConfigError> for ConfigError {
    fn from(err: crate::healthcare::HealthcareConfigError) -> Self {
        ConfigError::Healthcare(err.to_string())
    }
}

impl From<crate::forms::FormConfigError> for ConfigError {
    fn from(err: crate::forms::FormConfigError) -> Self {
        ConfigError::Forms(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_new_embedded_configs() {
        temp_env::with_vars(
            [
                ("APP_CONFIG_PATH", None::<&str>),
                ("ALLERGIES_CONFIG_PATH", None::<&str>),
                ("APPOINTMENTS_CONFIG_PATH", None::<&str>),
                ("CLINICAL_CONFIG_PATH", None::<&str>),
                ("SOCIAL_HISTORY_CONFIG_PATH", None::<&str>),
                ("PATIENT_CONFIG_PATH", None::<&str>),
                ("THEME_CONFIG_PATH", None::<&str>),
                ("PRACTICE_CONFIG_PATH", None::<&str>),
            ],
            || {
                assert!(load_app_config().is_ok());
                assert!(load_allergy_config().is_ok());
                assert!(load_appointment_config().is_ok());
                assert!(load_clinical_config().is_ok());
                assert!(load_social_history_config().is_ok());
                assert!(load_patient_config().is_ok());
                assert!(load_theme_config().is_ok());
                assert!(load_practice_config().is_ok());
            },
        );
    }

    #[test]
    fn validates_at_least_one_enabled_option() {
        let mut config = AllergyConfig::default();
        config.allergy_types.insert(
            "drug".to_string(),
            EnumOption {
                label: "Drug".to_string(),
                enabled: false,
            },
        );
        config.severities.insert(
            "mild".to_string(),
            EnumOption {
                label: "Mild".to_string(),
                enabled: false,
            },
        );

        assert!(config.validate().is_err());
    }

    #[test]
    fn default_config_has_new_fields() {
        let config = Config::default();
        assert!(config.allergies.allergy_types.is_empty());
        assert!(config.appointments.types.is_empty());
        assert!(config.clinical.condition_status.is_empty());
        assert!(config.social_history.smoking_status.is_empty());
        assert!(config.patient.gender.is_empty());
        assert!(config.practice.profile.name.is_empty());
    }

    #[test]
    fn config_from_env_loads_all() {
        temp_env::with_vars(
            [
                ("ENCRYPTION_KEY", Some("abcdef")),
                ("APP_CONFIG_PATH", None::<&str>),
                ("ALLERGIES_CONFIG_PATH", None::<&str>),
                ("APPOINTMENTS_CONFIG_PATH", None::<&str>),
                ("CLINICAL_CONFIG_PATH", None::<&str>),
                ("SOCIAL_HISTORY_CONFIG_PATH", None::<&str>),
                ("PATIENT_CONFIG_PATH", None::<&str>),
                ("THEME_CONFIG_PATH", None::<&str>),
                ("PRACTICE_CONFIG_PATH", None::<&str>),
                ("HEALTHCARE_CONFIG_PATH", None::<&str>),
                ("FORMS_CONFIG_PATH", None::<&str>),
            ],
            || {
                let config = Config::from_env().expect("config should load");
                assert_eq!(config.encryption_key, "abcdef");
            },
        );
    }

    #[test]
    fn test_ui_config_default() {
        let config = UiConfig::default();
        assert_eq!(config.theme, "dark");
        assert!(config.show_scrollbars);
        assert!(config.mouse_support);
        assert_eq!(config.tick_rate_ms, 16);
        assert_eq!(config.min_terminal_width, 80);
        assert_eq!(config.min_terminal_height, 24);
    }

    #[test]
    fn test_ui_config_min_terminal_defaults() {
        let toml_str = r#"
            theme = "light"
            show_scrollbars = false
            mouse_support = false
            tick_rate_ms = 32
        "#;
        let config: UiConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.min_terminal_width, 80);
        assert_eq!(config.min_terminal_height, 24);
    }

    #[test]
    fn test_ui_config_min_terminal_custom() {
        let toml_str = r#"
            theme = "light"
            show_scrollbars = false
            mouse_support = false
            tick_rate_ms = 32
            min_terminal_width = 120
            min_terminal_height = 40
        "#;
        let config: UiConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.min_terminal_width, 120);
        assert_eq!(config.min_terminal_height, 40);
    }
}
