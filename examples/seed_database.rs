use opengp::config::Config;
use opengp::domain::patient::PatientRepository;
use opengp::infrastructure::crypto::EncryptionService;
use opengp::infrastructure::database::repositories::SqlxPatientRepository;
use opengp::infrastructure::database::{create_pool, run_migrations};
use opengp::infrastructure::fixtures::{PatientGenerator, PatientGeneratorConfig};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut patient_count = 100;
    let mut clear_first = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--patients" | "-p" => {
                if i + 1 < args.len() {
                    patient_count = args[i + 1].parse().unwrap_or(100);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--clear" | "-c" => {
                clear_first = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("═══════════════════════════════════════════════════════════");
    println!("   OpenGP Database Seeding Tool");
    println!("═══════════════════════════════════════════════════════════\n");

    let config = Config::from_env()?;
    let pool = create_pool(&config.database).await?;
    run_migrations(&pool).await?;

    println!("✓ Database connected: {}\n", config.database.url);

    let crypto = Arc::new(EncryptionService::new()?);
    let patient_repository = Arc::new(SqlxPatientRepository::new(pool.clone(), crypto));

    println!("Configuration:");
    println!("  • Patient count: {}", patient_count);
    println!("  • Clear before seed: {}\n", if clear_first { "Yes" } else { "No" });

    let gen_config = PatientGeneratorConfig {
        count: patient_count,
        min_age: 0,
        max_age: 100,
        include_children: true,
        include_seniors: true,
        medicare_percentage: 0.95,
        ihi_percentage: 0.90,
        mobile_percentage: 0.85,
        email_percentage: 0.70,
        emergency_contact_percentage: 0.70,
        concession_percentage: 0.25,
        atsi_percentage: 0.05,
        interpreter_percentage: 0.05,
        preferred_name_percentage: 0.15,
        middle_name_percentage: 0.60,
        use_australian_names: true,
    };

    print!("Generating {} patients...", gen_config.count);
    let mut generator = PatientGenerator::new(gen_config);
    let patients = generator.generate();
    println!(" ✓\n");

    println!("Inserting patients into database...");

    let mut success_count = 0;
    let mut error_count = 0;
    let total = patients.len();
    let progress_interval = (total / 20).max(1);

    for (i, patient) in patients.iter().enumerate() {
        match patient_repository.create(patient.clone()).await {
            Ok(_) => {
                success_count += 1;
                if (i + 1) % progress_interval == 0 || i == 0 || i == total - 1 {
                    let percentage = ((i + 1) as f32 / total as f32 * 100.0) as u32;
                    let bar_filled = (percentage / 5) as usize;
                    let bar_empty = 20 - bar_filled;
                    print!(
                        "\r  [{}{}] {}/{}",
                        "█".repeat(bar_filled),
                        "░".repeat(bar_empty),
                        i + 1,
                        total
                    );
                    use std::io::{self, Write};
                    let _ = io::stdout().flush();
                }
            }
            Err(e) => {
                error_count += 1;
                println!("\n  ✗ Error inserting patient {}: {}", patient.first_name, e);
            }
        }
    }

    println!("\n");
    println!("═══════════════════════════════════════════════════════════");
    println!("Results:");
    println!("═══════════════════════════════════════════════════════════");
    println!("  ✓ Successfully inserted: {}", success_count);
    println!("  ✗ Errors: {}", error_count);
    println!();

    if error_count > 0 {
        println!("⚠ Warning: Some patients failed to insert. Check messages above.");
        println!();
        return Err("Seeding completed with errors".into());
    }

    println!("✓ Database seeding completed successfully!");
    println!();

    Ok(())
}

fn print_help() {
    println!("OpenGP Database Seeding Tool");
    println!();
    println!("Usage: cargo run --example seed_database -- [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -p, --patients <COUNT>  Number of patients to generate (default: 100)");
    println!("  -c, --clear             Clear existing data before seeding (not implemented)");
    println!("  -h, --help              Show this help message");
    println!();
    println!("Examples:");
    println!("  # Generate 500 patients");
    println!("  cargo run --example seed_database -- --patients 500");
    println!();
    println!("  # Generate 1000 patients");
    println!("  cargo run --example seed_database -- -p 1000");
}
