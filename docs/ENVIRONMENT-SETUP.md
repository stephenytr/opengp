# Environment Setup Guide

## Quick Start

For local development, a pre-configured `.env` file is provided:

```bash
# File is already created - you're ready to go!
cargo run
```

## Environment Variables

### Required

| Variable | Description | Format |
|----------|-------------|--------|
| `ENCRYPTION_KEY` | AES-256 encryption key | 64 hex characters (32 bytes) |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Database connection string | `sqlite://opengp.db` |
| `DATABASE_MAX_CONNECTIONS` | Max connection pool size | `10` |
| `DATABASE_MIN_CONNECTIONS` | Min connection pool size | `2` |
| `SESSION_TIMEOUT_SECS` | Session timeout in seconds | `900` (15 minutes) |
| `LOG_LEVEL` | Logging verbosity | `info` |
| `DATA_DIR` | Data storage directory | `./data` |

## Development vs Production

### Development (Current Setup)

The provided `.env` file contains a **development-only** encryption key:

```bash
ENCRYPTION_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DATABASE_URL=sqlite://opengp.db
LOG_LEVEL=debug
```

⚠️ **WARNING**: This key is publicly visible and MUST NOT be used in production!

### Production Setup

For production environments:

1. **Generate a Secure Key**:
   ```bash
   openssl rand -hex 32
   ```

2. **Set Environment Variables**:
   ```bash
   export ENCRYPTION_KEY=your_generated_key_here
   export DATABASE_URL=postgresql://user:pass@host/dbname
   export LOG_LEVEL=warn
   ```

3. **Never commit `.env` to version control** (already in `.gitignore`)

## Database Options

### SQLite (Development)

```bash
DATABASE_URL=sqlite://opengp.db
```

Best for:
- Local development
- Single-user testing
- Demos
- Quick prototyping

### PostgreSQL (Production)

```bash
DATABASE_URL=postgresql://username:password@localhost:5432/opengp
```

Best for:
- Production deployment
- Multi-user environments
- High concurrent access
- Better performance at scale

## Encryption Key Security

### Why It Matters

The `ENCRYPTION_KEY` encrypts:
- Clinical notes (SOAP notes)
- Sensitive patient information
- Prescription details
- Any PII marked for encryption

**Loss of this key = permanent data loss** - encrypted data cannot be recovered.

### Best Practices

**Development:**
- ✅ Use the provided dev key for convenience
- ✅ Store in `.env` (gitignored)
- ✅ Share dev key with team

**Production:**
- ✅ Generate unique key per environment
- ✅ Store in secure secret manager (AWS Secrets Manager, HashiCorp Vault, etc.)
- ✅ Rotate keys periodically (with re-encryption strategy)
- ✅ Back up keys securely
- ❌ NEVER commit to git
- ❌ NEVER use dev key
- ❌ NEVER share in logs or error messages

### Generating Keys

```bash
# Linux/macOS
openssl rand -hex 32

# Output example:
# a7f8e9d0c1b2a3f4e5d6c7b8a9f0e1d2c3b4a5f6e7d8c9b0a1f2e3d4c5b6a7f8

# Verify it's 64 characters
openssl rand -hex 32 | wc -c
# Should output: 65 (64 chars + newline)
```

## Verifying Configuration

### Check Environment is Loaded

```bash
cargo run --example generate_patients
```

If successful, you'll see patient data. If it fails with "Missing encryption key", check your `.env` file.

### Test Database Connection

```bash
cargo run --example seed_database
```

Should connect to database and attempt to seed patients.

## Troubleshooting

### "Missing encryption key"

**Problem**: `ENCRYPTION_KEY` not set or `.env` not loaded

**Solutions:**
1. Ensure `.env` file exists in project root
2. Check file contains `ENCRYPTION_KEY=...`
3. Verify key is exactly 64 hex characters
4. Try setting directly: `export ENCRYPTION_KEY=0123456789abcdef...`

### "Invalid configuration"

**Problem**: Encryption key has wrong format

**Solution**: Must be exactly 64 hexadecimal characters (0-9, a-f)

### "Failed to connect to database"

**Problem**: Database URL incorrect or database doesn't exist

**Solutions:**
1. For SQLite: File will be created automatically
2. For PostgreSQL: Ensure database exists and credentials are correct
3. Check `DATABASE_URL` format is valid

### "Permission denied" on database file

**Problem**: SQLite file permissions

**Solution:**
```bash
chmod 644 opengp.db
# Or delete and let app recreate:
rm opengp.db
```

## File Structure

```
/home/stephenp/Documents/opengp/
├── .env                    # Your local config (gitignored)
├── .env.example            # Template (committed)
├── .gitignore              # Ensures .env is not committed
└── opengp.db              # SQLite database (gitignored)
```

## Security Checklist

Before deploying:

- [ ] Generated new encryption key (not using dev key)
- [ ] Stored key in secure secret manager
- [ ] Updated `DATABASE_URL` to production database
- [ ] Set `LOG_LEVEL` to `warn` or `error`
- [ ] Verified `.env` is in `.gitignore`
- [ ] Documented key backup procedure
- [ ] Tested key rotation procedure

## Additional Resources

- [Config Module Documentation](../src/config.rs)
- [Security Guidelines](../ARCHITECTURE.md#security)
- [Database Setup](../DATABASE_SETUP.md)
