use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentSearchCriteria, AppointmentService, AvailabilityService,
    CalendarAppointment,
};
use opengp_domain::domain::audit::{
    AuditEntry, AuditRepository, AuditRepositoryError, AuditService,
};
use opengp_domain::domain::clinical::{
    Allergy, AllergyRepository, ClinicalRepositories, ClinicalService, ConsultationRepository,
    FamilyHistory, FamilyHistoryRepository, MedicalHistory, MedicalHistoryRepository,
    RepositoryError as ClinicalRepositoryError, SocialHistory, SocialHistoryRepository, VitalSigns,
    VitalSignsRepository,
};
use opengp_domain::domain::error::RepositoryError;
use opengp_domain::domain::patient::{PatientRepository, PatientService};
use opengp_domain::domain::user::{
    AuthService, PasswordError, PasswordHasher, SessionRepository, UserRepository,
    WorkingHours, WorkingHoursRepository,
};
#[cfg(test)]
use opengp_domain::domain::user::{Permission, Role, User};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use opengp_infrastructure::infrastructure::database::mocks::{
    MockAppointmentRepository, MockConsultationRepository, MockPatientRepository,
};
use opengp_infrastructure::infrastructure::database::repositories::InMemorySessionRepository;
#[cfg(not(test))]
use opengp_infrastructure::infrastructure::database::repositories::PostgresUserRepository;
use sqlx::PgPool;
#[cfg(test)]
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
    pub availability_service: Arc<AvailabilityService>,
    pub clinical_service: Arc<ClinicalService>,
}

impl ApiServices {
    pub async fn new(config: &ApiConfig, pool: &PgPool) -> Result<Self, ApiError> {
        unsafe {
            std::env::set_var("ENCRYPTION_KEY", &config.encryption_key);
        }

        let encryption_service = Arc::new(
            EncryptionService::new().map_err(|e| ApiError::EncryptionInit(e.to_string()))?,
        );

        let password_hasher: Arc<dyn PasswordHasher> = Arc::new(DevPasswordHasher);
        let user_repository = build_user_repository(pool, password_hasher.clone());
        let session_repository: Arc<dyn SessionRepository> =
            Arc::new(InMemorySessionRepository::new());
        let auth_service = Arc::new(AuthService::new(
            user_repository.clone(),
            password_hasher,
            session_repository,
            config.session_timeout_minutes,
        ));

        let patient_repository: Arc<dyn PatientRepository> = Arc::new(
            opengp_infrastructure::infrastructure::database::repositories::SqlxPatientRepository::new(
                pool.clone(),
                encryption_service.clone(),
            ),
        );
        let patient_service = Arc::new(PatientService::new(patient_repository));

        let audit_repository: Arc<dyn AuditRepository> = Arc::new(NoopAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repository));

        let appointment_repository = Arc::new(
            opengp_infrastructure::infrastructure::database::repositories::SqlxAppointmentRepository::new(
                pool.clone(),
            ),
        );
        let appointment_service = Arc::new(AppointmentService::new(
            appointment_repository.clone() as Arc<dyn opengp_domain::domain::appointment::AppointmentRepository>,
            audit_service.clone(),
            appointment_repository.clone() as Arc<dyn opengp_domain::domain::appointment::AppointmentCalendarQuery>,
        ));
        let working_hours_repository: Arc<dyn WorkingHoursRepository> = Arc::new(
            opengp_infrastructure::infrastructure::database::repositories::SqlxWorkingHoursRepository::new(
                pool.clone(),
            ),
        );
        let availability_service = Arc::new(AvailabilityService::new(
            appointment_repository,
            working_hours_repository,
        ));

        let consultation_repository: Arc<dyn ConsultationRepository> = Arc::new(
            opengp_infrastructure::infrastructure::database::repositories::SqlxClinicalRepository::new(
                pool.clone(),
                encryption_service.clone(),
            ),
        );
        let clinical_repositories = ClinicalRepositories {
            consultation: consultation_repository,
            allergy: Arc::new(
                opengp_infrastructure::infrastructure::database::repositories::SqlxAllergyRepository::new(
                    pool.clone(),
                    encryption_service.clone(),
                ),
            ),
            medical_history: Arc::new(
                opengp_infrastructure::infrastructure::database::repositories::SqlxMedicalHistoryRepository::new(
                    pool.clone(),
                    encryption_service.clone(),
                ),
            ),
            vital_signs: Arc::new(
                opengp_infrastructure::infrastructure::database::repositories::SqlxVitalSignsRepository::new(
                    pool.clone(),
                    encryption_service.clone(),
                ),
            ),
            social_history: Arc::new(
                opengp_infrastructure::infrastructure::database::repositories::SqlxSocialHistoryRepository::new(
                    pool.clone(),
                    encryption_service.clone(),
                ),
            ),
            family_history: Arc::new(
                opengp_infrastructure::infrastructure::database::repositories::SqlxFamilyHistoryRepository::new(
                    pool.clone(),
                    encryption_service.clone(),
                ),
            ),
        };
        let clinical_service = Arc::new(ClinicalService::new(
            clinical_repositories,
            patient_service.clone(),
            audit_service.clone(),
        ));

        Ok(Self {
            audit_service,
            encryption_service,
            auth_service,
            patient_service,
            appointment_service,
            availability_service,
            clinical_service,
        })
    }
}

