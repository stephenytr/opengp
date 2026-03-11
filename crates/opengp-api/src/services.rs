use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentSearchCriteria, AppointmentService, CalendarAppointment,
};
use opengp_domain::domain::error::RepositoryError;
use opengp_domain::domain::audit::{AuditEntry, AuditRepository, AuditRepositoryError, AuditService};
use opengp_domain::domain::patient::{PatientService, PatientRepository};
use opengp_domain::domain::user::{
    AuthService, PasswordError, PasswordHasher, Permission, Role, SessionRepository, User,
    UserRepository,
};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::mocks::{
    MockAppointmentRepository, MockPatientRepository,
};
use opengp_infrastructure::infrastructure::database::repositories::InMemorySessionRepository;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{ApiConfig, ApiError};

#[derive(Clone)]
pub struct ApiServices {
    pub audit_service: Arc<AuditService>,
    pub encryption_service: Arc<EncryptionService>,
    pub auth_service: Arc<AuthService>,
    pub patient_service: Arc<PatientService>,
    pub appointment_service: Arc<AppointmentService>,
}

impl ApiServices {
    pub async fn new(config: &ApiConfig) -> Result<Self, ApiError> {
        unsafe {
            std::env::set_var("ENCRYPTION_KEY", &config.encryption_key);
        }

        let encryption_service = Arc::new(
            EncryptionService::new().map_err(|e| ApiError::EncryptionInit(e.to_string()))?,
        );

        let password_hasher: Arc<dyn PasswordHasher> = Arc::new(DevPasswordHasher);
        let user_repository: Arc<dyn UserRepository> = Arc::new(
            InMemoryUserRepository::with_default_users(password_hasher.clone()),
        );
        let session_repository: Arc<dyn SessionRepository> = Arc::new(InMemorySessionRepository::new());
        let auth_service = Arc::new(AuthService::new(
            user_repository.clone(),
            password_hasher,
            session_repository,
        ));

        let patient_repository: Arc<dyn PatientRepository> = Arc::new(MockPatientRepository::new());
        let patient_service = Arc::new(PatientService::new(patient_repository));

        let audit_repository: Arc<dyn AuditRepository> = Arc::new(NoopAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repository));

        let appointment_repository = Arc::new(MockAppointmentRepository::new());
        let appointment_service = Arc::new(AppointmentService::new(
            appointment_repository,
            audit_service.clone(),
            Arc::new(NoopAppointmentCalendarQuery),
        ));

        Ok(Self {
            audit_service,
            encryption_service,
            auth_service,
            patient_service,
            appointment_service,
        })
    }
}

struct NoopAppointmentCalendarQuery;

#[async_trait]
impl AppointmentCalendarQuery for NoopAppointmentCalendarQuery {
    async fn find_calendar_appointments(
        &self,
        _criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<CalendarAppointment>, RepositoryError> {
        Ok(vec![])
    }
}

struct InMemoryUserRepository {
    users: RwLock<Vec<User>>,
}

struct DevPasswordHasher;

impl PasswordHasher for DevPasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        if password.is_empty() {
            return Err(PasswordError::EmptyPassword);
        }

        Ok(password.to_string())
    }

    fn verify_password(&self, password_hash: &str, password: &str) -> Result<(), PasswordError> {
        if password_hash == password {
            Ok(())
        } else {
            Err(PasswordError::VerificationFailed)
        }
    }
}

impl InMemoryUserRepository {
    fn with_default_users(password_hasher: Arc<dyn PasswordHasher>) -> Self {
        let now = Utc::now();
        let password_hash = password_hasher
            .hash_password("correct-horse-battery-staple")
            .expect("default auth user password hash should be generated");

        let receptionist_password_hash = password_hasher
            .hash_password("desk-passphrase")
            .expect("default receptionist password hash should be generated");

        Self {
            users: RwLock::new(vec![
                User {
                    id: Uuid::new_v4(),
                    username: "dr_smith".to_string(),
                    password_hash: Some(password_hash),
                    email: Some("dr_smith@example.com".to_string()),
                    first_name: "Sarah".to_string(),
                    last_name: "Smith".to_string(),
                    role: Role::Doctor,
                    additional_permissions: vec![Permission::PatientRead],
                    is_active: true,
                    is_locked: false,
                    failed_login_attempts: 0,
                    last_login: None,
                    password_changed_at: now,
                    created_at: now,
                    updated_at: now,
                },
                User {
                    id: Uuid::new_v4(),
                    username: "recep_amy".to_string(),
                    password_hash: Some(receptionist_password_hash),
                    email: Some("recep_amy@example.com".to_string()),
                    first_name: "Amy".to_string(),
                    last_name: "Frontdesk".to_string(),
                    role: Role::Receptionist,
                    additional_permissions: vec![],
                    is_active: true,
                    is_locked: false,
                    failed_login_attempts: 0,
                    last_login: None,
                    password_changed_at: now,
                    created_at: now,
                    updated_at: now,
                },
            ]),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .users
            .read()
            .await
            .iter()
            .find(|u| u.id == id)
            .cloned())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .users
            .read()
            .await
            .iter()
            .find(|u| u.username == username)
            .cloned())
    }

    async fn find_all(&self) -> Result<Vec<User>, RepositoryError> {
        Ok(self.users.read().await.clone())
    }

    async fn find_by_role(&self, role: Role) -> Result<Vec<User>, RepositoryError> {
        Ok(self
            .users
            .read()
            .await
            .iter()
            .filter(|u| u.role == role)
            .cloned()
            .collect())
    }

    async fn create(&self, user: User) -> Result<User, RepositoryError> {
        let mut users = self.users.write().await;
        if users.iter().any(|existing| existing.username == user.username) {
            return Err(RepositoryError::ConstraintViolation(
                "Username already exists".to_string(),
            ));
        }

        users.push(user.clone());
        Ok(user)
    }

    async fn update(&self, user: User) -> Result<User, RepositoryError> {
        let mut users = self.users.write().await;
        let Some(existing) = users.iter_mut().find(|u| u.id == user.id) else {
            return Err(RepositoryError::NotFound);
        };

        *existing = user.clone();
        Ok(user)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let mut users = self.users.write().await;
        let Some(existing) = users.iter_mut().find(|u| u.id == id) else {
            return Err(RepositoryError::NotFound);
        };

        existing.is_active = false;
        existing.updated_at = Utc::now();
        Ok(())
    }
}

struct NoopAuditRepository;

#[async_trait]
impl AuditRepository for NoopAuditRepository {
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, AuditRepositoryError> {
        Ok(entry)
    }

    async fn find_by_entity(
        &self,
        _entity_type: &str,
        _entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }

    async fn find_by_user(&self, _user_id: Uuid) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }

    async fn find_by_time_range(
        &self,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }

    async fn find_by_entity_and_time_range(
        &self,
        _entity_type: &str,
        _entity_id: Uuid,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }
}
