use chrono::{Duration, Utc};
use opengp::config::Config;
use opengp::domain::appointment::AppointmentRepository;
use opengp::domain::patient::PatientRepository;
use opengp::domain::user::PractitionerRepository;
use opengp::infrastructure::crypto::EncryptionService;
use opengp::infrastructure::database::repositories::{
    SqlxAppointmentRepository, SqlxPatientRepository, SqlxPractitionerRepository,
};
use opengp::infrastructure::database::{create_pool, run_migrations};
use opengp::infrastructure::fixtures::{
    AppointmentGenerator, AppointmentGeneratorConfig, GenerationStats,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OpenGP Appointment Generator with Database\n");
    println!("==========================================\n");

    let config = Config::from_env()?;
    let pool = create_pool(&config.app.api_server.database).await?;
    run_migrations(&pool).await?;
    let pool = pool.as_postgres().clone();

    println!("Database connected: {}\n", config.app.api_server.database.url);

    let crypto = Arc::new(EncryptionService::new()?);
    let patient_repo = Arc::new(SqlxPatientRepository::new(pool.clone(), crypto));
    let practitioner_repo = Arc::new(SqlxPractitionerRepository::new(pool.clone()));
    let appointment_repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));

    println!("Fetching existing patients and practitioners...\n");

    let patients = patient_repo.list_active(None).await?;
    let practitioners = practitioner_repo.list_active().await?;

    println!("Found {} active patients", patients.len());
    println!("Found {} active practitioners\n", practitioners.len());

    if patients.is_empty() {
        println!("No patients found in database!");
        println!("Run 'cargo run --example seed_database' first to create patients.");
        return Ok(());
    }

    if practitioners.is_empty() {
        println!("No practitioners found in database!");
        return Ok(());
    }

    let patient_ids: Vec<_> = patients.iter().map(|p| p.id).collect();
    let practitioner_ids: Vec<_> = practitioners.iter().map(|p| p.id).collect();

    let fixture_config = AppointmentGeneratorConfig {
        fill_rate: 0.60,
        start_date: Some(Utc::now()),
        end_date: Some(Utc::now() + Duration::days(7)),
        patient_ids: Some(patient_ids),
        practitioner_ids: Some(practitioner_ids),
        slot_duration_minutes: 15,
        business_hours_start: 9,
        business_hours_end: 17,
        exclude_weekends: true,
        exclude_lunch_hour: false,
        ..Default::default()
    };

    print_config(&fixture_config, patients.len(), practitioners.len());

    let mut generator = AppointmentGenerator::new(fixture_config);
    let (appointments, stats) = generator.generate_schedule();

    println!("Generated {} appointments\n", appointments.len());

    print_stats(&stats);
    print_sample_appointments(&appointments, &patients, &practitioners, 10);

    println!("\nSave appointments to database? (y/n): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        println!("\nSaving appointments to database...");
        let mut success_count = 0;
        let mut error_count = 0;

        for (i, appointment) in appointments.iter().enumerate() {
            match appointment_repo.create(appointment.clone()).await {
                Ok(_) => {
                    success_count += 1;
                    if (i + 1) % 10 == 0 {
                        println!("  Saved {} appointments...", i + 1);
                    }
                }
                Err(e) => {
                    error_count += 1;
                    eprintln!("  Error saving appointment: {}", e);
                }
            }
        }

        println!("\n==========================================");
        println!("Results:");
        println!("==========================================");
        println!("  Successfully saved: {}", success_count);
        println!("  Errors: {}", error_count);
    } else {
        println!("\nAppointments not saved (dry run).");
    }

    Ok(())
}

fn print_config(
    config: &AppointmentGeneratorConfig,
    patient_count: usize,
    practitioner_count: usize,
) {
    println!("==========================================");
    println!("Configuration:");
    println!("==========================================");
    println!("  Fill rate: {:.0}%", config.fill_rate * 100.0);

    if let Some(start) = config.start_date {
        if let Some(end) = config.end_date {
            let days = (end - start).num_days();
            println!(
                "  Date range: {} to {} ({} days)",
                start.format("%d/%m/%Y"),
                end.format("%d/%m/%Y"),
                days
            );
        }
    }

    println!(
        "  Business hours: {}:00 - {}:00",
        config.business_hours_start, config.business_hours_end
    );
    println!("  Slot duration: {} minutes", config.slot_duration_minutes);
    println!("  Exclude weekends: {}", config.exclude_weekends);
    println!("  Using {} patients from database", patient_count);
    println!("  Using {} practitioners from database", practitioner_count);
    println!("==========================================\n");
}

fn print_stats(stats: &GenerationStats) {
    println!("Schedule Statistics:");
    println!("  Total slots: {}", stats.total_slots);
    println!(
        "  Filled: {} ({:.1}%)",
        stats.filled_slots,
        stats.actual_fill_rate * 100.0
    );
    println!("  Available: {}", stats.available_slots);

    if !stats.by_status.is_empty() {
        println!("\n  By Status:");
        let mut status_vec: Vec<_> = stats.by_status.iter().collect();
        status_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (status, count) in status_vec {
            println!("    {:?}: {}", status, count);
        }
    }
    println!();
}

fn print_sample_appointments(
    appointments: &[opengp::domain::appointment::Appointment],
    patients: &[opengp::domain::patient::Patient],
    practitioners: &[opengp::domain::user::Practitioner],
    count: usize,
) {
    let to_show = count.min(appointments.len());

    if to_show == 0 {
        println!("No appointments generated.");
        return;
    }

    println!("Sample appointments (first {}):", to_show);
    println!(
        "  {:20} | {:20} | {:15} | {:15}",
        "Date/Time", "Patient", "Type", "Status"
    );
    println!(
        "  {:-<20}-+-{:-<20}-+-{:-<15}-+-{}",
        "",
        "",
        "",
        "-".repeat(15)
    );

    for appt in appointments.iter().take(to_show) {
        let patient_name = patients
            .iter()
            .find(|p| p.id == appt.patient_id)
            .map(|p| format!("{} {}", p.first_name, p.last_name))
            .unwrap_or_else(|| "Unknown".to_string());

        let practitioner_name = practitioners
            .iter()
            .find(|pr| pr.id == appt.practitioner_id)
            .map(|pr| format!("Dr. {}", pr.last_name))
            .unwrap_or_else(|| "Unknown".to_string());

        let patient_display = if patient_name.len() > 18 {
            format!("{}..", &patient_name[..18])
        } else {
            patient_name
        };

        println!(
            "  {:20} | {:20} | {:15} | {:?}",
            appt.start_time.format("%d/%m/%Y %H:%M"),
            patient_display,
            format!("{:?}", appt.appointment_type)
                .chars()
                .take(15)
                .collect::<String>(),
            appt.status
        );
        println!("  {:20} | Practitioner: {}\n", "", practitioner_name);
    }
}
