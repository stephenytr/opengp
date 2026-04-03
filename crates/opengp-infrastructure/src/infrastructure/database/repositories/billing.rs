use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::infrastructure::database::sqlx_to_repository_error;
use opengp_domain::domain::billing::{
    BillingRepository, ClaimStatus, Invoice, InvoiceItem, InvoiceStatus, MBSItem, MedicareClaim,
    Payment, RepositoryError,
};

#[derive(Debug, FromRow)]
struct InvoiceRow {
    id: String,
    invoice_number: String,
    patient_id: String,
    practitioner_id: String,
    consultation_id: Option<String>,
    billing_type: String,
    status: String,
    issue_date: String,
    due_date: Option<String>,
    subtotal: f64,
    gst_amount: f64,
    total_amount: f64,
    amount_paid: f64,
    amount_outstanding: f64,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct InvoiceItemRow {
    id: String,
    description: String,
    item_code: Option<String>,
    quantity: i64,
    unit_price: f64,
    amount: f64,
    is_gst_free: i64,
}

#[derive(Debug, FromRow)]
struct PaymentRow {
    id: String,
    invoice_id: String,
    patient_id: String,
    payment_date: String,
    amount: f64,
    payment_method: String,
    reference: Option<String>,
    notes: Option<String>,
    created_at: String,
    created_by: String,
}

#[derive(Debug, FromRow)]
struct ClaimRow {
    id: String,
    invoice_id: Option<String>,
    patient_id: String,
    practitioner_id: String,
    claim_type: String,
    status: String,
    service_date: String,
    total_claimed: f64,
    total_benefit: f64,
    reference_number: Option<String>,
    submitted_at: Option<String>,
    processed_at: Option<String>,
    created_at: String,
}

pub struct SqlxBillingRepository {
    pool: PgPool,
}

impl SqlxBillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn map_invoice_row(&self, row: InvoiceRow) -> Result<Invoice, RepositoryError> {
        let invoice_id = parse_uuid(&row.id, "invoices.id")?;
        let items = self.find_invoice_items(invoice_id).await?;

        Ok(Invoice {
            id: invoice_id,
            patient_id: parse_uuid(&row.patient_id, "invoices.patient_id")?,
            practitioner_id: parse_uuid(&row.practitioner_id, "invoices.practitioner_id")?,
            consultation_id: row
                .consultation_id
                .as_deref()
                .map(|value| parse_uuid(value, "invoices.consultation_id"))
                .transpose()?,
            invoice_number: row.invoice_number,
            invoice_date: parse_naive_date(&row.issue_date, "invoices.issue_date")?,
            due_date: row
                .due_date
                .as_deref()
                .map(|value| parse_naive_date(value, "invoices.due_date"))
                .transpose()?,
            items,
            subtotal: row.subtotal,
            gst_amount: row.gst_amount,
            total_amount: row.total_amount,
            amount_paid: row.amount_paid,
            amount_outstanding: row.amount_outstanding,
            status: parse_enum(&row.status, "invoices.status")?,
            billing_type: parse_enum(&row.billing_type, "invoices.billing_type")?,
            notes: row.notes,
            created_at: parse_datetime(&row.created_at, "invoices.created_at")?,
            updated_at: parse_datetime(&row.updated_at, "invoices.updated_at")?,
            created_by: parse_uuid(&row.practitioner_id, "invoices.practitioner_id")?,
            updated_by: None,
        })
    }

    fn map_invoice_item_row(row: InvoiceItemRow) -> Result<InvoiceItem, RepositoryError> {
        let quantity =
            u32::try_from(row.quantity).map_err(|_| invalid_data("invoice_items.quantity", row.quantity))?;

        Ok(InvoiceItem {
            id: parse_uuid(&row.id, "invoice_items.id")?,
            description: row.description,
            item_code: row.item_code,
            quantity,
            unit_price: row.unit_price,
            amount: row.amount,
            is_gst_free: row.is_gst_free != 0,
        })
    }

    fn map_payment_row(row: PaymentRow) -> Result<Payment, RepositoryError> {
        Ok(Payment {
            id: parse_uuid(&row.id, "payments.id")?,
            invoice_id: parse_uuid(&row.invoice_id, "payments.invoice_id")?,
            patient_id: parse_uuid(&row.patient_id, "payments.patient_id")?,
            payment_date: parse_datetime(&row.payment_date, "payments.payment_date")?,
            amount: row.amount,
            payment_method: parse_enum(&row.payment_method, "payments.payment_method")?,
            reference: row.reference,
            notes: row.notes,
            created_at: parse_datetime(&row.created_at, "payments.created_at")?,
            created_by: parse_uuid(&row.created_by, "payments.created_by")?,
        })
    }

