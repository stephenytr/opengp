use chrono::{Duration, NaiveDate, Utc};
use opengp_domain::domain::audit::{AuditEmitter, AuditRepository, AuditService};
use opengp_domain::domain::clinical::{
    ClinicalRepositories, ClinicalService, ConsultationRepository, NewConsultationData,
};
use opengp_domain::domain::patient::{
    Address, Gender, NewPatientData, PatientRepository, PatientService,
};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::{
    SqlxAllergyRepository, SqlxAuditRepository, SqlxClinicalRepository,
    SqlxFamilyHistoryRepository, SqlxMedicalHistoryRepository, SqlxPatientRepository,
    SqlxSocialHistoryRepository, SqlxVitalSignsRepository,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_database() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn create_mock_audit_service(pool: &SqlitePool) -> Arc<dyn AuditEmitter> {
    let audit_repository: Arc<dyn AuditRepository> =
        Arc::new(SqlxAuditRepository::new(pool.clone()));
    Arc::new(AuditService::new(audit_repository))
}

async fn create_test_patient(pool: &SqlitePool) -> Uuid {
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let repository: Arc<dyn PatientRepository> =
        Arc::new(SqlxPatientRepository::new(pool.clone(), crypto));
    let service = PatientService::new(repository);

    let data = NewPatientData {
        ihi: None,
        medicare_number: Some(format!("{:010}", Uuid::new_v4().as_u128() % 10000000000)),
        medicare_irn: Some(1),
        medicare_expiry: None,
        title: Some("Ms".to_string()),
        first_name: "Test".to_string(),
        middle_name: None,
        last_name: "Patient".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(1985, 6, 15).unwrap(),
        gender: Gender::Female,
        address: Address::default(),
        phone_home: None,
        phone_mobile: Some("0412345678".to_string()),
        email: Some("test.patient@example.com".to_string()),
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: None,
        interpreter_required: None,
        aboriginal_torres_strait_islander: None,
    };

    service
        .register_patient(data)
        .await
        .expect("Failed to create test patient")
        .id
}

async fn create_test_practitioner(pool: &SqlitePool) -> Uuid {
    let id = Uuid::new_v4();
    let username = format!("dr_{}", id);
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, is_active, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(username)
    .bind("test_hash")
    .bind("Doctor")
    .bind(true)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create test practitioner");

    id
}

fn create_clinical_service(pool: &SqlitePool) -> ClinicalService {
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));

    let consultation_repo: Arc<dyn ConsultationRepository> =
        Arc::new(SqlxClinicalRepository::new(pool.clone(), crypto.clone()));

    let patient_crypto =
        Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let patient_repository: Arc<dyn PatientRepository> =
        Arc::new(SqlxPatientRepository::new(pool.clone(), patient_crypto));
    let patient_service = Arc::new(PatientService::new(patient_repository));

    let audit_service = create_mock_audit_service(pool);

    let repos = ClinicalRepositories {
        consultation: consultation_repo,
        allergy: Arc::new(SqlxAllergyRepository::new(pool.clone(), crypto.clone())),
        medical_history: Arc::new(SqlxMedicalHistoryRepository::new(
            pool.clone(),
            crypto.clone(),
        )),
        vital_signs: Arc::new(SqlxVitalSignsRepository::new(pool.clone(), crypto.clone())),
        social_history: Arc::new(SqlxSocialHistoryRepository::new(
            pool.clone(),
            crypto.clone(),
        )),
        family_history: Arc::new(SqlxFamilyHistoryRepository::new(
            pool.clone(),
            crypto.clone(),
        )),
    };

    ClinicalService::new(repos, patient_service, audit_service)
}

#[tokio::test]
async fn test_create_consultation_with_reason() {
    let pool = setup_test_database().await;
    let service = create_clinical_service(&pool);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let data = NewConsultationData {
        patient_id,
        practitioner_id,
        appointment_id: None,
        reason: Some("Chest pain and shortness of breath".to_string()),
        clinical_notes: None,
    };
    let consultation = service
        .create_consultation(data, practitioner_id)
        .await
        .expect("Failed to create consultation");

    let consultation = service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Annual check-up".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create consultation");

    assert!(!consultation.is_signed);

    let updated = service
        .update_clinical_notes(
            consultation.id,
            None,
            Some("Patient reports fatigue for 2 weeks\nBP 120/80, HR 72, afebrile\nFatigue, likely iron deficiency anaemia\nFBC, iron studies. Review in 1 week.".to_string()),
            practitioner_id,
        )
        .await
        .expect("Should be able to update clinical notes on unsigned consultation");

    assert_eq!(
        updated.clinical_notes,
        Some("Patient reports fatigue for 2 weeks\nBP 120/80, HR 72, afebrile\nFatigue, likely iron deficiency anaemia\nFBC, iron studies. Review in 1 week.".to_string())
    );
    assert_eq!(updated.updated_by, Some(practitioner_id));
}

