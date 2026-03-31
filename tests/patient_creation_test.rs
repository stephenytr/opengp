use chrono::NaiveDate;
use opengp_domain::domain::patient::{
    Address, Gender, NewPatientData, PatientRepository, PatientService,
};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::SqlxPatientRepository;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

static MIGRATIONS: OnceCell<()> = OnceCell::const_new();

async fn setup_test_database() -> PgPool {
    let database_url = std::env::var("API_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://opengp:opengp_dev_password@127.0.0.1:5432/opengp".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL test database");

    MIGRATIONS
        .get_or_init(|| async {
            if let Err(err) = sqlx::migrate!("./migrations").run(&pool).await {
                let msg = err.to_string();
                assert!(
                    msg.contains("users_pkey") && msg.contains("duplicate key value"),
                    "Failed to run migrations: {err}"
                );
            }
        })
        .await;

    pool
}

fn generate_unique_medicare() -> String {
    let random_num = Uuid::new_v4().as_u128() % 10000000000;
    format!("{:010}", random_num)
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_create_patient_with_database() {
    let pool = setup_test_database().await;

    // Initialize encryption service for tests
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));

    let repository: Arc<dyn PatientRepository> = Arc::new(SqlxPatientRepository::new(pool, crypto));
    let service = PatientService::new(repository);

    let medicare_number = generate_unique_medicare();

    let data = NewPatientData {
        ihi: None,
        medicare_number: Some(medicare_number.clone().into()),
        medicare_irn: Some(1),
        medicare_expiry: None,
        title: Some("Mr".to_string()),
        first_name: "Test".to_string(),
        middle_name: None,
        last_name: "Patient".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
        gender: Gender::Male,
        address: Address::default(),
        phone_home: None,
        phone_mobile: Some("0412345678".to_string().into()),
        email: Some("test@example.com".to_string()),
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: None,
        interpreter_required: None,
        aboriginal_torres_strait_islander: None,
    };

    let result = service.register_patient(data).await;

    assert!(
        result.is_ok(),
        "Failed to create patient: {:?}",
        result.err()
    );

    let patient = result.unwrap();
    assert_eq!(patient.first_name, "Test");
    assert_eq!(patient.last_name, "Patient");
    assert_eq!(
        patient.medicare_number.as_ref().map(|m| m.as_str()),
        Some(medicare_number.as_str())
    );
    assert!(patient.is_active);
    assert!(!patient.is_deceased);
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_duplicate_medicare_number() {
    let pool = setup_test_database().await;

    // Initialize encryption service for tests
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));

    let repository: Arc<dyn PatientRepository> = Arc::new(SqlxPatientRepository::new(pool, crypto));
    let service = PatientService::new(repository);

    let unique_medicare = generate_unique_medicare();

    let data = NewPatientData {
        ihi: None,
        medicare_number: Some(unique_medicare.into()),
        medicare_irn: Some(1),
        medicare_expiry: None,
        title: Some("Ms".to_string()),
        first_name: "Duplicate".to_string(),
        middle_name: None,
        last_name: "Test".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(1985, 6, 15).unwrap(),
        gender: Gender::Female,
        address: Address::default(),
        phone_home: None,
        phone_mobile: Some("0423456789".to_string().into()),
        email: None,
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: None,
        interpreter_required: None,
        aboriginal_torres_strait_islander: None,
    };

    let first_result = service.register_patient(data.clone()).await;
    assert!(first_result.is_ok());

    let second_result = service.register_patient(data).await;
    assert!(
        second_result.is_err(),
        "Should fail with duplicate Medicare number"
    );
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_find_patient_by_id() {
    let pool = setup_test_database().await;

    // Initialize encryption service for tests
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));

    let repository: Arc<dyn PatientRepository> = Arc::new(SqlxPatientRepository::new(pool, crypto));
    let service = PatientService::new(repository.clone());

    let data = NewPatientData {
        ihi: None,
        medicare_number: Some(generate_unique_medicare().into()),
        medicare_irn: Some(2),
        medicare_expiry: None,
        title: None,
        first_name: "FindMe".to_string(),
        middle_name: None,
        last_name: "Test".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(2000, 12, 25).unwrap(),
        gender: Gender::Other,
        address: Address::default(),
        phone_home: None,
        phone_mobile: None,
        email: None,
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: None,
        interpreter_required: None,
        aboriginal_torres_strait_islander: None,
    };

    let created = service.register_patient(data).await.unwrap();
    let patient_id = created.id;

    let found = repository.find_by_id(patient_id).await.unwrap();

    assert!(found.is_some());
    let found_patient = found.unwrap();
    assert_eq!(found_patient.id, patient_id);
    assert_eq!(found_patient.first_name, "FindMe");
    assert_eq!(found_patient.last_name, "Test");
}
