# Audit Decoupling: Event-Based Mechanism Decision

**Date:** 2026-03-04  
**Status:** Decision Made  
**Scope:** `opengp-domain` crate — `ClinicalService`, `AppointmentService`, and all future domain services

---

## Context

Domain services currently take `Arc<AuditService>` as a direct constructor dependency. This creates two problems:

1. **Constructor bloat** — `ClinicalService` already has 8 constructor arguments (6 repos + `patient_service` + `audit_logger`). The `service!` macro generates positional `new()` calls, so every new dependency adds another positional arg.

2. **Inconsistent error handling** — `ClinicalService` uses `.await.ok()` (fire-and-forget, silently swallows audit failures) while `AppointmentService` uses `.await?` (propagates audit failures to callers). This inconsistency is a latent bug: a transient DB error in the audit repo can fail a clinical operation in one service but not another.

---

## Options Evaluated

### Option A: `Vec<DomainEvent>` returned alongside results

Services return `(T, Vec<DomainEvent>)` tuples. The caller (infrastructure layer) is responsible for persisting events.

```rust
pub async fn create_consultation(
    &self,
    data: NewConsultationData,
    user_id: Uuid,
) -> Result<(Consultation, Vec<DomainEvent>), ServiceError> {
    let saved = self.consultation_repo.create(consultation).await?;
    let events = vec![DomainEvent::ConsultationCreated { id: saved.id, patient_id: saved.patient_id, user_id }];
    Ok((saved, events))
}
```

**Pros:**
- Pure domain — no audit dependency in service constructors at all
- Events are explicit, typed, and testable without mocks
- Caller decides what to do with events (persist, publish, ignore)
- Aligns with DDD event sourcing patterns

**Cons:**
- **Breaking change to all public service APIs** — every method signature changes
- Callers must handle the tuple; easy to accidentally discard events with `let (result, _) = ...`
- `Vec<DomainEvent>` must be defined somewhere in the domain crate — adds a new type hierarchy
- Doesn't solve the `.ok()` vs `?` inconsistency — callers still decide whether to propagate
- Calendar/query methods that don't mutate state still return `Vec<DomainEvent>` (empty, but present)

---

### Option B: `AuditEmitter` trait injected instead of `AuditService`

Replace `Arc<AuditService>` with `Arc<dyn AuditEmitter>` where `AuditEmitter` is a thin trait:

```rust
#[async_trait]
pub trait AuditEmitter: Send + Sync {
    async fn emit(&self, entry: AuditEntry) -> Result<(), AuditEmitterError>;
}
```

`AuditService` implements `AuditEmitter`. Tests inject a `MockAuditEmitter` or `NoOpAuditEmitter`.

**Pros:**
- Minimal change to service method signatures — only the constructor field type changes
- Testability: inject `NoOpAuditEmitter` in unit tests without a real DB
- Consistent interface — all services use the same trait
- Solves the `.ok()` vs `?` inconsistency by standardising on one approach in the trait contract

**Cons:**
- Still a constructor dependency — `ClinicalService` still has 8 args
- Adds a new trait + error type to the domain crate
- `async_trait` required (already used in repositories, so no new dep)
- Doesn't reduce constructor arity — the core bloat problem remains

---

### Option C: Group dependencies into `ClinicalContext` struct

Bundle related dependencies into a context struct passed to the constructor:

```rust
pub struct ClinicalContext {
    pub consultation_repo: Arc<dyn ConsultationRepository>,
    pub allergy_repo: Arc<dyn AllergyRepository>,
    pub medical_history_repo: Arc<dyn MedicalHistoryRepository>,
    pub vital_signs_repo: Arc<dyn VitalSignsRepository>,
    pub social_history_repo: Arc<dyn SocialHistoryRepository>,
    pub family_history_repo: Arc<dyn FamilyHistoryRepository>,
}

service! {
    ClinicalService {
        context: ClinicalContext,
        patient_service: Arc<PatientService>,
        audit_logger: Arc<AuditService>,
    }
}
```

**Pros:**
- Reduces visible constructor arity (3 args instead of 8)
- No change to method signatures
- `ClinicalContext` can be built once and reused

**Cons:**
- **Doesn't solve the audit coupling problem** — `audit_logger` is still a direct dep
- Doesn't improve testability of audit behaviour
- The `service!` macro would need updating or bypassing for the context pattern
- Grouping is somewhat arbitrary — why group repos but not `patient_service`?
- Adds indirection without addressing the root issue

---

## Decision: **Option B — `AuditEmitter` trait**

### Rationale

The core problem is **testability and consistency**, not constructor arity. Option B directly addresses both:

1. **Testability**: Unit tests for `ClinicalService` and `AppointmentService` currently cannot be written without a real `AuditRepository` implementation. With `AuditEmitter`, a `NoOpAuditEmitter` (returns `Ok(())` immediately) makes unit tests trivial.

