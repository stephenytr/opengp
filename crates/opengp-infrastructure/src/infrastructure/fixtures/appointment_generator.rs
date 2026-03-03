use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use uuid::Uuid;

use opengp_domain::domain::appointment::{Appointment, AppointmentStatus, AppointmentType};

/// Configuration for appointment generation
///
/// Controls how many appointments are generated and their characteristics.
#[derive(Debug, Clone)]
pub struct AppointmentGeneratorConfig {
    /// Number of appointments to generate (for simple generation mode)
    pub count: usize,
    /// Percentage of appointments that should be in the future (0.0-1.0)
    pub future_percentage: f32,
    /// Percentage of appointments that should be confirmed (0.0-1.0)
    pub confirmed_percentage: f32,
    /// Percentage of appointments that should be urgent (0.0-1.0)
    pub urgent_percentage: f32,
    /// Percentage of appointments with notes (0.0-1.0)
    pub notes_percentage: f32,
    /// Percentage of time slots to fill with appointments (0.0-1.0, default 0.60)
    pub fill_rate: f32,
    /// Start date for schedule generation (inclusive)
    pub start_date: Option<DateTime<Utc>>,
    /// End date for schedule generation (inclusive)
    pub end_date: Option<DateTime<Utc>>,
    /// Duration of each time slot in minutes (default 15)
    pub slot_duration_minutes: i64,
    /// Business hours start hour (default 9)
    pub business_hours_start: u8,
    /// Business hours end hour (default 17)
    pub business_hours_end: u8,
    /// Pool of patient IDs to use for appointments
    pub patient_ids: Option<Vec<Uuid>>,
    /// Pool of practitioner IDs to use for appointments
    pub practitioner_ids: Option<Vec<Uuid>>,
    /// Whether to exclude weekends (default true)
    pub exclude_weekends: bool,
    /// Whether to exclude lunch hour (12pm-1pm, default false)
    pub exclude_lunch_hour: bool,
}

impl Default for AppointmentGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            future_percentage: 0.70,
            confirmed_percentage: 0.60,
            urgent_percentage: 0.10,
            notes_percentage: 0.40,
            fill_rate: 0.60,
            start_date: None,
            end_date: None,
            slot_duration_minutes: 15,
            business_hours_start: 9,
            business_hours_end: 17,
            patient_ids: None,
            practitioner_ids: None,
            exclude_weekends: true,
            exclude_lunch_hour: false,
        }
    }
}

/// Statistics about generated schedule
#[derive(Debug, Clone)]
pub struct GenerationStats {
    /// Total number of time slots generated
    pub total_slots: usize,
    /// Number of slots filled with appointments
    pub filled_slots: usize,
    /// Number of empty slots
    pub available_slots: usize,
    /// Actual fill rate (filled_slots / total_slots)
    pub actual_fill_rate: f32,
    /// Distribution of appointments by status
    pub by_status: HashMap<AppointmentStatus, usize>,
    /// Distribution of appointments by type
    pub by_type: HashMap<AppointmentType, usize>,
    /// Number of unique patients used
    pub unique_patients: usize,
    /// Number of unique practitioners used
    pub unique_practitioners: usize,
}

/// Represents a time slot in the schedule
#[derive(Debug, Clone)]
struct TimeSlot {
    start_time: DateTime<Utc>,
    duration: Duration,
    _filled: bool,
}