#[tokio::test]
async fn test_update_soap_notes_fails_on_signed_consultation() {
    let pool = setup_test_database().await;
    let service = create_clinical_service(&pool);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let consultation = service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Follow-up".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create consultation");

    service
        .sign_consultation(consultation.id, practitioner_id)
        .await
        .expect("Failed to sign consultation");

    let result = service
        .update_clinical_notes(
            consultation.id,
            None,
            Some("Attempting to edit signed note".to_string()),
            practitioner_id,
        )
        .await;

    assert!(
        result.is_err(),
        "Updating clinical notes on a signed consultation should return an error"
    );

    let err = result.unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("sign"),
        "Error should indicate the consultation is already signed, got: {}",
        err
    );
}

#[tokio::test]
async fn test_sign_consultation() {
    let pool = setup_test_database().await;
    let service = create_clinical_service(&pool);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let consultation = service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Hypertension review".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create consultation");

    assert!(!consultation.is_signed);

    service
        .update_clinical_notes(
            consultation.id,
            None,
            Some("BP well controlled on current medication\nBP 128/82, HR 68\nHypertension — well controlled\nContinue current medication. Review in 3 months.".to_string()),
            practitioner_id,
        )
        .await
        .expect("Failed to update clinical notes");

    service
        .sign_consultation(consultation.id, practitioner_id)
        .await
        .expect("Failed to sign consultation");

    let signed = service
        .find_consultation(consultation.id)
        .await
        .expect("Failed to find consultation")
        .expect("Consultation not found");

    assert!(signed.is_signed);
    assert!(signed.signed_at.is_some());
    assert_eq!(signed.signed_by, Some(practitioner_id));
}

#[tokio::test]
async fn test_sign_consultation_twice_fails() {
    let pool = setup_test_database().await;
    let service = create_clinical_service(&pool);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let consultation = service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: None,
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create consultation");

    service
        .sign_consultation(consultation.id, practitioner_id)
        .await
        .expect("First sign should succeed");

    let result = service
        .sign_consultation(consultation.id, practitioner_id)
        .await;

    assert!(
        result.is_err(),
        "Signing an already-signed consultation should fail"
    );
}

#[tokio::test]
async fn test_find_consultations_by_date_range_for_patient() {
    let pool = setup_test_database().await;
    let service = create_clinical_service(&pool);

    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let consultation_repo: Arc<dyn ConsultationRepository> =
        Arc::new(SqlxClinicalRepository::new(pool.clone(), crypto));

    let patient_id = create_test_patient(&pool).await;
    let other_patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Recent visit".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create first consultation");

    service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Second visit".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create second consultation");

    service
        .create_consultation(
            NewConsultationData {
                patient_id: other_patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Other patient visit".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create other patient consultation");

    let start = Utc::now() - Duration::days(7);
    let end = Utc::now() + Duration::hours(1);

    let results = consultation_repo
        .find_by_date_range(patient_id, start, end)
        .await
        .expect("Failed to query consultations by date range");

    assert!(
        !results.is_empty(),
        "Should find consultations within date range"
    );

    for consultation in &results {
        assert_eq!(
            consultation.patient_id, patient_id,
            "find_by_date_range should only return consultations for the specified patient"
        );
    }

    let other_patient_count = results
        .iter()
        .filter(|c| c.patient_id == other_patient_id)
        .count();
    assert_eq!(
        other_patient_count, 0,
        "Results should not include consultations from other patients"
    );
}

#[tokio::test]
async fn test_find_consultations_by_date_range_future_returns_empty() {
    let pool = setup_test_database().await;
    let service = create_clinical_service(&pool);

    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let consultation_repo: Arc<dyn ConsultationRepository> =
        Arc::new(SqlxClinicalRepository::new(pool.clone(), crypto));

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    service
        .create_consultation(
            NewConsultationData {
                patient_id,
                practitioner_id,
                appointment_id: None,
                reason: Some("Current visit".to_string()),
                clinical_notes: None,
            },
            practitioner_id,
        )
        .await
        .expect("Failed to create consultation");

    let start = Utc::now() + Duration::days(30);
    let end = Utc::now() + Duration::days(60);

    let results = consultation_repo
        .find_by_date_range(patient_id, start, end)
        .await
        .expect("Failed to query consultations by date range");

    assert!(
        results.is_empty(),
        "Future date range should return no consultations"
    );
}
