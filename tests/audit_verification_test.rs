use chrono::{Duration, NaiveDate, Utc};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentRepository, AppointmentService, NewAppointmentData,
};
use opengp_domain::domain::audit::{
    AuditAction, AuditEmitter, AuditEntry, AuditRepository, AuditService,
};
use opengp_domain::domain::clinical::{
    ClinicalRepositories, ClinicalService, Consultation, ConsultationRepository,
};
use opengp_domain::domain::patient::{
    Address, Gender, NewPatientData, PatientRepository, PatientService,
};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::{
    SqlxAllergyRepository, SqlxAppointmentRepository, SqlxAuditRepository, SqlxClinicalRepository,
    SqlxFamilyHistoryRepository, SqlxMedicalHistoryRepository, SqlxPatientRepository,
    SqlxSocialHistoryRepository, SqlxVitalSignsRepository,
};
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

async fn create_test_practitioner(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let username = format!("dr_{}", id);
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
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
    .expect("Failed to create practitioner");

    id
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance with specific migration state"]
async fn audit_comprehensive() {
    let pool = setup_test_database().await;

    let practitioner_a = create_test_practitioner(&pool).await;
    let practitioner_b = create_test_practitioner(&pool).await;

    let audit_repository: Arc<dyn AuditRepository> =
        Arc::new(SqlxAuditRepository::new(pool.clone()));
    let audit_emitter: Arc<dyn AuditEmitter> =
        Arc::new(AuditService::new(audit_repository.clone()));

    let patient_crypto =
        Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let patient_repository: Arc<dyn PatientRepository> =
        Arc::new(SqlxPatientRepository::new(pool.clone(), patient_crypto));
    let patient_service = Arc::new(PatientService::new(patient_repository));

    let patient = patient_service
        .register_patient(NewPatientData {
            ihi: None,
            medicare_number: Some(format!("{:010}", Uuid::new_v4().as_u128() % 10000000000).into()),
            medicare_irn: Some(1),
            medicare_expiry: None,
            title: Some("Mr".to_string()),
            first_name: "Audit".to_string(),
            middle_name: None,
            last_name: "Patient".to_string(),
            preferred_name: None,
            date_of_birth: NaiveDate::from_ymd_opt(1991, 2, 3).unwrap(),
            gender: Gender::Male,
            address: Address::default(),
            phone_home: None,
            phone_mobile: Some("0412345678".to_string().into()),
            email: Some("audit.patient@example.com".to_string()),
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: None,
            interpreter_required: None,
            aboriginal_torres_strait_islander: None,
            occupation: None,
            employment_status: None,
            health_fund: None,
            dva_card_type: None,
        })
        .await
        .expect("Failed to create patient");

    audit_emitter
        .emit(AuditEntry::new_created(
            "patient",
            patient.id,
            serde_json::to_string(&patient).expect("patient should serialize"),
            practitioner_a,
        ))
        .await
        .expect("Failed to emit patient create audit");

    let appointment_repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let appointment_repository: Arc<dyn AppointmentRepository> = appointment_repo.clone();
    let appointment_calendar_query: Arc<dyn AppointmentCalendarQuery> = appointment_repo;
    let appointment_service = AppointmentService::new(
        appointment_repository,
        audit_emitter.clone(),
        appointment_calendar_query,
    );

    let appointment = appointment_service
        .create_appointment(
            NewAppointmentData {
                patient_id: patient.id,
                practitioner_id: practitioner_b,
                start_time: Utc::now() + Duration::hours(2),
                duration: Duration::minutes(20),
                appointment_type: opengp_domain::domain::appointment::AppointmentType::Standard,
                reason: Some("Audit flow appointment".to_string()),
                is_urgent: false,
            },
            practitioner_b,
        )
        .await
        .expect("Failed to create appointment");

    audit_emitter
        .emit(AuditEntry::new_created(
            "appointment",
            appointment.id,
            serde_json::to_string(&appointment).expect("appointment should serialize"),
            practitioner_b,
        ))
        .await
        .expect("Failed to emit appointment create audit");

    let clinical_crypto =
        Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let consultation_repository: Arc<dyn ConsultationRepository> = Arc::new(
        SqlxClinicalRepository::new(pool.clone(), clinical_crypto.clone()),
    );

    let consultation = consultation_repository
        .create(Consultation::new(
            patient.id,
            practitioner_a,
            Some(appointment.id),
            practitioner_a,
        ))
        .await
        .expect("Failed to seed consultation");

    let clinical_repos = ClinicalRepositories {
        consultation: consultation_repository,
        allergy: Arc::new(SqlxAllergyRepository::new(
            pool.clone(),
            clinical_crypto.clone(),
        )),
        medical_history: Arc::new(SqlxMedicalHistoryRepository::new(
            pool.clone(),
            clinical_crypto.clone(),
        )),
        vital_signs: Arc::new(SqlxVitalSignsRepository::new(
            pool.clone(),
            clinical_crypto.clone(),
        )),
        social_history: Arc::new(SqlxSocialHistoryRepository::new(
            pool.clone(),
            clinical_crypto.clone(),
        )),
        family_history: Arc::new(SqlxFamilyHistoryRepository::new(
            pool.clone(),
            clinical_crypto,
        )),
    };
    let clinical_service = ClinicalService::new(clinical_repos, patient_service, audit_emitter);

    let _updated_consultation = clinical_service
        .update_clinical_notes(
            consultation.id,
            Some("Review results".to_string()),
            Some("Consultation notes updated by practitioner A".to_string()),
            consultation.version,
            practitioner_a,
        )
        .await
        .expect("Failed to update consultation");

    let patient_audit = audit_repository
        .find_by_entity("patient", patient.id)
        .await
        .expect("Failed to query patient audit");
    assert_eq!(patient_audit.len(), 1, "Expected one patient audit entry");
    assert_eq!(patient_audit[0].changed_by, practitioner_a);
    assert_eq!(patient_audit[0].action, AuditAction::Created);

    let appointment_audit = audit_repository
        .find_by_entity("appointment", appointment.id)
        .await
        .expect("Failed to query appointment audit");
    assert_eq!(
        appointment_audit.len(),
        1,
        "Expected one appointment audit entry"
    );
    assert_eq!(appointment_audit[0].changed_by, practitioner_b);
    assert_eq!(appointment_audit[0].action, AuditAction::Created);

    let consultation_audit = audit_repository
        .find_by_entity("consultation", consultation.id)
        .await
        .expect("Failed to query consultation audit");
    assert_eq!(
        consultation_audit.len(),
        1,
        "Expected one consultation audit entry"
    );
    assert_eq!(consultation_audit[0].changed_by, practitioner_a);
    assert_eq!(consultation_audit[0].action, AuditAction::Updated);

    for entry in patient_audit
        .iter()
        .chain(appointment_audit.iter())
        .chain(consultation_audit.iter())
    {
        assert_ne!(
            entry.changed_by,
            Uuid::nil(),
            "Audit entry used system_user_id instead of authenticated user"
        );
    }
}
