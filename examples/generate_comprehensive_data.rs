use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use opengp::infrastructure::crypto::EncryptionService;
use opengp::infrastructure::fixtures::{
    AppointmentGenerator, AppointmentGeneratorConfig, ClinicalDataGeneratorConfig,
    ComprehensivePatientGenerator, ComprehensivePatientGeneratorConfig, PatientGeneratorConfig,
};
use opengp_api::ApiConfig;
use opengp_domain::domain::appointment::Appointment;
use opengp_domain::domain::clinical::{
    AlcoholStatus, Allergy, Consultation, ExerciseFrequency, MedicalHistory, SmokingStatus,
    SocialHistory,
};
use opengp_domain::domain::patient::Patient;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use uuid::Uuid;

const DEFAULT_PATIENTS: usize = 100;
const DEFAULT_APPOINTMENTS_MAX: usize = 5;
const DEFAULT_CONSULTATIONS_MAX: usize = 3;
const DEFAULT_MEDICAL_HISTORY_MAX: usize = 5;
const DEFAULT_ALLERGIES_MAX: usize = 3;

#[derive(Debug, Clone)]
struct CliOptions {
    patients: usize,
    appointments_max: usize,
    consultations_max: usize,
}

#[derive(Debug, Default)]
struct InsertStats {
    patients_created: usize,
    appointments_created: usize,
    consultations_created: usize,
    medical_history_created: usize,
    allergies_created: usize,
    social_history_created: usize,
    patient_failures: usize,
    record_failures: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let cli = match parse_args(&args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("Error: {message}\n");
            print_help();
            return Err("invalid CLI arguments".into());
        }
    };

    println!("═══════════════════════════════════════════════════════════");
    println!("   OpenGP Comprehensive PostgreSQL Data Generator");
    println!("═══════════════════════════════════════════════════════════\n");

    let api_config = ApiConfig::from_env()?;
    let db_url = api_config.database_url.clone();
    println!("Connecting to PostgreSQL via API_DATABASE_URL...");

    let connect_options = PgConnectOptions::from_str(&db_url)
        .map_err(|e| format!("Invalid API_DATABASE_URL: {e}"))?;

    let pool = PgPoolOptions::new()
        .max_connections(api_config.database_max_connections)
        .min_connections(api_config.database_min_connections)
        .acquire_timeout(Duration::from_secs(api_config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(api_config.idle_timeout_secs))
        .connect_with(connect_options)
        .await?;

    println!("✓ PostgreSQL connected");
    println!("  URL: {db_url}\n");

    println!("Running migrations from ./migrations ...");
    run_postgres_migrations(&pool).await?;
    normalize_clinical_encrypted_columns(&pool).await?;
    println!("✓ Migrations complete\n");

    ensure_encryption_key();
    let crypto = EncryptionService::new()?;

    let practitioner_ids = load_practitioner_ids(&pool).await?;
    if practitioner_ids.is_empty() {
        return Err("No active users found in users table after migration".into());
    }
    let default_actor = practitioner_ids[0];

    println!("Configuration:");
    println!("  • Patients: {}", cli.patients);
    println!("  • Max appointments/patient: {}", cli.appointments_max);
    println!("  • Max consultations/patient: {}", cli.consultations_max);
    println!(
        "  • Max medical history/patient: {}",
        DEFAULT_MEDICAL_HISTORY_MAX
    );
    println!("  • Max allergies/patient: {}", DEFAULT_ALLERGIES_MAX);
    println!(
        "  • Active practitioners found: {}\n",
        practitioner_ids.len()
    );

    let generator_config = ComprehensivePatientGeneratorConfig {
        patient_count: cli.patients,
        practitioner_ids: practitioner_ids.clone(),
        patient_config: PatientGeneratorConfig {
            count: cli.patients,
            ..Default::default()
        },
        clinical_config: ClinicalDataGeneratorConfig {
            consultation_count: cli.consultations_max.max(1),
            medical_history_count: DEFAULT_MEDICAL_HISTORY_MAX,
            allergy_count: DEFAULT_ALLERGIES_MAX,
            ..Default::default()
        },
    };

    println!("Generating comprehensive profiles...");
    let generator = ComprehensivePatientGenerator::new(generator_config);
    let mut profiles = generator.generate();
    println!("✓ Generated {} patient profiles\n", profiles.len());

    let mut stats = InsertStats::default();
    let total_patients = profiles.len();

    println!("Inserting records into PostgreSQL...");
    for (index, profile) in profiles.iter_mut().enumerate() {
        let seed = pseudo_seed(profile.patient.id, index);

        let consultation_count = bounded_count(seed, cli.consultations_max);
        profile.consultations.truncate(consultation_count);
        for consultation in &mut profile.consultations {
            consultation.created_by = default_actor;
            consultation.updated_by = consultation.updated_by.map(|_| default_actor);
            consultation.signed_by = consultation.signed_by.map(|_| default_actor);
            if consultation.practitioner_id.is_nil() {
                consultation.practitioner_id = default_actor;
            }
        }

        let history_count = bounded_count(seed.rotate_left(11), DEFAULT_MEDICAL_HISTORY_MAX);
        profile.medical_history.truncate(history_count);
        for history in &mut profile.medical_history {
            history.created_by = default_actor;
            history.updated_by = history.updated_by.map(|_| default_actor);
        }

        let allergy_count = bounded_count(seed.rotate_left(17), DEFAULT_ALLERGIES_MAX);
        profile.allergies.truncate(allergy_count);
        for allergy in &mut profile.allergies {
            allergy.created_by = default_actor;
            allergy.updated_by = allergy.updated_by.map(|_| default_actor);
        }

        let mut appointment_generator = AppointmentGenerator::new(AppointmentGeneratorConfig {
            count: bounded_count(seed.rotate_left(7), cli.appointments_max),
            future_percentage: 1.0,
            patient_ids: Some(vec![profile.patient.id]),
            practitioner_ids: Some(practitioner_ids.clone()),
            ..Default::default()
        });
        let appointments = appointment_generator.generate();

        let social_history = generate_social_history(profile.patient.id, default_actor, seed);

        match insert_patient(&pool, &crypto, &profile.patient, default_actor).await {
            Ok(_) => {
                stats.patients_created += 1;

                for appointment in &appointments {
                    if insert_appointment(&pool, appointment).await.is_ok() {
                        stats.appointments_created += 1;
                    } else {
                        stats.record_failures += 1;
                    }
                }

                for consultation in &profile.consultations {
                    if insert_consultation(&pool, &crypto, consultation)
                        .await
                        .is_ok()
                    {
                        stats.consultations_created += 1;
                    } else {
                        stats.record_failures += 1;
                    }
                }

                for history in &profile.medical_history {
                    if insert_medical_history(&pool, &crypto, history)
                        .await
                        .is_ok()
                    {
                        stats.medical_history_created += 1;
                    } else {
                        stats.record_failures += 1;
                    }
                }

                for allergy in &profile.allergies {
                    if insert_allergy(&pool, &crypto, allergy).await.is_ok() {
                        stats.allergies_created += 1;
                    } else {
                        stats.record_failures += 1;
                    }
                }

                if insert_social_history(&pool, &crypto, &social_history)
                    .await
                    .is_ok()
                {
                    stats.social_history_created += 1;
                } else {
                    stats.record_failures += 1;
                }
            }
            Err(e) => {
                stats.patient_failures += 1;
                eprintln!(
                    "\n✗ Failed patient insert [{} {}]: {}",
                    profile.patient.first_name, profile.patient.last_name, e
                );
            }
        }

        render_progress(index + 1, total_patients)?;
    }

    println!("\n\n═══════════════════════════════════════════════════════════");
    println!("Statistics");
    println!("═══════════════════════════════════════════════════════════");
    println!("  ✓ Patients created: {}", stats.patients_created);
    println!("  ✓ Appointments created: {}", stats.appointments_created);
    println!("  ✓ Consultations created: {}", stats.consultations_created);
    println!(
        "  ✓ Medical history entries: {}",
        stats.medical_history_created
    );
    println!("  ✓ Allergies created: {}", stats.allergies_created);
    println!(
        "  ✓ Social history records: {}",
        stats.social_history_created
    );
    println!("  ✗ Patient insert failures: {}", stats.patient_failures);
    println!("  ✗ Related-record failures: {}", stats.record_failures);
    println!();

    if stats.patient_failures > 0 || stats.record_failures > 0 {
        return Err("Data generation completed with errors".into());
    }

    println!("✓ Comprehensive data generation completed successfully.");
    Ok(())
}

