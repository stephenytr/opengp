//! Example: Generate comprehensive patient profiles with all clinical data
//!
//! This example demonstrates how to use the ComprehensivePatientGenerator
//! to create complete patient profiles including demographics, medical history,
//! allergies, and consultation records.
//!
//! Run with: cargo run --example generate_comprehensive_patients

use opengp_infrastructure::infrastructure::fixtures::{
    ClinicalDataGeneratorConfig, ComprehensivePatientGenerator,
    ComprehensivePatientGeneratorConfig, PatientGeneratorConfig,
};
use uuid::Uuid;

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  OpenGP Comprehensive Patient Generator Example");
    println!("═══════════════════════════════════════════════════════════\n");

    // Example 1: Generate 5 patients with default settings
    println!("Example 1: Generate 5 patients with default settings");
    println!("─────────────────────────────────────────────────────────");

    let config = ComprehensivePatientGeneratorConfig {
        patient_count: 5,
        ..Default::default()
    };

    let generator = ComprehensivePatientGenerator::new(config);
    let profiles = generator.generate();

    for (i, profile) in profiles.iter().enumerate() {
        println!(
            "\n  Patient {}: {} {} (DOB: {})",
            i + 1,
            profile.patient.first_name,
            profile.patient.last_name,
            profile.patient.date_of_birth
        );
        println!("    Medicare: {:?}", profile.patient.medicare_number);
        println!("    Medical conditions: {}", profile.medical_history.len());
        println!("    Allergies: {}", profile.allergies.len());
        println!("    Consultations: {}", profile.consultations.len());
    }

    // Example 2: Generate senior patients with more clinical data
    println!("\n\nExample 2: Generate 3 senior patients (65+) with extensive clinical data");
    println!("─────────────────────────────────────────────────────────");

    let patient_config = PatientGeneratorConfig {
        count: 1,
        min_age: 65,
        max_age: 85,
        include_children: false,
        include_seniors: true,
        medicare_percentage: 1.0,
        ihi_percentage: 0.95,
        mobile_percentage: 0.70,
        email_percentage: 0.60,
        emergency_contact_percentage: 0.90,
        concession_percentage: 0.50,
        atsi_percentage: 0.05,
        interpreter_percentage: 0.05,
        preferred_name_percentage: 0.15,
        middle_name_percentage: 0.60,
        use_australian_names: true,
    };

    let clinical_config = ClinicalDataGeneratorConfig {
        consultation_count: 8,
        medical_history_count: 5,
        allergy_count: 2,
        notes_percentage: 0.90,
        signed_percentage: 0.85,
        severe_allergy_percentage: 0.15,
    };

    let config = ComprehensivePatientGeneratorConfig {
        patient_count: 3,
        patient_config,
        clinical_config,
        practitioner_ids: Vec::new(),
    };

    let generator = ComprehensivePatientGenerator::new(config);
    let profiles = generator.generate();

    for (i, profile) in profiles.iter().enumerate() {
        let age = chrono::Utc::now()
            .date_naive()
            .years_since(profile.patient.date_of_birth)
            .unwrap_or(0);

        println!(
            "\n  Senior Patient {}: {} {} (Age: {})",
            i + 1,
            profile.patient.first_name,
            profile.patient.last_name,
            age
        );
        println!("    Address: {:?}", profile.patient.address.suburb);
        println!(
            "    Emergency contact: {:?}",
            profile.patient.emergency_contact.is_some()
        );
        println!("    Concession type: {:?}", profile.patient.concession_type);
        println!("    Medical conditions: {}", profile.medical_history.len());

        for (j, condition) in profile.medical_history.iter().enumerate() {
            println!("      {}. {:?}", j + 1, condition.condition);
        }

        println!("    Allergies: {}", profile.allergies.len());
        for (j, allergy) in profile.allergies.iter().enumerate() {
            println!(
                "      {}. {:?} ({})",
                j + 1,
                allergy.allergen,
                allergy.severity
            );
        }

        println!("    Consultations: {}", profile.consultations.len());
    }

    // Example 3: Generate patients with specific practitioners
    println!("\n\nExample 3: Generate 10 patients assigned to 2 practitioners");
    println!("─────────────────────────────────────────────────────────");

    let practitioner_ids = vec![
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
    ];

    let config = ComprehensivePatientGeneratorConfig {
        patient_count: 10,
        practitioner_ids: practitioner_ids.clone(),
        ..Default::default()
    };

    let generator = ComprehensivePatientGenerator::new(config);
    let profiles = generator.generate();

    let mut practitioner_counts = std::collections::HashMap::new();
    for profile in &profiles {
        for consultation in &profile.consultations {
            *practitioner_counts
                .entry(consultation.practitioner_id)
                .or_insert(0) += 1;
        }
    }

    println!("\n  Generated {} patients", profiles.len());
    for (practitioner_id, count) in practitioner_counts {
        println!(
            "    Practitioner {}: {} consultations",
            practitioner_id, count
        );
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Generation complete!");
    println!("═══════════════════════════════════════════════════════════\n");
}
