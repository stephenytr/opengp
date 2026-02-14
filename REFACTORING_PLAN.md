# OpenGP Developer-Friendliness Refactoring Plan

**Date**: 2026-02-14  
**Status**: Analysis Complete - Ready for Implementation  
**Total Estimated Effort**: 60-80 hours over 6-8 weeks  
**Expected Code Reduction**: 1,500+ lines of boilerplate  

---

## Executive Summary

This comprehensive refactoring plan addresses developer experience pain points across three layers of the OpenGP codebase:

1. **Domain Layer** (6 incomplete modules, constructor complexity, missing patterns)
2. **Infrastructure Layer** (660+ lines of repository boilerplate, empty integrations)
3. **UI Layer** (3,164-line calendar component, 500+ lines of widget duplication)

**Key Insight**: The codebase has a solid architectural foundation but is at a critical inflection point. Without refactoring, complexity will compound exponentially as new features are added.

---

## Table of Contents

1. [Current State Assessment](#current-state-assessment)
2. [Domain Layer Refactoring](#domain-layer-refactoring)
3. [Infrastructure Layer Refactoring](#infrastructure-layer-refactoring)
4. [UI Layer Refactoring](#ui-layer-refactoring)
5. [Cross-Cutting Improvements](#cross-cutting-improvements)
6. [Implementation Roadmap](#implementation-roadmap)
7. [Success Metrics](#success-metrics)

---

## Current State Assessment

### Codebase Statistics

| Metric | Current Value |
|--------|---------------|
| Total Rust Files | 73 |
| Total Lines of Code | ~14,686 |
| Largest File | `calendar.rs` (3,164 lines) |
| Average Component Size | 1,070 lines |
| Estimated Boilerplate | ~1,500 lines (10%) |
| Test Coverage | ~20% |

### Architecture Health

| Layer | Status | Issues | Priority |
|-------|--------|--------|----------|
| Domain | ⚠️ Inconsistent | 6 incomplete modules, constructor complexity | High |
| Infrastructure | ⚠️ Boilerplate-heavy | 660+ lines duplication, empty integrations | High |
| UI | ⚠️ Complexity creep | 3,164-line component, 500+ lines duplication | Critical |

### Code Quality Issues

**Critical (Blocking Development)**:
- ❌ 6 domain modules incomplete (Prescription, Immunisation, Referral, Pathology, Clinical, Billing)
- ❌ Calendar component approaching unmaintainability (3,164 lines)
- ❌ 660+ lines of repository boilerplate

**High Priority (Slowing Development)**:
- ⚠️ Constructor complexity (9 parameters in Prescription/Immunisation)
- ⚠️ 500+ lines of UI widget duplication
- ⚠️ Inconsistent error handling patterns

**Medium Priority (Technical Debt)**:
- ⚠️ 73 `.unwrap()` / `.expect()` calls (mostly in tests, some in fixtures)
- ⚠️ Empty integration stubs (Medicare, PBS, AIR, HI Service)
- ⚠️ Underutilized Theme system

---

## Domain Layer Refactoring

### Problem Statement

**6 incomplete domain modules** lack service/repository/error/dto layers, preventing business logic enforcement and healthcare compliance.

**Constructor complexity** makes Prescription and Immunisation models error-prone (9 parameters each).

### Phase 1: Complete Missing Domain Modules (20-25 hours)

#### 1.1 Implement Builder Pattern for Complex Models (4-5 hours)

**Problem**: Prescription and Immunisation constructors have 9 parameters each.

**Solution**: Implement type-state builder pattern for compile-time safety.

**Before**:
```rust
// ❌ Hard to use, error-prone
let prescription = Prescription::new(
    patient_id,
    practitioner_id,
    consultation_id,
    medication,
    dosage,
    quantity,
    repeats,
    directions,
    created_by,
);
```

**After**:
```rust
// ✅ Clear, type-safe, discoverable
let prescription = Prescription::builder()
    .patient(patient_id)
    .practitioner(practitioner_id)
    .medication(medication)
    .dosage("10mg twice daily")
    .quantity(60)
    .repeats(5)
    .directions("Take with food")
    .created_by(user_id)
    .build()?;
```

**Implementation**:
- Add `derive_builder` crate to Cargo.toml
- Create `PrescriptionBuilder` and `ImmunisationBuilder`
- Mark required fields with `#[builder(required)]`
- Add validation in `build()` method

**Files to Create/Modify**:
- `src/domain/prescription/model.rs` (add builder)
- `src/domain/immunisation/model.rs` (add builder)

**Estimated Time**: 4-5 hours

---

#### 1.2 Complete Prescription Domain (4-5 hours)

**Files to Create**:
- `src/domain/prescription/service.rs` (business logic)
- `src/domain/prescription/repository.rs` (persistence trait)
- `src/domain/prescription/error.rs` (domain errors)
- `src/domain/prescription/dto.rs` (data transfer objects)

**Service Methods**:
```rust
pub struct PrescriptionService {
    repository: Arc<dyn PrescriptionRepository>,
    audit_logger: Arc<AuditLogger>,
}

impl PrescriptionService {
    pub async fn create_prescription(&self, data: NewPrescriptionData, user: &User) -> Result<Prescription>;
    pub async fn cancel_prescription(&self, id: Uuid, reason: String, user: &User) -> Result<()>;
    pub async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Prescription>>;
    pub async fn check_interactions(&self, patient_id: Uuid, medication: &Medication) -> Result<Vec<Interaction>>;
}
```

**Error Types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum PrescriptionError {
    #[error("Prescription not found: {0}")]
    NotFound(Uuid),
    
    #[error("Prescription already cancelled")]
    AlreadyCancelled,
    
    #[error("Drug interaction detected: {0}")]
    DrugInteraction(String),
    
    #[error("PBS authority required for {0}")]
    AuthorityRequired(String),
}
```

**Estimated Time**: 4-5 hours

---

#### 1.3 Complete Immunisation Domain (4-5 hours)

**Files to Create**:
- `src/domain/immunisation/service.rs`
- `src/domain/immunisation/repository.rs`
- `src/domain/immunisation/error.rs`
- `src/domain/immunisation/dto.rs`

**Service Methods**:
```rust
pub struct ImmunisationService {
    repository: Arc<dyn ImmunisationRepository>,
    air_client: Arc<dyn AIRClient>,
    audit_logger: Arc<AuditLogger>,
}

impl ImmunisationService {
    pub async fn record_immunisation(&self, data: NewImmunisationData, user: &User) -> Result<Immunisation>;
    pub async fn submit_to_air(&self, id: Uuid) -> Result<String>; // Returns transaction ID
    pub async fn get_schedule(&self, patient_id: Uuid) -> Result<ImmunisationSchedule>;
    pub async fn check_overdue(&self, patient_id: Uuid) -> Result<Vec<Immunisation>>;
}
```

**Estimated Time**: 4-5 hours

---

#### 1.4 Complete Remaining Domains (8-10 hours)

**Referral Domain** (2-3 hours):
- Service: create, send, track referrals
- Repository: CRUD operations
- Error types: validation, delivery failures
- DTOs: create, update, search

**Pathology Domain** (2-3 hours):
- Service: order tests, receive results, acknowledge
- Repository: CRUD operations
- Error types: validation, HL7 parsing
- DTOs: order, result, search

**Clinical Domain** (2-3 hours):
- Service: create consultations, sign notes
- Repository: CRUD operations with encryption
- Error types: validation, signing
- DTOs: SOAP notes, medical history

**Billing Domain** (2-3 hours):
- Service: create invoices, process payments, submit claims
- Repository: CRUD operations
- Error types: validation, claim submission
- DTOs: invoice, payment, claim

**Estimated Time**: 8-10 hours

---

### Phase 2: Extract Duplicate Code (5-6 hours)

#### 2.1 Extract Overlap Checking Logic (1-2 hours)

**Problem**: Overlap checking duplicated in `create_appointment()` and `reschedule_appointment()`.

**Solution**: Extract to dedicated method.

**Before** (duplicated in 2 places):
```rust
// In create_appointment()
let overlapping = self.repository
    .find_overlapping(practitioner_id, start_time, end_time)
    .await?;
if !overlapping.is_empty() {
    return Err(ServiceError::AppointmentOverlap);
}

// In reschedule_appointment() - SAME CODE
let overlapping = self.repository
    .find_overlapping(practitioner_id, start_time, end_time)
    .await?;
if !overlapping.is_empty() {
    return Err(ServiceError::AppointmentOverlap);
}
```

**After**:
```rust
impl AppointmentService {
    async fn check_no_overlap(
        &self,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        let overlapping = self.repository
            .find_overlapping(practitioner_id, start_time, end_time)
            .await?;
        
        let has_overlap = overlapping.iter()
            .any(|a| exclude_id.map_or(true, |id| a.id != id));
        
        if has_overlap {
            Err(ServiceError::AppointmentOverlap)
        } else {
            Ok(())
        }
    }
}
```

**Estimated Time**: 1-2 hours

---

#### 2.2 Extract Status Transition Validation (1-2 hours)

**Problem**: Status transition validation duplicated in 3 methods.

**Solution**: Extract to dedicated method with state machine pattern.

**Before** (duplicated in 3 places):
```rust
// In mark_arrived()
if appointment.status != AppointmentStatus::Scheduled 
    && appointment.status != AppointmentStatus::Confirmed {
    return Err(ServiceError::InvalidStatusTransition);
}

// In mark_completed() - SIMILAR CODE
// In mark_no_show() - SIMILAR CODE
```

**After**:
```rust
impl AppointmentService {
    fn validate_transition(
        &self,
        current: AppointmentStatus,
        target: AppointmentStatus,
    ) -> Result<(), ServiceError> {
        use AppointmentStatus::*;
        
        let valid = match (current, target) {
            (Scheduled | Confirmed, Arrived) => true,
            (Arrived | InProgress, Completed) => true,
            (Scheduled | Confirmed, NoShow) => true,
            _ => false,
        };
        
        if valid {
            Ok(())
        } else {
            Err(ServiceError::InvalidStatusTransition {
                from: current,
                to: target,
            })
        }
    }
}
```

**Estimated Time**: 1-2 hours

---

#### 2.3 Extract Audit Logging Pattern (2-3 hours)

**Problem**: Audit logging pattern repeated in 10+ methods.

**Solution**: Extract to helper method with builder pattern.

**Before** (repeated 10+ times):
```rust
self.audit_logger.log(AuditEntry {
    user_id: user.id,
    action: AuditAction::AppointmentUpdate,
    entity_id: appointment.id,
    timestamp: Utc::now(),
}).await?;
```

**After**:
```rust
impl AppointmentService {
    async fn audit_log(
        &self,
        action: AuditAction,
        entity_id: Uuid,
        user: &User,
    ) -> Result<(), ServiceError> {
        self.audit_logger.log(AuditEntry {
            user_id: user.id,
            action,
            entity_id,
            timestamp: Utc::now(),
        }).await?;
        Ok(())
    }
}

// Usage
self.audit_log(AuditAction::AppointmentUpdate, appointment.id, user).await?;
```

**Estimated Time**: 2-3 hours

---

## Infrastructure Layer Refactoring

### Problem Statement

**660+ lines of repository boilerplate** across patient, appointment, and audit repositories due to:
- Enum-to-String conversions (80+ lines)
- UUID byte conversions (150+ lines)
- DateTime RFC3339 conversions (90+ lines)
- Database error mapping (60+ lines)
- SELECT statement duplication (60+ lines)
- Row-to-Entity conversions (220+ lines)

### Phase 3: Database Helpers Module (5-6 hours)

#### 3.1 Create Database Helpers (3-4 hours)

**File to Create**: `src/infrastructure/database/helpers.rs`

```rust
//! Database helper functions to reduce boilerplate

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Convert UUID to SQLite-compatible byte array
pub fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

/// Convert SQLite byte array to UUID
pub fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, RepositoryError> {
    Uuid::from_slice(bytes)
        .map_err(|e| RepositoryError::InvalidData(format!("Invalid UUID: {}", e)))
}

/// Convert DateTime to RFC3339 string for SQLite
pub fn datetime_to_string(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Convert RFC3339 string to DateTime
pub fn string_to_datetime(s: &str) -> Result<DateTime<Utc>, RepositoryError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| RepositoryError::InvalidData(format!("Invalid datetime: {}", e)))
}

/// Map database errors to repository errors
pub fn map_db_error(err: sqlx::Error) -> RepositoryError {
    match err {
        sqlx::Error::Database(db_err) => {
            let msg = db_err.message();
            if msg.contains("UNIQUE constraint") {
                RepositoryError::ConstraintViolation("Duplicate entry".to_string())
            } else if msg.contains("FOREIGN KEY constraint") {
                RepositoryError::ConstraintViolation("Referenced entity does not exist".to_string())
            } else if msg.contains("NOT NULL constraint") {
                RepositoryError::ConstraintViolation("Required field is missing".to_string())
            } else {
                RepositoryError::Database(sqlx::Error::Database(db_err))
            }
        }
        _ => RepositoryError::Database(err),
    }
}
```

**Estimated Time**: 3-4 hours

---

#### 3.2 Implement Enum Serialization with Strum (2-3 hours)

**Problem**: 6+ identical enum-to-string match blocks (80+ lines).

**Solution**: Use `strum` crate for automatic enum serialization.

**Add to Cargo.toml**:
```toml
strum = { version = "0.26", features = ["derive"] }
strum_macros = "0.26"
```

**Before** (80+ lines of boilerplate):
```rust
impl AppointmentStatus {
    pub fn to_string(&self) -> String {
        match self {
            AppointmentStatus::Scheduled => "Scheduled".to_string(),
            AppointmentStatus::Confirmed => "Confirmed".to_string(),
            AppointmentStatus::Arrived => "Arrived".to_string(),
            // ... 5 more variants
        }
    }
    
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        match s {
            "Scheduled" => Ok(AppointmentStatus::Scheduled),
            "Confirmed" => Ok(AppointmentStatus::Confirmed),
            // ... 5 more variants
            _ => Err(ParseError::InvalidStatus(s.to_string())),
        }
    }
}
```

**After** (2 lines):
```rust
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    Arrived,
    InProgress,
    Completed,
    NoShow,
    Cancelled,
    Rescheduled,
}
```

**Apply to**:
- `AppointmentStatus` (8 variants)
- `AppointmentType` (13 variants)
- `Gender` (4 variants)
- `InvoiceStatus` (7 variants)
- `ClaimStatus` (6 variants)
- All other enums in domain models

**Estimated Time**: 2-3 hours

---

### Phase 4: Refactor Repositories (8-10 hours)

#### 4.1 Refactor Patient Repository (2-3 hours)

**Before** (287 lines with boilerplate):
```rust
async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
    let id_bytes = id.as_bytes().to_vec(); // Boilerplate
    
    let row = sqlx::query_as::<_, PatientRow>(
        r#"SELECT id, ihi, medicare_number, ... FROM patients WHERE id = ?"#,
    )
    .bind(id_bytes)
    .fetch_optional(&self.pool)
    .await?;
    
    match row {
        Some(r) => Ok(Some(r.into_patient()?)),
        None => Ok(None),
    }
}
```

**After** (using helpers):
```rust
use crate::infrastructure::database::helpers::*;

async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
    let row = sqlx::query_as::<_, PatientRow>(PATIENT_SELECT_QUERY)
        .bind(uuid_to_bytes(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
    
    row.map(|r| r.into_patient()).transpose()
}
```

**Estimated Time**: 2-3 hours

---

#### 4.2 Refactor Appointment Repository (3-4 hours)

**Before** (896 lines with massive boilerplate):
- 4 identical SELECT statements (60+ lines)
- 6 enum conversions (80+ lines)
- 50+ UUID conversions (150+ lines)

**After** (using helpers and constants):
```rust
const APPOINTMENT_SELECT_QUERY: &str = r#"
    SELECT id, patient_id, practitioner_id, start_time, end_time,
           appointment_type, status, notes, is_urgent, confirmed,
           created_at, updated_at
    FROM appointments
"#;

impl AppointmentRepository for SqlxAppointmentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Appointment>, RepositoryError> {
        let query = format!("{} WHERE id = ?", APPOINTMENT_SELECT_QUERY);
        
        sqlx::query_as::<_, AppointmentRow>(&query)
            .bind(uuid_to_bytes(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)?
            .map(|r| r.into_appointment())
            .transpose()
    }
}
```

**Estimated Time**: 3-4 hours

---

#### 4.3 Refactor Audit Repository (1-2 hours)

**Estimated Time**: 1-2 hours

---

#### 4.4 Create Repository Test Utilities (2-3 hours)

**File to Create**: `src/infrastructure/database/test_utils.rs`

```rust
//! Test utilities for repository testing

use sqlx::SqlitePool;

/// Create an in-memory SQLite pool for testing
pub async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

/// Create a mock patient for testing
pub fn create_test_patient() -> Patient {
    Patient::new(NewPatientData {
        first_name: "Test".to_string(),
        last_name: "Patient".to_string(),
        date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
        gender: Gender::Male,
    }).unwrap()
}

// Similar helpers for Appointment, Prescription, etc.
```

**Estimated Time**: 2-3 hours

---

### Phase 5: Implement External Integrations (16-20 hours)

#### 5.1 Create Integration Client Base (3-4 hours)

**File to Create**: `src/integrations/client.rs`

```rust
//! Base HTTP client for external API integrations

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        Self { client, base_url, api_key }
    }
    
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, IntegrationError> {
        // Implementation with retry logic, logging, error handling
    }
    
    pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R, IntegrationError> {
        // Implementation with retry logic, logging, error handling
    }
}
```

**Add to Cargo.toml**:
```toml
reqwest = { version = "0.12", features = ["json"] }
backoff = "0.4"
```

**Estimated Time**: 3-4 hours

---

#### 5.2 Implement Medicare Online Integration (4-5 hours)

**Files to Create**:
- `src/integrations/medicare/client.rs`
- `src/integrations/medicare/models.rs`
- `src/integrations/medicare/error.rs`

**Client Methods**:
```rust
pub struct MedicareClient {
    client: ApiClient,
}

impl MedicareClient {
    pub async fn verify_medicare_number(&self, number: &str, irn: u8) -> Result<bool>;
    pub async fn submit_claim(&self, claim: &MedicareClaim) -> Result<ClaimResponse>;
    pub async fn check_claim_status(&self, claim_id: &str) -> Result<ClaimStatus>;
}
```

**Estimated Time**: 4-5 hours

---

#### 5.3 Implement PBS API Integration (3-4 hours)

**Client Methods**:
```rust
pub struct PBSClient {
    client: ApiClient,
}

impl PBSClient {
    pub async fn search_medication(&self, query: &str) -> Result<Vec<Medication>>;
    pub async fn get_pbs_status(&self, medication_code: &str) -> Result<PBSStatus>;
    pub async fn request_authority(&self, request: &AuthorityRequest) -> Result<AuthorityResponse>;
}
```

**Estimated Time**: 3-4 hours

---

#### 5.4 Implement AIR Integration (3-4 hours)

**Client Methods**:
```rust
pub struct AIRClient {
    client: ApiClient,
}

impl AIRClient {
    pub async fn submit_immunisation(&self, immunisation: &Immunisation) -> Result<String>;
    pub async fn get_immunisation_history(&self, patient_ihi: &str) -> Result<Vec<Immunisation>>;
}
```

**Estimated Time**: 3-4 hours

---

#### 5.5 Implement HI Service Integration (3-4 hours)

**Client Methods**:
```rust
pub struct HIServiceClient {
    client: ApiClient,
}

impl HIServiceClient {
    pub async fn verify_ihi(&self, ihi: &str) -> Result<bool>;
    pub async fn search_ihi(&self, demographics: &Demographics) -> Result<Option<String>>;
    pub async fn verify_hpi_i(&self, hpi_i: &str) -> Result<bool>;
}
```

**Estimated Time**: 3-4 hours

---

## UI Layer Refactoring

### Problem Statement

**Calendar component approaching unmaintainability** (3,164 lines) with:
- 20+ state fields mixing multiple concerns
- 20+ render methods scattered throughout
- No clear separation of responsibilities

**500+ lines of widget duplication** across components:
- List selection logic repeated 3 times
- Search filtering logic repeated 2 times
- Modal handling boilerplate repeated 5+ times

### Phase 6: Extract Reusable Widgets (10-12 hours)

#### 6.1 Create ListSelector Widget (2-3 hours)

**File to Create**: `src/ui/widgets/list_selector.rs`

```rust
//! Reusable list selection widget with keyboard navigation

use ratatui::widgets::TableState;

pub struct ListSelector<T> {
    items: Vec<T>,
    state: TableState,
}

impl<T> ListSelector<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut state = TableState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self { items, state }
    }
    
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }
    
    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(self.items.len() - 1));
        }
    }
    
    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
    
    pub fn items(&self) -> &[T] {
        &self.items
    }
    
    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }
}
```

**Usage**:
```rust
// Before (repeated in 3 components)
pub struct PatientListComponent {
    patients: Vec<Patient>,
    table_state: TableState,
}

