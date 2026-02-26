# PROJECT KNOWLEDGE BASE

**Generated:** 2026-02-21
**Commit:** 73451a1
**Branch:** master
**Project:** OpenGP - Australian General Practice Management Software

## OVERVIEW

Rust terminal application for Australian general practice management. Clean Architecture with domain/infrastructure/ui layers. Ratatui TUI framework, SQLite database with PostgreSQL migration path.

## STRUCTURE

```
opengp/
├── src/
│   ├── domain/          # Business logic (10 modules)
│   ├── infrastructure/ # DB, auth, crypto, fixtures
│   ├── integrations/    # Medicare/PBS/AIR (stubs)
│   └── ui/              # Ratatui terminal interface
├── tests/               # Integration tests
├── examples/           # Example scripts
├── migrations/         # SQL schema
├── wiki/               # Git-backed documentation
└── docs/               # Implementation docs
```

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
- Use Context7 for documentation.
- You must not revert git changes without explicit permission.
- Make git commits as required and update and close gitea issues.
- If anything is unclear use the tool provided to ask questions about anything required, do not assume my intent if my instruction is unclear.

## GITEA 

- Open gitea issues using the gitea mcp, whenever you find problems in the codebase.
- Close gitea issues with a commit, push and closing the gitea issue via the gitea-mcp

## ANTI-PATTERNS (THIS PROJECT)

- ❌ NEVER commit `.env` files
- ❌ NEVER use dev API keys in production
- ⚠️ `password_hash` field exists but unused (no bcrypt/argon2)
- ⚠️ `EncryptionService` exists but unused in domain
- ⚠️ Medicare/PBS/AIR integrations are empty stubs
- ⚠️ Audit log has no hash chain (tamperable)
- ⚠️ No MFA/TOTP implementation despite audit flags

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
