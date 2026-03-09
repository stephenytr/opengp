use chrono::{Duration, NaiveDate, Utc};
use opengp_domain::domain::appointment::{
    Appointment, AppointmentCalendarQuery, AppointmentRepository, AppointmentService,
    AppointmentStatus, AppointmentType,
};
use opengp_domain::domain::audit::{AuditEmitter, AuditRepository, AuditService};
use opengp_domain::domain::patient::{Address, Gender, NewPatientData, PatientRepository, PatientService};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::{
    SqlxAppointmentRepository, SqlxAuditRepository, SqlxPatientRepository,
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
        title: Some("Mr".to_string()),
        first_name: "Test".to_string(),
        middle_name: None,
        last_name: "Patient".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
        gender: Gender::Male,
        address: Address::default(),
        phone_home: None,
        phone_mobile: Some("0412345678".to_string()),
        email: Some("test@example.com".to_string()),
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
        .expect("Failed to create patient")
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
    .expect("Failed to create practitioner");

    id
}

fn create_test_appointment(
    patient_id: Uuid,
    practitioner_id: Uuid,
    start_time: chrono::DateTime<Utc>,
    status: AppointmentStatus,
) -> Appointment {
    Appointment {
        id: Uuid::new_v4(),
        patient_id,
        practitioner_id,
        start_time,
        end_time: start_time + Duration::minutes(15),
        appointment_type: AppointmentType::Standard,
        status,
        reason: Some("Test appointment".to_string()),
        notes: None,
        is_urgent: false,
        confirmed: false,
        reminder_sent: false,
        cancellation_reason: None,
        created_at: Utc::now(),
        created_by: Some(practitioner_id),
        updated_at: Utc::now(),
        updated_by: None,
    }
}

#[tokio::test]
async fn test_mark_arrived_updates_status() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now() + Duration::hours(1);

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::Confirmed,
    );
    let appointment_id = appointment.id;

    let created = repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");
    assert_eq!(created.status, AppointmentStatus::Confirmed);

    let updated = service
        .mark_arrived(appointment_id, user_id)
        .await
        .expect("Failed to mark as arrived");

    assert_eq!(updated.status, AppointmentStatus::Arrived);
    assert_eq!(updated.updated_by, Some(user_id));
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_mark_completed_updates_status() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now() + Duration::hours(1);

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::InProgress,
    );
    let appointment_id = appointment.id;

    let created = repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");
    assert_eq!(created.status, AppointmentStatus::InProgress);

    let updated = service
        .mark_completed(appointment_id, user_id)
        .await
        .expect("Failed to mark as completed");

    assert_eq!(updated.status, AppointmentStatus::Completed);
    assert_eq!(updated.updated_by, Some(user_id));
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_mark_no_show_updates_status() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now() - Duration::minutes(30);

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::Arrived,
    );
    let appointment_id = appointment.id;

    let created = repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");
    assert_eq!(created.status, AppointmentStatus::Arrived);

    let updated = service
        .mark_no_show(appointment_id, user_id)
        .await
        .expect("Failed to mark as no show");

    assert_eq!(updated.status, AppointmentStatus::NoShow);
    assert_eq!(updated.updated_by, Some(user_id));
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_mark_arrived_not_found_returns_error() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository, audit_service, calendar_query);

    let invalid_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = service.mark_arrived(invalid_id, user_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_mark_completed_persists_to_database() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now() + Duration::hours(1);

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::InProgress,
    );
    let appointment_id = appointment.id;

    repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");

    service
        .mark_completed(appointment_id, user_id)
        .await
        .expect("Failed to mark as completed");

    let found = repository
        .find_by_id(appointment_id)
        .await
        .expect("Failed to query database")
        .expect("Appointment not found");

    assert_eq!(found.status, AppointmentStatus::Completed);
    assert_eq!(found.updated_by, Some(user_id));
}