impl PatientListComponent {
    fn next(&mut self) { /* 10 lines of boilerplate */ }
    fn previous(&mut self) { /* 10 lines of boilerplate */ }
    fn select_first(&mut self) { /* 5 lines of boilerplate */ }
    fn select_last(&mut self) { /* 5 lines of boilerplate */ }
}

// After (reusable)
pub struct PatientListComponent {
    selector: ListSelector<Patient>,
}

impl PatientListComponent {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Down => { self.selector.next(); Action::Render }
            KeyCode::Up => { self.selector.previous(); Action::Render }
            KeyCode::Char('g') => { self.selector.select_first(); Action::Render }
            KeyCode::Char('G') => { self.selector.select_last(); Action::Render }
            _ => Action::None,
        }
    }
}
```

**Estimated Time**: 2-3 hours

---

#### 6.2 Create SearchFilter Widget (2-3 hours)

**File to Create**: `src/ui/widgets/search_filter.rs`

```rust
//! Reusable search/filter widget with fuzzy matching

use sublime_fuzzy::best_match;

pub struct SearchFilter<T> {
    query: String,
    items: Vec<T>,
    filtered: Vec<T>,
    extract_text: Box<dyn Fn(&T) -> String>,
}

impl<T: Clone> SearchFilter<T> {
    pub fn new<F>(items: Vec<T>, extract_text: F) -> Self
    where
        F: Fn(&T) -> String + 'static,
    {
        let filtered = items.clone();
        Self {
            query: String::new(),
            items,
            filtered,
            extract_text: Box::new(extract_text),
        }
    }
    
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.update_filtered();
    }
    
    pub fn query(&self) -> &str {
        &self.query
    }
    
    pub fn filtered(&self) -> &[T] {
        &self.filtered
    }
    
    fn update_filtered(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.items.clone();
        } else {
            self.filtered = self.items
                .iter()
                .filter(|item| {
                    let text = (self.extract_text)(item);
                    best_match(&self.query, &text).is_some()
                })
                .cloned()
                .collect();
        }
    }
}
```

**Estimated Time**: 2-3 hours

---

#### 6.3 Create ModalHandler Trait (2-3 hours)

**File to Create**: `src/ui/widgets/modal_handler.rs`

```rust
//! Trait for components that show modals

