# Clinical Tab Implementation Plan

**Project**: OpenGP - Open Source General Practice Management Software  
**Feature**: Clinical Record Management System  
**Document Version**: 1.0  
**Last Updated**: 2026-02-16  
**Status**: Planning

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Implementation Roadmap](#implementation-roadmap)
4. [Phase 1: Domain Layer](#phase-1-domain-layer)
5. [Phase 2: Repository Layer](#phase-2-repository-layer)
6. [Phase 3: Component Layer](#phase-3-component-layer)
7. [Phase 4: App Integration](#phase-4-app-integration)
8. [Detailed Component Design](#detailed-component-design)
9. [Security & Compliance](#security--compliance)
10. [Testing Strategy](#testing-strategy)
11. [Timeline & Effort](#timeline--effort)
12. [Gitea Issues](#gitea-issues)

---

## Executive Summary

This document provides a comprehensive implementation plan for the Clinical Tab feature in OpenGP. The clinical tab will provide general practitioners with a complete clinical record management system, including SOAP notes, allergies, medical history, vital signs, and family/social history tracking.

### Key Features

- **Patient Clinical Overview** - Central dashboard with patient demographics, allergies, conditions, and recent consultations
- **SOAP Notes Editor** - Full-screen editor for Subjective, Objective, Assessment, and Plan notes
- **Vital Signs Recording** - Track BP, HR, temperature, O2 sat, height/weight with auto-calculated BMI
- **Allergy Management** - Track allergens with severity levels and reactions
- **Medical History** - Active and historical medical conditions
- **Family History** - Track hereditary conditions by relative
- **Social History** - Document lifestyle factors (smoking, alcohol, exercise)

### Estimated Scope

- **Timeline**: 12-17 days
- **Effort**: ~4,000 lines of code
- **Files**: 20+ new files, 5+ modified files
- **Phases**: 4 implementation phases + testing

---

## Current State Analysis

### ✅ Already Implemented

#### Domain Models (`src/domain/clinical/model.rs`)
The following domain entities exist:

| Entity | Status | Description |
|--------|--------|-------------|
| `Consultation` | ✅ Complete | Clinical encounter with SOAP notes |
| `SOAPNotes` | ✅ Complete | Subjective, Objective, Assessment, Plan |
| `MedicalHistory` | ✅ Complete | Patient medical conditions |
| `Allergy` | ✅ Complete | Allergen tracking with severity |
| `VitalSigns` | ✅ Complete | BP, HR, temperature, O2, height, weight, BMI |
| `SocialHistory` | ✅ Complete | Smoking, alcohol, exercise, occupation |
| `FamilyHistory` | ✅ Complete | Family medical conditions |

#### Repository Traits (`src/domain/clinical/repository.rs`)
Partially implemented:
- ✅ `ConsultationRepository` - CRUD + sign operations
- ✅ `SocialHistoryRepository` - Find/create/update
- ❌ `AllergyRepository` - Missing
- ❌ `MedicalHistoryRepository` - Missing
- ❌ `VitalSignsRepository` - Missing
- ❌ `FamilyHistoryRepository` - Missing

#### Infrastructure (`src/infrastructure/database/repositories/clinical.rs`)
- ✅ `SqlxClinicalRepository` - Full implementation with encryption
- ✅ `SqlxSocialHistoryRepository` - Full implementation with encryption
- ❌ Missing: Allergy, MedicalHistory, VitalSigns, FamilyHistory implementations

#### Database Schema
- ✅ `consultations` table exists with encrypted SOAP fields
- ✅ `social_history` table exists
- ❌ Missing: `allergies`, `medical_history`, `vital_signs`, `family_history` tables

### ❌ Missing Components

1. **Service Layer** - No `ClinicalService` implementation
2. **DTOs** - No data transfer objects
3. **Error Types** - No clinical-specific errors
4. **UI Components** - Empty `components/clinical/mod.rs`
5. **Actions** - No clinical actions in `components/mod.rs`
6. **App Integration** - Clinical tab placeholder only

---

## Implementation Roadmap

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         UI Layer (Ratatui)                           │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │              ClinicalComponent (src/components/clinical/)        ││
│  │  • Patient Selector    • SOAP Editor    • Vital Signs Form      ││
│  │  • Patient Overview    • Allergy List   • History Views         ││
│  └─────────────────────────────────────────────────────────────────┘│
└────────────────────────────────┬────────────────────────────────────┘
                                 │
┌────────────────────────────────┴────────────────────────────────────┐
│                      Application Layer (App)                         │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  Action Routing • Component Lifecycle • State Management        ││
│  └─────────────────────────────────────────────────────────────────┘│
└────────────────────────────────┬────────────────────────────────────┘
                                 │
┌────────────────────────────────┴────────────────────────────────────┐
│                         Domain Layer                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ Consult- │ │ Allergy  │ │ Medical  │ │  Vital   │ │  Social  │  │
│  │  ation   │ │  Model   │ │ History  │ │  Signs   │ │ History  │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       │            │            │            │            │         │
│  ┌────┴────────────┴────────────┴────────────┴────────────┴────┐   │
│  │                   ClinicalService                             │   │
│  │  • Business Logic • Validation • Audit Logging               │   │
│  └───────────────────────────────────────────────────────────────┘   │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
┌────────────────────────────────┴────────────────────────────────────┐
│                      Repository Layer (SQLx)                         │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  SqlxClinicalRepository • SqlxAllergyRepository • etc.          ││
│  │  Encryption/Decryption • Audit Logging                          ││
│  └─────────────────────────────────────────────────────────────────┘│
└────────────────────────────────┬────────────────────────────────────┘
                                 │
┌────────────────────────────────┴────────────────────────────────────┐
│                      Database Layer (SQLite)                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │consulta- │ │allergies │ │ medical_ │ │  vital_  │ │  social_ │  │
│  │  tions   │ │          │ │ history  │ │  signs   │ │ history  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Domain Layer

**Gitea Issue**: [#52 - Clinical Tab: Phase 1 - Domain Layer Implementation](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/52)

**Duration**: 2-3 days  
**Effort**: ~600 lines of code

### Files to Create

#### 1. `src/domain/clinical/error.rs`
Clinical-specific error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Consultation not found: {0}")]
    ConsultationNotFound(Uuid),
    
    #[error("Patient not found: {0}")]
    PatientNotFound(Uuid),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Consultation already signed")]
    AlreadySigned,
}
```

#### 2. `src/domain/clinical/dto.rs`
Data transfer objects for all clinical operations:

```rust
// Consultation DTOs
pub struct NewConsultationData {
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub appointment_id: Option<Uuid>,
}

pub struct UpdateSOAPNotesData {
    pub subjective: Option<String>,
    pub objective: Option<String>,
    pub assessment: Option<String>,
    pub plan: Option<String>,
}

// Allergy DTOs
pub struct NewAllergyData {
    pub patient_id: Uuid,
    pub allergen: String,
    pub allergy_type: AllergyType,
    pub severity: Severity,
    pub reaction: Option<String>,
    pub onset_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

// Medical History DTOs
pub struct NewMedicalHistoryData {
    pub patient_id: Uuid,
    pub condition: String,
    pub diagnosis_date: Option<NaiveDate>,
    pub status: ConditionStatus,
    pub severity: Option<Severity>,
    pub notes: Option<String>,
}

// Vital Signs DTOs
pub struct NewVitalSignsData {
    pub patient_id: Uuid,
    pub consultation_id: Option<Uuid>,
    pub systolic_bp: Option<u16>,
    pub diastolic_bp: Option<u16>,
    pub heart_rate: Option<u16>,
    pub respiratory_rate: Option<u16>,
    pub temperature: Option<f32>,
    pub oxygen_saturation: Option<u8>,
    pub height_cm: Option<u16>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
}

// Social History DTOs
pub struct UpdateSocialHistoryData {
    pub smoking_status: SmokingStatus,
    pub cigarettes_per_day: Option<u8>,
    pub smoking_quit_date: Option<NaiveDate>,
    pub alcohol_status: AlcoholStatus,
    pub standard_drinks_per_week: Option<u8>,
    pub exercise_frequency: Option<ExerciseFrequency>,
    pub occupation: Option<String>,
    pub living_situation: Option<String>,
    pub support_network: Option<String>,
    pub notes: Option<String>,
}

// Family History DTOs
pub struct NewFamilyHistoryData {
    pub patient_id: Uuid,
    pub relative_relationship: String,
    pub condition: String,
    pub age_at_diagnosis: Option<u8>,
    pub notes: Option<String>,
}
```

#### 3. `src/domain/clinical/service.rs`
Main clinical service with business logic:

```rust
pub struct ClinicalService {
    consultation_repo: Arc<dyn ConsultationRepository>,
    allergy_repo: Arc<dyn AllergyRepository>,
    medical_history_repo: Arc<dyn MedicalHistoryRepository>,
    vital_signs_repo: Arc<dyn VitalSignsRepository>,
    social_history_repo: Arc<dyn SocialHistoryRepository>,
    family_history_repo: Arc<dyn FamilyHistoryRepository>,
    patient_service: Arc<PatientService>,
    audit_logger: Arc<AuditService>,
    crypto: Arc<EncryptionService>,
}

impl ClinicalService {
    // Consultation Management
    pub async fn create_consultation(&self, data: NewConsultationData, user_id: Uuid) 
        -> Result<Consultation, ServiceError>;
    
    pub async fn find_consultation(&self, id: Uuid) 
        -> Result<Option<Consultation>, ServiceError>;
    
    pub async fn list_patient_consultations(&self, patient_id: Uuid) 
        -> Result<Vec<Consultation>, ServiceError>;
    
    pub async fn update_soap_notes(&self, consultation_id: Uuid, data: UpdateSOAPNotesData, user_id: Uuid) 
        -> Result<Consultation, ServiceError>;
    
    pub async fn sign_consultation(&self, consultation_id: Uuid, user_id: Uuid) 
        -> Result<(), ServiceError>;
    
    // Allergy Management
    pub async fn add_allergy(&self, data: NewAllergyData, user_id: Uuid) 
        -> Result<Allergy, ServiceError>;
    
    pub async fn list_patient_allergies(&self, patient_id: Uuid, active_only: bool) 
        -> Result<Vec<Allergy>, ServiceError>;
    
    pub async fn deactivate_allergy(&self, allergy_id: Uuid, user_id: Uuid) 
        -> Result<(), ServiceError>;
    
    // Medical History
    pub async fn add_medical_history(&self, data: NewMedicalHistoryData, user_id: Uuid) 
        -> Result<MedicalHistory, ServiceError>;
    
    pub async fn list_medical_history(&self, patient_id: Uuid, active_only: bool) 
        -> Result<Vec<MedicalHistory>, ServiceError>;
    
    pub async fn update_condition_status(&self, history_id: Uuid, status: ConditionStatus, user_id: Uuid) 
        -> Result<MedicalHistory, ServiceError>;
    
    // Vital Signs
    pub async fn record_vital_signs(&self, data: NewVitalSignsData, user_id: Uuid) 
        -> Result<VitalSigns, ServiceError>;
    
    pub async fn get_latest_vital_signs(&self, patient_id: Uuid) 
        -> Result<Option<VitalSigns>, ServiceError>;
    
    pub async fn list_vital_signs_history(&self, patient_id: Uuid, limit: usize) 
        -> Result<Vec<VitalSigns>, ServiceError>;
    
    // Social History
    pub async fn update_social_history(&self, patient_id: Uuid, data: UpdateSocialHistoryData, user_id: Uuid) 
        -> Result<SocialHistory, ServiceError>;
    
    pub async fn get_social_history(&self, patient_id: Uuid) 
        -> Result<Option<SocialHistory>, ServiceError>;
    
    // Family History
    pub async fn add_family_history(&self, data: NewFamilyHistoryData, user_id: Uuid) 
        -> Result<FamilyHistory, ServiceError>;
    
    pub async fn list_family_history(&self, patient_id: Uuid) 
        -> Result<Vec<FamilyHistory>, ServiceError>;
    
    pub async fn delete_family_history(&self, history_id: Uuid, user_id: Uuid) 
        -> Result<(), ServiceError>;
}
```

#### 4. Update `src/domain/clinical/mod.rs`

```rust
mod dto;
mod error;
mod model;
mod repository;
mod service;

pub use dto::*;
pub use error::*;
pub use model::*;
pub use repository::*;
pub use service::*;
```

---

## Phase 2: Repository Layer

**Gitea Issue**: [#53 - Clinical Tab: Phase 2 - Repository Layer Implementation](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/53)

**Duration**: 3-4 days  
**Effort**: ~1,000 lines of code

### Files to Modify/Create

#### 1. Update `src/domain/clinical/repository.rs`

Add missing repository traits:

```rust
#[async_trait]
pub trait AllergyRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Allergy>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Allergy>, RepositoryError>;
    async fn find_active_by_patient(&self, patient_id: Uuid) -> Result<Vec<Allergy>, RepositoryError>;
    async fn create(&self, allergy: Allergy) -> Result<Allergy, RepositoryError>;
    async fn update(&self, allergy: Allergy) -> Result<Allergy, RepositoryError>;
    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait MedicalHistoryRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MedicalHistory>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<MedicalHistory>, RepositoryError>;
    async fn find_active_by_patient(&self, patient_id: Uuid) -> Result<Vec<MedicalHistory>, RepositoryError>;
    async fn create(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError>;
    async fn update(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError>;
}

#[async_trait]
pub trait VitalSignsRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VitalSigns>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid, limit: usize) -> Result<Vec<VitalSigns>, RepositoryError>;
    async fn find_latest_by_patient(&self, patient_id: Uuid) -> Result<Option<VitalSigns>, RepositoryError>;
    async fn create(&self, vitals: VitalSigns) -> Result<VitalSigns, RepositoryError>;
}

#[async_trait]
pub trait FamilyHistoryRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FamilyHistory>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<FamilyHistory>, RepositoryError>;
    async fn create(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError>;
    async fn update(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}
```

#### 2. Create `migrations/20260217_clinical_tables.sql`

```sql
-- allergies table
CREATE TABLE IF NOT EXISTS allergies (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    allergen TEXT NOT NULL,
    allergy_type TEXT NOT NULL CHECK(allergy_type IN ('Drug', 'Food', 'Environmental', 'Other')),
    severity TEXT NOT NULL CHECK(severity IN ('Mild', 'Moderate', 'Severe')),
    reaction TEXT,
    onset_date DATE,
    notes BLOB, -- Encrypted
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    updated_by BLOB,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_allergies_patient ON allergies(patient_id);
CREATE INDEX idx_allergies_active ON allergies(patient_id, is_active) WHERE is_active = TRUE;

-- medical_history table
CREATE TABLE IF NOT EXISTS medical_history (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    condition TEXT NOT NULL,
    diagnosis_date DATE,
    status TEXT NOT NULL CHECK(status IN ('Active', 'Resolved', 'Chronic', 'Recurring', 'InRemission')),
    severity TEXT CHECK(severity IN ('Mild', 'Moderate', 'Severe')),
    notes BLOB, -- Encrypted
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    updated_by BLOB,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_medical_history_patient ON medical_history(patient_id);
CREATE INDEX idx_medical_history_active ON medical_history(patient_id, is_active) WHERE is_active = TRUE;

-- vital_signs table
CREATE TABLE IF NOT EXISTS vital_signs (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    consultation_id BLOB,
    measured_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    systolic_bp INTEGER,
    diastolic_bp INTEGER,
    heart_rate INTEGER,
    respiratory_rate INTEGER,
    temperature REAL,
    oxygen_saturation INTEGER,
    height_cm INTEGER,
    weight_kg REAL,
    bmi REAL,
    notes BLOB, -- Encrypted
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (consultation_id) REFERENCES consultations(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_vital_signs_patient ON vital_signs(patient_id);
CREATE INDEX idx_vital_signs_measured ON vital_signs(patient_id, measured_at);

-- family_history table
CREATE TABLE IF NOT EXISTS family_history (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    relative_relationship TEXT NOT NULL,
    condition TEXT NOT NULL,
    age_at_diagnosis INTEGER,
    notes BLOB, -- Encrypted
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BLOB NOT NULL,
    updated_by BLOB,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

CREATE INDEX idx_family_history_patient ON family_history(patient_id);
```

#### 3. Update `src/infrastructure/database/repositories/clinical.rs`

Add implementations for:
- `SqlxAllergyRepository`
- `SqlxMedicalHistoryRepository`
- `SqlxVitalSignsRepository`
- `SqlxFamilyHistoryRepository`

Follow the existing pattern from `SqlxClinicalRepository` with encryption for sensitive fields.

---

## Phase 3: Component Layer

**Gitea Issue**: [#54 - Clinical Tab: Phase 3 - UI Component Implementation](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/54)

**Duration**: 5-7 days  
**Effort**: ~2,000 lines of code

### Component Structure

```
src/components/clinical/
├── mod.rs                    # Module exports
├── component.rs              # Main ClinicalComponent (500 LOC)
├── state.rs                  # State management (200 LOC)
├── patient_selector.rs       # Patient search/selection (250 LOC)
├── patient_overview.rs       # Clinical summary view (300 LOC)
├── consultation_list.rs      # List consultations (200 LOC)
├── consultation_form.rs      # SOAP editor (400 LOC)
├── allergy_list.rs           # View allergies (150 LOC)
├── allergy_form.rs           # Add/edit allergy (150 LOC)
├── vital_signs_form.rs       # Record vitals (200 LOC)
├── medical_history_list.rs   # View conditions (100 LOC)
├── medical_history_form.rs   # Add/edit conditions (100 LOC)
├── family_history_list.rs    # View family history (100 LOC)
├── family_history_form.rs    # Add/edit family history (100 LOC)
├── social_history_form.rs    # Edit social history (150 LOC)
└── renderers.rs              # UI helpers (100 LOC)
```

### Key Files

#### 1. `src/components/clinical/component.rs`

Main component implementing the `Component` trait:

```rust
pub struct ClinicalComponent {
    clinical_service: Arc<ClinicalService>,
    patient_service: Arc<PatientService>,
    
    current_patient: Option<Patient>,
    current_view: ClinicalView,
    
    // Data caches
    consultations: Vec<Consultation>,
    allergies: Vec<Allergy>,
    medical_history: Vec<MedicalHistory>,
    family_history: Vec<FamilyHistory>,
    social_history: Option<SocialHistory>,
    latest_vitals: Option<VitalSigns>,
    
    // Form states
    soap_editor: SOAPEditorState,
    vital_signs_form: VitalSignsFormState,
    allergy_form: AllergyFormState,
    medical_history_form: MedicalHistoryFormState,
    family_history_form: FamilyHistoryFormState,
    social_history_form: SocialHistoryFormState,
    
    // Patient selector
    patient_search: String,
    patient_search_results: Vec<Patient>,
    patient_selector_open: bool,
    
    // UI state
    modal_state: ModalState,
    error_message: Option<String>,
    showing_help: bool,
}

pub enum ClinicalView {
    PatientSelector,                // Search/select patient
    PatientOverview,                // Clinical summary
    ConsultationList,               // List of consultations
    ConsultationEditor(Uuid),       // SOAP editor
    AllergyList,                    // All allergies
    AllergyEditor(Option<Uuid>),    // Add/edit allergy
    MedicalHistoryList,             // Medical conditions
    MedicalHistoryEditor(Option<Uuid>),
    FamilyHistoryList,              // Family history
    FamilyHistoryEditor(Option<Uuid>),
    SocialHistoryEditor,            // Social history
    VitalSignsEditor,               // Record vitals
}
```

#### 2. `src/components/mod.rs` - Add Actions

```rust
pub enum Action {
    // ... existing actions ...
    
    // Clinical navigation
    ClinicalPatientSelect(Uuid),
    ClinicalPatientClear,
    
    // Consultation actions
    ClinicalConsultationCreate(Uuid),
    ClinicalConsultationEdit(Uuid),
    ClinicalConsultationSign(Uuid),
    ClinicalConsultationSave(Uuid),
    ClinicalConsultationCancel,
    
    // Allergy actions
    ClinicalAllergyAdd(Uuid),
    ClinicalAllergyEdit(Uuid),
    ClinicalAllergyDeactivate(Uuid),
    ClinicalAllergySave,
    ClinicalAllergyCancel,
    
    // Vital signs actions
    ClinicalVitalSignsRecord(Uuid),
    ClinicalVitalSignsSave,
    ClinicalVitalSignsCancel,
    
    // History actions
    ClinicalMedicalHistoryAdd(Uuid),
    ClinicalMedicalHistoryEdit(Uuid),
    ClinicalMedicalHistorySave,
    ClinicalMedicalHistoryCancel,
    
    ClinicalFamilyHistoryAdd(Uuid),
    ClinicalFamilyHistoryEdit(Uuid),
    ClinicalFamilyHistoryDelete(Uuid),
    ClinicalFamilyHistorySave,
    ClinicalFamilyHistoryCancel,
    
    ClinicalSocialHistoryEdit(Uuid),
    ClinicalSocialHistorySave,
    ClinicalSocialHistoryCancel,
    
    // View mode actions
    ClinicalShowOverview,
    ClinicalShowConsultations,
    ClinicalShowAllergies,
    ClinicalShowMedicalHistory,
    ClinicalShowFamilyHistory,
    ClinicalShowSocialHistory,
}
```

---

## Phase 4: App Integration

**Gitea Issue**: [#55 - Clinical Tab: Phase 4 - App Integration and Finalization](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/55)

**Duration**: 2-3 days  
**Effort**: ~200 lines of code

### Modifications to `src/app.rs`

#### 1. Add Clinical Service

```rust
pub struct App {
    config: Config,
    db_pool: SqlitePool,
    patient_service: Arc<PatientService>,
    appointment_service: Arc<AppointmentService>,
    clinical_service: Arc<ClinicalService>,  // NEW
    practitioner_service: Arc<PractitionerService>,
    should_quit: bool,
    active_screen: Screen,
    patient_component: Option<Box<dyn Component>>,
    patient_form_component: Option<Box<dyn Component>>,
    appointment_component: Option<Box<dyn Component>>,
    appointment_form_component: Option<Box<dyn Component>>,
    clinical_component: Option<Box<dyn Component>>,  // NEW
    billing_component: Option<Box<dyn Component>>,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    showing_form: bool,
    tabs_area: Option<Rect>,
}
```

#### 2. Initialize Clinical Service

```rust
impl App {
    pub fn new(config: Config, db_pool: SqlitePool) -> Result<Self> {
        // ... existing service initialization ...
        
        // Initialize clinical repositories
        let clinical_repository = Arc::new(SqlxClinicalRepository::new(
            db_pool.clone(),
            crypto.clone(),
        ));
        let allergy_repository = Arc::new(SqlxAllergyRepository::new(
            db_pool.clone(),
            crypto.clone(),
        ));
        let medical_history_repository = Arc::new(SqlxMedicalHistoryRepository::new(
            db_pool.clone(),
            crypto.clone(),
        ));
        let vital_signs_repository = Arc::new(SqlxVitalSignsRepository::new(
            db_pool.clone(),
        ));
        let social_history_repository = Arc::new(SqlxSocialHistoryRepository::new(
            db_pool.clone(),
            crypto.clone(),
        ));
        let family_history_repository = Arc::new(SqlxFamilyHistoryRepository::new(
            db_pool.clone(),
            crypto.clone(),
        ));
        
        // Initialize clinical service
        let clinical_service = Arc::new(ClinicalService::new(
            clinical_repository,
            allergy_repository,
            medical_history_repository,
            vital_signs_repository,
            social_history_repository,
            family_history_repository,
            patient_service.clone(),
            audit_service.clone(),
            crypto.clone(),
        ));
        
        Ok(Self {
            // ... existing fields ...
            clinical_service,
            clinical_component: None,
            // ... other fields ...
        })
    }
}
```

#### 3. Initialize Clinical Component

```rust
async fn init_components(&mut self) -> Result<()> {
    // ... existing component initialization ...
    
    let mut clinical_component = ClinicalComponent::new(
        self.clinical_service.clone(),
        self.patient_service.clone(),
    );
    clinical_component.init().await?;
    self.clinical_component = Some(Box::new(clinical_component));
    
    Ok(())
}
```

#### 4. Update get_active_component_mut()

```rust
fn get_active_component_mut(&mut self) -> Option<&mut Box<dyn Component>> {
    match self.active_screen {
        Screen::Patients => self.patient_component.as_mut(),
        Screen::Appointments => self.appointment_component.as_mut(),
        Screen::Clinical => self.clinical_component.as_mut(),  // NEW
        Screen::Billing => self.billing_component.as_mut(),
    }
}
```

---

## Detailed Component Design

### Patient Overview Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ Clinical Record: SMITH, John                                 [#52]  │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─ Patient Summary ───────────┐  ┌─ Recent Consultations ───────┐  │
│  │ DOB: 15/03/1965 (60y)      │  │ Date       │ Status    │ Type│  │
│  │ Medicare: 1234567890-1      │  │────────────┼───────────┼─────│  │
│  │ IHI: 8003xxxxxxxxxxxx       │  │ 16/02/2026 │ Signed    │ Std │  │
│  │                             │  │ 10/01/2026 │ Signed    │ Lng │  │
│  │ ⚠️ ALLERGIES:               │  │ 05/12/2025 │ Signed    │ Std │  │
│  │ • Penicillin (Severe)       │  │            │           │     │  │
│  │ • Sulfa (Moderate)          │  │            │           │     │  │
│  │                             │  │            │           │     │  │
│  │ ACTIVE CONDITIONS:          │  │            │           │     │  │
│  │ • Type 2 Diabetes (Chronic) │  │            │           │     │  │
│  │ • Hypertension (Chronic)    │  │            │           │     │  │
│  │                             │  └──────────────────────────────┘  │
│  │ LATEST VITALS (15/02/2026): │                                     │
│  │ BP: 140/90  HR: 72  Temp: 37│                                     │
│  └─────────────────────────────┘                                     │
├─────────────────────────────────────────────────────────────────────┤
│  [F1] New Consultation  [F2] Vitals  [F3] History  [F4] Allergies   │
└─────────────────────────────────────────────────────────────────────┘
```

### SOAP Editor Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ SOAP Notes - Consultation: 16/02/2026                        [#53]  │
├─────────────────────────────────────────────────────────────────────┤
│  SUBJECTIVE (Patient's symptoms, complaints, history)               │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ Pt reports increased thirst and polyuria for 2 weeks.         │  │
│  │ No chest pain, SOB, or visual changes.                        │  │
│  │ FHx: Father with T2DM.                                        │  │
│  │                                                                 │  │
│  │                                                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  OBJECTIVE (Physical examination, investigations)                   │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ General: Well, alert, not distressed                          │  │
│  │ Vitals: BP 140/90, HR 72, Temp 37.0, BMI 28.5                 │  │
│  │ Examination: Chest clear, heart sounds normal                 │  │
│  │                                                                 │  │
│  │                                                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ASSESSMENT (Diagnosis, differential diagnosis)                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 1. Type 2 Diabetes Mellitus - poor glycaemic control          │  │
│  │ 2. Hypertension - suboptimally controlled                     │  │
│  │                                                                 │  │
│  │                                                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  PLAN (Management, prescriptions, referrals, follow-up)             │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ • Increase metformin to 1000mg BD                             │  │
│  │ • Add empagliflozin 10mg daily                                │  │
│  │ • FBC, UEC, HbA1c, lipid profile in 3 months                  │  │
│  │ • Review BP in 2 weeks                                        │  │
│  │ • Diabetes educator referral                                  │  │
│  │                                                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│  [Ctrl+S] Save Draft    [Ctrl+X] Sign & Finalize    [Esc] Cancel    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Security & Compliance

### Australian Healthcare Compliance

| Requirement | Implementation |
|-------------|----------------|
| **Privacy Act 1988 (APP 11)** | All patient data access audited |
| **Encryption at Rest** | AES-256-GCM for all clinical notes |
| **Audit Logging** | Every read/write operation logged |
| **Immutable Records** | Signed consultations cannot be altered |
| **Access Control** | Role-based permissions enforced |
| **My Health Records Act 2012** | Structure supports future MHR integration |
| **Healthcare Identifiers Act 2010** | IHI field present on patient records |

### Encryption Requirements

All sensitive fields encrypted using `EncryptionService`:

| Entity | Encrypted Fields |
|--------|------------------|
| Consultation | subjective, objective, assessment, plan (SOAP notes) |
| Allergy | notes |
| MedicalHistory | notes |
| FamilyHistory | notes |
| SocialHistory | notes |
| VitalSigns | notes |

### Audit Trail

All operations logged via `AuditService`:
- Consultation created, updated, signed, viewed
- SOAP notes updated
- Allergy added, updated, deactivated
- Medical history added, updated
- Vital signs recorded
- Family history added, updated, deleted
- Social history updated

---

## Testing Strategy

### Unit Tests

**Domain Layer:**
- [ ] Clinical service validation logic
- [ ] DTO validation rules
- [ ] Error type conversions

**Repository Layer:**
- [ ] Mock repository implementations
- [ ] Encryption/decryption roundtrip
- [ ] UUID and DateTime conversions

**Component Layer:**
- [ ] State management transitions
- [ ] Action handling
- [ ] Input validation

### Integration Tests

- [ ] Database CRUD operations
- [ ] Service layer with real repositories
- [ ] Component integration with services
- [ ] End-to-end workflows

### Manual Testing Checklist

#### Patient Selection
- [ ] Navigate to Clinical tab (press `3`)
- [ ] Search for patient by name
- [ ] Search for patient by Medicare number
- [ ] Select patient - overview loads correctly

#### Consultation Workflow
- [ ] Create new consultation
- [ ] Fill in all SOAP sections
- [ ] Save draft
- [ ] Edit draft
- [ ] Sign consultation
- [ ] Verify signed consultation is read-only

#### Allergy Management
- [ ] View allergies in patient overview
- [ ] Add new allergy with all fields
- [ ] Verify severity badges display correctly
- [ ] Deactivate allergy
- [ ] Verify deactivated allergies hidden by default

#### Vital Signs
- [ ] Record vital signs
- [ ] Verify BMI auto-calculated from height/weight
- [ ] View latest vitals in overview
- [ ] View vitals history

#### Medical History
- [ ] Add medical condition
- [ ] Update condition status
- [ ] View active conditions in overview
- [ ] View full medical history

#### Family History
- [ ] Add family history entry
- [ ] View family history list
- [ ] Edit family history
- [ ] Delete family history entry

#### Social History
- [ ] View social history
- [ ] Edit all fields
- [ ] Save changes
- [ ] Verify changes persisted

---

## Timeline & Effort

### Implementation Phases

| Phase | Duration | Files | LOC | Complexity |
|-------|----------|-------|-----|------------|
| Phase 1: Domain Layer | 2-3 days | 4 | ~600 | Medium |
| Phase 2: Repository Layer | 3-4 days | 2 | ~1,000 | High |
| Phase 3: UI Components | 5-7 days | 14 | ~2,000 | High |
| Phase 4: App Integration | 2-3 days | 3 | ~200 | Medium |
| Testing & Polish | 2-3 days | - | - | Medium |
| **Total** | **14-20 days** | **23** | **~3,800** | **High** |

### Resource Requirements

- **Developer**: 1 experienced Rust developer
- **Code Review**: Recommended for each phase
- **Testing**: Unit tests + manual testing
- **Documentation**: Wiki updates

---

## Gitea Issues

All work is tracked in Gitea with the following issues:

| Issue | Title | Priority | Duration |
|-------|-------|----------|----------|
| [#52](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/52) | Phase 1 - Domain Layer Implementation | High | 2-3 days |
| [#53](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/53) | Phase 2 - Repository Layer Implementation | High | 3-4 days |
| [#54](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/54) | Phase 3 - UI Component Implementation | High | 5-7 days |
| [#55](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/55) | Phase 4 - App Integration and Finalization | High | 2-3 days |
| [#56](https://gitea.snares-kitchen.ts.net/stephenp/opengp/issues/56) | Epic - Complete Clinical Record Management System | Epic | 12-17 days |

---

## References

### Internal Documentation
- [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture
- [REQUIREMENTS.md](../REQUIREMENTS.md) - Clinical requirements
- [AGENTS.md](../AGENTS.md) - Development guidelines

### External References
- [Australian Privacy Principles](https://www.oaic.gov.au/privacy/australian-privacy-principles)
- [RACGP Standards 5th Edition](https://www.racgp.org.au/running-a-practice/practice-standards/standards-5th-edition)
- [Ratatui Documentation](https://ratatui.rs/)
- [SQLx Documentation](https://docs.rs/sqlx/)

---

## Appendix: Key Bindings Reference

### Clinical Tab Navigation

| Key | Action | Context |
|-----|--------|---------|
| `3` | Navigate to Clinical | Global |
| `F1` | New Consultation | Patient Overview |
| `F2` | Record Vital Signs | Patient Overview |
| `F3` | View History | Patient Overview |
| `F4` | Manage Allergies | Patient Overview |
| `F5` | Manage Medical History | Patient Overview |
| `/` | Search Patient | Patient Selector |
| `↑/↓` or `j/k` | Navigate lists | All lists |
| `Enter` | Select/Edit item | All lists |
| `Esc` | Cancel/Go back | All views |
| `?` | Show help | All views |

### SOAP Editor

| Key | Action |
|-----|--------|
| `Tab` | Move to next section |
| `Shift+Tab` | Move to previous section |
| `Ctrl+S` | Save draft |
| `Ctrl+X` | Sign consultation |
| `Esc` | Cancel (with confirmation) |

---

**Document End**

*This implementation plan was generated for the OpenGP project on 2026-02-16.*
