# PROJECT KNOWLEDGE BASE

# OVERVIEW

OpenGP is a Rust, terminal-first GP management platform for Australian healthcare workflows.
Codebase follows layered domain-driven architecture, organised as a Cargo workspace.
Two runnable binaries: TUI app (`src/main.rs`) and REST API server (`crates/opengp-api/src/main.rs`).


## DO NOT EVER REVERT OR DISCARD CHANGES WITHOUT PERMISSION THEY ARE NOT EVER SCOPE CREEP. THEY ARE USUALLY UNCOMMITED CHANGES
## CURRENT STRUCTURE

```text
opengp/
├── crates/
│   ├── opengp-domain/          # Core business logic, traits, domain services
│   ├── opengp-infrastructure/  # SQLx repositories, auth, crypto, MBS, fixtures
│   ├── opengp-ui/              # Ratatui TUI, components, app state/services
│   ├── opengp-config/          # Configuration loading/validation
│   ├── opengp-api/             # Axum REST API server (routes, state, services)
│   └── opengp-cache/           # Redis caching — pool, circuit breaker, stampede guard
├── src/
│   ├── main.rs                 # TUI binary — wires all dependencies
│   ├── lib.rs
│   └── conversions.rs          # Domain↔UI type conversions
├── migrations/                 # PostgreSQL schema migrations (20 files, latest Apr 2026)
├── tests/                      # Integration-level tests
├── wiki/                       # Contributor and integration guides
├── docs/                       # Quick reference and research docs
├── scripts/                    # Dev tooling scripts
├── tools/                      # Additional tooling
├── examples/                   # Usage examples
├── data/                       # Seed / reference data
├── ARCHITECTURE.md             # Deep architecture reference
└── REQUIREMENTS.md             # Product/compliance requirements
```

## DOMAIN MODULES

All modules live under `crates/opengp-domain/src/domain/`:

| Module | Notes |
|--------|-------|
| `api` | Shared API DTOs used by TUI↔API boundary (`dto.rs`) |
| `appointment` | Scheduling, status, calendar logic |
| `audit` | Audit event emission and storage |
| `billing` | MBS billing, bulk bill, consultations |
| `clinical` | Consultations, vitals, allergies, history — complex orchestration |
| `immunisation` | Immunisation records *(feature-gated)* |
| `pathology` | Pathology requests/results *(feature-gated)* |
| `patient` | Patient demographics, search, soft delete |
| `prescription` | Prescription management *(feature-gated)* |
| `referral` | Referral letters *(feature-gated)* |
| `user` | Auth, sessions, practitioners, working hours |
| `error` | Base `RepositoryError` type (Database, NotFound, ConstraintViolation, Conflict) |
| `macros` | Shared derive/utility macros |

Feature-gated modules require explicit Cargo feature flags: `immunisation`, `pathology`, `prescription`, `referral`.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add domain model | `crates/opengp-domain/src/domain/{module}/model.rs` | Keep domain pure |
| Add repository trait | `crates/opengp-domain/src/domain/{module}/repository.rs` | Trait only |
| Add service logic | `crates/opengp-domain/src/domain/{module}/service.rs` | Business rules + orchestration |
| Add domain DTOs | `crates/opengp-domain/src/domain/{module}/dto.rs` | Request/response shapes |
| Add DB implementation | `crates/opengp-infrastructure/src/infrastructure/database/repositories/{module}.rs` | SQLx impl |
| Add UI workflow | `crates/opengp-ui/src/ui/components/{module}/` | `state.rs`, `form.rs`, `list.rs` patterns |
| UI↔Domain bridge | `crates/opengp-ui/src/ui/services/{module}_service.rs` | Translate errors for UI |
| Add API route | `crates/opengp-api/src/routes/{module}.rs` | Register in `routes/mod.rs` |
| Add caching | `crates/opengp-cache/src/` | Use `CacheService`, specialised caches for patient/appointment/search |
| App wiring (TUI) | `src/main.rs` | Compose repositories/services/UI services |
| MBS item data | `crates/opengp-infrastructure/src/infrastructure/mbs/` | XML importer + parser |

