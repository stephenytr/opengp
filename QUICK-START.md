# OpenGP Quick Start Guide

## Get Up and Running in 60 Seconds

### 1. Prerequisites

- Rust 1.75+ installed
- Git installed

### 2. Clone and Setup

```bash
git clone <your-repo-url> opengp
cd opengp
```

### 3. Environment Configuration

✅ **Already Done!** A development `.env` file is included with sensible defaults.

**What's configured:**
- SQLite database (file-based, no server needed)
- Development encryption key (NOT for production)
- Debug logging enabled
- All required settings

### 4. Build and Run

```bash
# Build the project
cargo build

# Run the application
cargo run
```

### 5. Generate Test Patients

```bash
# Generate and view 20 patients
cargo run --example generate_patients

# Seed database with 50 patients
cargo run --example seed_database
```

## That's It!

You're ready to develop. The application will:
- ✅ Create database file automatically
- ✅ Run migrations automatically
- ✅ Load configuration from `.env`
- ✅ Be ready to use

## Common Tasks

### Run Tests
```bash
cargo test
```

### Generate Patients
```bash
# View generated patients
cargo run --example generate_patients

# Add to database
cargo run --example seed_database
```

### Clean Build
```bash
cargo clean && cargo build
```

### Check for Errors
```bash
cargo check
```

### Lint Code
```bash
cargo clippy -- -D warnings
```

### Format Code
```bash
cargo fmt
```

## Project Structure

```
opengp/
├── .env                    # Your config (auto-loaded)
├── src/                    # Source code
│   ├── main.rs            # Entry point
│   ├── domain/            # Business logic
│   ├── infrastructure/    # Database, crypto, fixtures
│   └── components/        # UI components
├── examples/              # Patient generation examples
├── migrations/            # Database migrations
└── tests/                 # Integration tests
```

## Next Steps

### Learn More

- [Architecture](./ARCHITECTURE.md) - System design and patterns
- [Requirements](./REQUIREMENTS.md) - Feature requirements
- [Agents Guide](./AGENTS.md) - AI development guidelines
- [Patient Generation](./PATIENT-GENERATION.md) - Test data generation

### Development

- [Environment Setup](./docs/ENVIRONMENT-SETUP.md) - Detailed config guide
- [Database Setup](./DATABASE_SETUP.md) - Database configuration

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Missing Dependencies

```bash
# Update dependencies
cargo update
```

### Database Issues

```bash
# Delete and recreate database
rm opengp.db
cargo run
```

## Security Note

⚠️ **The included `.env` file contains a DEVELOPMENT KEY ONLY**

For production:
1. Generate a new key: `openssl rand -hex 32`
2. Store securely (secret manager)
3. Never commit to git
4. See [Environment Setup](./docs/ENVIRONMENT-SETUP.md) for details

## Support

- Check [ARCHITECTURE.md](./ARCHITECTURE.md) for design patterns
- Check [AGENTS.md](./AGENTS.md) for development guidelines
- Run tests to verify your setup: `cargo test`

---

**Welcome to OpenGP Development! 🎉**