#[tokio::test]
async fn test_status_update_audit_trail() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_a = create_test_practitioner(&pool).await;
    let user_b = create_test_practitioner(&pool).await;
    let start_time = Utc::now() + Duration::hours(1);

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::Confirmed,
    );
    let appointment_id = appointment.id;

    repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");

    let arrived = service
        .mark_arrived(appointment_id, user_a)
        .await
        .expect("Failed to mark as arrived");
    assert_eq!(arrived.updated_by, Some(user_a));
    let arrived_time = arrived.updated_at;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut in_progress_appt = repository
        .find_by_id(appointment_id)
        .await
        .unwrap()
        .unwrap();
    in_progress_appt.status = AppointmentStatus::InProgress;
    in_progress_appt.updated_by = Some(user_a);
    in_progress_appt.updated_at = Utc::now();
    repository
        .update(in_progress_appt)
        .await
        .expect("Failed to update to InProgress");

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let completed = service
        .mark_completed(appointment_id, user_b)
        .await
        .expect("Failed to mark as completed");
    assert_eq!(completed.updated_by, Some(user_b));
    assert!(completed.updated_at > arrived_time);
}

#[tokio::test]
async fn test_concurrent_status_updates() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = Arc::new(AppointmentService::new(
        repository.clone(),
        audit_service,
        calendar_query,
    ));

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now() + Duration::hours(1);

    let mut appointment_ids = Vec::new();
    let statuses = vec![
        AppointmentStatus::Confirmed,
        AppointmentStatus::InProgress,
        AppointmentStatus::Arrived,
    ];

    for status in statuses {
        let appointment = create_test_appointment(patient_id, practitioner_id, start_time, status);
        let id = appointment.id;
        repository
            .create(appointment)
            .await
            .expect("Failed to create appointment");
        appointment_ids.push(id);
    }

    let mut handles = Vec::new();
    for (i, &appt_id) in appointment_ids.iter().enumerate() {
        let service_clone = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            match i {
                0 => service_clone.mark_arrived(appt_id, user_id).await,
                1 => service_clone.mark_completed(appt_id, user_id).await,
                2 => service_clone.mark_no_show(appt_id, user_id).await,
                _ => unreachable!(),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
        let service_result = result.unwrap();
        assert!(service_result.is_ok());
    }

    let appt1 = repository
        .find_by_id(appointment_ids[0])
        .await
        .expect("Failed to query")
        .expect("Not found");
    assert_eq!(appt1.status, AppointmentStatus::Arrived);

    let appt2 = repository
        .find_by_id(appointment_ids[1])
        .await
        .expect("Failed to query")
        .expect("Not found");
    assert_eq!(appt2.status, AppointmentStatus::Completed);

    let appt3 = repository
        .find_by_id(appointment_ids[2])
        .await
        .expect("Failed to query")
        .expect("Not found");
    assert_eq!(appt3.status, AppointmentStatus::NoShow);
}

#[tokio::test]
async fn test_status_update_from_scheduled_to_arrived() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now() + Duration::minutes(30);

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::Confirmed,
    );
    let appointment_id = appointment.id;

    repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");

    let updated = service
        .mark_arrived(appointment_id, user_id)
        .await
        .expect("Failed to mark as arrived");

    assert_eq!(updated.status, AppointmentStatus::Arrived);
    assert_eq!(updated.patient_id, patient_id);
    assert_eq!(updated.practitioner_id, practitioner_id);
}

#[tokio::test]
async fn test_status_update_from_arrived_to_completed() {
    let pool = setup_test_database().await;
    let repo = Arc::new(SqlxAppointmentRepository::new(pool.clone()));
    let repository: Arc<dyn AppointmentRepository> = repo.clone();
    let calendar_query: Arc<dyn AppointmentCalendarQuery> = repo.clone();
    let audit_service = create_mock_audit_service(&pool);
    let service = AppointmentService::new(repository.clone(), audit_service, calendar_query);

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let user_id = practitioner_id;
    let start_time = Utc::now();

    let appointment = create_test_appointment(
        patient_id,
        practitioner_id,
        start_time,
        AppointmentStatus::InProgress,
    );
    let appointment_id = appointment.id;

    repository
        .create(appointment)
        .await
        .expect("Failed to create appointment");

    let updated = service
        .mark_completed(appointment_id, user_id)
        .await
        .expect("Failed to mark as completed");

    assert_eq!(updated.status, AppointmentStatus::Completed);
    assert_eq!(updated.patient_id, patient_id);
    assert_eq!(updated.practitioner_id, practitioner_id);
}
