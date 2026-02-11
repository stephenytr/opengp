# AGENTS.md - OpenGP Development Guide for AI Coding Agents

**Project**: OpenGP - Open Source General Practice Management Software  
**Language**: Rust (Edition 2021)  
**Framework**: Ratatui (TUI), SQLx (Database), Tokio (Async Runtime)  
**Target**: Australian healthcare providers  
**License**: AGPL-3.0

---

## Quick Reference

**Build**: `cargo build`  
**Run**: `cargo run`  
**Test All**: `cargo test`  
**Test Single**: `cargo test test_name`  
**Test Module**: `cargo test module_name::`  
**Lint**: `cargo clippy -- -D warnings`  
**Format**: `cargo fmt`  
**Type Check**: `cargo check`

---

## Build & Test Commands

### Development Workflow

```bash
# Check code compiles (fast feedback)
cargo check

# Run all tests
cargo test

# Run specific test
cargo test test_patient_creation

# Run tests in specific module
cargo test domain::patient::

# Run tests with output
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture

# Lint (must pass - no warnings allowed)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Database Commands

```bash
# Install sqlx-cli
cargo install sqlx-cli --features sqlite

# Create new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Feature Flags

```bash
# Build with SQLite (default)
cargo build

# Build with PostgreSQL
cargo build --features postgres --no-default-features

# Test with specific database
cargo test --features postgres --no-default-features
```

---

## Project Structure

```
src/
├── main.rs                  # Entry point
├── lib.rs                   # Library exports
├── app.rs                   # Main App struct (orchestration)
├── config.rs                # Configuration management
├── error.rs                 # Top-level error types
├── ui/                      # UI Layer (Ratatui)
│   ├── tui.rs              # Terminal setup
│   ├── event.rs            # Event handling
│   ├── theme.rs            # Styling
│   └── widgets/            # Custom widgets
├── components/              # UI Components (trait-based)
│   ├── patient/            # Patient UI components
│   ├── appointment/        # Appointment UI components
│   └── clinical/           # Clinical UI components
├── domain/                  # Domain Layer (business logic)
│   ├── patient/            # Patient domain
│   │   ├── model.rs        # Domain entities
│   │   ├── service.rs      # Business logic
│   │   ├── repository.rs   # Persistence interface (trait)
│   │   ├── dto.rs          # Data transfer objects
│   │   └── error.rs        # Domain errors
│   ├── appointment/
│   ├── clinical/
│   └── billing/
├── infrastructure/          # Infrastructure Layer
│   ├── database/           # Database implementation
│   │   └── repositories/   # Repository implementations
│   ├── crypto/             # Encryption/hashing
│   ├── audit/              # Audit logging
│   └── auth/               # Authentication
└── integrations/            # External APIs
    ├── medicare/           # Medicare Online
    ├── pbs/                # PBS API
    ├── air/                # Immunisation Register
    └── hi_service/         # Healthcare Identifiers
```

**Layer Dependency Rule**: Outer → Inner only. Domain layer NEVER depends on infrastructure or UI.

---

## Code Style Guidelines

### Naming Conventions

```rust
// ✅ Types: PascalCase
pub struct PatientService { }
pub enum Gender { Male, Female }
pub trait PatientRepository { }

// ✅ Functions: snake_case
pub async fn register_patient() { }
pub fn calculate_age() -> u32 { }

// ✅ Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const SESSION_TIMEOUT: Duration = Duration::from_secs(900);

// ✅ Variables: snake_case
let patient_id = Uuid::new_v4();
let medicare_number = "1234567890";
```

### Import Organization

**Order**: std → external crates → internal modules → parent modules

```rust
// ✅ Good - organized imports
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::patient::{Patient, PatientRepository};
use crate::infrastructure::audit::AuditLogger;
use super::dto::NewPatientData;

// ❌ Bad - disorganized
use crate::domain::patient::Patient;
use uuid::Uuid;
use super::dto::NewPatientData;
use std::sync::Arc;
```

### Error Handling

