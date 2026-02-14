# OpenGP Code Review - Issue Report

Generated: 2026-02-15

This document contains all issues found during code review against REQUIREMENTS.md.

---

## CRITICAL ISSUES

### Issue 1: Missing Password Hashing Implementation
**Severity**: CRITICAL  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authentication > Password Requirements

**Description**:
- User model has `password_hash: Option<String>` field but it's NEVER populated
- User service creates users with `password_hash: None`
- No password hashing library (bcrypt/argon2/scrypt) in Cargo.toml
- REQUIREMENTS.md mandates: "Minimum 12 characters, complexity: uppercase, lowercase, numbers, special characters, password history: prevent reuse of last 10 passwords, password expiry: 90 days"

**Files Affected**:
- src/domain/user/model.rs
- src/domain/user/service.rs  
- Cargo.toml

**Recommendation**: Add bcrypt or argon2 crate, implement password hashing in user service

---

### Issue 2: No Encryption of Sensitive Patient Data
**Severity**: CRITICAL  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Encryption > Data at Rest

**Description**:
- REQUIREMENTS.md mandates encryption for: clinical notes (SOAP notes), prescription details, social history, patient financial information, Medicare numbers, IHI
- Patient model stores Medicare number, IHI in PLAIN TEXT
- Clinical model stores SOAPNotes in PLAIN TEXT
- Social history notes stored in PLAIN TEXT
- EncryptionService exists but is NEVER USED in domain models

**Files Affected**:
- src/domain/patient/model.rs
- src/domain/clinical/model.rs
- src/domain/patient/repository.rs
- src/infrastructure/database/repositories/patient.rs

**Recommendation**: Integrate EncryptionService into repository layer for sensitive fields

---

### Issue 3: All Australian Government Integration Modules Are Empty
**Severity**: CRITICAL  
**Category**: Missing Feature  
**REQUIREMENTS.md Reference**: Integration Requirements

**Description**:
| Module | Status | REQUIREMENTS.md Requirement |
|--------|--------|----------------------------|
| medicare/mod.rs | EMPTY | Medicare Online, ECLIPSE claiming |
| pbs/mod.rs | EMPTY | PBS API for real-time pricing |
| air/mod.rs | EMPTY | Australian Immunisation Register (MANDATORY) |
| hi_service/mod.rs | EMPTY | HI Service for IHI/HPI-I/HPI-O |

**Files Affected**:
- src/integrations/medicare/mod.rs
- src/integrations/pbs/mod.rs
- src/integrations/air/mod.rs
- src/integrations/hi_service/mod.rs

---

### Issue 4: No MFA Implementation
**Severity**: HIGH  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authentication > Multi-Factor Authentication

**Description**:
- REQUIREMENTS.md requires: "Multi-Factor Authentication (MFA): Required for all privileged access, recommended for all users"
- AuditAction enum has MFAEnabled, MFADisabled, MFAFailed but no implementation
- AuthError has MFARequired, InvalidMFAToken but never used
- No TOTP or other MFA mechanism implemented

---

### Issue 5: Audit Log Not Protected Against Tampering
**Severity**: HIGH  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Audit Logging > Audit Log Protection

**Description**:
- REQUIREMENTS.md requires: "Immutability: Append-only, no deletion or modification allowed"
- "Integrity: Digital signatures or cryptographic hashing (SHA-256)"
- "Tamper detection: Hash chain linking logs together"
- Current audit_log table has no protection - records can be deleted/modified
- No hash chain implementation

**Files Affected**:
- migrations/20260212224118_create_audit_logs.sql
- src/infrastructure/database/repositories/audit.rs

---

## HIGH PRIORITY ISSUES

### Issue 6: Missing Database Tables
**Severity**: HIGH  
**Category**: Missing Feature  
**REQUIREMENTS.md Reference**: Database Design

**Description**:
REQUIREMENTS.md specifies these tables but they're MISSING from migrations:

| Table | Status | Required For |
|-------|--------|--------------|
| practitioners | MISSING | Multi-practitioner support, HPI-I storage |
| appointments | MISSING | Appointment scheduling |
| consultations | MISSING | Clinical notes (SOAP) |
| prescriptions | MISSING | Prescription management |
| patient_allergies | MISSING | Allergy management |
| immunisations | MISSING | AIR integration |
| referrals | MISSING | Referral management |

**Files Affected**:
- migrations/20260211_initial_schema.sql

