use opengp::config::Config;
use opengp::domain::patient::PatientRepository;
use opengp::infrastructure::database::repositories::SqlxPatientRepository;
use opengp::infrastructure::database::{create_pool, run_migrations};
use opengp::infrastructure::fixtures::{PatientGenerator, PatientGeneratorConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OpenGP Database Seeding Tool\n");
    println!("============================\n");

    let config = Config::from_env()?;
    let pool = create_pool(&config.database).await?;
    run_migrations(&pool).await?;

    println!("Database connected: {}\n", config.database.url);

    let patient_repository = Arc::new(SqlxPatientRepository::new(pool.clone()));

    println!("Ready to seed database\n");

    let gen_config = PatientGeneratorConfig {
        count: 1000,
        min_age: 0,
        max_age: 100,
        include_children: true,
        include_seniors: true,
        medicare_percentage: 0.95,
        ihi_percentage: 0.90,
        mobile_percentage: 0.85,
        email_percentage: 0.70,
    };

    println!("Generating {} patients...", gen_config.count);
    let mut generator = PatientGenerator::new(gen_config);
    let patients = generator.generate();

    println!("Inserting patients into database...\n");

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
                eprintln!("  Error inserting patient {}: {}", patient.first_name, e);
            }
        }
    }

    println!();
    println!("============================");
    println!("Results:");
    println!("============================");
    println!("  Successfully inserted: {}", success_count);
    println!("  Errors: {}", error_count);
    println!();

    if error_count > 0 {
        println!("Warning: Some patients failed to insert. Check error messages above.");
        return Err("Seeding completed with errors".into());
    }

    println!("Database seeding completed successfully!");

    Ok(())
}