    fn map_claim_row(row: ClaimRow) -> Result<MedicareClaim, RepositoryError> {
        let total_claimed = row.total_claimed;
        let total_benefit = row.total_benefit;

        Ok(MedicareClaim {
            id: parse_uuid(&row.id, "medicare_claims.id")?,
            patient_id: parse_uuid(&row.patient_id, "medicare_claims.patient_id")?,
            practitioner_id: parse_uuid(&row.practitioner_id, "medicare_claims.practitioner_id")?,
            consultation_id: None,
            invoice_id: row
                .invoice_id
                .as_deref()
                .map(|value| parse_uuid(value, "medicare_claims.invoice_id"))
                .transpose()?,
            claim_reference: row.reference_number,
            service_date: parse_naive_date(&row.service_date, "medicare_claims.service_date")?,
            items: Vec::<MBSItem>::new(),
            total_claimed,
            total_benefit,
            patient_contribution: total_claimed - total_benefit,
            claim_type: parse_enum(&row.claim_type, "medicare_claims.claim_type")?,
            status: parse_enum(&row.status, "medicare_claims.status")?,
            submitted_at: row
                .submitted_at
                .as_deref()
                .map(|value| parse_datetime(value, "medicare_claims.submitted_at"))
                .transpose()?,
            processed_at: row
                .processed_at
                .as_deref()
                .map(|value| parse_datetime(value, "medicare_claims.processed_at"))
                .transpose()?,
            rejection_reason: None,
            created_at: parse_datetime(&row.created_at, "medicare_claims.created_at")?,
            created_by: parse_uuid(&row.practitioner_id, "medicare_claims.practitioner_id")?,
        })
    }