**MANDATORY**: Never use `.unwrap()` or `.expect()` in production code. Always propagate errors.

```rust
// ✅ Good - explicit error handling
pub async fn find_patient(&self, id: Uuid) -> Result<Option<Patient>, ServiceError> {
    let patient = self.repository.find_by_id(id).await?;
    Ok(patient)
}

// ✅ Good - custom error types with thiserror
#[derive(Debug, thiserror::Error)]
pub enum PatientError {
    #[error("Patient not found: {0}")]
    NotFound(Uuid),
    
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

// ❌ Bad - unwrap in production code
let patient = self.repository.find_by_id(id).await.unwrap();  // NEVER DO THIS

// ❌ Bad - panic
if patient.is_none() { panic!("Patient not found"); }  // NEVER DO THIS
```

### Async Patterns

**Always use `async_trait` for async trait methods**:

```rust
// ✅ Good - async trait with async_trait macro
use async_trait::async_trait;

#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>>;
    async fn create(&self, patient: Patient) -> Result<Patient>;
}

// ✅ Good - Arc for shared state in async
pub struct PatientService {
    repository: Arc<dyn PatientRepository>,
    audit_logger: Arc<AuditLogger>,
}
```

### Logging with Tracing

**Use structured logging, not println!**

```rust
// ✅ Good - structured logging with tracing
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

// ❌ Bad - println! in production code
println!("Finding patient {}", id);  // NEVER DO THIS
```

**Log Levels**:
- `error!`: Actionable errors requiring attention
- `warn!`: Concerning but not immediately critical
- `info!`: Important business events (patient created, claim submitted)
- `debug!`: Detailed diagnostic information
- `trace!`: Very verbose (usually disabled)

### Type Annotations

**Prefer explicit types for public APIs, inference for local variables**:

```rust
// ✅ Good - explicit return types on public functions
pub async fn register_patient(
    &self,
    data: NewPatientData,
    user: &User,
) -> Result<Patient, ServiceError> {
    // ✅ Good - inference for local variables
    let patient = Patient::new(data)?;
    let saved = self.repository.create(patient).await?;
    Ok(saved)
}

// ❌ Bad - unclear return type
pub async fn register_patient(&self, data: NewPatientData, user: &User) {
    // Unclear what this returns
}
```

### Module Organization

**Every domain module follows this pattern**:

```rust
// src/domain/{module}/mod.rs
mod model;         // Domain entities
mod service;       // Business logic
mod repository;    // Persistence trait
mod dto;           // Data transfer objects
mod error;         // Domain-specific errors

pub use model::*;
pub use service::*;
pub use repository::*;
pub use dto::*;
pub use error::*;
```

---

## Testing Conventions

### Unit Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_patient_creation_with_valid_data() {
        // Arrange
        let data = NewPatientData {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            gender: Gender::Male,
        };
        
        // Act
        let patient = Patient::new(data).unwrap();
        
        // Assert
        assert_eq!(patient.first_name, "John");
        assert_eq!(patient.last_name, "Smith");
        assert!(patient.age() > 40);
    }
    
    #[test]
    fn test_patient_creation_with_invalid_name() {
        // Arrange
        let data = NewPatientData {
            first_name: "".to_string(),  // Invalid
            last_name: "Smith".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            gender: Gender::Male,
        };
        
        // Act
        let result = Patient::new(data);
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyName));
    }
}
```

### Integration Test Structure

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
    let patient = create_test_patient();
    let saved = repo.create(patient.clone()).await.unwrap();
    
    // Find patient
    let found = repo.find_by_id(saved.id).await.unwrap();
    
    assert!(found.is_some());
    assert_eq!(found.unwrap().first_name, "John");
}

fn create_test_patient() -> Patient {
    Patient::new(NewPatientData {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
        gender: Gender::Male,
    }).unwrap()
}
```

---

## Australian Healthcare Compliance

### Critical Security Requirements

**NO EXCEPTIONS** - These are legal requirements:

1. **Never log sensitive health data** (PII, clinical notes, Medicare numbers)
2. **Encrypt all clinical notes** before storing in database
3. **Audit log all patient record access** (read, write, delete)
4. **Use secure random for tokens/IDs** (`rand::thread_rng()`)
5. **Validate all user input** (SQL injection, XSS prevention)
6. **Never hard delete clinical data** (soft delete with `is_active = false`)

### Encryption Pattern

```rust
// ✅ Always encrypt sensitive fields before storage
let encrypted_notes = self.crypto.encrypt(&consultation.subjective)?;

sqlx::query!(
    "INSERT INTO consultations (id, subjective) VALUES (?, ?)",
    id,
    encrypted_notes
)
.execute(&self.pool)
.await?;
```

### Audit Logging Pattern

```rust
// ✅ Log all patient data access
self.audit_logger.log(AuditEvent {
    user_id: user.id,
    action: AuditAction::PatientRead,
    entity_id: patient.id,
    timestamp: Utc::now(),
}).await?;
```

---

## Git Workflow

### Commit Message Format

Follow Conventional Commits:

```
feat: add patient search functionality
fix: resolve Medicare number validation bug
docs: update ARCHITECTURE.md with HL7 parsing
chore: update dependencies
refactor: extract validation logic to separate module
test: add integration tests for AIR client
```

### Pre-Commit Checklist

- [ ] `cargo fmt` - Code is formatted
- [ ] `cargo clippy -- -D warnings` - No clippy warnings
- [ ] `cargo test` - All tests pass
- [ ] No sensitive data (API keys, passwords, patient data)
- [ ] Audit logs tested for new features

---

## Boundaries & Rules

### ✅ ALWAYS DO

- Use `Result<T, E>` for fallible operations
- Use `async_trait` for async trait methods
- Use `Arc<dyn Trait>` for shared dependencies
- Add `#[instrument]` to functions that need tracing
- Write unit tests for domain logic
- Write integration tests for repository implementations
- Use structured logging (`tracing::info!`, not `println!`)
- Validate user input at domain layer
- Encrypt sensitive data before storage
- Audit log patient data access
- Use UUIDs for primary keys
- Soft delete clinical data (never hard delete)

### ⚠️ ASK FIRST

- Adding new dependencies to Cargo.toml
- Changing database schema (requires migration)
- Modifying security/encryption logic
- Adding new external integrations
- Changing API contracts (breaking changes)

### 🚫 NEVER DO

- Use `.unwrap()` or `.expect()` in production code (use `?` operator)
- Use `panic!` in production code
- Use `println!` for logging (use `tracing` macros)
- Log sensitive patient data (PII, clinical notes, Medicare numbers)
- Hard delete clinical records (use soft delete)
- Commit secrets, API keys, or patient data
- Ignore clippy warnings (must fix all)
- Skip writing tests for new features
- Use `unsafe` without explicit justification
- Store encryption keys in code or git

---

## Code Examples

### Domain Model Pattern

```rust
// src/domain/patient/model.rs
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub medicare_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Patient {
    pub fn new(data: NewPatientData) -> Result<Self, ValidationError> {
        Self::validate_names(&data.first_name, &data.last_name)?;
        Self::validate_date_of_birth(data.date_of_birth)?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            first_name: data.first_name,
            last_name: data.last_name,
            date_of_birth: data.date_of_birth,
            medicare_number: data.medicare_number,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    pub fn age(&self) -> u32 {
        let today = Utc::now().date_naive();
        today.years_since(self.date_of_birth).unwrap_or(0)
    }
}
```

### Service Layer Pattern