/// Generator for realistic appointment test data
///
/// Creates appointments with realistic time slots (9am-5pm weekdays),
/// various types, and statuses. Supports configurable practitioner and patient IDs.
///
/// # Example
///
/// ```
/// use opengp_infrastructure::infrastructure::fixtures::{AppointmentGenerator, AppointmentGeneratorConfig};
/// use chrono::{Duration, Utc};
///
/// let config = AppointmentGeneratorConfig {
///     fill_rate: 0.60,
///     start_date: Some(Utc::now()),
///     end_date: Some(Utc::now() + Duration::days(31)),
///     ..Default::default()
/// };
///
/// let mut generator = AppointmentGenerator::new(config);
/// let (appointments, stats) = generator.generate_schedule();
/// ```
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

    /// Generate a vector of appointments using simple mode
    ///
    /// This maintains backward compatibility with the original API.
    /// Generates `count` appointments with random times.
    pub fn generate(&mut self) -> Vec<Appointment> {
        (0..self.config.count)
            .map(|_| self.generate_appointment())
            .collect()
    }

    /// Generate a schedule with time slots and fill rate
    ///
    /// Generates a realistic schedule across the configured date range,
    /// filling slots based on the fill_rate configuration.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Vector of generated appointments
    /// - Statistics about the generation
    pub fn generate_schedule(&mut self) -> (Vec<Appointment>, GenerationStats) {
        let slots = self.generate_time_slots();
        let total_slots = slots.len();
        let slots_to_fill = (total_slots as f32 * self.config.fill_rate) as usize;

        let mut appointments = Vec::with_capacity(slots_to_fill);
        let mut by_status: HashMap<AppointmentStatus, usize> = HashMap::new();
        let mut by_type: HashMap<AppointmentType, usize> = HashMap::new();
        let mut used_patients: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut used_practitioners: std::collections::HashSet<Uuid> =
            std::collections::HashSet::new();

        // Randomly select which slots to fill
        let mut slot_indices: Vec<usize> = (0..total_slots).collect();
        slot_indices.shuffle(&mut self.rng);
        let selected_indices: std::collections::HashSet<usize> =
            slot_indices.into_iter().take(slots_to_fill).collect();

        for (idx, slot) in slots.iter().enumerate() {
            if !selected_indices.contains(&idx) {
                continue; // Skip unfilled slots
            }

            let appointment = self.generate_appointment_for_slot(slot);

            // Track statistics
            *by_status.entry(appointment.status).or_insert(0) += 1;
            *by_type.entry(appointment.appointment_type).or_insert(0) += 1;
            used_patients.insert(appointment.patient_id);
            used_practitioners.insert(appointment.practitioner_id);

            appointments.push(appointment);
        }

        let stats = GenerationStats {
            total_slots,
            filled_slots: appointments.len(),
            available_slots: total_slots - appointments.len(),
            actual_fill_rate: appointments.len() as f32 / total_slots as f32,
            by_status,
            by_type,
            unique_patients: used_patients.len(),
            unique_practitioners: used_practitioners.len(),
        };

        (appointments, stats)
    }

    /// Generate all possible time slots within the configured date range
    fn generate_time_slots(&mut self) -> Vec<TimeSlot> {
        let mut slots = Vec::new();

        let start_date = self
            .config
            .start_date
            .unwrap_or_else(|| Utc::now() - Duration::days(7));
        let end_date = self
            .config
            .end_date
            .unwrap_or_else(|| Utc::now() + Duration::days(7));

        let slot_duration = Duration::minutes(self.config.slot_duration_minutes);
        let start_hour = self.config.business_hours_start as u32;
        let end_hour = self.config.business_hours_end as u32;

        let mut current_date = start_date.date_naive();
        let end_naive = end_date.date_naive();

        while current_date <= end_naive {
            let weekday = current_date.weekday();

            // Skip weekends if configured
            if self.config.exclude_weekends && (weekday == Weekday::Sat || weekday == Weekday::Sun)
            {
                current_date += Duration::days(1);
                continue;
            }

            // Generate slots for each hour in business hours
            for hour in start_hour..end_hour {
                // Skip lunch hour if configured
                if self.config.exclude_lunch_hour && hour == 12 {
                    continue;
                }

                // Generate slots based on slot duration
                let slots_per_hour = 60 / self.config.slot_duration_minutes as u32;
                for slot_idx in 0..slots_per_hour {
                    let minute = slot_idx * self.config.slot_duration_minutes as u32;

                    if let Some(start_time) = current_date.and_hms_opt(hour, minute, 0) {
                        slots.push(TimeSlot {
                            start_time: start_time.and_utc(),
                            duration: slot_duration,
                            _filled: false,
                        });
                    }
                }
            }

            current_date += Duration::days(1);
        }

        slots
    }

    /// Generate an appointment for a specific time slot
    fn generate_appointment_for_slot(&mut self, slot: &TimeSlot) -> Appointment {
        let patient_id = self.random_patient_id();
        let practitioner_id = self.random_practitioner_id();
        let appointment_type = self.random_appointment_type();

        let mut appointment = Appointment::new(
            patient_id,
            practitioner_id,
            slot.start_time,
            slot.duration,
            appointment_type,
            None,
        );

        // Set status based on appointment time relative to now
        appointment.status = self.status_for_time(slot.start_time);

        // Set confirmation based on status and configured percentage
        appointment.confirmed = match appointment.status {
            AppointmentStatus::Scheduled => {
                self.rng.gen_bool(self.config.confirmed_percentage as f64)
            }
            AppointmentStatus::Confirmed
            | AppointmentStatus::Arrived
            | AppointmentStatus::InProgress
            | AppointmentStatus::Completed => true,
            _ => false,
        };

        // Set urgency
        appointment.is_urgent = self.rng.gen_bool(self.config.urgent_percentage as f64);

        // Set reason
        appointment.reason = Some(self.random_reason());

        // Set notes
        if self.rng.gen_bool(self.config.notes_percentage as f64) {
            appointment.notes = Some(self.random_notes());
        }

        // Set reminder sent for appointments that are confirmed or in progress
        if appointment.confirmed {
            appointment.reminder_sent = self.rng.gen_bool(0.85);
        }

        appointment
    }

    /// Select a random patient ID from pool or generate new
    fn random_patient_id(&mut self) -> Uuid {
        if let Some(ref patients) = self.config.patient_ids {
            if !patients.is_empty() {
                return *patients
                    .choose(&mut self.rng)
                    .expect("patient pool not empty");
            }
        }
        Uuid::new_v4()
    }

    /// Select a random practitioner ID from pool or generate new
    fn random_practitioner_id(&mut self) -> Uuid {
        if let Some(ref practitioners) = self.config.practitioner_ids {
            if !practitioners.is_empty() {
                return *practitioners
                    .choose(&mut self.rng)
                    .expect("practitioner pool not empty");
            }
        }
        Uuid::new_v4()
    }

    /// Determine appropriate status based on appointment time
    fn status_for_time(&mut self, appointment_time: DateTime<Utc>) -> AppointmentStatus {
        let now = Utc::now();
        let today = now.date_naive();
        let appointment_date = appointment_time.date_naive();

        if appointment_time < now {
            // Past appointments
            let roll: u8 = self.rng.gen_range(0..100);
            if roll < 70 {
                AppointmentStatus::Completed
            } else if roll < 85 {
                AppointmentStatus::NoShow
            } else if roll < 95 {
                AppointmentStatus::Cancelled
            } else {
                AppointmentStatus::Rescheduled
            }
        } else if appointment_date == today {
            // Today's appointments - mix of all active statuses
            let roll: u8 = self.rng.gen_range(0..100);
            if roll < 40 {
                AppointmentStatus::Scheduled
            } else if roll < 65 {
                AppointmentStatus::Confirmed
            } else if roll < 80 {
                AppointmentStatus::Arrived
            } else if roll < 90 {
                AppointmentStatus::InProgress
            } else {
                AppointmentStatus::Completed
            }
        } else {
            // Future appointments
            if self.rng.gen_bool(0.70) {
                AppointmentStatus::Scheduled
            } else {
                AppointmentStatus::Confirmed
            }
        }
    }

    /// Generate a single appointment with random data (legacy mode)
    fn generate_appointment(&mut self) -> Appointment {
        let patient_id = self.random_patient_id();
        let practitioner_id = self.random_practitioner_id();
        let appointment_type = self.random_appointment_type();
        let start_time = self.random_start_time();
        let duration = Duration::minutes(appointment_type.default_duration_minutes());

        let mut appointment = Appointment::new(
            patient_id,
            practitioner_id,
            start_time,
            duration,
            appointment_type,
            None,
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
    use chrono::Timelike;

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

    #[test]
    fn test_generate_schedule_basic() {
        let start = Utc::now();
        let end = start + Duration::days(5);

        let config = AppointmentGeneratorConfig {
            fill_rate: 0.60,
            start_date: Some(start),
            end_date: Some(end),
            slot_duration_minutes: 15,
            business_hours_start: 9,
            business_hours_end: 17,
            exclude_weekends: true,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, stats) = generator.generate_schedule();

        assert!(stats.total_slots > 0, "Should have generated time slots");
        assert_eq!(
            stats.filled_slots,
            appointments.len(),
            "Filled slots should match appointments count"
        );
        assert!(
            stats.actual_fill_rate >= 0.55 && stats.actual_fill_rate <= 0.65,
            "Fill rate should be approximately 60%, got {}%",
            stats.actual_fill_rate * 100.0
        );
    }

    #[test]
    fn test_different_slot_durations() {
        let start = Utc::now();
        let end = start;

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            slot_duration_minutes: 30,
            business_hours_start: 9,
            business_hours_end: 11,
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (_, stats_30min) = generator.generate_schedule();

        assert_eq!(
            stats_30min.total_slots, 4,
            "2 hours with 30-min slots = 4 slots (2 slots/hour * 2 hours)"
        );

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            slot_duration_minutes: 60,
            business_hours_start: 9,
            business_hours_end: 12,
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (_, stats_60min) = generator.generate_schedule();

        assert_eq!(
            stats_60min.total_slots, 3,
            "3 hours with 60-min slots = 3 slots (1 slot/hour * 3 hours)"
        );
    }

    #[test]
    fn test_generate_schedule_business_hours() {
        let start = Utc::now();
        let end = start + Duration::days(3);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            business_hours_start: 10,
            business_hours_end: 14,
            exclude_weekends: true,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, _) = generator.generate_schedule();

        for appointment in &appointments {
            let hour = appointment.start_time.hour();
            assert!(
                hour >= 10 && hour < 14,
                "Appointment hour {} should be between 10 and 14",
                hour
            );
        }
    }

    #[test]
    fn test_generate_schedule_weekend_exclusion() {
        let start = Utc::now();
        let end = start + Duration::days(7);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            exclude_weekends: true,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, _) = generator.generate_schedule();

        for appointment in &appointments {
            let weekday = appointment.start_time.weekday();
            assert!(
                weekday != Weekday::Sat && weekday != Weekday::Sun,
                "Should not have appointments on weekends"
            );
        }
    }

    #[test]
    fn test_generate_schedule_weekend_inclusion() {
        let start = Utc::now();
        let end = start + Duration::days(7);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (_, stats) = generator.generate_schedule();

        assert!(
            stats.total_slots > 0,
            "Should have generated slots including weekends"
        );
    }

    #[test]
    fn test_patient_pool_assignment() {
        let patient_ids = vec![
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
        ];

        let start = Utc::now();
        let end = start + Duration::days(3);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            patient_ids: Some(patient_ids.clone()),
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, stats) = generator.generate_schedule();

        for appointment in &appointments {
            assert!(
                patient_ids.contains(&appointment.patient_id),
                "Patient ID should be from the provided pool"
            );
        }

        assert!(
            stats.unique_patients <= patient_ids.len(),
            "Should only use patients from the pool"
        );
    }

    #[test]
    fn test_practitioner_pool_assignment() {
        let practitioner_ids = vec![
            Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
        ];

        let start = Utc::now();
        let end = start + Duration::days(2);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            practitioner_ids: Some(practitioner_ids.clone()),
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, _) = generator.generate_schedule();

        for appointment in &appointments {
            assert!(
                practitioner_ids.contains(&appointment.practitioner_id),
                "Practitioner ID should be from the provided pool"
            );
        }
    }

    #[test]
    fn test_status_distribution_by_time() {
        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let tomorrow = now + Duration::days(1);

        let config = AppointmentGeneratorConfig {
            start_date: Some(yesterday),
            end_date: Some(yesterday + Duration::hours(8)),
            fill_rate: 1.0,
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (past_appointments, _) = generator.generate_schedule();

        for appt in &past_appointments {
            assert!(
                matches!(
                    appt.status,
                    AppointmentStatus::Completed
                        | AppointmentStatus::NoShow
                        | AppointmentStatus::Cancelled
                        | AppointmentStatus::Rescheduled
                ),
                "Past appointment should have terminal status, got {:?}",
                appt.status
            );
        }

        let config = AppointmentGeneratorConfig {
            start_date: Some(tomorrow),
            end_date: Some(tomorrow + Duration::hours(8)),
            fill_rate: 1.0,
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (future_appointments, _) = generator.generate_schedule();

        for appt in &future_appointments {
            assert!(
                matches!(
                    appt.status,
                    AppointmentStatus::Scheduled | AppointmentStatus::Confirmed
                ),
                "Future appointment should be Scheduled or Confirmed, got {:?}",
                appt.status
            );
        }
    }

    #[test]
    fn test_lunch_hour_exclusion() {
        let start = Utc::now();
        let end = start + Duration::days(1);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            business_hours_start: 9,
            business_hours_end: 14,
            exclude_lunch_hour: true,
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, _) = generator.generate_schedule();

        for appointment in &appointments {
            let hour = appointment.start_time.hour();
            assert_ne!(
                hour, 12,
                "Should not have appointments during lunch hour (12pm)"
            );
        }
    }

    #[test]
    fn test_stats_accuracy() {
        let start = Utc::now();
        let end = start + Duration::days(5);

        let config = AppointmentGeneratorConfig {
            fill_rate: 0.75,
            start_date: Some(start),
            end_date: Some(end),
            exclude_weekends: true,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, stats) = generator.generate_schedule();

        assert_eq!(
            stats.filled_slots + stats.available_slots,
            stats.total_slots,
            "Filled + available should equal total"
        );

        let status_sum: usize = stats.by_status.values().sum();
        assert_eq!(
            status_sum,
            appointments.len(),
            "Status counts should sum to total appointments"
        );

        let type_sum: usize = stats.by_type.values().sum();
        assert_eq!(
            type_sum,
            appointments.len(),
            "Type counts should sum to total appointments"
        );
    }

    #[test]
    fn test_zero_fill_rate() {
        let start = Utc::now();
        let end = start + Duration::days(2);

        let config = AppointmentGeneratorConfig {
            fill_rate: 0.0,
            start_date: Some(start),
            end_date: Some(end),
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, stats) = generator.generate_schedule();

        assert!(
            appointments.is_empty(),
            "Should have no appointments with 0% fill rate"
        );
        assert_eq!(stats.filled_slots, 0);
        assert!(stats.total_slots > 0, "Should still generate slots");
        assert_eq!(stats.actual_fill_rate, 0.0);
    }

    #[test]
    fn test_full_fill_rate() {
        let start = Utc::now();
        let end = start + Duration::days(1);

        let config = AppointmentGeneratorConfig {
            fill_rate: 1.0,
            start_date: Some(start),
            end_date: Some(end),
            exclude_weekends: false,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let (appointments, stats) = generator.generate_schedule();

        assert_eq!(
            stats.filled_slots, stats.total_slots,
            "Should fill all slots with 100% fill rate"
        );
        assert_eq!(appointments.len(), stats.total_slots);
        assert!((stats.actual_fill_rate - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_backward_compatibility() {
        let config = AppointmentGeneratorConfig {
            count: 10,
            ..Default::default()
        };

        let mut generator = AppointmentGenerator::new(config);
        let appointments = generator.generate();

        assert_eq!(appointments.len(), 10);

        for appointment in &appointments {
            assert!(appointment.start_time.hour() >= 9);
            assert!(appointment.start_time.hour() < 17);
        }
    }
}