    async fn insert_invoice_items(
        &self,
        invoice_id: Uuid,
        items: &[InvoiceItem],
        created_at: DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        for item in items {
            let quantity = i64::from(item.quantity);
            sqlx::query(
                r#"
                INSERT INTO invoice_items (
                    id, invoice_id, description, item_code, quantity, unit_price, amount, is_gst_free, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(item.id.to_string())
            .bind(invoice_id.to_string())
            .bind(&item.description)
            .bind(&item.item_code)
            .bind(quantity)
            .bind(item.unit_price)
            .bind(item.amount)
            .bind(if item.is_gst_free { 1_i64 } else { 0_i64 })
            .bind(created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(sqlx_to_repository_error)?;
        }

        Ok(())
    }
}

#[async_trait]
impl BillingRepository for SqlxBillingRepository {
    async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, RepositoryError> {
        let row = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT
                id, invoice_number, patient_id, practitioner_id, consultation_id,
                billing_type, status, issue_date, due_date,
                subtotal, gst_amount, total_amount, amount_paid, amount_outstanding,
                notes, created_at, updated_at
            FROM invoices
            WHERE id = $1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        match row {
            Some(invoice_row) => Ok(Some(self.map_invoice_row(invoice_row).await?)),
            None => Ok(None),
        }
    }

    async fn find_invoices_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Invoice>, RepositoryError> {
        let rows = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT
                id, invoice_number, patient_id, practitioner_id, consultation_id,
                billing_type, status, issue_date, due_date,
                subtotal, gst_amount, total_amount, amount_paid, amount_outstanding,
                notes, created_at, updated_at
            FROM invoices
            WHERE patient_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(patient_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            invoices.push(self.map_invoice_row(row).await?);
        }
        Ok(invoices)
    }

    async fn find_invoices_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Invoice>, RepositoryError> {
        let rows = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT
                id, invoice_number, patient_id, practitioner_id, consultation_id,
                billing_type, status, issue_date, due_date,
                subtotal, gst_amount, total_amount, amount_paid, amount_outstanding,
                notes, created_at, updated_at
            FROM invoices
            WHERE created_at BETWEEN $1 AND $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(start.to_rfc3339())
        .bind(end.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            invoices.push(self.map_invoice_row(row).await?);
        }
        Ok(invoices)
    }

    async fn find_invoices_by_status(
        &self,
        status: InvoiceStatus,
    ) -> Result<Vec<Invoice>, RepositoryError> {
        let rows = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT
                id, invoice_number, patient_id, practitioner_id, consultation_id,
                billing_type, status, issue_date, due_date,
                subtotal, gst_amount, total_amount, amount_paid, amount_outstanding,
                notes, created_at, updated_at
            FROM invoices
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(status.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            invoices.push(self.map_invoice_row(row).await?);
        }
        Ok(invoices)
    }

    async fn create_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO invoices (
                id, invoice_number, patient_id, practitioner_id, consultation_id,
                billing_type, status, issue_date, due_date,
                subtotal, gst_amount, total_amount, amount_paid, amount_outstanding,
                notes, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13, $14,
                $15, $16, $17
            )
            "#,
        )
        .bind(invoice.id.to_string())
        .bind(&invoice.invoice_number)
        .bind(invoice.patient_id.to_string())
        .bind(invoice.practitioner_id.to_string())
        .bind(invoice.consultation_id.map(|id| id.to_string()))
        .bind(invoice.billing_type.to_string())
        .bind(invoice.status.to_string())
        .bind(invoice.invoice_date.format("%Y-%m-%d").to_string())
        .bind(invoice.due_date.map(|date| date.format("%Y-%m-%d").to_string()))
        .bind(invoice.subtotal)
        .bind(invoice.gst_amount)
        .bind(invoice.total_amount)
        .bind(invoice.amount_paid)
        .bind(invoice.amount_outstanding)
        .bind(&invoice.notes)
        .bind(invoice.created_at.to_rfc3339())
        .bind(invoice.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        self.insert_invoice_items(invoice.id, &invoice.items, invoice.created_at)
            .await?;

        Ok(invoice)
    }

    async fn update_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
        sqlx::query(
            r#"
            UPDATE invoices SET
                invoice_number = $1,
                patient_id = $2,
                practitioner_id = $3,
                consultation_id = $4,
                billing_type = $5,
                status = $6,
                issue_date = $7,
                due_date = $8,
                subtotal = $9,
                gst_amount = $10,
                total_amount = $11,
                amount_paid = $12,
                amount_outstanding = $13,
                notes = $14,
                updated_at = $15
            WHERE id = $16
            "#,
        )
        .bind(&invoice.invoice_number)
        .bind(invoice.patient_id.to_string())
        .bind(invoice.practitioner_id.to_string())
        .bind(invoice.consultation_id.map(|id| id.to_string()))
        .bind(invoice.billing_type.to_string())
        .bind(invoice.status.to_string())
        .bind(invoice.invoice_date.format("%Y-%m-%d").to_string())
        .bind(invoice.due_date.map(|date| date.format("%Y-%m-%d").to_string()))
        .bind(invoice.subtotal)
        .bind(invoice.gst_amount)
        .bind(invoice.total_amount)
        .bind(invoice.amount_paid)
        .bind(invoice.amount_outstanding)
        .bind(&invoice.notes)
        .bind(invoice.updated_at.to_rfc3339())
        .bind(invoice.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        sqlx::query("DELETE FROM invoice_items WHERE invoice_id = $1")
            .bind(invoice.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(sqlx_to_repository_error)?;

        self.insert_invoice_items(invoice.id, &invoice.items, invoice.updated_at)
            .await?;

        Ok(invoice)
    }

    async fn update_invoice_status(
        &self,
        id: Uuid,
        status: InvoiceStatus,
    ) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE invoices SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(status.to_string())
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(sqlx_to_repository_error)?;

        Ok(())
    }

    async fn find_claim_by_id(&self, id: Uuid) -> Result<Option<MedicareClaim>, RepositoryError> {
        let row = sqlx::query_as::<_, ClaimRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, practitioner_id,
                claim_type, status, service_date,
                total_claimed, total_benefit,
                reference_number, submitted_at, processed_at, created_at
            FROM medicare_claims
            WHERE id = $1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        row.map(Self::map_claim_row).transpose()
    }

    async fn create_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO medicare_claims (
                id, invoice_id, patient_id, practitioner_id,
                claim_type, status, service_date,
                total_claimed, total_benefit,
                reference_number, submitted_at, processed_at, created_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7,
                $8, $9,
                $10, $11, $12, $13
            )
            "#,
        )
        .bind(claim.id.to_string())
        .bind(claim.invoice_id.map(|id| id.to_string()))
        .bind(claim.patient_id.to_string())
        .bind(claim.practitioner_id.to_string())
        .bind(claim.claim_type.to_string())
        .bind(claim.status.to_string())
        .bind(claim.service_date.format("%Y-%m-%d").to_string())
        .bind(claim.total_claimed)
        .bind(claim.total_benefit)
        .bind(&claim.claim_reference)
        .bind(claim.submitted_at.map(|date| date.to_rfc3339()))
        .bind(claim.processed_at.map(|date| date.to_rfc3339()))
        .bind(claim.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        Ok(claim)
    }

    async fn find_claims_by_status(
        &self,
        status: ClaimStatus,
    ) -> Result<Vec<MedicareClaim>, RepositoryError> {
        let rows = sqlx::query_as::<_, ClaimRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, practitioner_id,
                claim_type, status, service_date,
                total_claimed, total_benefit,
                reference_number, submitted_at, processed_at, created_at
            FROM medicare_claims
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(status.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_claim_row).collect()
    }

    async fn find_claims_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<MedicareClaim>, RepositoryError> {
        let rows = sqlx::query_as::<_, ClaimRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, practitioner_id,
                claim_type, status, service_date,
                total_claimed, total_benefit,
                reference_number, submitted_at, processed_at, created_at
            FROM medicare_claims
            WHERE patient_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(patient_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_claim_row).collect()
    }

    async fn find_claims_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MedicareClaim>, RepositoryError> {
        let start_date = start.date_naive().format("%Y-%m-%d").to_string();
        let end_date = end.date_naive().format("%Y-%m-%d").to_string();

        let rows = sqlx::query_as::<_, ClaimRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, practitioner_id,
                claim_type, status, service_date,
                total_claimed, total_benefit,
                reference_number, submitted_at, processed_at, created_at
            FROM medicare_claims
            WHERE service_date BETWEEN $1 AND $2
            ORDER BY service_date DESC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_claim_row).collect()
    }