fn parse_args(args: &[String]) -> Result<CliOptions, String> {
    let mut patients = DEFAULT_PATIENTS;
    let mut appointments_max = DEFAULT_APPOINTMENTS_MAX;
    let mut consultations_max = DEFAULT_CONSULTATIONS_MAX;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--patients" | "-p" => {
                i += 1;
                let value = args.get(i).ok_or("Missing value for --patients")?;
                patients = value
                    .parse::<usize>()
                    .map_err(|_| "--patients must be a positive integer")?;
            }
            "--appointments" | "-a" => {
                i += 1;
                let value = args.get(i).ok_or("Missing value for --appointments")?;
                appointments_max = value
                    .parse::<usize>()
                    .map_err(|_| "--appointments must be a positive integer")?;
            }
            "--consultations" | "-c" => {
                i += 1;
                let value = args.get(i).ok_or("Missing value for --consultations")?;
                consultations_max = value
                    .parse::<usize>()
                    .map_err(|_| "--consultations must be a positive integer")?;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("Unknown option: {other}")),
        }
        i += 1;
    }

    if patients == 0 {
        return Err("--patients must be >= 1".to_string());
    }
    if appointments_max == 0 {
        return Err("--appointments must be >= 1".to_string());
    }
    if consultations_max == 0 {
        return Err("--consultations must be >= 1".to_string());
    }

    Ok(CliOptions {
        patients,
        appointments_max,
        consultations_max,
    })
}

