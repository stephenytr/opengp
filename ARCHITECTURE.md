# OpenGP Architecture Documentation

**Version**: 1.0  
**Last Updated**: 2026-02-11  
**Status**: Living Document

---

## Table of Contents

1. [Overview](#overview)
2. [Architectural Principles](#architectural-principles)
3. [System Architecture](#system-architecture)
4. [Layer Architecture](#layer-architecture)
5. [Module Structure](#module-structure)
6. [Component Architecture](#component-architecture)
7. [Data Architecture](#data-architecture)
8. [Security Architecture](#security-architecture)
9. [Integration Architecture](#integration-architecture)
10. [Development Patterns](#development-patterns)
11. [Testing Strategy](#testing-strategy)
12. [Deployment Architecture](#deployment-architecture)
13. [Performance Considerations](#performance-considerations)
14. [Decision Log](#decision-log)

---

## Overview

OpenGP is built using a **layered, component-based architecture** with clear separation of concerns. The system is designed to be:

- **Modular**: Each domain is isolated with well-defined interfaces
- **Testable**: Dependency injection and trait-based abstractions enable comprehensive testing
- **Maintainable**: Clear boundaries between layers reduce coupling
- **Scalable**: Architecture supports migration from SQLite to PostgreSQL
- **Secure**: Security is built into every layer, not bolted on

### Key Characteristics

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Language** | Rust | Memory safety, performance, strong type system |
| **UI Framework** | Ratatui | Terminal-based, cross-platform, resource-efficient |
| **Async Runtime** | Tokio | Industry standard, excellent ecosystem |
| **Database** | SQLx | Compile-time query validation, multi-DB support |
| **Architecture Style** | Layered + Domain-Driven Design | Clear boundaries, business logic isolation |

---

## Architectural Principles

### 1. Separation of Concerns

Each layer has a single, well-defined responsibility:

```
UI Layer        → Rendering and user interaction
Application     → Coordination and workflow orchestration
Domain          → Business logic and rules
Data            → Persistence and queries
Infrastructure  → Cross-cutting concerns (auth, crypto, audit)
```

### 2. Dependency Rule

**Dependencies point inward**: Outer layers depend on inner layers, never the reverse.

```
UI → Application → Domain ← Data
              ↓
        Infrastructure
```

### 3. Interface Segregation

Modules communicate through **traits (interfaces)**, not concrete implementations:

```rust
// Domain defines what it needs
pub trait PatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>>;
    async fn save(&self, patient: Patient) -> Result<()>;
}

// Data layer implements it
impl PatientRepository for SqlitePatientRepository { ... }
```

### 4. Explicit Over Implicit

- No magic: All behavior should be traceable
- No hidden global state
- Explicit error handling (no panics in production code)
- Clear ownership and lifetimes

### 5. Fail-Safe Defaults

- Authentication required by default
- Audit logging on by default
- Encryption enabled by default
- Least privilege access model

---

## System Architecture

### High-Level System Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Terminal UI (Ratatui)                        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────┐ │
│  │  Patient UI  │ │Appointment UI│ │  Clinical UI │ │ Billing UI │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────────┐
│                      Application Layer (App)                         │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │              Component Manager & Event Router                   │ │
│  │  • State Management        • Action Dispatch                    │ │
│  │  • Event Handling          • Component Lifecycle                │ │
│  └────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────────┐
│                          Domain Layer                                │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ Patient  │ │Appointment│ │ Clinical │ │  Billing │ │  User   │ │
│  │  Module  │ │  Module   │ │  Module  │ │  Module  │ │ Module  │ │
│  └────┬─────┘ └─────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬────┘ │
│       │             │             │             │             │      │
│  ┌────┴─────────────┴─────────────┴─────────────┴─────────────┴───┐ │
│  │             Domain Services & Business Logic                    │ │
│  └──────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────────┐
│                        Data Layer (SQLx)                             │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │         Repository Implementations (Trait-based)                │ │
│  │  • PatientRepository    • AppointmentRepository                 │ │
│  │  • ClinicalRepository   • BillingRepository                     │ │
│  └────────────────┬───────────────────────────┬───────────────────┘ │
│                   │                           │                      │
│          ┌────────┴────────┐         ┌───────┴────────┐            │
│          │  SQLite Pool    │   OR    │ PostgreSQL Pool│            │
│          └─────────────────┘         └────────────────┘            │
└─────────────────────────────────────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────────┐
│                    Infrastructure Layer                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │   Auth   │ │  Crypto  │ │  Audit   │ │   HTTP   │ │  Config  │ │
│  │  Module  │ │  Module  │ │  Logger  │ │  Client  │ │  Module  │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────────┐
│                      External Systems                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │ Medicare │ │   PBS    │ │    HI    │ │   MHR    │ │Pathology │ │
│  │  Online  │ │   API    │ │ Service  │ │   API    │ │   Labs   │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Layer Architecture

### 1. UI Layer (Ratatui)

**Responsibility**: Rendering terminal UI and handling user input.

```rust
// src/ui/mod.rs
pub mod tui;      // Terminal setup and management
pub mod event;    // Event handling and key mappings
pub mod theme;    // Color schemes and styling
pub mod widgets;  // Custom reusable widgets

// src/ui/tui.rs
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_rx: UnboundedReceiver<Event>,
}

impl Tui {
    pub fn new() -> Result<Self> { ... }
    pub fn enter(&mut self) -> Result<()> { ... }
    pub fn exit(&mut self) -> Result<()> { ... }
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    { ... }
    pub async fn next(&mut self) -> Result<Event> { ... }
}
```

**Key Patterns**:
- **Immediate Mode Rendering**: UI rebuilt every frame
- **Event-Driven**: Async event handling via Tokio channels
- **Stateless Widgets**: UI components don't hold state
- **Layout Engine**: Constraint-based responsive layouts

**Widget Architecture**:
```rust
// src/ui/widgets/patient_table.rs
pub struct PatientTable<'a> {
    patients: &'a [Patient],
    selected: Option<usize>,
    theme: &'a Theme,
}

impl<'a> Widget for PatientTable<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render logic
    }
}
```

### 2. Application Layer

**Responsibility**: Coordinate components, manage application state, route events.

```rust
// src/app.rs
pub struct App {
    // Configuration
    config: Config,
    
    // Infrastructure
    db: DbPool,
    audit_logger: Arc<AuditLogger>,
    
    // State
    current_user: Option<User>,
    mode: AppMode,
    active_screen: Screen,
    
    // Components (one per major feature)
    patient_component: Box<dyn Component>,
    appointment_component: Box<dyn Component>,
    clinical_component: Box<dyn Component>,
    billing_component: Box<dyn Component>,
    
    // Communication
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> { ... }
    
    pub async fn run(&mut self, mut tui: Tui) -> Result<()> {
        loop {
            // Get next event
            let event = tui.next().await?;
            
            // Convert to action
            let action = self.handle_event(event);
            
            // Process action
            self.update(action).await?;
            
            // Render
            tui.draw(|f| self.render(f))?;
            
            // Check exit condition
            if self.should_quit { break; }
        }
        Ok(())
    }
    
    fn handle_event(&mut self, event: Event) -> Action { ... }
    async fn update(&mut self, action: Action) -> Result<()> { ... }
    fn render(&mut self, frame: &mut Frame) { ... }
}
```

**Event Flow**:
```
User Input (Key Press)
    ↓
Event (crossterm::Event)
    ↓
App::handle_event()
    ↓
Action (domain-specific)
    ↓
Component::handle_action()
    ↓
Domain Service Call
    ↓
Repository Operation
    ↓
State Update
    ↓
Re-render
```

### 3. Domain Layer

**Responsibility**: Business logic, domain models, validation rules.

#### Module Structure

Each domain module follows this pattern:

```rust
// src/domain/patient/mod.rs
mod model;         // Domain entities
mod service;       // Business logic
mod repository;    // Persistence interface
mod dto;           // Data transfer objects
mod error;         // Domain-specific errors

pub use model::*;
pub use service::*;
pub use repository::*;
pub use dto::*;
pub use error::*;
```

#### Domain Model Example

```rust
// src/domain/patient/model.rs
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;

/// Core patient entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Uuid,
    
    // Healthcare Identifiers
    pub ihi: Option<String>,              // Individual Healthcare Identifier
    pub medicare_number: Option<String>,
    pub medicare_irn: Option<u8>,         // 1-9
    pub medicare_expiry: Option<NaiveDate>,
    
    // Demographics
    pub title: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub preferred_name: Option<String>,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    
    // Contact
    pub address: Address,
    pub phone_home: Option<String>,
    pub phone_mobile: Option<String>,
    pub email: Option<String>,
    
    // Emergency Contact
    pub emergency_contact: Option<EmergencyContact>,
    
    // Status
    pub is_active: bool,
    pub is_deceased: bool,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Patient {
    /// Create new patient with validation
    pub fn new(data: NewPatientData) -> Result<Self, ValidationError> {
        // Validation logic
        Self::validate_names(&data.first_name, &data.last_name)?;
        Self::validate_date_of_birth(data.date_of_birth)?;
        if let Some(ref medicare) = data.medicare_number {
            Self::validate_medicare_number(medicare)?;
        }
        
        Ok(Self {
            id: Uuid::new_v4(),
            first_name: data.first_name,
            last_name: data.last_name,
            date_of_birth: data.date_of_birth,
            // ... initialize other fields
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    /// Calculate age in years
    pub fn age(&self) -> u32 {
        let today = Utc::now().date_naive();
        today.years_since(self.date_of_birth).unwrap_or(0)
    }
    
    /// Check if patient is a child (under 18)
    pub fn is_child(&self) -> bool {
        self.age() < 18
    }
    
    /// Update patient details
    pub fn update(&mut self, data: UpdatePatientData) -> Result<(), ValidationError> {
        if let Some(first_name) = data.first_name {
            Self::validate_name(&first_name)?;
            self.first_name = first_name;
        }
        // ... update other fields
        self.updated_at = Utc::now();
        Ok(())
    }
    
    // Private validation methods
    fn validate_names(first: &str, last: &str) -> Result<(), ValidationError> { ... }
    fn validate_date_of_birth(dob: NaiveDate) -> Result<(), ValidationError> { ... }
    fn validate_medicare_number(num: &str) -> Result<(), ValidationError> { ... }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub suburb: Option<String>,
    pub state: Option<String>,
    pub postcode: Option<String>,
    pub country: String,  // Default: "Australia"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub name: String,
    pub phone: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}
```

#### Service Layer Example

```rust
// src/domain/patient/service.rs
use async_trait::async_trait;

/// Patient service handles all patient-related business logic
pub struct PatientService {
    repository: Arc<dyn PatientRepository>,
    audit_logger: Arc<AuditLogger>,
    medicare_validator: Arc<dyn MedicareValidator>,
}

impl PatientService {
    pub fn new(
        repository: Arc<dyn PatientRepository>,
        audit_logger: Arc<AuditLogger>,
        medicare_validator: Arc<dyn MedicareValidator>,
    ) -> Self {
        Self {
            repository,
            audit_logger,
            medicare_validator,
        }
    }
    
    /// Register a new patient
    pub async fn register_patient(
        &self,
        data: NewPatientData,
        user: &User,
    ) -> Result<Patient, ServiceError> {
        // Business logic
        
        // 1. Check for duplicates
        if let Some(ref medicare) = data.medicare_number {
            if self.repository.find_by_medicare(medicare).await?.is_some() {
                return Err(ServiceError::DuplicatePatient);
            }
        }
        
        // 2. Validate Medicare number with external service (if provided)
        if let Some(ref medicare) = data.medicare_number {
            self.medicare_validator.validate(medicare).await?;
        }
        
        // 3. Create patient entity (domain validation happens here)
        let patient = Patient::new(data)?;
        
        // 4. Save to database
        let saved = self.repository.create(patient).await?;
        
        // 5. Audit log
        self.audit_logger.log(AuditEvent {
            user_id: user.id,
            action: AuditAction::PatientCreate,
            entity_id: saved.id,
            timestamp: Utc::now(),
        }).await?;
        
        Ok(saved)
    }
    
    /// Find patient with access control
    pub async fn find_patient(
        &self,
        id: Uuid,
        user: &User,
    ) -> Result<Option<Patient>, ServiceError> {
        // Check authorization
        if !user.can_access_patient(id) {
            return Err(ServiceError::Unauthorized);
        }
        
        // Fetch patient
        let patient = self.repository.find_by_id(id).await?;
        
        // Audit log
        if patient.is_some() {
            self.audit_logger.log(AuditEvent {
                user_id: user.id,
                action: AuditAction::PatientRead,
                entity_id: id,
                timestamp: Utc::now(),
            }).await?;
        }
        
        Ok(patient)
    }
    
    /// Search patients with pagination
    pub async fn search_patients(
        &self,
        query: PatientSearchQuery,
        user: &User,
    ) -> Result<PagedResult<Patient>, ServiceError> {
        // Authorization check
        if !user.has_permission(Permission::PatientSearch) {
            return Err(ServiceError::Unauthorized);
        }
        
        // Search
        let result = self.repository.search(query).await?;
        
        // Audit log
        self.audit_logger.log(AuditEvent {
            user_id: user.id,
            action: AuditAction::PatientSearch,
            entity_id: Uuid::nil(),  // No specific patient
            metadata: Some(serde_json::to_value(&query)?),
            timestamp: Utc::now(),
        }).await?;
        
        Ok(result)
    }
}

#[derive(Debug)]
pub enum ServiceError {
    DuplicatePatient,
    Unauthorized,
    ValidationError(ValidationError),
    RepositoryError(RepositoryError),
    AuditError(AuditError),
}
```

#### Repository Interface

```rust
// src/domain/patient/repository.rs
use async_trait::async_trait;

/// Repository interface for patient persistence
/// 
/// This trait defines what the domain needs from persistence,
/// without knowing how it's implemented.
#[async_trait]
pub trait PatientRepository: Send + Sync {
    /// Find patient by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError>;
    
    /// Find patient by Medicare number
    async fn find_by_medicare(&self, medicare: &str) -> Result<Option<Patient>, RepositoryError>;
    
    /// Search patients
    async fn search(&self, query: PatientSearchQuery) -> Result<PagedResult<Patient>, RepositoryError>;
    
    /// Create new patient
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError>;
    
    /// Update existing patient
    async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError>;
    
    /// Soft delete patient
    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError>;
    
    /// Check if patient exists
    async fn exists(&self, id: Uuid) -> Result<bool, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PatientSearchQuery {
    pub name: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub medicare_number: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}
```

### 4. Data Layer (SQLx)

**Responsibility**: Implement repository interfaces, manage database connections.

```rust
// src/infrastructure/database/repositories/patient.rs
use sqlx::{Pool, Sqlite};  // or Postgres
use async_trait::async_trait;

pub struct SqlxPatientRepository {
    pool: Pool<Sqlite>,
}

impl SqlxPatientRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PatientRepository for SqlxPatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
        let row = sqlx::query_as!(
            PatientRow,
            r#"
            SELECT 
                id, ihi, medicare_number, medicare_irn, medicare_expiry,
                title, first_name, middle_name, last_name, preferred_name,
                date_of_birth, gender,
                address_line1, address_line2, suburb, state, postcode, country,
                phone_home, phone_mobile, email,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                is_active, is_deceased,
                created_at, updated_at
            FROM patients
            WHERE id = ? AND is_active = TRUE
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into_domain()))
    }
    
    async fn search(&self, query: PatientSearchQuery) -> Result<PagedResult<Patient>, RepositoryError> {
        // Build dynamic query
        let mut sql = String::from("SELECT * FROM patients WHERE is_active = TRUE");
        let mut params = vec![];
        
        if let Some(ref name) = query.name {
            sql.push_str(" AND (first_name LIKE ? OR last_name LIKE ?)");
            params.push(format!("%{}%", name));
            params.push(format!("%{}%", name));
        }
        
        if let Some(dob) = query.date_of_birth {
            sql.push_str(" AND date_of_birth = ?");
            params.push(dob.to_string());
        }
        
        sql.push_str(" LIMIT ? OFFSET ?");
        let offset = query.page * query.page_size;
        
        // Execute query (simplified - use query builder in production)
        let rows = sqlx::query_as::<_, PatientRow>(&sql)
            .fetch_all(&self.pool)
            .await?;
        
        let total = self.count_search_results(&query).await?;
        
        Ok(PagedResult {
            items: rows.into_iter().map(|r| r.into_domain()).collect(),
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }
    
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO patients (
                id, ihi, medicare_number, medicare_irn, medicare_expiry,
                first_name, last_name, date_of_birth, gender,
                address_line1, suburb, state, postcode, country,
                phone_mobile, is_active, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            patient.id,
            patient.ihi,
            patient.medicare_number,
            patient.medicare_irn,
            patient.medicare_expiry,
            patient.first_name,
            patient.last_name,
            patient.date_of_birth,
            patient.gender.to_string(),
            patient.address.line1,
            patient.address.suburb,
            patient.address.state,
            patient.address.postcode,
            patient.address.country,
            patient.phone_mobile,
            patient.is_active,
            patient.created_at,
            patient.updated_at
        )
        .execute(&self.pool)
        .await?;
        
        Ok(patient)
    }
    
    // ... other methods
}

/// Database row representation
#[derive(sqlx::FromRow)]
struct PatientRow {
    id: Vec<u8>,  // UUID as bytes in SQLite
    ihi: Option<String>,
    medicare_number: Option<String>,
    first_name: String,
    last_name: String,
    date_of_birth: String,  // Date as string in SQLite
    // ... other fields
}

impl PatientRow {
    /// Convert database row to domain model
    fn into_domain(self) -> Patient {
        Patient {
            id: Uuid::from_slice(&self.id).unwrap(),
            ihi: self.ihi,
            medicare_number: self.medicare_number,
            first_name: self.first_name,
            last_name: self.last_name,
            date_of_birth: NaiveDate::parse_from_str(&self.date_of_birth, "%Y-%m-%d").unwrap(),
            // ... map other fields
        }
    }
}
```

### 5. Infrastructure Layer

**Responsibility**: Cross-cutting concerns (authentication, encryption, audit logging, configuration).

#### Authentication Module

```rust
// src/infrastructure/auth/mod.rs
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;

pub struct AuthService {
    repository: Arc<dyn UserRepository>,
    audit_logger: Arc<AuditLogger>,
    session_timeout: Duration,
}

impl AuthService {
    /// Authenticate user with username and password
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Session, AuthError> {
        // Find user
        let user = self.repository
            .find_by_username(username)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;
        
        // Check if active
        if !user.is_active {
            return Err(AuthError::AccountDisabled);
        }
        
        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AuthError::InvalidHash)?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;
        
        // Create session
        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id,
            created_at: Utc::now(),
            expires_at: Utc::now() + self.session_timeout,
        };
        
        // Audit log
        self.audit_logger.log(AuditEvent {
            user_id: user.id,
            action: AuditAction::Login,
            timestamp: Utc::now(),
        }).await?;
        
        Ok(session)
    }
    
    /// Hash password for storage
    pub fn hash_password(password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::HashingFailed)?
            .to_string();
        
        Ok(hash)
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
```

#### Encryption Module

```rust
// src/infrastructure/crypto/mod.rs
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// Initialize with key from environment
    pub fn new() -> Result<Self, CryptoError> {
        let key_bytes = Self::load_key_from_env()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|_| CryptoError::InvalidKey)?;
        
        Ok(Self { cipher })
    }
    
    /// Encrypt sensitive data
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, CryptoError> {
        // Generate random nonce
        let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        // Prepend nonce to ciphertext for storage
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        
        Ok(result)
    }
    
    /// Decrypt sensitive data
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, CryptoError> {
        if encrypted.len() < 12 {
            return Err(CryptoError::InvalidCiphertext);
        }
        
        // Extract nonce (first 12 bytes)
        let nonce = Nonce::from_slice(&encrypted[..12]);
        
        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, &encrypted[12..])
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| CryptoError::InvalidUtf8)
    }
    
    fn load_key_from_env() -> Result<Vec<u8>, CryptoError> {
        let key_hex = std::env::var("ENCRYPTION_KEY")
            .map_err(|_| CryptoError::MissingKey)?;
        
        hex::decode(key_hex)
            .map_err(|_| CryptoError::InvalidKeyFormat)
    }
}
```

#### Audit Logger

```rust
// src/infrastructure/audit/mod.rs
pub struct AuditLogger {
    repository: Arc<dyn AuditLogRepository>,
}

impl AuditLogger {
    pub async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        self.repository.create(event).await?;
        Ok(())
    }
    
    pub async fn search(
        &self,
        query: AuditSearchQuery,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        self.repository.search(query).await
    }
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub user_id: Uuid,
    pub action: AuditAction,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    // Authentication
    Login,
    Logout,
    LoginFailed,
    
    // Patient operations
    PatientCreate,
    PatientRead,
    PatientUpdate,
    PatientDelete,
    PatientSearch,
    
    // Clinical operations
    ConsultationCreate,
    ConsultationRead,
    ConsultationSign,
    
    // Prescription operations
    PrescriptionCreate,
    PrescriptionRead,
    PrescriptionCancel,
    
    // Data export
    ReportGenerate,
    DataExport,
}
```

---

## Component Architecture

### Component Trait

Every major UI feature is a `Component`:

```rust
// src/components/mod.rs
use async_trait::async_trait;
use ratatui::layout::Rect;
use ratatui::Frame;
use crossterm::event::{KeyEvent, MouseEvent};

/// Component trait for modular UI components
#[async_trait]
pub trait Component: Send {
    /// Initialize component (called once)
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Handle raw events and convert to actions
    fn handle_events(&mut self, event: Option<Event>) -> Action {
        match event {
            Some(Event::Key(key)) => self.handle_key_events(key),
            Some(Event::Mouse(mouse)) => self.handle_mouse_events(mouse),
            Some(Event::Tick) => Action::Tick,
            Some(Event::Render) => Action::Render,
            _ => Action::None,
        }
    }
    
    /// Handle keyboard events
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        Action::None
    }
    
    /// Handle mouse events
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Action {
        Action::None
    }
    
    /// Update component state based on action
    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    
    /// Render component to terminal
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Tick,
    Render,
    Quit,
    
    // Navigation
    NavigateToPatients,
    NavigateToAppointments,
    NavigateToClinical,
    NavigateToBilling,
    
    // Patient actions
    PatientSelect(Uuid),
    PatientCreate,
    PatientEdit(Uuid),
    PatientSearch(String),
    
    // ... more actions
}
```

### Example Component: Patient List

```rust
// src/components/patient/list.rs
pub struct PatientListComponent {
    // Services
    patient_service: Arc<PatientService>,
    
    // State
    patients: Vec<Patient>,
    selected_index: Option<usize>,
    search_query: String,
    is_loading: bool,
    error: Option<String>,
    
    // Pagination
    current_page: u32,
    total_pages: u32,
}

impl PatientListComponent {
    pub fn new(patient_service: Arc<PatientService>) -> Self {
        Self {
            patient_service,
            patients: vec![],
            selected_index: None,
            search_query: String::new(),
            is_loading: false,
            error: None,
            current_page: 1,
            total_pages: 1,
        }
    }
    
    async fn load_patients(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error = None;
        
        let query = PatientSearchQuery {
            name: if self.search_query.is_empty() {
                None
            } else {
                Some(self.search_query.clone())
            },
            page: self.current_page,
            page_size: 20,
            ..Default::default()
        };
        
        match self.patient_service.search_patients(query).await {
            Ok(result) => {
                self.patients = result.items;
                self.total_pages = (result.total as f64 / result.page_size as f64).ceil() as u32;
                self.is_loading = false;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load patients: {}", e));
                self.is_loading = false;
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Component for PatientListComponent {
    async fn init(&mut self) -> Result<()> {
        self.load_patients().await
    }
    
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                Action::Render
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                Action::Render
            }
            KeyCode::Enter => {
                if let Some(patient) = self.selected_patient() {
                    Action::PatientSelect(patient.id)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('n') => Action::PatientCreate,
            KeyCode::Char('/') => Action::StartSearch,
            _ => Action::None,
        }
    }
    
    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::PatientSearch(query) => {
                self.search_query = query;
                self.current_page = 1;
                self.load_patients().await?;
                Ok(Some(Action::Render))
            }
            Action::Tick => {
                // Periodic refresh if needed
                Ok(None)
            }
            _ => Ok(None),
        }
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Build patient table widget
        let rows: Vec<Row> = self.patients
            .iter()
            .map(|p| {
                Row::new(vec![
                    Cell::from(format!("{}, {}", p.last_name, p.first_name)),
                    Cell::from(p.date_of_birth.format("%d/%m/%Y").to_string()),
                    Cell::from(p.medicare_number.as_ref().map_or("", |s| s.as_str())),
                    Cell::from(p.phone_mobile.as_ref().map_or("", |s| s.as_str())),
                ])
            })
            .collect();
        
        let header = Row::new(vec!["Name", "DOB", "Medicare", "Phone"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        
        let table = Table::new(rows, [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(30),
        ])
        .header(header)
        .block(Block::bordered().title("Patients"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");
        
        frame.render_stateful_widget(table, area, &mut self.table_state);
        
        // Render loading/error overlays if needed
        if self.is_loading {
            self.render_loading(frame, area);
        } else if let Some(ref error) = self.error {
            self.render_error(frame, area, error);
        }
    }
}
```

---

## Data Architecture

### Database Schema Design

#### Core Principles

1. **Normalization**: 3NF minimum, denormalize only for proven performance needs
2. **Audit Columns**: Every table has `created_at`, `updated_at`, `created_by`, `updated_by`
3. **Soft Deletes**: Use `is_active` flag, never hard delete clinical data
4. **UUIDs**: Use UUIDs for primary keys (better for distributed systems, privacy)
5. **Encryption**: Sensitive columns encrypted at application level before storage

#### Schema Example: Patients Table

```sql
CREATE TABLE patients (
    -- Primary Key
    id BLOB PRIMARY KEY,  -- UUID as BLOB in SQLite
    
    -- Healthcare Identifiers
    ihi TEXT,  -- Individual Healthcare Identifier (16 digits)
    medicare_number TEXT,
    medicare_irn INTEGER CHECK(medicare_irn BETWEEN 1 AND 9),
    medicare_expiry DATE,
    
    -- Demographics
    title TEXT,
    first_name TEXT NOT NULL,
    middle_name TEXT,
    last_name TEXT NOT NULL,
    preferred_name TEXT,
    date_of_birth DATE NOT NULL,
    gender TEXT NOT NULL CHECK(gender IN ('Male', 'Female', 'Other', 'PreferNotToSay')),
    sex_at_birth TEXT,
    
    -- Contact Information
    address_line1 TEXT,
    address_line2 TEXT,
    suburb TEXT,
    state TEXT,
    postcode TEXT,
    country TEXT DEFAULT 'Australia',
    phone_home TEXT,
    phone_mobile TEXT,
    phone_work TEXT,
    email TEXT,
    
    -- Emergency Contact
    emergency_contact_name TEXT,
    emergency_contact_phone TEXT,
    emergency_contact_relationship TEXT,
    
    -- Additional Information
    concession_type TEXT,  -- 'DVA', 'Pensioner', 'Healthcare Card'
    concession_number TEXT,
    preferred_language TEXT DEFAULT 'English',
    interpreter_required BOOLEAN DEFAULT FALSE,
    aboriginal_torres_strait_islander TEXT,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_deceased BOOLEAN DEFAULT FALSE,
    deceased_date DATE,
    
    -- Audit
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB,  -- References users(id)
    updated_by BLOB,  -- References users(id)
    
    -- Indexes
    UNIQUE(medicare_number, medicare_irn)
);

-- Indexes for common queries
CREATE INDEX idx_patients_name ON patients(last_name, first_name);
CREATE INDEX idx_patients_dob ON patients(date_of_birth);
CREATE INDEX idx_patients_medicare ON patients(medicare_number);
CREATE INDEX idx_patients_active ON patients(is_active) WHERE is_active = TRUE;

-- Full-text search index (SQLite FTS5)
CREATE VIRTUAL TABLE patients_fts USING fts5(
    patient_id UNINDEXED,
    first_name,
    last_name,
    content=patients,
    content_rowid=rowid
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER patients_ai AFTER INSERT ON patients BEGIN
    INSERT INTO patients_fts(rowid, patient_id, first_name, last_name)
    VALUES (new.rowid, new.id, new.first_name, new.last_name);
END;

CREATE TRIGGER patients_ad AFTER DELETE ON patients BEGIN
    DELETE FROM patients_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER patients_au AFTER UPDATE ON patients BEGIN
    UPDATE patients_fts 
    SET first_name = new.first_name, last_name = new.last_name
    WHERE rowid = new.rowid;
END;
```

### Migration Strategy

Using `sqlx-cli` for database migrations:

```bash
# Create new migration
sqlx migrate add create_patients_table

# Apply migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

Migration file example:

```sql
-- migrations/001_create_patients.sql
-- migrate:up
CREATE TABLE patients (
    -- ... schema definition
);

-- migrate:down
DROP TABLE IF EXISTS patients;
```

---

## Security Architecture

### Defense in Depth

Security implemented at multiple layers:

```
Layer 1: Network/OS     → Firewall, TLS, OS hardening
Layer 2: Application    → Authentication, authorization
Layer 3: Business Logic → Input validation, business rules
Layer 4: Data Access    → SQL injection prevention, parameterized queries
Layer 5: Storage        → Encryption at rest, backups
Layer 6: Audit          → Comprehensive logging, monitoring
```

### Authentication Flow

```
1. User enters credentials
2. AuthService.authenticate(username, password)
3. Hash password with Argon2
4. Compare with stored hash
5. Check account status (active, not locked)
6. Create session with expiry
7. Log authentication event
8. Return session token to client
9. Store session in memory (or Redis for distributed)
10. Include session token in subsequent requests
```

### Authorization Model (RBAC)

```rust
// src/infrastructure/auth/rbac.rs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Doctor,
    Nurse,
    Receptionist,
    Billing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Permission {
    // Patient permissions
    PatientRead,
    PatientCreate,
    PatientUpdate,
    PatientDelete,
    PatientSearch,
    PatientExport,
    
    // Clinical permissions
    ClinicalRead,
    ClinicalWrite,
    ClinicalSign,
    ClinicalDelete,
    
    // Prescription permissions
    PrescriptionCreate,
    PrescriptionCancel,
    PrescriptionAuthority,
    
    // Billing permissions
    BillingRead,
    BillingCreate,
    BillingProcess,
    
    // Admin permissions
    UserManage,
    SystemConfig,
    AuditView,
}

impl Role {
    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;
        
        match self {
            Role::Admin => &[
                // All permissions
                PatientRead, PatientCreate, PatientUpdate, PatientDelete,
                ClinicalRead, ClinicalWrite, ClinicalSign, ClinicalDelete,
                PrescriptionCreate, PrescriptionCancel, PrescriptionAuthority,
                BillingRead, BillingCreate, BillingProcess,
                UserManage, SystemConfig, AuditView,
            ],
            Role::Doctor => &[
                PatientRead, PatientCreate, PatientUpdate,
                ClinicalRead, ClinicalWrite, ClinicalSign,
                PrescriptionCreate, PrescriptionCancel, PrescriptionAuthority,
                BillingRead,
            ],
            Role::Nurse => &[
                PatientRead, PatientUpdate,
                ClinicalRead, ClinicalWrite,
                PrescriptionCreate,  // Limited prescribing
            ],
            Role::Receptionist => &[
                PatientRead, PatientCreate, PatientUpdate, PatientSearch,
                BillingRead, BillingCreate,
            ],
            Role::Billing => &[
                PatientRead,
                BillingRead, BillingCreate, BillingProcess,
            ],
        }
    }
}

pub struct User {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    pub permissions: Vec<Permission>,  // Can override role permissions
}

impl User {
    pub fn has_permission(&self, permission: Permission) -> bool {
        // Check role permissions
        if self.role.permissions().contains(&permission) {
            return true;
        }
        
        // Check user-specific permissions
        self.permissions.contains(&permission)
    }
    
    pub fn can_access_patient(&self, patient_id: Uuid) -> bool {
        // Check general read permission
        if !self.has_permission(Permission::PatientRead) {
            return false;
        }
        
        // Could add additional checks here:
        // - Is patient assigned to this practitioner?
        // - Is patient in this clinic?
        // - etc.
        
        true
    }
}
```

### Encryption Implementation

**Fields requiring encryption**:
- Clinical notes (SOAP notes, confidential notes)
- Prescription details
- Social history information
- Any PII marked as sensitive

**Encryption approach**:
1. Application-level encryption (transparent to database)
2. Encrypt before insert/update
3. Decrypt after select
4. Key stored in environment variable (or KMS in production)

```rust
// Usage in repository
impl SqlxPatientRepository {
    async fn create_consultation(&self, consultation: Consultation) -> Result<()> {
        // Encrypt sensitive fields
        let encrypted_subjective = self.crypto.encrypt(&consultation.subjective)?;
        let encrypted_assessment = self.crypto.encrypt(&consultation.assessment)?;
        let encrypted_plan = self.crypto.encrypt(&consultation.plan)?;
        
        sqlx::query!(
            "INSERT INTO consultations (..., subjective, assessment, plan, ...) 
             VALUES (..., ?, ?, ?, ...)",
            encrypted_subjective,
            encrypted_assessment,
            encrypted_plan
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

---

## Integration Architecture

### External System Integration Pattern

All external integrations follow a consistent pattern:

```rust
// src/integrations/mod.rs
pub mod medicare;     // Medicare Online
pub mod pbs;          // PBS API
pub mod hi_service;   // Healthcare Identifiers
pub mod mhr;          // My Health Record
pub mod pathology;    // Pathology labs

// Common integration pattern
#[async_trait]
pub trait ExternalService: Send + Sync {
    async fn health_check(&self) -> Result<HealthStatus, IntegrationError>;
    fn service_name(&self) -> &str;
}

pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: u64,
    pub last_check: DateTime<Utc>,
}
```

### Medicare Online Integration

```rust
// src/integrations/medicare/client.rs
pub struct MedicareClient {
    http_client: reqwest::Client,
    base_url: String,
    proda_auth: Arc<ProdaAuthService>,
}

impl MedicareClient {
    pub async fn submit_claim(&self, claim: MedicareClaim) -> Result<ClaimResponse, MedicareError> {
        // 1. Get PRODA token
        let token = self.proda_auth.get_token().await?;
        
        // 2. Build request
        let request = self.http_client
            .post(format!("{}/claims", self.base_url))
            .bearer_auth(token)
            .json(&claim)
            .build()?;
        
        // 3. Send with retry logic
        let response = self.send_with_retry(request, 3).await?;
        
        // 4. Parse response
        let claim_response: ClaimResponse = response.json().await?;
        
        // 5. Audit log
        info!("Medicare claim submitted: claim_id={}", claim_response.claim_id);
        
        Ok(claim_response)
    }
    
    async fn send_with_retry(
        &self,
        request: reqwest::Request,
        max_retries: u32,
    ) -> Result<reqwest::Response, MedicareError> {
        // Retry logic with exponential backoff
        for attempt in 0..max_retries {
            match self.http_client.execute(request.try_clone().unwrap()).await {
                Ok(response) if response.status().is_success() => return Ok(response),
                Ok(response) => {
                    error!("Medicare API error: status={}", response.status());
                    if attempt < max_retries - 1 {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                        continue;
                    }
                    return Err(MedicareError::ApiError(response.status().as_u16()));
                }
                Err(e) => {
                    error!("Medicare API request failed: {}", e);
                    if attempt < max_retries - 1 {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                        continue;
                    }
                    return Err(MedicareError::NetworkError(e));
                }
            }
        }
        
        Err(MedicareError::MaxRetriesExceeded)
    }
}
```

---

## Development Patterns

### Error Handling Strategy

**No panics in production code.** Use `Result<T, E>` everywhere:

```rust
// Domain-specific error types
#[derive(Debug, thiserror::Error)]
pub enum PatientError {
    #[error("Patient not found: {0}")]
    NotFound(Uuid),
    
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    
    #[error("Duplicate patient: Medicare number already exists")]
    Duplicate,
}

// Convert to generic application error at boundaries
impl From<PatientError> for AppError {
    fn from(err: PatientError) -> Self {
        match err {
            PatientError::NotFound(_) => AppError::NotFound,
            PatientError::Validation(_) => AppError::BadRequest(err.to_string()),
            _ => AppError::Internal(err.to_string()),
        }
    }
}
```

### Logging Strategy

Use `tracing` for structured logging:

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self), fields(patient_id = %id))]
pub async fn find_patient(&self, id: Uuid) -> Result<Option<Patient>> {
    debug!("Finding patient by ID");
    
    let patient = self.repository.find_by_id(id).await?;
    
    if patient.is_some() {
        info!("Patient found");
    } else {
        warn!("Patient not found");
    }
    
    Ok(patient)
}
```

Log levels:
- `error!`: Actionable errors requiring attention
- `warn!`: Concerning but not immediately actionable
- `info!`: Important business events
- `debug!`: Detailed diagnostic information
- `trace!`: Very verbose, usually disabled

### Configuration Management

```rust
// src/config.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub integrations: IntegrationConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        // Load from multiple sources (precedence: env vars > config file > defaults)
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::Environment::with_prefix("OPENGP"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

---

## Testing Strategy

### Test Pyramid

```
        /\
       /  \     E2E Tests (5%)
      /----\    
     /      \   Integration Tests (20%)
    /--------\  
   /          \ Unit Tests (75%)
  /____________\
```

### Unit Tests

Test domain logic in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_patient_creation_with_valid_data() {
        let data = NewPatientData {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            gender: Gender::Male,
        };
        
        let patient = Patient::new(data).unwrap();
        
        assert_eq!(patient.first_name, "John");
        assert_eq!(patient.last_name, "Smith");
        assert_eq!(patient.age() > 40, true);
    }
    
    #[test]
    fn test_patient_creation_with_invalid_name() {
        let data = NewPatientData {
            first_name: "".to_string(),  // Invalid
            last_name: "Smith".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            gender: Gender::Male,
        };
        
        let result = Patient::new(data);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyName));
    }
}
```

### Integration Tests

Test repository implementations with real database:

```rust
// tests/integration/patient_repository_test.rs
use sqlx::SqlitePool;
use opengp::infrastructure::database::SqlxPatientRepository;
use opengp::domain::patient::*;

#[tokio::test]
async fn test_create_and_find_patient() {
    // Setup in-memory database
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    let repo = SqlxPatientRepository::new(pool);
    
    // Create patient
    let patient = Patient::new(NewPatientData {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
        gender: Gender::Male,
    }).unwrap();
    
    let saved = repo.create(patient.clone()).await.unwrap();
    
    // Find patient
    let found = repo.find_by_id(saved.id).await.unwrap();
    
    assert!(found.is_some());
    assert_eq!(found.unwrap().first_name, "John");
}
```

### TUI Testing

Use `TestBackend` for TUI testing:

```rust
#[test]
fn test_patient_list_rendering() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut component = PatientListComponent::new(mock_service());
    component.patients = vec![
        Patient { first_name: "John".to_string(), last_name: "Smith".to_string(), ... },
        Patient { first_name: "Jane".to_string(), last_name: "Doe".to_string(), ... },
    ];
    
    terminal.draw(|f| {
        component.render(f, f.size());
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    
    // Assert that patient names appear in buffer
    assert!(buffer.content().contains("John"));
    assert!(buffer.content().contains("Smith"));
}
```

---

## Deployment Architecture

### Development Environment

```bash
# Local development with SQLite
export DATABASE_URL="sqlite:./data/opengp.db"
export ENCRYPTION_KEY="<generated_key>"
export RUST_LOG="opengp=debug"

cargo run
```

### Production Environment

```
┌─────────────────────────────────────────┐
│         Load Balancer (nginx)           │
└────────────────┬────────────────────────┘
                 │
         ┌───────┴───────┐
         │               │
    ┌────▼────┐    ┌────▼────┐
    │ OpenGP  │    │ OpenGP  │
    │Instance1│    │Instance2│
    └────┬────┘    └────┬────┘
         │               │
         └───────┬───────┘
                 │
        ┌────────▼────────┐
        │   PostgreSQL    │
        │   (Primary)     │
        └────────┬────────┘
                 │
        ┌────────▼────────┐
        │   PostgreSQL    │
        │   (Replica)     │
        └─────────────────┘
```

### Configuration Files

```toml
# config/production.toml
[database]
url = "${DATABASE_URL}"
max_connections = 20
min_connections = 5

[auth]
session_timeout_minutes = 15
password_min_length = 12

[integrations]
medicare_api_url = "https://api.servicesaustralia.gov.au/medicare"
pbs_api_url = "https://data.pbs.gov.au/api"
```

---

## Performance Considerations

### Database Query Optimization

1. **Use indexes** for all foreign keys and frequently queried columns
2. **Batch operations** where possible
3. **Connection pooling** with appropriate limits
4. **Query analysis** with `EXPLAIN QUERY PLAN`

### UI Performance

1. **Lazy loading**: Load data on-demand, not all at once
2. **Pagination**: Never load all records into memory
3. **Debouncing**: Debounce search inputs to reduce queries
4. **Virtual scrolling**: For very large lists (future optimization)

### Async Performance

1. **Concurrent requests**: Use `tokio::join!` for parallel operations
2. **Timeout all external calls**: Never wait forever
3. **Circuit breaker**: Prevent cascading failures from external services

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-11 | Use SQLx over Diesel | Compile-time query validation, better async support, multi-DB |
| 2026-02-11 | Use Ratatui over cursive | More active development, better documentation |
| 2026-02-11 | Use AGPL-3.0 license | Ensures modifications stay open source |
| 2026-02-11 | Component-based architecture | Modularity, testability, maintainability |
| 2026-02-11 | Repository pattern for data access | Abstracts database, enables testing, supports DB migration |
| 2026-02-11 | Application-level encryption | Defense in depth, works with any database |
| 2026-02-11 | Trait-based dependency injection | Testability, flexibility, follows Rust idioms |

---

**Document Maintenance**: This document should be updated as architectural decisions are made or patterns evolve. All significant architectural changes should be logged in the Decision Log section.

**Contributors**: OpenGP Architecture Team  
**Review Cycle**: Quarterly or with major releases