    async fn update_claim_status(
        &self,
        id: Uuid,
        status: ClaimStatus,
    ) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE medicare_claims SET status = $1 WHERE id = $2")
            .bind(status.to_string())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(sqlx_to_repository_error)?;

        Ok(())
    }

    async fn record_payment(&self, payment: Payment) -> Result<Payment, RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO payments (
                id, invoice_id, patient_id, amount,
                payment_method, payment_date, reference,
                notes, created_by, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(payment.id.to_string())
        .bind(payment.invoice_id.to_string())
        .bind(payment.patient_id.to_string())
        .bind(payment.amount)
        .bind(payment.payment_method.to_string())
        .bind(payment.payment_date.to_rfc3339())
        .bind(&payment.reference)
        .bind(&payment.notes)
        .bind(payment.created_by.to_string())
        .bind(payment.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        Ok(payment)
    }

    async fn find_payments_by_invoice(&self, invoice_id: Uuid) -> Result<Vec<Payment>, RepositoryError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, payment_date, amount,
                payment_method, reference, notes, created_at, created_by
            FROM payments
            WHERE invoice_id = $1
            ORDER BY payment_date DESC
            "#,
        )
        .bind(invoice_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_payment_row).collect()
    }

    async fn find_payments_by_patient(&self, patient_id: Uuid) -> Result<Vec<Payment>, RepositoryError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, payment_date, amount,
                payment_method, reference, notes, created_at, created_by
            FROM payments
            WHERE patient_id = $1
            ORDER BY payment_date DESC
            "#,
        )
        .bind(patient_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_payment_row).collect()
    }

    async fn find_payments_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Payment>, RepositoryError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT
                id, invoice_id, patient_id, payment_date, amount,
                payment_method, reference, notes, created_at, created_by
            FROM payments
            WHERE payment_date BETWEEN $1 AND $2
            ORDER BY payment_date DESC
            "#,
        )
        .bind(start.to_rfc3339())
        .bind(end.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_payment_row).collect()
    }

    async fn find_invoice_items(&self, invoice_id: Uuid) -> Result<Vec<InvoiceItem>, RepositoryError> {
        let rows = sqlx::query_as::<_, InvoiceItemRow>(
            r#"
            SELECT id, description, item_code, quantity, unit_price, amount, is_gst_free
            FROM invoice_items
            WHERE invoice_id = $1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(invoice_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_invoice_item_row).collect()
    }

    async fn next_invoice_number(&self, year: i32) -> Result<String, RepositoryError> {
        let prefix = format!("INV-{}-", year);
        let like_pattern = format!("{}%", prefix);

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM invoices WHERE invoice_number LIKE $1",
        )
        .bind(like_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        let sequence = count + 1;
        Ok(format!("{}{:05}", prefix, sequence))
    }
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, RepositoryError> {
    Uuid::parse_str(value).map_err(|err| {
        RepositoryError::Database(format!("Invalid UUID in {field}: {err}"))
    })
}

fn parse_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, RepositoryError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|err| RepositoryError::Database(format!("Invalid RFC3339 datetime in {field}: {err}")))
}

fn parse_naive_date(value: &str, field: &str) -> Result<NaiveDate, RepositoryError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|err| RepositoryError::Database(format!("Invalid date in {field}: {err}")))
}

fn parse_enum<T>(value: &str, field: &str) -> Result<T, RepositoryError>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|err| RepositoryError::Database(format!("Invalid enum in {field}: {err}")))
}

fn invalid_data(field: &str, value: i64) -> RepositoryError {
    RepositoryError::Database(format!("Invalid numeric value in {field}: {value}"))
}
