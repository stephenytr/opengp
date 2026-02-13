# OpenGP: Strategic Development Plan - Next Steps

**Date**: 2026-02-13  
**Based On**: REQUIREMENTS.md Phase 1-5 Analysis  
**Current State**: Phase 1 Complete + Early Phase 2 Work  
**Status**: 🔵 Ready for Execution

---

## 📊 Current State Assessment

### ✅ What's Complete (Phase 1 - Foundation)

**Core Infrastructure:**
- [x] Project architecture and module structure
- [x] Database schema (SQLite with SQLx)
- [x] Migration framework (sqlx-cli)
- [x] Configuration management
- [x] Error handling framework
- [x] Encryption service (AES-256-GCM)
- [x] Audit logging foundation (domain + repository)
- [x] Basic TUI framework (Ratatui + component architecture)

**Domain Models (Skeleton):**
- [x] Patient domain (model, service, repository, DTO)
- [x] Appointment domain (model, service, repository, DTO, query layer)
- [x] User domain (model, service, repository)
- [x] Audit domain (model, service, repository)
- [x] Clinical/Billing/Prescription/Referral/Immunisation/Pathology (models only - no service/repository yet)

**UI Components:**
- [x] Patient list and form components
- [x] Appointment calendar (Day/Week/Month views)
- [x] Appointment list component
- [x] Advanced calendar features (search, filters, status updates, multi-select)

**Testing:**
- [x] Unit test framework
- [x] Integration test framework
- [x] Patient creation tests
- [x] Appointment status tests (currently broken - see issues)

### 🚧 Partially Complete (Early Phase 2)

**Appointment Scheduling:**
- [x] Calendar views (Day/Week/Month)
- [x] Multi-practitioner support (data model ready)
- [x] Appointment types and durations
- [x] Advanced calendar features (search, filters, audit history)
- [ ] Waitlist management (NOT YET IMPLEMENTED)
- [ ] SMS/email reminders (NOT YET IMPLEMENTED - requires external integration)

### 🔴 Critical Issues to Fix First

1. **Test Suite Broken** - Appointment tests failing due to service constructor changes
2. **Patient Names Not Displaying** - Fixed in last commit but needs verification
3. **Missing Patient Repository Implementation** - Service exists but SQLx repository incomplete
4. **No Authentication System** - User domain exists but auth not implemented

---

## 🎯 Strategic Priorities (Next 3-6 Months)

### Priority 1: Stabilize Foundation (2-3 weeks)

**Goal**: Fix broken tests, complete Phase 1 properly, get to "green build"

### Priority 2: Complete Phase 2 (2-3 months)

**Goal**: Fully functional clinical workflow - MVP usable for consultations

### Priority 3: Begin Phase 3 (Prescribing & Billing) (2-3 months)

**Goal**: Revenue-generating features - critical for practice adoption

---

## 📋 Detailed Implementation Plan

---

## Phase 1.5: Foundation Stabilization (IMMEDIATE - 2-3 Weeks)

### Block 1: Fix Broken Tests & Build ⚠️ CRITICAL
**Priority**: P0 - Must Do First  
**Estimated Time**: 2-3 days

#### Task 1.5.1: Fix Appointment Service Tests
- [ ] Update `tests/appointment_status_test.rs` to provide third parameter (AppointmentCalendarQuery)
- [ ] Review all test files for similar constructor breakages
- [ ] Run `cargo test` - must be 100% green
- [ ] **Files**: `tests/appointment_status_test.rs`, possibly others
- **Verification**: `cargo test` passes with zero failures

#### Task 1.5.2: Verify Patient Name Display Fix
- [ ] Run application: `cargo run`
- [ ] Navigate to appointment calendar
- [ ] Confirm patient names display (not UUIDs)
- [ ] Test edge cases: appointments with deleted patients
- [ ] **Manual testing + screenshots**
- **Verification**: Visual confirmation

#### Task 1.5.3: Run Full Quality Check
- [ ] `cargo check` - zero errors
- [ ] `cargo clippy -- -D warnings` - zero warnings
- [ ] `cargo fmt -- --check` - properly formatted
- [ ] `cargo test` - all tests pass
- **Verification**: All quality gates pass

---

### Block 2: Complete Patient Domain Implementation
**Priority**: P0 - Foundation Layer  
**Estimated Time**: 3-5 days

