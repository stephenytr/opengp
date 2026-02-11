# Encryption Implementation Guide

This document describes the encryption implementation in OpenGP for securing sensitive patient data.

## Overview

OpenGP uses **AES-256-GCM** (Advanced Encryption Standard with Galois/Counter Mode) for encrypting sensitive data at the application level before storing in the database. This provides:

- **Confidentiality**: Data is unreadable without the encryption key
- **Authenticity**: GCM mode includes authentication tags to detect tampering
- **Performance**: Hardware-accelerated AES on modern CPUs

## Architecture

Encryption is implemented in the infrastructure layer at `src/infrastructure/crypto/mod.rs` and provides:

```rust
pub struct EncryptionService {
    // Initialized with key from environment
}

impl EncryptionService {
    pub fn new() -> Result<Self, CryptoError>;
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, CryptoError>;
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, CryptoError>;
}
```

## Setup

### 1. Generate Encryption Key

Generate a random 32-byte (256-bit) key:

```bash
# Using openssl
openssl rand -hex 32

# Using Python
python3 -c "import secrets; print(secrets.token_hex(32))"

# Example output (DO NOT USE THIS KEY):
# a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2
```

### 2. Set Environment Variable

```bash
export ENCRYPTION_KEY="your_64_character_hex_string_here"
```

**CRITICAL**: 
- Never commit the encryption key to version control
- Store securely in environment variables or key management system
- Use different keys for development, staging, and production
- Rotate keys periodically per security policy

### 3. Initialize Service

```rust
use opengp::infrastructure::crypto::EncryptionService;

let crypto = EncryptionService::new()?;
```

## Usage in Repository Layer

As shown in ARCHITECTURE.md, encryption should be applied transparently in repository implementations:

```rust
use std::sync::Arc;
use sqlx::SqlitePool;
use opengp::infrastructure::crypto::EncryptionService;

pub struct SqlxClinicalRepository {
    pool: SqlitePool,
    crypto: Arc<EncryptionService>,
}

impl SqlxClinicalRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
    
    pub async fn create_consultation(&self, consultation: Consultation) -> Result<()> {
        // Encrypt sensitive SOAP notes before storage
        let encrypted_subjective = self.crypto.encrypt(&consultation.subjective)?;
        let encrypted_objective = self.crypto.encrypt(&consultation.objective)?;
        let encrypted_assessment = self.crypto.encrypt(&consultation.assessment)?;
        let encrypted_plan = self.crypto.encrypt(&consultation.plan)?;
        
        sqlx::query!(
            r#"
            INSERT INTO consultations (
                id, patient_id, practitioner_id, date,
                subjective, objective, assessment, plan
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            consultation.id,
            consultation.patient_id,
            consultation.practitioner_id,
            consultation.date,
            encrypted_subjective,
            encrypted_objective,
            encrypted_assessment,
            encrypted_plan
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>> {
        let row = sqlx::query!(
            "SELECT * FROM consultations WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let consultation = row.map(|r| {
            // Decrypt when reading
            Consultation {
                id: r.id,
                patient_id: r.patient_id,
                practitioner_id: r.practitioner_id,
                date: r.date,
                subjective: self.crypto.decrypt(&r.subjective)?,
                objective: self.crypto.decrypt(&r.objective)?,
                assessment: self.crypto.decrypt(&r.assessment)?,
                plan: self.crypto.decrypt(&r.plan)?,
            }
        }).transpose()?;
        
        Ok(consultation)
    }
}
```

## Fields Requiring Encryption

Per ARCHITECTURE.md and Australian healthcare regulations:

### MUST Encrypt:
- **Clinical notes**: SOAP notes (Subjective, Objective, Assessment, Plan)
- **Consultation notes**: All confidential clinical observations
- **Prescription details**: Medication notes, instructions
- **Social history**: Sensitive patient background information
- **Mental health notes**: Psychiatric observations
- **Test results**: Pathology results, imaging reports

