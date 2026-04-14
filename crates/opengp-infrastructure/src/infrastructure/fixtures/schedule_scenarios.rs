use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use uuid::Uuid;

use opengp_domain::domain::appointment::{
    AppointmentStatus, AppointmentType, CalendarAppointment, CalendarDayView, PractitionerSchedule,
};
use opengp_domain::domain::user::WorkingHours;

/// Deterministic schedule fixture scenarios for testing calendar views
///
/// All scenarios use fixed UUIDs and dates for reproducible test data.
pub struct ScheduleScenario;

impl ScheduleScenario {
    /// Empty day with one practitioner and zero appointments
    pub fn empty_day(date: NaiveDate) -> CalendarDayView {
        let practitioner_id = Uuid::from_u128(1);

        CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Empty".to_string(),
                appointments: vec![],
                working_hours: None,
            }],
        }
    }

    /// Single 15-minute Standard appointment at 09:00
    pub fn single_appointment(date: NaiveDate, practitioner_id: Uuid) -> CalendarDayView {
        let start_time = date_time_at(date, 9, 0);
        let end_time = start_time + Duration::minutes(15);

        let appointment = CalendarAppointment {
            id: Uuid::from_u128(100),
            patient_id: Uuid::from_u128(1000),
            patient_name: "John Doe".to_string(),
            practitioner_id,
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            is_overlapping: false,
            reason: Some("General checkup".to_string()),
            notes: None,
        };

        CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Smith".to_string(),
                appointments: vec![appointment],
                working_hours: None,
            }],
        }
    }

    /// Two overlapping appointments at the same time
    pub fn two_overlapping(date: NaiveDate, practitioner_id: Uuid) -> CalendarDayView {
        let start_time = date_time_at(date, 9, 0);
        let end_time = start_time + Duration::minutes(15);

        let appt1 = CalendarAppointment {
            id: Uuid::from_u128(101),
            patient_id: Uuid::from_u128(1001),
            patient_name: "Alice Smith".to_string(),
            practitioner_id,
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            is_overlapping: true,
            reason: None,
            notes: None,
        };

        let appt2 = CalendarAppointment {
            id: Uuid::from_u128(102),
            patient_id: Uuid::from_u128(1002),
            patient_name: "Bob Jones".to_string(),
            practitioner_id,
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            is_overlapping: true,
            reason: None,
            notes: None,
        };

        CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Busy".to_string(),
                appointments: vec![appt1, appt2],
                working_hours: None,
            }],
        }
    }

    /// Full morning: 12 back-to-back 15-minute appointments from 09:00 to 12:00
    pub fn full_morning(date: NaiveDate, practitioner_id: Uuid) -> CalendarDayView {
        let mut appointments = Vec::new();

        for i in 0..12 {
            let start_time = date_time_at(date, 9, 0) + Duration::minutes(i as i64 * 15);
            let end_time = start_time + Duration::minutes(15);

            let appointment = CalendarAppointment {
                id: Uuid::from_u128(200 + i as u128),
                patient_id: Uuid::from_u128(2000 + i as u128),
                patient_name: format!("Patient {}", i + 1),
                practitioner_id,
                start_time,
                end_time,
                appointment_type: AppointmentType::Standard,
                status: AppointmentStatus::Scheduled,
                is_urgent: false,
                slot_span: 1,
                is_overlapping: false,
                reason: None,
                notes: None,
            };

            appointments.push(appointment);
        }

        CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Booked".to_string(),
                appointments,
                working_hours: None,
            }],
        }
    }

    /// Multiple practitioners with one appointment each
    pub fn multi_practitioner(date: NaiveDate, practitioner_ids: &[Uuid]) -> CalendarDayView {
        let start_time = date_time_at(date, 9, 0);
        let end_time = start_time + Duration::minutes(15);

        let practitioners = practitioner_ids
            .iter()
            .enumerate()
            .map(|(idx, &practitioner_id)| {
                let appointment = CalendarAppointment {
                    id: Uuid::from_u128(300 + idx as u128),
                    patient_id: Uuid::from_u128(3000 + idx as u128),
                    patient_name: format!("Patient {}", idx + 1),
                    practitioner_id,
                    start_time,
                    end_time,
                    appointment_type: AppointmentType::Standard,
                    status: AppointmentStatus::Scheduled,
                    is_urgent: false,
                    slot_span: 1,
                    is_overlapping: false,
                    reason: None,
                    notes: None,
                };

                PractitionerSchedule {
                    practitioner_id,
                    practitioner_name: format!("Dr. Practitioner {}", idx + 1),
                    appointments: vec![appointment],
                    working_hours: None,
                }
            })
            .collect();

        CalendarDayView {
            date,
            practitioners,
        }
    }

    /// Practitioner with custom working hours
    pub fn with_working_hours(
        date: NaiveDate,
        practitioner_id: Uuid,
        start_hour: u8,
        end_hour: u8,
    ) -> CalendarDayView {
        let start_time = date_time_at(date, start_hour, 0);
        let end_time = start_time + Duration::minutes(15);

        let appointment = CalendarAppointment {
            id: Uuid::from_u128(400),
            patient_id: Uuid::from_u128(4000),
            patient_name: "Test Patient".to_string(),
            practitioner_id,
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            is_overlapping: false,
            reason: None,
            notes: None,
        };

        let working_hours = WorkingHours {
            id: Uuid::from_u128(5000),
            practitioner_id,
            day_of_week: date.weekday().number_from_monday() as u8 - 1,
            start_time: NaiveTime::from_hms_opt(start_hour as u32, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(end_hour as u32, 0, 0).unwrap(),
            is_active: true,
            created_at: date_time_at(date, 0, 0),
            updated_at: date_time_at(date, 0, 0),
        };

        CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Scheduled".to_string(),
                appointments: vec![appointment],
                working_hours: Some(working_hours),
            }],
        }
    }

    /// Practitioner with no working hours set
    pub fn no_working_hours(date: NaiveDate, practitioner_id: Uuid) -> CalendarDayView {
        let start_time = date_time_at(date, 9, 0);
        let end_time = start_time + Duration::minutes(15);

        let appointment = CalendarAppointment {
            id: Uuid::from_u128(401),
            patient_id: Uuid::from_u128(4001),
            patient_name: "Another Patient".to_string(),
            practitioner_id,
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            is_overlapping: false,
            reason: None,
            notes: None,
        };

        CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Flexible".to_string(),
                appointments: vec![appointment],
                working_hours: None,
            }],
        }
    }
}

