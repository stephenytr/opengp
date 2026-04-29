use opengp::infrastructure::fixtures::{
    AppointmentHistoryGeneratorConfig, BillingGeneratorConfig, ClinicalDataGeneratorConfig,
    ComprehensiveFixtureGenerator, ComprehensiveFixtureGeneratorConfig, PatientGeneratorConfig,
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("OpenGP Comprehensive Fixture Generator Example\n");
    println!("==============================================\n");

    let config = ComprehensiveFixtureGeneratorConfig {
        patient_count: 50,
        patient_config: PatientGeneratorConfig {
            count: 50,
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
            family_medicare_percentage: 0.20,
            avg_family_size: 2.5,
        },
        appointment_history_config: AppointmentHistoryGeneratorConfig {
            min_appointments_per_patient: 2,
            max_appointments_per_patient: 8,
            ..Default::default()
        },
        billing_config: BillingGeneratorConfig {
            bulk_billing_percentage: 0.40,
            private_billing_percentage: 0.50,
            dva_percentage: 0.10,
            medicare_claim_percentage: 0.70,
            invoice_paid_percentage: 0.80,
            invoice_overdue_percentage: 0.10,
            average_items_per_invoice: 3,
            max_items_per_invoice: 6,
        },
        clinical_config: ClinicalDataGeneratorConfig {
            consultation_count: 3,
            medical_history_count: 2,
            allergy_count: 1,
            ..Default::default()
        },
        practitioner_ids: vec![Uuid::new_v4()],
    };

    println!("Configuration:");
    println!("  Patient count: {}", config.patient_count);
    println!(
        "  Min appointments per patient: {}",
        config
            .appointment_history_config
            .min_appointments_per_patient
    );
    println!(
        "  Max appointments per patient: {}",
        config
            .appointment_history_config
            .max_appointments_per_patient
    );
    println!(
        "  Bulk billing: {}%",
        config.billing_config.bulk_billing_percentage * 100.0
    );
    println!(
        "  Private billing: {}%",
        config.billing_config.private_billing_percentage * 100.0
    );
    println!(
        "  DVA billing: {}%",
        config.billing_config.dva_percentage * 100.0
    );
    println!();

    print!("Generating comprehensive fixtures...");
    let mut generator = ComprehensiveFixtureGenerator::new(config);
    let profiles = generator.generate();
    println!(" ✓\n");

    print_summary_statistics(&profiles);
}

fn print_summary_statistics(
    profiles: &[opengp::infrastructure::fixtures::ComprehensiveFixtureProfile],
) {
    println!("==============================================");
    println!("Summary Statistics:");
    println!("==============================================\n");

    println!("Patient Data:");
    println!("  Total patients generated: {}", profiles.len());

    let mut employment_dist: HashMap<String, usize> = HashMap::new();
    let mut dva_dist: HashMap<String, usize> = HashMap::new();

    for profile in profiles {
        if let Some(ref emp) = profile.patient.employment_status {
            *employment_dist.entry(format!("{:?}", emp)).or_insert(0) += 1;
        }
        if let Some(ref dva) = profile.patient.dva_card_type {
            *dva_dist.entry(format!("{:?}", dva)).or_insert(0) += 1;
        }
    }

    if !employment_dist.is_empty() {
        println!("\n  Employment Status Distribution:");
        for (status, count) in employment_dist.iter() {
            println!("    {}: {}", status, count);
        }
    }

    if !dva_dist.is_empty() {
        println!("\n  DVA Card Type Distribution:");
        for (card_type, count) in dva_dist.iter() {
            println!("    {}: {}", card_type, count);
        }
    }

    let total_appointments: usize = profiles.iter().map(|p| p.appointments.len()).sum();
    println!("\nAppointment Data:");
    println!("  Total appointments generated: {}", total_appointments);
    println!(
        "  Average appointments per patient: {:.1}",
        total_appointments as f32 / profiles.len() as f32
    );

    let total_invoices: usize = profiles.iter().map(|p| p.billing.invoices.len()).sum();
    let total_medicare_claims: usize = profiles
        .iter()
        .map(|p| p.billing.medicare_claims.len())
        .sum();
    let total_dva_claims: usize = profiles.iter().map(|p| p.billing.dva_claims.len()).sum();
    let total_payments: usize = profiles.iter().map(|p| p.billing.payments.len()).sum();

    println!("\nBilling Data:");
    println!("  Total invoices: {}", total_invoices);
    println!("  Total Medicare claims: {}", total_medicare_claims);
    println!("  Total DVA claims: {}", total_dva_claims);
    println!("  Total payments: {}", total_payments);

    let total_medical_history: usize = profiles.iter().map(|p| p.medical_history.len()).sum();
    let total_allergies: usize = profiles.iter().map(|p| p.allergies.len()).sum();
    let total_consultations: usize = profiles.iter().map(|p| p.consultations.len()).sum();

    println!("\nClinical Data:");
    println!("  Total medical history records: {}", total_medical_history);
    println!("  Total allergy records: {}", total_allergies);
    println!("  Total consultations: {}", total_consultations);

    println!("\n==============================================");
    println!("✓ Comprehensive fixture generation complete!");
    println!("==============================================");
}