#[cfg(test)]
fn build_user_repository(
    _pool: &PgPool,
    password_hasher: Arc<dyn PasswordHasher>,
) -> Arc<dyn UserRepository> {
    Arc::new(InMemoryUserRepository::with_default_users(password_hasher))
}

#[cfg(not(test))]
fn build_user_repository(
    pool: &PgPool,
    _password_hasher: Arc<dyn PasswordHasher>,
) -> Arc<dyn UserRepository> {
    Arc::new(PostgresUserRepository::new(pool.clone()))
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

struct NoopWorkingHoursRepository;

#[async_trait]
impl WorkingHoursRepository for NoopWorkingHoursRepository {
    async fn find_by_practitioner(
        &self,
        _practitioner_id: Uuid,
    ) -> Result<Vec<WorkingHours>, RepositoryError> {
        Ok(vec![])
    }

    async fn find_for_day(
        &self,
        _practitioner_id: Uuid,
        _day_of_week: u8,
    ) -> Result<Option<WorkingHours>, RepositoryError> {
        Ok(None)
    }

    async fn save(&self, working_hours: WorkingHours) -> Result<WorkingHours, RepositoryError> {
        Ok(working_hours)
    }

    async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
        Ok(())
    }
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        Ok(self.users.read().await.iter().find(|u| u.id == id).cloned())
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
        if users
            .iter()
            .any(|existing| existing.username == user.username)
        {
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

struct NoopAllergyRepository;

#[async_trait]
impl AllergyRepository for NoopAllergyRepository {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<Allergy>, ClinicalRepositoryError> {
        Ok(None)
    }

    async fn find_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Vec<Allergy>, ClinicalRepositoryError> {
        Ok(vec![])
    }

    async fn find_active_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Vec<Allergy>, ClinicalRepositoryError> {
        Ok(vec![])
    }

    async fn create(&self, allergy: Allergy) -> Result<Allergy, ClinicalRepositoryError> {
        Ok(allergy)
    }

    async fn update(&self, allergy: Allergy) -> Result<Allergy, ClinicalRepositoryError> {
        Ok(allergy)
    }

    async fn deactivate(&self, _id: Uuid) -> Result<(), ClinicalRepositoryError> {
        Ok(())
    }
}

struct NoopMedicalHistoryRepository;

#[async_trait]
impl MedicalHistoryRepository for NoopMedicalHistoryRepository {
    async fn find_by_id(
        &self,
        _id: Uuid,
    ) -> Result<Option<MedicalHistory>, ClinicalRepositoryError> {
        Ok(None)
    }

    async fn find_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Vec<MedicalHistory>, ClinicalRepositoryError> {
        Ok(vec![])
    }

    async fn find_active_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Vec<MedicalHistory>, ClinicalRepositoryError> {
        Ok(vec![])
    }

    async fn create(
        &self,
        history: MedicalHistory,
    ) -> Result<MedicalHistory, ClinicalRepositoryError> {
        Ok(history)
    }

    async fn update(
        &self,
        history: MedicalHistory,
    ) -> Result<MedicalHistory, ClinicalRepositoryError> {
        Ok(history)
    }
}

struct NoopVitalSignsRepository;

#[async_trait]
impl VitalSignsRepository for NoopVitalSignsRepository {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<VitalSigns>, ClinicalRepositoryError> {
        Ok(None)
    }

    async fn find_by_patient(
        &self,
        _patient_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<VitalSigns>, ClinicalRepositoryError> {
        Ok(vec![])
    }

    async fn find_latest_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Option<VitalSigns>, ClinicalRepositoryError> {
        Ok(None)
    }

    async fn create(&self, vitals: VitalSigns) -> Result<VitalSigns, ClinicalRepositoryError> {
        Ok(vitals)
    }
}

struct NoopSocialHistoryRepository;

#[async_trait]
impl SocialHistoryRepository for NoopSocialHistoryRepository {
    async fn find_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Option<SocialHistory>, ClinicalRepositoryError> {
        Ok(None)
    }

    async fn create(
        &self,
        history: SocialHistory,
    ) -> Result<SocialHistory, ClinicalRepositoryError> {
        Ok(history)
    }

    async fn update(
        &self,
        history: SocialHistory,
    ) -> Result<SocialHistory, ClinicalRepositoryError> {
        Ok(history)
    }
}

struct NoopFamilyHistoryRepository;

#[async_trait]
impl FamilyHistoryRepository for NoopFamilyHistoryRepository {
    async fn find_by_id(
        &self,
        _id: Uuid,
    ) -> Result<Option<FamilyHistory>, ClinicalRepositoryError> {
        Ok(None)
    }

    async fn find_by_patient(
        &self,
        _patient_id: Uuid,
    ) -> Result<Vec<FamilyHistory>, ClinicalRepositoryError> {
        Ok(vec![])
    }

    async fn create(
        &self,
        history: FamilyHistory,
    ) -> Result<FamilyHistory, ClinicalRepositoryError> {
        Ok(history)
    }

    async fn update(
        &self,
        history: FamilyHistory,
    ) -> Result<FamilyHistory, ClinicalRepositoryError> {
        Ok(history)
    }

    async fn delete(&self, _id: Uuid) -> Result<(), ClinicalRepositoryError> {
        Ok(())
    }
}
