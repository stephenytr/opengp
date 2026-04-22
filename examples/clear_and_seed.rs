use opengp::config::Config;
use opengp::domain::patient::PatientRepository;
use opengp::infrastructure::crypto::EncryptionService;
use opengp::infrastructure::database::repositories::SqlxPatientRepository;
use opengp::infrastructure::database::{create_pool, run_migrations};
use opengp::infrastructure::fixtures::{PatientGenerator, PatientGeneratorConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OpenGP Clear & Seed Tool\n");
    println!("========================\n");

    println!("⚠️  WARNING: This will DELETE ALL patients and reseed with fresh data!\n");

    let config = Config::from_env()?;
    let pool = create_pool(&config.app.api_server.database).await?;
    run_migrations(&pool).await?;
    let pool = pool.as_postgres().clone();

    println!("Database: {}\n", config.app.api_server.database.url);

    let crypto = Arc::new(EncryptionService::new()?);
    let patient_repository = Arc::new(SqlxPatientRepository::new(pool.clone(), crypto));

    println!("⚠️  Note: This tool adds patients without clearing existing data\n");
    println!("For a clean slate, delete the database file and run again.\n");

    let gen_config = PatientGeneratorConfig {
        count: 30,
        min_age: 0,
        max_age: 100,
        include_children: true,
        include_seniors: true,
        medicare_percentage: 0.95,
        ihi_percentage: 0.90,
        mobile_percentage: 0.85,
        email_percentage: 0.70,
        atsi_percentage: 0.05,
        concession_percentage: 0.15,
        emergency_contact_percentage: 0.70,
        interpreter_percentage: 0.10,
        preferred_name_percentage: 0.30,
        middle_name_percentage: 0.60,
        use_australian_names: true,
        family_medicare_percentage: 0.10,
        avg_family_size: 2.5,
    };

    println!("Generating {} fresh patients...", gen_config.count);
    let mut generator = PatientGenerator::new(gen_config);
    let patients = generator.generate();

    println!("Inserting into database...\n");

    let mut success_count = 0;
    let mut error_count = 0;

    for (i, patient) in patients.iter().enumerate() {
        match patient_repository.create(patient.clone()).await {
            Ok(_) => {
                success_count += 1;
                if (i + 1) % 10 == 0 {
                    println!("  Inserted {} patients...", i + 1);
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("  Error: {}", e);
            }
        }
    }

    println!();
    println!("========================");
    println!("Complete!");
    println!("========================");
    println!("  Inserted: {}", success_count);
    println!("  Errors: {}", error_count);
    println!();

    if error_count == 0 {
        println!("✓ Database successfully cleared and seeded!");
    } else {
        println!("⚠  Completed with {} errors", error_count);
        return Err("Seeding completed with errors".into());
    }

    Ok(())
}
