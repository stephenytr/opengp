# PROJECT KNOWLEDGE BASE

**Updated:** 2026-04-13
**Branch:** master
**Project:** OpenGP - Australian General Practice Management Software

## OVERVIEW

OpenGP is a Rust, terminal-first GP management platform for Australian healthcare workflows.
Codebase follows layered domain-driven architecture, organised as a Cargo workspace.
Two runnable binaries: TUI app (`src/main.rs`) and REST API server (`crates/opengp-api/src/main.rs`).

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
├── migrations/                 # SQLite schema migrations (20 files, latest Apr 2026)
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
- `wiki/User-Guide.md` — end-user guide
- `wiki/Integration-UI-Guide.md` — UI integration walkthrough
- `wiki/Integration-Database-Guide.md` — DB integration walkthrough
- `wiki/Integration-External-Guide.md` — external integration guide
- `wiki/Integration-End-to-End-Checklist.md` — end-to-end checklist
- `docs/QUICK_REFERENCE.md` — quick reference card
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
- PostgreSQL migration path exists in design but SQLite is the active database.

## PRACTICAL REFERENCE MODULES

- **Simple pattern**: `crates/opengp-domain/src/domain/immunisation/` — model, repo, service, error only
- **Core pattern**: `crates/opengp-domain/src/domain/patient/` — full set including dto, types
- **Complex orchestration**: `crates/opengp-domain/src/domain/clinical/` — multi-repo service, suggest_mbs_level helper
- **API DTOs**: `crates/opengp-domain/src/domain/api/dto.rs` — shared request/response types used by TUI↔API
- **Cache usage**: `crates/opengp-cache/src/service.rs` + `patient_cache.rs` for pattern
