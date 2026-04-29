use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use opengp::infrastructure::crypto::EncryptionService;
use opengp::infrastructure::fixtures::{
    AppointmentGenerator, AppointmentGeneratorConfig, BillingGeneratorConfig,
    ClinicalDataGeneratorConfig, ComprehensivePatientGenerator,
    ComprehensivePatientGeneratorConfig, PatientGeneratorConfig,
};
use opengp_config::Config;
use opengp_domain::domain::appointment::Appointment;
use opengp_domain::domain::billing::{DVAClaim, Invoice, MedicareClaim, Payment};
use opengp_domain::domain::clinical::{
    AlcoholStatus, Allergy, Consultation, ExerciseFrequency, FamilyHistory, MedicalHistory,
    SmokingStatus, SocialHistory, VitalSigns,
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
    vitals_created: usize,
    family_history_created: usize,
    invoices_created: usize,
    medicare_claims_created: usize,
    dva_claims_created: usize,
    payments_created: usize,
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

    let config = Config::from_env()?;
    let db_url = config.app.api_server.database.url.clone();
    println!("Connecting to PostgreSQL via API_DATABASE_URL...");

    let connect_options = PgConnectOptions::from_str(&db_url)
        .map_err(|e| format!("Invalid API_DATABASE_URL: {e}"))?;

    let pool = PgPoolOptions::new()
        .max_connections(config.app.api_server.database.max_connections)
        .min_connections(config.app.api_server.database.min_connections)
        .acquire_timeout(Duration::from_secs(
            config.app.api_server.database.connect_timeout_secs,
        ))
        .idle_timeout(Duration::from_secs(
            config.app.api_server.database.idle_timeout_secs,
        ))
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
        billing_config: BillingGeneratorConfig::default(),
    };

    println!("Generating comprehensive profiles...");
    let generator = ComprehensivePatientGenerator::new(generator_config);
    let mut profiles = generator.generate();
    println!("✓ Generated {} patient profiles\n", profiles.len());

    let mut stats = InsertStats::default();
    let total_patients = profiles.len();

    println!("Inserting records into PostgreSQL...");
    let mut error_samples: Vec<String> = Vec::new();
    let mut log_err = |ctx: &str, e: &dyn std::fmt::Display| {
        if error_samples.len() < 10 {
            error_samples.push(format!("{ctx}: {e}"));
        }
    };
    for (index, profile) in profiles.iter_mut().enumerate() {
        let seed = pseudo_seed(profile.patient.id, index);

        let consultation_count = bounded_count(seed, cli.consultations_max);
        profile.consultations.truncate(consultation_count);
        let valid_consultation_ids: std::collections::HashSet<Uuid> =
            profile.consultations.iter().map(|c| c.id).collect();
        for vital in &mut profile.vitals {
            if let Some(cid) = vital.consultation_id {
                if !valid_consultation_ids.contains(&cid) {
                    vital.consultation_id = None;
                }
            }
        }
        for invoice in &mut profile.billing.invoices {
            if let Some(cid) = invoice.consultation_id {
                if !valid_consultation_ids.contains(&cid) {
                    invoice.consultation_id = None;
                }
            }
        }
        for claim in &mut profile.billing.medicare_claims {
            if let Some(cid) = claim.consultation_id {
                if !valid_consultation_ids.contains(&cid) {
                    claim.consultation_id = None;
                }
            }
        }
        for claim in &mut profile.billing.dva_claims {
            if let Some(cid) = claim.consultation_id {
                if !valid_consultation_ids.contains(&cid) {
                    claim.consultation_id = None;
                }
            }
        }
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
            future_percentage: 0.5,
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
                    match insert_appointment(&pool, appointment).await {
                        Ok(_) => stats.appointments_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("appointment", &e);
                        }
                    }
                }

                for consultation in &profile.consultations {
                    match insert_consultation(&pool, &crypto, consultation).await {
                        Ok(_) => stats.consultations_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("consultation", &e);
                        }
                    }
                }

                for history in &profile.medical_history {
                    match insert_medical_history(&pool, &crypto, history).await {
                        Ok(_) => stats.medical_history_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("medical_history", &e);
                        }
                    }
                }

                for allergy in &profile.allergies {
                    match insert_allergy(&pool, &crypto, allergy).await {
                        Ok(_) => stats.allergies_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("allergy", &e);
                        }
                    }
                }

                match insert_social_history(&pool, &crypto, &social_history).await {
                    Ok(_) => stats.social_history_created += 1,
                    Err(e) => {
                        stats.record_failures += 1;
                        log_err("social_history", &e);
                    }
                }

                for vital in &profile.vitals {
                    match insert_vitals(&pool, vital, default_actor).await {
                        Ok(_) => stats.vitals_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("vitals", &e);
                        }
                    }
                }

                for fh in &profile.family_history {
                    match insert_family_history(&pool, &crypto, fh, default_actor).await {
                        Ok(_) => stats.family_history_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("family_history", &e);
                        }
                    }
                }

                let mut inserted_invoice_ids: std::collections::HashSet<Uuid> =
                    std::collections::HashSet::new();
                for invoice in &profile.billing.invoices {
                    match insert_invoice(&pool, invoice).await {
                        Ok(_) => {
                            stats.invoices_created += 1;
                            inserted_invoice_ids.insert(invoice.id);
                        }
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("invoice", &e);
                        }
                    }
                }

                for claim in &profile.billing.medicare_claims {
                    let mut c = claim.clone();
                    if let Some(iid) = c.invoice_id {
                        if !inserted_invoice_ids.contains(&iid) {
                            c.invoice_id = None;
                        }
                    }
                    match insert_medicare_claim(&pool, &c).await {
                        Ok(_) => stats.medicare_claims_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("medicare_claim", &e);
                        }
                    }
                }

                for claim in &profile.billing.dva_claims {
                    match insert_dva_claim(&pool, claim).await {
                        Ok(_) => stats.dva_claims_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("dva_claim", &e);
                        }
                    }
                }

                for payment in &profile.billing.payments {
                    if !inserted_invoice_ids.contains(&payment.invoice_id) {
                        stats.record_failures += 1;
                        continue;
                    }
                    match insert_payment(&pool, payment, default_actor).await {
                        Ok(_) => stats.payments_created += 1,
                        Err(e) => {
                            stats.record_failures += 1;
                            log_err("payment", &e);
                        }
                    }
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
    println!("  ✓ Vital signs records: {}", stats.vitals_created);
    println!(
        "  ✓ Family history records: {}",
        stats.family_history_created
    );
    println!("  ✓ Invoices created: {}", stats.invoices_created);
    println!(
        "  ✓ Medicare claims created: {}",
        stats.medicare_claims_created
    );
    println!("  ✓ DVA claims created: {}", stats.dva_claims_created);
    println!("  ✓ Payments created: {}", stats.payments_created);
    println!("  ✗ Patient insert failures: {}", stats.patient_failures);
    println!("  ✗ Related-record failures: {}", stats.record_failures);
    if !error_samples.is_empty() {
        println!();
        println!("Sample errors (first {}):", error_samples.len());
        for msg in &error_samples {
            println!("  • {msg}");
        }
    }
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
    ensure_column_is_bytea(pool, "family_history", "notes").await?;
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

async fn insert_vitals(
    pool: &PgPool,
    vitals: &VitalSigns,
    actor_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO vital_signs (
            id, patient_id, consultation_id, measured_at,
            systolic_bp, diastolic_bp, heart_rate, respiratory_rate,
            temperature, oxygen_saturation, height_cm, weight_kg, bmi,
            notes, created_at, created_by
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8,
            $9, $10, $11, $12, $13,
            $14, $15, $16
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(vitals.id)
    .bind(vitals.patient_id)
    .bind(vitals.consultation_id)
    .bind(vitals.measured_at)
    .bind(vitals.systolic_bp.map(i32::from))
    .bind(vitals.diastolic_bp.map(i32::from))
    .bind(vitals.heart_rate.map(i32::from))
    .bind(vitals.respiratory_rate.map(i32::from))
    .bind(vitals.temperature)
    .bind(vitals.oxygen_saturation.map(i32::from))
    .bind(vitals.height_cm.map(i32::from))
    .bind(vitals.weight_kg)
    .bind(vitals.bmi)
    .bind(&vitals.notes)
    .bind(vitals.created_at)
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_family_history(
    pool: &PgPool,
    crypto: &EncryptionService,
    fh: &FamilyHistory,
    actor_id: Uuid,
) -> Result<(), sqlx::Error> {
    let encrypted_notes = fh
        .notes
        .as_ref()
        .map(|text| crypto.encrypt(text))
        .transpose()
        .map_err(|e| sqlx::Error::Protocol(format!("encryption failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO family_history (
            id, patient_id, relative_relationship, condition,
            age_at_diagnosis, notes, created_at, created_by
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(fh.id)
    .bind(fh.patient_id)
    .bind(&fh.relative_relationship)
    .bind(&fh.condition)
    .bind(fh.age_at_diagnosis.map(i32::from))
    .bind(encrypted_notes)
    .bind(fh.created_at)
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_invoice(pool: &PgPool, invoice: &Invoice) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, invoice_number, patient_id, practitioner_id, consultation_id,
            billing_type, status, issue_date, due_date,
            subtotal, gst_amount, total_amount, amount_paid, amount_outstanding,
            notes, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9,
            $10, $11, $12, $13, $14,
            $15, $16, $17
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(invoice.id)
    .bind(&invoice.invoice_number)
    .bind(invoice.patient_id)
    .bind(invoice.practitioner_id)
    .bind(invoice.consultation_id)
    .bind(invoice.billing_type.to_string())
    .bind(invoice.status.to_string())
    .bind(invoice.invoice_date)
    .bind(invoice.due_date)
    .bind(invoice.subtotal)
    .bind(invoice.gst_amount)
    .bind(invoice.total_amount)
    .bind(invoice.amount_paid)
    .bind(invoice.amount_outstanding)
    .bind(&invoice.notes)
    .bind(invoice.created_at)
    .bind(invoice.updated_at)
    .execute(pool)
    .await?;

    for item in &invoice.items {
        sqlx::query(
            r#"
            INSERT INTO invoice_items (
                id, invoice_id, description, item_code,
                quantity, unit_price, amount, is_gst_free,
                created_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9
            )
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(item.id)
        .bind(invoice.id)
        .bind(&item.description)
        .bind(&item.item_code)
        .bind(item.quantity as i32)
        .bind(item.unit_price)
        .bind(item.amount)
        .bind(item.is_gst_free)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn insert_medicare_claim(pool: &PgPool, claim: &MedicareClaim) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO medicare_claims (
            id, invoice_id, patient_id, practitioner_id,
            claim_type, status, service_date,
            total_claimed, total_benefit, reference_number,
            submitted_at, processed_at, created_at
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7,
            $8, $9, $10,
            $11, $12, $13
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(claim.id)
    .bind(claim.invoice_id)
    .bind(claim.patient_id)
    .bind(claim.practitioner_id)
    .bind(claim.claim_type.to_string())
    .bind(claim.status.to_string())
    .bind(claim.service_date)
    .bind(claim.total_claimed)
    .bind(claim.total_benefit)
    .bind(&claim.claim_reference)
    .bind(claim.submitted_at)
    .bind(claim.processed_at)
    .bind(claim.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_dva_claim(pool: &PgPool, claim: &DVAClaim) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO dva_claims (
            id, patient_id, practitioner_id, consultation_id,
            dva_file_number, card_type, service_date,
            items, total_claimed, status,
            submitted_at, processed_at, created_at, created_by
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7,
            $8, $9, $10,
            $11, $12, $13, $14
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(claim.id)
    .bind(claim.patient_id)
    .bind(claim.practitioner_id)
    .bind(claim.consultation_id)
    .bind(&claim.dva_file_number)
    .bind(claim.card_type.to_string())
    .bind(claim.service_date)
    .bind(sqlx::types::Json(&claim.items))
    .bind(claim.total_claimed)
    .bind(claim.status.to_string())
    .bind(claim.submitted_at)
    .bind(claim.processed_at)
    .bind(claim.created_at)
    .bind(claim.created_by)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_payment(
    pool: &PgPool,
    payment: &Payment,
    actor_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, invoice_id, patient_id, amount,
            payment_method, payment_date, reference, notes,
            created_by, created_at
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8,
            $9, $10
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(payment.id)
    .bind(payment.invoice_id)
    .bind(payment.patient_id)
    .bind(payment.amount)
    .bind(payment.payment_method.to_string())
    .bind(payment.payment_date.date_naive())
    .bind(&payment.reference)
    .bind(&payment.notes)
    .bind(actor_id)
    .bind(payment.created_at)
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
    let empty: usize = width.saturating_sub(filled);

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