---

### Issue 7: No Account Lockout Implementation
**Severity**: HIGH  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authentication

**Description**:
- REQUIREMENTS.md requires: "Account lockout: 5 failed attempts, 15-minute lockout"
- User model has `failed_login_attempts` and `is_locked` fields but NO lockout logic
- No increment of failed attempts on bad password

---

### Issue 8: Session Management Issues
**Severity**: MEDIUM  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authentication

**Description**:
- REQUIREMENTS.md requires: "Session timeout: 15 minutes inactivity"
- Config has `session_timeout_secs` but sessions are never validated for expiration
- Session::is_expired() exists but not used

---

### Issue 9: No Patient Data Export/Correction (Privacy Act APP 12, 13)
**Severity**: HIGH  
**Category**: Compliance  
**REQUIREMENTS.md Reference**: Australian Regulatory Compliance > APP 12, 13

**Description**:
- REQUIREMENTS.md requires: "Patients can request access to their records (APP 12)"
- "Patients can request corrections (APP 13)"
- No functionality to export patient data
- No functionality to request corrections

---

### Issue 10: No Password Policy Validation
**Severity**: MEDIUM  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authentication

**Description**:
- REQUIREMENTS.md requires: "Minimum 12 characters, complexity requirements"
- User model has no password validation
- No complexity checking

---

### Issue 11: RBAC Incomplete
**Severity**: MEDIUM  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authorization

**Description**:
- REQUIREMENTS.md requires: "Field-level permissions: Granular access control"
- Current Permission enum is coarse-grained (PatientRead vs PatientWrite)
- No field-level access control

---

### Issue 12: No Break-the-Glass Emergency Access
**Severity**: MEDIUM  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Authorization

**Description**:
- REQUIREMENTS.md requires: "Break-the-glass emergency access: With mandatory audit logging"
- AuditAction has BreakGlassAccess but no implementation

---

## MEDIUM PRIORITY ISSUES

### Issue 13: Unused EncryptionService in Clinical Domain
**Severity**: MEDIUM  
**Category**: Code Quality  
**Description**:
- EncryptionService is fully implemented with AES-256-GCM
- Tests exist and pass
- But NEVER USED in any repository or service
- Clinical notes, prescriptions should use it

---

### Issue 14: Missing TOTP/MFA Dependencies
**Severity**: MEDIUM  
**Category**: Dependencies  
**Description**:
- No TOTP library in Cargo.toml for MFA
- Need totp-lite or similar for Google Authenticator compatibility

---

### Issue 15: Incomplete User Repository Password Hash
**Severity**: LOW  
**Category**: Code Quality  
**Description**:
- User repository has `password_hash_new` column reference but it's never set
- Could cause confusion

---

### Issue 16: Prescription Domain Has No Service Layer
**Severity**: MEDIUM  
**Category**: Architecture  
**Description**:
- Prescription domain has model, repository, error, dto but NO service.rs
- Other domains (patient, appointment, user) have service layer
- Inconsistent architecture

---

## SUMMARY

| Severity | Count |
|----------|-------|
| CRITICAL | 4 |
| HIGH | 7 |
| MEDIUM | 5 |
| LOW | 1 |

**Total Issues**: 17

**Most Critical**:
1. Password hashing missing (C1)
2. No encryption of patient data (C2) 
3. Empty integration modules (C3)
4. No MFA (C4)

---

### Issue 29: No MBS Item Validation
**Severity**: MEDIUM  
**Category**: Validation  
**REQUIREMENTS.md Reference**: Core Features > Billing > Medicare Claiming

**Description**:
- Billing model has `MBSItem` with `item_number` field
- No validation of MBS item numbers
- REQUIREMENTS.md mandates: "MBS (Medicare Benefits Schedule) item search and selection"
- No check that item numbers are valid MBS items

**Files Affected**:
- src/domain/billing/model.rs

---

### Issue 30: No Patient Data Export (Privacy Act APP 12)
**Severity**: HIGH  
**Category**: Compliance  
**REQUIREMENTS.md Reference**: Australian Regulatory Compliance > APP 12

**Description**:
- REQUIREMENTS.md mandates: "Patients can request access to their records (APP 12)"
- No functionality to export patient data in standard format
- Need: Patient data export (likely JSON or PDF)

**Files Affected**:
- No export functionality

---

