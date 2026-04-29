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
    id: Uuid,
    invoice_number: String,
    patient_id: Uuid,
    practitioner_id: Uuid,
    consultation_id: Option<Uuid>,
    billing_type: String,
    status: String,
    issue_date: NaiveDate,
    due_date: Option<NaiveDate>,
    subtotal: f64,
    gst_amount: f64,
    total_amount: f64,
    amount_paid: f64,
    amount_outstanding: f64,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct InvoiceItemRow {
    id: Uuid,
    description: String,
    item_code: Option<String>,
    quantity: i32,
    unit_price: f64,
    amount: f64,
    is_gst_free: bool,
}

#[derive(Debug, FromRow)]
struct PaymentRow {
    id: Uuid,
    invoice_id: Uuid,
    patient_id: Uuid,
    payment_date: NaiveDate,
    amount: f64,
    payment_method: String,
    reference: Option<String>,
    notes: Option<String>,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct ClaimRow {
    id: Uuid,
    invoice_id: Option<Uuid>,
    patient_id: Uuid,
    practitioner_id: Uuid,
    claim_type: String,
    status: String,
    service_date: NaiveDate,
    total_claimed: f64,
    total_benefit: f64,
    reference_number: Option<String>,
    submitted_at: Option<DateTime<Utc>>,
    processed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

pub struct SqlxBillingRepository {
    pool: PgPool,
}

impl SqlxBillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn map_invoice_row(&self, row: InvoiceRow) -> Result<Invoice, RepositoryError> {
        let items = self.find_invoice_items(row.id).await?;

        Ok(Invoice {
            id: row.id,
            patient_id: row.patient_id,
            practitioner_id: row.practitioner_id,
            consultation_id: row.consultation_id,
            invoice_number: row.invoice_number,
            invoice_date: row.issue_date,
            due_date: row.due_date,
            items,
            subtotal: row.subtotal,
            gst_amount: row.gst_amount,
            total_amount: row.total_amount,
            amount_paid: row.amount_paid,
            amount_outstanding: row.amount_outstanding,
            status: parse_enum(&row.status, "invoices.status")?,
            billing_type: parse_enum(&row.billing_type, "invoices.billing_type")?,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.practitioner_id,
            updated_by: None,
        })
    }

    fn map_invoice_item_row(row: InvoiceItemRow) -> Result<InvoiceItem, RepositoryError> {
        Ok(InvoiceItem {
            id: row.id,
            description: row.description,
            item_code: row.item_code,
            quantity: row.quantity as u32,
            unit_price: row.unit_price,
            amount: row.amount,
            is_gst_free: row.is_gst_free,
        })
    }

    fn map_payment_row(row: PaymentRow) -> Result<Payment, RepositoryError> {
        Ok(Payment {
            id: row.id,
            invoice_id: row.invoice_id,
            patient_id: row.patient_id,
            payment_date: row
                .payment_date
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                .and_utc(),
            amount: row.amount,
            payment_method: parse_enum(&row.payment_method, "payments.payment_method")?,
            reference: row.reference,
            notes: row.notes,
            created_at: row.created_at,
            created_by: row.created_by,
        })
    }

    fn map_claim_row(row: ClaimRow) -> Result<MedicareClaim, RepositoryError> {
        let total_claimed = row.total_claimed;
        let total_benefit = row.total_benefit;

        Ok(MedicareClaim {
            id: row.id,
            patient_id: row.patient_id,
            practitioner_id: row.practitioner_id,
            consultation_id: None,
            invoice_id: row.invoice_id,
            claim_reference: row.reference_number,
            service_date: row.service_date,
            items: Vec::<MBSItem>::new(),
            total_claimed,
            total_benefit,
            patient_contribution: total_claimed - total_benefit,
            claim_type: parse_enum(&row.claim_type, "medicare_claims.claim_type")?,
            status: parse_enum(&row.status, "medicare_claims.status")?,
            submitted_at: row.submitted_at,
            processed_at: row.processed_at,
            rejection_reason: None,
            created_at: row.created_at,
            created_by: row.practitioner_id,
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
            .bind(item.id)
            .bind(invoice_id)
            .bind(&item.description)
            .bind(&item.item_code)
            .bind(quantity)
            .bind(item.unit_price)
            .bind(item.amount)
            .bind(if item.is_gst_free { 1_i64 } else { 0_i64 })
            .bind(created_at)
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
        .bind(id)
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
        .bind(patient_id)
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
        .bind(start)
        .bind(end)
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
        .bind(invoice.id)
        .bind(&invoice.invoice_number)
        .bind(invoice.patient_id)
        .bind(invoice.practitioner_id)
        .bind(invoice.consultation_id)
        .bind(invoice.billing_type.to_string())
        .bind(invoice.status.to_string())
        .bind(invoice.invoice_date)
        .bind(invoice.due_date)
        .bind(invoice.subtotal)
        .bind(invoice.gst_amount)
        .bind(invoice.total_amount)
        .bind(invoice.amount_paid)
        .bind(invoice.amount_outstanding)
        .bind(&invoice.notes)
        .bind(invoice.created_at)
        .bind(invoice.updated_at)
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
        .bind(invoice.patient_id)
        .bind(invoice.practitioner_id)
        .bind(invoice.consultation_id)
        .bind(invoice.billing_type.to_string())
        .bind(invoice.status.to_string())
        .bind(invoice.invoice_date)
        .bind(invoice.due_date)
        .bind(invoice.subtotal)
        .bind(invoice.gst_amount)
        .bind(invoice.total_amount)
        .bind(invoice.amount_paid)
        .bind(invoice.amount_outstanding)
        .bind(&invoice.notes)
        .bind(invoice.updated_at)
        .bind(invoice.id)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        sqlx::query("DELETE FROM invoice_items WHERE invoice_id = $1")
            .bind(invoice.id)
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
            .bind(Utc::now())
            .bind(id)
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
        .bind(id)
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
        .bind(claim.id)
        .bind(claim.invoice_id)
        .bind(claim.patient_id)
        .bind(claim.practitioner_id)
        .bind(claim.claim_type.to_string())
        .bind(claim.status.to_string())
        .bind(claim.service_date)
        .bind(claim.total_claimed)
        .bind(claim.total_benefit)
        .bind(&claim.claim_reference)
        .bind(claim.submitted_at)
        .bind(claim.processed_at)
        .bind(claim.created_at)
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
        .bind(patient_id)
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
            .bind(id)
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
        .bind(payment.id)
        .bind(payment.invoice_id)
        .bind(payment.patient_id)
        .bind(payment.amount)
        .bind(payment.payment_method.to_string())
        .bind(payment.payment_date.date_naive())
        .bind(&payment.reference)
        .bind(&payment.notes)
        .bind(payment.created_by)
        .bind(payment.created_at)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        Ok(payment)
    }

    async fn find_payments_by_invoice(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<Payment>, RepositoryError> {
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
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_payment_row).collect()
    }

    async fn find_payments_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Payment>, RepositoryError> {
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
        .bind(patient_id)
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
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_repository_error)?;

        rows.into_iter().map(Self::map_payment_row).collect()
    }

    async fn find_invoice_items(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceItem>, RepositoryError> {
        let rows = sqlx::query_as::<_, InvoiceItemRow>(
            r#"
            SELECT id, description, item_code, quantity, unit_price, amount, is_gst_free
            FROM invoice_items
            WHERE invoice_id = $1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(invoice_id)
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

fn parse_enum<T>(value: &str, field: &str) -> Result<T, RepositoryError>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|err| RepositoryError::Database(format!("Invalid enum in {field}: {err}")))
}
