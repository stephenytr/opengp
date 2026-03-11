# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-03
**Commit:** b57d966
**Branch:** master
**Project:** OpenGP - Australian General Practice Management Software

## OVERVIEW

Rust terminal application for Australian general practice management. Clean Architecture with domain/infrastructure/ui layers. Ratatui TUI framework, SQLite database with PostgreSQL migration path.

## STRUCTURE

```
opengp/
├── src/                      # ACTUAL SOURCE (monolithic)
│   ├── domain/              # Business logic (10 modules)
│   ├── infrastructure/      # DB, auth, crypto, fixtures
│   ├── integrations/        # Medicare/PBS/AIR (stubs)
│   └── ui/                  # Ratatui terminal interface
├── crates/                  # Workspace crates (stubs - migration in progress)
│   ├── opengp-domain/
│   ├── opengp-infrastructure/
│   ├── opengp-ui/
│   └── opengp-config/
├── tests/                   # Integration tests
├── examples/                # Example scripts
├── migrations/              # SQL schema
├── wiki/                    # Git-backed documentation
└── docs/                    # Implementation docs
```

> **Note**: Dual structure - code in `/src/` but workspace crates in `/crates/` exist as stubs. Migration to pure workspace structure incomplete.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add domain entity | `src/domain/{module}/model.rs` | Follow existing module pattern |
| Add repository | `src/domain/{module}/repository.rs` | Trait + impl |
| Add service | `src/domain/{module}/service.rs` | Business logic |
| DB queries | `src/infrastructure/database/repositories/` | sqlx queries |
| UI component | `src/ui/components/` | Ratatui widgets |
| Run app | `cargo run --release` | |
| Tests | `cargo test` | |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `Patient` | Model | `src/domain/patient/model.rs` | Core entity |
| `Address` | Model | `src/domain/patient/model.rs` | Patient address |
| `User` | Model | `src/domain/user/model.rs` | System user |
| `Practitioner` | Model | `src/domain/user/model.rs` | Doctor/nurse |
| `Appointment` | Model | `src/domain/appointment/model.rs` | Scheduling |
| `Consultation` | Model | `src/domain/clinical/model.rs` | Clinical visit |
| `SOAPNotes` | Model | `src/domain/clinical/model.rs` | Clinical notes |
| `Prescription` | Model | `src/domain/prescription/model.rs` | E-prescribing |
| `Medication` | Model | `src/domain/prescription/model.rs` | Drug details |
| `Invoice` | Model | `src/domain/billing/model.rs` | Patient invoice |
| `MedicareClaim` | Model | `src/domain/billing/model.rs` | Medicare claim |
| `AuditEntry` | Model | `src/domain/audit/model.rs` | Audit log |
| `ClinicalRepository` | Trait | `src/domain/clinical/repository.rs` | Data access |
| `AuditService` | Service | `src/domain/audit/service.rs` | Audit logging |
| `EncryptionService` | Service | `src/infrastructure/crypto/mod.rs` | AES-GCM (unused) |
| `run()` | Function | `src/main.rs` | Entry point |
| `App` | Struct | `src/ui/app.rs` | TUI application |

## CONVENTIONS

- **Domain layer**: Pure Rust, no infrastructure deps
- **Repository pattern**: Trait in domain, impl in infrastructure
- **Error handling**: `thiserror` + `color-eyre`
- **Async**: `tokio::test` for async tests
- **Naming**: `snake_case` files, `PascalCase` types
- **Tests**: write tests first. TDD approach.

## RULES

- Do not use `rm` command without explicit permission.
- Logic that may be used over and over again should be abstracted.
- Use Context7 MCP for up to date information on code.
- You must not revert git changes without explicit permission.
- Make git commits as required and update and close gitea issues.
- If anything is unclear use the tool provided to ask questions about anything required, do not assume my intent if my instruction is unclear.
- do not use worktrees

## GITEA 

- Open gitea issues using the gitea mcp, whenever you find problems in the codebase.
- Close gitea issues with a commit, push and closing the gitea issue via the gitea-mcp

## PLANS
- Always make sure plan files are updated.
- Always make git commits for any change.

## ANTI-PATTERNS (THIS PROJECT)