### Issue 31: No Patient Data Correction Request (Privacy Act APP 13)
**Severity**: HIGH  
**Category**: Compliance  
**REQUIREMENTS.md Reference**: Australian Regulatory Compliance > APP 13

**Description**:
- REQUIREMENTS.md mandates: "Patients can request corrections (APP 13)"
- No workflow for patients to request data corrections
- Need: Correction request workflow with audit trail

**Files Affected**:
- No correction request functionality

---

### CQ-17: Flaky Crypto Tests
**Severity**: MEDIUM  
**Category**: Testing  
**Description**:
- Crypto tests intermittently fail (test_encrypt_unicode, test_new_without_key)
- Different test fails each run - suggests test isolation issue or env pollution
- Tests set/remove env vars without proper cleanup

**Files**: `src/infrastructure/crypto/mod.rs`

---

## CODE QUALITY ISSUES

### CQ-1: Excessive .unwrap() and .expect() Usage
**Severity**: MEDIUM  
**Category**: Code Quality  
**Count**: 153 occurrences across 21 files

**Description**:
- Using `.unwrap()` and `.expect()` instead of proper error handling
- Many are in non-test code where panics could crash the app
- Violates AGENTS.md rule: "Never use .unwrap() or .expect() in production code"

**Production-Critical Examples**:
- `src/main.rs` line 39: `.expect("Failed to open log file")`
- `src/infrastructure/database/mod.rs` line 145: `create_pool(...).unwrap()`
- `src/components/appointment/form.rs` line 353: `self.selected_patient_id.unwrap()`
- `src/components/appointment/form.rs` line 366: hardcoded UUID with `.expect("valid UUID")`

**Recommendation**: Replace with proper error handling using `?` operator

---

### CQ-2: Unimplemented Functions (todo!)
**Severity**: HIGH  
**Category**: Code Quality  
**Description**:
- `src/infrastructure/database/repositories/patient.rs` line 235: `todo!("Implement update")`
- `src/infrastructure/database/repositories/patient.rs` line 239: `todo!("Implement deactivate")`

**Impact**: Patient update and deactivate don't work in production

---

### CQ-3: TODO Comments Not Addressed
**Severity**: MEDIUM  
**Category**: Technical Debt  
**Description**:
- `src/domain/prescription/service.rs` line 106: "TODO: Implement actual drug interaction checking"
- `src/components/patient/list.rs` line 239: "TODO: Implement patient detail view"
- `src/components/appointment/calendar/component.rs` line 895: "TODO: Fetch audit history from service"

---

### CQ-4: Panic! in Production Code
**Severity**: HIGH  
**Category**: Code Quality  
**Description**:
- `src/infrastructure/fixtures/prescription_generator.rs` line 377: `panic!("Invalid prescription type")`
- `src/infrastructure/database/helpers.rs` line 236: `panic!("Expected ConstraintViolation error")`
- `src/domain/audit/model.rs` lines 328, 347, 366: `panic!` in match arms

**Impact**: These will crash the application rather than returning errors

---

### CQ-5: Excessive #[allow(dead_code)]
**Severity**: LOW  
**Category**: Code Quality  
**Count**: 24 instances (mostly in calendar component)

**Description**:
- Many dead_code allowances suggest incomplete implementations
- `src/components/appointment/calendar/component.rs` has 20+ allowances
- May indicate code rot or abandoned features

---

### CQ-6: Hardcoded Test UUIDs in Production Code
**Severity**: MEDIUM  
**Category**: Code Quality  
**Description**:
- `src/components/appointment/form.rs` line 366: `Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789").unwrap()`
- `src/components/appointment/calendar/component.rs` lines 2987, 3056: same hardcoded UUID

**Impact**: Will cause bugs if this UUID doesn't exist

---

### CQ-7: Missing Error Handling in main.rs
**Severity**: MEDIUM  
**Category**: Code Quality  
**Description**:
- Line 31: `unwrap_or()` with default could mask configuration errors
- Line 33: `.ok()` on directory creation - silently fails
- Line 39: `.expect()` - crashes on log file issues

---

### CQ-8: Wildcard Imports (use super::*)
**Severity**: LOW  
**Category**: Code Quality  
**Count**: 25 instances across codebase

**Description**:
- Makes it harder to track where types come from
- Could cause naming conflicts
- AGENTS.md recommends explicit imports

---

