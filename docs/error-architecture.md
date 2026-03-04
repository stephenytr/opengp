# Error Architecture — Domain Crate

**Date:** 2026-03-04  
**Status:** Decision document (pre-implementation)  
**Scope:** `crates/opengp-domain/src/domain/`

---

## 1. Current State Inventory

### 1.1 Shared Base (`domain/error.rs`)

```rust
pub enum RepositoryError {
    Database(String),
    NotFound,
    ConstraintViolation(String),
}

pub trait InfrastructureError { ... }
```

Three variants. No entity context (no `Uuid`). Used as a base type that modules convert `From<>`.

---

### 1.2 Module Error Files — Full Inventory

| Module | File | RepositoryError type | ServiceError type | ValidationError type | Notes |
|--------|------|---------------------|-------------------|---------------------|-------|
| `domain` | `error.rs` | `RepositoryError` (shared base) | — | — | Base type |
| `patient` | `patient/error.rs` | Local `RepositoryError` | `ServiceError` | `ValidationError` | Extends base + `Encryption(String)` |
| `clinical` | `clinical/error.rs` | Local `RepositoryError` | `ServiceError` | — | Extends base + `Encryption`, `Decryption`; **BUG on line 64** |
| `appointment` | `appointment/error.rs` | `pub use` base directly | `ServiceError` | `ValidationError` | Clean re-export; no local repo error |
| `prescription` | `prescription/error.rs` | `pub use` base directly | `ServiceError` | `ValidationError` | Clean re-export; no local repo error |
| `audit` | `audit/error.rs` | `AuditRepositoryError` | `ServiceError` | `ValidationError` | Extends base + `ImmutableViolation` |
| `user` | `user/error.rs` | `pub use` base as `UserRepositoryError` | `UserError` (combined) | — | No separate ServiceError; combined pattern |

---

## 2. Bug Report: `Uuid::nil()` in `clinical/error.rs:64`

### Location
`crates/opengp-domain/src/domain/clinical/error.rs`, line 64

### Code
```rust
impl From<BaseRepositoryError> for RepositoryError {
    fn from(err: BaseRepositoryError) -> Self {
        match err {
            BaseRepositoryError::Database(e) => RepositoryError::Database(e),
            BaseRepositoryError::NotFound => RepositoryError::NotFound(Uuid::nil()),  // ← BUG
            BaseRepositoryError::ConstraintViolation(s) => RepositoryError::ConstraintViolation(s),
        }
    }
}
```

