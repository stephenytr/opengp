# OpenGP

OpenGP is an open-source, terminal-first general practice management system for Australian healthcare providers.

## What this project is

- **Language**: Rust (workspace-based)
- **UI**: Ratatui + Crossterm terminal interface
- **Architecture**: Domain / Infrastructure / UI layers with trait-based boundaries
- **Data**: SQLite today, PostgreSQL migration path
- **Focus**: Australian clinical, billing, and compliance workflows

## Workspace layout

```text
opengp/
├── crates/
│   ├── opengp-domain/          # Domain models, services, repository traits
│   ├── opengp-infrastructure/  # SQLx repositories, crypto, auth, fixtures
│   ├── opengp-ui/              # Ratatui app, components, UI service bridges
│   ├── opengp-config/          # Configuration loading and validation
│   └── opengp-api/             # API-facing crate (in progress)
├── src/main.rs                 # Binary wiring and dependency injection
├── migrations/                 # SQL schema and migration scripts
├── wiki/                       # Contributor and integration guides
├── ARCHITECTURE.md             # Deep architecture documentation
└── REQUIREMENTS.md             # Product + compliance requirements
```

## Quick start

### 1) Prerequisites

- Rust toolchain (stable)
- SQLite available locally
- Redis (optional, for caching performance improvements)

### 1a) Redis Setup (Optional)

OpenGP supports Redis caching to improve performance. Redis is optional—if not configured, the system operates without caching.

**Install Redis:**

- **macOS**: `brew install redis`
- **Ubuntu/Debian**: `sudo apt-get install redis-server`
- **Docker**: `docker run -d -p 6379:6379 redis:latest`

**Configure in `.env`:**

```bash
# Copy .env.example to .env
cp .env.example .env

# Edit .env and set:
REDIS_URL=redis://localhost:6379
REDIS_MAX_CONNECTIONS=32
REDIS_MIN_CONNECTIONS=2
REDIS_TTL_DEFAULT_SECS=3600
```

**Start Redis:**

```bash
# Locally
redis-server

# Or with Docker
docker run -d -p 6379:6379 redis:latest
```

### 2) Build

```bash
cargo build --release
```

### 3) Run tests

```bash
cargo test
```

### 4) Run the app

```bash
cargo run --release
```

## Development workflow

1. Pick a module (for example `patient`, `clinical`, `appointment`)
2. Add/update domain contracts in `crates/opengp-domain`
3. Implement persistence/integration in `crates/opengp-infrastructure`
4. Wire interaction in `crates/opengp-ui`
5. Connect dependencies in `src/main.rs`
6. Run tests and verify build

## Integration guides (simple, practical)

The wiki now includes step-by-step integration docs:

- [Wiki Home](wiki/Home.md)
- [UI Integration Guide](wiki/Integration-UI-Guide.md)
- [Database Integration Guide](wiki/Integration-Database-Guide.md)
- [External Integration Guide](wiki/Integration-External-Guide.md)
- [End-to-End Integration Checklist](wiki/Integration-End-to-End-Checklist.md)

## Core references

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [REQUIREMENTS.md](REQUIREMENTS.md)
- [AGENTS.md](AGENTS.md)

## Project status notes

- Workspace architecture is active and used by the main binary.
- Australian integrations (Medicare/PBS/AIR/etc.) are still evolving.
- Security and audit capabilities exist, with some compliance work still pending.
