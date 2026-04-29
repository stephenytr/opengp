use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use opengp_domain::domain::appointment::{Appointment, AppointmentStatus, AppointmentType};

#[derive(Debug, Clone)]
pub struct AppointmentHistoryGeneratorConfig {
    pub min_appointments_per_patient: usize,
    pub max_appointments_per_patient: usize,
    pub percentage_completed: f32,
    pub percentage_no_show: f32,
    pub percentage_cancelled: f32,
    pub percentage_with_notes: f32,
    pub lookback_days: i64,
}

impl Default for AppointmentHistoryGeneratorConfig {
    fn default() -> Self {
        Self {
            min_appointments_per_patient: 1,
            max_appointments_per_patient: 8,
            percentage_completed: 0.70,
            percentage_no_show: 0.10,
            percentage_cancelled: 0.15,
            percentage_with_notes: 0.40,
            lookback_days: 365,
        }
    }
}

pub struct AppointmentHistoryGenerator {
    config: AppointmentHistoryGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl AppointmentHistoryGenerator {
    pub fn new(config: AppointmentHistoryGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_for_patient(
        &mut self,
        patient_id: Uuid,
        practitioner_ids: Vec<Uuid>,
    ) -> Vec<Appointment> {
        let count = if self.config.min_appointments_per_patient
            >= self.config.max_appointments_per_patient
        {
            self.config.min_appointments_per_patient
        } else {
            self.rng.gen_range(
                self.config.min_appointments_per_patient..=self.config.max_appointments_per_patient,
            )
        };

        (0..count)
            .map(|_| {
                let appointment_type = self.random_appointment_type();
                let start_time = self.random_past_start_time();
                let duration = Duration::minutes(appointment_type.default_duration_minutes());
                let practitioner_id = practitioner_ids
                    .choose(&mut self.rng)
                    .copied()
                    .unwrap_or_else(Uuid::new_v4);

                let mut appointment = Appointment::new(
                    patient_id,
                    practitioner_id,
                    start_time,
                    duration,
                    appointment_type,
                    None,
                );

                appointment.status = self.random_status();
                appointment.reason = Some(self.reason_for_type(appointment_type));
                appointment.confirmed = matches!(
                    appointment.status,
                    AppointmentStatus::Completed | AppointmentStatus::NoShow
                );
                appointment.reminder_sent = appointment.confirmed;

                if self
                    .rng
                    .gen_bool(self.config.percentage_with_notes.clamp(0.0, 1.0) as f64)
                {
                    appointment.notes = Some(self.random_note());
                }

                if appointment.status == AppointmentStatus::Cancelled {
                    appointment.cancellation_reason = Some("Patient unavailable".to_string());
                }

                appointment
            })
            .collect()
    }

    fn random_past_start_time(&mut self) -> chrono::DateTime<Utc> {
        let lookback_days = self.config.lookback_days.max(1);
        let max_offset_seconds = lookback_days * 24 * 60 * 60;
        let offset_seconds = if max_offset_seconds <= 3600 {
            3600
        } else {
            self.rng.gen_range(3600..=max_offset_seconds)
        };

        Utc::now() - Duration::seconds(offset_seconds)
    }

    fn random_appointment_type(&mut self) -> AppointmentType {
        let roll: u8 = self.rng.gen_range(0..100);
        match roll {
            0..=39 => AppointmentType::Standard,
            40..=54 => AppointmentType::Long,
            55..=69 => AppointmentType::Brief,
            70..=74 => AppointmentType::NewPatient,
            75..=79 => AppointmentType::HealthAssessment,
            80..=84 => AppointmentType::ChronicDiseaseReview,
            85..=89 => AppointmentType::MentalHealthPlan,
            90..=94 => AppointmentType::Procedure,
            95..=97 => AppointmentType::Telephone,
            _ => AppointmentType::Telehealth,
        }
    }

    fn random_status(&mut self) -> AppointmentStatus {
        let completed = self.config.percentage_completed.clamp(0.0, 1.0);
        let no_show = self.config.percentage_no_show.clamp(0.0, 1.0);
        let cancelled = self.config.percentage_cancelled.clamp(0.0, 1.0);

        let roll = self.rng.gen_range(0.0..1.0);
        if roll < completed {
            AppointmentStatus::Completed
        } else if roll < completed + no_show {
            AppointmentStatus::NoShow
        } else if roll < completed + no_show + cancelled {
            AppointmentStatus::Cancelled
        } else {
            AppointmentStatus::Rescheduled
        }
    }

    fn reason_for_type(&self, appointment_type: AppointmentType) -> String {
        match appointment_type {
            AppointmentType::Standard => "General consultation".to_string(),
            AppointmentType::Long => "Complex health concerns".to_string(),
            AppointmentType::Brief => "Quick review and scripts".to_string(),
            AppointmentType::NewPatient => "Initial patient assessment".to_string(),
            AppointmentType::HealthAssessment => "Preventive health check".to_string(),
            AppointmentType::ChronicDiseaseReview => "Chronic condition management".to_string(),
            AppointmentType::MentalHealthPlan => "Mental health care plan review".to_string(),
            AppointmentType::Procedure => "Minor procedure".to_string(),
            AppointmentType::Telephone => "Telephone follow-up".to_string(),
            AppointmentType::Telehealth => "Telehealth review".to_string(),
            AppointmentType::Immunisation => "Vaccination".to_string(),
            AppointmentType::HomeVisit => "Home visit".to_string(),
            AppointmentType::Emergency => "Urgent review".to_string(),
        }
    }

    fn random_note(&mut self) -> String {
        let notes = [
            "Follow-up advised in 2 weeks",
            "Patient requested copy of care plan",
            "Discussed ongoing symptom monitoring",
            "Reviewed medications and adherence",
            "Provided lifestyle management guidance",
        ];

        notes
            .choose(&mut self.rng)
            .copied()
            .unwrap_or("Follow-up advised in 2 weeks")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_past_appointments() {
        let config = AppointmentHistoryGeneratorConfig {
            min_appointments_per_patient: 50,
            max_appointments_per_patient: 50,
            ..Default::default()
        };
        let mut generator = AppointmentHistoryGenerator::new(config);

        let patient_id = Uuid::new_v4();
        let practitioner_ids = vec![Uuid::new_v4()];
        let appointments = generator.generate_for_patient(patient_id, practitioner_ids);

        let now = Utc::now();
        assert!(appointments
            .iter()
            .all(|appointment| appointment.start_time < now));
    }

    #[test]
    fn test_status_distribution() {
        let config = AppointmentHistoryGeneratorConfig {
            min_appointments_per_patient: 4000,
            max_appointments_per_patient: 4000,
            percentage_completed: 0.70,
            percentage_no_show: 0.10,
            percentage_cancelled: 0.15,
            ..Default::default()
        };
        let mut generator = AppointmentHistoryGenerator::new(config.clone());

        let patient_id = Uuid::new_v4();
        let practitioner_ids = vec![Uuid::new_v4()];
        let appointments = generator.generate_for_patient(patient_id, practitioner_ids);

        let total = appointments.len() as f32;
        let completed_ratio = appointments
            .iter()
            .filter(|appointment| appointment.status == AppointmentStatus::Completed)
            .count() as f32
            / total;
        let no_show_ratio = appointments
            .iter()
            .filter(|appointment| appointment.status == AppointmentStatus::NoShow)
            .count() as f32
            / total;
        let cancelled_ratio = appointments
            .iter()
            .filter(|appointment| appointment.status == AppointmentStatus::Cancelled)
            .count() as f32
            / total;

        let tolerance = 0.05;
        assert!((completed_ratio - config.percentage_completed).abs() <= tolerance);
        assert!((no_show_ratio - config.percentage_no_show).abs() <= tolerance);
        assert!((cancelled_ratio - config.percentage_cancelled).abs() <= tolerance);
    }

    #[test]
    fn test_appointment_count_in_range() {
        let config = AppointmentHistoryGeneratorConfig {
            min_appointments_per_patient: 3,
            max_appointments_per_patient: 9,
            ..Default::default()
        };
        let mut generator = AppointmentHistoryGenerator::new(config.clone());

        let patient_id = Uuid::new_v4();
        let practitioner_ids = vec![Uuid::new_v4()];
        let appointments = generator.generate_for_patient(patient_id, practitioner_ids);

        assert!(appointments.len() >= config.min_appointments_per_patient);
        assert!(appointments.len() <= config.max_appointments_per_patient);
    }
}