fn print_help() {
    println!("OpenGP Comprehensive PostgreSQL Data Generator");
    println!();
    println!("Usage:");
    println!(
        "  cargo run --example generate_comprehensive_data -- [--patients N] [--appointments N] [--consultations N]"
    );
    println!();
    println!("Options:");
    println!(
        "  -p, --patients <N>        Number of patients to generate (default: {DEFAULT_PATIENTS})"
    );
    println!(
        "  -a, --appointments <N>    Maximum appointments per patient (default: {DEFAULT_APPOINTMENTS_MAX})"
    );
    println!(
        "  -c, --consultations <N>   Maximum consultations per patient (default: {DEFAULT_CONSULTATIONS_MAX})"
    );
    println!("  -h, --help                Show this help message");
    println!();
    println!("Environment:");
    println!("  API_DATABASE_URL          PostgreSQL connection string");
    println!("  ENCRYPTION_KEY            64-char hex key for encrypted fields");
}

fn ensure_encryption_key() {
    if env::var("ENCRYPTION_KEY").is_ok() {
        return;
    }
    unsafe {
        env::set_var(
            "ENCRYPTION_KEY",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
    }
}

async fn load_practitioner_ids(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE is_active = TRUE ORDER BY created_at")
        .fetch_all(pool)
        .await
}

async fn run_postgres_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let migrations_dir = Path::new("./migrations");
    let mut files = fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "sql")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    files.sort_by_key(|entry| entry.file_name());

    for file in files {
        let path = file.path();
        let sql = fs::read_to_string(&path)?;
        if sql.trim().is_empty() {
            continue;
        }

        for statement in sql.split(';') {
            let stmt = statement.trim();
            if stmt.is_empty() {
                continue;
            }

            if let Err(err) = sqlx::query(stmt).execute(pool).await {
                let message = err.to_string();
                let duplicate = message.contains("duplicate key")
                    || message.contains("already exists")
                    || message.contains("already applied");
                if !duplicate {
                    return Err(format!(
                        "Migration failed in {}: {}",
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        err
                    )
                    .into());
                }
            }
        }
    }

    Ok(())
}

async fn normalize_clinical_encrypted_columns(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_consultation_reason_column(pool).await?;
    ensure_column_is_bytea(pool, "consultations", "clinical_notes").await?;
    ensure_column_is_bytea(pool, "medical_history", "notes").await?;
    ensure_column_is_bytea(pool, "allergies", "reaction").await?;
    ensure_column_is_bytea(pool, "allergies", "notes").await?;
    ensure_column_is_bytea(pool, "social_history", "notes").await?;
    Ok(())
}