2. **Consistency**: The `.ok()` vs `?` split is a real bug risk. With a trait, we can standardise: audit failures in clinical operations should be fire-and-forget (`.ok()`), while audit failures in appointment status transitions may warrant propagation. The trait makes this a deliberate per-call decision rather than an accidental one.

3. **Minimal disruption**: No method signature changes. Only the constructor field type changes from `Arc<AuditService>` to `Arc<dyn AuditEmitter>`. The `service!` macro works unchanged.

4. **Option A rejected**: Returning `Vec<DomainEvent>` is the right long-term architecture for event sourcing, but it's a large breaking change to all public APIs. It should be revisited if the project moves toward event sourcing. For now, it's over-engineering.

5. **Option C rejected**: It doesn't solve the audit coupling problem at all — it only cosmetically reduces constructor arity. The audit dependency remains identical.

---

## Impact on Service Constructors

### Before

```rust
service! {
    ClinicalService {
        consultation_repo: Arc<dyn ConsultationRepository>,
        allergy_repo: Arc<dyn AllergyRepository>,
        medical_history_repo: Arc<dyn MedicalHistoryRepository>,
        vital_signs_repo: Arc<dyn VitalSignsRepository>,
        social_history_repo: Arc<dyn SocialHistoryRepository>,
        family_history_repo: Arc<dyn FamilyHistoryRepository>,
        patient_service: Arc<PatientService>,
        audit_logger: Arc<AuditService>,  // ← concrete type
    }
}

service! {
    AppointmentService {
        repository: Arc<dyn AppointmentRepository>,
        audit_service: Arc<AuditService>,  // ← concrete type
        calendar_query: Arc<dyn AppointmentCalendarQuery>,
    }
}
```

### After

```rust
service! {
    ClinicalService {
        consultation_repo: Arc<dyn ConsultationRepository>,
        allergy_repo: Arc<dyn AllergyRepository>,
        medical_history_repo: Arc<dyn MedicalHistoryRepository>,
        vital_signs_repo: Arc<dyn VitalSignsRepository>,
        social_history_repo: Arc<dyn SocialHistoryRepository>,
        family_history_repo: Arc<dyn FamilyHistoryRepository>,
        patient_service: Arc<PatientService>,
        audit_logger: Arc<dyn AuditEmitter>,  // ← trait object
    }
}

service! {
    AppointmentService {
        repository: Arc<dyn AppointmentRepository>,
        audit_service: Arc<dyn AuditEmitter>,  // ← trait object
        calendar_query: Arc<dyn AppointmentCalendarQuery>,
    }
}
```

### Infrastructure wiring (unchanged behaviour)

```rust
// In infrastructure layer — AuditService implements AuditEmitter
let audit_emitter: Arc<dyn AuditEmitter> = Arc::new(AuditService::new(audit_repo));
let clinical_service = ClinicalService::new(
    consultation_repo,
    allergy_repo,
    // ...
    audit_emitter,
);
```

### Unit test wiring (new capability)

```rust
// In tests — no DB required
struct NoOpAuditEmitter;

#[async_trait]
impl AuditEmitter for NoOpAuditEmitter {
    async fn emit(&self, _entry: AuditEntry) -> Result<(), AuditEmitterError> {
        Ok(())
    }
}

let service = ClinicalService::new(
    Arc::new(MockConsultationRepo::new()),
    // ...
    Arc::new(NoOpAuditEmitter),
);
```

---

## Error Handling Standardisation

The `.ok()` vs `?` inconsistency should be resolved per-domain, not per-service:

| Domain | Audit failure behaviour | Rationale |
|--------|------------------------|-----------|
| Clinical | `.ok()` (fire-and-forget) | Clinical operations must not fail due to audit DB issues |
| Appointment | `.ok()` (fire-and-forget) | Consistent with clinical; audit is observability, not business logic |
| Prescription | `.ok()` (fire-and-forget) | Same rationale |

The `AuditEmitter` trait itself returns `Result<(), AuditEmitterError>`, preserving the ability to propagate if a future domain explicitly requires it (e.g., a compliance-critical operation where audit failure must block the action).

---

## New Types Required

```rust
// In crates/opengp-domain/src/domain/audit/mod.rs

#[derive(Debug, thiserror::Error)]
pub enum AuditEmitterError {
    #[error("Failed to emit audit entry: {0}")]
    Emit(String),
}

#[async_trait::async_trait]
pub trait AuditEmitter: Send + Sync {
    async fn emit(&self, entry: AuditEntry) -> Result<(), AuditEmitterError>;
}
```

`AuditService` gains an `impl AuditEmitter` block in the domain crate. The infrastructure `AuditRepository` implementation is unchanged.

---

## Future Consideration: Option A

If OpenGP moves toward event sourcing or CQRS, Option A (`Vec<DomainEvent>`) becomes the right choice. At that point:
- `DomainEvent` enum replaces `AuditEntry` as the primary event type
- Services return events; infrastructure persists them
- The audit log becomes a projection of the event stream

This is a larger architectural shift and should be a separate decision when the project is ready for it.