pub trait ModalHandler {
    fn is_modal_active(&self) -> bool;
    fn handle_modal_event(&mut self, key: KeyEvent) -> Action;
}

pub enum ModalType {
    Help,
    Detail,
    Search,
    Confirmation,
    Error,
}

pub struct ModalState {
    active: Option<ModalType>,
}

impl ModalState {
    pub fn new() -> Self {
        Self { active: None }
    }
    
    pub fn show(&mut self, modal: ModalType) {
        self.active = Some(modal);
    }
    
    pub fn hide(&mut self) {
        self.active = None;
    }
    
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }
    
    pub fn active_type(&self) -> Option<&ModalType> {
        self.active.as_ref()
    }
}
```

**Estimated Time**: 2-3 hours

---

#### 6.4 Create Additional Widgets (4-5 hours)

**ConfirmationDialog** (1-2 hours):
```rust
pub struct ConfirmationDialog {
    message: String,
    on_confirm: Box<dyn Fn() -> Action>,
    on_cancel: Box<dyn Fn() -> Action>,
}
```

**StatusBadge** (1 hour):
```rust
pub struct StatusBadge {
    status: String,
    color: Color,
}
```

**FormField** (2 hours):
```rust
pub struct FormField {
    label: String,
    value: String,
    is_focused: bool,
    validator: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
}
```

**Estimated Time**: 4-5 hours

---

### Phase 7: Refactor Calendar Component (8-10 hours)

#### 7.1 Extract State Structs (2-3 hours)

**Before** (20+ fields in one struct):
```rust
pub struct AppointmentCalendarComponent {
    // Calendar state
    current_date: NaiveDate,
    current_month_start: NaiveDate,
    selected_month_day: u32,
    view_mode: ViewMode,
    week_start_date: NaiveDate,
    
