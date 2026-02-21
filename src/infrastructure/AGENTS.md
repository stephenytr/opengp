# Infrastructure Layer

**Path:** `src/infrastructure/`

## OVERVIEW

Database, authentication, encryption, and external service implementations.

## SUBDIRS

| Directory | Purpose |
|-----------|---------|
| `database/` | SQLx connections, repositories |
| `fixtures/` | Test data generators (fake crate) |
| `auth/` | Authentication infrastructure |
| `crypto/` | AES-GCM encryption |
| `audit/` | Audit infrastructure |

## DATABASE STRUCTURE

```
database/
├── mod.rs           # Connection pool, helpers
├── helpers.rs       # SQL helpers
├── mocks.rs         # Mock implementations for tests
├── test_utils.rs    # Test utilities
└── repositories/    # Concrete DB implementations
    ├── patient.rs
    ├── appointment.rs
    ├── clinical.rs
    ├── prescription.rs
    ├── audit.rs
    ├── user.rs
    └── practitioner.rs
```

## CONVENTIONS

- Repository implementations live in `infrastructure/database/repositories/`
- Use `sqlx` with SQLite (default) / PostgreSQL (optional)
- Mocks in `mocks.rs` for testing without DB
- Fixtures use `fake` crate for test data

## KEY PATTERNS

- `SqlitePool` / `PgPool` for connection
- `#[derive(sqlx::FromRow)]` for row mapping
- Async repository methods

## TEST PATTERNS

- **In-memory SQLite**: Use `sqlite::memory:` for test isolation
- **Mock repositories**: `src/infrastructure/database/mocks.rs` - in-memory `Arc<Mutex<Vec<T>>>`
- **Test utilities**: `src/infrastructure/database/test_utils.rs` - `create_test_pool()`, `create_test_patient()`
- **Fixture generators**: `src/infrastructure/fixtures/` - uses `fake` crate for Australian healthcare data

## SEE ALSO

- Parent: `/AGENTS.md`
- Domain: `/src/domain/`
