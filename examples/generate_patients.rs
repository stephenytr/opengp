use opengp::domain::patient::Patient;
use opengp::infrastructure::fixtures::{PatientGenerator, PatientGeneratorConfig};

fn main() {
    println!("OpenGP Patient Generator Example\n");
    println!("=================================\n");

    let config = PatientGeneratorConfig {
        count: 1000,
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
    };

    println!("Configuration:");
    println!("  Count: {}", config.count);
    println!("  Age range: {}-{}", config.min_age, config.max_age);
    println!("  Include children: {}", config.include_children);
    println!("  Include seniors: {}", config.include_seniors);
    println!(
        "  Medicare percentage: {}%",
        config.medicare_percentage * 100.0
    );
    println!("  IHI percentage: {}%", config.ihi_percentage * 100.0);
    println!("  Mobile percentage: {}%", config.mobile_percentage * 100.0);
    println!("  Email percentage: {}%", config.email_percentage * 100.0);
    println!();

    let mut generator = PatientGenerator::new(config);
    let patients = generator.generate();

    println!("Generated {} patients:\n", patients.len());

    for (i, patient) in patients.iter().enumerate() {
        print_patient_summary(i + 1, patient);
    }

    print_statistics(&patients);
}

fn print_patient_summary(num: usize, patient: &Patient) {
    let name = format!(
        "{}, {}",
        patient.last_name,
        patient
            .preferred_name
            .as_ref()
            .unwrap_or(&patient.first_name)
    );

    let age = patient.age();
    let gender = match patient.gender {
        opengp::domain::patient::Gender::Male => "M",
        opengp::domain::patient::Gender::Female => "F",
        opengp::domain::patient::Gender::Other => "O",
        opengp::domain::patient::Gender::PreferNotToSay => "P",
    };

    let medicare = patient
        .medicare_number
        .as_ref()
        .map(|m| {
            if let Some(irn) = patient.medicare_irn {
                format!("{}-{}", m, irn)
            } else {
                m.clone()
            }
        })
        .unwrap_or_else(|| "-".to_string());

    let phone = patient
        .phone_mobile
        .as_ref()
        .or(patient.phone_home.as_ref())
        .cloned()
        .unwrap_or_else(|| "-".to_string());

    println!(
        "{:3}. {:30} Age: {:3} ({}) Medicare: {:15} Phone: {}",
        num, name, age, gender, medicare, phone
    );
}

fn print_statistics(patients: &[Patient]) {
    println!("\n=================================");
    println!("Statistics:");
    println!("=================================\n");

    let male_count = patients
        .iter()
        .filter(|p| p.gender == opengp::domain::patient::Gender::Male)
        .count();
    let female_count = patients
        .iter()
        .filter(|p| p.gender == opengp::domain::patient::Gender::Female)
        .count();

    println!("Gender Distribution:");
    println!("  Male: {}", male_count);
    println!("  Female: {}", female_count);
    println!();

    let children = patients.iter().filter(|p| p.age() < 18).count();
    let adults = patients
        .iter()
        .filter(|p| p.age() >= 18 && p.age() < 65)
        .count();
    let seniors = patients.iter().filter(|p| p.age() >= 65).count();

    println!("Age Distribution:");
    println!("  Children (<18): {}", children);
    println!("  Adults (18-64): {}", adults);
    println!("  Seniors (65+): {}", seniors);
    println!();

    let with_medicare = patients
        .iter()
        .filter(|p| p.medicare_number.is_some())
        .count();
    let with_ihi = patients.iter().filter(|p| p.ihi.is_some()).count();
    let with_mobile = patients.iter().filter(|p| p.phone_mobile.is_some()).count();
    let with_email = patients.iter().filter(|p| p.email.is_some()).count();

    println!("Contact Information:");
    println!(
        "  With Medicare: {} ({:.1}%)",
        with_medicare,
        (with_medicare as f32 / patients.len() as f32) * 100.0
    );
    println!(
        "  With IHI: {} ({:.1}%)",
        with_ihi,
        (with_ihi as f32 / patients.len() as f32) * 100.0
    );
    println!(
        "  With Mobile: {} ({:.1}%)",
        with_mobile,
        (with_mobile as f32 / patients.len() as f32) * 100.0
    );
    println!(
        "  With Email: {} ({:.1}%)",
        with_email,
        (with_email as f32 / patients.len() as f32) * 100.0
    );
}
