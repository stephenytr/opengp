# Domain Layer

**Path:** `src/domain/`

## OVERVIEW

Business logic layer following Clean Architecture. Pure Rust with zero infrastructure dependencies. Contains 10 domain modules.

## MODULES

| Module | Status | Purpose |
|--------|--------|---------|
| `patient/` | Active | Patient demographics, records |
| `appointment/` | Active | Scheduling, appointments |
| `clinical/` | Active | Consultations, notes |
| `prescription/` | Active | E-prescribing |
| `audit/` | Active | Audit logging |
| `user/` | Active | Practitioners, staff |
| `billing/` | Stub | Billing (TODO) |
| `pathology/` | Stub | Pathology orders (TODO) |
| `immunisation/` | Stub | Immunisations (TODO) |
| `referral/` | Stub | Specialist referrals (TODO) |

## MODULE PATTERN

Each domain module follows identical structure:

```
{module}/
├── model.rs      # Domain entity (struct, impl)
├── dto.rs        # Data transfer objects
├── error.rs      # Domain errors (thiserror)
├── repository.rs # Repository trait (domain)
├── service.rs    # Business logic
└── mod.rs        # Exports
```

## CONVENTIONS

- Repository traits defined in domain, implemented in infrastructure
- Services use dependency injection via traits
- Error types use `thiserror` derive
- DTOs for external communication

## ANTI-PATTERNS

- ❌ No infrastructure imports in domain (sqlx, tokio, etc.)
- ❌ No direct database access in services

## KEY FILES

- `macros.rs` - Shared domain macros
- `mod.rs` - Module exports and re-exports

## SEE ALSO

- Parent: `/AGENTS.md`
- Infrastructure: `/src/infrastructure/`