#### Task 1.5.4: Complete SqlxPatientRepository
- [ ] Review `src/infrastructure/database/repositories/patient.rs`
- [ ] Implement all PatientRepository trait methods:
  - [ ] `find_by_id()` - retrieve patient by UUID
  - [ ] `find_by_medicare()` - search by Medicare number
  - [ ] `search()` - search by name, DOB, Medicare
  - [ ] `create()` - insert new patient
  - [ ] `update()` - update existing patient
  - [ ] `delete()` - soft delete (set is_active = false)
- [ ] Use `sqlx::query_as!` for type safety
- [ ] Handle encryption for sensitive fields (if needed)
- [ ] Add proper error handling and logging
- **Files**: `src/infrastructure/database/repositories/patient.rs`
- **Verification**: `cargo check`, unit tests

#### Task 1.5.5: Add Patient Repository Integration Tests
- [ ] Create `tests/patient_repository_test.rs`
- [ ] Test all CRUD operations
- [ ] Test search functionality
- [ ] Test duplicate detection (Medicare number)
- [ ] Test soft delete behavior
- [ ] Use in-memory SQLite for tests
- **Files**: `tests/patient_repository_test.rs` (new)
- **Verification**: `cargo test patient_repository`

#### Task 1.5.6: Complete Patient Service Implementation
- [ ] Review `src/domain/patient/service.rs`
- [ ] Implement all business logic methods:
  - [ ] `register_patient()` - create with validation
  - [ ] `update_patient_details()` - update with audit
  - [ ] `find_patient()` - retrieve with access logging
  - [ ] `search_patients()` - search with permissions check
  - [ ] `deactivate_patient()` - soft delete with audit
- [ ] Add duplicate detection logic
- [ ] Integrate audit logging for all operations
- [ ] Add validation rules (names, DOB, Medicare format)
- **Files**: `src/domain/patient/service.rs`
- **Verification**: `cargo check`, service tests

---

### Block 3: Implement Authentication System
**Priority**: P0 - Security Foundation  
**Estimated Time**: 5-7 days

#### Task 1.5.7: Design Authentication Architecture
- [ ] Review REQUIREMENTS.md security section
- [ ] Design session management strategy
- [ ] Choose password hashing library (argon2 recommended)
- [ ] Design MFA strategy (TOTP with google-authenticator crate)
- [ ] Document in ARCHITECTURE.md
- **Deliverable**: Architecture decision record
- **Verification**: Technical review

#### Task 1.5.8: Implement Password Authentication
- [ ] Create `src/infrastructure/auth/password.rs`
- [ ] Implement password hashing with argon2
- [ ] Add password validation rules:
  - Minimum 12 characters
  - Complexity requirements (upper, lower, number, special)
  - Password history (prevent reuse of last 10)
- [ ] Implement password expiry (90 days configurable)
- [ ] Add account lockout (5 failed attempts, 15 min lockout)
- **Files**: `src/infrastructure/auth/password.rs` (new)
- **Verification**: Unit tests

#### Task 1.5.9: Implement Session Management
- [ ] Create `src/infrastructure/auth/session.rs`
- [ ] Design session storage (in-memory with SQLite persistence)
- [ ] Implement session creation, validation, invalidation
- [ ] Add session timeout (15 minutes inactivity, configurable)
- [ ] Add session token generation (secure random)
- [ ] Audit log all authentication events
- **Files**: `src/infrastructure/auth/session.rs` (new)
- **Verification**: Integration tests

#### Task 1.5.10: Implement RBAC (Role-Based Access Control)
- [ ] Review `src/infrastructure/auth/rbac.rs`
- [ ] Define roles: Admin, Doctor, Nurse, Receptionist, Billing
- [ ] Implement permission checking
- [ ] Add field-level permissions
- [ ] Implement "break-the-glass" emergency access
- [ ] Audit log all authorization decisions
- **Files**: `src/infrastructure/auth/rbac.rs`
- **Verification**: Unit tests + integration tests

#### Task 1.5.11: Integrate Authentication into App
- [ ] Add login screen to TUI
- [ ] Implement login flow
- [ ] Store session state in App
- [ ] Add logout functionality
- [ ] Add session timeout detection
- [ ] Test full authentication workflow
- **Files**: `src/app.rs`, new UI components
- **Verification**: Manual testing + integration tests

---

### Block 4: Database Schema Completion
**Priority**: P1 - Foundation for Phase 2  
**Estimated Time**: 3-4 days

