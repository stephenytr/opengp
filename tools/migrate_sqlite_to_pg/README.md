# SQLite → PostgreSQL Migration Tool

> **Historical tool.** OpenGP now uses PostgreSQL exclusively. This tool is for one-time data migration of existing `opengp.db` SQLite databases from earlier installs.

This tool migrates all OpenGP data from `opengp.db` (SQLite) into PostgreSQL.

## What it does

- Discovers and migrates every non-`sqlite_%` table from SQLite
- Preserves foreign-key-safe order based on SQLite dependencies
- Converts SQLite UUID BLOBs (`16-byte`) into PostgreSQL UUID values
- Normalizes timestamp/date/time values for PostgreSQL types
- Converts BLOB payloads to PostgreSQL `BYTEA` hex format
- Creates missing PostgreSQL tables (with SQLite→PostgreSQL type mapping)
- Refuses to run if destination tables are non-empty (idempotency safeguard)
- Verifies row counts table-by-table after migration

## Prerequisites

- `sqlite3` database file available (default: `./opengp.db`)
- `psql` CLI installed and able to connect to PostgreSQL
- PostgreSQL schema already migrated (recommended via `migrations_postgres/*.sql`)

## Usage

From repository root:

```bash
python3 tools/migrate_sqlite_to_pg/migrate.py \
  --sqlite-path opengp.db \
  --pg-url "postgres://USER:PASSWORD@localhost:5432/opengp"
```

Or with `DATABASE_URL`:

```bash
export DATABASE_URL="postgres://USER:PASSWORD@localhost:5432/opengp"
python3 tools/migrate_sqlite_to_pg/migrate.py --sqlite-path opengp.db
```

## Re-running intentionally

By default, re-runs are blocked if target tables contain rows.

If you intentionally want to re-run, use:

```bash
python3 tools/migrate_sqlite_to_pg/migrate.py \
  --sqlite-path opengp.db \
  --pg-url "postgres://USER:PASSWORD@localhost:5432/opengp" \
  --truncate-first
```

`--truncate-first` truncates only the migrated target tables and then reloads data.

## Verification

The tool prints per-table row counts for SQLite and PostgreSQL and fails if any mismatch exists.
