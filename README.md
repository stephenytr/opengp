# OpenGP

OpenGP is an open-source, terminal-first general practice management system for Australian healthcare providers.

<img width="2880" height="1800" alt="image" src="https://github.com/user-attachments/assets/0886f666-5ab4-4417-8e42-20b49d99bedd" />

## What this project is

- **Language**: Rust (workspace-based, requires 1.86.0+)
- **UI**: Ratatui + Crossterm terminal interface
- **Architecture**: Domain / Infrastructure / UI layers with trait-based boundaries
- **Database**: PostgreSQL (via SQLx)
- **Focus**: Australian clinical, billing, and compliance workflows
- **Platform**: Linux primary; other platforms untested

## Goals

- Lightweight terminal interface — fast and keyboard-driven
- Clinical workflow simplification (minimal clicking vs Best Practice)
- Portability: core domain usable via WASM in browsers
- Single-window interface with multiple patient tabs
- SSH access for remote usage

## Workspace layout

```text
opengp/
├── Cargo.toml                 # Workspace manifest
├── Cargo.lock
├── build.rs
├── Makefile                   # Development shortcuts
├── LICENSE
├── README.md
├── AGENTS.md                  # AI agent instructions
├── mcp.json                   # MCP server configuration
├── .env.example               # Environment template
├── docker-compose.yml         # Docker development setup
├── Dockerfile.api             # API server container
├── crates/
│   ├── opengp-domain/         # Domain models, services, repository traits
│   ├── opengp-infrastructure/  # SQLx repositories, crypto, auth, fixtures
│   ├── opengp-ui/             # Ratatui app, components, UI service bridges
│   ├── opengp-config/         # Configuration loading and validation
│   ├── opengp-api/            # REST API server (Axum)
│   └── opengp-cache/          # Redis caching layer
├── src/
│   ├── main.rs                # TUI binary — wires all dependencies
│   ├── lib.rs                 # Library root — re-exports domain, infrastructure, ui, config
│   ├── conversions.rs         # Domain↔API type conversions
│   └── bin/
│       └── opengp-api.rs      # API server binary — delegates to opengp-api crate
├── migrations/                 # SQL schema and migration scripts
├── docs/                      # Architecture and requirements references
├── wiki/                      # Contributor and integration guides
├── examples/                  # Usage examples
├── scripts/                   # Development tooling
├── tests/                     # Integration tests
├── data/                      # Seed and reference data
└── logs/                      # Application logs
```

## Quick start

### Prerequisites

- Rust toolchain (stable, 1.86.0+)
- PostgreSQL 14+
- Redis (optional — caching only, degrades gracefully without it)

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

### 3) Run the API server (separate binary)

```bash
cargo run --release -p opengp-api
```

### 4) Run the TUI

```bash
cargo run --release -p opengp
```

## Docker setup (alternative)

One-command startup for development:

```bash
docker-compose up --build
```

This starts PostgreSQL (port 5432) and the API server (port 8080) with a pre-configured bridge network. PostgreSQL waits for the API to be healthy before starting.

**Note**: Docker setup uses hardcoded development credentials. Use native run for production.

## Development shortcuts

The Makefile provides common development commands:

| Command | Description |
|---------|-------------|
| `make test` | Run all tests |
| `make test-ui` | Run UI crate tests |
| `make test-domain` | Run domain crate tests |
| `make test-infra` | Run infrastructure crate tests |
| `make test-config` | Run config crate tests |
| `make build` | Build release binary |
| `make run` | Run release TUI binary |
| `make dev` | Run debug TUI binary |
| `make fmt` | Format code with rustfmt |
| `make lint` | Run clippy linter |
| `make check` | Full validation (fmt + clippy + tests) |
| `make clean` | Clean build artifacts |
| `make watch` | Watch for changes and run tests (requires cargo-watch) |
| `make help` | Display help message |

## Feature flags

Optional modules require explicit feature flags:

| Feature | Enables |
|---------|---------|
| `immunisation` | Immunisation records and AIR integration |
| `pathology` | Pathology requests and results |
| `prescription` | Prescription management |
| `referral` | Referral letter management |

Enable multiple features:

```bash
# TUI with all features
cargo run --release --features "immunisation,pathology,prescription,referral"

# API with all features
cargo run --release -p opengp-api --features "immunisation,pathology,prescription,referral"
```

Without features, the system works with core workflows: patients, appointments, clinical records, billing, and authentication.

## Development workflow

1. Pick a module (`patient`, `clinical`, `appointment`, etc.)
2. Add/update domain contracts in `crates/opengp-domain`
3. Implement persistence in `crates/opengp-infrastructure`
4. Wire interaction in `crates/opengp-ui`
5. Connect dependencies in `src/main.rs`
6. Run `make check` to validate

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

- [ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [REQUIREMENTS.md](docs/REQUIREMENTS.md)
- [AGENTS.md](AGENTS.md)

## Project status

- Active development — core TUI workflows operational
- Patients, appointments, clinical records, and billing all functional
- Australian Medicare/PBS/AIR integrations partially implemented — MBS XML importer exists; end-to-end claiming is in progress
- REST API functional but not production-hardened
- Feature-gated modules (`immunisation`, `pathology`, `prescription`, `referral`) have complete domain/infrastructure but UI coverage varies