#### Task 1.5.12: Review and Complete All Domain Tables
- [ ] Review existing migrations
- [ ] Create missing tables for:
  - [ ] Clinical notes (consultations) - SOAP format
  - [ ] Medical history
  - [ ] Allergies and adverse reactions
  - [ ] Current medications
  - [ ] Vital signs
  - [ ] Prescriptions (full schema)
  - [ ] Referrals
  - [ ] Billing/invoices
- [ ] Add proper indexes for foreign keys
- [ ] Add indexes for common query patterns
- [ ] Ensure audit columns on all clinical tables
- **Files**: `migrations/YYYYMMDD_*` (new)
- **Verification**: `sqlx migrate run`, schema review

#### Task 1.5.13: Add Database Constraints
- [ ] Add CHECK constraints for data validation
- [ ] Add UNIQUE constraints for business rules
- [ ] Add NOT NULL constraints where appropriate
- [ ] Add foreign key constraints
- [ ] Test constraint enforcement
- **Files**: Migration files
- **Verification**: Integration tests

---

## Phase 2: Clinical Core (2-3 Months)

### Block 5: Consultation & Clinical Notes (3-4 weeks)

#### Task 2.1: Consultation Domain Implementation
- [ ] Complete `src/domain/clinical/model.rs` - Consultation entity with SOAP structure
- [ ] Create `src/domain/clinical/service.rs` - Business logic
- [ ] Create `src/domain/clinical/repository.rs` - Persistence trait
- [ ] Implement SQLx repository
- [ ] Add consultation creation, update, retrieval
- [ ] Encrypt clinical notes (SOAP subjective, objective, assessment, plan)
- [ ] Integrate audit logging
- **Files**: Multiple in `src/domain/clinical/`
- **Verification**: Unit + integration tests

#### Task 2.2: Medical History Management
- [ ] Create medical history data structures
- [ ] Implement surgical history
- [ ] Implement family history
- [ ] Implement social history (encrypted)
- [ ] Create UI components for data entry
- [ ] Add search/filter capabilities
- **Files**: Domain + UI components
- **Verification**: Tests + manual UI testing

#### Task 2.3: Allergy Management
- [ ] Complete allergy domain model
- [ ] Implement allergy service with validation
- [ ] Create allergy alert system
- [ ] Integrate with prescription workflow (Phase 3 prep)
- [ ] Add UI components (list, add, edit)
- **Files**: Domain + UI
- **Verification**: Tests + manual testing

#### Task 2.4: Current Medications List
- [ ] Create medication management domain
- [ ] Track active medications
- [ ] Track medication history
- [ ] Add start/stop dates
- [ ] Prepare for prescribing integration (Phase 3)
- **Files**: Domain + UI
- **Verification**: Tests

#### Task 2.5: Vital Signs Tracking
- [ ] Create vital signs data model
- [ ] Implement vital signs recording
- [ ] Add graphing/trending over time
- [ ] Common vital signs: BP, HR, temp, weight, height, BMI, SpO2
- [ ] Create UI components
- **Files**: Domain + UI
- **Verification**: Tests + manual testing

#### Task 2.6: Clinical Templates & Quick Text
- [ ] Design template system
- [ ] Implement template storage
- [ ] Add template variables/placeholders
- [ ] Create template selection UI
- [ ] Allow user-defined templates
- **Files**: Domain + UI
- **Verification**: Manual testing

---

### Block 6: Drug Database Integration (2-3 weeks) - CRITICAL FOR PHASE 3

#### Task 2.7: Evaluate Drug Database Options
- [ ] Research MIMS (Monthly Index of Medical Specialties) - subscription required
- [ ] Research AusDI (Australian Drug Information) - open data option
- [ ] Research PBS Schedule Data API - free government data
- [ ] Estimate costs and licensing
- [ ] Choose best option for MVP
- **Deliverable**: Decision document
- **Verification**: Technical review

#### Task 2.8: Implement Drug Database Integration
- [ ] Create `src/integrations/drug_database/` module
- [ ] Implement data import/sync
- [ ] Create local drug database schema
- [ ] Implement drug search functionality
- [ ] Add drug information lookup
- [ ] Schedule regular updates
- **Files**: `src/integrations/drug_database/*`
- **Verification**: Integration tests

#### Task 2.9: Implement Drug Interaction Checking
- [ ] Implement interaction checking engine
- [ ] Severity levels: minor, moderate, severe, contraindicated
- [ ] Check against all current medications
- [ ] Display clinical guidance
- [ ] Log all interaction checks
- **Files**: Drug database module
- **Verification**: Unit tests with known interactions