/// Helper function to create a DateTime<Utc> from a date and time components
fn date_time_at(date: NaiveDate, hour: u8, minute: u8) -> DateTime<Utc> {
    let naive_datetime = date
        .and_hms_opt(hour as u32, minute as u32, 0)
        .expect("Invalid datetime");
    chrono::Local
        .from_local_datetime(&naive_datetime)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_day() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let view = ScheduleScenario::empty_day(date);

        assert_eq!(view.date, date);
        assert_eq!(view.practitioners.len(), 1);
        assert_eq!(view.practitioners[0].appointments.len(), 0);
    }

    #[test]
    fn test_single_appointment() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(10);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);

        assert_eq!(view.date, date);
        assert_eq!(view.practitioners.len(), 1);
        assert_eq!(view.practitioners[0].appointments.len(), 1);
        assert_eq!(view.practitioners[0].practitioner_id, practitioner_id);

        let appt = &view.practitioners[0].appointments[0];
        assert_eq!(appt.appointment_type, AppointmentType::Standard);
        assert_eq!(appt.duration_minutes(), 15);
    }

    #[test]
    fn test_two_overlapping() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(11);
        let view = ScheduleScenario::two_overlapping(date, practitioner_id);

        assert_eq!(view.practitioners.len(), 1);
        assert_eq!(view.practitioners[0].appointments.len(), 2);

        // Both appointments should be marked as overlapping
        assert!(view.practitioners[0].appointments[0].is_overlapping);
        assert!(view.practitioners[0].appointments[1].is_overlapping);

        // Both should have the same start and end times
        assert_eq!(
            view.practitioners[0].appointments[0].start_time,
            view.practitioners[0].appointments[1].start_time
        );
        assert_eq!(
            view.practitioners[0].appointments[0].end_time,
            view.practitioners[0].appointments[1].end_time
        );
    }

    #[test]
    fn test_full_morning() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(12);
        let view = ScheduleScenario::full_morning(date, practitioner_id);

        assert_eq!(view.practitioners.len(), 1);
        assert_eq!(view.practitioners[0].appointments.len(), 12);

        // Verify appointments are back-to-back
        let appts = &view.practitioners[0].appointments;
        for i in 0..11 {
            assert_eq!(appts[i].end_time, appts[i + 1].start_time);
        }

        // First appointment should start at 09:00
        let first_start = appts[0].start_time;
        let expected_start = date_time_at(date, 9, 0);
        assert_eq!(first_start, expected_start);

        // Last appointment should end at 12:00
        let last_end = appts[11].end_time;
        let expected_end = date_time_at(date, 12, 0);
        assert_eq!(last_end, expected_end);
    }

    #[test]
    fn test_multi_practitioner() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_ids = vec![
            Uuid::from_u128(20),
            Uuid::from_u128(21),
            Uuid::from_u128(22),
        ];
        let view = ScheduleScenario::multi_practitioner(date, &practitioner_ids);

        assert_eq!(view.practitioners.len(), 3);

        // Each practitioner should have exactly one appointment
        for (idx, schedule) in view.practitioners.iter().enumerate() {
            assert_eq!(schedule.practitioner_id, practitioner_ids[idx]);
            assert_eq!(schedule.appointments.len(), 1);
        }
    }

    #[test]
    fn test_with_working_hours() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(30);
        let view = ScheduleScenario::with_working_hours(date, practitioner_id, 8, 17);

        assert_eq!(view.practitioners.len(), 1);
        let schedule = &view.practitioners[0];

        assert!(schedule.working_hours.is_some());
        let wh = schedule.working_hours.as_ref().unwrap();
        assert_eq!(wh.start_time, NaiveTime::from_hms_opt(8, 0, 0).unwrap());
        assert_eq!(wh.end_time, NaiveTime::from_hms_opt(17, 0, 0).unwrap());
        assert!(wh.is_active);
    }

    #[test]
    fn test_no_working_hours() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(31);
        let view = ScheduleScenario::no_working_hours(date, practitioner_id);

        assert_eq!(view.practitioners.len(), 1);
        let schedule = &view.practitioners[0];

        assert!(schedule.working_hours.is_none());
        assert_eq!(schedule.appointments.len(), 1);
    }
}