    // Detail modal state
    selected_appointment: Option<Uuid>,
    showing_detail_modal: bool,
    modal_patient: Option<Patient>,
    
    // Reschedule modal state
    showing_reschedule_modal: bool,
    reschedule_new_start_time: Option<DateTime<Utc>>,
    reschedule_new_duration: i64,
    reschedule_conflict_warning: Option<String>,
    
    // ... 10+ more fields
}
```

**After** (nested structs):
```rust
pub struct AppointmentCalendarComponent {
    calendar_state: CalendarState,
    modal_state: ModalState,
    filter_state: FilterState,
    history_state: HistoryState,
    
    // Services
    appointment_service: Arc<AppointmentService>,
    practitioner_service: Arc<PractitionerService>,
    patient_service: Arc<PatientService>,
}

struct CalendarState {
    current_date: NaiveDate,
    current_month_start: NaiveDate,
    selected_month_day: u32,
    view_mode: ViewMode,
    week_start_date: NaiveDate,
    time_slot_state: TableState,
    focus_area: FocusArea,
}

enum ModalState {
    None,
    Detail {
        appointment_id: Uuid,
        patient: Option<Patient>,
    },
    Reschedule {
        appointment_id: Uuid,
        new_start_time: Option<DateTime<Utc>>,
        new_duration: i64,
        conflict_warning: Option<String>,
    },
    Search {
        query: String,
        results: Vec<CalendarAppointment>,
        selected_index: usize,
    },
    Confirmation {
        message: String,
        pending_status: AppointmentStatus,
        pending_appointment_id: Uuid,
    },
    Error {
        message: String,
    },
    AuditHistory {
        entries: Vec<AuditEntry>,
        selected_index: usize,
    },
}