### CQ-9: println!/eprintln! in Comments (Doc Issues)
**Severity**: LOW  
**Category**: Documentation  
**Description**:
- `src/domain/audit/service.rs` lines 93, 102: `println!` and `eprintln!` in comments
- These appear to be leftover debug statements

---

### CQ-10: Inconsistent Repository Patterns
**Severity**: MEDIUM  
**Category**: Architecture  
**Description**:
- Patient repository has `todo!` implementations for update/deactivate
- User repository has full implementation
- Inconsistent - some features work, others don't

---

### CQ-11: Magic Numbers in Code
**Severity**: LOW  
**Category**: Code Quality  
**Examples**:
- `src/domain/prescription/service.rs`: hardcoded `10` for polypharmacy warning
- Session timeouts: 900 seconds (15 min) scattered across config

**Recommendation**: Extract to constants

---

### CQ-12: Large Calendar Component File
**Severity**: LOW  
**Category**: Maintainability  
**Description**:
- `src/components/appointment/calendar/component.rs` is ~3000 lines
- 20+ #[allow(dead_code)] attributes
- Hard to maintain - consider splitting

---

### CQ-13: Duplicate Code in Tests
**Severity**: LOW  
**Category**: Code Quality  
**Description**:
- Multiple test files have similar setup code
- Could extract to common test utilities

---

### CQ-14: Empty Integration Modules (Code Smell)
**Severity**: HIGH  
**Category**: Architecture  
**Description**:
- `src/integrations/medicare/mod.rs` - empty
- `src/integrations/pbs/mod.rs` - empty  
- `src/integrations/air/mod.rs` - empty
- `src/integrations/hi_service/mod.rs` - empty
- These are core features but have zero implementation

---

### CQ-15: Dead Code in Models
**Severity**: LOW  
**Category**: Code Quality  
**Description**:
- Prescription model has `hl7_message_sent`, `hl7_message_id` - never used
- Referral model has `secure_messaging_address` - never used
- Either implement or remove

---

### CQ-16: Config Error Handling
**Severity**: LOW  
**Category**: Code Quality  
**Description**:
- `Config::from_env()` can fail but main.rs uses `?` which is good
- But some configs silently use defaults (line 31 in main.rs)

---

### Issue 17: Logging Sensitive Health Data (Medicare Numbers)
**Severity**: CRITICAL  
**Category**: Security  
**REQUIREMENTS.md Reference**: Australian Regulatory Compliance - "Never log sensitive health data"

**Description**:
- `src/domain/patient/service.rs` lines 26, 28 log Medicare numbers in PLAIN TEXT
- `info!("Checking for duplicate Medicare number: {}", medicare)`
- `error!("Duplicate Medicare number found: {}", medicare)`
- This violates REQUIREMENTS.md security requirement to never log PII

**Files Affected**:
- src/domain/patient/service.rs

---

### Issue 18: Missing Medicare/IHI Number Validation
**Severity**: HIGH  
**Category**: Validation  
**REQUIREMENTS.md Reference**: Core Features > Patient Management

**Description**:
- No validation of Medicare number format (10 digits + checksum)
- No validation of IHI format (16 digits)
- Patient can be registered with invalid identifiers

**Files Affected**:
- src/domain/patient/model.rs
- src/domain/patient/service.rs

---

### Issue 19: No Audit Logging in Patient Service
**Severity**: MEDIUM  
**Category**: Security  
**REQUIREMENTS.md Reference**: Security Requirements > Audit Logging

**Description**:
- PatientService does NOT log patient create/read/update/search operations
- PrescriptionService and AppointmentService DO have audit logging
- Inconsistent - patient access should also be audited per APP 11

**Files Affected**:
- src/domain/patient/service.rs

---

### Issue 20: Missing Domain Service Layers
**Severity**: HIGH  
**Category**: Architecture  
**REQUIREMENTS.md Reference**: Technical Architecture

**Description**:
REQUIREMENTS.md specifies domain services but these are MISSING:

| Domain | Status | Has Service? |
|--------|--------|--------------|
| referral | Model only | NO |
| pathology | Model only | NO |
| immunisation | Model only | NO |
| billing | Model only | NO |

**Files Affected**:
- src/domain/referral/mod.rs
- src/domain/pathology/mod.rs
- src/domain/immunisation/mod.rs
- src/domain/billing/mod.rs

---

### Issue 21: No HL7/FHIR Parsing Implementation
**Severity**: HIGH  
**Category**: Missing Feature  
**REQUIREMENTS.md Reference**: Integration Requirements > HL7 v2.x, FHIR

