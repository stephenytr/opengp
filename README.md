# OpenGP

OpenGP is an open-source, terminal-first general practice management system for Australian healthcare providers.

## What this project is

- **Language**: Rust (workspace-based)
- **UI**: Ratatui + Crossterm terminal interface
- **Architecture**: Domain / Infrastructure / UI layers with trait-based boundaries
- **Database**: PostgreSQL (via SQLx)
- **Focus**: Australian clinical, billing, and compliance workflows

## Workspace layout

```text
opengp/
├── crates/
│   ├── opengp-domain/          # Domain models, services, repository traits
│   ├── opengp-infrastructure/  # SQLx repositories, crypto, auth, fixtures
│   ├── opengp-ui/              # Ratatui app, components, UI service bridges
│   ├── opengp-config/          # Configuration loading and validation
│   ├── opengp-api/             # REST API server (Axum)
│   └── opengp-cache/           # Redis caching layer
├── src/main.rs                 # TUI binary wiring and dependency injection
├── migrations/                 # SQL schema and migration scripts
├── wiki/                       # Contributor and integration guides
├── ARCHITECTURE.md             # Deep architecture documentation
└── REQUIREMENTS.md             # Product + compliance requirements
```

## Quick start

### Prerequisites

- Rust toolchain (stable)
- PostgreSQL 14+
- Redis (optional — caching only)

### 1) Database setup

```bash
# Create database
createdb opengp

# Set DATABASE_URL
export DATABASE_URL="postgres://user:password@localhost/opengp"
```

### 2) Environment configuration

```bash
cp .env.example .env
# Edit .env — set ENCRYPTION_KEY and API_DATABASE_URL at minimum
```

See [Configuration Guide](wiki/Configuration.md) for all options.

### 3) Build

```bash
cargo build --release
```

### 4) Run tests

```bash
cargo test
```

### 5) Run the TUI

```bash
cargo run --release
```

### 6) Run the API server (separate binary)

```bash
cargo run --release -p opengp-api
```

## Development workflow

1. Pick a module (`patient`, `clinical`, `appointment`, etc.)
2. Add/update domain contracts in `crates/opengp-domain`
3. Implement persistence in `crates/opengp-infrastructure`
4. Wire interaction in `crates/opengp-ui`
5. Connect dependencies in `src/main.rs`
6. Run tests and verify build

## Redis setup (optional)

Redis improves performance for patient search, appointment calendars, and permission checks. Without Redis the system falls back to direct database queries.

**Install Redis:**

- **Ubuntu/Debian**: `sudo apt-get install redis-server`
- **macOS**: `brew install redis`
- **Docker**: `docker run -d -p 6379:6379 redis:latest`

**Configure in `.env`:**

```bash
REDIS_URL=redis://localhost:6379
```

## Integration guides

- [Wiki Home](wiki/Home.md)
- [Configuration Guide](wiki/Configuration.md)
- [UI Integration Guide](wiki/Integration-UI-Guide.md)
- [Database Integration Guide](wiki/Integration-Database-Guide.md)
- [External Integration Guide](wiki/Integration-External-Guide.md)
- [End-to-End Integration Checklist](wiki/Integration-End-to-End-Checklist.md)

## Core references

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [REQUIREMENTS.md](REQUIREMENTS.md)
- [AGENTS.md](AGENTS.md)

## Project status

- Core TUI workflows (patients, appointments, clinical, billing) are active.
- Australian integrations (Medicare/PBS/AIR) are partially implemented — MBS XML importer exists; end-to-end claiming is in progress.
- REST API (`opengp-api`) is functional but not production-hardened.
- Feature-gated modules (`immunisation`, `pathology`, `prescription`, `referral`) may have incomplete UI coverage.