struct FilterState {
    active_status_filters: HashSet<AppointmentStatus>,
    active_practitioner_filters: HashSet<Uuid>,
    showing_filter_menu: bool,
    showing_practitioner_menu: bool,
}

struct HistoryState {
    recent_status_changes: Vec<(Uuid, AppointmentStatus)>,
    undo_timestamp: Option<DateTime<Utc>>,
}
```

**Estimated Time**: 2-3 hours

---

#### 7.2 Extract Render Methods (3-4 hours)

**Before** (20+ render methods scattered):
```rust
impl AppointmentCalendarComponent {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // 200+ lines of rendering logic
    }
    
    fn render_month_view(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    fn render_day_view(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    fn render_week_view(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    fn render_detail_modal(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    fn render_reschedule_modal(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    // ... 15+ more render methods
}
```

**After** (organized into renderer structs):
```rust
struct CalendarRenderer;
struct ModalRenderer;
struct FilterRenderer;

impl CalendarRenderer {
    fn render_month_view(
        state: &CalendarState,
        appointments: &[CalendarAppointment],
        frame: &mut Frame,
        area: Rect,
    ) {
        // Rendering logic
    }
    
    fn render_day_view(/* ... */) { /* ... */ }
    fn render_week_view(/* ... */) { /* ... */ }
}

impl ModalRenderer {
    fn render(
        modal_state: &ModalState,
        frame: &mut Frame,
        area: Rect,
    ) {
        match modal_state {
            ModalState::Detail { appointment_id, patient } => {
                Self::render_detail_modal(appointment_id, patient, frame, area)
            }
            ModalState::Reschedule { .. } => {
                Self::render_reschedule_modal(/* ... */)
            }
            // ... other modals
            ModalState::None => {}
        }
    }
}
```

**Estimated Time**: 3-4 hours

---

#### 7.3 Simplify Event Handling (2-3 hours)

**Before** (complex nested match):
```rust
fn handle_key_events(&mut self, key: KeyEvent) -> Action {
    if self.showing_detail_modal {
        // 50+ lines of modal event handling
    } else if self.showing_reschedule_modal {
        // 50+ lines of modal event handling
    } else if self.showing_search_modal {
        // 50+ lines of modal event handling
    } else {
        // 100+ lines of calendar event handling
    }
}
```

**After** (delegated to handlers):
```rust
fn handle_key_events(&mut self, key: KeyEvent) -> Action {
    match &self.modal_state {
        ModalState::None => self.handle_calendar_events(key),
        modal => self.handle_modal_events(key, modal),
    }
}

fn handle_calendar_events(&mut self, key: KeyEvent) -> Action {
    match self.calendar_state.focus_area {
        FocusArea::MonthView => self.handle_month_view_events(key),
        FocusArea::DayView => self.handle_day_view_events(key),
    }
}

fn handle_modal_events(&mut self, key: KeyEvent, modal: &ModalState) -> Action {
    match key.code {
        KeyCode::Esc => {
            self.modal_state = ModalState::None;
            Action::Render
        }
        _ => match modal {
            ModalState::Detail { .. } => self.handle_detail_modal_events(key),
            ModalState::Reschedule { .. } => self.handle_reschedule_modal_events(key),
            // ... other modals
            ModalState::None => Action::None,
        }
    }
}
```

**Estimated Time**: 2-3 hours

---

## Cross-Cutting Improvements

### Phase 8: Testing Infrastructure (6-8 hours)

#### 8.1 Create Mock Repository Implementations (3-4 hours)

**File to Create**: `src/infrastructure/database/mocks.rs`

```rust
//! Mock repository implementations for testing

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

pub struct MockPatientRepository {
    patients: Arc<Mutex<Vec<Patient>>>,
}

impl MockPatientRepository {
    pub fn new() -> Self {
        Self {
            patients: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn with_patients(patients: Vec<Patient>) -> Self {
        Self {
            patients: Arc::new(Mutex::new(patients)),
        }
    }
}

#[async_trait]
impl PatientRepository for MockPatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
        let patients = self.patients.lock().unwrap();
        Ok(patients.iter().find(|p| p.id == id).cloned())
    }
    
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        let mut patients = self.patients.lock().unwrap();
        patients.push(patient.clone());
        Ok(patient)
    }
    
    // ... other methods
}
```

**Create mocks for**:
- PatientRepository
- AppointmentRepository
- AuditRepository
- PrescriptionRepository
- ImmunisationRepository

**Estimated Time**: 3-4 hours

---

#### 8.2 Create Test Fixtures (2-3 hours)

**File to Create**: `src/infrastructure/fixtures/mod.rs`

```rust
//! Test fixture generators

pub mod patient_generator; // Already exists
pub mod appointment_generator;
pub mod prescription_generator;
pub mod immunisation_generator;

pub use patient_generator::PatientGenerator;
pub use appointment_generator::AppointmentGenerator;
pub use prescription_generator::PrescriptionGenerator;
pub use immunisation_generator::ImmunisationGenerator;
```

**Estimated Time**: 2-3 hours

---

#### 8.3 Create Assertion Helpers (1-2 hours)

**File to Create**: `tests/helpers/assertions.rs`

```rust
//! Test assertion helpers

pub fn assert_patient_eq(actual: &Patient, expected: &Patient) {
    assert_eq!(actual.id, expected.id);
    assert_eq!(actual.first_name, expected.first_name);
    assert_eq!(actual.last_name, expected.last_name);
    // ... other fields
}

pub fn assert_appointment_eq(actual: &Appointment, expected: &Appointment) {
    // Similar implementation
}
```

**Estimated Time**: 1-2 hours

---

### Phase 9: Documentation & Best Practices (4-5 hours)

#### 9.1 Update AGENTS.md (1-2 hours)

Add sections:
- Builder pattern usage
- Repository testing patterns
- Widget reusability guidelines
- State management best practices

**Estimated Time**: 1-2 hours

---

#### 9.2 Create ARCHITECTURE.md (2-3 hours)

Document:
- Layer responsibilities
- Dependency rules
- Common patterns (builder, repository, service)
- Testing strategies

**Estimated Time**: 2-3 hours

---

#### 9.3 Create INTEGRATIONS.md (1 hour)

Document:
- Integration client usage
- Error handling
- Retry logic
- Rate limiting

**Estimated Time**: 1 hour

---

## Implementation Roadmap

### Week 1-2: Domain Layer Foundation (20-25 hours)

**Priority**: Critical - Blocking new feature development

| Task | Estimated Time | Dependencies |
|------|----------------|--------------|
| 1.1 Implement Builder Pattern | 4-5 hours | None |
| 1.2 Complete Prescription Domain | 4-5 hours | 1.1 |
| 1.3 Complete Immunisation Domain | 4-5 hours | 1.1 |
| 1.4 Complete Remaining Domains | 8-10 hours | 1.1 |

**Deliverables**:
- ✅ All 6 domain modules complete (service, repository, error, dto)
- ✅ Builder pattern for Prescription and Immunisation
- ✅ Comprehensive error types
- ✅ DTOs for all domains

---

### Week 3-4: Infrastructure Layer Optimization (13-16 hours)

**Priority**: High - Reducing technical debt

| Task | Estimated Time | Dependencies |
|------|----------------|--------------|
| 3.1 Create Database Helpers | 3-4 hours | None |
| 3.2 Implement Enum Serialization | 2-3 hours | None |
| 4.1 Refactor Patient Repository | 2-3 hours | 3.1, 3.2 |
| 4.2 Refactor Appointment Repository | 3-4 hours | 3.1, 3.2 |
| 4.3 Refactor Audit Repository | 1-2 hours | 3.1, 3.2 |
| 4.4 Create Repository Test Utilities | 2-3 hours | 4.1-4.3 |

**Deliverables**:
- ✅ Database helpers module
- ✅ Strum-based enum serialization
- ✅ Refactored repositories (660+ lines removed)
- ✅ Repository test utilities

---

### Week 5-6: UI Layer Refactoring (18-22 hours)

**Priority**: Critical - Calendar component approaching unmaintainability

| Task | Estimated Time | Dependencies |
|------|----------------|--------------|
| 6.1 Create ListSelector Widget | 2-3 hours | None |
| 6.2 Create SearchFilter Widget | 2-3 hours | None |
| 6.3 Create ModalHandler Trait | 2-3 hours | None |
| 6.4 Create Additional Widgets | 4-5 hours | None |
| 7.1 Extract Calendar State Structs | 2-3 hours | 6.3 |
| 7.2 Extract Calendar Render Methods | 3-4 hours | 7.1 |
| 7.3 Simplify Calendar Event Handling | 2-3 hours | 7.1, 6.3 |

**Deliverables**:
- ✅ Reusable widget library (ListSelector, SearchFilter, ModalHandler, etc.)
- ✅ Refactored calendar component (3,164 → ~1,500 lines)
- ✅ Improved state management
- ✅ Simplified event handling

---

### Week 7-8: External Integrations (16-20 hours)

**Priority**: Medium - Required for production but not blocking current development

| Task | Estimated Time | Dependencies |
|------|----------------|--------------|
| 5.1 Create Integration Client Base | 3-4 hours | None |
| 5.2 Implement Medicare Online | 4-5 hours | 5.1 |
| 5.3 Implement PBS API | 3-4 hours | 5.1 |
| 5.4 Implement AIR | 3-4 hours | 5.1 |
| 5.5 Implement HI Service | 3-4 hours | 5.1 |

**Deliverables**:
- ✅ Base HTTP client with retry logic
- ✅ Medicare Online integration
- ✅ PBS API integration
- ✅ AIR integration
- ✅ HI Service integration

---

### Week 9-10: Testing & Documentation (10-13 hours)

**Priority**: High - Improving maintainability

| Task | Estimated Time | Dependencies |
|------|----------------|--------------|
| 8.1 Create Mock Repositories | 3-4 hours | Week 3-4 |
| 8.2 Create Test Fixtures | 2-3 hours | Week 1-2 |
| 8.3 Create Assertion Helpers | 1-2 hours | Week 1-2 |
| 9.1 Update AGENTS.md | 1-2 hours | All phases |
| 9.2 Create ARCHITECTURE.md | 2-3 hours | All phases |
| 9.3 Create INTEGRATIONS.md | 1 hour | Week 7-8 |

**Deliverables**:
- ✅ Mock repository implementations
- ✅ Comprehensive test fixtures
- ✅ Assertion helpers
- ✅ Updated documentation

---

## Success Metrics

### Code Quality Metrics

| Metric | Before | Target | Improvement |
|--------|--------|--------|-------------|
| Total Lines of Code | 14,686 | 13,186 | -1,500 lines (-10%) |
| Largest File | 3,164 lines | <1,500 lines | -1,664 lines (-53%) |
| Avg Component Size | 1,070 lines | <500 lines | -570 lines (-53%) |
| Code Duplication | ~15% | <5% | -10% |
| Test Coverage | ~20% | >60% | +40% |

### Developer Experience Metrics

| Metric | Before | Target | Improvement |
|--------|--------|--------|-------------|
| Time to Add New Domain | 8-10 hours | 4-5 hours | -50% |
| Time to Add New Component | 6-8 hours | 3-4 hours | -50% |
| Time to Add New Integration | N/A | 3-4 hours | Baseline |
| Repository Boilerplate | 660+ lines | <100 lines | -85% |

### Architecture Health

| Metric | Before | Target |
|--------|--------|--------|
| Complete Domain Modules | 4/10 (40%) | 10/10 (100%) |
| Reusable Widgets | 2 | 8+ |
| External Integrations | 0/4 (0%) | 4/4 (100%) |
| Mock Implementations | 1 | 5+ |

---

## Risk Assessment

### High Risk

**Calendar Component Refactoring** (Phase 7)
- **Risk**: Breaking existing functionality
- **Mitigation**: 
  - Comprehensive testing before refactoring
  - Incremental refactoring (state → render → events)
  - Feature flag for new implementation
  - Parallel implementation with gradual migration

### Medium Risk

**Repository Refactoring** (Phase 4)
- **Risk**: Database query regressions
- **Mitigation**:
  - Comprehensive integration tests
  - Test against production-like data
  - Gradual rollout (patient → audit → appointment)

**External Integrations** (Phase 5)
- **Risk**: API changes, rate limiting, authentication issues
- **Mitigation**:
  - Comprehensive error handling
  - Retry logic with exponential backoff
  - Circuit breaker pattern
  - Extensive logging

### Low Risk

**Domain Module Completion** (Phase 1)
- **Risk**: Minimal - following established patterns
- **Mitigation**: Use Appointment domain as template

**Widget Extraction** (Phase 6)
- **Risk**: Minimal - additive changes
- **Mitigation**: Gradual adoption in existing components

---

## Dependencies to Add

```toml
[dependencies]
# Existing dependencies remain...

# New dependencies for refactoring
derive_builder = "0.20"          # Builder pattern
strum = { version = "0.26", features = ["derive"] }
strum_macros = "0.26"            # Enum serialization
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
backoff = "0.4"                  # Retry logic

[dev-dependencies]
mockall = "0.13"                 # Mock generation
mockito = "1.5"                  # HTTP mocking
```

---

## Conclusion

This refactoring plan addresses the three major pain points in the OpenGP codebase:

1. **Domain Layer**: Complete 6 incomplete modules, implement builder pattern, extract duplicate code
2. **Infrastructure Layer**: Reduce 660+ lines of boilerplate, implement external integrations
3. **UI Layer**: Refactor 3,164-line calendar component, extract 500+ lines of widget duplication

**Total Effort**: 60-80 hours over 8-10 weeks  
**Expected Outcome**: 1,500+ lines removed, 50% faster development, 100% domain completion

The plan is structured to deliver value incrementally, with each phase building on the previous one. High-risk items (calendar refactoring) are scheduled later to allow for comprehensive testing and gradual migration.

**Next Steps**:
1. Review this plan with the team
2. Prioritize phases based on current development needs
3. Create GitHub issues for each task
4. Begin with Phase 1 (Domain Layer Foundation)
