#![cfg(feature = "billing")]

use chrono::{Datelike, NaiveDate, Utc};
use opengp_domain::domain::billing::{
    BillingError, BillingRepository, BillingService, BillingType, Invoice, InvoiceItem,
    InvoiceStatus, ServiceError as BillingServiceError, ValidationError as BillingValidationError,
};
use opengp_domain::domain::clinical::{Consultation, ConsultationRepository};
use opengp_domain::domain::patient::{
    Address, Gender, NewPatientData, PatientRepository, PatientService,
};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::repositories::{
    SqlxBillingRepository, SqlxClinicalRepository, SqlxPatientRepository,
};
use serde_json::Value;
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

fn build_billing_service(pool: &PgPool) -> BillingService {
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let billing_repo: Arc<dyn BillingRepository> =
        Arc::new(SqlxBillingRepository::new(pool.clone()));
    let clinical_repo: Arc<dyn ConsultationRepository> =
        Arc::new(SqlxClinicalRepository::new(pool.clone(), crypto));

    BillingService::new(billing_repo, clinical_repo)
}

fn build_billing_repository(pool: &PgPool) -> Arc<dyn BillingRepository> {
    Arc::new(SqlxBillingRepository::new(pool.clone()))
}

async fn create_test_patient(pool: &PgPool) -> Uuid {
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let repository: Arc<dyn PatientRepository> =
        Arc::new(SqlxPatientRepository::new(pool.clone(), crypto));
    let service = PatientService::new(repository);

    let medicare_number = format!("{:010}", Uuid::new_v4().as_u128() % 10_000_000_000);

    let data = NewPatientData {
        ihi: None,
        medicare_number: Some(medicare_number.into()),
        medicare_irn: Some(1),
        medicare_expiry: None,
        title: Some("Ms".to_string()),
        first_name: "Billing".to_string(),
        middle_name: None,
        last_name: "TestPatient".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(1987, 2, 14).expect("valid date"),
        gender: Gender::Female,
        address: Address::default(),
        phone_home: None,
        phone_mobile: Some("0412345678".to_string().into()),
        email: Some("billing.test.patient@example.com".to_string()),
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
    };

    service
        .register_patient(data)
        .await
        .expect("Failed to create test patient")
        .id
}

async fn create_test_practitioner(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let username = format!("billing_dr_{id}");
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
    .expect("Failed to create test practitioner");

    id
}

async fn create_consultation(
    pool: &PgPool,
    patient_id: Uuid,
    practitioner_id: Uuid,
    signed: bool,
) -> Uuid {
    let crypto = Arc::new(EncryptionService::new().expect("Failed to initialize encryption"));
    let consultation_repo: Arc<dyn ConsultationRepository> =
        Arc::new(SqlxClinicalRepository::new(pool.clone(), crypto));

    let mut consultation = Consultation::new(patient_id, practitioner_id, None, practitioner_id);
    consultation.reason = Some("Billing test consultation".to_string());
    let consultation = consultation_repo
        .create(consultation)
        .await
        .expect("Failed to create consultation");

    if signed {
        consultation_repo
            .sign(consultation.id, practitioner_id)
            .await
            .expect("Failed to sign consultation");
    }

    consultation.id
}

