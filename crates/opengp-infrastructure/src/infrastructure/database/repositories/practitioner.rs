use async_trait::async_trait;
use chrono::Utc;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use opengp_domain::domain::user::{Practitioner, PractitionerRepository, RepositoryError};
use crate::infrastructure::database::helpers::{bytes_to_uuid, uuid_to_bytes};

#[derive(Debug, FromRow)]
struct PractitionerQueryRow {
    id: Vec<u8>,
    first_name: String,
    last_name: String,
    email: Option<String>,
    role: String,
}

impl PractitionerQueryRow {
    fn into_practitioner(self) -> Result<Practitioner, RepositoryError> {
        let user_id = bytes_to_uuid(&self.id)
            .map_err(|_| RepositoryError::ConstraintViolation("Invalid UUID bytes".to_string()))?;

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

pub struct SqlxPractitionerRepository {
    pool: SqlitePool,
}

impl SqlxPractitionerRepository {
    pub fn new(pool: SqlitePool) -> Self {
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
        let id_bytes = uuid_to_bytes(&id);

        let row = sqlx::query_as::<_, PractitionerQueryRow>(
            r#"
            SELECT id, first_name, last_name, email, role
            FROM users
            WHERE id = ? AND (role = 'Doctor' OR role = 'Nurse') AND is_active = TRUE
            "#,
        )
        .bind(id_bytes)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into_practitioner()).transpose()?)
    }
}
