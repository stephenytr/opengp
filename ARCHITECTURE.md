# OpenGP Architecture Documentation

**Version**: 1.4  
**Last Updated**: 2026-02-17  
**Status**: Living Document

---

## Table of Contents

1. [Overview](#overview)
2. [Architectural Principles](#architectural-principles)
3. [System Architecture](#system-architecture)
4. [TUI Architecture](#tui-architecture)
5. [Layer Architecture](#layer-architecture)
6. [Module Structure](#module-structure)
7. [Component Architecture](#component-architecture)
8. [Data Architecture](#data-architecture)
9. [Security Architecture](#security-architecture)
10. [Integration Architecture](#integration-architecture)
11. [Development Patterns](#development-patterns)
12. [Testing Strategy](#testing-strategy)
13. [Deployment Architecture](#deployment-architecture)
14. [Performance Considerations](#performance-considerations)
15. [Decision Log](#decision-log)

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
| **UI Framework** | Ratatui + Custom Wrappers | Terminal-based, cross-platform, resource-efficient |
| **UI Integration** | tui-realm (integrated) | adapters for form inputs, lists, and selects |
| **Async Runtime** | Tokio | Industry standard, excellent ecosystem |
| **Database** | SQLx | Compile-time query validation, multi-DB support |
| **Architecture Style** | Layered + Domain-Driven Design + Trait Abstractions | Clear boundaries, business logic isolation, dependency injection |

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

### Abstraction Patterns

The codebase uses **trait-based abstractions** throughout for loose coupling and testability:

#### Repository Pattern (Domain Layer)

```rust
// src/domain/patient/repository.rs
#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>>;
    async fn create(&self, patient: Patient) -> Result<Patient>;
    async fn update(&self, patient: Patient) -> Result<Patient>;
    // ... more operations
}
```

#### Service Layer (Dependency Injection)

```rust
// src/domain/patient/service.rs
pub struct PatientService {
    repository: Arc<dyn PatientRepository>,     // Abstract repository
    audit_logger: Arc<dyn AuditLogger>,        // Abstract logger
}

// Concrete implementations injected at runtime:
let service = PatientService::new(
    Arc::new(SqlitePatientRepository::new(pool)),  // Or MockPatientRepository for tests
    Arc::new(AuditLogger::new(repo)),
);
```

#### Query Abstraction (Read Models)

```rust
// src/domain/appointment/query.rs
/// Separate read model for optimized calendar queries
#[async_trait]
pub trait AppointmentCalendarQuery: Send + Sync {
    async fn for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<CalendarAppointment>>;
}
```

#### UI Component Abstraction

```rust
// src/components/mod.rs - Component trait
#[async_trait]
pub trait Component: Send {
    async fn init(&mut self) -> Result<()>;
    fn handle_events(&mut self, event: Option<Event>) -> Action;
    async fn update(&mut self, action: Action) -> Result<Option<Action>>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

// Components are stored as trait objects
struct App {
    patient_component: Box<dyn Component>,
    appointment_component: Box<dyn Component>,
    // Or with Arc for shared ownership
    clinical_component: Arc<dyn Component>,
}
```

#### UI Component Traits

```rust
// src/ui/components/traits.rs - Additional UI abstractions
pub trait InteractiveComponent {
    fn get_state(&self) -> ComponentState;
    fn is_focused(&self) -> bool;
    fn set_focus(&mut self, focused: bool);
}

pub trait Renderable {
    fn render(&mut self, area: Rect, frame: &mut Frame);
}
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
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │Prescription│ │Immunisatn│ │Pathology │ │  Referral │ │   Audit  │ │
│  │  Module   │ │  Module   │ │  Module  │ │  Module   │ │  Module  │ │
│  └────┬─────┘ └─────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ │
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
│  │  • ClinicalRepository   • UserRepository                        │ │
│  │  • AuditRepository      • PractitionerRepository                │ │
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

## TUI Architecture

The terminal user interface lives in the `opengp-ui` crate. It runs as a thin client that talks to the backend over HTTP instead of calling domain services directly.

### Client server split

- The TUI uses an `ApiClient` to send HTTP requests to the `opengp-api` backend.
- All domain services such as `PatientService`, `ClinicalService`, and similar types run on the server side inside `opengp-api`.
- The TUI never holds `Arc<dyn ...Service>` instances and never calls service methods inside the terminal process.

### Relationship to opengp-domain

- `opengp-ui` depends on `opengp-domain` only for shared types, for example models, DTOs, and enums that are used in request and response payloads.
- Business rules, validation, and orchestration in the domain layer are executed on the server when the API handles a request.

This separation is intentional. The TUI is treated as an external client, even though it ships in the same repository as the backend.

### Validation and enforcement

Because the TUI bypasses domain services locally and goes through HTTP instead, domain validation only applies when the API enforces it. If the API handler does not call the appropriate domain service or validation method, the TUI will not get those guarantees.

## Layer Architecture

### 1. UI Layer (Ratatui + Custom Wrappers)

**Responsibility**: Rendering terminal UI and handling user input.

```rust
// src/ui/mod.rs
pub mod tui;           // Terminal setup and management
pub mod event;         // Event handling and key mappings
pub mod theme;         // Color schemes and styling
pub mod widgets;       // Custom reusable widgets
pub mod components;    // Reusable UI components with traits
pub mod keybinds;      // Keyboard binding definitions
pub mod app;           // UI application wrapper
pub mod msg;           // Message types for component communication
pub mod component_id;  // Component identifier definitions
```

**Key Patterns**:
- **Immediate Mode Rendering**: UI rebuilt every frame
- **Event-Driven**: Async event handling via Tokio channels
- **Component Traits**: `InteractiveComponent`, `Renderable`, `KeyboardInput` for abstractions
- **Custom Wrappers**: `InputWrapper`, `SelectWrapper`, `ListPicker` for form elements
- **tui-realm Integration**: `RealmInput`, `RealmList`, `RealmSelect` adapters for tui-realm

**Reusable UI Components** (`src/ui/components/`):

```rust
// Basic widgets
pub mod buttons;       // Button widgets
pub mod checkboxes;    // Checkbox widgets
pub mod inputs;        // Input field widgets
pub mod selects;       // Select dropdown widgets
pub mod list_picker;   // List picker with fuzzy filtering
pub mod modal;         // Modal dialogs
pub mod tab_view;      // Tab navigation
pub mod state;         // Component state management
pub mod traits;        // Component traits (InteractiveComponent, Renderable)

// tui-realm adapters
pub mod realm_input;   // tui-realm input adapter
pub mod realm_list;    // tui-realm list adapter
pub mod realm_select;  // tui-realm select adapter
pub mod theme_adapter; // Theme adapter for tui-realm
```

**Custom UI Component Wrappers**:

```rust
// src/ui/components/inputs.rs
pub struct InputWrapper {
    value: String,
    placeholder: String,
    is_focused: bool,
    // ... manages input state
}

// src/ui/components/selects.rs  
pub struct SelectWrapper<T> {
    options: Vec<T>,
    selected_index: usize,
    is_open: bool,
    // ... manages selection state
}

// src/ui/components/list_picker.rs
pub struct ListPicker<T> {
    items: Vec<T>,
    filter_query: String,
    selected_index: Option<usize>,
    // ... manages list navigation and filtering
}
```

**Widget Architecture** (`src/ui/widgets/`):
```rust
// src/ui/widgets/list_selector.rs
pub struct ListSelector<T> { ... }

// src/ui/widgets/search_filter.rs
pub struct SearchFilter<T> { ... }

// src/ui/widgets/modal_handler.rs
pub struct ModalHandler { ... }

// src/ui/widgets/confirmation_dialog.rs
pub struct ConfirmationDialog { ... }

// src/ui/widgets/form_field.rs
pub struct FormField { ... }

// src/ui/widgets/status_badge.rs
pub struct StatusBadge { ... }

// src/ui/widgets/help_modal.rs
pub struct HelpModal { ... }

// src/ui/widgets/month_calendar.rs
pub struct MonthCalendar { ... }

// src/ui/widgets/time_slot_picker.rs
pub struct TimeSlotPicker { ... }

// src/ui/widgets/mouse_debug.rs
pub struct MouseDebug { ... }
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

Every major UI feature implements the `Component` trait:

```rust
// src/components/mod.rs
use async_trait::async_trait;
use ratatui::layout::Rect;
use ratatui::Frame;
use crossterm::event::{KeyEvent, MouseEvent};

/// Component trait for modular UI components
/// 
/// This trait provides a common interface for all UI components,
/// enabling loose coupling and testability through dependency injection.
#[async_trait]
pub trait Component: Send {
    /// Initialize component (called once)
    async fn init(&mut self) -> crate::error::Result<()> {
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
    fn handle_key_events(&mut self, key: KeyEvent) -> Action;
    
    /// Handle mouse events
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Action;
    
    /// Update component state based on action
    async fn update(&mut self, action: Action) -> crate::error::Result<Option<Action>>;
    
    /// Render component to terminal
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
```

### UI Component Traits

Additional traits for interactive UI components (`src/ui/components/traits.rs`):

```rust
/// Trait for interactive components that can receive focus and handle events
pub trait InteractiveComponent {
    fn get_state(&self) -> ComponentState;
    fn is_focused(&self) -> bool;
    fn set_focus(&mut self, focused: bool);
    fn reset(&mut self);
}

/// Trait for components that can be rendered
pub trait Renderable {
    fn render(&mut self, area: Rect, frame: &mut Frame);
}

/// Marker trait for components that can handle keyboard input
pub trait KeyboardInput {
    fn on_key(&mut self, key: KeyEvent) -> bool;
}
```

### Component Implementations

Components implement the `Component` trait and are injected via `Arc<dyn Component>`:

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

### Integration Principles

All external integrations follow these principles:

1. **Resilience**: Retry logic with exponential backoff
2. **Circuit Breaker**: Prevent cascading failures
3. **Timeouts**: Never wait indefinitely
4. **Fallback**: Graceful degradation when service unavailable
5. **Audit**: Log all external calls
6. **Health Checks**: Monitor service availability

### Common Integration Pattern

```rust
// src/integrations/mod.rs
pub mod medicare;          // Medicare Online
pub mod pbs;               // PBS API
pub mod air;               // Australian Immunisation Register
pub mod hi_service;        // Healthcare Identifiers
pub mod mhr;               // My Health Record
pub mod proda;             // PRODA Authentication
pub mod secure_messaging;  // HealthLink, Medical Objects
pub mod pathology;         // Lab integrations
pub mod drug_database;     // MIMS or AusDI
pub mod hl7;               // HL7 v2.x parser
pub mod fhir;              // FHIR client

// Common integration trait
#[async_trait]
pub trait ExternalService: Send + Sync {
    async fn health_check(&self) -> Result<HealthStatus, IntegrationError>;
    fn service_name(&self) -> &str;
    fn is_critical(&self) -> bool;  // Can app function without this service?
}

pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: u64,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
}
```

### PRODA Authentication Service

**Critical Foundation**: All Services Australia APIs require PRODA

```rust
// src/integrations/proda/mod.rs
pub struct ProdaAuthService {
    client_id: String,
    client_secret: String,
    token_url: String,
    http_client: reqwest::Client,
    token_cache: Arc<RwLock<Option<CachedToken>>>,
}

impl ProdaAuthService {
    pub async fn get_token(&self) -> Result<String, ProdaError> {
        // Check cache first
        if let Some(token) = self.get_cached_token().await {
            if !token.is_expired() {
                return Ok(token.access_token);
            }
        }
        
        // Request new token
        let response = self.http_client
            .post(&self.token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .timeout(Duration::from_secs(30))
            .send()
            .await?;
        
        let token_response: TokenResponse = response.json().await?;
        
        // Cache token
        let cached = CachedToken {
            access_token: token_response.access_token.clone(),
            expires_at: Utc::now() + Duration::from_secs(token_response.expires_in),
        };
        
        *self.token_cache.write().await = Some(cached);
        
        Ok(token_response.access_token)
    }
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
        
        // 2. Build SOAP request (simplified)
        let soap_request = self.build_claim_request(&claim)?;
        
        // 3. Send with retry logic
        let response = self.send_with_retry(soap_request, 3).await?;
        
        // 4. Parse SOAP response
        let claim_response = self.parse_claim_response(&response)?;
        
        // 5. Audit log
        info!("Medicare claim submitted: claim_id={}", claim_response.claim_id);
        
        Ok(claim_response)
    }
    
    pub async fn verify_patient_eligibility(
        &self,
        medicare_number: &str,
        irn: u8,
    ) -> Result<EligibilityResponse, MedicareError> {
        let token = self.proda_auth.get_token().await?;
        
        // SOAP request to verify eligibility
        let response = self.http_client
            .post(format!("{}/verification", self.base_url))
            .bearer_auth(token)
            .header("SOAPAction", "verify")
            .body(self.build_verification_request(medicare_number, irn)?)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;
        
        self.parse_eligibility_response(&response.text().await?)
    }
}
```

### Drug Database Integration

**Critical for Prescription Safety**

```rust
// src/integrations/drug_database/mod.rs
pub trait DrugDatabase: Send + Sync {
    async fn search_drug(&self, query: &str) -> Result<Vec<Drug>, DrugDbError>;
    async fn get_drug(&self, code: &str) -> Result<Drug, DrugDbError>;
    async fn check_interactions(&self, drugs: &[String]) -> Result<Vec<Interaction>, DrugDbError>;
    async fn check_allergy(&self, drug: &str, allergies: &[String]) -> Result<Vec<AllergyAlert>, DrugDbError>;
}

// MIMS implementation
pub struct MimsClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

impl MimsClient {
    pub async fn check_interactions(
        &self,
        drug_codes: &[String],
    ) -> Result<Vec<Interaction>, DrugDbError> {
        let response = self.http_client
            .post(format!("{}/interactions", self.base_url))
            .header("X-API-Key", &self.api_key)
            .json(&json!({ "drugs": drug_codes }))
            .send()
            .await?;
        
        let interactions: Vec<Interaction> = response.json().await?;
        
        // Sort by severity
        let mut sorted = interactions;
        sorted.sort_by_key(|i| std::cmp::Reverse(i.severity));
        
        Ok(sorted)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub drug_a: String,
    pub drug_b: String,
    pub severity: InteractionSeverity,
    pub description: String,
    pub clinical_guidance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InteractionSeverity {
    Minor = 1,
    Moderate = 2,
    Severe = 3,
    Contraindicated = 4,
}
```

### AIR (Australian Immunisation Register) Integration

```rust
// src/integrations/air/client.rs
pub struct AirClient {
    proda_auth: Arc<ProdaAuthService>,
    base_url: String,
    soap_client: SoapClient,
}

impl AirClient {
    pub async fn record_vaccination(
        &self,
        vaccination: VaccinationRecord,
    ) -> Result<AirResponse, AirError> {
        // Build AIR notification message
        let notification = self.build_notification(&vaccination)?;
        
        // Get PRODA token
        let token = self.proda_auth.get_token().await?;
        
        // Send SOAP request
        let response = self.soap_client
            .send(notification)
            .with_auth(token)
            .call()
            .await?;
        
        // Parse response
        let air_response = self.parse_response(&response)?;
        
        // Handle errors (duplicates, validation)
        if let Some(error) = air_response.error {
            if error.code == "DUPLICATE_NOTIFICATION" {
                warn!("Duplicate AIR notification: {}", vaccination.id);
                return Ok(air_response);  // Accept duplicate
            } else {
                return Err(AirError::ApiError(error));
            }
        }
        
        info!("AIR notification successful: notification_id={}", air_response.notification_id);
        
        Ok(air_response)
    }
    
    pub async fn retrieve_history(
        &self,
        ihi: &str,
    ) -> Result<Vec<VaccinationRecord>, AirError> {
        let token = self.proda_auth.get_token().await?;
        
        let request = self.build_history_request(ihi)?;
        let response = self.soap_client
            .send(request)
            .with_auth(token)
            .call()
            .await?;
        
        self.parse_history_response(&response)
    }
}
```

### HL7 v2.x Message Parser

```rust
// src/integrations/hl7/parser.rs
pub struct Hl7Parser;

impl Hl7Parser {
    pub fn parse_oru_message(message: &str) -> Result<PathologyResult, Hl7Error> {
        // Parse HL7 message
        let segments = message.lines().collect::<Vec<_>>();
        
        // Extract MSH (Message Header)
        let msh = Self::parse_msh(segments[0])?;
        
        // Extract PID (Patient Identification)
        let pid = segments.iter()
            .find(|s| s.starts_with("PID"))
            .ok_or(Hl7Error::MissingSegment("PID"))?;
        let patient_info = Self::parse_pid(pid)?;
        
        // Extract OBR (Observation Request)
        let obr = segments.iter()
            .find(|s| s.starts_with("OBR"))
            .ok_or(Hl7Error::MissingSegment("OBR"))?;
        let test_info = Self::parse_obr(obr)?;
        
        // Extract OBX (Observation/Result) segments
        let obx_segments: Vec<_> = segments.iter()
            .filter(|s| s.starts_with("OBX"))
            .collect();
        
        let results = obx_segments.iter()
            .map(|obx| Self::parse_obx(obx))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(PathologyResult {
            message_id: msh.message_id,
            patient: patient_info,
            test_info,
            results,
            received_at: Utc::now(),
        })
    }
    
    fn parse_pid(segment: &str) -> Result<PatientInfo, Hl7Error> {
        let fields: Vec<&str> = segment.split('|').collect();
        
        Ok(PatientInfo {
            medicare_number: fields.get(3).map(|s| s.to_string()),
            name: fields.get(5).map(|s| s.to_string()),
            dob: fields.get(7).and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok()),
        })
    }
    
    fn parse_obx(segment: &str) -> Result<TestResult, Hl7Error> {
        let fields: Vec<&str> = segment.split('|').collect();
        
        Ok(TestResult {
            test_code: fields.get(3).map(|s| s.to_string()).unwrap_or_default(),
            test_name: fields.get(3).map(|s| s.to_string()).unwrap_or_default(),
            value: fields.get(5).map(|s| s.to_string()),
            units: fields.get(6).map(|s| s.to_string()),
            reference_range: fields.get(7).map(|s| s.to_string()),
            abnormal_flag: fields.get(8).map(|s| s.to_string()),
        })
    }
}
```

### FHIR Client Architecture

```rust
// src/integrations/fhir/client.rs
pub struct FhirClient {
    base_url: String,
    http_client: reqwest::Client,
    auth_provider: Arc<dyn FhirAuthProvider>,
}

impl FhirClient {
    pub async fn create_patient(&self, patient: &Patient) -> Result<FhirPatient, FhirError> {
        // Convert domain model to FHIR resource
        let fhir_patient = FhirPatient {
            resource_type: "Patient".to_string(),
            identifier: vec![
                FhirIdentifier {
                    system: "http://ns.electronichealth.net.au/id/hi/ihi/1.0".to_string(),
                    value: patient.ihi.clone().unwrap_or_default(),
                },
                FhirIdentifier {
                    system: "http://ns.electronichealth.net.au/id/medicare-number".to_string(),
                    value: patient.medicare_number.clone().unwrap_or_default(),
                },
            ],
            name: vec![
                FhirName {
                    family: patient.last_name.clone(),
                    given: vec![patient.first_name.clone()],
                },
            ],
            birth_date: patient.date_of_birth.to_string(),
            // ... more FHIR mappings
        };
        
        // POST to FHIR server
        let token = self.auth_provider.get_token().await?;
        
        let response = self.http_client
            .post(format!("{}/Patient", self.base_url))
            .bearer_auth(token)
            .json(&fhir_patient)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(FhirError::ServerError(error_text));
        }
        
        response.json().await.map_err(Into::into)
    }
    
    pub async fn upload_document(
        &self,
        document: ClinicalDocument,
    ) -> Result<String, FhirError> {
        // Convert to FHIR DocumentReference
        let doc_ref = FhirDocumentReference {
            resource_type: "DocumentReference".to_string(),
            status: "current".to_string(),
            doc_status: "final".to_string(),
            r#type: CodeableConcept {
                coding: vec![
                    Coding {
                        system: "http://loinc.org".to_string(),
                        code: "60591-5".to_string(),  // Patient summary
                        display: "Patient Summary".to_string(),
                    },
                ],
            },
            subject: Reference {
                reference: format!("Patient/{}", document.patient_id),
            },
            content: vec![
                Content {
                    attachment: Attachment {
                        content_type: "application/pdf".to_string(),
                        data: base64::encode(&document.pdf_data),
                    },
                },
            ],
        };
        
        let token = self.auth_provider.get_token().await?;
        
        let response = self.http_client
            .post(format!("{}/DocumentReference", self.base_url))
            .bearer_auth(token)
            .json(&doc_ref)
            .send()
            .await?;
        
        let created: FhirDocumentReference = response.json().await?;
        
        Ok(created.id.unwrap_or_default())
    }
}
```

### Secure Messaging Integration

```rust
// src/integrations/secure_messaging/mod.rs
pub mod healthlink;
pub mod medical_objects;
pub mod argus;

pub trait SecureMessagingProvider: Send + Sync {
    async fn send_message(&self, message: SecureMessage) -> Result<MessageId, MessagingError>;
    async fn receive_messages(&self) -> Result<Vec<SecureMessage>, MessagingError>;
    async fn acknowledge(&self, message_id: &str) -> Result<(), MessagingError>;
}

// HealthLink implementation
pub struct HealthLinkClient {
    endpoint: String,
    practice_id: String,
    certificate: Certificate,
    http_client: reqwest::Client,
}

impl HealthLinkClient {
    pub async fn send_referral(
        &self,
        referral: Referral,
    ) -> Result<MessageId, MessagingError> {
        // Build CDA document
        let cda_document = self.build_referral_cda(&referral)?;
        
        // Create secure message
        let message = SecureMessage {
            id: Uuid::new_v4().to_string(),
            from: self.practice_id.clone(),
            to: referral.recipient_provider_id,
            subject: format!("Referral: {} {}", referral.patient_first_name, referral.patient_last_name),
            body: cda_document,
            attachments: vec![],
            priority: MessagePriority::Normal,
        };
        
        // Send via HealthLink SOAP API
        let response = self.http_client
            .post(&self.endpoint)
            .header("Content-Type", "application/soap+xml")
            .body(self.build_soap_envelope(&message)?)
            .send()
            .await?;
        
        let message_id = self.parse_send_response(&response.text().await?)?;
        
        info!("HealthLink message sent: {}", message_id);
        
        Ok(message_id)
    }
}
```

### Drug Interaction Checking Architecture

```rust
// src/integrations/drug_database/interaction_engine.rs
pub struct InteractionEngine {
    drug_db: Arc<dyn DrugDatabase>,
    cache: Arc<RwLock<HashMap<String, Vec<Interaction>>>>,
}

impl InteractionEngine {
    pub async fn check_prescription(
        &self,
        new_medication: &str,
        current_medications: &[String],
        allergies: &[String],
    ) -> Result<SafetyCheckResult, DrugDbError> {
        let mut warnings = Vec::new();
        let mut alerts = Vec::new();
        
        // 1. Check drug-drug interactions
        let mut all_meds = current_medications.to_vec();
        all_meds.push(new_medication.to_string());
        
        let interactions = self.drug_db.check_interactions(&all_meds).await?;
        
        for interaction in interactions {
            match interaction.severity {
                InteractionSeverity::Contraindicated | InteractionSeverity::Severe => {
                    alerts.push(SafetyAlert::Interaction(interaction));
                }
                InteractionSeverity::Moderate => {
                    warnings.push(SafetyWarning::Interaction(interaction));
                }
                _ => {}
            }
        }
        
        // 2. Check allergies
        let allergy_alerts = self.drug_db.check_allergy(new_medication, allergies).await?;
        
        for alert in allergy_alerts {
            if alert.severity >= AllergySeverity::Severe {
                alerts.push(SafetyAlert::Allergy(alert));
            } else {
                warnings.push(SafetyWarning::Allergy(alert));
            }
        }
        
        Ok(SafetyCheckResult {
            is_safe: alerts.is_empty(),
            alerts,
            warnings,
        })
    }
}

#[derive(Debug)]
pub struct SafetyCheckResult {
    pub is_safe: bool,
    pub alerts: Vec<SafetyAlert>,      // Must address before prescribing
    pub warnings: Vec<SafetyWarning>,   // Should review but can proceed
}

#[derive(Debug)]
pub enum SafetyAlert {
    Interaction(Interaction),
    Allergy(AllergyAlert),
    Contraindication(String),
}
```

### Pathology Lab Integration Architecture

```rust
// src/integrations/pathology/mod.rs
pub trait PathologyLab: Send + Sync {
    async fn send_order(&self, order: PathologyOrder) -> Result<OrderConfirmation, PathologyError>;
    async fn fetch_results(&self) -> Result<Vec<PathologyResult>, PathologyError>;
    fn lab_name(&self) -> &str;
}

// Generic HL7 receiver for all labs
pub struct Hl7ResultReceiver {
    hl7_parser: Arc<Hl7Parser>,
    patient_matcher: Arc<PatientMatcher>,
    result_repository: Arc<dyn PathologyResultRepository>,
}

impl Hl7ResultReceiver {
    pub async fn process_incoming_message(
        &self,
        raw_message: String,
    ) -> Result<(), PathologyError> {
        // 1. Parse HL7 message
        let parsed = self.hl7_parser.parse_oru_message(&raw_message)?;
        
        // 2. Match to patient
        let patient_id = self.patient_matcher
            .match_patient(&parsed.patient)
            .await?
            .ok_or(PathologyError::PatientNotFound)?;
        
        // 3. Store result
        let result = PathologyResult {
            id: Uuid::new_v4(),
            patient_id,
            lab_name: parsed.test_info.lab_name,
            test_name: parsed.test_info.test_name,
            collected_at: parsed.test_info.collected_at,
            results: parsed.results,
            pdf_report: None,  // Fetch separately if available
            status: ResultStatus::Final,
            is_acknowledged: false,
        };
        
        self.result_repository.create(result).await?;
        
        // 4. Check for abnormal results
        let has_abnormal = parsed.results.iter()
            .any(|r| r.abnormal_flag.as_deref() == Some("H") || r.abnormal_flag.as_deref() == Some("L"));
        
        if has_abnormal {
            // Trigger alert to practitioner
            warn!("Abnormal pathology result received for patient {}", patient_id);
        }
        
        Ok(())
    }
}

// Patient matching logic (fuzzy matching on Medicare, DOB, name)
pub struct PatientMatcher {
    patient_repository: Arc<dyn PatientRepository>,
}

impl PatientMatcher {
    pub async fn match_patient(&self, hl7_patient: &PatientInfo) -> Result<Option<Uuid>, PatientError> {
        // Try Medicare number first (most reliable)
        if let Some(ref medicare) = hl7_patient.medicare_number {
            if let Some(patient) = self.patient_repository.find_by_medicare(medicare).await? {
                return Ok(Some(patient.id));
            }
        }
        
        // Fall back to name + DOB matching
        if let (Some(ref name), Some(dob)) = (&hl7_patient.name, hl7_patient.dob) {
            let search_results = self.patient_repository.search(PatientSearchQuery {
                name: Some(name.clone()),
                date_of_birth: Some(dob),
                ..Default::default()
            }).await?;
            
            if search_results.items.len() == 1 {
                return Ok(Some(search_results.items[0].id));
            }
        }
        
        // No match or ambiguous - requires manual matching
        Ok(None)
    }
}
```

### Circuit Breaker Pattern for External Services

```rust
// src/integrations/circuit_breaker.rs
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed,                          // Normal operation
    Open { opened_at: DateTime<Utc> },  // Blocking calls
    HalfOpen,                        // Testing if service recovered
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        // Check circuit state
        let state = self.state.read().await.clone();
        
        match state {
            CircuitState::Open { opened_at } => {
                // Check if timeout elapsed
                if Utc::now() - opened_at > self.timeout {
                    // Try half-open
                    *self.state.write().await = CircuitState::HalfOpen;
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            _ => {}
        }
        
        // Execute call
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(CircuitBreakerError::CallFailed(e))
            }
        }
    }
    
    async fn on_failure(&self) {
        // Increment failure counter, open circuit if threshold reached
        // Implementation details...
    }
    
    async fn on_success(&self) {
        // Reset failure counter, close circuit if in half-open
        // Implementation details...
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

### Mock Repositories

OpenGP uses **in-memory mock repository implementations** to test service-layer logic without a real database.

- **Location**: `src/infrastructure/database/mocks.rs`
- **Pattern**: `Arc<Mutex<Vec<T>>>` storage for async + thread-safe tests
- **Purpose**: make service tests fast, deterministic, and independent of SQLx/SQLite/PostgreSQL

#### Test Dependency Flow

```
Service Layer Tests
       |
       v
Mock Repositories (in-memory)
       |
       v
Fixture Generators (test data)
       |
       v
Assertion Helpers (verification)
```

This keeps the **domain/service layer** testable via dependency injection while preserving the layer rule:

```
tests/ (outer)
  |
  v uses
domain::*/service.rs
  |
  v depends on
domain::*/repository.rs (traits)
  |
  v implemented by (in tests)
infrastructure::database::mocks::*
```

**Example (from `src/infrastructure/database/mocks.rs`):**

```rust
#[derive(Clone)]
pub struct MockPatientRepository {
    storage: Arc<Mutex<Vec<Patient>>>,
}

impl MockPatientRepository {
    /// Create a new empty mock patient repository
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock repository with initial patients (for testing)
    pub fn with_patients(patients: Vec<Patient>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(patients)),
        }
    }
}

impl Default for MockPatientRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PatientRepository for MockPatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage.iter().find(|p| p.id == id).cloned())
    }

    async fn find_by_medicare(
        &self,
        medicare: &str,
    ) -> Result<Option<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage
            .iter()
            .find(|p| p.medicare_number.as_deref() == Some(medicare))
            .cloned())
    }

    async fn list_active(&self) -> Result<Vec<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage.iter().filter(|p| p.is_active).cloned().collect())
    }

    async fn create(&self, patient: Patient) -> Result<Patient, PatientRepositoryError> {
        let mut storage = self.storage.lock().await;
        storage.push(patient.clone());
        Ok(patient)
    }

    async fn update(&self, patient: Patient) -> Result<Patient, PatientRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(pos) = storage.iter().position(|p| p.id == patient.id) {
            storage[pos] = patient.clone();
            Ok(patient)
        } else {
            Err(PatientRepositoryError::NotFound)
        }
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), PatientRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(patient) = storage.iter_mut().find(|p| p.id == id) {
            patient.is_active = false;
            Ok(())
        } else {
            Err(PatientRepositoryError::NotFound)
        }
    }
}
```

### Fixture Generators

OpenGP provides **fixture generators** that create realistic domain entities for tests, demos, and seeded environments.

- **Location**: `src/infrastructure/fixtures/`
- **Pattern**: `Config + Default` + `Generator::new(config)` + `generate() -> Vec<T>`
- **Goal**: reduce brittle, repetitive test setup and keep test data realistic (e.g., weekday business hours for appointments)

The module re-exports generator types to keep usage consistent across the codebase:

```rust
pub mod appointment_generator;
pub mod audit_generator;
pub mod immunisation_generator;
pub mod patient_generator;
pub mod prescription_generator;

pub use appointment_generator::{AppointmentGenerator, AppointmentGeneratorConfig};
pub use audit_generator::{AuditGenerator, AuditGeneratorConfig};
pub use immunisation_generator::{ImmunisationGenerator, ImmunisationGeneratorConfig};
pub use patient_generator::{PatientGenerator, PatientGeneratorConfig};
pub use prescription_generator::{PrescriptionGenerator, PrescriptionGeneratorConfig};
```

**Excerpt (from `src/infrastructure/fixtures/appointment_generator.rs`):**

```rust
/// Configuration for appointment generation
///
/// Controls how many appointments are generated and their characteristics.
#[derive(Debug, Clone)]
pub struct AppointmentGeneratorConfig {
    /// Number of appointments to generate
    pub count: usize,
    /// Percentage of appointments that should be in the future (0.0-1.0)
    pub future_percentage: f32,
    /// Percentage of appointments that should be confirmed (0.0-1.0)
    pub confirmed_percentage: f32,
    /// Percentage of appointments that should be urgent (0.0-1.0)
    pub urgent_percentage: f32,
    /// Percentage of appointments with notes (0.0-1.0)
    pub notes_percentage: f32,
}

impl Default for AppointmentGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            future_percentage: 0.70,
            confirmed_percentage: 0.60,
            urgent_percentage: 0.10,
            notes_percentage: 0.40,
        }
    }
}

/// Generator for realistic appointment test data
///
/// Creates appointments with realistic time slots (9am-5pm weekdays),
/// various types, and statuses. Supports configurable practitioner and patient IDs.
pub struct AppointmentGenerator {
    config: AppointmentGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl AppointmentGenerator {
    /// Create a new appointment generator with the given configuration
    pub fn new(config: AppointmentGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a vector of appointments
pub fn generate(&mut self) -> Vec<Appointment> {
    (0..self.config.count)
        .map(|_| self.generate_appointment())
        .collect()
}
```

### Assertion Helpers

Tests use **assertion helpers** to compare domain entities field-by-field with clear error messages.

- **Location**: `tests/helpers/assertions.rs`
- **Approach**: explicit comparisons for each important field, with targeted failure messages
- **Benefit**: faster diagnosis than large `assert_eq!(actual, expected)` diffs (especially for nested structs and timestamps)

**Excerpt (from `tests/helpers/assertions.rs`):**

```rust
/// Assert that two Patient entities are equal
///
/// Compares all important fields including IDs, names, dates, contact info,
/// and status flags. Provides field-by-field error messages on failure.
pub fn assert_patient_eq(actual: &Patient, expected: &Patient) {
    assert_eq!(actual.id, expected.id, "Patient ID mismatch");
    assert_eq!(actual.ihi, expected.ihi, "Patient IHI mismatch");
    assert_eq!(
        actual.medicare_number, expected.medicare_number,
        "Patient Medicare number mismatch"
    );
    assert_eq!(
        actual.medicare_irn, expected.medicare_irn,
        "Patient Medicare IRN mismatch"
    );
    assert_eq!(
        actual.medicare_expiry, expected.medicare_expiry,
        "Patient Medicare expiry mismatch"
    );
```

Timestamp comparisons are intentionally normalized to avoid flaky tests due to sub-second differences:

```rust
/// Helper function to compare DateTime values with approximate equality
///
/// Compares two DateTime<Utc> values to the same second, allowing for
/// minor differences in microseconds that may occur during test execution.
fn assert_datetime_eq(
    actual: DateTime<chrono::Utc>,
    expected: DateTime<chrono::Utc>,
    message: &str,
) {
    let actual_secs = actual.timestamp();
    let expected_secs = expected.timestamp();
    assert_eq!(actual_secs, expected_secs, "{}", message);
}
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

## Audit Domain Module

### Overview
The audit module provides comprehensive change tracking for compliance with Australian healthcare regulations (Privacy Act 1988, My Health Records Act 2012).

### Architecture

```
src/domain/audit/
├── mod.rs          # Module exports
├── model.rs        # AuditEntry and AuditAction types
├── service.rs      # Audit service layer
├── repository.rs   # Repository trait
└── error.rs        # Audit-specific errors
```

### Key Components

#### AuditEntry
Domain entity representing a single audit log entry.

```rust
pub struct AuditEntry {
    pub id: Uuid,
    pub entity_type: String,      // "appointment", "patient", etc.
    pub entity_id: Uuid,
    pub action: AuditAction,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}
```

#### AuditAction
Enum representing different types of auditable actions.

```rust
pub enum AuditAction {
    Created,
    Updated,
    StatusChanged { from: String, to: String },
    Rescheduled { from: DateTime<Utc>, to: DateTime<Utc> },
    Cancelled { reason: String },
}
```

#### AuditRepository Trait
Defines persistence interface for audit logs.

```rust
#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry>;
    async fn find_by_entity(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<AuditEntry>>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuditEntry>>;
    async fn find_by_time_range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<AuditEntry>>;
}
```

#### AuditService
Business logic layer for audit operations.

```rust
pub struct AuditService {
    repository: Arc<dyn AuditRepository>,
}

impl AuditService {
    pub async fn log(&self, entry: AuditEntry) -> Result<()>;
    pub async fn get_appointment_history(&self, appointment_id: Uuid) -> Result<Vec<AuditEntry>>;
    pub async fn get_user_activity(&self, user_id: Uuid) -> Result<Vec<AuditEntry>>;
}
```

### Database Schema

```sql
CREATE TABLE audit_logs (
    id BLOB PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id BLOB NOT NULL,
    action TEXT NOT NULL,  -- JSON serialized AuditAction
    old_value TEXT,
    new_value TEXT,
    changed_by BLOB NOT NULL,
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (changed_by) REFERENCES users(id)
);

CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_user ON audit_logs(changed_by);
CREATE INDEX idx_audit_time ON audit_logs(changed_at);
CREATE INDEX idx_audit_entity_time ON audit_logs(entity_type, entity_id, changed_at);
```

### Design Decisions

#### Append-Only Design
Audit logs are **append-only** - no UPDATE or DELETE operations allowed. This ensures:
- Immutable audit trail
- Compliance with healthcare regulations
- Forensic integrity

#### JSON Serialization
`AuditAction` enum is serialized to JSON for storage, allowing:
- Complex action types with associated data
- Easy querying and filtering
- Future extensibility

#### Automatic Logging
Audit entries are created automatically by service layer methods:

```rust
pub async fn mark_arrived(&self, appointment_id: Uuid, user_id: Uuid) -> Result<Appointment> {
    let mut appt = self.repository.find_by_id(appointment_id).await?
        .ok_or(ServiceError::NotFound)?;
    
    let old_status = appt.status;
    appt.mark_arrived(user_id);
    
    let updated = self.repository.update(appt.clone()).await?;
    
    // Automatic audit logging
    let audit_entry = AuditEntry::for_status_change(
        appointment_id,
        old_status,
        updated.status,
        user_id
    );
    self.audit_service.log(audit_entry).await?;
    
    Ok(updated)
}
```

### Integration Points

#### Appointment Service
All appointment status changes automatically create audit entries:
- `mark_arrived()` → StatusChanged audit entry
- `mark_completed()` → StatusChanged audit entry
- `mark_no_show()` → StatusChanged audit entry
- `reschedule_appointment()` → Rescheduled audit entry

#### UI Components
Audit history accessible via:
- 'H' keybind in appointment detail modal
- Displays chronological timeline
- Color-coded by action type
- Sortable and filterable

### Compliance Features

#### Australian Healthcare Requirements
- **Privacy Act 1988**: All access to patient data logged
- **My Health Records Act 2012**: Complete audit trail maintained
- **Retention**: Audit logs retained indefinitely
- **Integrity**: Append-only design prevents tampering

#### Query Capabilities
- By entity (all changes to specific appointment)
- By user (all actions by specific user)
- By time range (changes within date range)
- Combined filters (entity + time range)

### Performance Considerations

#### Indexing Strategy
Four indexes optimize common queries:
1. `idx_audit_entity`: Entity lookups (most common)
2. `idx_audit_user`: User activity reports
3. `idx_audit_time`: Time-based queries
4. `idx_audit_entity_time`: Combined entity + time (fastest)

#### Pagination
For large audit trails, implement pagination:
```rust
pub async fn get_appointment_history_paginated(
    &self,
    appointment_id: Uuid,
    page: usize,
    page_size: usize,
) -> Result<(Vec<AuditEntry>, usize)>
```

### Future Enhancements

#### Planned Features
- [ ] User name resolution (currently shows UUIDs)
- [ ] CSV export for audit reports
- [ ] Advanced filtering in UI
- [ ] Audit log retention policies
- [ ] Automated compliance reports

#### Extension Points
- Additional entity types (patient, prescription, etc.)
- Custom action types per entity
- Webhook notifications for critical actions
- Integration with external audit systems