#### Task 2.10: Implement Allergy Cross-Checking
- [ ] Cross-reference prescriptions with patient allergies
- [ ] Check for cross-sensitivities (e.g., penicillin family)
- [ ] Display warnings before prescription creation
- [ ] Require acknowledgment to proceed
- **Files**: Drug database + prescription modules
- **Verification**: Integration tests

---

### Block 7: AIR (Australian Immunisation Register) Integration (2-3 weeks)

#### Task 2.11: PRODA Account Setup
- [ ] Create individual PRODA accounts for developers
- [ ] Register organization in Health Systems Developer Portal
- [ ] Request AIR test environment access
- [ ] Obtain test credentials
- [ ] Document setup process
- **Deliverable**: Working test environment access
- **Verification**: Successful test connection

#### Task 2.12: Implement AIR Client
- [ ] Create `src/integrations/air/` module
- [ ] Implement SOAP web service client
- [ ] Implement PRODA OAuth 2.0 authentication
- [ ] Add vaccination recording
- [ ] Add AIR history retrieval
- [ ] Handle error responses
- **Files**: `src/integrations/air/*`
- **Verification**: Integration tests (test environment)

#### Task 2.13: Implement Immunisation Domain
- [ ] Complete `src/domain/immunisation/model.rs`
- [ ] Create immunisation service
- [ ] Create immunisation repository
- [ ] Implement NIP schedule engine
- [ ] Add overdue vaccination alerts
- [ ] Integrate with AIR client
- **Files**: Domain + integration
- **Verification**: Tests

#### Task 2.14: Create Immunisation UI
- [ ] Immunisation history view
- [ ] Vaccination recording form
- [ ] Schedule reminder system
- [ ] AIR sync status indicator
- [ ] Batch upload for historical records
- **Files**: UI components
- **Verification**: Manual testing

---

### Block 8: Basic Clinical Decision Support (1-2 weeks)

#### Task 2.15: Implement Preventive Care Reminders
- [ ] Age/gender-based health check rules
- [ ] Cervical screening tracking (25-74 years, 5-yearly)
- [ ] Bowel cancer screening (50-74 years, 2-yearly)
- [ ] Cardiovascular risk assessment prompts
- [ ] Mental health screening prompts
- [ ] Falls risk assessment (65+)
- **Files**: Clinical decision support module
- **Verification**: Unit tests with known cases

#### Task 2.16: Implement Basic Risk Calculators
- [ ] BMI calculator and interpretation
- [ ] eGFR calculator (kidney function)
- [ ] Basic cardiovascular risk scores
- [ ] Add to clinical workflow
- **Files**: Clinical module
- **Verification**: Unit tests with known values

---

## Phase 3: Prescribing & Billing (2-3 Months)

### Block 9: Electronic Prescribing (3-4 weeks)

#### Task 3.1: Prescription Domain Implementation
- [ ] Complete `src/domain/prescription/model.rs`
- [ ] Create prescription service with business logic
- [ ] Create prescription repository
- [ ] Implement repeat prescriptions
- [ ] Implement authority prescriptions
- [ ] Add PBS/RPBS integration
- **Files**: Domain + infrastructure
- **Verification**: Tests

#### Task 3.2: E-Prescribing Integration
- [ ] Research e-prescribing providers (eRx, MediSecure)
- [ ] Choose provider for MVP
- [ ] Implement token-based delivery
- [ ] Add QR code generation
- [ ] Test with provider test environment
- **Files**: Integration module
- **Verification**: Conformance testing

#### Task 3.3: Prescription UI
- [ ] Prescription creation form
- [ ] Drug selection with search
- [ ] Dosage calculator
- [ ] Interaction warnings display
- [ ] Allergy warnings display
- [ ] Prescription history view
- **Files**: UI components
- **Verification**: Manual testing

---

### Block 10: Medicare Claiming (4-5 weeks)

#### Task 3.4: Medicare Online Integration Setup
- [ ] Complete PRODA registration for Medicare Online
- [ ] Request test environment access
- [ ] Obtain test credentials and certificates
- [ ] Document setup process
- **Deliverable**: Working test environment
- **Verification**: Successful test connection

#### Task 3.5: Implement Medicare Claiming
- [ ] Create `src/integrations/medicare/` module
- [ ] Implement SOAP web services client
- [ ] Implement patient claiming (bulk billing + private)
- [ ] Implement patient eligibility verification
- [ ] Implement MBS item selection
- [ ] Implement claim submission
- [ ] Implement claim status tracking
- [ ] Handle claim responses
- **Files**: Integration module
- **Verification**: Test environment validation

