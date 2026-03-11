# PROJECT KNOWLEDGE BASE

**Updated:** 2026-03-11  
**Branch:** master  
**Project:** OpenGP - Australian General Practice Management Software

## OVERVIEW

OpenGP is a Rust, terminal-first GP management platform for Australian healthcare workflows.  
The codebase follows layered architecture and is organized as a Cargo workspace.

## CURRENT STRUCTURE

```text
opengp/
├── crates/
│   ├── opengp-domain/          # Core business logic, traits, domain services
│   ├── opengp-infrastructure/  # SQLx repositories, auth, crypto, fixtures
│   ├── opengp-ui/              # Ratatui UI, components, app state/services
│   ├── opengp-config/          # Configuration loading/validation
│   └── opengp-api/             # API-facing crate (in progress)
├── src/main.rs                 # Binary composition root
├── migrations/                 # Database schema migrations
├── tests/                      # Integration-level tests
├── wiki/                       # Contributor and integration guides
├── ARCHITECTURE.md             # Deep architecture reference
└── REQUIREMENTS.md             # Product/compliance requirements
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add domain entity/model | `crates/opengp-domain/src/domain/{module}/model.rs` | Keep domain pure |
| Add repository trait | `crates/opengp-domain/src/domain/{module}/repository.rs` | Trait only |
| Add service logic | `crates/opengp-domain/src/domain/{module}/service.rs` | Business rules + orchestration |
| Add DB implementation | `crates/opengp-infrastructure/src/infrastructure/database/repositories/{module}.rs` | SQLx implementation |
| Add UI workflow | `crates/opengp-ui/src/ui/components/{module}/` | `state.rs`, `form.rs`, `list.rs` patterns |
| UI↔Domain bridge | `crates/opengp-ui/src/ui/services/{module}_service.rs` | Translate errors for UI |
| App wiring | `src/main.rs` | Compose repositories/services/UI services |

## ARCHITECTURAL CONVENTIONS

- **Domain-first contracts**: Define interfaces in `opengp-domain`, implement in infrastructure.
- **Trait-based repositories**: `Arc<dyn ...Repository>` across service boundaries.
- **Error layering**:
  - Base: `crates/opengp-domain/src/domain/error.rs`
  - Module: wraps base error + module-specific variants
- **Async I/O boundaries**: Repository methods are async.
- **Encryption path**: Sensitive fields handled in infrastructure repositories using crypto service.
- **Soft delete semantics**: Clinical/patient data prefers deactivation over hard delete.

## IMPLEMENTATION PATTERN (NEW MODULE)

1. Create domain module files: `model.rs`, `repository.rs`, `service.rs`, `error.rs`, `dto.rs`.
2. Export module in `domain/mod.rs`.
3. Implement repository in infrastructure with SQLx mapping.
4. Add/extend UI service bridge for user-facing operations.
5. Add/update UI components and keybind handlers.
6. Wire concrete dependencies in `src/main.rs`.
7. Add tests at relevant layers and run full verification.

## TESTING & VERIFICATION

Standard commands:

```bash
cargo test
cargo build --release
cargo run --release
```

Recommended focused checks:

```bash
cargo test -p opengp-domain
cargo test -p opengp-infrastructure
cargo test -p opengp-ui
```

## DOCUMENTATION MAP

- `README.md` — entry point and contributor navigation
- `wiki/Home.md` — practical integration guide index
- `ARCHITECTURE.md` — architecture deep dive
- `REQUIREMENTS.md` — compliance and product expectations

## REPOSITORY RULES

- Do not run destructive commands without explicit instruction.
- Do not revert user-authored git changes without explicit instruction.
- Never commit secrets (`.env`, credentials, private keys).
- Keep docs and plan files in sync with major changes.

## KNOWN GAPS / RISK AREAS

- External Australian integrations are not complete end-to-end (Medicare/PBS/AIR).
- Some security roadmap items remain (e.g., stronger auth hardening and broader compliance automation).
- API crate exists but is still evolving.

## PRACTICAL REFERENCE MODULES

- **Simple pattern**: `crates/opengp-domain/src/domain/immunisation/`
- **Core pattern**: `crates/opengp-domain/src/domain/patient/`
- **Complex orchestration**: `crates/opengp-domain/src/domain/clinical/`
