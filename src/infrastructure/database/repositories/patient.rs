use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::patient::{Patient, PatientRepository, RepositoryError};

pub struct SqlxPatientRepository {
    pool: SqlitePool,
}

impl SqlxPatientRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PatientRepository for SqlxPatientRepository {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<Patient>, RepositoryError> {
        todo!("Implement find_by_id")
    }
    
    async fn find_by_medicare(&self, _medicare: &str) -> Result<Option<Patient>, RepositoryError> {
        todo!("Implement find_by_medicare")
    }
    
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        todo!("Implement create")
    }
    
    async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        todo!("Implement update")
    }
    
    async fn deactivate(&self, _id: Uuid) -> Result<(), RepositoryError> {
        todo!("Implement deactivate")
    }
}