## CRATE INVENTORY

### `opengp-domain`
Pure business logic. No I/O. Repository traits only.
- `src/domain/{module}/` — model, repository, service, error, dto per module
- `src/domain/mod.rs` — exports all modules

### `opengp-infrastructure`
All I/O implementations.
- `src/infrastructure/database/repositories/` — SQLx impls: appointment, audit, billing, clinical, patient, practitioner, session, user, working_hours, postgres_user
- `src/infrastructure/auth/` — authentication
- `src/infrastructure/crypto/` — encryption service (used by repos for sensitive fields)
- `src/infrastructure/audit/` — audit emitter impl
- `src/infrastructure/mbs/` — MBS XML importer + parser
- `src/infrastructure/fixtures/` — test/seed fixtures

### `opengp-ui`
Ratatui TUI application.
- `src/ui/components/` — appointment/, billing/, clinical/, patient/, shared/, snapshots/
- `src/ui/services/` — appointment_service, billing_service, clinical_service, patient_service, shared
- `src/ui/app/` — App state, AppCommand channel, event loop

### `opengp-api`
Axum REST API server (separate binary).
- `src/routes/` — appointments, auth, consultations, middleware, patients, practitioners
- `src/state.rs` — shared ApiState
- `src/services.rs` — service wiring
- `src/migrations.rs` — runs migrations on startup
- `src/error.rs` — ApiError type

### `opengp-cache`
Redis caching layer (optional, degrades gracefully).
- `service.rs` — `CacheService` / `CacheServiceImpl` / `CacheConfig`
- `pool.rs` — `RedisPool` connection management
- `circuit.rs` — `CircuitBreaker` / `CircuitState`
- `stampede.rs` — `StampedeGuard` (cache stampede protection)
- `patient_cache.rs`, `appointment_cache.rs`, `search_cache.rs` — domain-specific helpers

### `opengp-config`
Config loading/validation from environment and TOML files.

## ARCHITECTURAL CONVENTIONS

- **Domain-first contracts**: Define interfaces in `opengp-domain`, implement in infrastructure.
- **Trait-based repositories**: `Arc<dyn ...Repository>` across service boundaries.
- **Error layering**:
  - Base: `crates/opengp-domain/src/domain/error.rs` → `RepositoryError`
  - Module: wraps base error + module-specific variants
- **Async I/O boundaries**: All repository methods are async.
- **Encryption path**: Sensitive fields handled in infrastructure repositories via `EncryptionService`.
- **Soft delete semantics**: Clinical/patient data prefers deactivation over hard delete.
- **Cache optional**: All cache calls degrade gracefully — app works without Redis.
- **AppCommand channel**: TUI uses `AppCommand` enum over channels instead of polling for inter-component communication.

## IMPLEMENTATION PATTERN (NEW MODULE)

1. Create domain module files: `model.rs`, `repository.rs`, `service.rs`, `error.rs`, `dto.rs`.
2. Export module in `crates/opengp-domain/src/domain/mod.rs`.
3. Implement repository in infrastructure with SQLx mapping.
4. Add/extend UI service bridge for user-facing operations.
5. Add/update UI components and keybind handlers.
6. Optionally add API route under `crates/opengp-api/src/routes/`.
7. Wire concrete dependencies in `src/main.rs`.
8. Add tests at relevant layers and run full verification.

## TESTING & VERIFICATION

Standard commands:

```bash
cargo test
cargo build --release
cargo run --release
```

Focused checks:

```bash
cargo test -p opengp-domain
cargo test -p opengp-infrastructure
cargo test -p opengp-ui
cargo test -p opengp-cache
```

## DOCUMENTATION MAP