### Root Cause
The base `RepositoryError::NotFound` carries no `Uuid` context (it's a unit variant). The clinical module's local `RepositoryError::NotFound` requires a `Uuid`. The conversion fills it with `Uuid::nil()` (all-zeros UUID) — a sentinel value that is **semantically meaningless** and **actively misleading** in error messages and logs.

### Impact
- Error messages will display `NotFound: 00000000-0000-0000-0000-000000000000`
- Callers cannot distinguish "not found with unknown ID" from "not found with ID nil"
- Audit logs and error traces become unreliable for clinical records

### Fix Plan (do not implement yet)
**Option A — Preferred**: Change `clinical::RepositoryError::NotFound` to a unit variant (no `Uuid`), matching the base type. Entity context belongs in `ServiceError`, not `RepositoryError`.

```rust
// clinical/error.rs — after fix
pub enum RepositoryError {
    Database(String),
    NotFound,                    // ← unit variant, no Uuid
    ConstraintViolation(String),
    Encryption(String),
    Decryption(String),
}
```

**Option B**: Change the base `RepositoryError::NotFound` to carry `Option<Uuid>`. Rejected — adds complexity to all modules for one module's needs.

**Option C**: Remove the `From<BaseRepositoryError>` impl for clinical entirely and map manually at call sites. Viable but verbose.

**Recommendation**: Option A. `RepositoryError` should not carry entity IDs — that context belongs in `ServiceError` variants like `ConsultationNotFound(Uuid)`.

---

## 3. Unification Decisions

### 3.1 `RepositoryError` → **Shared (keep and extend)**

**Decision**: The shared `domain::error::RepositoryError` is the canonical repository error type. Modules that need only `Database`, `NotFound`, `ConstraintViolation` **must re-export it directly** (as `appointment` and `prescription` already do correctly).

**Rationale**:
- `appointment/error.rs` and `prescription/error.rs` already demonstrate the correct pattern: `pub use crate::domain::error::RepositoryError;`
- Duplicating these three variants in every module creates drift (patient and clinical already diverged)
- Infrastructure implementations only need to know one type for the common path

**Modules that need extension** (local repo error type is justified):

| Module | Extra variants | Justification |
|--------|---------------|---------------|
| `patient` | `Encryption(String)` | Patient data is encrypted at rest; encryption failures are a distinct failure mode |
| `clinical` | `Encryption(String)`, `Decryption(String)` | Same as patient; clinical notes are encrypted |
| `audit` | `ImmutableViolation` | Audit log is append-only; this variant enforces the invariant at the type level |

**Modules that should use shared directly** (no local repo error needed):

| Module | Current state | Action |
|--------|--------------|--------|
| `appointment` | ✅ Already re-exports base | No change |
| `prescription` | ✅ Already re-exports base | No change |
| `user` | ✅ Re-exports base as `UserRepositoryError` | Rename alias to `RepositoryError` for consistency (minor) |

### 3.2 `ServiceError` → **Module-specific (stay)**

**Decision**: Each module keeps its own `ServiceError`. Do **not** unify into a shared `ServiceError`.

**Rationale**:
- Service errors are domain-semantic: `ConsultationNotFound(Uuid)`, `AppointmentConflict(String)`, `PBSAuthorityRequired(String)` — these are meaningless outside their module
- A shared `ServiceError` would be a grab-bag enum with 30+ variants, violating the principle of least surprise
- Callers of a service only care about that service's errors; cross-module error handling is done at the application layer
- The `appointment::ServiceError` already cross-references `audit::ServiceError` via `#[from]` — this is the correct pattern for inter-module dependencies

### 3.3 `ValidationError` → **Module-specific (stay)**

**Decision**: Each module keeps its own `ValidationError`. Do **not** unify.

**Rationale**:
- Validation rules are entirely domain-specific (Medicare number format vs. appointment time range vs. PBS status)
- No shared validation logic exists to extract
- Unifying would produce a meaningless enum

### 3.4 `UserError` (combined pattern) → **Normalise to split pattern**

**Decision**: `user/error.rs` uses a combined `UserError` that merges service and repository concerns. This should be split into `ServiceError` + re-exported `RepositoryError` to match the pattern used by all other modules.

**Current**:
```rust
pub enum UserError {
    Validation(String),
    NotFound(String),
    Duplicate(String),
    AuthenticationFailed,
    AccountLocked,
    Repository(#[from] UserRepositoryError),  // ← mixed concerns
}
```

**Target**:
```rust
pub use crate::domain::error::RepositoryError;

pub enum ServiceError {
    Validation(String),
    NotFound(Uuid),          // use Uuid not String for consistency
    Duplicate(String),
    AuthenticationFailed,
    AccountLocked,
    Repository(#[from] RepositoryError),
}
```

---

## 4. Target Architecture Summary

```
domain/error.rs
└── RepositoryError { Database, NotFound, ConstraintViolation }   ← shared base
└── InfrastructureError (trait)

patient/error.rs
├── RepositoryError { Database, NotFound, ConstraintViolation, Encryption }  ← extends base
├── ValidationError { EmptyName, InvalidDateOfBirth, InvalidMedicareNumber }
└── ServiceError { DuplicatePatient, NotFound(Uuid), Validation, Repository }

clinical/error.rs
├── RepositoryError { Database, NotFound, ConstraintViolation, Encryption, Decryption }  ← extends base (fix NotFound to unit)
└── ServiceError { ConsultationNotFound(Uuid), PatientNotFound(Uuid), ..., Repository }

appointment/error.rs
├── pub use domain::error::RepositoryError  ← shared directly (no change)
├── ValidationError { InvalidTime, EndTimeBeforeStartTime, ... }
└── ServiceError { NotFound(Uuid), Conflict, ValidationError, InvalidTransition, Repository, Audit }

prescription/error.rs
├── pub use domain::error::RepositoryError  ← shared directly (no change)
├── ValidationError { EmptyField, InvalidQuantity, ... }
└── ServiceError { NotFound(Uuid), AlreadyCancelled, PrescriptionExpired, ... }

audit/error.rs
├── AuditRepositoryError { Database, NotFound, ConstraintViolation, ImmutableViolation }  ← extends base
├── ValidationError { InvalidEntityType, InvalidEntityId, ... }
└── ServiceError { NotFound(Uuid), ValidationError, Repository }

user/error.rs
├── pub use domain::error::RepositoryError  ← normalise to shared (currently aliased)
└── ServiceError { Validation, NotFound(Uuid), Duplicate, AuthenticationFailed, AccountLocked, Repository }
```

---

## 5. Implementation Order (for future tasks)

1. **Fix `clinical/error.rs:64` bug** — change `NotFound(Uuid)` to `NotFound` unit variant; update all match arms in clinical module
2. **Normalise `user/error.rs`** — split `UserError` into `ServiceError` + re-export `RepositoryError`; update `user/service.rs` and `user/repository.rs`
3. **Verify `patient/error.rs`** — `Encryption` variant is justified; `From<BaseRepositoryError>` impl is correct
4. **Verify `audit/error.rs`** — `ImmutableViolation` is justified; `From<BaseRepositoryError>` impl is correct
5. **No changes needed** for `appointment`, `prescription` (already correct pattern)

---

## 6. Invariants to Enforce

- `RepositoryError` variants must **not** carry entity `Uuid` — that context belongs in `ServiceError`
- `ServiceError` must **not** be shared across modules
- Modules without encryption must **not** define local `RepositoryError` — use the shared base
- `From<BaseRepositoryError>` impls must map all variants without sentinel values (`Uuid::nil()` is forbidden)