### DO NOT Encrypt:
- Patient name, DOB (needed for search/indexing)
- Medicare number (needed for claims processing)
- Contact information (needed for communications)
- Appointment dates/times (needed for scheduling)
- Non-sensitive metadata

## Binary Format

Encrypted data is stored as binary with this structure:

```
[12 bytes: nonce][variable: ciphertext][16 bytes: authentication tag]
```

- **Nonce**: Random 12-byte value, unique per encryption
- **Ciphertext**: Encrypted plaintext
- **Tag**: GCM authentication tag for integrity verification

Total overhead: 28 bytes per encrypted field

## Database Schema

Store encrypted fields as BLOB (SQLite) or BYTEA (PostgreSQL):

```sql
CREATE TABLE consultations (
    id BLOB PRIMARY KEY,
    patient_id BLOB NOT NULL,
    date TIMESTAMP NOT NULL,
    
    -- Encrypted fields stored as BLOB
    subjective BLOB NOT NULL,    -- Encrypted
    objective BLOB NOT NULL,     -- Encrypted
    assessment BLOB NOT NULL,    -- Encrypted
    plan BLOB NOT NULL,          -- Encrypted
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Security Best Practices

### Key Management
1. **Never hard-code keys** in source code
2. **Use environment variables** for development
3. **Use KMS** (AWS KMS, Azure Key Vault) for production
4. **Rotate keys** annually or after security incidents
5. **Back up keys** securely separate from database backups

### Key Rotation
When rotating keys:
1. Keep old key available for decryption
2. Re-encrypt all data with new key
3. Update ENCRYPTION_KEY environment variable
4. Remove old key once all data migrated

### Compliance
- **RACGP Standards**: Encryption satisfies requirement for securing clinical records
- **Privacy Act 1988**: Protects personally identifiable information
- **My Health Records Act**: Meets security requirements for health data

## Error Handling

```rust
use opengp::infrastructure::crypto::CryptoError;

match crypto.encrypt("sensitive data") {
    Ok(encrypted) => {
        // Store in database
    }
    Err(CryptoError::MissingKey) => {
        // ENCRYPTION_KEY not set
    }
    Err(CryptoError::InvalidKey) => {
        // Key format invalid
    }
    Err(e) => {
        // Other encryption errors
    }
}
```

## Testing

Run encryption tests:

```bash
# Run crypto module tests
cargo test infrastructure::crypto

# Run with output
cargo test infrastructure::crypto -- --nocapture
```

## Performance

Benchmarks on typical hardware (approximate):

- Encryption: ~1-2 μs per operation
- Decryption: ~1-2 μs per operation
- Clinical note (1KB): < 5 μs
- Batch (1000 notes): < 5 ms

AES-GCM is hardware-accelerated on modern CPUs (AES-NI), providing excellent performance.

## Troubleshooting

### "Missing encryption key" error

```
Error: Missing encryption key - set ENCRYPTION_KEY environment variable
```

**Solution**: Set ENCRYPTION_KEY in your environment:
```bash
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

### "Invalid key length" error

```
Error: Invalid key length: expected 32 bytes, got 16 bytes
```

**Solution**: Key must be exactly 64 hex characters (32 bytes):
```bash
# Wrong: 32 hex chars = 16 bytes
export ENCRYPTION_KEY="a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"

# Correct: 64 hex chars = 32 bytes
export ENCRYPTION_KEY="a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
```

### "Decryption failed" error

```
Error: Decryption failed - authentication tag invalid, wrong key, or corrupted data
```

**Possible causes**:
1. Wrong encryption key (most common)
2. Data corrupted in database
3. Attempting to decrypt non-encrypted data
4. Database encoding issues (store as BLOB, not TEXT)

## References

- [NIST Special Publication 800-38D](https://csrc.nist.gov/publications/detail/sp/800-38d/final): GCM specification
- [RFC 5116](https://tools.ietf.org/html/rfc5116): Authenticated Encryption
- [OWASP Cryptographic Storage](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [Australian Cyber Security Centre (ACSC) Guidance](https://www.cyber.gov.au/)
