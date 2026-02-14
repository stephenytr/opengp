use chrono::{Datelike, Duration, Utc, Weekday};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use crate::domain::appointment::{Appointment, AppointmentStatus, AppointmentType};

/// Configuration for appointment generation
///
/// Controls how many appointments are generated and their characteristics.
#[derive(Debug, Clone)]
pub struct AppointmentGeneratorConfig {
    /// Number of appointments to generate
    pub count: usize,
    /// Percentage of appointments that should be in the future (0.0-1.0)
    pub future_percentage: f32,
    /// Percentage of appointments that should be confirmed (0.0-1.0)
    pub confirmed_percentage: f32,
    /// Percentage of appointments that should be urgent (0.0-1.0)
    pub urgent_percentage: f32,
    /// Percentage of appointments with notes (0.0-1.0)
    pub notes_percentage: f32,
}

impl Default for AppointmentGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            future_percentage: 0.70,
            confirmed_percentage: 0.60,
            urgent_percentage: 0.10,
            notes_percentage: 0.40,
        }
    }
}

/// Generator for realistic appointment test data
///
/// Creates appointments with realistic time slots (9am-5pm weekdays),
/// various types, and statuses. Supports configurable practitioner and patient IDs.
pub struct AppointmentGenerator {
    config: AppointmentGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl AppointmentGenerator {
    /// Create a new appointment generator with the given configuration
    pub fn new(config: AppointmentGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a vector of appointments
    pub fn generate(&mut self) -> Vec<Appointment> {
        (0..self.config.count)
            .map(|_| self.generate_appointment())
            .collect()
    }

    /// Generate a single appointment with random data
    fn generate_appointment(&mut self) -> Appointment {
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();
        let appointment_type = self.random_appointment_type();
        let start_time = self.random_start_time();
        let duration = Duration::minutes(appointment_type.default_duration_minutes());

        let mut appointment = Appointment::new(
            patient_id,
            practitioner_id,
            start_time,
            duration,
            appointment_type,
            Some(Uuid::new_v4()),
        );

        // Set status
        appointment.status = self.random_status();

        // Set confirmation
        appointment.confirmed = self.rng.gen_bool(self.config.confirmed_percentage as f64);

        // Set urgency
        appointment.is_urgent = self.rng.gen_bool(self.config.urgent_percentage as f64);

        // Set reason
        appointment.reason = Some(self.random_reason());

        // Set notes
        if self.rng.gen_bool(self.config.notes_percentage as f64) {
            appointment.notes = Some(self.random_notes());
        }

        // Set reminder sent for past appointments
        if appointment.is_past() {
            appointment.reminder_sent = self.rng.gen_bool(0.80);
        }

        appointment
    }

    /// Generate a random appointment type
    fn random_appointment_type(&mut self) -> AppointmentType {
        let types = [
            AppointmentType::Standard,
            AppointmentType::Long,
            AppointmentType::Brief,
            AppointmentType::NewPatient,
            AppointmentType::HealthAssessment,
            AppointmentType::ChronicDiseaseReview,
            AppointmentType::MentalHealthPlan,
            AppointmentType::Immunisation,
            AppointmentType::Procedure,
            AppointmentType::Telephone,
            AppointmentType::Telehealth,
        ];

        *types
            .choose(&mut self.rng)
            .expect("appointment types not empty")
    }

    /// Generate a random appointment status
    fn random_status(&mut self) -> AppointmentStatus {
        let statuses = [
            AppointmentStatus::Scheduled,
            AppointmentStatus::Confirmed,
            AppointmentStatus::Arrived,
            AppointmentStatus::InProgress,
            AppointmentStatus::Completed,
        ];

        *statuses.choose(&mut self.rng).expect("statuses not empty")
    }

    /// Generate a random start time (9am-5pm on weekdays)
    fn random_start_time(&mut self) -> chrono::DateTime<Utc> {
        let now = Utc::now();
        let is_future = self.rng.gen_bool(self.config.future_percentage as f64);

        // Generate a date (future or past)
        let days_offset = if is_future {
            self.rng.gen_range(1..30)
        } else {
            -self.rng.gen_range(1..30)
        };

        let appointment_date = (now + Duration::days(days_offset)).date_naive();

        // Find next weekday if weekend
        let weekday = appointment_date.weekday();
        let adjusted_date = match weekday {
            Weekday::Sat => appointment_date + Duration::days(2),
            Weekday::Sun => appointment_date + Duration::days(1),
            _ => appointment_date,
        };

        // Generate time between 9am and 5pm
        let hour = self.rng.gen_range(9..17);
        let minute = self.rng.gen_range(0..60);

        adjusted_date
            .and_hms_opt(hour, minute, 0)
            .expect("valid datetime")
            .and_utc()
    }

    /// Generate a random appointment reason
    fn random_reason(&mut self) -> String {
        let reasons = [
            "General check-up",
            "Follow-up consultation",
            "Chronic disease management",
            "Mental health review",
            "Immunisation",
            "Wound care",
            "Medication review",
            "Health assessment",
            "Preventive care",
            "Acute illness",
        ];

        reasons
            .choose(&mut self.rng)
            .expect("reasons not empty")
            .to_string()
    }

    /// Generate random appointment notes
    fn random_notes(&mut self) -> String {
        let notes = [
            "Patient requested early morning slot",
            "Requires interpreter",
            "Mobility assistance needed",
            "Bring recent test results",
            "Follow-up from previous visit",
            "Complex case - allow extra time",
            "Patient has anxiety - be gentle",
            "Requires privacy for sensitive discussion",
        ];

        notes
            .choose(&mut self.rng)
            .expect("notes not empty")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_appointments() {
        let config = AppointmentGeneratorConfig {
            count: 5,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let appointments = generator.generate();

        assert_eq!(appointments.len(), 5);

        for appointment in &appointments {
            assert_ne!(appointment.patient_id, Uuid::nil());
            assert_ne!(appointment.practitioner_id, Uuid::nil());
            assert!(appointment.start_time < appointment.end_time);
            assert!(appointment.reason.is_some());
        }
    }

    #[test]
    fn test_appointment_times_are_weekday_business_hours() {
        let config = AppointmentGeneratorConfig {
            count: 20,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let appointments = generator.generate();

        for appointment in &appointments {
            let hour = appointment.start_time.hour();
            assert!(
                hour >= 9 && hour < 17,
                "Appointment hour {} not in business hours",
                hour
            );

            let weekday = appointment.start_time.weekday();
            assert!(
                weekday != Weekday::Sat && weekday != Weekday::Sun,
                "Appointment on weekend: {:?}",
                weekday
            );
        }
    }

    #[test]
    fn test_appointment_duration_matches_type() {
        let config = AppointmentGeneratorConfig {
            count: 10,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let appointments = generator.generate();

        for appointment in &appointments {
            let expected_duration = appointment.appointment_type.default_duration_minutes();
            let actual_duration = appointment.duration_minutes();
            assert_eq!(
                actual_duration, expected_duration,
                "Duration mismatch for {:?}",
                appointment.appointment_type
            );
        }
    }

    #[test]
    fn test_config_future_percentage() {
        let config = AppointmentGeneratorConfig {
            count: 20,
            future_percentage: 0.80,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let appointments = generator.generate();

        let future_count = appointments.iter().filter(|a| !a.is_past()).count();
        assert!(future_count > 10, "Expected mostly future appointments");
    }

    #[test]
    fn test_config_confirmed_percentage() {
        let config = AppointmentGeneratorConfig {
            count: 20,
            confirmed_percentage: 0.80,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let appointments = generator.generate();

        let confirmed_count = appointments.iter().filter(|a| a.confirmed).count();
        assert!(
            confirmed_count > 10,
            "Expected mostly confirmed appointments"
        );
    }
}
