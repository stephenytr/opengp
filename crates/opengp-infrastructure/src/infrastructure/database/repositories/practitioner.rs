use async_trait::async_trait;
use chrono::Utc;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use opengp_domain::domain::user::{Practitioner, PractitionerRepository, RepositoryError};

#[derive(Debug, FromRow)]
struct PractitionerQueryRow {
    id: Uuid,
    first_name: String,
    last_name: String,
    email: Option<String>,
    role: String,
}

impl PractitionerQueryRow {
    fn into_practitioner(self) -> Result<Practitioner, RepositoryError> {
        let user_id = self.id;

        let (title, speciality, qualifications) = match self.role.as_str() {
            "Doctor" => (
                "Dr".to_string(),
                Some("General Practice".to_string()),
                vec!["MBBS".to_string()],
            ),
            "Nurse" => (
                "Nurse".to_string(),
                Some("Nursing".to_string()),
                vec!["RN".to_string()],
            ),
            _ => ("".to_string(), None, vec![]),
        };

        Ok(Practitioner {
            id: user_id,
            user_id: Some(user_id),
            first_name: self.first_name,
            middle_name: None,
            last_name: self.last_name,
            title,
            hpi_i: None,
            ahpra_registration: None,
            prescriber_number: None,
            provider_number: "PENDING".to_string(),
            speciality,
            qualifications,
            phone: None,
            email: self.email,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

/// SQLx-backed practitioner repository for PostgreSQL
///
/// Reads practitioner-facing details from the `users` table,
/// projecting user records into domain `Practitioner` models.
pub struct SqlxPractitionerRepository {
    pool: PgPool,
}

impl SqlxPractitionerRepository {
    /// Create a new practitioner repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PractitionerRepository for SqlxPractitionerRepository {
    async fn list_active(&self) -> Result<Vec<Practitioner>, RepositoryError> {
        let rows = sqlx::query_as::<_, PractitionerQueryRow>(
            r#"
        SELECT id, first_name, last_name, email, role
        FROM users
        WHERE (role = 'Doctor' OR role = 'Nurse') AND is_active = TRUE
        ORDER BY last_name, first_name
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let practitioners = rows
            .into_iter()
            .map(|row| row.into_practitioner())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(practitioners)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Practitioner>, RepositoryError> {
        let row = sqlx::query_as::<_, PractitionerQueryRow>(
            r#"
        SELECT id, first_name, last_name, email, role
        FROM users
        WHERE id = $1 AND (role = 'Doctor' OR role = 'Nurse') AND is_active = TRUE
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into_practitioner()).transpose()?)
    }
}