- ❌ NEVER commit `.env` files
- ❌ NEVER use dev API keys in production
- ⚠️ `password_hash` field exists but unused (no bcrypt/argon2)
- ⚠️ `EncryptionService` exists but unused in domain
- ⚠️ Medicare/PBS/AIR integrations are empty stubs
- ⚠️ Audit log has no hash chain (tamperable)
- ⚠️ No MFA/TOTP implementation despite audit flags
- ⚠️ Dual structure: `/src/` has actual code, `/crates/` are empty stubs - incomplete workspace migration

## UNIQUE STYLES

- Australian healthcare: Medicare, PBS, AIR integration stubs
- Encrypted patient data at rest (AES-GCM)
- Comprehensive audit logging
- Ratatui TUI (not web/CLI)

## COMMANDS

```bash
cargo run --release    # Build and run
cargo test             # Run all tests
cargo test --test '*_test'  # Integration tests only
cargo build --release  # Release build
```

## NOTES

- Wiki lives in `/wiki` as git-tracked Markdown
- SQLite db embedded at project root (`opengp.db`)
- No CI/CD configured (early-stage project)

## ARCHITECTURE UPDATES (2026-03-05)

### Workspace Migration Complete
- **Status**: Workspace crates now contain actual implementation (no longer stubs)
- **Structure**: `crates/opengp-domain/`, `crates/opengp-infrastructure/`, `crates/opengp-ui/`, `crates/opengp-config/`
- **Build**: `cargo build --release` produces 8.1MB binary with all features

### Unified Error Handling
- **RepositoryError Pattern**: Base error type `crate::domain::error::RepositoryError` with variants: `Database`, `NotFound`, `ConstraintViolation`
- **Module-Specific Errors**: Each module (`patient`, `clinical`, `audit`, etc.) wraps base via `Base(#[from] BaseRepositoryError)` + module-specific variants
- **Infrastructure Mapping**: Mocks and database implementations map errors using `RepositoryError::Base(domain::error::RepositoryError::NotFound)` pattern

### AuditEmitter Trait for Decoupling
- **Trait**: `pub trait AuditEmitter: Send + Sync` with `async fn emit(&self, entry: AuditEntry) -> Result<(), AuditRepositoryError>`
- **Implementation**: `AuditService` implements `AuditEmitter` by delegating to `self.log()`
- **Usage**: Services accept `Arc<dyn AuditEmitter>` instead of `Arc<AuditService>` for loose coupling
- **Coercion**: `Arc<AuditService>` cleanly coerces to `Arc<dyn AuditEmitter>`

### ClinicalService Constructor Reduction
- **Before**: 8+ constructor arguments (clinical_repo, patient_repo, audit_service, etc.)
- **After**: 3 arguments via `ClinicalRepositories` struct pattern
  - `repositories: Arc<ClinicalRepositories>` (wraps clinical, patient, appointment repos)
  - `audit_emitter: Arc<dyn AuditEmitter>`
  - `encryption_service: Arc<EncryptionService>`
- **Benefit**: Reduced cognitive load, easier testing, cleaner API

### Completed Stub Modules
- **Immunisation**: Full service with `record_immunisation()`, `find_by_patient()`, `get_due_schedule()` + 4 tests
- **Referral**: Full service with `create_referral()`, `mark_sent()`, `find_by_status()` + 3 tests
- **Pathology**: Full service with `create_order()`, `create_result()`, `find_orders_by_status()` + 3 tests
- **Billing**: Full service with `create_invoice()`, `record_payment()`, `find_claims_by_status()` + 3 tests
- **Pattern**: Each module follows `error.rs` → `repository.rs` → `service.rs` with inline mocks in tests

### Test Coverage
- **Domain Crate**: 52 unit tests (all passing)
- **Infrastructure Crate**: 92 tests including fixtures and mocks (all passing)
- **UI Crate**: 263 widget tests (all passing)
- **Config Crate**: 7 configuration tests (all passing)
- **Total**: 414 tests, 0 failures

### Build Status
- **Release Binary**: 8.1MB ELF executable at `target/release/opengp`
- **Compilation**: Clean with only unused code warnings (dead code in UI calendar widget)
- **Dependencies**: All workspace members compile successfully