fn build_draft_invoice(
    patient_id: Uuid,
    practitioner_id: Uuid,
    created_by: Uuid,
    invoice_number: String,
) -> Invoice {
    let now = Utc::now();
    Invoice {
        id: Uuid::new_v4(),
        patient_id,
        practitioner_id,
        consultation_id: None,
        invoice_number,
        invoice_date: now.date_naive(),
        due_date: Some(now.date_naive()),
        items: vec![InvoiceItem {
            id: Uuid::new_v4(),
            description: "Consultation fee".to_string(),
            item_code: Some("23".to_string()),
            quantity: 1,
            unit_price: 100.0,
            amount: 100.0,
            is_gst_free: true,
        }],
        subtotal: 0.0,
        gst_amount: 0.0,
        total_amount: 0.0,
        amount_paid: 0.0,
        amount_outstanding: 0.0,
        status: InvoiceStatus::Draft,
        billing_type: BillingType::PrivateBilling,
        notes: None,
        created_at: now,
        updated_at: now,
        created_by,
        updated_by: None,
    }
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_create_invoice_from_consultation() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let consultation_id = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice = billing_service
        .create_invoice_from_consultation(
            consultation_id,
            vec![
                ("23".to_string(), 89.0, true),
                ("10990".to_string(), 25.0, false),
            ],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create invoice from signed consultation");

    assert_eq!(invoice.consultation_id, Some(consultation_id));
    assert_eq!(invoice.items.len(), 2);
    assert_eq!(invoice.items[0].item_code.as_deref(), Some("23"));
    assert_eq!(invoice.items[1].item_code.as_deref(), Some("10990"));
    assert_eq!(invoice.subtotal, 114.0);
    assert_eq!(invoice.gst_amount, 2.5);
    assert_eq!(invoice.total_amount, 116.5);
    assert_eq!(invoice.status, InvoiceStatus::Issued);
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_record_cash_payment_full() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let consultation_id = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice = billing_service
        .create_invoice_from_consultation(
            consultation_id,
            vec![("23".to_string(), 89.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create invoice");

    let (_, updated_invoice) = billing_service
        .record_cash_payment(invoice.id, invoice.total_amount, practitioner_id)
        .await
        .expect("Failed to record full cash payment");

    assert_eq!(updated_invoice.status, InvoiceStatus::Paid);
    assert_eq!(updated_invoice.amount_outstanding, 0.0);
    assert_eq!(updated_invoice.amount_paid, invoice.total_amount);
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_record_cash_payment_partial() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let consultation_id = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice = billing_service
        .create_invoice_from_consultation(
            consultation_id,
            vec![("23".to_string(), 120.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create invoice");

    let partial_amount = 40.0;
    let (_, updated_invoice) = billing_service
        .record_cash_payment(invoice.id, partial_amount, practitioner_id)
        .await
        .expect("Failed to record partial cash payment");

    assert_eq!(updated_invoice.status, InvoiceStatus::PartiallyPaid);
    assert_eq!(updated_invoice.amount_paid, partial_amount);
    assert_eq!(
        updated_invoice.amount_outstanding,
        invoice.total_amount - partial_amount
    );
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_prepare_claim_json() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let consultation_id = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice = billing_service
        .create_invoice_from_consultation(
            consultation_id,
            vec![("23".to_string(), 89.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create invoice");

    let claim_json = billing_service
        .prepare_claim_json(invoice.id)
        .await
        .expect("Failed to prepare claim JSON");

    let parsed: Value = serde_json::from_str(&claim_json).expect("Claim JSON should be valid");
    assert_eq!(parsed["invoice_id"], invoice.id.to_string());
    assert_eq!(parsed["patient_id"], invoice.patient_id.to_string());
    assert_eq!(
        parsed["practitioner_id"],
        invoice.practitioner_id.to_string()
    );
    assert!(parsed["items"].is_array(), "items should be an array");
    assert!(
        parsed["total_claimed"].is_number(),
        "total_claimed should be numeric"
    );
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_invoice_numbering_sequential() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let year = Utc::now().year();
    let prefix = format!("INV-{year}-%");

    sqlx::query("DELETE FROM payments")
        .execute(&pool)
        .await
        .expect("Failed to clear payments");
    sqlx::query("DELETE FROM invoice_items")
        .execute(&pool)
        .await
        .expect("Failed to clear invoice items");
    sqlx::query("DELETE FROM invoices WHERE invoice_number LIKE $1")
        .bind(prefix)
        .execute(&pool)
        .await
        .expect("Failed to clear invoice rows for current year");

    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let consultation_1 = create_consultation(&pool, patient_id, practitioner_id, true).await;
    let consultation_2 = create_consultation(&pool, patient_id, practitioner_id, true).await;
    let consultation_3 = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice_1 = billing_service
        .create_invoice_from_consultation(
            consultation_1,
            vec![("23".to_string(), 89.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create first invoice");
    let invoice_2 = billing_service
        .create_invoice_from_consultation(
            consultation_2,
            vec![("23".to_string(), 89.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create second invoice");
    let invoice_3 = billing_service
        .create_invoice_from_consultation(
            consultation_3,
            vec![("23".to_string(), 89.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create third invoice");

    assert_eq!(invoice_1.invoice_number, format!("INV-{year}-00001"));
    assert_eq!(invoice_2.invoice_number, format!("INV-{year}-00002"));
    assert_eq!(invoice_3.invoice_number, format!("INV-{year}-00003"));
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_patient_balance_calculation() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let consultation_1 = create_consultation(&pool, patient_id, practitioner_id, true).await;
    let consultation_2 = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice_1 = billing_service
        .create_invoice_from_consultation(
            consultation_1,
            vec![("23".to_string(), 100.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create first invoice");

    let invoice_2 = billing_service
        .create_invoice_from_consultation(
            consultation_2,
            vec![("10990".to_string(), 50.0, false)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create second invoice");

    billing_service
        .record_cash_payment(invoice_1.id, 30.0, practitioner_id)
        .await
        .expect("Failed to apply payment to first invoice");
    billing_service
        .record_cash_payment(invoice_2.id, 20.0, practitioner_id)
        .await
        .expect("Failed to apply payment to second invoice");

    let balance = billing_service
        .find_patient_balance(patient_id)
        .await
        .expect("Failed to calculate patient balance");

    let expected = (invoice_1.total_amount - 30.0) + (invoice_2.total_amount - 20.0);
    assert_eq!(balance, expected);
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_gst_calculation() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let consultation_id = create_consultation(&pool, patient_id, practitioner_id, true).await;

    let invoice = billing_service
        .create_invoice_from_consultation(
            consultation_id,
            vec![
                ("23".to_string(), 80.0, true),
                ("SUPPLY-1".to_string(), 50.0, false),
            ],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await
        .expect("Failed to create mixed GST invoice");

    assert_eq!(invoice.subtotal, 130.0);
    assert_eq!(invoice.gst_amount, 5.0);
    assert_eq!(invoice.total_amount, 135.0);
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_invoice_status_transitions() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let billing_repo = build_billing_repository(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;

    let year = Utc::now().year();
    let invoice_number = billing_repo
        .next_invoice_number(year)
        .await
        .expect("Failed to generate invoice number");

    let draft_invoice =
        build_draft_invoice(patient_id, practitioner_id, practitioner_id, invoice_number);
    let created = billing_service
        .create_invoice(draft_invoice)
        .await
        .expect("Failed to create draft invoice");
    assert_eq!(created.status, InvoiceStatus::Draft);

    billing_repo
        .update_invoice_status(created.id, InvoiceStatus::Issued)
        .await
        .expect("Failed to mark draft as issued");

    let issued = billing_service
        .find_invoice_by_id(created.id)
        .await
        .expect("Failed to load issued invoice")
        .expect("Issued invoice should exist");
    assert_eq!(issued.status, InvoiceStatus::Issued);

    let (_, paid_invoice) = billing_service
        .record_cash_payment(created.id, issued.total_amount, practitioner_id)
        .await
        .expect("Failed to record payment for issued invoice");
    assert_eq!(paid_invoice.status, InvoiceStatus::Paid);
    assert_eq!(paid_invoice.amount_outstanding, 0.0);
}

#[tokio::test]
#[ignore = "requires running PostgreSQL instance"]
async fn test_cannot_create_invoice_from_unsigned_consultation() {
    let pool = setup_test_database().await;
    let billing_service = build_billing_service(&pool);
    let patient_id = create_test_patient(&pool).await;
    let practitioner_id = create_test_practitioner(&pool).await;
    let consultation_id = create_consultation(&pool, patient_id, practitioner_id, false).await;

    let result = billing_service
        .create_invoice_from_consultation(
            consultation_id,
            vec![("23".to_string(), 89.0, true)],
            BillingType::PrivateBilling,
            practitioner_id,
        )
        .await;

    assert!(matches!(
        result,
        Err(BillingError::Validation(
            BillingValidationError::ConsultationNotSigned
        ))
    ));
    assert!(!matches!(result, Err(BillingServiceError::Repository(_))));
}