```rust
// src/domain/patient/service.rs
use async_trait::async_trait;
use std::sync::Arc;

pub struct PatientService {
    repository: Arc<dyn PatientRepository>,
    audit_logger: Arc<AuditLogger>,
}

impl PatientService {
    pub async fn register_patient(
        &self,
        data: NewPatientData,
        user: &User,
    ) -> Result<Patient, ServiceError> {
        // 1. Check for duplicates
        if let Some(ref medicare) = data.medicare_number {
            if self.repository.find_by_medicare(medicare).await?.is_some() {
                return Err(ServiceError::DuplicatePatient);
            }
        }
        
        // 2. Create patient (validation happens in domain model)
        let patient = Patient::new(data)?;
        
        // 3. Save to database
        let saved = self.repository.create(patient).await?;
        
        // 4. Audit log
        self.audit_logger.log(AuditEvent {
            user_id: user.id,
            action: AuditAction::PatientCreate,
            entity_id: saved.id,
            timestamp: Utc::now(),
        }).await?;
        
        Ok(saved)
    }
}
```

### Repository Trait Pattern

```rust
// src/domain/patient/repository.rs
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError>;
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError>;
    async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError>;
}
```

### Component Trait Pattern

```rust
// src/components/mod.rs
use async_trait::async_trait;

#[async_trait]
pub trait Component: Send {
    async fn init(&mut self) -> Result<()> { Ok(()) }
    fn handle_key_events(&mut self, key: KeyEvent) -> Action;
    async fn update(&mut self, action: Action) -> Result<Option<Action>>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
```

---

## Documentation Standards

### Function Documentation

```rust
/// Register a new patient in the system
///
/// # Arguments
/// * `data` - Patient registration data
/// * `user` - User performing the registration
///
/// # Returns
/// * `Ok(Patient)` - Successfully registered patient
/// * `Err(ServiceError::DuplicatePatient)` - Medicare number already exists
/// * `Err(ServiceError::Validation)` - Invalid patient data
///
/// # Example
/// ```
/// let patient = service.register_patient(data, &current_user).await?;
/// ```
pub async fn register_patient(
    &self,
    data: NewPatientData,
    user: &User,
) -> Result<Patient, ServiceError> {
    // Implementation
}
```

### Module Documentation

```rust
//! Patient domain module
//!
//! This module contains the patient entity, business logic, and repository interface.
//! It handles patient registration, updates, and search operations.

mod model;
mod service;
mod repository;
```

---

## Critical Healthcare Context

**This is medical software - patient safety is paramount.**

### Validation Requirements

All patient data MUST be validated:
- Names: Non-empty, reasonable length
- Date of birth: Not in future, reasonable age range
- Medicare number: 10 digits, valid checksum
- Phone numbers: Valid Australian format
- Email: RFC 5322 compliant

### Clinical Data Integrity

- SOAP notes cannot be deleted once signed
- Prescriptions cannot be edited once created (only cancelled)
- Allergy records must trigger alerts in prescribing
- Immunisation records must be reported to AIR within 24 hours

### Performance Requirements

- Patient search: <500ms for <10k patients
- Appointment calendar: <200ms render time
- Database queries: Use indexes for all foreign keys
- HL7 message processing: <1s per message

---

## Common Patterns

### SQLx Query Pattern

```rust
// ✅ Use query_as! for type-safe queries
let patient = sqlx::query_as!(
    PatientRow,
    "SELECT * FROM patients WHERE id = ? AND is_active = TRUE",
    id
)
.fetch_optional(&self.pool)
.await?;

// ✅ Use query! for INSERT/UPDATE
sqlx::query!(
    "INSERT INTO patients (id, first_name, last_name) VALUES (?, ?, ?)",
    patient.id,
    patient.first_name,
    patient.last_name
)
.execute(&self.pool)
.await?;
```

### Arc Pattern for Shared State

```rust
// ✅ Use Arc for shared dependencies
pub struct PatientService {
    repository: Arc<dyn PatientRepository>,  // Trait object
    audit_logger: Arc<AuditLogger>,          // Concrete type
    crypto: Arc<EncryptionService>,
}
```

---

## References

- **Architecture**: See ARCHITECTURE.md for detailed patterns
- **Requirements**: See REQUIREMENTS.md for features and compliance
- **Rust Style**: Follow official Rust style guide
- **Clippy**: All clippy warnings must be fixed
- **Healthcare Compliance**: Privacy Act 1988, My Health Records Act 2012

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-11  
**Maintainer**: OpenGP Development Team