#### Task 3.6: Billing Domain Implementation
- [ ] Complete `src/domain/billing/model.rs`
- [ ] Create billing service
- [ ] Create billing repository
- [ ] Implement invoice generation
- [ ] Implement payment tracking
- [ ] Implement accounts receivable
- **Files**: Domain layer
- **Verification**: Tests

#### Task 3.7: Billing UI
- [ ] Billing screen with MBS item search
- [ ] Invoice creation workflow
- [ ] Payment processing screen
- [ ] Receipt generation
- [ ] Outstanding accounts view
- **Files**: UI components
- **Verification**: Manual testing

---

## 🧪 Continuous Quality Assurance

### Every Sprint:
- [ ] Run full test suite: `cargo test`
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Format code: `cargo fmt`
- [ ] Update documentation
- [ ] Security review (PII handling, encryption, audit logs)
- [ ] Manual TUI testing

### Before Each Phase Completion:
- [ ] Integration testing across all modules
- [ ] Performance testing (esp. database queries)
- [ ] Security audit
- [ ] Documentation review
- [ ] User acceptance testing (if possible)

---

## 📊 Success Metrics

### Phase 1.5 Complete When:
- [ ] All tests passing (100% green)
- [ ] Authentication fully implemented
- [ ] Patient domain fully functional
- [ ] Zero clippy warnings
- [ ] Documentation up-to-date

### Phase 2 Complete When:
- [ ] Consultations can be created and stored
- [ ] Medical history fully captured
- [ ] Allergies tracked and alerting
- [ ] Medications tracked
- [ ] Vital signs recorded
- [ ] AIR integration functional
- [ ] Basic clinical decision support active

### Phase 3 Complete When:
- [ ] Electronic prescriptions can be created
- [ ] Medicare claims can be submitted
- [ ] Bulk billing workflow complete
- [ ] Invoices generated
- [ ] Payments tracked
- [ ] Full end-to-end clinical + billing workflow functional

---

## 🚀 Recommended Next Action

### Option 1: Fix Critical Issues First (RECOMMENDED)
**Start with Block 1: Fix Broken Tests & Build**
- Estimated: 2-3 days
- High impact, low complexity
- Unblocks everything else

### Option 2: Continue Calendar Enhancement
**Complete remaining Phase 7 tasks from patient-names-display-fix plan**
- Waitlist management
- SMS/email reminder integration
- Additional calendar polish

### Option 3: Tackle Authentication (High Value)
**Jump to Block 3: Implement Authentication System**
- Critical security foundation
- Enables multi-user testing
- Required before any real deployment

---

## 🤔 Recommendation

**I recommend Option 1: Fix Critical Issues First**

**Reasoning:**
1. **Build stability** - Tests must pass before adding features
2. **Quality foundation** - Clippy warnings indicate potential bugs
3. **Fast turnaround** - Can complete in 2-3 days
4. **Unblocks future work** - Clean build enables confident development

**After Block 1, continue with:**
- Block 2 (Patient Domain Completion) - 3-5 days
- Block 3 (Authentication) - 5-7 days
- Then proceed to Phase 2 (Clinical Core)

---

## 📝 Notes

- **Parallelization**: Many tasks can be done in parallel (e.g., different domain implementations)
- **External Dependencies**: AIR and Medicare integrations require government approval timelines (6-12 months)
- **Start conformance processes early** - they run in parallel with development
- **Budget for conformance testing**: $25k-$50k total
- **Consider hiring healthcare domain expert** for Phase 2-3 validation

---

## 📅 Estimated Timeline

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1.5 (Stabilization) | 2-3 weeks | Green build, auth, complete patient domain |
| Phase 2 (Clinical Core) | 2-3 months | Consultations, history, allergies, medications, AIR, CDS |
| Phase 3 (Prescribing & Billing) | 2-3 months | E-prescribing, Medicare claiming, invoicing |
| **Total for MVP** | **5-7 months** | Fully functional clinical + billing system |

Add 6-12 months for conformance processes (can run in parallel).

---

**Status**: 🟢 Ready to Begin  
**Next Step**: Review and approve plan, then start Block 1 Task 1.5.1

---

*Generated: 2026-02-13*  
*Based on: REQUIREMENTS.md, current codebase state, git history*