**Description**:
- Pathology model has `hl7_message_sent` and `hl7_message_id` fields
- But NO actual HL7 parsing/generation code
- No FHIR client or parser
- REQUIREMENTS.md mandates HL7 ORU/ORM, FHIR support for pathology integration

**Files Affected**:
- src/domain/pathology/model.rs
- No src/integrations/hl7/ or src/integrations/fhir/ modules

---

### Issue 22: No Secure Messaging Implementation
**Severity**: HIGH  
**Category**: Missing Feature  
**REQUIREMENTS.md Reference**: Integration Requirements > Secure Messaging

**Description**:
- Referral model has `secure_messaging_address` field
- But NO actual secure messaging implementation
- REQUIREMENTS.md mandates HealthLink, Medical Objects, Argus, SeNT integration

**Files Affected**:
- src/domain/referral/model.rs
- No src/integrations/secure_messaging/ module

---

### Issue 23: Drug Interaction Checking Not Implemented
**Severity**: HIGH  
**Category**: Missing Feature  
**REQUIREMENTS.md Reference**: Core Features > Prescriptions > Drug Database Integration

**Description**:
- PrescriptionService has `check_drug_interactions()` but only returns empty vec
- TODO comment: "TODO: Implement actual drug interaction checking"
- REQUIREMENTS.md mandates: "Real-time drug interaction checking, severity levels, clinical guidance"

**Files Affected**:
- src/domain/prescription/service.rs (line 106)

---

### Issue 24: Unused Prescription Domain Fields
**Severity**: LOW  
**Category**: Code Quality  
**Description**:
- Prescription model has `hl7_message_sent`, `hl7_message_id` fields
- No code populates or uses them
- Dead code or incomplete implementation

**Files Affected**:
- src/domain/prescription/model.rs

---

### Issue 25: Missing Config for Australian Data Sovereignty
**Severity**: MEDIUM  
**Category**: Configuration  
**REQUIREMENTS.md Reference**: Australian Regulatory Compliance > Data Sovereignty

**Description**:
- REQUIREMENTS.md requires: "All patient health data MUST be physically stored within Australian territory"
- Config has no database region/zone setting
- No verification of data sovereignty compliance

**Files Affected**:
- src/config.rs

---

### Issue 26: Hardcoded Test UUIDs in Production Code
**Severity**: MEDIUM  
**Category**: Code Quality  
**Description**:
- `src/components/appointment/form.rs` line 366: hardcoded UUID "a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789"
- `src/components/appointment/calendar/component.rs` lines 2987, 3056: same hardcoded UUID
- Should use proper error handling, not hardcoded test values

**Files Affected**:
- src/components/appointment/form.rs
- src/components/appointment/calendar/component.rs

---

### Issue 27: Patient Detail View Not Implemented
**Severity**: MEDIUM  
**Category**: Missing Feature  
**Description**:
- TODO comment: `src/components/patient/list.rs` line 239: "TODO: Implement patient detail view"
- Partial implementation - need full detail component

**Files Affected**:
- src/components/patient/list.rs

---

### Issue 28: Audit History Not Fetched in Calendar
**Severity**: LOW  
**Category**: Missing Feature  
**Description**:
- TODO comment: `src/components/appointment/calendar/component.rs` line 895: "TODO: Fetch audit history from service"
- Calendar component should show appointment change history

**Files Affected**:
- src/components/appointment/calendar/component.rs

---

## SUMMARY

| Severity | Count |
|----------|-------|
| CRITICAL | 5 |
| HIGH | 17 |
| MEDIUM | 15 |
| LOW | 9 |

**Total Issues**: 46 (31 REQUIREMENTS + 15 CODE QUALITY)

---

## RECOMMENDED PRIORITY

1. **Immediate**: Implement password hashing (Issue 1)
2. **Immediate**: Add encryption to repositories (Issue 2)
3. **Immediate**: Remove Medicare logging (Issue 17)
4. **S2**: Implement MFA (Issue 4)
5. **S2**: Add audit log protection (Issue 5)
6. **S2**: Add Medicare/IHI validation (Issue 18)
7. **S2**: Create missing database tables (Issue 6)
8. **S2**: Complete integration modules (Issue 3)
9. **S3**: Add audit logging to PatientService (Issue 19)
10. **S3**: Implement drug interaction checking (Issue 23)