async fn ensure_consultation_reason_column(pool: &PgPool) -> Result<(), sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'consultations'
              AND column_name = 'reason'
        )
        "#,
    )
    .fetch_one(pool)
    .await?;

    if !exists {
        sqlx::query("ALTER TABLE consultations ADD COLUMN reason TEXT")
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn ensure_column_is_bytea(
    pool: &PgPool,
    table: &str,
    column: &str,
) -> Result<(), sqlx::Error> {
    let udt_name = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT udt_name
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = $1
          AND column_name = $2
        "#,
    )
    .bind(table)
    .bind(column)
    .fetch_optional(pool)
    .await?
    .flatten();

    match udt_name.as_deref() {
        Some("bytea") => Ok(()),
        Some("uuid") => {
            let alter = format!(
                "ALTER TABLE {table} ALTER COLUMN {column} TYPE BYTEA USING decode(replace({column}::text, '-', ''), 'hex')"
            );
            sqlx::query(&alter).execute(pool).await?;
            Ok(())
        }
        None if table == "consultations" && column == "clinical_notes" => {
            sqlx::query("ALTER TABLE consultations ADD COLUMN clinical_notes BYTEA")
                .execute(pool)
                .await?;
            Ok(())
        }
        Some(other) => Err(sqlx::Error::Protocol(format!(
            "Unsupported column type for {}.{}: {}",
            table, column, other
        ))),
        None => Err(sqlx::Error::Protocol(format!(
            "Column {}.{} not found",
            table, column
        ))),
    }
}