- `README.md` — entry point and contributor navigation
- `wiki/Home.md` — practical integration guide index
- `wiki/Configuration.md` — full configuration reference (env vars, TOML files, all options)
- `wiki/User-Guide.md` — end-user guide
- `wiki/Integration-UI-Guide.md` — UI integration walkthrough
- `wiki/Integration-Database-Guide.md` — DB integration walkthrough
- `wiki/Integration-External-Guide.md` — external integration guide
- `wiki/Integration-End-to-End-Checklist.md` — end-to-end checklist
- `ARCHITECTURE.md` — architecture deep dive
- `REQUIREMENTS.md` — compliance and product expectations

## REPOSITORY RULES

- Do not run destructive commands without explicit instruction.
- Do not revert user-authored git changes without explicit instruction.
- Never commit secrets (`.env`, credentials, private keys).
- Keep docs and plan files in sync with major changes.

## KNOWN GAPS / RISK AREAS

- Medicare/PBS/AIR integrations: MBS XML importer exists (`opengp-infrastructure/mbs/`), billing domain is implemented, but end-to-end Medicare claiming and AIR immunisation reporting are not complete.
- Security roadmap: Auth hardening and broader compliance automation still in progress.
- `opengp-api` is functional (Axum, full route set) but not production-hardened — auth middleware, rate limiting, and observability still evolving.
- Feature-gated modules (`immunisation`, `pathology`, `prescription`, `referral`) may have incomplete UI coverage.
- PostgreSQL is the active database. Connection configured via `DATABASE_URL` / `API_DATABASE_URL` env var.

## PRACTICAL REFERENCE MODULES

- **Simple pattern**: `crates/opengp-domain/src/domain/immunisation/` — model, repo, service, error only
- **Core pattern**: `crates/opengp-domain/src/domain/patient/` — full set including dto, types
- **Complex orchestration**: `crates/opengp-domain/src/domain/clinical/` — multi-repo service, suggest_mbs_level helper
- **API DTOs**: `crates/opengp-domain/src/domain/api/dto.rs` — shared request/response types used by TUI↔API
- **Cache usage**: `crates/opengp-cache/src/service.rs` + `patient_cache.rs` for pattern

## UI CONSISTENCY PATTERNS (Validated)

### A. Error Field Pattern

- Standard: each form or state uses an `error: Option<String>` field and exposes a `set_error(&mut self, error: Option<String>)` method.
- Deprecated names: `save_error`, `error_message` and similar variants were renamed to the standard pattern.
- Current validated usages live in `appointment/form.rs`, `login.rs`, and `status_bar.rs`.

### B. Form Trait Pattern

- Canonical form navigation uses the `FormNavigation` trait plus the `FormState<F>` enum for type-safe state transitions.
- `DynamicForm` is deprecated and has been removed from all forms, with six forms migrated to the canonical pattern.
- Do not implement both `FormNavigation` and `DynamicForm` on the same form type.

### C. Pagination Patterns

Three pagination patterns are in active use and are intentionally distinct:

- `PaginatedState` provides page-based offsets for patient, appointment, and billing lists.
- `PaginatedList<T>` provides scroll-based pagination with wrapping for small billing selection lists.
- `ClinicalTableList<T>` provides scroll-based pagination for clinical records with richer interaction.

Do not mix these pagination patterns within the same component.

### D. Error Mapping at Domain Boundaries

- Use `map_ui_err()` when mapping `ServiceError` and other domain-level errors into UI errors.
- Use `map_ui_repo_err()` when mapping repository-level errors into UI errors.
- These functions are intentionally separate because they handle different error types and layering concerns.

### E. ClinicalState Architecture

- `ClinicalState` is an orchestrator that coordinates clinical sub-states, not a monolith to be flattened.
- It is intentionally heavy on orchestration and delegation, with roughly two thirds orchestration and one third direct state.
- Do not refactor or collapse the clinical sub-state types; they are first-class units of behavior.
- Sub-states such as Vitals, Allergy, FamilyHistory, and MedicalHistory each own and manage their internal state.

### F. Removed Dead Code

- The `ClinicalSubState` trait was removed as dead code; it had four implementations and no polymorphic call sites.
- Nine unused `AppCommand` variants were removed; only three variants remain active in the TUI event loop.
