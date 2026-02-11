# Database Setup Guide

## Quick Start

The database (`opengp.db`) already exists with the schema migrated. To verify:

```bash
# Check database
sqlite3 opengp.db ".tables"

# View patient count
sqlite3 opengp.db "SELECT COUNT(*) FROM patients;"

# Run the application
cargo run --release
```

## Database Architecture

**Database**: SQLite (with PostgreSQL migration path)  
**ORM**: SQLx 0.8 with compile-time query verification  
**Migrations**: Located in `migrations/` directory

### Current Schema

Tables:
- `patients` - Patient demographics and healthcare identifiers
- `users` - System users with RBAC
- `audit_log` - Compliance audit trail
- `sessions` - User session management

## Patient Creation Flow

### 1. UI Layer (Terminal)

User navigates to Patients screen (press `1`) and presses `n` to create a new patient.

### 2. Form Component

`PatientFormComponent` collects:
- **Required**: First name, last name, date of birth, gender
- **Optional**: Medicare number/IRN, mobile phone, email

Validates input and triggers `Action::PatientFormSubmit`.

### 3. Service Layer

`PatientService::register_patient()`:
1. Checks for duplicate Medicare numbers
2. Creates `Patient` entity (domain validation)
3. Persists via repository
4. Returns created patient

### 4. Repository Layer

`SqlxPatientRepository::create()`:
- Converts domain model to SQL parameters
- Handles type mapping (NaiveDate, DateTime, UUID→BLOB)
- Executes INSERT with parameterized query
- Returns persisted entity

### 5. Database

SQLite stores patient record with:
- UUID as BLOB (16 bytes)
- Dates as TEXT (ISO 8601)
- Timestamps as TEXT (RFC 3339)
- Booleans as INTEGER (0/1)

## Type Mappings

| Rust Type | SQLite Type | Storage Format |
|-----------|-------------|----------------|
| `Uuid` | BLOB | 16 bytes |
| `NaiveDate` | DATE (TEXT) | YYYY-MM-DD |
| `DateTime<Utc>` | TIMESTAMP (TEXT) | RFC 3339 |
| `bool` | BOOLEAN (INTEGER) | 0 or 1 |
| `Option<String>` | TEXT | NULL or string |
| `Gender` (enum) | TEXT | "Male", "Female", etc. |

## Testing

### Integration Tests

```bash
# Run patient creation tests
cargo test --test patient_creation_test

# Tests verify:
# - Patient creation with full persistence
# - Duplicate Medicare number detection
# - Patient lookup by ID
```

### Manual Testing

```bash
# Start application
cargo run --release

# In the TUI:
# 1. Press '1' to go to Patients screen
# 2. Press 'n' to create new patient
# 3. Fill in form fields (Tab to navigate)
# 4. Press Enter to submit
# 5. Patient appears in list
```

## Database Maintenance

### View Data

```bash
# List all patients
sqlite3 opengp.db "SELECT first_name, last_name, date_of_birth FROM patients;"

# View recent patients
sqlite3 opengp.db "SELECT * FROM patients ORDER BY created_at DESC LIMIT 10;"

# Search by Medicare number
sqlite3 opengp.db "SELECT * FROM patients WHERE medicare_number = '1234567890';"
```

### Cleanup Test Data

```bash
# Delete test patients (use with caution!)
sqlite3 opengp.db "DELETE FROM patients WHERE first_name = 'Test';"

# Count remaining patients
sqlite3 opengp.db "SELECT COUNT(*) FROM patients;"
```

### Backup Database

```bash
# Create backup
cp opengp.db opengp.db.backup.$(date +%Y%m%d_%H%M%S)

# Restore from backup
cp opengp.db.backup.20260211_123456 opengp.db
```

## Troubleshooting

### SQLx Query Macro Errors

If you see `DATABASE_URL` errors during development:

```bash
# Option 1: Set environment variable
export DATABASE_URL="sqlite:opengp.db"
cargo build

# Option 2: Use offline mode (pre-generate query cache)
cargo sqlx prepare
cargo build
```

### Migration Issues

```bash
# Check migration status
sqlite3 opengp.db "SELECT * FROM _sqlx_migrations;"

# If migrations need to be re-run
rm opengp.db
./scripts/setup_db.sh
```

### Connection Pool Errors

If the app fails to start with database errors:

```bash
# Verify database file exists and is readable
ls -lh opengp.db
sqlite3 opengp.db ".tables"

# Check SQLite version (should be 3.x)
sqlite3 --version

# Verify schema is correct
sqlite3 opengp.db ".schema patients"
```

## Production Considerations

**Current State**: Development mode with SQLite

**For Production**:
1. Migrate to PostgreSQL (feature flag: `--features postgres`)
2. Implement connection pooling tuning
3. Add database health checks
4. Set up automated backups
5. Configure WAL mode for better concurrency
6. Add database monitoring and alerting

## Compliance Notes

Per Australian healthcare regulations:
- ✅ All patient access is auditable (audit_log table)
- ✅ Patient records use soft deletes (`is_active` flag)
- ✅ Medicare numbers have uniqueness constraints
- ⚠️ Clinical notes require encryption (implement before production)
- ⚠️ Implement backup retention policies (7+ years for clinical records)

---

**Last Updated**: 2026-02-11  
**Database Version**: 1.0 (Initial Schema)