async fn insert_patient(
    pool: &PgPool,
    crypto: &EncryptionService,
    patient: &Patient,
    actor_id: Uuid,
) -> Result<(), sqlx::Error> {
    let encrypted_ihi = patient
        .ihi
        .as_ref()
        .map(|s| crypto.encrypt(s.as_str()))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("IHI encryption failed: {e}")))?;

    let encrypted_medicare = patient
        .medicare_number
        .as_ref()
        .map(|s| crypto.encrypt(s.as_str()))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("Medicare encryption failed: {e}")))?;

    let phone_home = patient.phone_home.as_ref().map(|p| p.to_string());
    let phone_mobile = patient.phone_mobile.as_ref().map(|p| p.to_string());

    sqlx::query(
        r#"
        INSERT INTO patients (
            id, ihi, medicare_number, medicare_irn, medicare_expiry,
            title, first_name, middle_name, last_name, preferred_name,
            date_of_birth, gender,
            address_line1, address_line2, suburb, state, postcode, country,
            phone_home, phone_mobile, email,
            emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
            is_active, is_deceased, version,
            created_at, updated_at, created_by, updated_by
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12,
            $13, $14, $15, $16, $17, $18,
            $19, $20, $21,
            $22, $23, $24,
            $25, $26, $27,
            $28, $29, $30, $31
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(patient.id)
    .bind(encrypted_ihi)
    .bind(encrypted_medicare)
    .bind(patient.medicare_irn.map(i32::from))
    .bind(patient.medicare_expiry)
    .bind(&patient.title)
    .bind(&patient.first_name)
    .bind(&patient.middle_name)
    .bind(&patient.last_name)
    .bind(&patient.preferred_name)
    .bind(patient.date_of_birth)
    .bind(patient.gender.to_string())
    .bind(&patient.address.line1)
    .bind(&patient.address.line2)
    .bind(&patient.address.suburb)
    .bind(&patient.address.state)
    .bind(&patient.address.postcode)
    .bind(&patient.address.country)
    .bind(phone_home)
    .bind(phone_mobile)
    .bind(&patient.email)
    .bind(patient.emergency_contact.as_ref().map(|c| c.name.clone()))
    .bind(patient.emergency_contact.as_ref().map(|c| c.phone.clone()))
    .bind(
        patient
            .emergency_contact
            .as_ref()
            .map(|c| c.relationship.clone()),
    )
    .bind(patient.is_active)
    .bind(patient.is_deceased)
    .bind(patient.version)
    .bind(patient.created_at)
    .bind(patient.updated_at)
    .bind(actor_id)
    .bind::<Option<Uuid>>(None)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_appointment(pool: &PgPool, appointment: &Appointment) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO appointments (
            id, patient_id, practitioner_id,
            start_time, end_time,
            appointment_type, status,
            reason, notes,
            is_urgent, reminder_sent, confirmed,
            cancellation_reason, version,
            created_at, updated_at, created_by, updated_by
        ) VALUES (
            $1, $2, $3,
            $4, $5,
            $6, $7,
            $8, $9,
            $10, $11, $12,
            $13, $14,
            $15, $16, $17, $18
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(appointment.id)
    .bind(appointment.patient_id)
    .bind(appointment.practitioner_id)
    .bind(appointment.start_time)
    .bind(appointment.end_time)
    .bind(appointment.appointment_type.to_string())
    .bind(appointment.status.to_string())
    .bind(&appointment.reason)
    .bind(&appointment.notes)
    .bind(appointment.is_urgent)
    .bind(appointment.reminder_sent)
    .bind(appointment.confirmed)
    .bind(&appointment.cancellation_reason)
    .bind(appointment.version)
    .bind(appointment.created_at)
    .bind(appointment.updated_at)
    .bind(appointment.created_by)
    .bind(appointment.updated_by)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_consultation(
    pool: &PgPool,
    crypto: &EncryptionService,
    consultation: &Consultation,
) -> Result<(), sqlx::Error> {
    let encrypted_notes = consultation
        .clinical_notes
        .as_ref()
        .map(|text| crypto.encrypt(text))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("encryption failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO consultations (
            id, patient_id, practitioner_id, appointment_id,
            consultation_date, reason, clinical_notes,
            is_signed, signed_at, signed_by,
            version, created_at, updated_at, created_by, updated_by
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7,
            $8, $9, $10,
            $11, $12, $13, $14, $15
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(consultation.id)
    .bind(consultation.patient_id)
    .bind(consultation.practitioner_id)
    .bind(consultation.appointment_id)
    .bind(consultation.consultation_date)
    .bind(&consultation.reason)
    .bind(encrypted_notes)
    .bind(consultation.is_signed)
    .bind(consultation.signed_at)
    .bind(consultation.signed_by)
    .bind(consultation.version)
    .bind(consultation.created_at)
    .bind(consultation.updated_at)
    .bind(consultation.created_by)
    .bind(consultation.updated_by)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_medical_history(
    pool: &PgPool,
    crypto: &EncryptionService,
    history: &MedicalHistory,
) -> Result<(), sqlx::Error> {
    let encrypted_notes = history
        .notes
        .as_ref()
        .map(|text| crypto.encrypt(text))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("encryption failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO medical_history (
            id, patient_id, condition, diagnosis_date,
            status, severity, notes, is_active,
            created_at, updated_at, created_by, updated_by
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8,
            $9, $10, $11, $12
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(history.id)
    .bind(history.patient_id)
    .bind(&history.condition)
    .bind(history.diagnosis_date)
    .bind(history.status.to_string())
    .bind(history.severity.map(|s| s.to_string()))
    .bind(encrypted_notes)
    .bind(history.is_active)
    .bind(history.created_at)
    .bind(history.updated_at)
    .bind(history.created_by)
    .bind(history.updated_by)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_allergy(
    pool: &PgPool,
    crypto: &EncryptionService,
    allergy: &Allergy,
) -> Result<(), sqlx::Error> {
    let encrypted_reaction = allergy
        .reaction
        .as_ref()
        .map(|text| crypto.encrypt(text))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("encryption failed: {e}")))?;

    let encrypted_notes = allergy
        .notes
        .as_ref()
        .map(|text| crypto.encrypt(text))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("encryption failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO allergies (
            id, patient_id, allergen, allergy_type,
            severity, reaction, onset_date, notes,
            is_active, created_at, updated_at, created_by, updated_by
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8,
            $9, $10, $11, $12, $13
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(allergy.id)
    .bind(allergy.patient_id)
    .bind(&allergy.allergen)
    .bind(allergy.allergy_type.to_string())
    .bind(allergy.severity.to_string())
    .bind(encrypted_reaction)
    .bind(allergy.onset_date)
    .bind(encrypted_notes)
    .bind(allergy.is_active)
    .bind(allergy.created_at)
    .bind(allergy.updated_at)
    .bind(allergy.created_by)
    .bind(allergy.updated_by)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_social_history(
    pool: &PgPool,
    crypto: &EncryptionService,
    social_history: &SocialHistory,
) -> Result<(), sqlx::Error> {
    let encrypted_notes = social_history
        .notes
        .as_ref()
        .map(|text| crypto.encrypt(text))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("encryption failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO social_history (
            id, patient_id,
            smoking_status, cigarettes_per_day, smoking_quit_date,
            alcohol_status, standard_drinks_per_week,
            exercise_frequency, occupation, living_situation, support_network,
            notes, updated_at, updated_by
        ) VALUES (
            $1, $2,
            $3, $4, $5,
            $6, $7,
            $8, $9, $10, $11,
            $12, $13, $14
        )
        ON CONFLICT (patient_id) DO NOTHING
        "#,
    )
    .bind(social_history.id)
    .bind(social_history.patient_id)
    .bind(social_history.smoking_status.to_string())
    .bind(social_history.cigarettes_per_day.map(i32::from))
    .bind(social_history.smoking_quit_date)
    .bind(social_history.alcohol_status.to_string())
    .bind(social_history.standard_drinks_per_week.map(i32::from))
    .bind(social_history.exercise_frequency.map(|f| f.to_string()))
    .bind(&social_history.occupation)
    .bind(&social_history.living_situation)
    .bind(&social_history.support_network)
    .bind(encrypted_notes)
    .bind(social_history.updated_at)
    .bind(social_history.updated_by)
    .execute(pool)
    .await?;

    Ok(())
}

fn generate_social_history(patient_id: Uuid, actor_id: Uuid, seed: u64) -> SocialHistory {
    let smoking_status = match seed % 3 {
        0 => SmokingStatus::NeverSmoked,
        1 => SmokingStatus::CurrentSmoker,
        _ => SmokingStatus::ExSmoker,
    };

    let alcohol_status = match (seed >> 3) % 4 {
        0 => AlcoholStatus::None,
        1 => AlcoholStatus::Occasional,
        2 => AlcoholStatus::Moderate,
        _ => AlcoholStatus::Heavy,
    };

    let exercise_frequency = match (seed >> 6) % 5 {
        0 => Some(ExerciseFrequency::None),
        1 => Some(ExerciseFrequency::Rarely),
        2 => Some(ExerciseFrequency::OnceOrTwicePerWeek),
        3 => Some(ExerciseFrequency::ThreeToFiveTimes),
        _ => Some(ExerciseFrequency::Daily),
    };

    let occupation = [
        "Teacher",
        "Electrician",
        "Engineer",
        "Nurse",
        "Chef",
        "Driver",
        "Administrator",
        "Student",
    ][(seed as usize) % 8]
        .to_string();

    let living_situation = [
        "Lives with partner",
        "Lives alone",
        "Lives with family",
        "Shared accommodation",
    ][((seed >> 9) as usize) % 4]
        .to_string();

    let support_network = [
        "Strong family support",
        "Moderate social support",
        "Community support group",
        "Limited support network",
    ][((seed >> 13) as usize) % 4]
        .to_string();

    let cigarettes_per_day = if smoking_status == SmokingStatus::CurrentSmoker {
        Some(((seed % 20) + 1) as u8)
    } else {
        None
    };

    let standard_drinks_per_week = match alcohol_status {
        AlcoholStatus::None => None,
        AlcoholStatus::Occasional => Some(((seed % 3) + 1) as u8),
        AlcoholStatus::Moderate => Some(((seed % 10) + 4) as u8),
        AlcoholStatus::Heavy => Some(((seed % 20) + 12) as u8),
    };

    SocialHistory {
        id: Uuid::new_v4(),
        patient_id,
        smoking_status,
        cigarettes_per_day,
        smoking_quit_date: None,
        alcohol_status,
        standard_drinks_per_week,
        exercise_frequency,
        occupation: Some(occupation),
        living_situation: Some(living_situation),
        support_network: Some(support_network),
        notes: Some("Generated social history profile".to_string()),
        updated_at: Utc::now(),
        updated_by: actor_id,
    }
}

fn render_progress(current: usize, total: usize) -> Result<(), io::Error> {
    let total = total.max(1);
    let percentage = ((current as f64 / total as f64) * 100.0).round() as usize;
    let width: usize = 30;
    let filled: usize = ((current as f64 / total as f64) * width as f64).round() as usize;
    let empty: usize = if filled >= width { 0 } else { width - filled };

    print!(
        "\r  [{}{}] {:>3}% ({}/{})",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage,
        current,
        total
    );
    io::stdout().flush()
}

fn pseudo_seed(id: Uuid, index: usize) -> u64 {
    let bytes = id.as_bytes();
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[0..8]);
    u64::from_le_bytes(arr) ^ (index as u64).rotate_left(9)
}

fn bounded_count(seed: u64, max: usize) -> usize {
    (seed as usize % max.max(1)) + 1
}
